use std::sync::LazyLock;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web};
use maud::{DOCTYPE, Markup, html};
use rust_embed::RustEmbed;

use crate::{
    config::MetaSearchConfig,
    engine::{
        MetaSearcher,
        answers::{
            AnswerEngineMetadata, dictionary::DictionaryAnswer, headers::HeadersAnswer,
            ip::IpAnswer, lorem_ipsum::LoremIpsumAnswer, numbat::NumbatAnswer,
            user_agent::UserAgentAnswer,
        },
        scrapers::{
            EngineMetadata, brave::BraveSearch, duckduckgo::DuckDuckGoSearch,
            marginalia::MarginaliaSearch, mojeek::MojeekSearch, wiby::WibySearch,
            yahoo_japan::YahooJapanSearch,
        },
    },
};

mod config;
mod index;
mod search;

#[derive(RustEmbed)]
#[folder = "src/web/assets"]
struct Assets;

pub async fn run(config: MetaSearchConfig) -> anyhow::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .service(index::get)
            .service(search::get)
            .service(static_handler)
            .default_service(web::to(not_found))
    });

    println!(
        "starting webserver on {}:{}",
        config.server.bind, config.server.port
    );

    server
        .bind((config.server.bind, config.server.port))?
        .run()
        .await?;

    Ok(())
}

pub static METASEARCHER: LazyLock<MetaSearcher> = LazyLock::new(|| {
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

#[get("/assets/{_:.*}")]
async fn static_handler(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}

fn handle_embedded_file(path: &str) -> Option<HttpResponse> {
    Assets::get(path).map(|f| {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();

        HttpResponse::Ok()
            .content_type(mime.as_ref())
            .body(f.data.into_owned())
    })
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

async fn not_found() -> impl Responder {
    let html = html! {
        (DOCTYPE)
        html {
            (html_head("404 not found"))
            body {
                 h1 { "page not found" }
            }
        }
    };

    HttpResponse::NotFound().body(html)
}
