use oauth2::{AuthorizationCode, CsrfToken};
use serde::Deserialize;

pub use oauth2::basic::BasicErrorResponse;

#[derive(Deserialize)]
pub struct CodeGrantResponse {
    pub code: AuthorizationCode,
    pub state: CsrfToken,
}

pub type CodeGrantResult = Result<CodeGrantResponse, BasicErrorResponse>;
