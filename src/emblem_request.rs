use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EmblemRequest {
    pub gamertag: String,
}
