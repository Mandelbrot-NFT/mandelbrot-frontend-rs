use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use leptonic::prelude::*;
use leptos::*;
use leptos_router::*;
use web3::{
    transports::{eip_1193::Eip1193, Either, Http},
    types::Address,
    Web3,
};

use crate::{
    evm::{
        contracts::{self, ERC1155Contract},
        types::Metadata,
    },
    components::{
        auction::Auction,
        bids::Bids,
    },
};


#[derive(Clone)]
struct State {
    address: Signal<Option<Address>>,
    mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
    erc1155_contract: ERC1155Contract,
    nav_history: RwSignal<Vec<Metadata>>,
    children: RwSignal<HashMap<u128, Metadata>>,
    bids: RwSignal<HashMap<u128, Metadata>>,
}


#[derive(Clone, Params, PartialEq)]
struct ControllerParams {
    token_id: Option<u128>
}


#[component]
pub fn Controller(
    cx: Scope,
    address: Signal<Option<Address>>,
) -> impl IntoView {
    let mandelbrot = expect_context::<Arc<Mutex<mandelbrot_explorer::Interface>>>(cx);
    let web3 = expect_context::<Web3<Either<Eip1193, Http>>>(cx);
    let handle_error = expect_context::<WriteSignal<Option<contracts::Error>>>(cx);
    let navigate = use_navigate(cx);

    let state = State {
        address: address.clone(),
        mandelbrot: mandelbrot.clone(),
        erc1155_contract: ERC1155Contract::new(&web3, Arc::new({
            move |error| handle_error.set(Some(error))
        })),
        nav_history: create_rw_signal(cx, Vec::new()),
        children: create_rw_signal(cx, HashMap::new()),
        bids: create_rw_signal(cx, HashMap::new()),
    };

    let params = use_params::<ControllerParams>(cx);
    let token_id = move || {
        if let Ok(params) = params.get() {
            params.token_id
        } else {
            None
        }
    };

    let query = use_query_map(cx);
    let preserve_log_level = move |uri| {
        if let Some(log_level) = query.get_untracked().get("RUST_LOG") {
            format!("{uri}?RUST_LOG={log_level}")
        } else {
            uri
        }
    };

    // query tokens and bids
    create_effect(cx, {
        let state = state.clone();
        let token_id = token_id.clone();
        move |_| {
            let state = state.clone();
            let token_id = token_id().unwrap_or(1);
            spawn_local(async move {
                if let (Ok(tokens), Ok(children), Ok(bids)) = (
                    state.erc1155_contract.get_ancestry_metadata(token_id).await,
                    state.erc1155_contract.get_children_metadata(token_id).await,
                    state.erc1155_contract.get_bids(token_id).await
                ) {
                    cx.batch(|| {
                        state.nav_history.update(|nav_history| {
                            nav_history.clear();
                            nav_history.extend(tokens.into_iter().rev());
                        });
                        state.children.update(|children_| {
                            children_.clear();
                            children_.extend(children.into_iter().map(|m| (m.token_id, m)));
                        });
                        state.bids.update(|bids_| {
                            bids_.clear();
                            bids_.extend(bids.into_iter().map(|bid| (bid.token_id, bid)));
                        });
                    });
                } else {
                    use_navigate(cx)(&preserve_log_level("/tokens/1".into()), Default::default());
                }
            });
        }
    });

    let first = store_value(cx, true);
    create_effect(cx, {
        let state = state.clone();
        move |_| {
            if first() {
                state.nav_history.with(|nav_history| {
                    if let Some(token) = nav_history.last() {
                        first.set_value(false);
                        state.mandelbrot.lock().unwrap().sample_location.move_into_frame(&token.to_frame(mandelbrot_explorer::FrameColor::Blue));
                    }
                });
            }
        }
    });

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
                            state.mandelbrot.lock().unwrap().sample_location.move_into_frame(&frame);
                            navigate(&preserve_log_level(format!("/tokens/{}", frame.id)), Default::default());
                        }
                        mandelbrot_explorer::FrameColor::Yellow |
                        mandelbrot_explorer::FrameColor::Lemon => {
                            state.bids.update(|bids| {
                                if let Some(bid) = bids.get_mut(&frame.id) {
                                    bid.selected = true;
                                }
                            });
                        }
                        mandelbrot_explorer::FrameColor::Green => {
                            state.bids.update(|bids| {
                                if let Some(bid) = bids.get_mut(&frame.id) {
                                    bid.selected = false;
                                }
                            });
                        }
                    }
                }
                mandelbrot_explorer::FrameEvent::Entered(frame) => {
                    match frame.color {
                        mandelbrot_explorer::FrameColor::Red |
                        mandelbrot_explorer::FrameColor::Pink => {
                            navigate(&preserve_log_level(format!("/tokens/{}", frame.id)), Default::default());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    mandelbrot.lock().unwrap().frame_event_callback = Some(Arc::new({
        let on_frame_event = on_frame_event.clone();
        move |frame_event| on_frame_event(frame_event)
    }));

    // check ownership
    create_effect(cx, move |_| {
        cx.batch(move || {
            state.children.track();
            state.bids.track();
            state.nav_history.track();
            if let Some(address) = state.address.get() {
                state.children.update(|children|
                    children.values_mut().for_each(|token| token.owned = token.owner == address)
                );
                state.bids.update(|bids|
                    bids.values_mut().for_each(|bid| bid.owned = bid.owner == address)
                );
                state.nav_history.update(|nav_history|
                    nav_history.iter_mut().for_each(|token| token.owned = token.owner == address)
                );
            }
        });
    });

    // update frames
    create_effect(cx, {
        let state = state.clone();
        move |_| {
            let mandelbrot = &mut state.mandelbrot.lock().unwrap();
            let frames = &mut mandelbrot.frames;
            frames.clear();
            frames.extend(state.children.get().values().map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Red)));
            frames.extend(state.bids.get().values().map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Yellow)));
            frames.extend(state.nav_history.get().iter().rev().map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Blue)));
            if let Some(redraw) = &mandelbrot.redraw {
                redraw();
            }
        }
    });

    let burn_token = create_action(cx, {
        let state = state.clone();
        move |token_id: &u128| {
            let state = state.clone();
            let token_id = *token_id;
            async move {
                if let Some(address) = state.address.get_untracked() {
                    state.erc1155_contract.burn(address, token_id).await;
                }
            }
        }
    });

    // let create_bid = create_action(cx, {
    //     let state = state.clone();
    //     move |_| {
    //         let state = state.clone();
    //         async move {
    //             if let Some(address) = state.address.get_untracked() {
    //                 let params = state.mandelbrot.lock().unwrap().sample_location.to_mandlebrot_params(0);
    //                 if let Some(token) = state.nav_history.get_untracked().last() {
    //                     state.erc1155_contract.bid(
    //                         address,
    //                         token.token_id,
    //                         Field {
    //                             x_min: params.x_min as f64,
    //                             y_min: params.y_min as f64,
    //                             x_max: params.x_max as f64,
    //                             y_max: params.y_max as f64
    //                         },
    //                         bid_amount.get_untracked(),
    //                         bids_minimum_price.get_untracked(),
    //                     ).await;
    //                 }
    //             };
    //         }
    //     }
    // });

    move || {
        if let Some(token) = state.nav_history.get().last() {
            let token = token.clone();
            let erc1155_contract = state.erc1155_contract.clone();
            let token_id = token.token_id;
            let minimum_price = token.minimum_price;
            view! { cx,
                <p>{format!("NFT id: {}", token_id)}</p>
                <p>{format!("Owner: {}", token.owner)}</p>
                <p>{format!("Locked FUEL: {}", token.locked_fuel)}</p>
                <p>{format!("Minimum bid: {}", minimum_price)}</p>
                <Show when=move || address.get().is_some() fallback=|_| {}>
                    <Button on_click=move |_| burn_token.dispatch(token_id)>"Burn"</Button>
                    {
                        let token = token.clone();
                        let erc1155_contract = erc1155_contract.clone();
                        view! { cx,
                            <Separator/>
                            <Auction
                                erc1155_contract=erc1155_contract.clone()
                                address
                                token=Signal::derive(cx, move || token.clone())
                            />
                            <Separator/>
                            <Show when=move || {(state.bids)().len() > 0} fallback=|_| {}>
                                <Bids
                                    erc1155_contract=erc1155_contract.clone()
                                    address
                                    bids=state.bids
                                />
                            </Show>
                        }
                    }
                </Show>
            }
        } else {
            Fragment::new(vec![])
        }
    }
}
