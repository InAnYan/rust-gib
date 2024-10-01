use url::Url;

/// Convert Url into a String without trailing backslash.
pub fn clear_url(url: Url) -> String {
    let url = url.to_string();

    match url.strip_suffix("/") {
        Some(s) => s.to_string(),
        None => url,
    }
}
