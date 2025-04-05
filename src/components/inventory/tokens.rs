use std::collections::HashMap;

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};
use mandelbrot_explorer::FrameColor;
use send_wrapper::SendWrapper;

use crate::{evm::types::Metadata, state::State, util::preserve_log_level};

#[component]
pub fn Tokens<T>(tokens: T) -> impl IntoView
where
    T: Get<Value = HashMap<u128, Metadata>> + Update<Value = HashMap<u128, Metadata>> + Copy + Send + Sync + 'static,
{
    let state = use_context::<SendWrapper<State>>().unwrap();
    let navigate = use_navigate();
    let query_map = use_query_map();

    let burn_token = Action::new_local({
        let state = state.clone();
        move |token_id: &u128| {
            let state = state.clone();
            let token_id = token_id.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    if let Some(_) = state.erc1155_contract.burn(address, token_id).await {
                        tokens.update(|tokens| {
                            tokens.remove(&token_id);
                        });
                    }
                }
            }
        }
    });

    let zoom_token = {
        let state = state.clone();
        move |token_id| {
            if let Some(token) = tokens.get().get(&token_id) {
                navigate(
                    &preserve_log_level(format!("/tokens/{}", token_id), query_map),
                    Default::default(),
                );
                let frame = token.to_frame(FrameColor::Blue);
                state.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
            }
        }
    };

    let edited_token = RwSignal::new(None);
    let bids_minimum_price = RwSignal::new(0.0);
    let edit_token = move |token: Metadata| {
        bids_minimum_price.set(token.minimum_price);
        edited_token.set(Some(token))
    };
    let edit_token_submit = Action::new_local({
        let state = state.clone();
        move |_| {
            let state = state.clone();
            async move {
                if let (Some(address), Some(token)) = (state.address.get_untracked(), edited_token.get_untracked()) {
                    state
                        .erc1155_contract
                        .set_minimum_bid(address, token.token_id, bids_minimum_price.get_untracked())
                        .await;
                }
                edited_token.set(None);
            }
        }
    });

    view! {
        <Show when=move || {tokens.get().len() > 0} fallback=|| {}>
            {
                let zoom_token = zoom_token.clone();
                view! {
                    <div id="content" class="p-4 space-y-4">
                        <For
                            each=move || tokens.get().into_values()
                            key=|token| token.token_id
                            children=move |token| view! {
                                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-4 rounded-md border border-gray-700 bg-gray-900/50">
                                    <div class="text-sm text-white">
                                        <div class="font-semibold">"Token Id: " {token.token_id}</div>
                                        <div class="text-accent2">"Locked OM: " {token.locked_OM.to_string()}</div>
                                    </div>

                                    <div class="flex flex-wrap gap-2">
                                        <button
                                            on:click={let zoom_token = zoom_token.clone(); move |_| zoom_token(token.token_id)}
                                            class="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-medium transition"
                                        >
                                            "Zoom"
                                        </button>
                                        <button
                                            on:click={let token = token.clone(); move |_| edit_token(token.clone())}
                                            class="px-3 py-1 bg-yellow-600 hover:bg-yellow-500 rounded-md text-white text-sm font-medium transition"
                                        >
                                            "Edit"
                                        </button>
                                        <button
                                            on:click=move |_| { burn_token.dispatch(token.token_id); }
                                            class="px-3 py-1 bg-red-600 hover:bg-red-500 rounded-md text-white text-sm font-medium transition"
                                        >
                                            "Burn"
                                        </button>
                                    </div>
                                </div>
                            }
                        />
                    </div>
                }
            }
        </Show>

        // <!-- Modal -->
        <Show when=move || edited_token.get().is_some()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
                <div class="bg-gray-900 text-white p-6 rounded-lg shadow-lg w-full max-w-md space-y-6">
                    <div class="text-xl font-bold">
                        "Token Id " {move || edited_token.get().map_or("".into(), |token| token.token_id.to_string())}
                    </div>

                    {
                        move || {
                            edited_token.get().map(|token| {
                                view! {
                                    <div class="flex flex-col gap-2">
                                        <label class="text-sm text-gray-300">"Minimum bid price:"</label>
                                        <input
                                            type="number"
                                            min={token.minimum_price}
                                            class="bg-gray-800 text-white p-2 rounded-md w-full focus:outline-none focus:ring-2 focus:ring-accent1"
                                            prop:value={bids_minimum_price.get()}
                                            on:input=move |ev| {
                                                if let Ok(value) = event_target_value(&ev).parse::<f64>() {
                                                    bids_minimum_price.set(value);
                                                }
                                            }
                                        />
                                    </div>
                                }
                            })
                        }
                    }

                    <div class="flex justify-end gap-4 pt-2 border-t border-gray-700">
                        <button
                            on:click=move |_| { edit_token_submit.dispatch(()); }
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-md text-white font-semibold transition"
                        >
                            "Save"
                        </button>
                        <button
                            on:click=move |_| edited_token.set(None)
                            class="px-4 py-2 bg-gray-600 hover:bg-gray-500 rounded-md text-white font-semibold transition"
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
