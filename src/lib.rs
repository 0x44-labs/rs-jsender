//! A Pure Rust implementation of the
//! [JSend](https://github.com/omniti-labs/jsend) specification for JSON
//! responses, providing constructors for `success`, `fail`, and `error`
//! response bodies. Each response is paired with the [StatusCode] it should be
//! served with, which is applied when serialising into a [Response].
//!
//! A **JSend response** is one of three kinds:
//! - A **success** response means the call completed without error, and always
//! always carries a `data` field wrapping whatever the call returns.
//! - A **fail** response means the call was rejected due to invalid data or
//! call conditions, and carries a `data` object describing what went wrong.
//! - An **error** response means a server-side failure occurred, and always
//! carries a `message`, with optional `code` and `data` fields for further
//! detail.
//!
//! Any [JSendResponse] can be serialised into an [http::Response] via the
//! [into_response](JSendResponse::into_response) method.
//!
//! # Example
//! ```
//! use http::StatusCode;
//! use jsender::JSendResponse;
//! use serde_json::json;
//!
//! fn main() {
//!     let response = JSendResponse::success(
//!         Some(json!({
//!             "title": "Touch",
//!             "album": "Random Access Memories",
//!             "writers": ["Daft Punk", "Julian Casablancas"],
//!             "duration": 499
//!         })),
//!         StatusCode::OK,
//!     );
//!
//!     let http_response = response.into_response();
//!     assert!(http_response.is_ok());
//! }
//! ```
mod error;
mod fail;
mod response;
mod success;

pub use crate::error::Error;
pub use crate::fail::Fail;
pub use crate::response::{IntoResponse, Status};
pub use crate::success::Success;
