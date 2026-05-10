use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::engine::scrapers::SearchContext;

pub mod google;

#[async_trait]
pub trait CompletionEngine: Send + Sync {
    async fn query(&self, search: SearchContext) -> anyhow::Result<Vec<CompletionResponse>>;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "data")]
pub enum CompletionResponse {
    Search(String),
}

pub struct CompletionEngineEntry {
    pub engine: Box<dyn CompletionEngine>,
    pub name: String,
}
