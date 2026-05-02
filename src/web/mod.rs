use axum::{Router, routing::get};
use maud::{Markup, html};

use crate::engine::{
    EngineMetadata, MetaSearcher,
    scrapers::{
        brave::BraveSearch, duckduckgo::DuckDuckGoSearch, marginalia::MarginaliaSearch,
        mojeek::MojeekSearch, wiby::WibySearch,
    },
};

mod index;
mod search;

pub async fn run() {
    let app = Router::new()
        .route("/", get(index::get))
        .route("/search", get(search::get));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn html_head(title: Option<&str>) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title {
                @if let Some(title) = title {
                    { (title) }
                    " - "
                }
                "sgeargcher"
            }
        }
    }
}

fn get_config() -> MetaSearcher {
    let mut searcher = MetaSearcher::new();
    searcher.add_engine(
        Box::new(DuckDuckGoSearch),
        EngineMetadata::new("duckduckgo").weight(1.0),
    );
    searcher.add_engine(
        Box::new(MarginaliaSearch),
        EngineMetadata::new("marginalia").weight(0.6),
    );
    searcher.add_engine(
        Box::new(BraveSearch),
        EngineMetadata::new("brave").weight(0.8),
    );
    searcher.add_engine(
        Box::new(WibySearch),
        EngineMetadata::new("wiby").weight(0.15),
    );
    searcher.add_engine(
        Box::new(MojeekSearch),
        EngineMetadata::new("mojeek").weight(0.5),
    );

    searcher
}
