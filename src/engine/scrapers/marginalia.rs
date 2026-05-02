use std::time::Duration;

use async_trait::async_trait;
use percent_encoding::utf8_percent_encode;
use scraper::{Html, Selector};
use wreq_util::{Emulation, EmulationOS, EmulationOption};

use crate::{
    engine::scrapers::{Engine, EngineResponse, SearchQuery},
    url::FRAGMENT,
};

const DOMAIN: &str = "https://old-search.marginalia.nu";

pub struct MarginaliaSearch;

#[async_trait]
impl Engine for MarginaliaSearch {
    async fn search(&self, query: SearchQuery) -> anyhow::Result<Vec<EngineResponse>> {
        let encoded = utf8_percent_encode(&query.query, FRAGMENT);
        let url = format!("{DOMAIN}/search?query={}", encoded);

        let client = wreq::Client::builder()
            .emulation(
                EmulationOption::builder()
                    .emulation(Emulation::Chrome133)
                    .emulation_os(EmulationOS::Windows)
                    .build(),
            )
            .build()?;

        let mut html = client.get(&url).send().await?.text().await?;
        // detect bot check
        if html.contains("sst=") && html.contains("barraged by queries from bots") {
            // TODO: proper logging
            println!("bot checked");
            // TODO: unwrap
            let new_url = extract_retry_url(&html).unwrap();
            tokio::time::sleep(Duration::from_millis(400)).await;
            html = client.get(&new_url).send().await?.text().await?;
        }

        Ok(parse_results(&html))
    }
}

pub fn extract_retry_url(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let sel = Selector::parse("a[href*='sst=']").unwrap();

    document
        .select(&sel)
        .next()
        .and_then(|el| el.attr("href"))
        .map(|href| format!("{DOMAIN}{href}"))
}

fn parse_results(html: &str) -> Vec<EngineResponse> {
    let document = Html::parse_document(html);

    let result_sel = Selector::parse(".search-result").unwrap();
    let title_sel = Selector::parse(".title").unwrap();
    let url_sel = Selector::parse(".url a").unwrap();
    let desc_sel = Selector::parse(".description").unwrap();

    let results = document
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
                .text()
                .collect::<String>()
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
        .collect();

    results
}
