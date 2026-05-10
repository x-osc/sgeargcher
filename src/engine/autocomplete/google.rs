use std::sync::LazyLock;

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use serde_json::Value;
use url::Url;
use wreq_util::{Emulation, EmulationOS};

use crate::engine::{
    autocomplete::{CompletionEngine, CompletionResponse},
    client::{CLIENT_POOL, ClientProfile},
    scrapers::SearchContext,
};

pub struct GoogleCompletion;

static CLIENT: LazyLock<ClientProfile> =
    LazyLock::new(|| ClientProfile::new(Emulation::Firefox139, EmulationOS::Linux));

#[async_trait]
impl CompletionEngine for GoogleCompletion {
    async fn query(&self, search: SearchContext) -> anyhow::Result<Vec<CompletionResponse>> {
        let url = Url::parse_with_params(
            "https://suggestqueries.google.com/complete/search",
            &[("client", "firefox"), ("hl", "US-en"), ("q", &search.query)],
        )?;

        let client = CLIENT_POOL.get(&CLIENT)?;

        let result = client.get(&url).send().await?.text().await?;

        Ok(parse_results(&result)?)
    }
}

fn parse_results(json: &str) -> anyhow::Result<Vec<CompletionResponse>> {
    let json: Vec<Value> = serde_json::from_str(json)?;

    let results: Vec<_> = json
        .iter()
        .nth(1)
        .context("response array does not have second element")?
        .as_array()
        .context("response could not be converted into array")?
        .iter()
        .filter_map(|v| Some(v.as_str()?.to_string()))
        .collect();

    if results.is_empty() {
        return Err(anyhow!("response array did not have any string elements"));
    }

    Ok(results
        .into_iter()
        .map(|r| CompletionResponse::Search(r))
        .collect())
}
