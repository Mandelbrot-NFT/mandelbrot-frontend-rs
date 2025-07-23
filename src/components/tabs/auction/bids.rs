use std::collections::HashMap;

use leptos::prelude::*;
use mandelbrot_explorer::FrameColor;
use send_wrapper::SendWrapper;

use crate::{context::Context, evm::types::Metadata};

#[component]
pub fn Bids<T>(bids: T) -> impl IntoView
where
    T: Get<Value = HashMap<u128, Metadata>> + Copy + Send + Sync + 'static,
{
    let context = use_context::<SendWrapper<Context>>().unwrap();

    let zoom_bid = {
        move |bid_id| {
            if let Some(bid) = bids.get().get(&bid_id) {
                let frame = bid.to_frame(FrameColor::Blue);
                context.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
            }
        }
    };

    let sorted_bids = Memo::new(move |_| {
        let mut bids: Vec<Metadata> = bids.get().values().map(|bid| bid.clone()).collect();
        bids.sort_by(|bid_a, bid_b| bid_b.locked_tokens.partial_cmp(&bid_a.locked_tokens).unwrap());
        bids
    });

    view! {
        <div class="space-y-4">
            <p class="text-lg font-semibold text-white">"Bids:"</p>

            <div id="content" class="p-4 bg-gray-900 rounded-md space-y-2">
                <For
                    each=move || sorted_bids.get()
                    key=|bid| bid.token_id
                    children=move |bid| {
                        let zoom_bid = zoom_bid.clone();
                        view! {
                            <div class="flex items-center justify-between bg-gray-800 text-white rounded px-4 py-2">
                                <span class="text-sm font-mono">
                                    {format!("{} {:?}", bid.locked_tokens.to_string(), bid.owner)}
                                </span>
                                <button
                                    on:click=move |_| zoom_bid(bid.token_id)
                                    class="px-3 py-1 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-md transition"
                                >
                                    "Zoom"
                                </button>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
