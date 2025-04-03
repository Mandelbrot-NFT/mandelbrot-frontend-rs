mod about;
mod account;
mod explorer;
mod guide;
mod inventory;
mod mandelbrot;
mod primitive;
mod sales;
mod state;

use std::{sync::{Arc, Mutex}, rc::Rc, cell::RefCell};

use leptos::*;
use leptos_ethereum_provider::{ConnectButton, EthereumContextProvider};
use leptos_router::Router;
use mandelbrot_explorer::ISample;

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
pub fn App() -> impl IntoView {
    let window = web_sys::window().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap() + 1.0;

    let interface = Arc::new(Mutex::new(mandelbrot_explorer::Interface::new(
        Rc::new(RefCell::new(mandelbrot_explorer::PerturbationEngine::new(height as u32, height as u32))),
        mandelbrot_explorer::Coloring {
            max_iterations: 1600,
            offset: 0.0,
            length: 360.0,
        },
    )));

    let account_open = create_rw_signal(false);
    let OM_balance = create_rw_signal(0.0);

    let selected_tab = create_rw_signal("explorer");

    view! {
        <div class="min-h-screen flex flex-col">
            <EthereumContextProvider>
                <StateContextProvider mandelbrot=interface.clone()>
                    <div class="flex flex-row gap-2 items-stretch">
                        <Mandelbrot interface=interface.clone()/>
                        <div class="relative w-full border overflow-auto">
                            <header class="h-12 z-10 bg-brand text-white flex items-center justify-between px-4">
                                <h3 class="text-lg font-bold">"Mandelbrot NFT"</h3>
                                <div class="flex items-center gap-4">
                                    <ConnectButton connected_html=view! {
                                        <AccountButton
                                            balance=OM_balance.read_only()
                                            on_click=move |_| account_open.update(|account_open| {
                                                *account_open = !*account_open;
                                            })
                                        />
                                    }.into_view()/>
                                </div>
                            </header>

                            // Tab buttons
                            <div class="flex space-x-2 border-b">
                                {
                                    vec![
                                        ("explorer", "Explore"),
                                        ("inventory", "Inventory"),
                                        ("sales", "Sales"),
                                        ("description", "Description"),
                                        ("how_to_use", "How to Use"),
                                    ]
                                        .into_iter()
                                        .map(|(name, label)| {
                                            view! {
                                                <button
                                                    class=move || tab_class(name, selected_tab.get())
                                                    on:click=move |_| selected_tab.set(name)
                                                >
                                                    {label}
                                                </button>
                                            }
                                        })
                                        .collect_view()
                                }
                            </div>
                            <Router>
                                <div class="w-full max-w-4xl mx-auto">
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

                        </div>
                    </div>
                    <Account OM_balance open=account_open/>
                </StateContextProvider>
            </EthereumContextProvider>
        </div>
    }
}
