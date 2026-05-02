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
        if !regex!("(^(what('s|s| is)? )?(my )?ip)").is_match(&search.query) {
            return None;
        }

        let ip = search.ip;

        Some(
            html! {
                p { "your ip is:" }
                span.ip { (ip) }
            }
            .into_string(),
        )
    }
}
