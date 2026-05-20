use std::fs;

use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web};
use maud::{DOCTYPE, Markup, html};
use rust_embed::RustEmbed;

use crate::{
    config::ResolvedConfig,
    engine::MetaSearcher,
    web::{config::metasearcher, settings::ClientSettings},
};

mod autocomplete;
mod config;
mod index;
mod search;
mod settings;
mod utils;

#[derive(RustEmbed)]
#[folder = "src/web/assets"]
struct Assets;

pub struct AppState {
    pub searcher: MetaSearcher,
    pub config: ResolvedConfig,
    pub available_themes: Vec<String>,
}

impl AppState {
    pub async fn new(config: ResolvedConfig) -> anyhow::Result<Self> {
        let available_themes = fs::read_dir(&config.themes_dir)?
            .filter_map(|e| {
                let e = e.ok()?;
                if !e.file_type().ok()?.is_file() {
                    return None;
                }

                let name = e.file_name().into_string().ok()?;
                if name.starts_with('.') || !name.ends_with(".css") {
                    return None;
                }

                Some(name.trim_end_matches(".css").to_owned())
            })
            .collect();

        let searcher = metasearcher(&config).await?;

        Ok(Self {
            searcher,
            config,
            available_themes,
        })
    }
}

pub async fn run(config: ResolvedConfig) -> anyhow::Result<()> {
    let state = AppState::new(config).await?;
    let config = state.config.clone();
    let data = web::Data::new(state);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .service(index::get)
            .service(search::get)
            .service(autocomplete::get)
            .service(settings::get)
            .service(settings::post)
            .service(Files::new("/themes", &data.config.themes_dir))
            .service(static_handler)
            .default_service(web::to(not_found))
    });

    let (bind, port) = (config.server.bind.clone(), config.server.port);
    println!("starting webserver on {}:{}", bind, port);
    println!("serving themes from {}", config.themes_dir.display());
    server.bind((bind.as_str(), port))?.run().await?;

    Ok(())
}

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

fn head_stuff(settings: &ClientSettings) -> Markup {
    html! {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1.0";
        link rel="stylesheet" href="/assets/style.css";
        link rel="stylesheet" href=(&format!("/themes/{}.css", settings.theme));
    }
}

async fn not_found(settings: ClientSettings) -> impl Responder {
    let html = html! {
        (DOCTYPE)
        html {
            head {
                (head_stuff(&settings))
                title { "404 not found" }
            }
            body {
                 h1 { "page not found" }
            }
        }
    };

    HttpResponse::NotFound().body(html)
}
