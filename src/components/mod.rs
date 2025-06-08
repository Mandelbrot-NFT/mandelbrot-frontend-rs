mod about;
mod account;
mod explorer;
mod guide;
mod inventory;
mod mandelbrot;
mod primitive;
mod sales;
mod state;

use std::{
    cell::RefCell,
    f32::consts::PI,
    rc::Rc,
    sync::{Arc, Mutex},
};

use leptos::prelude::*;
use leptos_ethereum_provider::{ConnectButton, EthereumContextProvider, EthereumInterface};
use leptos_router::components::Router;

use {
    about::About,
    account::{Account, AccountButton},
    explorer::Explorer,
    guide::Guide,
    inventory::Inventory,
    mandelbrot::Mandelbrot,
    sales::Sales,
    state::StateContextProvider,
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

    ethereum.map(|ethereum| {
        view! {
            <div class="h-[8vh] flex space-x-2 border-b">
                {
                    move || {
                        vec![
                            ("explorer", "Explore", true),
                            ("inventory", "Inventory", ethereum.connected()),
                            ("sales", "Sales", ethereum.connected()),
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

            <Router>
                <div class="w-full mx-auto overflow-y-auto max-h-[84vh] scroll-smooth">
                    <div class="p-4 space-y-4">
                        <div class=move || if selected_tab.get() == "explorer" { "block" } else { "hidden" }>
                            <Explorer />
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
            </Router>
        }
    })
}

#[component]
pub fn App() -> impl IntoView {
    let window = web_sys::window().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap() + 1.0;

    let interface = LocalStorage::wrap(Arc::new(Mutex::new(mandelbrot_explorer::Interface::new(
        Rc::new(RefCell::new(
            mandelbrot_explorer::Perturbation::new(height as u32, height as u32).into(),
            // mandelbrot_explorer::Optimised::new(height as u32, height as u32).into(),
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
    let OM_balance = RwSignal::new(0.0);

    view! {
        <div class="min-h-screen flex flex-col">
            <EthereumContextProvider>
                <StateContextProvider mandelbrot=interface.clone()>
                    <div class="flex flex-row gap-2 items-stretch">
                        <Mandelbrot interface=interface.clone()/>
                        <div class="relative w-full border overflow-auto">
                            <header class="h-[8vh] z-10 bg-brand text-white flex items-center justify-between px-4">
                                <h3 class="text-lg font-bold">"Mandelbrot NFT"</h3>
                                <div class="flex items-center gap-4">
                                    <ConnectButton connected_html=move || view! {
                                        <AccountButton
                                            balance=OM_balance.read_only()
                                            on_click=move || account_open.update(|account_open| {
                                                *account_open = !*account_open;
                                            })
                                        />
                                    }/>
                                </div>
                            </header>
                            <Content/>
                        </div>
                    </div>
                    <Account OM_balance open=account_open/>
                </StateContextProvider>
            </EthereumContextProvider>
        </div>
    }
}
