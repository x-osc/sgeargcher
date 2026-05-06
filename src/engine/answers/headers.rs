use async_trait::async_trait;
use maud::html;

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
            @for (header, value) in search.headers.iter() {
                div {
                    span.header { (header) } ": " span.header-value { (value) }
                }
            }
        };

        Some(html.into_string())
    }
}
