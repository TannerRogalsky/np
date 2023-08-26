pub mod order;

#[derive(Debug, Clone, serde::Serialize)]
pub struct APIRequest {
    pub game_number: u64,
    pub code: std::borrow::Cow<'static, str>,
    pub api_version: &'static str,
}

impl APIRequest {
    pub fn v0_1<C: Into<std::borrow::Cow<'static, str>>>(game_number: u64, code: C) -> Self {
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
