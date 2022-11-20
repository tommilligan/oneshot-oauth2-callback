use crate::response::{BasicErrorResponse, CodeGrantResponse, CodeGrantResult};
use crate::ui::{Headings, ToHeadings};
use axum::{
    extract::{Extension, RawQuery},
    response::Html,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::oneshot::{channel, Sender};
use tokio::sync::Mutex;

/// Errors that may lead to the OAuth2 code grant not being successfully completed.
#[derive(Error, Debug)]
pub enum Error {
    /// We heard a response from the identity server, stating the flow could not
    /// be completed.
    #[error("OAuth2 flow responded with a well-definied error")]
    ErrorResponse {
        /// The standard OAuth2 error response.
        response: BasicErrorResponse,
    },

    /// There was an error with our local server listening for the response.
    #[error("Internal error in listener")]
    Listener(hyper::Error),

    /// A response was received but could not be parsed correctly.
    #[error("OAuth2 response was malformed or invalid")]
    InvalidResponse,

    /// The listener did not receive a response before shutdown.
    #[error("No response received")]
    NoResponse,
}

type Result<T> = std::result::Result<T, Error>;

struct State {
    pub shutdown: Option<Sender<()>>,
    pub code_grant_result: Option<Result<CodeGrantResponse>>,
}

type SharedState = Arc<Mutex<State>>;

/// Listen at the given address for a single OAuth2 code grant callback.
pub async fn oneshot(address: &std::net::SocketAddr) -> Result<CodeGrantResponse> {
    let (tx, rx) = channel::<()>();
    let state = Arc::new(Mutex::new(State {
        shutdown: Some(tx),
        code_grant_result: None,
    }));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/oauth2/callback", get(oauth2_callback))
        .layer(Extension(state.clone()));

    axum::Server::bind(address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            rx.await.ok();
        })
        .await
        .map_err(Error::Listener)?;

    let mut state = state.lock().await;
    state.code_grant_result.take().ok_or(Error::NoResponse)?
}

async fn root() -> &'static str {
    "waiting for callback"
}

async fn health() -> &'static str {
    "ok"
}

const INVALID_RESPONSE_HEADINGS: Headings<'static> =
    Headings::new("Login failed.", "Received invalid OAuth2 response.");
const INTERNAL_ERROR_HEADINGS: Headings<'static> =
    Headings::new("Login failed.", "Internal error receiving response.");

fn parse_oauth2_response_query(query: Option<String>) -> Result<CodeGrantResponse> {
    let query = query.ok_or(Error::InvalidResponse)?;

    /// Private implementation of `Result` so we can implement deserialize as an untagged enum.
    ///
    /// Once we've deserialized, translate to `Result` and use that.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum CodeGrantResultCustom {
        Ok(CodeGrantResponse),
        Err(BasicErrorResponse),
    }

    impl From<CodeGrantResultCustom> for CodeGrantResult {
        fn from(other: CodeGrantResultCustom) -> Self {
            match other {
                CodeGrantResultCustom::Ok(response) => Ok(response),
                CodeGrantResultCustom::Err(response) => Err(response),
            }
        }
    }

    let code_grant_result: CodeGrantResultCustom =
        serde_urlencoded::from_str(&query).map_err(|_| Error::InvalidResponse)?;
    CodeGrantResult::from(code_grant_result).map_err(|response| Error::ErrorResponse { response })
}

async fn oauth2_callback(
    Extension(state): Extension<SharedState>,
    RawQuery(query): RawQuery,
) -> Html<String> {
    let code_grant_result = parse_oauth2_response_query(query);
    let headings = match &code_grant_result {
        Ok(code_grant) => code_grant.to_headings(),
        Err(Error::ErrorResponse { response }) => response.to_headings(),
        Err(Error::InvalidResponse) => INVALID_RESPONSE_HEADINGS,
        Err(_) => INTERNAL_ERROR_HEADINGS,
    };

    let html = format!(
        r#"<html>
    <body>
        <div style="
            width: 100%;
            top: 50%;
            margin-top: 100px;
            text-align: center;
            font-family: sans-serif;
        ">
            <h1>{}</h1>
            <h2>{}</h2>
        </div>
    </body>
</html>"#,
        headings.title, headings.subheader
    );
    let mut state = state.lock().await;
    state.code_grant_result = Some(code_grant_result);
    if let Some(shutdown) = state.shutdown.take() {
        shutdown.send(()).expect("failed to send shutdown");
    }
    Html(html)
}
