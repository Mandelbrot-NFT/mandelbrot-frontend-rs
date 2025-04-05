mod auction;
mod bids;
mod info;
mod visuals;

use std::sync::Arc;

use leptos::{prelude::*, task::spawn_local};
use leptos_router::{components::{Route, Routes}, hooks::{use_navigate, use_params, use_query_map}, params::Params, path};
use send_wrapper::SendWrapper;

use crate::{
    state::{ExplorerStateStoreFields, SalesStateStoreFields, State},
    util::preserve_log_level,
};
use {
    auction::Auction,
    bids::Bids,
    info::Info,
    visuals::Visuals,
};


#[component]
pub fn Explorer() -> impl IntoView {
    view! {
        <Routes fallback=|| "Not found.">
            <Route path=path!("/tokens/:token_id") view=Controller/>
            // <Route path="/" view=move |cx| view! { cx, <Controller address/> }/>
            <Route path=path!("*") view=Controller/>
        </Routes>
    }
}


#[derive(Clone, Params, PartialEq)]
struct ControllerParams {
    token_id: Option<u128>
}


#[component]
fn Controller() -> impl IntoView {
    let state = use_context::<SendWrapper<State>>().unwrap();
    let navigate = use_navigate();
    let query_map = use_query_map();

    let params = use_params::<ControllerParams>();
    let token_id = move || {
        if let Ok(params) = params.get() {
            params.token_id
        } else {
            None
        }
    };

    // query tokens and bids
    Effect::new({
        let state = state.clone();
        let navigate = navigate.clone();
        let token_id = token_id.clone();
        move |_| {
            let state = state.clone();
            let navigate = navigate.clone();
            let token_id = token_id().unwrap_or(1);
            spawn_local(async move {
                if let (Ok(tokens), Ok(children), Ok(bids)) = (
                    state.erc1155_contract.get_ancestry_metadata(token_id).await,
                    state.erc1155_contract.get_children_metadata(token_id).await,
                    state.erc1155_contract.get_bids(token_id).await
                ) {
                    state.explorer.nav_history().update(|nav_history| {
                        nav_history.clear();
                        nav_history.extend(tokens.into_iter().rev());
                    });
                    state.explorer.children().update(|children_| {
                        children_.clear();
                        children_.extend(children.into_iter().map(|m| (m.token_id, m)));
                    });
                    state.explorer.bids().update(|bids_| {
                        bids_.clear();
                        bids_.extend(bids.into_iter().map(|bid| (bid.token_id, bid)));
                    });
                } else {
                    navigate(&preserve_log_level("/tokens/1".into(), query_map), Default::default());
                }
            });
        }
    });

    let first = StoredValue::new(true);
    Effect::new({
        let state = state.clone();
        move |_| {
            if first.get_value() {
                state.explorer.nav_history().with(|nav_history| {
                    if let Some(token) = nav_history.last() {
                        first.set_value(false);
                        state.mandelbrot.lock().unwrap().move_into_bounds(&token.to_frame(mandelbrot_explorer::FrameColor::Blue).bounds);
                    }
                });
            }
        }
    });

    let select_bid = {
        let state = state.clone();
        move |bid_id, selected| {
            state.explorer.bids().update(|bids| {
                if let Some(bid) = bids.get_mut(&bid_id) {
                    bid.selected = selected;
                }
            });
            state.sales.bids().update(|bids| {
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
        let state = state.clone();
        move |frame_event: mandelbrot_explorer::FrameEvent| {
            match frame_event {
                mandelbrot_explorer::FrameEvent::DoubleClicked(frame) => {
                    match frame.color {
                        mandelbrot_explorer::FrameColor::Red |
                        mandelbrot_explorer::FrameColor::Pink |
                        mandelbrot_explorer::FrameColor::Blue |
                        mandelbrot_explorer::FrameColor::LightBlue => {
                            state.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds);
                            navigate(&preserve_log_level(format!("/tokens/{}", frame.id), query_map), Default::default());
                        }
                        mandelbrot_explorer::FrameColor::Yellow |
                        mandelbrot_explorer::FrameColor::Lemon => {
                            select_bid(frame.id, true);
                        }
                        mandelbrot_explorer::FrameColor::Green => {
                            select_bid(frame.id, false);
                        }
                    }
                }
                mandelbrot_explorer::FrameEvent::Entered(frame) => {
                    match frame.color {
                        mandelbrot_explorer::FrameColor::Red |
                        mandelbrot_explorer::FrameColor::Pink => {
                            navigate(&preserve_log_level(format!("/tokens/{}", frame.id), query_map), Default::default());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    state.mandelbrot.lock().unwrap().frame_event_callback = Some(Arc::new({
        let on_frame_event = on_frame_event.clone();
        move |frame_event| on_frame_event(frame_event)
    }));

    // check ownership
    Effect::new({
        let state = state.clone();
        move |_| {
            if let Some(address) = state.address.get() {
                state.explorer.children().update(|children|
                    children.values_mut().for_each(|token| token.owned = token.owner == address)
                );
                state.explorer.bids().update(|bids|
                    bids.values_mut().for_each(|bid| bid.owned = bid.owner == address)
                );
                state.explorer.nav_history().update(|nav_history|
                    nav_history.iter_mut().for_each(|token| token.owned = token.owner == address)
                );
            }
        }
    });

    // update frames
    Effect::new({
        let state = state.clone();
        move |_| {
            let mandelbrot = &mut state.mandelbrot.lock().unwrap();
            let frames = &mut mandelbrot.frames;
            frames.clear();
            frames.extend(state.explorer.children().get().values().map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Red)));
            frames.extend(state.explorer.bids().get().values().map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Yellow)));
            frames.extend(state.explorer.nav_history().get().iter().rev().map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Blue)));
            if let Some(redraw) = &mandelbrot.redraw {
                redraw();
            }
        }
    });

    // let create_bid = create_action(cx, {
    //     let state = state.clone();
    //     move |_| {
    //         let state = state.clone();
    //         async move {
    //             if let Some(address) = state.address.get_untracked() {
    //                 let bounds = state.mandelbrot.lock().unwrap().sample.get_bounds();
    //                 if let Some(token) = state.nav_history.get_untracked().last() {
    //                     state.erc1155_contract.bid(
    //                         address,
    //                         token.token_id,
    //                         Field {
    //                             x_min: bounds.x_min as f64,
    //                             y_min: bounds.y_min as f64,
    //                             x_max: bounds.x_max as f64,
    //                             y_max: bounds.y_max as f64
    //                         },
    //                         bid_amount.get_untracked(),
    //                         bids_minimum_price.get_untracked(),
    //                     ).await;
    //                 }
    //             };
    //         }
    //     }
    // });

    view! {
        <div class="flex flex-col">
            <Visuals/>
    
            {
                move || state.explorer.nav_history().get().last().cloned().map(|token| {
                    let state = state.clone();
                    view! {
                        <div class="bg-gray-800 text-white rounded-md shadow p-4">
                            <Info token=token.clone() />
                        </div>
    
                        <Show when={let state = state.clone(); move || state.address.get().is_some()} fallback=|| {} >
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
                        <Show when={let state = state.clone(); move || state.explorer.bids().get().len() > 0} fallback=|| {} >
                            <div class="bg-gray-800 text-white rounded-md shadow p-4">
                                <Bids bids=state.explorer.bids() />
                            </div>
                        </Show>
                    }.into_any()
                })
            }
        </div>
    }
}
