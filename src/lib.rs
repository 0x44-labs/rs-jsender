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
use http::{Response, StatusCode, header::CONTENT_TYPE};
use serde::Serialize;
use serde_json::Value;

pub trait IntoResponse {
    fn http_status(&self) -> StatusCode;

    /// Serialises this `JSendResponse` to JSON and wraps it in an
    /// [`http::Response`].
    ///
    /// This serialises [self] to a JSON byte vector and builds an http
    /// response using the `http_status` this response was constructed with.
    /// The `Content-Type` header is set to `application/json` so callers
    /// reading the raw `http::Response` know how to interpret the body.
    ///
    /// The
    /// [JSend specification](https://github.com/omniti-labs/jsend#whither-http)
    /// advises pairing a JSend response body with whatever HTTP status code is
    /// most appropriate to it; this method carries out that pairing.
    fn into_response(self) -> Result<Response<Vec<u8>>, serde_json::Error>
    where
        Self: Serialize + Sized,
    {
        let status = self.http_status();
        let body = serde_json::to_vec(&self)?;

        Ok(http::Response::builder()
            .status(status)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .expect("status code and header value are always valid"))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Success,
    Fail,
    Error,
}

#[derive(Serialize)]
pub struct Success<T> {
    status: Status,
    data: Option<T>,
    #[serde(skip)]
    http_status: StatusCode,
}

impl<T: Serialize> Success<T> {
    /// Builds a [JSend success](https://github.com/omniti-labs/jsend#success)
    /// response.
    ///
    /// A `success` response means the call completed without error. The
    /// `status` field is always serialised as **"success"**, and `data` is
    /// always present as the wrapper for whatever the call returns. Pass
    /// [None] when there is nothing to return, such as after a delete; it
    /// serialises as `data: null` rather than omitting the key.
    ///
    /// The `http_status` parameter is the HTTP status code this response will
    /// be paired to when passed to [into_response](Self::into_response). The
    /// specification does not prescribe one, but a **`2xx`** code is typical
    /// for a success response.
    pub fn new(data: Option<T>, http_status: StatusCode) -> Self {
        Self {
            status: Status::Success,
            data,
            http_status,
        }
    }
}

#[derive(Serialize)]
pub struct Fail {
    status: Status,
    data: Value,
    #[serde(skip)]
    http_status: StatusCode,
}

impl Fail {
    /// Builds a [JSend fail](https://github.com/omniti-labs/jsend#fail)
    /// response.
    ///
    /// A `fail` response means the call was rejected due to invalid data or
    /// call conditions. The `status` field is always serialised as **"fail"**.
    /// The `data` field is built from the given key/value pairs into a JSON
    /// object describing what went wrong, typically validation errors keyed by
    /// he offending field name. An empty iterator produces `data: {}` rather
    /// than omitting the key.
    ///
    /// The `http_status` parameter is the HTTP status code this response will
    /// be paired to when passed to [into_response](Self::into_response). The
    /// specification does not prescribe one, but a **`4xx`** code is typical
    /// for a fail response.
    pub fn new<I, K, V>(data: I, http_status: StatusCode) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        let fail_data: serde_json::Map<String, Value> = data
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();

        Self {
            status: Status::Fail,
            data: Value::Object(fail_data),
            http_status,
        }
    }
}

#[derive(Serialize)]
pub struct Error {
    status: Status,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip)]
    http_status: StatusCode,
}

impl Error {
    /// Builds a [JSend error](https://github.com/omniti-labs/jsend#error)
    /// response.
    ///
    /// An `error` response means a server-side failure has occurred. The
    /// `status` field is always serialised as **"error"**, and `message` is
    /// always present as a meaningful user-readable message explaining what
    /// went wrong. The `code` and `data` fields are optional, where `code` is
    /// a numeric error code, and `data` is a free-form container for extra
    /// information about the error (e.g. conditions causing the error, stack
    /// trace). Passing `None` for either omits that key.
    ///
    /// The `http_status` parameter is the HTTP status code this response will
    /// be paired to when passed to [into_response](Self::into_response). The
    /// specification does not prescribe one, but a **`5xx`** code is typical
    /// for an error response.
    pub fn new(
        message: impl Into<String>,
        code: Option<usize>,
        data: Option<Value>,
        http_status: StatusCode,
    ) -> Self {
        Self {
            status: Status::Error,
            message: message.into(),
            code,
            data,
            http_status,
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_slice, json};

    /// Triggers the serialisation failure path for
    /// [into_response](JSendResponse::into_response).
    struct AlwaysFailsToSerialise;

    impl Serialize for AlwaysFailsToSerialise {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("forced serialisation failure"))
        }
    }

    fn to_json<T: Serialize>(response: JSendResponse<T>) -> Value {
        let http_response = response
            .into_response()
            .expect("serialisation should succeed");
        from_slice(http_response.body()).expect("body should be valid JSON")
    }

    #[test]
    fn success_includes_status_and_data() {
        let response = JSendResponse::success(
            Some(json!({
                "title": "Touch",
                "album": "Random Access Memories",
                "writers": ["Daft Punk", "Julian Casablancas"],
                "duration": 499
            })),
            StatusCode::OK,
        );
        let body = to_json(response);

        // Success response must report a "success" status
        assert_eq!(body["status"], "success");

        // Success response must carry the provided data verbatim
        assert!(body.get("data").is_some());
        assert_eq!(body["data"]["title"], "Touch");
        assert_eq!(body["data"]["album"], "Random Access Memories");

        // Success response must omit optional keys that were not provided
        assert!(body.get("message").is_none());
        assert!(body.get("code").is_none());
    }

    #[test]
    fn success_no_data_serialises_as_null() {
        let response: JSendResponse<Value> =
            JSendResponse::success(None, StatusCode::NO_CONTENT);
        let body = to_json(response);

        // Absent data must serialise as an explicit null
        assert!(body.get("data").is_some());
        assert_eq!(body["data"], Value::Null);
    }

    #[test]
    fn fail_builds_data_object_from_pairs() {
        let response: JSendResponse<Value> = JSendResponse::fail(
            [("title", "is required"), ("duration", "must be positive")],
            StatusCode::UNPROCESSABLE_ENTITY,
        );
        let body = to_json(response);

        // Fail response must report a "fail" status
        assert_eq!(body["status"], "fail");

        // Fail response must expose each provided pair under data
        assert_eq!(body["data"]["title"], "is required");

        // Fail response must omit unused optional keys
        assert!(body.get("message").is_none());
        assert!(body.get("code").is_none());
    }

    #[test]
    fn fail_accepts_non_string_values() {
        let response: JSendResponse<Value> = JSendResponse::fail(
            [("duration", json!(-5))],
            StatusCode::UNPROCESSABLE_ENTITY,
        );
        let body = to_json(response);

        // Non-string values must be preserved as their original JSON type
        assert_eq!(body["data"]["duration"], -5);
    }

    #[test]
    fn fail_no_entries_serialises_empty_object() {
        let response: JSendResponse<Value> = JSendResponse::fail(
            Vec::<(&str, &str)>::new(),
            StatusCode::BAD_REQUEST,
        );

        // An empty pair list must still produce a data object
        assert_eq!(to_json(response)["data"], json!({}));
    }

    #[test]
    fn error_requires_only_message() {
        let response: JSendResponse<Value> = JSendResponse::error(
            "playback service unreachable",
            None,
            None,
            StatusCode::INTERNAL_SERVER_ERROR,
        );
        let body = to_json(response);

        // Error response must report an "error" status
        assert_eq!(body["status"], "error");

        // Error response must carry the provided message verbatim
        assert_eq!(body["message"], "playback service unreachable");

        // Error response must omit optional keys that were not provided
        assert!(body.get("code").is_none());
        assert!(body.get("data").is_none());
    }

    #[test]
    fn error_includes_code_when_present() {
        let response: JSendResponse<Value> = JSendResponse::error(
            "rate limited",
            Some(67),
            None,
            StatusCode::TOO_MANY_REQUESTS,
        );

        // Code must be included in the error response when provided
        assert_eq!(to_json(response)["code"], 67);
    }

    #[test]
    fn error_includes_data_when_present() {
        let response: JSendResponse<Value> = JSendResponse::error(
            "upstream failure",
            None,
            Some(json!({"service": "playback_api", "trace_id": "42736"})),
            StatusCode::BAD_GATEWAY,
        );

        // Data must be included in the error response when provided
        assert_eq!(to_json(response)["data"]["trace_id"], "42736");
    }

    #[test]
    fn http_status_field_is_not_serialised() {
        let response = JSendResponse::success(
            Some(json!({"title": "End of the World Sun"})),
            StatusCode::IM_A_TEAPOT,
        );

        // The bound HTTP status must never appear in the JSON body
        assert!(to_json(response).get("http_status").is_none());
    }

    #[test]
    fn into_response_reflects_provided_status_code() {
        let response = JSendResponse::success(
            Some(json!({"title": "Outlier/EOTWS_Variation1"})),
            StatusCode::NOT_FOUND,
        );

        let http_response = response
            .into_response()
            .expect("serialisation should succeed");

        // Response must carry the HTTP status code it was constructed with
        assert_eq!(http_response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn into_response_sets_json_content_type() {
        let response = JSendResponse::success(
            Some(json!({"writers": ["65daysofstatic"]})),
            StatusCode::OK,
        );

        let http_response = response
            .into_response()
            .expect("serialisation should succeed");

        // Returned HTTP response must declare a JSON content type
        assert_eq!(
            http_response.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[test]
    fn into_response_propagates_serialisation_errors() {
        let response = JSendResponse::success(
            Some(AlwaysFailsToSerialise),
            StatusCode::OK,
        );

        // A payload that fails to serialise must surface as an error
        assert!(response.into_response().is_err());
    }
}
*/
