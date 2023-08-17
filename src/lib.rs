#![deny(unsafe_code, missing_docs, clippy::all)]

//! # oneshot-auth2-callback
//!
//!

/// Standard OAuth2 response types not implemented in the `oauth2` crate.
pub mod response;
mod server;
#[cfg(feature = "async")]
mod server_async;
#[cfg(feature = "sync")]
mod server_sync;
mod ui;

#[cfg(feature = "async")]
pub use server_async::{oneshot, Error};
#[cfg(feature = "sync")]
/// A blocking API
pub mod blocking {
    pub use crate::server_sync::{oneshot, Error};
}
