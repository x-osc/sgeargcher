use std::{sync::LazyLock, time::Duration};

use crate::{
    config::ResolvedConfig,
    engine::{
        MetaSearcher,
        answers::{
            AnswerEngineMetadata, dictionary::DictionaryAnswer, headers::HeadersAnswer,
            ip::IpAnswer, lorem_ipsum::LoremIpsumAnswer, numbat::NumbatAnswer, tldr::TldrAnswer,
            user_agent::UserAgentAnswer,
        },
        autocomplete::google::GoogleCompletion,
        config::{CustomRank, EngineSetting, SearchConfig},
        scrapers::{
            EngineMetadata, brave::BraveSearch, duckduckgo::DuckDuckGoSearch,
            marginalia::MarginaliaSearch, mojeek::MojeekSearch, wiby::WibySearch,
            yahoo_japan::YahooJapanSearch,
        },
    },
};

pub async fn metasearcher(config: &ResolvedConfig) -> anyhow::Result<MetaSearcher> {
    let mut searcher = MetaSearcher::new();
    searcher.add_engine(
        Box::new(DuckDuckGoSearch),
        EngineMetadata::new("duckduckgo"),
    );
    searcher.add_engine(
        Box::new(MarginaliaSearch),
        EngineMetadata::new("marginalia"),
    );
    searcher.add_engine(Box::new(BraveSearch), EngineMetadata::new("brave"));
    searcher.add_engine(Box::new(WibySearch), EngineMetadata::new("wiby"));
    searcher.add_engine(Box::new(MojeekSearch), EngineMetadata::new("mojeek"));
    searcher.add_engine(
        Box::new(YahooJapanSearch),
        EngineMetadata::new("yahoo_japan"),
    );

    searcher.add_completion_engine(Box::new(GoogleCompletion), "google".into());

    searcher.add_answer_engine(Box::new(IpAnswer), AnswerEngineMetadata::new("ip"));
    searcher.add_answer_engine(
        Box::new(LoremIpsumAnswer),
        AnswerEngineMetadata::new("lorem ipsum"),
    );
    searcher.add_answer_engine(
        Box::new(DictionaryAnswer),
        AnswerEngineMetadata::new("wiktionary"),
    );
    searcher.add_answer_engine(Box::new(NumbatAnswer), AnswerEngineMetadata::new("numbat"));
    searcher.add_answer_engine(
        Box::new(UserAgentAnswer),
        AnswerEngineMetadata::new("user agent"),
    );
    searcher.add_answer_engine(
        Box::new(HeadersAnswer),
        AnswerEngineMetadata::new("headers"),
    );
    searcher.add_answer_engine(
        Box::new(TldrAnswer::new(config.cache_dir.join("tldr")).await?),
        AnswerEngineMetadata::new("tldr"),
    );

    Ok(searcher)
}

pub static DEFAULT_USER_CONFIG: LazyLock<SearchConfig> = LazyLock::new(|| SearchConfig {
    engine_settings: [
        ("duckduckgo".into(), EngineSetting::new().weight(0.9)),
        ("marginalia".into(), EngineSetting::new().weight(0.7)),
        ("brave".into(), EngineSetting::new().weight(0.6)),
        ("yahoo_japan".into(), EngineSetting::new().weight(1.1)),
        ("wiby".into(), EngineSetting::new().weight(0.15)),
        ("mojeek".into(), EngineSetting::new().weight(0.4)),
    ]
    .into(),

    // ref https://kagi.com/stats?stat=insights&sub_ins=domains&k=-1
    custom_rank: [
        CustomRank::domain("wikipedia.org", 1.5),
        CustomRank::domain("wiktionary.org", 1.2),
        CustomRank::domain("github.com", 1.4),
        CustomRank::domain("gitlab.com", 1.5),
        CustomRank::domain("codeberg.org", 1.5),
        CustomRank::domain("news.ycombinator.com", 1.2),
        CustomRank::domain("reddit.com", 1.1),
        CustomRank::domain("stackoverflow.com", 1.1),
        CustomRank::domain("stackexchange.com", 1.1),
        CustomRank::domain("superuser.com", 1.1),
        CustomRank::domain("developer.mozilla.org", 1.4),
        CustomRank::domain("wiki.archlinux.org", 1.5),
        CustomRank::domain("doc.rust-lang.org", 1.5),
        CustomRank::domain("users.rust-lang.org", 1.2),
        CustomRank::domain("docs.rs", 1.4),
        CustomRank::domain("css-tricks.com", 1.2),
        CustomRank::domain("minecraft.wiki", 1.25),
        CustomRank::domain("modrinth.com", 1.15),
        //
        CustomRank::domain("quora.com", 0.7),
        CustomRank::domain("facebook.com", 0.7),
        CustomRank::domain("medium.com", 0.7),
        CustomRank::domain("dev.to", 0.8),
        CustomRank::domain("linkedin.com", 0.6),
        CustomRank::domain("fandom.com", 0.65),
        CustomRank::domain("tiktok.com", 0.8),
        CustomRank::domain("amazon.com", 0.8),
        CustomRank::domain("pinterest.com", 0.6),
        CustomRank::domain("w3schools.com", 0.7),
        CustomRank::domain("geeksforgeeks.org", 0.7),
        CustomRank::domain("freecodecamp.net", 0.9),
        CustomRank::domain("alternativeto.net", 0.7),
        CustomRank::domain("play.google.com", 0.35),
        CustomRank::domain("apps.apple.com", 0.35),
        CustomRank::domain("apps.microsoft.com", 0.35),
        CustomRank::domain("wikihow.com", 1.0),
    ]
    .into(),

    timeout: Duration::from_millis(3500),
});
