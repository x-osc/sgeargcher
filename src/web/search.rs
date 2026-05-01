use std::collections::HashMap;

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use maud::html;

pub async fn get(Query(params): Query<HashMap<String, String>>) -> Response {
    let query = params.get("q").map(|s| s.trim()).unwrap_or("");
    if query.is_empty() {
        return Redirect::to("/").into_response();
    }

    html! {
        p { "hi" }
    }
    .into_response()
}
