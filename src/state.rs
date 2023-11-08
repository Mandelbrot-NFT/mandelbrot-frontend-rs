use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use leptos::*;

use crate::evm::{
    contracts::ERC1155Contract,
    types::Metadata,
};


#[derive(Clone)]
pub struct ExplorerState {
    pub nav_history: RwSignal<Vec<Metadata>>,
    pub children: RwSignal<HashMap<u128, Metadata>>,
    pub bids: RwSignal<HashMap<u128, Metadata>>,
}


#[derive(Clone)]
pub struct InventoryState {
    pub tokens: RwSignal<HashMap<u128, Metadata>>,
    pub bids: RwSignal<HashMap<u128, Metadata>>,
}


#[derive(Clone)]
pub struct SalesState {
    pub bids: RwSignal<HashMap<u128, HashMap<u128, Metadata>>>,
}


#[derive(Clone)]
pub struct State {
    pub mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
    pub address: Signal<Option<web3::types::Address>>,
    pub erc1155_contract: ERC1155Contract,
    pub explorer: ExplorerState,
    pub inventory: InventoryState,
    pub sales: SalesState,
}

impl State {
    pub async fn reload_inventory(&self) {
        if let Some(address) = self.address.get_untracked() {
            if let Ok((tokens, bids)) = self.erc1155_contract.get_owned_items(address).await {
                self.inventory.tokens.update(|tokens_| {
                    tokens_.clear();
                    tokens_.extend(tokens.into_iter().map(|token| (token.token_id, token)));
                });
                self.inventory.bids.update(|bids_| {
                    bids_.clear();
                    bids_.extend(bids.into_iter().map(|bid| (bid.token_id, bid)));
                });
            }
            self.reload_sales().await;
        }
    }

    pub async fn reload_sales(&self) {
        let bids = futures::future::join_all(self.inventory.tokens.get_untracked().keys().map(|token_id| {
            async move {
                (*token_id, self.erc1155_contract.get_bids(*token_id).await)
            }
        })).await.into_iter().map(|(token_id, result)| (
            token_id,
            if let Ok(bids) = result {
                bids.into_iter().map(|bid| (bid.token_id, bid)).collect::<HashMap<_, _>>()
            } else {
                HashMap::new()
            }
        )).collect::<HashMap<_, _>>();
        self.sales.bids.set(bids);
    }
}
