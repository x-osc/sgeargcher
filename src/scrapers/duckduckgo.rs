use core::str;
use std::error::Error;

use scraper::{Html, Selector};
use wreq_util::{Emulation, EmulationOS, EmulationOption};

use crate::scrapers::SearchResult;

pub async fn search(query: &str) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let encoded = urlencoding::encode(query);
    let url = format!("https://html.duckduckgo.com/html/?q={}", encoded);

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

fn parse_results(html: &str) -> Vec<SearchResult> {
    let document = Html::parse_document(html);

    let result_sel = Selector::parse(".result__body").unwrap();
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

            Some(SearchResult {
                title,
                url,
                description,
            })
        })
        .collect()
}
