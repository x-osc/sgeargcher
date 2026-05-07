use std::{collections::HashMap, net::SocketAddr, sync::LazyLock};

use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use maud::{DOCTYPE, Markup, html};
use rust_embed::RustEmbed;

use crate::engine::{
    MetaSearcher,
    answers::{
        AnswerEngineMetadata, dictionary::DictionaryAnswer, headers::HeadersAnswer, ip::IpAnswer,
        lorem_ipsum::LoremIpsumAnswer, numbat::NumbatAnswer, user_agent::UserAgentAnswer,
    },
    config::{EngineSetting, SearchConfig},
    scrapers::{
        EngineMetadata, brave::BraveSearch, duckduckgo::DuckDuckGoSearch,
        marginalia::MarginaliaSearch, mojeek::MojeekSearch, wiby::WibySearch,
        yahoo_japan::YahooJapanSearch,
    },
};

mod index;
mod search;

#[derive(RustEmbed)]
#[folder = "src/web/assets"]
struct Assets;

pub async fn run() {
    let app = Router::new()
        .route("/", get(index::get))
        .route("/search", get(search::get))
        .route("/assets/{*file}", get(static_handler))
        .fallback(get(not_found));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

pub const DEFAULT_USER_CONFIG: LazyLock<SearchConfig> = LazyLock::new(|| SearchConfig {
    engine_settings: [
        ("duckduckgo".into(), EngineSetting::new().weight(0.9)),
        ("marginalia".into(), EngineSetting::new().weight(0.7)),
        ("brave".into(), EngineSetting::new().weight(0.6)),
        ("yahoo_japan".into(), EngineSetting::new().weight(1.1)),
        ("wiby".into(), EngineSetting::new().weight(0.15)),
        ("mojeek".into(), EngineSetting::new().weight(0.4)),
    ]
    .into(),
    domain_weights: HashMap::new(),
});

pub const METASEARCHER: LazyLock<MetaSearcher> = LazyLock::new(|| {
    let mut searcher = MetaSearcher::new();
    searcher.add_engine(
        Box::new(DuckDuckGoSearch),
        EngineMetadata::new("duckduckgo"),
    );
    searcher.add_engine(
        Box::new(MarginaliaSearch),
        EngineMetadata::new("marginalia"),
    );
    searcher.add_engine(Box::new(BraveSearch), EngineMetadata::new("brave"));
    searcher.add_engine(Box::new(WibySearch), EngineMetadata::new("wiby"));
    searcher.add_engine(Box::new(MojeekSearch), EngineMetadata::new("mojeek"));
    searcher.add_engine(
        Box::new(YahooJapanSearch),
        EngineMetadata::new("yahoo_japan"),
    );

    searcher.add_answer_engine(Box::new(IpAnswer), AnswerEngineMetadata::new("ip"));
    searcher.add_answer_engine(
        Box::new(LoremIpsumAnswer),
        AnswerEngineMetadata::new("lorem ipsum"),
    );
    searcher.add_answer_engine(
        Box::new(DictionaryAnswer),
        AnswerEngineMetadata::new("wiktionary"),
    );
    searcher.add_answer_engine(Box::new(NumbatAnswer), AnswerEngineMetadata::new("numbat"));
    searcher.add_answer_engine(
        Box::new(UserAgentAnswer),
        AnswerEngineMetadata::new("user agent"),
    );
    searcher.add_answer_engine(
        Box::new(HeadersAnswer),
        AnswerEngineMetadata::new("headers"),
    );

    searcher
});

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    StaticFile(path)
}

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Assets::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}

fn html_head(title: &str) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title { (title) }
            link rel="stylesheet" href="/assets/style.css";
        }
    }
}

async fn not_found() -> Markup {
    html! {
        (DOCTYPE)
        html {
            (html_head("404 not found"))
            body {
                 h1 { "page not found" }
            }
        }

    }
}
