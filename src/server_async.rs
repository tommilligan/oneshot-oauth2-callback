use crate::response::CodeGrantResponse;
use axum::{
    extract::{Extension, RawQuery},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::oneshot::{channel, Sender};
use tokio::sync::Mutex;

use crate::server;

/// Errors that may lead to the OAuth2 code grant not being successfully completed.
#[derive(Error, Debug)]
pub enum Error {
    /// We heard a response from the identity server, stating the flow could not
    /// be completed.
    #[error("OAuth2 reponse error")]
    Response(#[from] server::Error),

    /// There was an error with our local server listening for the response.
    #[error("Internal error in listener")]
    Listener(hyper::Error),
}

struct State {
    pub shutdown: Option<Sender<()>>,
    pub code_grant_result: Option<Result<CodeGrantResponse, server::Error>>,
}

type SharedState = Arc<Mutex<State>>;

/// Listen at the given address for a single OAuth2 code grant callback.
pub async fn oneshot(
    address: &std::net::SocketAddr,
    path: &str,
) -> Result<CodeGrantResponse, Error> {
    let (tx, rx) = channel::<()>();
    let state = Arc::new(Mutex::new(State {
        shutdown: Some(tx),
        code_grant_result: None,
    }));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route(path, get(oauth2_callback))
        .layer(Extension(state.clone()));

    axum::Server::bind(address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            rx.await.ok();
        })
        .await
        .map_err(Error::Listener)?;

    let mut state = state.lock().await;
    let code_grant_result = state
        .code_grant_result
        .take()
        .ok_or(server::Error::Timeout)?;
    Ok(code_grant_result?)
}

async fn root() -> &'static str {
    server::PENDING_TEXT
}

async fn health() -> &'static str {
    server::HEALTH_OK_TEXT
}

async fn oauth2_callback(
    Extension(state): Extension<SharedState>,
    RawQuery(query): RawQuery,
) -> Html<String> {
    let (code_grant_result, html) =
        server::handle_oauth2_response(query.as_deref().unwrap_or_default());

    let mut state = state.lock().await;
    state.code_grant_result = Some(code_grant_result);
    if let Some(shutdown) = state.shutdown.take() {
        shutdown.send(()).expect("failed to send shutdown");
    }
    Html(html)
}
