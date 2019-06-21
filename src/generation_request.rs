use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GenerationRequest {
    pub template: String,
    pub date: String,
    pub collections: HashMap<String, Vec<HashMap<String, String>>>,
}