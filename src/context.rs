use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use leptos::prelude::*;
use reactive_stores::Store;

use crate::evm::{contracts::ERC1155Contract, types::Metadata};

#[derive(Clone, Store)]
pub struct ExplorerState {
    pub nav_history: Vec<Metadata>,
    pub children: HashMap<u128, Metadata>,
    pub bids: HashMap<u128, Metadata>,
}

#[derive(Clone, Store)]
pub struct InventoryState {
    pub tokens: HashMap<u128, Metadata>,
    pub bids: HashMap<u128, Metadata>,
}

#[derive(Clone, Store)]
pub struct SalesState {
    pub bids: HashMap<u128, HashMap<u128, Metadata>>,
}

#[derive(Clone, Store)]
pub struct State {
    pub address: Option<web3::types::Address>,
    pub explorer: ExplorerState,
    pub inventory: InventoryState,
    pub sales: SalesState,
}

#[derive(Clone)]
pub struct Context {
    pub mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
    pub erc1155_contract: ERC1155Contract,
    pub state: Store<State>,
}

impl Context {
    pub async fn reload_inventory(&self) {
        if let Some(address) = self.state.address().get_untracked() {
            if let Ok((tokens, bids)) = self.erc1155_contract.get_owned_items(address).await {
                self.state.inventory().tokens().update(|tokens_| {
                    tokens_.clear();
                    tokens_.extend(tokens.into_iter().map(|token| (token.token_id, token)));
                });
                self.state.inventory().bids().update(|bids_| {
                    bids_.clear();
                    bids_.extend(bids.into_iter().map(|bid| (bid.token_id, bid)));
                });
            }
            self.reload_sales().await;
        }
    }

    pub async fn reload_sales(&self) {
        let selected_bids = self
            .state
            .explorer()
            .bids()
            .get_untracked()
            .iter()
            .filter_map(|(&bid_id, &Metadata { selected, .. })| selected.then(|| bid_id))
            .collect::<Vec<_>>();

        let bids = futures::future::join_all(
            self.state
                .inventory()
                .tokens()
                .get_untracked()
                .keys()
                .map(|token_id| async move { (*token_id, self.erc1155_contract.get_bids(*token_id).await) }),
        )
        .await
        .into_iter()
        .map(|(token_id, result)| {
            (
                token_id,
                if let Ok(bids) = result {
                    bids.into_iter()
                        .map(|mut bid| {
                            bid.selected = selected_bids.contains(&bid.token_id);
                            (bid.token_id, bid)
                        })
                        .collect::<HashMap<_, _>>()
                } else {
                    HashMap::new()
                },
            )
        })
        .collect::<HashMap<_, _>>();
        self.state.sales().bids().set(bids);
    }
}
