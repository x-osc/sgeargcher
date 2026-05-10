use std::{fs, path::PathBuf};

use actix_files::Files;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware, web};
use maud::{DOCTYPE, Markup, html};
use rust_embed::RustEmbed;

use crate::{config::MetaSearchConfig, web::settings::ClientSettings};

mod autocomplete;
mod config;
mod index;
mod search;
mod settings;
mod utils;

#[derive(RustEmbed)]
#[folder = "src/web/assets"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub config: MetaSearchConfig,
    pub themes_dir: PathBuf,
    pub available_themes: Vec<String>,
}

impl AppState {
    pub fn new(config: MetaSearchConfig) -> anyhow::Result<Self> {
        let themes_dir = config.config_dir.join(&config.themes_dir);
        let available_themes = fs::read_dir(&themes_dir)?
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

        Ok(Self {
            config,
            themes_dir,
            available_themes,
        })
    }
}

pub async fn run(config: MetaSearchConfig) -> anyhow::Result<()> {
    let state = AppState::new(config)?;
    let data = web::Data::new(state.clone());

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
            .service(Files::new("/themes", &data.themes_dir))
            .service(static_handler)
            .default_service(web::to(not_found))
    });

    let (bind, port) = (state.config.server.bind.clone(), state.config.server.port);
    println!("starting webserver on {}:{}", bind, port);
    println!("serving themes from {}", state.themes_dir.display());
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
