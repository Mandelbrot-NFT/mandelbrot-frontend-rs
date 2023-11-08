mod chain;
mod components;
mod evm;
mod state;
mod util;

use leptos::*;

use components::app::App;
use util::parse_url_query_string;


fn main() {
    let query_string = web_sys::window().unwrap().location().search().unwrap();
    let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
        .and_then(|x| x.parse().ok())
        .unwrap_or(log::Level::Error);
    console_log::init_with_level(level).expect("could not initialize logger");
    mount_to_body(|| view! { <App/> })
}
