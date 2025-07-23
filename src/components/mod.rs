mod about;
mod account;
mod context;
mod error_handler;
mod explorer;
mod frame_control;
mod guide;
mod inventory;
mod mandelbrot;
mod primitive;
mod sales;

use std::{
    cell::RefCell,
    // f32::consts::PI,
    rc::Rc,
    sync::{Arc, Mutex},
};

use leptos::prelude::*;
use leptos_ethereum_provider::{ConnectButton, EthereumContextProvider, EthereumInterface};
use leptos_router::hooks::use_query_map;
use reactive_stores::Store;

use crate::{context::StateStoreFields, util::preserve_log_level};
use frame_control::FrameControl;

use {
    about::About,
    account::{Account, AccountButton},
    context::ContextProvider,
    explorer::Explorer,
    guide::Guide,
    inventory::Inventory,
    mandelbrot::Mandelbrot,
    sales::Sales,
};

fn tab_class(tab_name: &str, selected_tab: &str) -> String {
    if tab_name == selected_tab {
        // Active tab styling
        "px-4 py-2 font-medium border-b-2 border-blue-600 text-blue-600".to_string()
    } else {
        // Inactive tab styling
        "px-4 py-2 font-medium text-gray-500 hover:text-blue-500 transition-colors".to_string()
    }
}

#[component]
pub fn Content() -> impl IntoView {
    let ethereum = use_context::<Option<EthereumInterface>>().unwrap();
    let selected_tab = RwSignal::new("explorer");

    view! {
        <div class="h-[8vh] flex space-x-2 border-b">
            {
                move || {
                    vec![
                        ("explorer", "Explore", true),
                        ("inventory", "Inventory", ethereum.as_ref().is_some_and(|eth| eth.connected())),
                        ("sales", "Sales", ethereum.as_ref().is_some_and(|eth| eth.connected())),
                        ("description", "Description", true),
                        ("how_to_use", "How to Use", true),
                    ]
                        .into_iter()
                        .filter_map(|(name, label, show)| {
                            show.then(|| view! {
                                <button
                                    class=move || tab_class(name, selected_tab.get())
                                    on:click=move |_| selected_tab.set(name)
                                >
                                    {label}
                                </button>
                            })
                        })
                        .collect_view()
                }
            }
        </div>

        <div class="w-full mx-auto overflow-y-auto max-h-[84vh] scroll-smooth">
            <div class="p-4 space-y-4">
                <div class=move || if selected_tab.get() == "explorer" { "block" } else { "hidden" }>
                    <Explorer/>
                </div>
                <div class=move || if selected_tab.get() == "inventory" { "block" } else { "hidden" }>
                    <Inventory />
                </div>
                <div class=move || if selected_tab.get() == "sales" { "block" } else { "hidden" }>
                    <Sales />
                </div>
                <div class=move || if selected_tab.get() == "description" { "block" } else { "hidden" }>
                    <About />
                </div>
                <div class=move || if selected_tab.get() == "how_to_use" { "block" } else { "hidden" }>
                    <Guide />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let query_map = use_query_map();
    let window = web_sys::window().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap() + 1.0;
    let state = Store::default();

    let on_focus_change = {
        move |focus| {
            let url = if let Some(token_id) = state.current_token_id().get_untracked() {
                preserve_log_level(format!("/tokens/{}?focus={}", token_id, focus), query_map)
            } else {
                preserve_log_level(format!("?focus={}", focus), query_map)
            };
            if let Ok(history) = window.history() {
                let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
            }
        }
    };

    let interface = LocalStorage::wrap(Arc::new(Mutex::new(mandelbrot_explorer::Interface::new(
        Rc::new(RefCell::new(
            mandelbrot_explorer::Perturbation::new(height as u32, height as u32, on_focus_change).into(),
            // mandelbrot_explorer::Optimised::new(height as u32, height as u32, on_focus_change).into(),
        )),
        mandelbrot_explorer::Palette {
            gradient: mandelbrot_explorer::Gradient::Wave(mandelbrot_explorer::WaveGradient {
                // neon
                red: mandelbrot_explorer::Wave::new(0.5, 0.7, 12.0, 0.0),
                green: mandelbrot_explorer::Wave::new(0.5, 0.7, 10.0, 1.5),
                blue: mandelbrot_explorer::Wave::new(0.5, 0.7, 8.0, 3.0),
                // red velvet
                // red: mandelbrot_explorer::Wave::new(0.337, 0.662, 6.28, 0.0),
                // green: mandelbrot_explorer::Wave::new(0.245, 0.586, 6.28, 0.0),
                // blue: mandelbrot_explorer::Wave::new(0.334, -0.343, 6.28, 0.0),

                // evil rainbow
                // red: mandelbrot_explorer::Wave::new(0.4, 0.6, 7.2, 1.2),
                // green: mandelbrot_explorer::Wave::new(0.4, 0.6, 5.9, -1.6),
                // blue: mandelbrot_explorer::Wave::new(0.4, 0.6, 3.8, 2.1),

                // winter sunrise
                // red: mandelbrot_explorer::Wave::new(1.0, 0.5, 6.28, 0.9),
                // green: mandelbrot_explorer::Wave::new(1.0, 0.5, 5.88, -PI),
                // blue: mandelbrot_explorer::Wave::new(1.0, 0.5, PI, -3.64)
            }),
            max_iterations: 1600,
            offset: 0.0,
            length: 360.0,
        },
    ))));

    let account_open = RwSignal::new(false);
    let token_balance = RwSignal::new(0.0);

    view! {
        <div class="min-h-screen flex flex-col">
            <div class="flex flex-row items-stretch">
                <Mandelbrot interface=interface.clone()/>
                <EthereumContextProvider>
                    <ContextProvider mandelbrot=interface.clone() state>
                        <FrameControl>
                            <div class="relative w-full overflow-auto">
                                <header class="h-[8vh] z-10 bg-brand text-white flex items-center justify-between px-4">
                                    <h3 class="text-lg font-bold">"Mandelbrot NFT"</h3>
                                    <ConnectButton connected_html=move || view! {
                                        <AccountButton
                                            balance=token_balance.read_only()
                                            on_click=move || account_open.update(|account_open| {
                                                *account_open = !*account_open;
                                            })
                                        />
                                    }/>
                                </header>
                                <Content/>
                            </div>
                            <Account token_balance open=account_open/>
                        </FrameControl>
                    </ContextProvider>
                </EthereumContextProvider>
            </div>
        </div>
    }
}
