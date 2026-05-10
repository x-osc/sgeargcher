use core::str;
use std::sync::LazyLock;

use async_trait::async_trait;
use percent_encoding::utf8_percent_encode;
use rand::rngs::SmallRng;
use scraper::{Html, Selector};
use wreq_util::{Emulation, EmulationOS};

use crate::{
    engine::{
        client::{CLIENT_POOL, ClientProfile},
        scrapers::{Engine, EngineResponse, SearchContext},
    },
    utils::{choose_weighted, url::FRAGMENT},
};

static CLIENTS: LazyLock<Box<[(ClientProfile, f64)]>> = LazyLock::new(|| {
    Box::new([
        (
            ClientProfile::new(Emulation::Chrome137, EmulationOS::Windows),
            1.0,
        ),
        (
            ClientProfile::new(Emulation::Safari18_5, EmulationOS::MacOS),
            0.5,
        ),
    ])
});

pub struct DuckDuckGoSearch;

#[async_trait]
impl Engine for DuckDuckGoSearch {
    async fn query(&self, query: SearchContext) -> anyhow::Result<Vec<EngineResponse>> {
        let encoded = utf8_percent_encode(&query.query, FRAGMENT);
        let url = format!("https://html.duckduckgo.com/html/?q={}", encoded);

        let mut rng: SmallRng = rand::make_rng();
        let profile = choose_weighted(&CLIENTS, &mut rng)?;
        let client = CLIENT_POOL.get(&profile)?;

        let html = client.get(&url).send().await?.text().await?;

        Ok(parse_results(&html))
    }
}

fn parse_results(html: &str) -> Vec<EngineResponse> {
    let document = Html::parse_document(html);

    let result_sel = Selector::parse(".result__body:not(:has(.badge--ad))").unwrap();
    let title_sel = Selector::parse(".result__title a").unwrap();
    let url_sel = Selector::parse(".result__url").unwrap();
    let desc_sel = Selector::parse(".result__snippet").unwrap();

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
        .collect()
}
