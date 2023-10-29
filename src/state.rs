use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use leptos::{RwSignal, Signal};

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
pub struct State {
    pub mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
    pub address: Signal<Option<web3::types::Address>>,
    pub erc1155_contract: ERC1155Contract,
    pub explorer: ExplorerState,
    pub inventory: InventoryState,
}
