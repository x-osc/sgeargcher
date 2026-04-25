use crate::engine::{
    MetaSearcher,
    scrapers::{
        SearchQuery, brave::BraveSearch, duckduckgo::DuckDuckGoSearch, marginalia::MarginaliaSearch,
    },
};

mod engine;

#[tokio::main]
async fn main() {
    let query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "goats".to_string());

    let mut searcher = MetaSearcher::new();
    searcher.add_engine("duckduckgo", Box::new(DuckDuckGoSearch));
    searcher.add_engine("marginalia", Box::new(MarginaliaSearch));
    searcher.add_engine("brave", Box::new(BraveSearch));

    let results = searcher.get_all_results(SearchQuery { query }).await;
    println!("{:?}", results)
}
