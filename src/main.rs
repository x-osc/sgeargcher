use scraper::{Html, Selector};
use wreq_util::{Emulation, EmulationOS, EmulationOption};

#[derive(Debug)]
struct SearchResult {
    title: String,
    url: String,
}

async fn search(query: &str) -> Result<Vec<SearchResult>, wreq::Error> {
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

    document
        .select(&result_sel)
        .filter_map(|result| {
            let title = result.select(&title_sel).next()?.text().collect::<String>();
            let url = result
                .select(&url_sel)
                .next()?
                .text()
                .collect::<String>()
                .trim()
                .to_string();

            if title.is_empty() || url.is_empty() {
                return None;
            }

            Some(SearchResult { title, url })
        })
        .collect()
}

#[tokio::main]
async fn main() {
    let query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "goats".to_string());

    match search(&query).await {
        Ok(results) if results.is_empty() => println!("no results"),
        Ok(results) => {
            for r in results.iter() {
                println!("{}", r.title);
                println!("{}\n", r.url);
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
