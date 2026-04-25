pub mod duckduckgo;
pub mod marginalia;

#[derive(Debug)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
}
