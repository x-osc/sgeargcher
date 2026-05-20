use async_trait::async_trait;

use crate::engine::scrapers::SearchContext;

pub mod dictionary;
pub mod headers;
pub mod ip;
pub mod lorem_ipsum;
pub mod numbat;
pub mod tldr;
pub mod user_agent;

#[async_trait]
pub trait AnswerEngine: Send + Sync {
    async fn query(&self, search: SearchContext) -> Option<String>;
}

#[derive(Debug, Clone)]
pub struct AnswerEngineMetadata {
    pub name: String,
}

impl AnswerEngineMetadata {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

impl Default for AnswerEngineMetadata {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
        }
    }
}

pub struct AnswerEngineEntry {
    pub engine: Box<dyn AnswerEngine>,
    pub metadata: AnswerEngineMetadata,
}
