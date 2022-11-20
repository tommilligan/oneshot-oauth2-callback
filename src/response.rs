use oauth2::{AuthorizationCode, CsrfToken};
use serde::Deserialize;

pub use oauth2::basic::BasicErrorResponse;

/// Represents the parameters in the redirect url, when fetched in the browser.
#[derive(Deserialize)]
pub struct CodeGrantResponse {
    /// Code used to perform token exchange.
    pub code: AuthorizationCode,
    /// State from the grant request. Must be verified by the caller.
    pub state: CsrfToken,
}

/// The authorization server may send us a well defined success or error.
pub type CodeGrantResult = Result<CodeGrantResponse, BasicErrorResponse>;
