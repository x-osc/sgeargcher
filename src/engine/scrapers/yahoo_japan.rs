use core::str;
use std::sync::LazyLock;

use anyhow::Context;
use async_trait::async_trait;
use rand::rngs::SmallRng;
use scraper::{Html, Selector};
use serde_json::Value;
use url::Url;
use wreq_util::{Emulation, EmulationOS};

use crate::{
    engine::{
        client::{CLIENT_POOL, ClientProfile},
        scrapers::{Engine, EngineResponse, SearchContext},
    },
    utils::choose_weighted,
};

static CLIENTS: LazyLock<Box<[(ClientProfile, f64)]>> = LazyLock::new(|| {
    Box::new([
        (
            ClientProfile::new(Emulation::Chrome137, EmulationOS::Windows),
            1.0,
        ),
        (
            ClientProfile::new(Emulation::Chrome135, EmulationOS::Windows),
            1.0,
        ),
        (
            ClientProfile::new(Emulation::Chrome136, EmulationOS::Windows),
            1.0,
        ),
        (
            ClientProfile::new(Emulation::Safari18_5, EmulationOS::MacOS),
            0.5,
        ),
        (
            ClientProfile::new(Emulation::Safari17_5, EmulationOS::MacOS),
            0.5,
        ),
        (
            ClientProfile::new(Emulation::Safari17_4_1, EmulationOS::MacOS),
            0.5,
        ),
        (
            ClientProfile::new(Emulation::Chrome137, EmulationOS::Android),
            2.0,
        ),
        (
            ClientProfile::new(Emulation::SafariIos18_1_1, EmulationOS::IOS),
            1.5,
        ),
    ])
});

// yahoo japan is pretty much just google results
pub struct YahooJapanSearch;

#[async_trait]
impl Engine for YahooJapanSearch {
    async fn query(&self, query: SearchContext) -> anyhow::Result<Vec<EngineResponse>> {
        // fr: referrer; edgesc is chrome(ium) identifier
        // qrw: dont do spell correction
        let url = Url::parse_with_params(
            "https://search.yahoo.co.jp/search",
            &[
                ("p", query.query.as_str()),
                ("ei", "UTF-8"),
                ("fr", "edgesc"),
                ("qrw", "0"),
            ],
        )?;

        let mut rng: SmallRng = rand::make_rng();
        let profile = choose_weighted(&CLIENTS, &mut rng)?;
        let client = CLIENT_POOL.get(&profile)?;

        let html = client.get(&url).send().await?.text().await?;

        Ok(parse_results(&html)?)
    }
}

fn parse_results(html: &str) -> anyhow::Result<Vec<EngineResponse>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("script#__NEXT_DATA__").unwrap();

    let element = document
        .select(&selector)
        .next()
        .context("__NEXT_DATA__ json tag not found")?;

    let data: Value = serde_json::from_str(&element.inner_html())?;
    let data_results = (|| -> Option<&Vec<Value>> {
        data.get("props")?
            .get("pageProps")?
            .get("initialProps")?
            .get("pageData")?
            .get("algos")?
            .as_array()
    })()
    .context("failed to get results field from json data")?;

    let results = data_results
        .iter()
        .filter_map(|r| {
            if r.get("type")? != "Algo" {
                return None;
            }

            let title_html = r.get("title")?.as_str()?;
            let title = strip_tags(title_html);

            let description_html = r.get("description")?.as_str()?;
            let description = strip_tags(description_html);

            let url = r.get("url")?.as_str()?.to_owned();

            Some(EngineResponse {
                title,
                description,
                url,
            })
        })
        .collect();

    Ok(results)
}

fn strip_tags(html: &str) -> String {
    let fragment = Html::parse_fragment(html);
    fragment.root_element().text().collect::<Vec<_>>().join("")
}
