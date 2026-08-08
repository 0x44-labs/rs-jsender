use http::StatusCode;
use serde::Serialize;
use serde_json::Value;

use crate::response::{IntoResponse, Status};

/// Represents a [JSend error](https://github.com/omniti-labs/jsend#error)
/// response body.
///
/// An `error` response means a server-side failure has occurred. The `status`
/// field is always serialised as **"error"**, and the `message` field is
/// always present. The `code` and `data` fields are optional.
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
    /// Builds a new `Error` response.
    ///
    /// Pass the `message` as a meaningful user-readable message explaining
    /// what went wrong. The optional `code` is a numeric error code, and the
    /// optional `data` is a free-form container for extra information about
    /// the error (e.g. conditions causing the error, stack trace). Passing
    /// `None` for either omits that key.
    ///
    /// The `http_status` parameter is the HTTP status code this response will
    /// be paired with when passed to [into_response](Self::into_response). The
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

    /// Returns the [Status] this response was constructed with; always
    /// [Error](Status::Error).
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Returns the message this response was constructed with.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the numeric error code this response was constructed with,
    /// if any.
    pub fn code(&self) -> Option<usize> {
        self.code
    }

    /// Returns the data this response was constructed with, if any.
    pub fn data(&self) -> Option<&Value> {
        self.data.as_ref()
    }
}

impl IntoResponse for Error {
    fn http_status(&self) -> StatusCode {
        self.http_status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_slice, json};

    fn to_json<R: IntoResponse + Serialize>(response: R) -> Value {
        let http_response = response
            .into_response()
            .expect("serialisation should succeed");
        from_slice(http_response.body()).expect("body should be valid JSON")
    }

    #[test]
    fn requires_only_message() {
        let response = Error::new(
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
    fn includes_code_when_present() {
        let response = Error::new(
            "rate limited",
            Some(67),
            None,
            StatusCode::TOO_MANY_REQUESTS,
        );

        // Code must be included in the error response when provided
        assert_eq!(to_json(response)["code"], 67);
    }

    #[test]
    fn includes_data_when_present() {
        let response = Error::new(
            "upstream failure",
            None,
            Some(json!({"service": "playback_api", "trace_id": "42736"})),
            StatusCode::BAD_GATEWAY,
        );

        // Data must be included in the error response when provided
        assert_eq!(to_json(response)["data"]["trace_id"], "42736");
    }

    #[test]
    fn status_is_error() {
        let response = Error::new(
            "playback service unreachable",
            None,
            None,
            StatusCode::INTERNAL_SERVER_ERROR,
        );

        // Getter must return Error
        assert_eq!(response.status(), &Status::Error);
    }

    #[test]
    fn message_returns_constructed_value() {
        let response = Error::new(
            "playback service unreachable",
            None,
            None,
            StatusCode::INTERNAL_SERVER_ERROR,
        );

        // Getter must return the constructed error message
        assert_eq!(response.message(), "playback service unreachable");
    }

    #[test]
    fn code_returns_constructed_value() {
        let response = Error::new(
            "rate limited",
            Some(67),
            None,
            StatusCode::TOO_MANY_REQUESTS,
        );

        // Getter must return the constructed error code
        assert_eq!(response.code(), Some(67));
    }

    #[test]
    fn code_returns_none_when_absent() {
        let response = Error::new(
            "playback service unreachable",
            None,
            None,
            StatusCode::INTERNAL_SERVER_ERROR,
        );

        // Getter must return None when constructed without a code
        assert_eq!(response.code(), None);
    }

    #[test]
    fn data_returns_constructed_value() {
        let response = Error::new(
            "upstream failure",
            None,
            Some(json!({"service": "playback_api", "trace_id": "42736"})),
            StatusCode::BAD_GATEWAY,
        );

        // Getter must return the data the response was constructed with
        assert_eq!(
            response.data(),
            Some(&json!({"service": "playback_api", "trace_id": "42736"}))
        );
    }

    #[test]
    fn data_returns_none_when_absent() {
        let response = Error::new(
            "playback service unreachable",
            None,
            None,
            StatusCode::INTERNAL_SERVER_ERROR,
        );

        // Getter must return None when constructed without data
        assert_eq!(response.data(), None);
    }
}
