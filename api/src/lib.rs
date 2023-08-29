pub mod order;

#[derive(Debug, Clone, serde::Serialize)]
pub struct APIRequest<'a> {
    pub game_number: i64,
    pub code: std::borrow::Cow<'a, str>,
    pub api_version: &'static str,
}

impl<'a> APIRequest<'a> {
    pub fn v0_1<C: Into<std::borrow::Cow<'a, str>>>(game_number: i64, code: C) -> Self {
        Self {
            game_number,
            code: code.into(),
            api_version: "0.1",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct APIResponse {
    pub scanning_data: order::Report,
}
