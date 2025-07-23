mod auction;
mod bids;
mod gradient;
mod info;
mod visuals;

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::context::{Context, ExplorerStoreFields, StateStoreFields};
use {auction::Auction, bids::Bids, info::Info, visuals::Visuals};

#[component]
pub fn Explorer() -> impl IntoView {
    let context = use_context::<SendWrapper<Context>>().unwrap();

    view! {
        <div class="flex flex-col">
            <Visuals/>

            {
                move || context.state.explorer().nav_history().get().last().cloned().map(|token| {
                    let context = context.clone();
                    view! {
                        <div class="bg-gray-800 text-white rounded-md shadow p-4">
                            <Info token=token.clone() />
                        </div>

                        <Show when={let context = context.clone(); move || context.state.address().get().is_some()} fallback=|| {} >
                            {
                                let token = token.clone();
                                view! {
                                    <div class="border-t border-gray-700 my-4" />
                                    <div class="bg-gray-800 text-white rounded-md shadow p-4">
                                        <Auction token />
                                    </div>
                                }
                            }
                        </Show>

                        <div class="border-t border-gray-700 my-4" />
                        <Show when={let context = context.clone(); move || context.state.explorer().bids().get().len() > 0} fallback=|| {} >
                            <div class="bg-gray-800 text-white rounded-md shadow p-4">
                                <Bids bids=context.state.explorer().bids() />
                            </div>
                        </Show>
                    }.into_any()
                })
            }
        </div>
    }
}
