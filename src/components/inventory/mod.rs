mod bids;
mod tokens;

use leptonic::prelude::*;
use leptos::*;

use crate::state::State;
use bids::Bids;
use tokens::Tokens;


#[component]
pub fn Inventory() -> impl IntoView {
    let state = use_context::<State>().unwrap();

    let refresh = create_action({
        let erc1155_contract = state.erc1155_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    if let Ok((tokens, bids)) = erc1155_contract.get_owned_items(address).await {
                        state.inventory.tokens.update(|tokens_| {
                            tokens_.clear();
                            tokens_.extend(tokens.into_iter().map(|token| (token.token_id, token)));
                        });
                        state.inventory.bids.update(|bids_| {
                            bids_.clear();
                            bids_.extend(bids.into_iter().map(|bid| (bid.token_id, bid)));
                        });
                    }
                }
            }
        }
    });

    create_effect(move |_| {
        if state.address.get().is_some() {
            refresh.dispatch(());
        }
    });

    view! {
        <p>"Tokens:"</p>
        <Tokens
            tokens=state.inventory.tokens
        />
        <p>"Bids:"</p>
        <Bids
            bids=state.inventory.bids
        />
        <Button on_click=move |_| refresh.dispatch(())>"Refresh"</Button>
    }
}
