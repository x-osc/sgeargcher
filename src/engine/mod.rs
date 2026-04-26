use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use futures::future::join_all;

use crate::engine::scrapers::{Engine, EngineResponse, SearchQuery};

pub mod ranking;
pub mod scrapers;

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

struct EngineEntry {
    pub engine: Box<dyn Engine>,
    pub metadata: EngineMetadata,
}

impl Default for EngineMetadata {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            weight: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
    pub score: f64,
    pub engines: Vec<String>,
    highest_engine_weight: f64,
}

pub struct MetaSearcher {
    engines: HashMap<String, EngineEntry>,
}

impl MetaSearcher {
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
        }
    }

    pub fn add_engine(&mut self, engine: Box<dyn Engine>, metadata: EngineMetadata) {
        self.engines
            .insert(metadata.name.clone(), EngineEntry { engine, metadata });
    }

    pub fn get_metadata(&self, engine_id: &str) -> Option<&EngineMetadata> {
        self.engines.get(engine_id).map(|e| &e.metadata)
    }

    pub async fn get_all_responses(
        &self,
        query: SearchQuery,
    ) -> HashMap<String, anyhow::Result<Vec<EngineResponse>>> {
        let futures = self.engines.iter().map(|(id, engine_entry)| {
            let q = query.clone();
            async move {
                let result =
                    tokio::time::timeout(Duration::from_secs(5), engine_entry.engine.search(q))
                        .await
                        .context("Search timed out")
                        .and_then(|res| res.map_err(anyhow::Error::from));

                (id.clone(), result)
            }
        });

        let results_list = join_all(futures).await;

        results_list.into_iter().collect()
    }
}
