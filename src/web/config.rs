use std::sync::LazyLock;

use crate::engine::config::{CustomRank, EngineSetting, SearchConfig};

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
});
