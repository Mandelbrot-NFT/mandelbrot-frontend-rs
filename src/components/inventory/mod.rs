mod bids;
mod tokens;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::context::{Context, InventoryStateStoreFields, StateStoreFields};
use bids::Bids;
use tokens::Tokens;

#[component]
pub fn Inventory() -> impl IntoView {
    let context = use_context::<SendWrapper<Context>>().unwrap();

    let refresh = Action::new_local({
        let context = context.clone();
        move |_| {
            let context = context.clone();
            async move {
                context.reload_inventory().await;
            }
        }
    });

    view! {
        <div class="space-y-4">
            <p class="text-lg font-semibold text-gray-700">"Tokens:"</p>
            <Tokens tokens=context.state.inventory().tokens() />

            <p class="text-lg font-semibold text-gray-700">"Bids:"</p>
            <Bids bids=context.state.inventory().bids() />

            <button
                class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition"
                on:click=move |_| { refresh.dispatch(()); }
            >
                "Refresh"
            </button>
        </div>
    }
}
