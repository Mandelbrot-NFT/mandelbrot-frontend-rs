mod about;
mod account;
mod explorer;
mod guide;
mod inventory;
mod mandelbrot;
mod sales;
mod state;

use std::{sync::{Arc, Mutex}, rc::Rc, cell::RefCell};

use leptonic::prelude::*;
use leptos::*;
use leptos_ethereum_provider::{ConnectButton, EthereumContextProvider};
use mandelbrot_explorer::ISample;
use wasm_bindgen::JsCast;

use {
    about::About,
    account::{Account, AccountButton},
    state::StateContextProvider,
    explorer::Explorer,
    guide::Guide,
    inventory::Inventory,
    mandelbrot::Mandelbrot,
    sales::Sales,
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
    
    let account_open = create_rw_signal(false);
    let fuel_balance = create_rw_signal(0.0);

    view! {
        <Root default_theme=LeptonicTheme::default()>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6) style="align-items: stretch;">
                <Mandelbrot
                    interface=interface.clone()
                    size=Signal::derive(move || (get_height.get(), get_height.get()))
                />
                <EthereumContextProvider>
                    <StateContextProvider mandelbrot=interface.clone()>
                        <Box style="position: relative; border: width: 100%; overflow: auto;">
                            <AppBar height=Size::Em(3.0) style="z-index: 1; background: var(--brand-color); color: white;">
                                <H3 style="margin-left: 1em; color: white;">"Mandelbrot NFT"</H3>
                                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(1.0) style="margin-right: 1em">
                                    <ConnectButton connected_html=view! {
                                        <AccountButton
                                            balance=fuel_balance.read_only()
                                            on_click=move |_| account_open.update(|account_open| {
                                                *account_open = !*account_open;
                                            })
                                        />
                                    }/>
                                </Stack>
                            </AppBar>
                            <Tabs mount=Mount::Once>
                                <Tab name="dapp" label="Explore".into_view()>
                                    <Explorer/>
                                </Tab>
                                <Tab name="inventory" label="Inventory".into_view()>
                                    <Inventory/>
                                </Tab>
                                <Tab name="sales" label="Sales".into_view()>
                                    <Sales/>
                                </Tab>
                                <Tab name="description" label="Description".into_view()>
                                    <About/>
                                </Tab>
                                <Tab name="how_to_use" label="How to Use".into_view()>
                                    <Guide/>
                                </Tab>
                            </Tabs>
                            <Account fuel_balance open=account_open/>
                        </Box>
                    </StateContextProvider>
                </EthereumContextProvider>
            </Stack>
        </Root>
    }
}
