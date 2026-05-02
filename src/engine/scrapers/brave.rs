use core::str;

use async_trait::async_trait;
use percent_encoding::utf8_percent_encode;
use scraper::{Html, Selector};
use wreq_util::{Emulation, EmulationOS, EmulationOption};

use crate::{
    engine::scrapers::{Engine, EngineResponse, SearchContext},
    utils::url::FRAGMENT,
};

pub struct BraveSearch;

#[async_trait]
impl Engine for BraveSearch {
    async fn query(&self, query: SearchContext) -> anyhow::Result<Vec<EngineResponse>> {
        let encoded = utf8_percent_encode(&query.query, FRAGMENT);
        let url = format!("https://search.brave.com/search?q={}", encoded);

        let client = wreq::Client::builder()
            .emulation(
                EmulationOption::builder()
                    .emulation(Emulation::Firefox128)
                    .emulation_os(EmulationOS::Windows)
                    .build(),
            )
            .build()?;

        let html = client.get(&url).send().await?.text().await?;

        Ok(parse_results(&html))
    }
}

fn parse_results(html: &str) -> Vec<EngineResponse> {
    let document = Html::parse_document(html);

    let result_sel = Selector::parse(".result-content").unwrap();
    let title_sel = Selector::parse(".title").unwrap();
    let url_sel = Selector::parse("a").unwrap();
    let desc_sel = Selector::parse(".content").unwrap();

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
