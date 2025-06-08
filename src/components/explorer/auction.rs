use leptos::prelude::*;
use mandelbrot_explorer::ISample;
use send_wrapper::SendWrapper;

use crate::{
    evm::types::{Field, Metadata},
    state::State,
};

#[component]
pub fn Auction(token: Metadata) -> impl IntoView {
    let state = use_context::<SendWrapper<State>>().unwrap();

    let bid_amount = RwSignal::new(token.minimum_price);
    let bids_minimum_price = RwSignal::new(token.minimum_price);

    let create_bid = Action::new_local({
        move |&token_id| {
            let state = state.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    let bounds = state.mandelbrot.lock().unwrap().engine.borrow().get_bounds();
                    state
                        .erc1155_contract
                        .bid(
                            address,
                            token_id,
                            Field {
                                x_min: bounds.x_min,
                                y_min: bounds.y_min,
                                x_max: bounds.x_max,
                                y_max: bounds.y_max,
                            },
                            bid_amount.get_untracked(),
                            bids_minimum_price.get_untracked(),
                        )
                        .await;
                };
            }
        }
    });

    view! {
        <div class="flex items-start gap-4">
            <div class="flex flex-col gap-4">
                <div class="flex items-center gap-3">
                    <label class="text-sm text-white">"Bid amount:"</label>
                    <input
                        type="number"
                        min={token.minimum_price}
                        placeholder="Bid amount"
                        prop:value=bid_amount
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                bid_amount.set(v);
                            }
                        }
                        class="w-40 px-2 py-1 bg-gray-800 text-white rounded-md border border-gray-600 focus:outline-none focus:ring-2 focus:ring-accent1"
                    />
                </div>

                <div class="flex items-center gap-3">
                    <label class="text-sm text-white">"Minimum bid price:"</label>
                    <input
                        type="number"
                        min={token.minimum_price}
                        placeholder="Minimum bid price"
                        prop:value=bids_minimum_price
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                bids_minimum_price.set(v);
                            }
                        }
                        class="w-40 px-2 py-1 bg-gray-800 text-white rounded-md border border-gray-600 focus:outline-none focus:ring-2 focus:ring-accent1"
                    />
                </div>
            </div>

            <button
                on:click=move |_| { create_bid.dispatch(token.token_id); }
                class="h-fit px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-md font-semibold transition"
            >
                "Bid"
            </button>
        </div>
    }
}
