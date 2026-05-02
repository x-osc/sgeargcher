use std::collections::HashMap;

use async_trait::async_trait;
use maud::{PreEscaped, html};
use percent_encoding::utf8_percent_encode;
use serde::Deserialize;
use url::Url;
use wreq_util::{Emulation, EmulationOS, EmulationOption};

use crate::{
    engine::{answers::AnswerEngine, scrapers::SearchContext},
    regex,
    utils::{to_title_case, url::FRAGMENT},
};

pub struct DictionaryAnswer;

#[async_trait]
impl AnswerEngine for DictionaryAnswer {
    async fn query(&self, search: SearchContext) -> Option<String> {
        let query = search.query.trim();

        let regex1 = regex!(r"^def(?:ine)?\s+(.+?)(?:please|pls)?$");
        let regex2 = regex!(r"^meaning(?: of)?\s+(.+?)$");
        let regex3 = regex!(
            r"^(?:(?:what(?:'s|s| is)?)|(?:give me)\s+)?(.+?)\s+(?:def|meaning|definition)$"
        );

        let word = if let Some(caps) = regex1.captures(query) {
            caps.get(1).map(|m| m.as_str())
        } else if let Some(caps) = regex2.captures(query) {
            caps.get(1).map(|m| m.as_str())
        } else if let Some(caps) = regex3.captures(query) {
            caps.get(1).map(|m| m.as_str())
        } else {
            None
        }?;

        let client = wreq::Client::builder()
            .emulation(
                EmulationOption::builder()
                    .emulation(Emulation::Firefox128)
                    .emulation_os(EmulationOS::Windows)
                    .build(),
            )
            .build()
            .ok()?;

        let lower = word.to_lowercase();
        let title = to_title_case(word);

        let mut variants = Vec::new();
        variants.push(word.to_string());
        if lower != word {
            variants.push(lower.clone());
        }
        if title != word && title != lower {
            variants.push(title);
        }

        let (actual_word, response) = async {
            for variant in &variants {
                let encoded = utf8_percent_encode(&variant, FRAGMENT);
                let url = format!(
                    "https://en.wiktionary.org/api/rest_v1/page/definition/{}",
                    encoded
                );

                let res = client.get(&url).send().await.ok()?;

                if !res.status().is_success() {
                    continue;
                }

                return Some((variant, res.text().await.ok()?));
            }
            None
        }
        .await?;

        generate_answer(actual_word, response)
    }
}

#[derive(Debug, Deserialize)]
pub struct WiktionaryResponse(pub HashMap<String, Vec<WiktionaryEntry>>);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[expect(dead_code)]
pub struct WiktionaryEntry {
    pub part_of_speech: String,
    pub language: String,
    pub definitions: Vec<WiktionaryDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiktionaryDefinition {
    pub definition: String,
    #[serde(default)]
    pub examples: Vec<String>,
}

// https://github.com/mat-1/metasearch2/blob/4ca209aa9734b6a00dbb94f0c3974b7c86f6721a/src/engines/answer/dictionary.rs#L54
fn generate_answer(word: &str, response: String) -> Option<String> {
    let Ok(data) = serde_json::from_str::<WiktionaryResponse>(&response) else {
        return None;
    };

    let Some(entries) = data.0.get("en") else {
        return None;
    };

    let mut cleaner = ammonia::Builder::new();
    cleaner
        .link_rel(None)
        .url_relative(ammonia::UrlRelative::RewriteWithBase(
            Url::parse("https://en.wiktionary.org").unwrap(),
        ));

    let mut html = String::new();

    html.push_str(
        &html! {
            h2.dictionary-word {
                a href={ "https://en.wiktionary.org/wiki/" (utf8_percent_encode(&word, FRAGMENT)) } {
                    (key_to_title(word))
                }
            }
        }
        .into_string(),
    );

    for entry in entries {
        html.push_str(
            &html! {
                span.dictionary-part-of-speech {
                    (entry.part_of_speech.to_lowercase())
                }
            }
            .into_string(),
        );

        html.push_str("<ol>");
        let mut previous_definitions: Vec<String> = Vec::new();
        for def_group in &entry.definitions {
            if def_group.definition.is_empty() {
                continue;
            }

            if previous_definitions
                .iter()
                .any(|d| d.contains(&def_group.definition))
            {
                continue;
            }

            previous_definitions.push(def_group.definition.clone());

            html.push_str("<li class=\"dictionary-definition\">");
            let smoothed_def = def_group.definition.replace('“', "\"");
            let definition_html = cleaner.clean(&smoothed_def).to_string();

            html.push_str(
                &html! {
                    p {
                        (PreEscaped(definition_html))
                    }
                }
                .into_string(),
            );

            for example in &def_group.examples {
                let example_html = cleaner.clean(example).to_string();
                html.push_str(
                    &html! {
                        blockquote.dictionary-example {
                            (PreEscaped(example_html))
                        }
                    }
                    .into_string(),
                );
            }

            html.push_str("</li>");
        }
        html.push_str("</ol>");
    }

    Some(html)
}

fn key_to_title(key: &str) -> String {
    key.trim().replace('_', " ")
}
