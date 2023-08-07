mod chain;
mod components;
mod evm;

use std::sync::{Arc, Mutex};

use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_ethereum_provider::{AccountLabel, ConnectButton, EthereumContextProvider};
use wasm_bindgen::JsCast;

use components::{
    blockchain::Blockchain,
    mandelbrot::Mandelbrot,
};


#[function_component]
fn App() -> Html {
    let window = web_sys::window().unwrap();
    let height = use_state(|| (window.inner_height().unwrap().as_f64().unwrap() + 1.0) * 2.0);
    let resize_state = use_state(|| wasm_bindgen::closure::Closure::<dyn FnMut()>::new({
        let window = window.clone();
        let height = height.clone();
        move || height.set((window.inner_height().unwrap().as_f64().unwrap() + 1.0) * 2.0)
    }));
    if window.onresize().is_none() {
        window.set_onresize(Some(resize_state.as_ref().unchecked_ref()));
    }

    let interface = Arc::new(Mutex::new(mandelbrot_explorer::Interface {
        sample_location: mandelbrot_explorer::SampleLocation::new(*height, *height),
        frames: Vec::new(),
        frame_selected_callback: None,
        redraw: None,
    }));

    html! {
        <BackdropViewer>
            <Split>
                <SplitItem fill={true}>
                    <Mandelbrot size={(*height, *height)} interface={interface.clone()}/>
                </SplitItem>
                <SplitItem>
                    <EthereumContextProvider>
                        <PageSection
                            r#type={PageSectionType::Default}
                            variant={PageSectionVariant::Light}
                            limit_width=true
                            sticky={[PageSectionSticky::Top]}
                            fill={PageSectionFill::Fill}
                        >
                            <ConnectButton/>
                            <AccountLabel/>
                        </PageSection>
                        <Blockchain mandelbrot={interface}/>
                    </EthereumContextProvider>
                </SplitItem>
            </Split>
        </BackdropViewer>
    }
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


fn main() {
    let query_string = web_sys::window().unwrap().location().search().unwrap();
    let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
        .and_then(|x| x.parse().ok())
        .unwrap_or(log::Level::Error);
    console_log::init_with_level(level).expect("could not initialize logger");
    yew::Renderer::<App>::new().render();
}
