use std::sync::Arc;

use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use serde::Serialize;
use tera::{Context, Tera};
use tokio::sync::Mutex;

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
    templates::{AuthorTemplate, IssueTemplate},
};

pub struct GitImproveFeature {
    llm: Arc<Mutex<dyn Llm + Send>>,
    template_engine: Tera,
    temperature: f32,
}

#[derive(Serialize)]
struct ImproveFeatureTemplate {
    pub issue: IssueTemplate,
}

impl GitImproveFeature {
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

        Ok(GitImproveFeature {
            llm,
            temperature,
            template_engine,
        })
    }
}

#[async_trait]
impl GitBotFeature for GitImproveFeature {
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
                    host.lock()
                        .await
                        .make_comment(event.repo_id, event.issue_id, ai_message.into())
                        .await?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "improve-issues".try_into().unwrap()
    }
}
