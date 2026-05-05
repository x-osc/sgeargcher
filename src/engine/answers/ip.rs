use async_trait::async_trait;
use maud::html;

use crate::{
    engine::{answers::AnswerEngine, scrapers::SearchContext},
    regex,
};

pub struct IpAnswer;

#[async_trait]
impl AnswerEngine for IpAnswer {
    async fn query(&self, search: SearchContext) -> Option<String> {
        let query = search.query.trim();

        if !regex!(r"^(what('s|s| is)?\s+)?(my\s+)?ip").is_match(&query) {
            return None;
        }

        let ip = search.ip;

        Some(
            html! {
                p.answer-query { "your ip is:" }
                p.answer-result.ip { (ip) }
            }
            .into_string(),
        )
    }
}
