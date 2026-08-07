use http::{Response, StatusCode, header::CONTENT_TYPE};
use serde::Serialize;

/// Enables a [JSend](https://github.com/omniti-labs/jsend) response body to be
/// serialised and paired with an HTTP [StatusCode], producing a complete
/// [`http::Response`].
pub trait IntoResponse {
    /// Serialises this response to JSON and wraps it in an [`http::Response`].
    ///
    /// This serialises `self` to a JSON byte vector and builds an http
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

    /// Returns the HTTP status code this response should be served with, as
    /// given at construction time.
    fn http_status(&self) -> StatusCode;
}

/// The status discriminator of a JSend response body, indicating whether the
/// call was a [Success], [Fail], or [Error].
#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Fail,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::header::CONTENT_TYPE;
    use serde_json::{Value, from_slice, json};

    use crate::success::Success;

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
