use std::collections::HashMap;

use leptonic::prelude::*;
use leptos::*;
use web3::types::Address;

use crate::evm::{types::Metadata, contracts::ERC1155Contract};


#[component]
pub fn Bids(
    cx: Scope,
    erc1155_contract: ERC1155Contract,
    address: Signal<Option<Address>>,
    bids: RwSignal<HashMap<u128, Metadata>>,
) -> impl IntoView {
    let toggle_bid = {
        move |bid_id, state_| {
            bids.update(|bids| {
                if let Some(bid) = bids.get_mut(&bid_id) {
                    bid.selected = state_;
                }
            });
        }
    };

    let approve_bids = create_action(cx, {
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

    let delete_bid = create_action(cx, {
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

    let sorted_bids = move || {
        let mut bids: Vec<Metadata> = bids.get().values().map(|bid| bid.clone()).collect();
        bids.sort_by(|bid_a, bid_b| bid_b.locked_fuel.partial_cmp(&bid_a.locked_fuel).unwrap());
        bids
    };
    let total_approve_amount = move || {
        bids.get().values().filter(|bid| bid.selected).map(|bid| bid.locked_fuel).sum::<f64>()
    };

    view! { cx,
        <p>"Bids:"</p>
        <Box id="content">
            <For
                each=move || sorted_bids()
                key=|bid| bid.token_id
                view={
                    move |cx, bid| view! { cx,
                        <p>
                            <Toggle
                                state=bid.selected
                                set_state=create_callback(cx, move |state: bool| toggle_bid(bid.token_id, state))
                                variant=ToggleVariant::Stationary
                            />
                            {format!("{} {:?}", bid.locked_fuel.to_string(), bid.owner)}
                            <Button on_click=move |_| delete_bid.dispatch(bid.token_id)>"Delete"</Button>
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
