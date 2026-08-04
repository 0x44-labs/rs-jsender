use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct JSendResponse<T> {
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<JSendData<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum JSendData<T> {
    Data(Option<T>),
    Errors(HashMap<String, String>),
}
impl<T> JSendResponse<T>
where
    T: Serialize,
{
    pub fn success(data: Option<T>) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(JSendData::Data(data)),
            message: None,
        }
    }

    pub fn fail<I, K, V>(errors: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let errors_map: HashMap<String, String> = errors
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();

        Self {
            status: "fail".to_string(),
            data: Some(JSendData::Errors(errors_map)),
            message: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            message: Some(message.into()),
        }
    }
}
