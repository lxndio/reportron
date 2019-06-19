use serde::Deserialize;

#[derive(Deserialize)]
pub struct GenerationRequest {
    pub template: String,
    pub date: String,
}