use std::{path::PathBuf, str::FromStr};

use non_empty_string::NonEmptyString;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use tokio::fs::read_to_string;
use tracing::error;

use crate::{
    githost::{
        events::{GitEvent, GitEventKind},
        host::GitHost,
    },
    llm::{
        llm::{CompletionParameters, Llm},
        messages::UserMessage,
    },
};

use super::templates::{AuthorTemplate, IssueTemplate, LabelTemplate};

#[derive(Debug, thiserror::Error)]
pub enum LabelFeatureError<GE, LE> {
    #[error("unable to read template file")]
    TemplateReadError(PathBuf, #[source] std::io::Error),

    #[error("template is empty")]
    TemplateEmptyError(PathBuf),

    #[error("unable to add template")]
    TemplateAddError(#[source] tera::Error),

    #[error("unable to render the template")]
    TemplateRenderError(#[source] tera::Error),

    #[error("rendered template is empty")]
    TemplateRenderIsEmptyError,

    #[error("error from LLM")]
    LlmError(#[source] LE), // NOTE: We can't use `#[from]` here, because of
    // https://github.com/dtolnay/thiserror/issues/323.
    #[error("unable to perform Git host action")]
    GitHostError(#[from] GE),

    #[error("unknown error ocurred")]
    UnknownError(#[source] Box<dyn std::error::Error + Send>),
}

pub type Result<T, GE, LE> = std::result::Result<T, LabelFeatureError<GE, LE>>;

#[derive(Deserialize)]
pub struct LabelFeatureConfig {
    system_message_template_path: PathBuf,
    user_message_template_path: PathBuf,
    temperature: f32,
}

pub struct LabelFeature<G, L> {
    githost: G,
    llm: L,
    temperature: f32,
    template_engine: Tera,
}

#[derive(Serialize)]
struct LabelFeatureTemplate {
    issue: IssueTemplate,
    labels: Vec<LabelTemplate>,
}

impl<G: GitHost, L: Llm> LabelFeature<G, L> {
    pub async fn build(
        config: LabelFeatureConfig,
        githost: G,
        llm: L,
    ) -> Result<Self, G::Error, L::Error> {
        Ok(LabelFeature::build_raw(
            githost,
            llm,
            config.temperature,
            read_to_string(config.system_message_template_path.clone())
                .await
                .map_err(|e| {
                    LabelFeatureError::TemplateReadError(
                        config.system_message_template_path.clone(),
                        e,
                    )
                })?
                .try_into()
                .map_err(|_| {
                    LabelFeatureError::TemplateEmptyError(config.system_message_template_path)
                })?,
            read_to_string(config.user_message_template_path.clone())
                .await
                .map_err(|e| {
                    LabelFeatureError::TemplateReadError(
                        config.user_message_template_path.clone(),
                        e,
                    )
                })?
                .try_into()
                .map_err(|_| {
                    LabelFeatureError::TemplateEmptyError(config.user_message_template_path)
                })?,
        )?)
    }

    fn build_raw(
        githost: G,
        llm: L,
        temperature: f32,
        system_message_template: NonEmptyString,
        user_message_template: NonEmptyString,
    ) -> Result<Self, G::Error, L::Error> {
        let mut template_engine = Tera::default();

        template_engine
            .add_raw_template("system_message", system_message_template.as_str())
            .map_err(LabelFeatureError::TemplateAddError)?;

        template_engine
            .add_raw_template("user_message", user_message_template.as_str())
            .map_err(LabelFeatureError::TemplateAddError)?;

        Ok(LabelFeature {
            githost,
            llm,
            temperature,
            template_engine,
        })
    }

    pub async fn process_event(&self, event: &GitEvent) -> Result<(), G::Error, L::Error> {
        match event.kind {
            GitEventKind::NewIssue => {
                let issue = self
                    .githost
                    .get_issue(event.repo_id, event.issue_id)
                    .await?;

                let author = self.githost.get_user(issue.author_user_id).await?;

                let template = LabelFeatureTemplate {
                    issue: IssueTemplate {
                        number: event.issue_id,
                        author: AuthorTemplate {
                            nickname: author.nickname,
                        },
                        title: issue.title,
                        body: issue.body,
                    },

                    labels: self
                        .githost
                        .get_repo_labels(event.repo_id)
                        .await?
                        .into_iter()
                        .map(|l| LabelTemplate {
                            name: l.name,
                            description: l.description,
                        })
                        .collect(),
                };

                let mut context = Context::new();

                context.insert("issue", &template.issue);
                context.insert("labels", &template.labels);

                let system_message = self
                    .template_engine
                    .render("system_message", &context)
                    .map_err(LabelFeatureError::TemplateRenderError)?;

                let user_message = self
                    .template_engine
                    .render("user_message", &context)
                    .map_err(LabelFeatureError::TemplateRenderError)?;

                let ai_message = self
                    .llm
                    .complete(
                        &system_message
                            .try_into()
                            .map_err(|_| LabelFeatureError::TemplateRenderIsEmptyError)?,
                        vec![UserMessage::from(
                            NonEmptyString::try_from(user_message)
                                .map_err(|_| LabelFeatureError::TemplateRenderIsEmptyError)?,
                        )
                        .into()],
                        &CompletionParameters {
                            temperature: self.temperature,
                        },
                    )
                    .await
                    .map_err(LabelFeatureError::LlmError)?;

                if !ai_message.as_str().starts_with("EMPTY") {
                    for label in ai_message.as_str().split(", ") {
                        match NonEmptyString::from_str(label) {
                            Err(e) => {
                                error!("AI has generated malformed result: {:?}. Skipping.", e);
                            }

                            Ok(label) => {
                                self.githost
                                    .assign_label(event.repo_id, event.issue_id, label)
                                    .await?;
                            }
                        }
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }
}
