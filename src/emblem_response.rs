use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EmblemResponse {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub emblem_url: String,
}