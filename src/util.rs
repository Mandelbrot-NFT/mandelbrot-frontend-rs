use leptos::prelude::*;
use leptos_router::params::ParamsMap;
use web_sys::window;

pub fn set_session_item(key: &str, value: &str) {
    if let Some(storage) = window().and_then(|w| w.session_storage().ok()).flatten() {
        let _ = storage.set_item(key, value);
    }
}

pub fn get_session_item(key: &str) -> Option<String> {
    window()
        .and_then(|w| w.session_storage().ok())
        .flatten()
        .and_then(|storage| storage.get_item(key).ok().flatten())
}

/// Parse the query string as returned by `web_sys::window()?.location().search()?` and get a
/// specific key out of it.
pub fn parse_url_query_string<'a>(query: &'a str, search_key: &str) -> Option<&'a str> {
    let query_string = query.strip_prefix('?')?;

    for pair in query_string.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next()?;
        let value = pair.next()?;

        if key == search_key {
            return Some(value);
        }
    }

    None
}

pub fn preserve_log_level(uri: String, query_map: Memo<ParamsMap>) -> String {
    if let Some(log_level) = query_map.get_untracked().get("RUST_LOG") {
        if uri.contains("?") {
            format!("{uri}&RUST_LOG={log_level}")
        } else {
            format!("{uri}?RUST_LOG={log_level}")
        }
    } else {
        uri
    }
}
