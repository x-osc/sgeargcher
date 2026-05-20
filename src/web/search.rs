use std::{collections::HashMap, time::Duration};

use actix_web::{HttpRequest, HttpResponse, Responder, get, http::header, web};
use maud::{DOCTYPE, Markup, html};

use crate::{
    engine::{SearchResult, scrapers::SearchContext},
    web::{AppState, config::DEFAULT_USER_CONFIG, head_stuff, settings::ClientSettings},
};

#[get("/search")]
pub async fn get(
    params: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    req: HttpRequest,
    settings: ClientSettings,
) -> impl Responder {
    let query = params.get("q").map(|s| s.trim()).unwrap_or("");
    if query.is_empty() {
        return HttpResponse::SeeOther()
            .insert_header((header::LOCATION, "/"))
            .finish();
    }

    let connection_info = req.connection_info();
    let ip = connection_info
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string();
    let headers: HashMap<_, _> = req
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();

    let mut config = DEFAULT_USER_CONFIG.merge_into_default(&data.searcher);
    config.timeout = Duration::from_millis(data.config.timeout);

    let response = data
        .searcher
        .run_search(
            SearchContext {
                query: query.to_owned(),
                ip,
                headers,
            },
            &config,
        )
        .await;

    let html = html! {
        (DOCTYPE)
        html {
            head {
                (head_stuff(&settings))
                title { (query) " - sgeargcher" }
                link rel="stylesheet" href="/assets/search.css";
                link rel="stylesheet" href="/assets/autocomplete.css";
                script src="/assets/autocomplete.js" defer {}
            }
            body.search-page {
                main {
                    form #search-form action="/search" method="get" {
                        div.search-input-wrapper {
                            input.search-input type="text" name="q" placeholder="sgeargch..." value=(query) autocomplete="off";
                        }
                        input.search-submit type="submit" value="sgeargch";
                    }

                    @if let Some(answer) = response.answer {
                        div.answer {
                            (answer.html);
                            div.engines {
                                span.engine-item { (answer.engine) }
                            }
                        }
                    }

                    @for result in response.results {
                        (single_search_result(result));
                    }
                }
            }
        }
    };

    HttpResponse::Ok().body(html.into_string())
}

fn single_search_result(result: SearchResult) -> Markup {
    let SearchResult {
        title,
        url,
        description,
        score: _,
        engines,
        ..
    } = result;

    html! {
        section.result {
            h3.title { a rel="nofollow external" href=(url) { (title) } }
            span.url { a rel="nofollow external" href=(url) { (url) } }
            p.description { (description) }
            div.engines {
                @for engine in engines {
                    span.engine-item { (engine) }
                }
            }
        }
    }
}
