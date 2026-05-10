use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use url::Url;

pub const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

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

    // if http change scheme to https
    if url.scheme() == "http" {
        url.set_scheme("https").unwrap();
    }

    // remove fragment (#section)
    url.set_fragment(None);

    // normalize percent encoding
    let decoded_path = percent_decode_str(url.path()).decode_utf8_lossy();
    let normalized_path = utf8_percent_encode(&decoded_path, FRAGMENT).to_string();
    url.set_path(&normalized_path);

    // remove trailing slash
    let path = url.path().to_string();
    if let Some(path) = path.strip_suffix('/') {
        url.set_path(path);
    }

    if let Some(host) = url.host_str() {
        match host {
            h if h.ends_with("reddit.com") => {
                let path = url.path().to_lowercase();
                url.set_path(&path);
            }
            "play.google.com" => {
                // google has no hl param
                // duckduckgo uses hl=en-US
                // brave & mojeek uses hl=en_US
                // ive also seen brave use hl=en_ZA ???
                let new_query_pairs: Vec<_> = url
                    .query_pairs()
                    .filter(|(k, _)| k != "hl")
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect();
                set_query_pairs(&mut url, new_query_pairs);
            }
            "github.com" => {
                let segments: Vec<_> = url
                    .path_segments()
                    .map(|s| s.collect())
                    .unwrap_or_else(Vec::new);

                // lowercase repo and owner only
                if segments.len() >= 2 {
                    let owner = segments[0].to_lowercase();
                    let repo = segments[1].to_lowercase();

                    let mut new_path = format!("/{}/{}", owner, repo);

                    if segments.len() > 2 {
                        new_path.push('/');
                        new_path.push_str(&segments[2..].join("/"));
                    }

                    url.set_path(&new_path);
                }
            }
            "docs.rs" => {
                let segments: Vec<_> = url
                    .path_segments()
                    .map(|s| s.collect())
                    .unwrap_or_else(Vec::new);

                // rewrite /crate/latest/crate -> /crate_name
                if segments.len() == 3 && segments[1] == "latest" && segments[0] == segments[2] {
                    let new_path = format!("/{}", segments[0]);
                    url.set_path(&new_path);
                }

                let path = url.path().to_string();
                let stripped = path.strip_suffix("/index.html");
                if let Some(stripped) = stripped {
                    url.set_path(stripped);
                }
            }

            _ => {}
        }
    }

    // remove tracking params
    const TRACKING_PARAMS: &[&str] = &["ref_src", "_sm_au_", "srsltid"];
    let new_query_pairs: Vec<_> = url
        .query_pairs()
        .filter(|(k, _)| !TRACKING_PARAMS.contains(&k.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    set_query_pairs(&mut url, new_query_pairs);

    url.to_string()
}

fn set_query_pairs(url: &mut Url, new_query_pairs: Vec<(String, String)>) {
    if new_query_pairs.is_empty() {
        url.set_query(None);
    } else {
        url.set_query(Some(
            &url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(new_query_pairs)
                .finish(),
        ));
    }
}
