use std::collections::HashMap;

use leptos::prelude::*;
use mandelbrot_explorer::FrameColor;
use send_wrapper::SendWrapper;

use crate::{evm::types::Metadata, state::State};

#[component]
pub fn Bids<T>(bids: T) -> impl IntoView
where
    T: Get<Value = HashMap<u128, Metadata>> + Update<Value = HashMap<u128, Metadata>> + Copy + Send + Sync + 'static,
{
    let state = use_context::<SendWrapper<State>>().unwrap();

    let delete_bid = Action::new_local({
        let state = state.clone();
        move |bid_id: &u128| {
            let state = state.clone();
            let bid_id = bid_id.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    if let Some(_) = state.erc1155_contract.delete_bid(address, bid_id).await {
                        bids.update(|bids| {
                            bids.remove(&bid_id);
                        });
                    }
                }
            }
        }
    });

    let zoom_bid = {
        move |bid_id| {
            if let Some(bid) = bids.get().get(&bid_id) {
                let frame = bid.to_frame(FrameColor::Blue);
                state.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
            }
        }
    };

    view! {
        <Show when=move || {bids.get().len() > 0} fallback=|| {}>
            {
                let zoom_bid = zoom_bid.clone();
                view! {
                    <div id="content" class="p-4 space-y-4">
                        <For
                            each=move || bids.get().into_values()
                            key=|bid| bid.token_id
                            children=move |bid| view! {
                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-4 rounded-md border border-gray-700 bg-gray-900/50">
                                    // <!-- Bid Info -->
                                    <div class="text-sm text-white">
                                        <div class="font-semibold">"Bid Id: " {bid.token_id}</div>
                                        <div class="text-accent2">"Proposed OM: " {bid.locked_tokens.to_string()}</div>
                                    </div>

                                    // <!-- Actions -->
                                    <div class="flex flex-wrap gap-2">
                                        <button
                                            on:click={let zoom_bid = zoom_bid.clone(); move |_| zoom_bid(bid.token_id)}
                                            class="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-medium transition"
                                        >
                                            "Zoom"
                                        </button>
                                        <button
                                            on:click=move |_| { delete_bid.dispatch(bid.token_id); }
                                            class="px-3 py-1 bg-red-600 hover:bg-red-500 rounded-md text-white text-sm font-medium transition"
                                        >
                                            "Delete"
                                        </button>
                                    </div>
                                </div>
                            }
                        />
                    </div>
                }
            }
        </Show>
    }
}
