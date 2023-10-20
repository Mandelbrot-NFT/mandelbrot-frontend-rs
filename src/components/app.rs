use std::{sync::{Arc, Mutex}, rc::Rc, cell::RefCell};

use leptonic::prelude::*;
use leptos::*;
use leptos_ethereum_provider::{AccountLabel, ConnectButton, EthereumContextProvider};
use mandelbrot_explorer::ISample;
use wasm_bindgen::JsCast;

use super::{
    about::About,
    blockchain::Blockchain,
    guide::Guide,
    mandelbrot::Mandelbrot,
};


#[component]
pub fn App() -> impl IntoView {
    let window = web_sys::window().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap() + 1.0;
    let (get_height, set_height) = create_signal(height);

    let resize_state = store_value(Arc::new(wasm_bindgen::closure::Closure::<dyn FnMut()>::new({
        let window = window.clone();
        move || set_height.set(window.inner_height().unwrap().as_f64().unwrap() + 1.0)
    })));
    if window.onresize().is_none() {
        window.set_onresize(Some((*resize_state.get_value()).as_ref().unchecked_ref()));
    }

    let interface = Arc::new(Mutex::new(mandelbrot_explorer::Interface::new(
        Rc::new(RefCell::new(mandelbrot_explorer::PerturbationEngine::new(height as u32, height as u32))),
        mandelbrot_explorer::Coloring {
            max_iterations: 1600,
            offset: 0.0,
            length: 360.0,
        },
    )));
    provide_context(interface);

    {
        let owner = Owner::current();
        view! {
            <Root default_theme=LeptonicTheme::default()>
                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                    <Mandelbrot
                        size=Signal::derive(move || (get_height.get(), get_height.get()))
                    />
                    <Tabs mount=Mount::Once>
                        <Tab name="dapp" label="DApp".into_view()>
                            <EthereumContextProvider>
                                <ConnectButton/>
                                <AccountLabel/>
                                <Separator/>
                                <Blockchain/>
                            </EthereumContextProvider>
                        </Tab>
                        <Tab name="description" label="Description".into_view()>
                            <About/>
                        </Tab>
                        <Tab name="how_to_use" label="How to Use".into_view()>
                            <Guide/>
                        </Tab>
                    </Tabs>
                </Stack>
            </Root>
        }
    }
}
