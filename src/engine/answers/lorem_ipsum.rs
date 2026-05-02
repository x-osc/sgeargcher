// lorem ipsum is terrible placeholder text i hate it with a passion !!!!!

use async_trait::async_trait;
use maud::html;

use crate::{
    engine::{answers::AnswerEngine, scrapers::SearchContext},
    regex,
};

pub struct LoremIpsumAnswer;

#[async_trait]
impl AnswerEngine for LoremIpsumAnswer {
    async fn query(&self, search: SearchContext) -> Option<String> {
        let query = search.query.trim().to_lowercase();

        let regex_matches = regex!(
            r"^(((generate)|(give me)|(i need)|(what is)|(full)|\w*)\s+)?((lorem ipsum)|(lipsum)|(ipsum dolor))"
        ).is_match(&query);

        let is_placeholder = [
            "placeholder text",
            "dummy text",
            "filler text",
            "sample text",
            "mock text",
            "fake latin paragraph",
        ]
        .iter()
        .any(|p| query.contains(p));

        if !(regex_matches || is_placeholder) {
            return None;
        }

        Some(
            html! {
                p.grey { "Lorem Ipsum" }
                p.lipsum {
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. "
                    "Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. "
                    "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. "
                    "Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
                }
            }
            .into_string(),
        )
    }
}
