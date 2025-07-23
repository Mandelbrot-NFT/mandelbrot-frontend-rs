mod bids;
mod form;
mod info;

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Routes},
    hooks::use_params,
    params::Params,
    path,
};
use send_wrapper::SendWrapper;

use crate::context::{Context, ExplorerStoreFields, StateStoreFields};
use {bids::Bids, form::Form, info::Info};

#[component]
pub fn Auction() -> impl IntoView {
    view! {
        <Routes fallback=|| "Not found.">
            <Route path=path!("/tokens/:token_id") view=move || view! { <Controller/> }/>
            <Route path=path!("*") view=move || view! { <Controller/> }/>
        </Routes>
    }
}

#[derive(Clone, Params, PartialEq)]
struct ControllerParams {
    token_id: Option<u128>,
}

#[component]
fn Controller() -> impl IntoView {
    let context = use_context::<SendWrapper<Context>>().unwrap();
    let params = use_params::<ControllerParams>();

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
        <div class="flex flex-col">
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
                                        <Form token />
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
