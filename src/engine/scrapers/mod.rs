use async_trait::async_trait;

pub mod brave;
pub mod duckduckgo;
pub mod marginalia;
pub mod mojeek;
pub mod wiby;

#[derive(Debug)]
pub struct EngineResponse {
    pub title: String,
    pub url: String,
    pub description: String,
}

#[async_trait]
pub trait Engine: Send + Sync {
    async fn query(&self, search: SearchContext) -> anyhow::Result<Vec<EngineResponse>>;
}

#[derive(Debug, Clone)]
pub struct SearchContext {
    pub query: String,
    pub ip: String,
}

#[derive(Debug, Clone)]
pub struct EngineMetadata {
    pub name: String,
    pub weight: f64,
}

impl EngineMetadata {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

impl Default for EngineMetadata {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            weight: 1.0,
        }
    }
}

pub struct EngineEntry {
    pub engine: Box<dyn Engine>,
    pub metadata: EngineMetadata,
}
