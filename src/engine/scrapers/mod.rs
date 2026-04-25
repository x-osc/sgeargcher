use async_trait::async_trait;

pub mod brave;
pub mod duckduckgo;
pub mod marginalia;

#[derive(Debug)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
}

#[async_trait]
pub trait Engine: Send + Sync {
    async fn search(&self, query: SearchQuery) -> anyhow::Result<Vec<SearchResult>>;
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
}
