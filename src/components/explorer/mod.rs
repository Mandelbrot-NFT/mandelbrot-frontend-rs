mod auction;
mod bids;
mod gradient;
mod info;
mod visuals;

use std::sync::Arc;

use leptos::{prelude::*, task::spawn_local};
use leptos_router::{
    components::{Route, Routes},
    hooks::{use_navigate, use_params, use_query, use_query_map},
    params::Params,
    path,
};
use mandelbrot_explorer::Focus;
use send_wrapper::SendWrapper;

use crate::{
    context::{Context, ExplorerStoreFields, SalesStoreFields, StateStoreFields},
    util::preserve_log_level,
};
use {auction::Auction, bids::Bids, info::Info, visuals::Visuals};

#[component]
pub fn Explorer() -> impl IntoView {
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

#[derive(Clone, Debug, Params, PartialEq)]
struct FocusQuery {
    focus: Option<Focus>,
}

#[component]
fn Controller() -> impl IntoView {
    let context = use_context::<SendWrapper<Context>>().unwrap();
    let navigate = use_navigate();
    let query_map = use_query_map();
    let params = use_params::<ControllerParams>();
    let mut focus = use_query::<FocusQuery>().get_untracked().unwrap().focus;

    Effect::new({
        let context = context.clone();
        move || context.state.current_token_id().set(params.get().ok().and_then(|params| params.token_id))
    });

    // query tokens and bids
    Effect::new({
        let context = context.clone();
        let navigate = navigate.clone();
        move || {
            let context = context.clone();
            let navigate = navigate.clone();
            let token_id = context.state.current_token_id().get().unwrap_or(1);
            spawn_local(async move {
                if let (Ok(tokens), Ok(children), Ok(bids)) = (
                    context.erc1155_contract.get_ancestry_metadata(token_id).await,
                    context.erc1155_contract.get_children_metadata(token_id).await,
                    context.erc1155_contract.get_bids(token_id).await,
                ) {
                    context.state.explorer().nav_history().update(|nav_history| {
                        nav_history.clear();
                        nav_history.extend(tokens.into_iter().rev());
                    });
                    context.state.explorer().children().update(|children_| {
                        children_.clear();
                        children_.extend(children.into_iter().map(|m| (m.token_id, m)));
                    });
                    context.state.explorer().bids().update(|bids_| {
                        bids_.clear();
                        bids_.extend(bids.into_iter().map(|bid| (bid.token_id, bid)));
                    });
                } else {
                    navigate(&preserve_log_level("/tokens/1".into(), query_map), Default::default());
                }
            });
        }
    });

    // zoom, but only on first page load
    let first = StoredValue::new(true);
    Effect::new({
        let context = context.clone();
        move || {
            if first.get_value() {
                if let Some(focus) = focus.take() {
                    first.set_value(false);
                    context.mandelbrot.lock().unwrap().move_into_focus(focus.clone());
                } else {
                    context.state.explorer().nav_history().with(|nav_history| {
                        if let Some(token) = nav_history.last() {
                            first.set_value(false);
                            context
                                .mandelbrot
                                .lock()
                                .unwrap()
                                .move_into_bounds(&token.to_frame(mandelbrot_explorer::FrameColor::Blue).bounds);
                        }
                    });
                }
            }
        }
    });

    let select_bid = {
        let context = context.clone();
        move |bid_id, selected| {
            context.state.explorer().bids().update(|bids| {
                if let Some(bid) = bids.get_mut(&bid_id) {
                    bid.selected = selected;
                }
            });
            context.state.sales().bids().update(|bids| {
                for token_bids in bids.values_mut() {
                    for bid in token_bids.values_mut() {
                        if bid.token_id == bid_id {
                            bid.selected = selected;
                        }
                    }
                }
            });
        }
    };

    let on_frame_event = Arc::new({
        let context = context.clone();
        move |frame_event: mandelbrot_explorer::FrameEvent| match frame_event {
            mandelbrot_explorer::FrameEvent::DoubleClicked(frame) => match frame.color {
                mandelbrot_explorer::FrameColor::Red
                | mandelbrot_explorer::FrameColor::Pink
                | mandelbrot_explorer::FrameColor::Blue
                | mandelbrot_explorer::FrameColor::LightBlue => {
                    context.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds);
                    navigate(
                        &preserve_log_level(format!("/tokens/{}", frame.id), query_map),
                        Default::default(),
                    );
                }
                mandelbrot_explorer::FrameColor::Yellow | mandelbrot_explorer::FrameColor::Lemon => {
                    select_bid(frame.id, true);
                }
                mandelbrot_explorer::FrameColor::Green => {
                    select_bid(frame.id, false);
                }
            },
            mandelbrot_explorer::FrameEvent::Entered(frame) => match frame.color {
                mandelbrot_explorer::FrameColor::Red | mandelbrot_explorer::FrameColor::Pink => {
                    navigate(
                        &preserve_log_level(format!("/tokens/{}", frame.id), query_map),
                        Default::default(),
                    );
                }
                mandelbrot_explorer::FrameColor::Blue | mandelbrot_explorer::FrameColor::LightBlue => {
                    if Some(frame.id) == context.state.current_token_id().get_untracked() {
                        navigate(
                            &preserve_log_level(format!("/tokens/{}", frame.id), query_map),
                            Default::default(),
                        );
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });

    context.mandelbrot.lock().unwrap().on_frame_event = Some(Arc::new({
        let on_frame_event = on_frame_event.clone();
        move |frame_event| on_frame_event(frame_event)
    }));

    // check ownership
    Effect::new({
        let context = context.clone();
        move || {
            if let Some(address) = context.state.address().get() {
                context.state.explorer().children().update(|children| {
                    children
                        .values_mut()
                        .for_each(|token| token.owned = token.owner == address)
                });
                context
                    .state
                    .explorer()
                    .bids()
                    .update(|bids| bids.values_mut().for_each(|bid| bid.owned = bid.owner == address));
                context.state.explorer().nav_history().update(|nav_history| {
                    nav_history
                        .iter_mut()
                        .for_each(|token| token.owned = token.owner == address)
                });
            }
        }
    });

    // update frames
    Effect::new({
        let context = context.clone();
        move || {
            let mandelbrot = &mut context.mandelbrot.lock().unwrap();
            let frames = &mut mandelbrot.frames;
            frames.clear();
            frames.extend(
                context
                    .state
                    .explorer()
                    .children()
                    .get()
                    .values()
                    .map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Red)),
            );
            frames.extend(
                context
                    .state
                    .explorer()
                    .bids()
                    .get()
                    .values()
                    .map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Yellow)),
            );
            frames.extend(
                context
                    .state
                    .explorer()
                    .nav_history()
                    .get()
                    .iter()
                    .rev()
                    .map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Blue)),
            );
            if let Some(redraw) = &mandelbrot.redraw {
                redraw();
            }
        }
    });

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
