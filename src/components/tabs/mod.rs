mod about;
mod explorer;
mod guide;
mod inventory;
mod sales;

use leptos::prelude::*;
use leptos_ethereum_provider::EthereumInterface;
use leptos_router::{hooks::use_params, params::Params};
use send_wrapper::SendWrapper;

use crate::context::{Context, StateStoreFields};

use {
    about::About,
    explorer::Explorer,
    guide::Guide,
    inventory::Inventory,
    sales::Sales,
};

#[derive(Clone, Params, PartialEq)]
struct ControllerParams {
    token_id: Option<u128>,
}

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
pub fn Tabs() -> impl IntoView {
    let ethereum = use_context::<Option<EthereumInterface>>().unwrap();
    let params = use_params::<ControllerParams>();
    let context = use_context::<SendWrapper<Context>>().unwrap();
    let selected_tab = RwSignal::new("explorer");

    Effect::new({
        let context = context.clone();
        move || {
            context
                .state
                .current_token_id()
                .set(params.get().ok().and_then(|params| params.token_id))
        }
    });

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
