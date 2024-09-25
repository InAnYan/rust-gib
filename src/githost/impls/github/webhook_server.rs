use axum::{body::Bytes, extract::State, http::header::HeaderMap, routing::post, serve, Router};
use log::{error, info};
use octocrab::models::{
    webhook_events::{
        payload::{
            IssueCommentWebhookEventAction, IssueCommentWebhookEventPayload,
            IssuesWebhookEventAction, IssuesWebhookEventPayload,
        },
        WebhookEvent, WebhookEventPayload,
    },
    Repository,
};
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tower_http::trace::TraceLayer;

use crate::{
    config::WebhookServerConfig,
    githost::events::{GitEvent, GitEventKind},
};

use super::errors::GithubError;

pub async fn listen_to_events(
    sender: Sender<GitEvent>,
    config: WebhookServerConfig,
) -> Result<(), GithubError> {
    let app = create_routes(sender);

    let listener = TcpListener::bind((config.addr, config.port))
        .await
        .map_err(GithubError::WebhookServerBindError)?;

    serve(listener, app.into_make_service())
        .await
        .map_err(GithubError::WebhookServerError)?;

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
        match WebhookEvent::try_from_header_and_body(
            match event_type.to_str() {
                Ok(r) => r,
                Err(e) => {
                    error!("Unable to convert X-GitHub-Event to string: {}", e);
                    return;
                }
            },
            &body,
        ) {
            Ok(event) => {
                handle_webhook_event(event, sender).await;
            }
            Err(err) => {
                error!("Unable to determine GitHub webhook event: {}", err);
                return;
            }
        }
    }
}

async fn handle_webhook_event(event: WebhookEvent, sender: Sender<GitEvent>) {
    if let Some(repo) = event.repository {
        match event.specific {
            WebhookEventPayload::Issues(payload) => {
                handle_issues_event(repo, *payload, sender).await
            }
            WebhookEventPayload::IssueComment(payload) => {
                handle_issue_comments_event(repo, *payload, sender).await
            }

            _ => {
                error!("Unsupported GitHub webhook event: {:?}", event.kind)
            }
        }
    } else {
        info!("Got a GitHub webhook event without repository. Currently, all supported events must have an asociated repository. Ignoring")
    }
}

async fn handle_issues_event(
    repo: Repository,
    payload: IssuesWebhookEventPayload,
    sender: Sender<GitEvent>,
) {
    match payload.action {
        IssuesWebhookEventAction::Opened => match sender
            .send(GitEvent {
                repo_id: repo.id.into(),
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
    repo: Repository,
    payload: IssueCommentWebhookEventPayload,
    sender: Sender<GitEvent>,
) {
    match payload.action {
        IssueCommentWebhookEventAction::Created => match sender
            .send(GitEvent {
                repo_id: repo.id.into(),
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
