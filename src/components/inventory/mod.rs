mod bids;
mod tokens;

use std::{sync::{Arc, Mutex}, collections::HashMap};

use leptonic::prelude::*;
use leptos::*;

use crate::{
    components::blockchain::{Web3, Address},
    evm::{
        contracts::{self, ERC1155Contract},
        types::Metadata
    },
};
use bids::Bids;
use tokens::Tokens;


#[derive(Clone)]
struct State {
    mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
    erc1155_contract: ERC1155Contract,
    tokens: RwSignal<HashMap<u128, Metadata>>,
    bids: RwSignal<HashMap<u128, Metadata>>,
}


#[component]
pub fn Inventory() -> impl IntoView {
    let mandelbrot = expect_context::<Arc<Mutex<mandelbrot_explorer::Interface>>>();
    let web3 = expect_context::<Web3>().0;
    let address = expect_context::<Address>().0;
    let handle_error = expect_context::<WriteSignal<Option<contracts::Error>>>();

    let state = State {
        mandelbrot: mandelbrot.clone(),
        erc1155_contract: ERC1155Contract::new(&web3, Arc::new({
            move |error| handle_error.set(Some(error))
        })),
        tokens: create_rw_signal(HashMap::new()),
        bids: create_rw_signal(HashMap::new()),
    };

    let refresh = create_action({
        let erc1155_contract = state.erc1155_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            async move {
                if let Some(address) = address.get_untracked() {
                    if let Ok((tokens, bids)) = erc1155_contract.get_owned_items(address).await {
                        state.tokens.update(|tokens_| {
                            tokens_.clear();
                            tokens_.extend(tokens.into_iter().map(|token| (token.token_id, token)));
                        });
                        state.bids.update(|bids_| {
                            bids_.clear();
                            bids_.extend(bids.into_iter().map(|bid| (bid.token_id, bid)));
                        });
                    }
                }
            }
        }
    });

    create_effect(move |_| {
        if address.get().is_some() {
            refresh.dispatch(());
        }
    });

    view! {
        <p>"Tokens:"</p>
        <Tokens
            erc1155_contract=state.erc1155_contract.clone()
            tokens=state.tokens
        />
        <p>"Bids:"</p>
        <Bids
            erc1155_contract=state.erc1155_contract.clone()
            bids=state.bids
        />
        <Button on_click=move |_| refresh.dispatch(())>"Refresh"</Button>
    }
}
