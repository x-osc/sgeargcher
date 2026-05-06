use std::collections::HashMap;

use async_trait::async_trait;
use maud::{Markup, html};

use crate::{
    engine::{answers::AnswerEngine, scrapers::SearchContext},
    regex,
};

pub struct HeadersAnswer;

#[async_trait]
impl AnswerEngine for HeadersAnswer {
    async fn query(&self, search: SearchContext) -> Option<String> {
        let query = search.query.trim();

        if !regex!(r"^(what('s|s| is| are|re|'re)?\s+)?(my\s+)?(https?\s+)?headers")
            .is_match(&query)
        {
            return None;
        }

        let html = html! {
            p.answer-query { "your headers are" }
            (headers_html(&search.headers))
        };

        Some(html.into_string())
    }
}

pub fn headers_html(headers: &HashMap<String, String>) -> Markup {
    html! {
        @for (header, value) in headers.iter() {
            div {
                span.header { (header) } ": " span.header-value { (value) }
            }
        }
    }
}
