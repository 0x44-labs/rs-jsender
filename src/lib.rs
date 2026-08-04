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
