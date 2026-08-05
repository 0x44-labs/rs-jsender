use http::{Response, StatusCode, header::CONTENT_TYPE};
use serde::Serialize;
use serde_json::{Error, Map, Value};

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Success,
    Fail,
    Error,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Data<T> {
    Data(Option<T>),
    Value(Value),
}

#[derive(Serialize)]
pub struct JSendResponse<T> {
    status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Data<T>>,
    #[serde(skip)]
    http_status: StatusCode,
}

impl<T> JSendResponse<T>
where
    T: Serialize,
{
    pub fn success(data: Option<T>, http_status: StatusCode) -> Self {
        Self {
            status: Status::Success,
            data: Some(Data::Data(data)),
            message: None,
            code: None,
            http_status,
        }
    }

    pub fn fail<I, K, V>(data: I, http_status: StatusCode) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        let fail_data: Map<String, Value> = data
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();

        Self {
            status: Status::Fail,
            data: Some(Data::Value(Value::Object(fail_data))),
            message: None,
            code: None,
            http_status,
        }
    }

    pub fn error(
        message: impl Into<String>,
        code: Option<usize>,
        data: Option<Value>,
        http_status: StatusCode,
    ) -> Self {
        Self {
            status: Status::Error,
            message: Some(message.into()),
            code,
            data: data.map(Data::Value),
            http_status,
        }
    }

    pub fn into_response(self) -> Result<Response<Vec<u8>>, Error> {
        let body = serde_json::to_vec(&self)?;

        Ok(http::Response::builder()
            .status(self.http_status)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .expect("status code and header value are always valid"))
    }
}

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
