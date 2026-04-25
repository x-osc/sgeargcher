use std::{collections::HashMap, hash::Hash, time::Duration};

use anyhow::{Context, anyhow};
use futures::future::join_all;

use crate::engine::scrapers::{Engine, SearchQuery, SearchResult};

mod ranking;
pub mod scrapers;

struct EngineMetadata {
    weight: f64,
}

impl Default for EngineMetadata {
    fn default() -> Self {
        Self { weight: 1.0 }
    }
}

pub struct MetaSearcher {
    engines: HashMap<String, Box<dyn Engine>>,
}

impl MetaSearcher {
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
        }
    }

    pub fn add_engine(&mut self, id: &str, engine: Box<dyn Engine>) {
        self.engines.insert(id.to_owned(), engine);
    }

    pub async fn get_all_results(
        &self,
        query: SearchQuery,
    ) -> HashMap<String, anyhow::Result<Vec<SearchResult>>> {
        let futures = self.engines.iter().map(|(id, engine)| {
            let q = query.clone();
            async move {
                let result = tokio::time::timeout(Duration::from_secs(2), engine.search(q))
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
