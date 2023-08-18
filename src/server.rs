use crate::response::{BasicErrorResponse, CodeGrantResponse, CodeGrantResult};
use crate::ui::{Headings, ToHeadings};
use serde::Deserialize;
use thiserror::Error;

/// Errors specific to the Oauth2 response
#[derive(Error, Debug)]
pub enum Error {
    /// We heard a response from the identity server, stating the flow could not
    /// be completed.
    #[error("OAuth2 flow responded with a well-defined error: {}", response.error())]
    Oauth {
        /// The standard OAuth2 error response.
        response: BasicErrorResponse,
    },

    /// A response was received but could not be parsed correctly.
    #[error("OAuth2 response was malformed or invalid")]
    Invalid,

    /// The listener did not receive a response before shutdown.
    #[error("No response received")]
    Timeout,
}

const INVALID_RESPONSE_HEADINGS: Headings<'static> =
    Headings::new("Login failed.", "Received invalid OAuth2 response.");
pub(crate) const INTERNAL_ERROR_HEADINGS: Headings<'static> =
    Headings::new("Login failed.", "Internal error receiving response.");

fn parse_oauth2_response_query(query: &str) -> Result<CodeGrantResponse, Error> {
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
        serde_urlencoded::from_str(query).map_err(|_| Error::Invalid)?;
    CodeGrantResult::from(code_grant_result).map_err(|response| Error::Oauth { response })
}

pub(crate) fn handle_oauth2_response(query: &str) -> (Result<CodeGrantResponse, Error>, String) {
    let code_grant_result = parse_oauth2_response_query(query);
    let headings = match &code_grant_result {
        Ok(code_grant) => code_grant.to_headings(),
        Err(Error::Oauth { response }) => response.to_headings(),
        Err(Error::Invalid) => INVALID_RESPONSE_HEADINGS,
        Err(_) => INTERNAL_ERROR_HEADINGS,
    };

    let html = headings.html();
    (code_grant_result, html)
}

pub(crate) const PENDING_TEXT: &str = "waiting for callback";
pub(crate) const HEALTH_OK_TEXT: &str = "ok";
