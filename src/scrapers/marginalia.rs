use std::{error::Error, time::Duration};

use scraper::{Html, Selector};
use wreq_util::{Emulation, EmulationOS, EmulationOption};

use crate::scrapers::SearchResult;

const DOMAIN: &str = "https://old-search.marginalia.nu";

pub async fn search(query: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let encoded = urlencoding::encode(query);
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

pub fn extract_retry_url(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let sel = Selector::parse("a[href*='sst=']").unwrap();

    document
        .select(&sel)
        .next()
        .and_then(|el| el.attr("href"))
        .map(|href| format!("{DOMAIN}{href}"))
}

fn parse_results(html: &str) -> Vec<SearchResult> {
    let document = Html::parse_document(html);

    let result_sel = Selector::parse(".search-result").unwrap();
    let title_sel = Selector::parse(".title").unwrap();
    let url_sel = Selector::parse(".url a").unwrap();
    let desc_sel = Selector::parse(".description").unwrap();

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

            Some(SearchResult {
                title,
                url,
                description,
            })
        })
        .collect()
}
