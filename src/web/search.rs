use std::collections::HashMap;

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use maud::{DOCTYPE, Markup, html};

use crate::{
    engine::{SearchResult, run_search, scrapers::SearchQuery},
    web::get_config,
};

pub async fn get(Query(params): Query<HashMap<String, String>>) -> Response {
    let query = params.get("q").map(|s| s.trim()).unwrap_or("");
    if query.is_empty() {
        return Redirect::to("/").into_response();
    }

    let results = run_search(
        get_config(),
        SearchQuery {
            query: query.to_owned(),
        },
    )
    .await;

    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "sgeargcher" }
            }
            body {
                form #searchinput action="/search" method="get" {
                    input type="text" name="q" placeholder="sgeargch..." value=(query) autofocus onfocus="this.select()" autocomplete="off";
                    input type="submit" value="sgeargch";
                }

                @for result in results {
                    (single_search_result(result))
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
