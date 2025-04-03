use std::collections::HashMap;

use leptos::*;
use leptos_router::use_navigate;
use mandelbrot_explorer::FrameColor;

use crate::{state::State, util::preserve_log_level, evm::types::Metadata};


#[component]
pub fn Sales() -> impl IntoView {
    let state = use_context::<State>().unwrap();

    let refresh = create_action({
        let state = state.clone();
        move |_| {
            let state = state.clone();
            async move {
                state.reload_inventory().await;
            }
        }
    });

    let zoom_token = {
        let mandelbrot = state.mandelbrot.clone();
        move |token_id| {
            if let Some(token) = state.inventory.tokens.get().get(&token_id) {
                use_navigate()(&preserve_log_level(format!("/tokens/{}", token_id)), Default::default());
                let frame = token.to_frame(FrameColor::Blue);
                mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
            }
        }
    };

    let zoom_bid = {
        move |token_id, bid_id| {
            let bids = state.sales.bids.get().get(&token_id).unwrap_or(&HashMap::new()).clone();
            if let Some(bid) = bids.get(&bid_id) {
                let frame = bid.to_frame(FrameColor::Blue);
                state.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
            }
        }
    };

    let toggle_bid = {
        move |token_id, bid_id, state_| {
            state.sales.bids.update(|bids| {
                if let Some(bids) = bids.get_mut(&token_id) {
                    if let Some(bid) = bids.get_mut(&bid_id) {
                        bid.selected = state_;
                    }
                }
            });
            state.explorer.bids.update(|bids| {
                if let Some(bid) = bids.get_mut(&bid_id) {
                    bid.selected = state_;
                }
            });
        }
    };

    let selected_bids = Signal::derive(move || {
        state.sales.bids
            .get()
            .values()
            .map(|bids| bids.values())
            .flatten()
            .filter(|bid| bid.selected)
            .map(|bid| bid.clone())
            .collect::<Vec<_>>()
    });

    let total_approve_amount = move || 0f64.max(selected_bids.get().iter().map(|bid| bid.locked_OM).sum::<f64>());

    let approve_bids = create_action({
        let erc1155_contract = state.erc1155_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    let selected_bids: Vec<u128> = selected_bids.get_untracked().iter().map(|bid| bid.token_id).collect();
                    erc1155_contract.batch_approve_bids(address, &selected_bids).await;
                }
            }
        }
    });

    view! {
        <div class="space-y-4">
            // <!-- Collapsible token list -->
            <div class="flex flex-col gap-3">
                <For
                    each=move || state.inventory.tokens.get().into_values()
                    key=|token| token.token_id
                    children=move |token| {
                        let zoom_token = zoom_token.clone();
                        let zoom_bid = zoom_bid.clone();
    
                        view! {
                            <details class="group border border-gray-700 rounded-md overflow-hidden bg-gray-800 text-white">
                                <summary class="flex items-center justify-between px-4 py-2 cursor-pointer select-none hover:bg-gray-700">
                                    <div class="flex items-center gap-3">
                                        <button
                                            on:click={let zoom_token = zoom_token.clone(); move |_| zoom_token(token.token_id)}
                                            class="px-2 py-1 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded-md transition"
                                        >
                                            "Zoom"
                                        </button>
                                        <span class="text-sm font-semibold">{"Token ID: "}{token.token_id}</span>
                                    </div>
                                </summary>
    
                                <div class="px-4 py-2 space-y-2 bg-gray-900">
                                    {
                                        move || {
                                            let zoom_bid = zoom_bid.clone();
                                            let bids = move || state.sales.bids.get().get(&token.token_id).unwrap_or(&HashMap::new()).clone();
                                            let sorted_bids = move || {
                                                let mut bids: Vec<Metadata> = bids().values().map(|bid| bid.clone()).collect();
                                                bids.sort_by(|a, b| b.locked_OM.partial_cmp(&a.locked_OM).unwrap());
                                                bids
                                            };
    
                                            view! {
                                                <For
                                                    each=move || sorted_bids()
                                                    key=|bid| bid.token_id
                                                    children=move |bid| {
                                                        view! {
                                                            <div class="flex items-center justify-between gap-4 bg-gray-800 rounded px-3 py-2">
                                                                <div class="flex items-center gap-3">
                                                                    {
                                                                        move || view! {
                                                                            <input
                                                                                type="checkbox"
                                                                                checked=bids()[&bid.token_id].selected
                                                                                on:change=move |ev| {
                                                                                    toggle_bid(
                                                                                        token.token_id,
                                                                                        bid.token_id,
                                                                                        event_target_checked(&ev),
                                                                                    );
                                                                                }
                                                                                class="accent-accent1 w-4 h-4"
                                                                            />
                                                                        }
                                                                    }
                                                                    <span class="text-sm font-mono">
                                                                        {format!("{} {:?}", bid.locked_OM.to_string(), bid.owner)}
                                                                    </span>
                                                                </div>
                                                                <button
                                                                    on:click={let zoom_bid = zoom_bid.clone(); move |_| zoom_bid(token.token_id, bid.token_id)}
                                                                    class="text-sm px-2 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white transition"
                                                                >
                                                                    "Zoom"
                                                                </button>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            }
                                        }
                                    }
                                </div>
                            </details>
                        }
                    }
                />
            </div>
    
            // <!-- Total approve + button -->
            <div class="flex items-center justify-between bg-gray-900 text-white p-4 rounded-md">
                <div class="flex flex-col">
                    <span class="text-xs text-gray-400">"Total OM"</span>
                    <span class="text-sm font-mono">{move || total_approve_amount()}</span>
                </div>
            
                <button
                    on:click=move |_| approve_bids.dispatch(())
                    class="px-4 py-2 bg-green-600 hover:bg-green-500 rounded-md text-sm font-semibold transition"
                >
                    "Approve"
                </button>
            </div>
    
            // <!-- Refresh Button -->
            <button
                on:click=move |_| refresh.dispatch(())
                class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-md text-sm font-semibold transition"
            >
                "Refresh"
            </button>
        </div>
    }
}
