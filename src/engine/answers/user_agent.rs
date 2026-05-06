use async_trait::async_trait;
use maud::html;

use crate::{
    engine::{
        answers::{AnswerEngine, headers::headers_html},
        scrapers::SearchContext,
    },
    regex,
};

pub struct UserAgentAnswer;

#[async_trait]
impl AnswerEngine for UserAgentAnswer {
    async fn query(&self, search: SearchContext) -> Option<String> {
        let query = search.query.trim();

        if !regex!(r"^(what('s|s| is)?\s+)?(my\s+)?(ua|user ?agent)").is_match(&query) {
            return None;
        }

        let user_agent = search.headers.get("user-agent");

        let all_headers_html = html! {
            details {
                summary.headers-title { "all headers" }
                (headers_html(&search.headers))
            }
        };

        let html = if let Some(user_agent) = user_agent {
            html! {
                p.answer-query { "your user agent is" }
                p.answer-result { (user_agent) }
                (all_headers_html)
            }
        } else {
            html! {
                p.answer-query { "user agent" }
                p.answer-result { "you don't seem to have a user agent" }
                (all_headers_html)
            }
        };

        Some(html.into_string())
    }
}
