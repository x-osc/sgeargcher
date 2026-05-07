use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::{ConnectInfo, Query},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use maud::{DOCTYPE, Markup, html};

use crate::{
    engine::{SearchResult, scrapers::SearchContext},
    web::{DEFAULT_USER_CONFIG, METASEARCHER, html_head},
};

pub async fn get(
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    let query = params.get("q").map(|s| s.trim()).unwrap_or("");
    if query.is_empty() {
        return Redirect::to("/").into_response();
    }

    let config = DEFAULT_USER_CONFIG.merge_into_default(&METASEARCHER);

    let response = METASEARCHER
        .run_search(
            SearchContext {
                query: query.to_owned(),
                ip: headers
                    .get("x-forwarded-for")
                    .map(|ip| ip.to_str().unwrap_or("").to_owned())
                    .unwrap_or_else(|| addr.ip().to_string()),
                headers: headers
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k.map(|k| k.to_string()).unwrap_or_default(),
                            v.to_str().unwrap_or_default().to_string(),
                        )
                    })
                    .collect(),
            },
            &config,
        )
        .await;

    html! {
        (DOCTYPE)
        html {
            (html_head(&format!("{query} - sgeargcher")))
            body.search-page {
                main {
                    form #search-form action="/search" method="get" {
                        input.search-input type="text" name="q" placeholder="sgeargch..." value=(query) autocomplete="off";
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
    }
    .into_response()
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
