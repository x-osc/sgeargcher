use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::Context;
use futures::{StreamExt, future::join_all, stream::FuturesUnordered};
use maud::PreEscaped;
use tokio::time::timeout;

use crate::engine::{
    answers::{AnswerEngine, AnswerEngineEntry, AnswerEngineMetadata},
    config::SearchConfig,
    ranking::merge_and_rank_responses,
    scrapers::{Engine, EngineEntry, EngineMetadata, EngineResponse, SearchContext},
};

pub mod answers;
pub mod config;
pub mod ranking;
pub mod scrapers;

pub struct MetaSearchResult {
    pub answer: Option<AnswerResult>,
    pub results: Vec<SearchResult>,
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

#[derive(Debug)]
pub struct AnswerResult {
    pub engine: String,
    pub html: PreEscaped<String>,
}

pub struct MetaSearcher {
    engines: Vec<EngineEntry>,
    answer_engines: Vec<AnswerEngineEntry>,
}

impl MetaSearcher {
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
            answer_engines: Vec::new(),
        }
    }

    pub fn add_engine(&mut self, engine: Box<dyn Engine>, metadata: EngineMetadata) {
        self.engines.push(EngineEntry { engine, metadata });
    }

    pub fn add_answer_engine(
        &mut self,
        engine: Box<dyn AnswerEngine>,
        metadata: AnswerEngineMetadata,
    ) {
        self.answer_engines
            .push(AnswerEngineEntry { engine, metadata });
    }

    pub async fn run_search(
        &self,
        query: SearchContext,
        config: &SearchConfig,
    ) -> MetaSearchResult {
        println!(r#"searching for "{}""#, query.query);

        let (responses, answer) = tokio::join!(
            self.get_all_responses(query.clone(), config),
            self.get_answer(query)
        );

        let responses: Vec<_> = responses
            .into_iter()
            .filter_map(|(id, r)| match r {
                Ok(r) => Some((id, r)),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            })
            .collect();
        let results = merge_and_rank_responses(responses, config);

        MetaSearchResult { answer, results }
    }

    pub async fn get_all_responses(
        &self,
        query: SearchContext,
        config: &SearchConfig,
    ) -> HashMap<String, anyhow::Result<Vec<EngineResponse>>> {
        let engines: Vec<_> = self
            .engines
            .iter()
            .filter(|engine_entry| {
                let Some(settings) = config.engine_settings.get(&engine_entry.metadata.name) else {
                    return false;
                };

                settings.enabled && settings.weight > 0.
            })
            .collect();

        let futures = engines.iter().map(|engine_entry| {
            let q = query.clone();
            async move {
                let start = Instant::now();

                let result =
                    tokio::time::timeout(Duration::from_millis(5000), engine_entry.engine.query(q))
                        .await
                        .with_context(|| format!("{} timed out", engine_entry.metadata.name))
                        .flatten();

                let elapsed = start.elapsed();

                match &result {
                    Ok(responses) => {
                        println!(
                            "{} completed in {}s, ({} results)",
                            engine_entry.metadata.name,
                            elapsed.as_secs_f64(),
                            responses.len()
                        )
                    }
                    Err(err) => {
                        println!(
                            "{} failed in {}s: {:?}",
                            engine_entry.metadata.name,
                            elapsed.as_secs_f64(),
                            err
                        )
                    }
                };

                (engine_entry.metadata.name.clone(), result)
            }
        });

        let results_list = join_all(futures).await;

        results_list.into_iter().collect()
    }

    pub async fn get_answer(&self, query: SearchContext) -> Option<AnswerResult> {
        let mut futures = FuturesUnordered::new();

        for (index, engine_entry) in self.answer_engines.iter().enumerate() {
            let q = query.clone();
            let name = engine_entry.metadata.name.clone();

            futures.push(async move {
                let result = engine_entry.engine.query(q).await;

                (index, name, result)
            });
        }

        // return highest priority Some value, or else None

        let fut = async {
            let mut next = 0;
            let mut pending = HashMap::new();

            while let Some((index, name, result)) = futures.next().await {
                if index == next {
                    if let Some(html) = result {
                        return Some(AnswerResult {
                            engine: name,
                            html: PreEscaped(html),
                        });
                    }

                    next += 1;

                    while let Some((name, value)) = pending.remove(&next) {
                        if let Some(html) = value {
                            return Some(AnswerResult {
                                engine: name,
                                html: PreEscaped(html),
                            });
                        }
                        next += 1;
                    }
                } else {
                    pending.insert(index, (name, result));
                }
            }

            None
        };

        timeout(Duration::from_secs(5), fut).await.ok().flatten()
    }
}
