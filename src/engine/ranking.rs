use std::collections::{HashMap, hash_map};

use url::Url;

use crate::{
    engine::{
        SearchResult,
        config::{CustomRankSelector, SearchConfig},
        scrapers::EngineResponse,
    },
    utils::url::normalize_url,
};

pub fn merge_and_rank_responses(
    responses: Vec<(String, Vec<EngineResponse>)>,
    config: &SearchConfig,
) -> Vec<SearchResult> {
    // url to result
    let mut final_results: HashMap<String, SearchResult> = HashMap::new();

    for (engine_name, results) in responses.clone() {
        let engine_weight = config
            .engine_settings
            .get(&engine_name)
            .map(|e| e.weight)
            .unwrap_or(1.0);

        for (response_index, engine_response) in results.into_iter().enumerate() {
            // 2 is adjustable constant
            let base_result_score = 1. / (response_index as f64 + 2.);
            let result_score = base_result_score * engine_weight;

            let url = normalize_url(&engine_response.url);

            match final_results.entry(url.clone()) {
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(SearchResult {
                        title: engine_response.title,
                        url: url,
                        description: engine_response.description,
                        score: result_score,
                        engines: vec![engine_name.clone()],
                        highest_engine_weight: engine_weight,
                    });
                }
                hash_map::Entry::Occupied(mut entry) => {
                    let existing = entry.get_mut();
                    existing.score += result_score;

                    // for some reason
                    // somehow
                    // mojeek returns both
                    // "https://play.google.com/store/apps/details?id=com.brave.browser&hl=en_US" and
                    // "https://play.google.com/store/apps/details?id=com.brave.browser&hl=en"
                    // for the query "brave browser"
                    // and the url param gets normalized out
                    // so mojeek is placed in engines twice
                    // so we make sure that doesnt happen
                    if !existing.engines.contains(&engine_name) {
                        existing.engines.push(engine_name.clone());
                    }

                    if engine_weight > existing.highest_engine_weight {
                        existing.title = engine_response.title;
                        existing.description = engine_response.description;
                        existing.highest_engine_weight = engine_weight;
                    }
                }
            };
        }
    }

    let mut results_vec: Vec<SearchResult> = final_results
        .into_iter()
        .filter_map(|(_k, result)| {
            let Ok(parsed_url) = Url::parse(&result.url) else {
                return Some(result);
            };
            let domain = parsed_url.domain().unwrap_or(&result.url);

            // rev so that lower settings are higher priority
            let Some(custom_rank_settings) =
                config.custom_rank.iter().rev().find(|r| match &r.selector {
                    CustomRankSelector::Domain(selector) => domain.contains(selector),
                    CustomRankSelector::Regex(regex) => regex.is_match(domain),
                })
            else {
                return Some(result);
            };

            if custom_rank_settings.blocked || !(custom_rank_settings.weight > 0.) {
                return None;
            }

            let mut result = result;
            result.score *= custom_rank_settings.weight;

            Some(result)
        })
        .collect();

    results_vec.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let engine_weights: HashMap<String, f64> = responses
        .iter()
        .map(|(engine, _)| {
            (
                engine.to_owned(),
                config
                    .engine_settings
                    .get(engine)
                    .map(|e| e.weight)
                    .unwrap_or(1.0),
            )
        })
        .collect();
    // cant be bothered to do this properly
    results_vec.iter_mut().for_each(|r| {
        r.engines.sort_by(|a, b| {
            engine_weights[b]
                .partial_cmp(&engine_weights[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    results_vec
}
