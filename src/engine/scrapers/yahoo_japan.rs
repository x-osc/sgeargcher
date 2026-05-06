use core::str;

use anyhow::Context;
use async_trait::async_trait;
use percent_encoding::utf8_percent_encode;
use scraper::{Html, Selector};
use serde_json::Value;
use wreq_util::{Emulation, EmulationOS, EmulationOption};

use crate::{
    engine::scrapers::{Engine, EngineResponse, SearchContext},
    utils::{choose_weighted, url::FRAGMENT},
};

const BROWSER: &[(Emulation, f64)] = &[
    (Emulation::Firefox136, 0.08),
    (Emulation::Firefox139, 0.05),
    (Emulation::Chrome100, 0.1),
    (Emulation::Chrome101, 0.1),
    (Emulation::Chrome104, 0.1),
    (Emulation::Chrome105, 0.1),
    (Emulation::Chrome106, 0.1),
    (Emulation::Chrome107, 0.1),
    (Emulation::Chrome108, 0.1),
    (Emulation::Chrome110, 0.1),
    (Emulation::Chrome114, 0.1),
    (Emulation::Chrome116, 0.1),
    (Emulation::Chrome117, 0.1),
    (Emulation::Chrome118, 0.1),
    (Emulation::Chrome119, 0.08),
    (Emulation::Chrome120, 0.05),
    (Emulation::Chrome123, 0.03),
];

const BROWSER_MOBILE: &[(Emulation, f64)] = &[
    (Emulation::FirefoxAndroid135, 0.1),
    (Emulation::Chrome100, 0.1),
    (Emulation::Chrome101, 0.1),
    (Emulation::Chrome104, 0.1),
    (Emulation::Chrome105, 0.1),
    (Emulation::Chrome106, 0.1),
    (Emulation::Chrome107, 0.1),
    (Emulation::Chrome108, 0.1),
    (Emulation::Chrome110, 0.1),
    (Emulation::Chrome114, 0.1),
    (Emulation::Chrome116, 0.1),
    (Emulation::Chrome117, 0.1),
    (Emulation::Chrome118, 0.1),
    (Emulation::Chrome119, 0.08),
    (Emulation::Chrome120, 0.05),
    (Emulation::Chrome123, 0.03),
];

const OS: &[(EmulationOS, f64)] = &[(EmulationOS::Windows, 0.1), (EmulationOS::MacOS, 0.1)];
const OS_MOBILE: &[(EmulationOS, f64)] = &[(EmulationOS::Android, 0.5), (EmulationOS::IOS, 0.6)];

// yahoo japan is pretty much just google results
pub struct YahooJapanSearch;

#[async_trait]
impl Engine for YahooJapanSearch {
    async fn query(&self, query: SearchContext) -> anyhow::Result<Vec<EngineResponse>> {
        let encoded = utf8_percent_encode(&query.query, FRAGMENT);
        let url = format!(
            "https://search.yahoo.co.jp/search?p={}&ei=UTF-8&fr=edgesc",
            encoded
        );

        let opt_browser_os: anyhow::Result<(&Emulation, &EmulationOS)> = {
            let mut rng = rand::rng();
            let browser = choose_weighted(BROWSER, &mut rng)?;
            let browser_mobile = choose_weighted(BROWSER_MOBILE, &mut rng)?;
            let os = choose_weighted(OS, &mut rng)?;
            let os_mobile = choose_weighted(OS_MOBILE, &mut rng)?;
            let result = choose_weighted(
                &[((browser, os), 0.1), ((browser_mobile, os_mobile), 0.5)],
                &mut rng,
            )?
            .to_owned();
            Ok(result)
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
