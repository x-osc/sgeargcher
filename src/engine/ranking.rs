use std::collections::{HashMap, hash_map};

use crate::{
    engine::{EngineMetadata, SearchResult, scrapers::EngineResponse},
    url::normalize_url,
};

pub fn merge_and_rank_responses(
    responses: Vec<(EngineMetadata, Vec<EngineResponse>)>,
) -> Vec<SearchResult> {
    // url to result
    let mut final_results: HashMap<String, SearchResult> = HashMap::new();

    let engine_weights: HashMap<String, f64> = responses
        .iter()
        .map(|(engine, _)| (engine.name.clone(), engine.weight))
        .collect();

    for (engine, results) in responses {
        for (response_index, engine_response) in results.into_iter().enumerate() {
            // 2 is adjustable constant
            let base_result_score = 1. / (response_index as f64 + 2.);
            let result_score = base_result_score * engine.weight;

            let url = normalize_url(&engine_response.url);

            match final_results.entry(url.clone()) {
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(SearchResult {
                        title: engine_response.title,
                        url: url,
                        description: engine_response.description,
                        score: result_score,
                        engines: vec![engine.name.clone()],
                        highest_engine_weight: engine.weight,
                    });
                }
                hash_map::Entry::Occupied(mut entry) => {
                    let existing = entry.get_mut();
                    existing.score += result_score;
                    existing.engines.push(engine.name.clone());

                    if engine.weight > existing.highest_engine_weight {
                        existing.title = engine_response.title;
                        existing.description = engine_response.description;
                        existing.highest_engine_weight = engine.weight;
                    }
                }
            };
        }
    }

    let mut results_vec: Vec<_> = final_results.into_values().collect();
    results_vec.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
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
