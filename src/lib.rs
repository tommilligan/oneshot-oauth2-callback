#![deny(unsafe_code, missing_docs, clippy::all)]

//! # oneshot-auth2-callback
//!
//!

/// Standard OAuth2 response types not implemented in the `oauth2` crate.
pub mod response;
mod server;
mod ui;

pub use server::{oneshot, Error};
