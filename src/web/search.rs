use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::{ConnectInfo, Query},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use maud::{DOCTYPE, Markup, html};

use crate::{
    engine::{SearchResult, run_search, scrapers::SearchContext},
    web::{get_config, html_head},
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

    let response = run_search(
        get_config(),
        SearchContext {
            query: query.to_owned(),
            ip: headers
                .get("x-forwarded-for")
                .map(|ip| ip.to_str().unwrap_or("").to_owned())
                .unwrap_or_else(|| addr.ip().to_string()),
        },
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
