use axum::{
    body::Bytes,
    extract::State,
    http::{header::HeaderMap, StatusCode},
    routing::post,
    serve, Router,
};
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

pub struct GithubWebhookServer {
    sender: Sender<GitEvent>,
    config: WebhookServerConfig,
}

impl GithubWebhookServer {
    pub fn new(sender: Sender<GitEvent>, config: WebhookServerConfig) -> Self {
        Self { sender, config }
    }

    pub async fn serve(&self) -> Result<(), GithubError> {
        let app = create_routes(self.sender.clone());

        let listener = TcpListener::bind((self.config.addr, self.config.port))
            .await
            .map_err(GithubError::WebhookServerBindError)?;

        serve(listener, app.into_make_service())
            .await
            .map_err(GithubError::WebhookServerError)?;

        Ok(())
    }
}

fn create_routes(sender: Sender<GitEvent>) -> Router {
    Router::new()
        .route("/", post(webhook))
        .with_state(sender)
        .layer(TraceLayer::new_for_http())
}

async fn webhook(
    State(sender): State<Sender<GitEvent>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    if let Some(event_type) = headers.get("X-GitHub-Event") {
        match WebhookEvent::try_from_header_and_body(
            match event_type.to_str() {
                Ok(r) => r,
                Err(e) => {
                    error!("Unable to convert X-GitHub-Event to string: {}", e);
                    return StatusCode::BAD_REQUEST;
                }
            },
            &body,
        ) {
            Ok(event) => handle_webhook_event(event, sender).await,
            Err(err) => {
                error!("Unable to determine GitHub webhook event: {}", err);
                StatusCode::BAD_REQUEST
            }
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

async fn handle_webhook_event(event: WebhookEvent, sender: Sender<GitEvent>) -> StatusCode {
    if let Some(repo) = event.repository {
        match event.specific {
            WebhookEventPayload::Issues(payload) => {
                handle_issues_event(repo, *payload, sender).await
            }

            WebhookEventPayload::IssueComment(payload) => {
                handle_issue_comments_event(repo, *payload, sender).await
            }

            _ => {
                error!("Unsupported GitHub webhook event: {:?}", event.kind);
                StatusCode::NOT_IMPLEMENTED
            }
        }
    } else {
        info!("Got a GitHub webhook event without repository. Currently, all supported events must have an asociated repository. Ignoring");
        StatusCode::NOT_IMPLEMENTED
    }
}

async fn handle_issues_event(
    repo: Repository,
    payload: IssuesWebhookEventPayload,
    sender: Sender<GitEvent>,
) -> StatusCode {
    match payload.action {
        IssuesWebhookEventAction::Opened => match sender
            .send(GitEvent {
                repo_id: repo.id.into(),
                issue_id: payload.issue.number.into(),
                kind: GitEventKind::NewIssue,
            })
            .await
        {
            Ok(_) => {
                info!("Received a GitEvent from webhook");
                StatusCode::OK
            }
            Err(e) => {
                error!("Unable to send GitEvent: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        },

        _ => {
            error!("Unsupported issues action: {:?}. Ignoring", payload.action);
            StatusCode::NOT_IMPLEMENTED
        }
    }
}

async fn handle_issue_comments_event(
    repo: Repository,
    payload: IssueCommentWebhookEventPayload,
    sender: Sender<GitEvent>,
) -> StatusCode {
    match payload.action {
        IssueCommentWebhookEventAction::Created => match sender
            .send(GitEvent {
                repo_id: repo.id.into(),
                issue_id: payload.issue.number.into(),
                kind: GitEventKind::NewComment(payload.comment.id.into()),
            })
            .await
        {
            Ok(_) => {
                info!("Received a GitEvent from webhook");
                StatusCode::OK
            }
            Err(e) => {
                error!("Unable to send GitEvent: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        },

        _ => {
            error!("Unsupported issues action: {:?}. Ignoring", payload.action);
            StatusCode::NOT_IMPLEMENTED
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use tokio::sync::mpsc::{channel, Receiver};

    use crate::githost::{
        events::{GitEvent, GitEventKind},
        model::{IssueId, RepoId},
    };

    use super::create_routes;

    #[tokio::test]
    async fn sends_issue_opened_event() {
        let (mut receiver, server) = make_test_server();

        let request = server
            .post("/")
            .add_header("X-GitHub-Event", "issues")
            .text(include_str!("issue_open_test.json"));

        let response = request.await;
        println!("{:?}", response);

        assert_eq!(response.status_code(), StatusCode::OK);

        let git_event = receiver.recv().await.unwrap();

        assert_eq!(
            git_event,
            GitEvent {
                repo_id: RepoId::from(987654321),
                issue_id: IssueId::from(1 as usize),
                kind: GitEventKind::NewIssue
            }
        )
    }

    fn make_test_server() -> (Receiver<GitEvent>, TestServer) {
        let (sender, receiver) = channel(42);

        let server = TestServer::new(create_routes(sender)).unwrap();

        (receiver, server)
    }
}
