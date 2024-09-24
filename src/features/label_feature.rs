use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use serde::Serialize;
use tera::{Context, Tera};
use tokio::sync::Mutex;
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

use super::{
    errors::{GitFeatureError, Result},
    feature_type::GitBotFeature,
    templates::{AuthorTemplate, IssueTemplate, LabelTemplate},
};

pub struct GitLabelFeature {
    llm: Arc<Mutex<dyn Llm + Send>>,
    temperature: f32,
    template_engine: Tera,
}

#[derive(Serialize)]
struct LabelFeatureTemplate {
    issue: IssueTemplate,
    labels: Vec<LabelTemplate>,
}

impl GitLabelFeature {
    pub fn build(
        llm: Arc<Mutex<dyn Llm + Send>>,
        temperature: f32,
        system_message_template: NonEmptyString,
        user_message_template: NonEmptyString,
    ) -> Result<Self> {
        let mut template_engine = Tera::default();

        template_engine
            .add_raw_template("system_message", system_message_template.as_str())
            .map_err(|_| GitFeatureError::TemplateAddError)?;

        template_engine
            .add_raw_template("user_message", user_message_template.as_str())
            .map_err(|_| GitFeatureError::TemplateAddError)?;

        Ok(GitLabelFeature {
            llm,
            temperature,
            template_engine,
        })
    }
}

#[async_trait]
impl GitBotFeature for GitLabelFeature {
    async fn process_event(
        &self,
        event: &GitEvent,
        host: Arc<Mutex<dyn GitHost + Send + Sync>>,
    ) -> Result<()> {
        match event.kind {
            GitEventKind::NewIssue => {
                let issue = host
                    .lock()
                    .await
                    .get_issue(event.repo_id, event.issue_id)
                    .await?;

                let author = host.lock().await.get_user(issue.author_user_id).await?;

                let template = LabelFeatureTemplate {
                    issue: IssueTemplate {
                        number: event.issue_id,
                        author: AuthorTemplate {
                            nickname: author.nickname,
                        },
                        title: issue.title,
                        body: issue.body,
                    },

                    labels: host
                        .lock()
                        .await
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
                    .map_err(|_| GitFeatureError::TemplateRenderError)?;

                let user_message = self
                    .template_engine
                    .render("user_message", &context)
                    .map_err(|_| GitFeatureError::TemplateRenderError)?;

                let ai_message = self
                    .llm
                    .lock()
                    .await
                    .complete(
                        &system_message
                            .try_into()
                            .map_err(|_| GitFeatureError::TemplateRenderIsEmptyError)?,
                        vec![UserMessage::from(
                            NonEmptyString::try_from(user_message)
                                .map_err(|_| GitFeatureError::TemplateRenderIsEmptyError)?,
                        )
                        .into()],
                        &CompletionParameters {
                            temperature: self.temperature,
                        },
                    )
                    .await?;

                if !ai_message.as_str().starts_with("EMPTY") {
                    for label in ai_message.as_str().split(", ") {
                        match NonEmptyString::from_str(label) {
                            Err(e) => {
                                error!("AI has generated malformed result: {:?}. Skipping.", e);
                            }

                            Ok(label) => {
                                host.lock()
                                    .await
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

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "label-issues".try_into().unwrap()
    }
}
