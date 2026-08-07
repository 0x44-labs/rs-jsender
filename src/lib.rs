//! A Pure Rust implementation of the
//! [JSend](https://github.com/omniti-labs/jsend) specification for JSON
//! responses, providing [Success], [Fail], and [Error] types for constructing
//! each kind of response body. Each response is paired with the
//! [StatusCode](http::StatusCode) it should be served with, which is applied
//! when serialising into a [Response](http::Response).
//!
//! A **JSend response** is one of three kinds:
//! - A **success** response means the call completed without error, and always
//! always carries a `data` field wrapping whatever the call returns.
//! - A **fail** response means the call was rejected due to invalid data or
//! call conditions, and carries a `data` object describing what went wrong.
//! - An **error** response means a server-side failure occurred, and always
//! carries a `message`, with optional `code` and `data` fields for a numeric
//! error code and further details respectively.
//!
//! Each type implements [IntoResponse], allowing each response type to be
//! serialised into an [http::Response] via the
//! [into_response](IntoResponse::into_response) method.
//!
//! # Example
//! ```
//! use http::StatusCode;
//! use jsender::{Error, Fail, IntoResponse, Status, Success};
//! use serde_json::json;
//!
//! fn main() {
//!     // Success response body paired with a 200 HTTP status
//!     let success = Success::new(
//!         Some(json!({
//!             "title": "Touch",
//!             "album": "Random Access Memories",
//!             "writers": ["Daft Punk", "Julian Casablancas"],
//!             "duration": 499
//!         })),
//!         StatusCode::OK,
//!     );
//!
//!     // Fail response body paired with a 422 HTTP status
//!     let fail = Fail::new(
//!         [("title", "is required"), ("duration", "must be positive")],
//!         StatusCode::UNPROCESSABLE_ENTITY,
//!     );
//!
//!     // Error response body paired with a 500 HTTP status
//!     let error = Error::new(
//!         "playback service unreachable",
//!         None,
//!         None,
//!         StatusCode::INTERNAL_SERVER_ERROR,
//!     );
//!
//!     // Each JSend response body status matches its type
//!     assert_eq!(success.status(), &Status::Success);
//!     assert_eq!(fail.status(), &Status::Fail);
//!     assert_eq!(error.status(), &Status::Error);
//!
//!     // Form an http::Response from each JSend response
//!     let result_1 = success.into_response();
//!     let result_2 = fail.into_response();
//!     let result_3 = error.into_response();
//!
//!     // All into_response results must be Ok
//!     assert!(result_1.is_ok());
//!     assert!(result_2.is_ok());
//!     assert!(result_3.is_ok());
//!     let success = result_1.unwrap();
//!     let fail = result_2.unwrap();
//!     let error = result_3.unwrap();
//!
//!     // All HTTP status codes carry through to the http::Response
//!     assert_eq!(success.status(), StatusCode::OK);
//!     assert_eq!(fail.status(), StatusCode::UNPROCESSABLE_ENTITY);
//!     assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
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
