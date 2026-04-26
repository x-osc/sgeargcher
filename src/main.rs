use crate::engine::{
    EngineMetadata, MetaSearcher,
    ranking::merge_and_rank_responses,
    scrapers::{
        SearchQuery, brave::BraveSearch, duckduckgo::DuckDuckGoSearch, marginalia::MarginaliaSearch,
    },
};

mod engine;
mod url;

#[tokio::main]
async fn main() {
    let query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "goats".to_string());

    let mut searcher = MetaSearcher::new();
    searcher.add_engine(
        Box::new(DuckDuckGoSearch),
        EngineMetadata::new("duckduckgo").weight(1.0),
    );
    searcher.add_engine(
        Box::new(MarginaliaSearch),
        EngineMetadata::new("marginalia").weight(0.4),
    );
    searcher.add_engine(
        Box::new(BraveSearch),
        EngineMetadata::new("brave").weight(0.7),
    );

    let responses = searcher.get_all_responses(SearchQuery { query }).await;
    let responses: Vec<_> = responses
        .into_iter()
        .filter_map(|(id, r)| match r {
            Ok(r) => Some((searcher.get_metadata(&id).unwrap().clone(), r)),
            Err(_) => None,
        })
        .collect();
    let results = merge_and_rank_responses(responses);

    println!("{:#?}", results)
}
