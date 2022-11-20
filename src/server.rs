use axum::{
    extract::{Extension, RawQuery},
    response::Html,
    routing::get,
    Router,
};
use tokio::sync::oneshot::{channel, Sender};
use crate::response::{CodeGrantResult, CodeGrantResponse, BasicErrorResponse};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Deserialize;
use crate::ui::{Headings, ToHeadings};

struct State {
    pub shutdown: Option<Sender<()>>,
    pub code_grant_result: Option<CodeGrantResult>,
}

type SharedState = Arc<Mutex<State>>;

pub async fn oneshot(address: &std::net::SocketAddr) -> CodeGrantResult {
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
        .unwrap();

    let mut state = state.lock().await;
    state
        .code_grant_result
        .take()
        .expect("code_grant_result set")
}

async fn root() -> &'static str {
    "waiting for callback"
}

async fn health() -> &'static str {
    "ok"
}

const INVALID_RESPONSE_HEADINGS: Headings<'static> = Headings::new("Login failed.", "Received invalid OAuth2 response.");

async fn oauth2_callback(
    Extension(state): Extension<SharedState>,
    RawQuery(query): RawQuery,
) -> Html<String> {
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

    let code_grant_result = if let Some(query) = query {
        let code_grant_result: CodeGrantResultCustom = serde_urlencoded::from_str(&query).unwrap();
        let code_grant_result: CodeGrantResult = code_grant_result.into();
        Some(code_grant_result)
    } else {
        None
    };
    let headings = if let Some(ref code_grant_result) = code_grant_result {
        code_grant_result.to_headings()
    } else {
        INVALID_RESPONSE_HEADINGS
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
    state.code_grant_result = code_grant_result;
    if let Some(shutdown) = state.shutdown.take() {
        shutdown.send(()).expect("failed to send shutdown");
    }
    Html(html)
}
