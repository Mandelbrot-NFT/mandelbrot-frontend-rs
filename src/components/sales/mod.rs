use std::collections::HashMap;

use leptonic::prelude::*;
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
        move |bid_id| {
            if let Some(bid) = state.inventory.bids.get().get(&bid_id) {
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

    let selected_bids = move || {
        state.sales.bids
            .get()
            .values()
            .map(|bids| bids.values())
            .flatten()
            .filter(|bid| bid.selected)
            .map(|bid| bid.clone())
            .collect::<Vec<_>>()
    };

    let total_approve_amount = move || selected_bids().iter().map(|bid| bid.locked_OM).sum::<f64>();

    let approve_bids = create_action({
        let erc1155_contract = state.erc1155_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    let selected_bids: Vec<u128> = selected_bids().iter().map(|bid| bid.token_id).collect();
                    erc1155_contract.batch_approve_bids(address, &selected_bids).await;
                }
            }
        }
    });

    view! {
        <Collapsibles default_on_open=OnOpen::CloseOthers>
            <Stack spacing=Size::Em(0.6)>
                <For
                    each=move || state.inventory.tokens.get().into_values()
                    key=|token| token.token_id
                    children=move |token| {
                        let zoom_token = zoom_token.clone();
                        let zoom_bid = zoom_bid.clone();
                        view! {
                            <Collapsible>
                                <CollapsibleHeader slot>
                                    <Button on_click={let zoom_token = zoom_token.clone(); move |_| zoom_token(token.token_id)}>"Zoom"</Button>
                                    {token.token_id}
                                </CollapsibleHeader>
                                <CollapsibleBody slot>
                                    {
                                        move || {
                                            let zoom_bid = zoom_bid.clone();
                                            let bids = move || state.sales.bids.get().get(&token.token_id).unwrap_or(&HashMap::new()).clone();
                                            let sorted_bids = move || {
                                                let mut bids: Vec<Metadata> = bids().values().map(|bid| bid.clone()).collect();
                                                bids.sort_by(|bid_a, bid_b| bid_b.locked_OM.partial_cmp(&bid_a.locked_OM).unwrap());
                                                bids
                                            };
                                            view! {
                                                <For
                                                    each=move || sorted_bids()
                                                    key=|bid| bid.token_id
                                                    children=move |bid| view! {
                                                        <p>
                                                            <Toggle
                                                                state=Signal::derive(move || bids()[&bid.token_id].selected)
                                                                set_state=move |state: bool| toggle_bid(token.token_id, bid.token_id, state)
                                                                variant=ToggleVariant::Stationary
                                                            />
                                                            {format!("{} {:?}", bid.locked_OM.to_string(), bid.owner)}
                                                            <Button on_click={let zoom_bid = zoom_bid.clone(); move |_| zoom_bid(bid.token_id)}>"Zoom"</Button>
                                                        </p>
                                                    }
                                                />
                                            }
                                        }
                                    }
                                </CollapsibleBody>
                            </Collapsible>
                        }
                    }
                />
            </Stack>
        </Collapsibles>
        <p>
            {move || total_approve_amount()}
            <Button on_click=move |_| approve_bids.dispatch(())>"Approve"</Button>
        </p>
        <Button on_click=move |_| refresh.dispatch(())>"Refresh"</Button>
    }
}
