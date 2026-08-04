use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JSendStatus {
    Success,
    Fail,
    Error,
}

#[derive(Serialize)]
pub struct JSendResponse<T> {
    pub(crate) status: JSendStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) code: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<JSendData<T>>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum JSendData<T> {
    Data(Option<T>),
    Value(Value),
}

impl<T> JSendResponse<T>
where
    T: Serialize,
{
    pub fn success(data: Option<T>) -> Self {
        Self {
            status: JSendStatus::Success,
            data: Some(JSendData::Data(data)),
            message: None,
            code: None,
        }
    }

    pub fn fail<I, K, V>(data: I) -> Self
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
            status: JSendStatus::Fail,
            data: Some(JSendData::Value(Value::Object(fail_data))),
            message: None,
            code: None,
        }
    }

    pub fn error(
        message: impl Into<String>,
        code: Option<usize>,
        data: Option<Value>,
    ) -> Self {
        Self {
            status: JSendStatus::Error,
            message: Some(message.into()),
            code,
            data: data.map(JSendData::Value),
        }
    }
}
