use std::{collections::HashMap, sync::{Arc, Mutex}};

use leptonic::prelude::*;
use leptos::*;
use mandelbrot_explorer::FrameColor;
use web3::types::Address;

use crate::evm::{types::Metadata, contracts::ERC1155Contract};


#[component]
pub fn Bids(
    erc1155_contract: ERC1155Contract,
    address: Signal<Option<Address>>,
    bids: RwSignal<HashMap<u128, Metadata>>,
) -> impl IntoView {
    let mandelbrot = expect_context::<Arc<Mutex<mandelbrot_explorer::Interface>>>();

    let toggle_bid = {
        move |bid_id, state_| {
            bids.update(|bids| {
                if let Some(bid) = bids.get_mut(&bid_id) {
                    bid.selected = state_;
                }
            });
        }
    };

    let approve_bids = create_action({
        let erc1155_contract = erc1155_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            async move {
                if let Some(address) = address.get_untracked() {
                    let selected_bids: Vec<u128> = bids.get_untracked()
                        .values()
                        .filter(|bid| bid.selected)
                        .map(|bid| bid.token_id)
                        .collect();
                    erc1155_contract.batch_approve_bids(address, &selected_bids).await;
                }
            }
        }
    });

    let delete_bid = create_action({
        let erc1155_contract = erc1155_contract.clone();
        move |bid_id: &u128| {
            let erc1155_contract = erc1155_contract.clone();
            let bid_id = bid_id.clone();
            async move {
                if let Some(address) = address.get_untracked() {
                    erc1155_contract.delete_bid(address, bid_id).await;
                }
            }
        }
    });


    let zoom_bid = {
        move |bid_id| {
            if let Some(bid) = bids.get().get(&bid_id) {
                let frame = bid.to_frame(FrameColor::Blue);
                mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
            }
        }
    };

    let sorted_bids = create_memo(move |_| {
        let mut bids: Vec<Metadata> = bids.get().values().map(|bid| bid.clone()).collect();
        bids.sort_by(|bid_a, bid_b| bid_b.locked_fuel.partial_cmp(&bid_a.locked_fuel).unwrap());
        bids
    });
    let total_approve_amount = move || {
        bids.get().values().filter(|bid| bid.selected).map(|bid| bid.locked_fuel).sum::<f64>()
    };

    view! {
        <p>"Bids:"</p>
        <Box id="content">
            <For
                each=move || sorted_bids.get()
                key=|bid| bid.token_id
                children={
                    move |bid| view! {
                        <p>
                            <Toggle
                                state=Signal::derive(move || {
                                    if let Some(bid) = sorted_bids.get().iter().find(|bid_| bid_.token_id == bid.token_id) {
                                        bid.selected
                                    } else {
                                        false
                                    }
                                })
                                set_state=move |state: bool| toggle_bid(bid.token_id, state)
                                variant=ToggleVariant::Stationary
                            />
                            {format!("{} {:?}", bid.locked_fuel.to_string(), bid.owner)}
                            <Button on_click=move |_| delete_bid.dispatch(bid.token_id)>"Delete"</Button>
                            <Button on_click={let zoom_bid = zoom_bid.clone(); move |_| zoom_bid(bid.token_id)}>"Zoom"</Button>
                        </p>
                    }
                }
            />
        </Box>
        <p>
            {move || total_approve_amount()}
            <Button on_click=move |_| approve_bids.dispatch(())>"Approve"</Button>
        </p>
    }
}
