mod chain;
mod components;
mod context;
mod evm;
mod util;

use leptos::prelude::*;

use components::App;
use leptos_router::components::Router;
use util::parse_url_query_string;

#[cfg(debug_assertions)]
fn init_debug_hooks() {
    console_error_panic_hook::set_once();
}

fn main() {
    #[cfg(debug_assertions)]
    init_debug_hooks();

    let query_string = web_sys::window().unwrap().location().search().unwrap();
    let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
        .and_then(|x| x.parse().ok())
        .unwrap_or(log::Level::Error);
    console_log::init_with_level(level).expect("could not initialize logger");
    mount_to_body(|| {
        view! {
            <Router>
                <App/>
            </Router>
        }
    })
}
