use core::str;
use std::sync::LazyLock;

use async_trait::async_trait;
use scraper::{Html, Selector};
use url::Url;
use wreq_util::{Emulation, EmulationOS};

use crate::engine::{
    client::{CLIENT_POOL, ClientProfile},
    scrapers::{Engine, EngineResponse, SearchContext},
};

static CLIENT: LazyLock<ClientProfile> =
    LazyLock::new(|| ClientProfile::new(Emulation::Firefox139, EmulationOS::Linux));

pub struct MojeekSearch;

#[async_trait]
impl Engine for MojeekSearch {
    async fn query(&self, query: SearchContext) -> anyhow::Result<Vec<EngineResponse>> {
        let url = Url::parse_with_params(
            "https://www.mojeek.com/search",
            &[("q", query.query.as_str())],
        )?;

        let client = CLIENT_POOL.get(&CLIENT)?;

        let html = client.get(&url).send().await?.text().await?;

        Ok(parse_results(&html))
    }
}

fn parse_results(html: &str) -> Vec<EngineResponse> {
    let document = Html::parse_document(html);

    let result_sel = Selector::parse(".results-standard li").unwrap();
    let title_sel = Selector::parse("a.title").unwrap();
    let url_sel = Selector::parse("a.ob").unwrap();
    let desc_sel = Selector::parse("p.s").unwrap();

    document
        .select(&result_sel)
        .filter_map(|result| {
            let title = result
                .select(&title_sel)
                .next()?
                .text()
                .collect::<String>()
                .trim()
                .to_string();
            let url = result
                .select(&url_sel)
                .next()?
                .attr("href")
                .unwrap()
                .trim()
                .to_string();
            let description = result
                .select(&desc_sel)
                .next()?
                .text()
                .collect::<String>()
                .trim()
                .to_string();

            if title.is_empty() || url.is_empty() || description.is_empty() {
                return None;
            }

            Some(EngineResponse {
                title,
                url,
                description,
            })
        })
        .collect()
}
