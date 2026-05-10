use itertools::Itertools;
use regex::Regex;
use std::{collections::HashMap, time::Duration};

use crate::engine::MetaSearcher;

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub engine_settings: HashMap<String, EngineSetting>,
    pub custom_rank: Vec<CustomRank>,
    pub timeout: Duration,
}

impl SearchConfig {
    pub fn default_config(searcher: &MetaSearcher) -> SearchConfig {
        SearchConfig {
            engine_settings: searcher
                .engines
                .iter()
                .map(|e| (e.metadata.name.clone(), EngineSetting::default()))
                .collect(),
            custom_rank: Vec::new(),
            timeout: Duration::from_millis(5000),
        }
    }

    pub fn merge_into_default(&self, searcher: &MetaSearcher) -> SearchConfig {
        let config = self.clone();
        let defaults = SearchConfig::default_config(searcher);

        let unknown_engines: Vec<&String> = config
            .engine_settings
            .keys()
            .filter(|name| !defaults.engine_settings.contains_key(*name))
            .collect();

        if !unknown_engines.is_empty() {
            println!(
                "unknown engines in config weights: {}; registered engines are: {}",
                unknown_engines.iter().join(", "),
                defaults.engine_settings.keys().join(", "),
            );
        }

        SearchConfig {
            engine_settings: {
                let mut settings = defaults.engine_settings;
                settings.extend(config.engine_settings);
                settings
            },
            ..config
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineSetting {
    pub weight: f64,
    pub enabled: bool,
}

impl EngineSetting {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    #[expect(dead_code)]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for EngineSetting {
    fn default() -> Self {
        Self {
            weight: 1.0,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CustomRank {
    pub selector: CustomRankSelector,
    pub weight: f64,
    pub blocked: bool,
}

impl CustomRank {
    pub fn domain(domain: &str, weight: f64) -> Self {
        Self {
            selector: CustomRankSelector::Domain(domain.into()),
            weight,
            blocked: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CustomRankSelector {
    #[expect(dead_code)]
    Regex(Regex),
    Domain(String),
}
