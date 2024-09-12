use std::net::IpAddr;

use axum::{body::Bytes, extract::State, http::header::HeaderMap, routing::post, serve, Router};
use log::{error, info};
use octocrab::models::webhook_events::payload::{
    IssueCommentWebhookEventAction, IssueCommentWebhookEventPayload, IssuesWebhookEventAction,
    IssuesWebhookEventPayload,
};
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tower_http::trace::TraceLayer;

use crate::{
    errors::{GibError, Result},
    githost::event::{GitEvent, GitEventKind},
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
        .layer(TraceLayer::new_for_http())
}

async fn webhook(State(sender): State<Sender<GitEvent>>, headers: HeaderMap, body: Bytes) {
    if let Some(event_type) = headers.get("X-GitHub-Event") {
        match event_type.as_ref() {
            b"issues" => {
                handle_issues_event(
                    sender,
                    match serde_json::from_slice(&*body) {
                        Ok(r) => r,
                        Err(e) => {
                            error!("Unable to deserialize GitHub event: {}", e);
                            return;
                        }
                    },
                )
                .await
            }

            b"issue_comment" => {
                handle_issue_comments_event(
                    sender,
                    match serde_json::from_slice(&*body) {
                        Ok(r) => r,
                        Err(e) => {
                            error!("Unable to deserialize GitHub event: {}", e);
                            return;
                        }
                    },
                )
                .await
            }
            _ => error!("Unsupported GitHub event type: {:?}", event_type.to_str()),
        }
    }
}

async fn handle_issues_event(sender: Sender<GitEvent>, payload: IssuesWebhookEventPayload) {
    match payload.action {
        IssuesWebhookEventAction::Opened => match sender
            .send(GitEvent {
                repo_id: payload.repository.id.into(),
                issue_id: payload.issue.number.into(),
                kind: GitEventKind::NewIssue,
            })
            .await
        {
            Ok(_) => info!("Received a GitEvent from webhook"),
            Err(e) => error!("Unable to send GitEvent: {:?}", e),
        },

        _ => error!("Unsupported issues action: {:?}. Ignoring", payload.action),
    }
}

async fn handle_issue_comments_event(
    sender: Sender<GitEvent>,
    payload: IssueCommentWebhookEventPayload,
) {
    match payload.action {
        IssueCommentWebhookEventAction::Created => match sender
            .send(GitEvent {
                repo_id: payload.repository.id.into(),
                issue_id: payload.issue.number.into(),
                kind: GitEventKind::NewComment(payload.comment.id.into()),
            })
            .await
        {
            Ok(_) => info!("Received a GitEvent from webhook"),
            Err(e) => error!("Unable to send GitEvent: {:?}", e),
        },
        _ => error!(
            "Unsupported issue comment action: {:?}. Ignoring",
            payload.action
        ),
    }
}
