use leptos::SignalGetUntracked;
use leptos_router::use_query_map;


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


pub fn preserve_log_level(uri: String) -> String {
    if let Some(log_level) = use_query_map().get_untracked().get("RUST_LOG") {
        format!("{uri}?RUST_LOG={log_level}")
    } else {
        uri
    }
}
