use std::path::PathBuf;

use non_empty_string::NonEmptyString;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use tokio::fs::read_to_string;

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

use super::templates::{AuthorTemplate, IssueTemplate};

#[derive(Debug, thiserror::Error)]
pub enum ImproveFeatureError<GE, LE> {
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

pub type Result<T, GE, LE> = std::result::Result<T, ImproveFeatureError<GE, LE>>;

#[derive(Deserialize)]
pub struct ImproveFeatureConfig {
    system_message_template_path: PathBuf,
    user_message_template_path: PathBuf,
    temperature: f32,
}

pub struct ImproveFeature<G, L> {
    githost: G,
    llm: L,
    template_engine: Tera,
    temperature: f32,
}

#[derive(Serialize)]
struct ImproveFeatureTemplate {
    pub issue: IssueTemplate,
}

impl<G: GitHost, L: Llm> ImproveFeature<G, L> {
    pub async fn build(
        config: ImproveFeatureConfig,
        githost: G,
        llm: L,
    ) -> Result<Self, G::Error, L::Error> {
        Ok(ImproveFeature::build_raw(
            githost,
            llm,
            config.temperature,
            read_to_string(config.system_message_template_path.clone())
                .await
                .map_err(|e| {
                    ImproveFeatureError::TemplateReadError(
                        config.system_message_template_path.clone(),
                        e,
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ImproveFeatureError::TemplateEmptyError(config.system_message_template_path)
                })?,
            read_to_string(config.user_message_template_path.clone())
                .await
                .map_err(|e| {
                    ImproveFeatureError::TemplateReadError(
                        config.user_message_template_path.clone(),
                        e,
                    )
                })?
                .try_into()
                .map_err(|_| {
                    ImproveFeatureError::TemplateEmptyError(config.user_message_template_path)
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
            .map_err(ImproveFeatureError::TemplateAddError)?;

        template_engine
            .add_raw_template("user_message", user_message_template.as_str())
            .map_err(ImproveFeatureError::TemplateAddError)?;

        Ok(ImproveFeature {
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

                let template = ImproveFeatureTemplate {
                    issue: IssueTemplate {
                        number: event.issue_id,
                        author: AuthorTemplate {
                            nickname: author.nickname,
                        },

                        title: issue.title,
                        body: issue.body,
                    },
                };

                let mut context = Context::new();

                context.insert("issue", &template.issue);

                let system_message = self
                    .template_engine
                    .render("system_message", &context)
                    .map_err(ImproveFeatureError::TemplateRenderError)?;

                let user_message = self
                    .template_engine
                    .render("user_message", &context)
                    .map_err(ImproveFeatureError::TemplateRenderError)?;

                let ai_message = self
                    .llm
                    .complete(
                        &system_message
                            .try_into()
                            .map_err(|_| ImproveFeatureError::TemplateRenderIsEmptyError)?,
                        vec![UserMessage::from(
                            NonEmptyString::try_from(user_message)
                                .map_err(|_| ImproveFeatureError::TemplateRenderIsEmptyError)?,
                        )
                        .into()],
                        &CompletionParameters {
                            temperature: self.temperature,
                        },
                    )
                    .await
                    .map_err(ImproveFeatureError::LlmError)?;

                if !ai_message.as_str().starts_with("EMPTY") {
                    self.githost
                        .make_comment(event.repo_id, event.issue_id, ai_message.into())
                        .await?;
                }
            }

            _ => {}
        }

        Ok(())
    }
}
