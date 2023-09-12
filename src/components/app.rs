use std::sync::{Arc, Mutex};

use leptonic::prelude::*;
use leptos::*;
use leptos_ethereum_provider::{AccountLabel, ConnectButton, EthereumContextProvider};
use wasm_bindgen::JsCast;

use super::{
    about::About,
    blockchain::Blockchain,
    guide::Guide,
    mandelbrot::Mandelbrot,
};


#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let window = web_sys::window().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap() + 1.0;
    let (get_height, set_height) = create_signal(cx, height);

    let resize_state = store_value(cx, Arc::new(wasm_bindgen::closure::Closure::<dyn FnMut()>::new({
        let window = window.clone();
        move || set_height(window.inner_height().unwrap().as_f64().unwrap() + 1.0)
    })));
    if window.onresize().is_none() {
        window.set_onresize(Some((*resize_state.get_value()).as_ref().unchecked_ref()));
    }

    let interface = Arc::new(Mutex::new(mandelbrot_explorer::Interface {
        sample_location: mandelbrot_explorer::SampleLocation::new(height, height),
        coloring: mandelbrot_explorer::Coloring {
            max_iterations: 1360,
            offset: 0.0,
        },
        frames: Vec::new(),
        frame_event_callback: None,
        redraw: None,
    }));
    provide_context(cx, interface);

    {
        view! { cx,
            <Root default_theme=LeptonicTheme::default()>
                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                    <Mandelbrot
                        size=Signal::derive(cx, move || (get_height(), get_height()))
                    />
                    <Tabs mount=Mount::Once>
                        <Tab name="dapp" label="DApp">
                            <EthereumContextProvider>
                                <ConnectButton/>
                                <AccountLabel/>
                                <Separator/>
                                <Blockchain/>
                            </EthereumContextProvider>
                        </Tab>
                        <Tab name="description" label="Description">
                            <About/>
                        </Tab>
                        <Tab name="how_to_use" label="How to Use">
                            <Guide/>
                        </Tab>
                    </Tabs>
                </Stack>
            </Root>
        }
    }
}
