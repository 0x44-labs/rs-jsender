use http::StatusCode;
use serde::Serialize;

use crate::response::{IntoResponse, Status};

/// Represents a [JSend success](https://github.com/omniti-labs/jsend#success)
/// response body, generic over the data type `T`.
///
/// A `success` response means the call completed without error. The `status`
/// field is always serialised as **"success"**, and the `data` field is always
/// present as the wrapper for whatever the call returns.
#[derive(Serialize)]
pub struct Success<T> {
    status: Status,
    data: Option<T>,
    #[serde(skip)]
    http_status: StatusCode,
}

impl<T: Serialize> Success<T> {
    /// Builds a new `Success` response.
    ///
    /// Pass the data to return in the response body as [Some], or [None] when
    /// there is nothing to return, such as after a delete. Passing
    /// `Some(data)` serialises as the wrapped value under the `data` key, and
    /// passing `None` serialises as `data: null`, never omitting the key.
    ///
    /// The `http_status` parameter is the HTTP status code this response will
    /// be paired with when passed to [into_response](Self::into_response). The
    /// specification does not prescribe one, but a **`2xx`** code is typical
    /// for a success response.
    pub fn new(data: Option<T>, http_status: StatusCode) -> Self {
        Self {
            status: Status::Success,
            data,
            http_status,
        }
    }

    /// Returns the [Status] this response was constructed with; always
    /// [Success](Status::Success).
    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Returns the data this response was constructed with, if any.
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }
}

impl<T: Serialize> IntoResponse for Success<T> {
    fn http_status(&self) -> StatusCode {
        self.http_status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::header::CONTENT_TYPE;
    use serde_json::{Value, from_slice, json};

    /// Triggers the serialisation failure path for
    /// [into_response](IntoResponse::into_response).
    struct AlwaysFailsToSerialise;

    impl Serialize for AlwaysFailsToSerialise {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("forced serialisation failure"))
        }
    }

    fn to_json<R: IntoResponse + Serialize>(response: R) -> Value {
        let http_response = response
            .into_response()
            .expect("serialisation should succeed");
        from_slice(http_response.body()).expect("body should be valid JSON")
    }

    #[test]
    fn includes_status_and_data() {
        let response = Success::new(
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
    }

    #[test]
    fn no_data_serialises_as_null() {
        let response = Success::<Value>::new(None, StatusCode::NO_CONTENT);
        let body = to_json(response);

        // Absent data must serialise as an explicit null
        assert!(body.get("data").is_some());
        assert_eq!(body["data"], Value::Null);
    }

    #[test]
    fn status_is_success() {
        let response = Success::new(
            Some(json!({"title": "Touch", "album": "Random Access Memories",})),
            StatusCode::OK,
        );

        // Getter must return Success
        assert_eq!(response.status(), &Status::Success);
    }

    #[test]
    fn data_returns_constructed_value() {
        let response = Success::new(
            Some(json!({"title": "Touch", "album": "Random Access Memories",})),
            StatusCode::OK,
        );

        // Getter must return the data the response was constructed with
        assert_eq!(
            response.data(),
            Some(
                &json!({"title": "Touch", "album": "Random Access Memories",})
            )
        );
    }

    #[test]
    fn data_returns_none_when_absent() {
        let response = Success::<Value>::new(None, StatusCode::NO_CONTENT);

        // Getter must return None when constructed without data
        assert_eq!(response.data(), None);
    }

    #[test]
    fn http_status_field_is_not_serialised() {
        let response = Success::new(
            Some(json!({"title": "End of the World Sun"})),
            StatusCode::IM_A_TEAPOT,
        );

        // The bound HTTP status must never appear in the JSON body
        assert!(to_json(response).get("http_status").is_none());
    }

    #[test]
    fn into_response_reflects_provided_status_code() {
        let response = Success::new(
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
        let response = Success::new(
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
        let response =
            Success::new(Some(AlwaysFailsToSerialise), StatusCode::OK);

        // A payload that fails to serialise must surface as an error
        assert!(response.into_response().is_err());
    }
}
