use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use log::info;
use non_empty_string::NonEmptyString;
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

use super::{errors::Result, feature_type::GitBotFeature};

pub struct GitLabelFeature {
    llm: Arc<Mutex<dyn Llm + Send>>,
}

impl GitLabelFeature {
    pub fn new(llm: Arc<Mutex<dyn Llm + Send>>) -> Self {
        GitLabelFeature { llm }
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
            GitEventKind::NewIssue => {}

            GitEventKind::NewComment(id) => {
                let comment = host
                    .lock()
                    .await
                    .get_comment(event.repo_id, event.issue_id, id)
                    .await?;

                let comment_author = host.lock().await.get_user(comment.user_id).await?;

                if &comment_author.nickname == host.lock().await.get_self_name() {
                    info!("Received message is from the bot. Ignoring");
                    return Ok(());
                }

                host.lock()
                    .await
                    .make_comment(
                        event.repo_id,
                        event.issue_id,
                        self.llm
                            .lock()
                            .await
                            .complete(
                                "you are a bot".try_into().unwrap(),
                                vec![UserMessage::from_str("say something about improving")
                                    .unwrap()
                                    .into()],
                                &CompletionParameters::default(),
                            )
                            .await?
                            .as_str()
                            .try_into()
                            .unwrap(),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "label-issues".try_into().unwrap()
    }
}
