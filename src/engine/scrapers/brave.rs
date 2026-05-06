use core::str;

use async_trait::async_trait;
use percent_encoding::utf8_percent_encode;
use scraper::{Html, Selector};
use wreq_util::{Emulation, EmulationOS, EmulationOption};

use crate::{
    engine::scrapers::{Engine, EngineResponse, SearchContext},
    utils::{choose_weighted, url::FRAGMENT},
};

const BROWSER: &[(Emulation, f64)] = &[
    (Emulation::Firefox117, 0.07),
    (Emulation::Firefox128, 0.09),
    (Emulation::Firefox133, 0.1),
    (Emulation::Firefox135, 0.1),
    (Emulation::Firefox136, 0.06),
    (Emulation::Firefox139, 0.01),
    (Emulation::Chrome132, 0.08),
    (Emulation::Chrome133, 0.1),
    (Emulation::Chrome134, 0.1),
    (Emulation::Chrome135, 0.1),
    (Emulation::Chrome136, 0.1),
    (Emulation::Chrome137, 0.08),
];

const OS: &[(EmulationOS, f64)] = &[
    (EmulationOS::Windows, 0.5),
    (EmulationOS::MacOS, 0.03),
    (EmulationOS::Linux, 0.05),
];

pub struct BraveSearch;

#[async_trait]
impl Engine for BraveSearch {
    async fn query(&self, query: SearchContext) -> anyhow::Result<Vec<EngineResponse>> {
        let encoded = utf8_percent_encode(&query.query, FRAGMENT);
        let url = format!("https://search.brave.com/search?q={}", encoded);

        let opt_browser_os: anyhow::Result<(&Emulation, &EmulationOS)> = {
            let mut rng = rand::rng();
            let browser = choose_weighted(BROWSER, &mut rng)?;
            let os = choose_weighted(OS, &mut rng)?;
            Ok((browser, os))
        };
        let (browser, os) = opt_browser_os?;

        let client = wreq::Client::builder()
            .emulation(
                EmulationOption::builder()
                    .emulation(*browser)
                    .emulation_os(*os)
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
