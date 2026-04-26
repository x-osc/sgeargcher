use url::Url;

// somewhat stolen from mat's metasearch
pub fn normalize_url(url: &str) -> String {
    let mut url = url.trim_end_matches('#').to_string();
    if url.is_empty() {
        return String::new();
    }

    // if bare domain add https
    if !url.contains("://") {
        url = format!("https://{}", url);
    }

    let Ok(mut url) = Url::parse(&url) else {
        return url.to_string();
    };

    // make sure the scheme is https
    if url.scheme() == "http" {
        url.set_scheme("https").unwrap();
    }

    // remove fragment (#section)
    url.set_fragment(None);

    // remove trailing slash
    let path = url.path().to_string();
    if let Some(path) = path.strip_suffix('/') {
        url.set_path(path);
    }

    // remove tracking params
    const TRACKING_PARAMS: &[&str] = &["ref_src", "_sm_au_"];
    let new_query_pairs: Vec<_> = url
        .query_pairs()
        .filter(|(k, _)| !TRACKING_PARAMS.contains(&k.as_ref()))
        .collect();

    if new_query_pairs.is_empty() {
        url.set_query(None);
    } else {
        url.set_query(Some(
            &url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(new_query_pairs)
                .finish(),
        ));
    }

    url.to_string()
}
