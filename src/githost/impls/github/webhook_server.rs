use std::net::IpAddr;

use axum::{
    extract::{Json, State},
    routing::post,
    serve, Router,
};
use log::{error, info};
use octocrab::models::webhook_events::{
    payload::{IssueCommentWebhookEventAction, IssuesWebhookEventAction},
    WebhookEventPayload,
};
use tokio::{net::TcpListener, sync::mpsc::Sender};

use crate::{
    errors::{GibError, Result},
    githost::{
        event::{GitEvent, GitEventKind},
        gittypes::{CommentId, IssueId, RepoId},
    },
};

pub async fn listen_to_events(sender: Sender<GitEvent>, addr: IpAddr, port: u16) -> Result<()> {
    let app = create_routes(sender);

    let listener = TcpListener::bind((addr, port))
        .await
        .map_err(|_| GibError::WebhookServerBindError)?;

    serve(listener, app.into_make_service())
        .await
        .map_err(|_| GibError::WebhookServerError)?;

    Ok(())
}

fn create_routes(sender: Sender<GitEvent>) -> Router {
    Router::new()
        .route("/webhook", post(webhook))
        .with_state(sender)
}

async fn webhook(State(sender): State<Sender<GitEvent>>, Json(payload): Json<WebhookEventPayload>) {
    match payload {
        WebhookEventPayload::Issues(issues_event) => match issues_event.action {
            IssuesWebhookEventAction::Opened => match sender
                .send(GitEvent {
                    repo_id: RepoId::from(*issues_event.repository.id as usize),
                    issue_id: IssueId::from(*issues_event.issue.id as usize),
                    kind: GitEventKind::NewIssue,
                })
                .await
            {
                Ok(_) => info!("Received a GitEvent from webhook"),
                Err(e) => error!("Unable to send GitEvent: {:?}", e),
            },

            _ => error!(
                "Unsupported issues action: {:?}. Ignoring",
                issues_event.action
            ),
        },

        WebhookEventPayload::IssueComment(issue_comment_event) => {
            match issue_comment_event.action {
                IssueCommentWebhookEventAction::Created => match sender
                    .send(GitEvent {
                        repo_id: RepoId::from(*issue_comment_event.repository.id as usize),
                        issue_id: IssueId::from(*issue_comment_event.issue.id as usize),
                        kind: GitEventKind::NewComment(CommentId::from(
                            *issue_comment_event.comment.id as usize,
                        )),
                    })
                    .await
                {
                    Ok(_) => info!("Received a GitEvent from webhook"),
                    Err(e) => error!("Unable to send GitEvent: {:?}", e),
                },
                _ => error!(
                    "Unsupported issue comment action: {:?}. Ignoring",
                    issue_comment_event.action
                ),
            }
        }

        _ => error!("Unsupported GitHub event type: {:?}. Ignoring", payload),
    }
}
