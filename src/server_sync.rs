use crate::response::CodeGrantResponse;
use rouille::router;
use rouille::Response;
use rouille::Server;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;

use crate::server;

/// Errors that may lead to the OAuth2 code grant not being successfully completed.
#[derive(Error, Debug)]
pub enum Error {
    /// We did not get a successful oauth response.
    #[error("No successful oauth response received")]
    Response(#[from] server::Error),

    /// There was an error with our local server listening for the response.
    #[error("Internal error in listener")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[derive(Error, Debug)]
#[error("Shared state poisoned unexpectedly")]
struct PoisonError;

struct State {
    pub code_grant_result: Option<Result<CodeGrantResponse, server::Error>>,
}

type SharedState = Arc<Mutex<State>>;

/// Listen at the given address for a single OAuth2 code grant callback.
pub fn oneshot(address: &std::net::SocketAddr, path: &str) -> Result<CodeGrantResponse, Error> {
    let state = Arc::new(Mutex::new(State {
        code_grant_result: None,
    }));

    let path = path.to_owned();
    let handler_state = state.clone();

    log::debug!("Listening for oauth callback at http://{address}{path}");
    let server = Server::new(address, move |request| {
        router!(request,
            (GET) ["/"] => rouille::Response::text(server::PENDING_TEXT),
            (GET) ["/health"] => rouille::Response::text(server::HEALTH_OK_TEXT),
            (GET) [&path] => oauth2_callback(request, &handler_state),
            _ => rouille::Response::empty_404()
        )
    })
    .map_err(Error::Internal)?;

    loop {
        server.poll_timeout(Duration::from_millis(100));
        let mut state = state
            .lock()
            .map_err(|_| Error::Internal(Box::new(PoisonError)))?;
        if let Some(code_grant_result) = state.code_grant_result.take() {
            server.join();
            return Ok(code_grant_result?);
        }
    }
}

fn oauth2_callback(request: &rouille::Request, state: &SharedState) -> rouille::Response {
    let (code_grant_result, html) = server::handle_oauth2_response(request.raw_query_string());

    match state.lock() {
        Ok(mut state) => {
            state.code_grant_result = Some(code_grant_result);
            Response::html(html)
        }
        // Only likely to happen if we're shutting down the server already, but handle nicely
        Err(_) => {
            log::error!("Failed to write to shared state");
            Response::html(server::INTERNAL_ERROR_HEADINGS.html()).with_status_code(500)
        }
    }
}
