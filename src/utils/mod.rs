pub mod url;

#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::RegexBuilder::new($re)
                .case_insensitive(true)
                .build()
                .unwrap()
        });
        &RE
    }};
}
