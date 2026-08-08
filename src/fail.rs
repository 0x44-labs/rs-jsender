use http::StatusCode;
use serde::Serialize;
use serde_json::Value;

use crate::response::{IntoResponse, Status};

/// Represents a [JSend fail](https://github.com/omniti-labs/jsend#fail)
/// response body.
///
/// A `fail` response means the call was rejected due to invalid data or call
/// conditions. The `status` field is always serialised as **"fail"**, and
/// the `data` field is always present.
#[derive(Serialize)]
pub struct Fail {
    status: Status,
    data: Value,
    #[serde(skip)]
    http_status: StatusCode,
}

impl Fail {
    /// Builds a new `Fail` response.
    ///
    /// Pass the key/value pairs describing what went wrong (typically
    /// validation errors keyed by the offending field name), building `data`
    /// as a JSON object from each pair. An empty iterator produces `data: {}`,
    /// never omitting the key.
    ///
    /// The `http_status` parameter is the HTTP status code this response will
    /// be paired with when passed to [into_response](Self::into_response). The
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

    /// Returns the [Status] this response was constructed with; always
    /// [Fail](Status::Fail).
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Returns the data object built from the pairs this response was
    /// constructed with.
    pub fn data(&self) -> &Value {
        &self.data
    }
}

impl IntoResponse for Fail {
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
    fn builds_data_object_from_pairs() {
        let response = Fail::new(
            [("title", "is required"), ("duration", "must be positive")],
            StatusCode::UNPROCESSABLE_ENTITY,
        );
        let body = to_json(response);

        // Fail response must report a "fail" status
        assert_eq!(body["status"], "fail");

        // Fail response must expose each provided pair under data
        assert_eq!(body["data"]["title"], "is required");
    }

    #[test]
    fn accepts_non_string_values() {
        let response = Fail::new(
            [("duration", json!(-5))],
            StatusCode::UNPROCESSABLE_ENTITY,
        );
        let body = to_json(response);

        // Non-string values must be preserved as their original JSON type
        assert_eq!(body["data"]["duration"], -5);
    }

    #[test]
    fn no_entries_serialises_empty_object() {
        let response =
            Fail::new(Vec::<(&str, &str)>::new(), StatusCode::BAD_REQUEST);

        // An empty pair list must still produce a data object
        assert_eq!(to_json(response)["data"], json!({}));
    }

    #[test]
    fn status_is_fail() {
        let response = Fail::new(
            [("title", "is required"), ("duration", "must be positive")],
            StatusCode::UNPROCESSABLE_ENTITY,
        );

        // Getter must return Fail
        assert_eq!(response.status(), &Status::Fail);
    }

    #[test]
    fn data_returns_constructed_object() {
        let response = Fail::new(
            [("title", "is required"), ("duration", "must be positive")],
            StatusCode::UNPROCESSABLE_ENTITY,
        );

        // Getter must return the object built from the pairs
        assert_eq!(
            response.data(),
            &json!({"title": "is required", "duration": "must be positive"})
        );
    }
}
