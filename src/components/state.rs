use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use leptos::{prelude::*, task::spawn_local};
use leptos_ethereum_provider::EthereumInterface;
use reactive_stores::Store;
use send_wrapper::SendWrapper;
use web3::transports::{eip_1193::Eip1193, Either, Http};

use crate::{
    chain::sepolia_testnet,
    evm::contracts::{self, ERC1155Contract},
    state::{ExplorerState, InventoryState, SalesState, State},
};

#[derive(Clone, Debug)]
pub struct Web3(pub web3::Web3<Either<Eip1193, Http>>);

#[component]
pub fn StateContextProvider(
    mandelbrot: SendWrapper<Arc<Mutex<mandelbrot_explorer::Interface>>>,
    children: Children,
) -> impl IntoView {
    let ethereum = use_context::<Option<EthereumInterface>>().unwrap();
    let transport = if let Some(ethereum) = &ethereum {
        Either::Left(Eip1193::new(ethereum.provider.clone()))
    } else {
        Either::Right(Http::new(&sepolia_testnet().rpc_urls[0]).unwrap())
    };
    let web3 = web3::Web3::new(transport);
    provide_context(LocalStorage::wrap(Web3(web3.clone())));

    let error = RwSignal::new(None);
    let error_message = Memo::new(move |_| {
        error.with(|error| {
            error.as_ref().map(|error| match error {
                contracts::Error::TokenNotFound => "Unable to find an NFT with this Id".into(),
                contracts::Error::NoRightsToBurn => "You don't have the necessary rights to burn this NFT".into(),
                contracts::Error::TokenNotEmpty => {
                    "It is not allowed to burn an NFT if it has minted NFTs inside".into()
                }
                contracts::Error::BidNotFound => "Unable to find a bid with this Id".into(),
                contracts::Error::BidTooLow => "Your bid is too low".into(),
                contracts::Error::MinimumBidTooLow => "Minimum bid for the NFT that you wish to mint is too low".into(),
                contracts::Error::TooManyChildTokens => "This NFT cannot contain any more NFTs".into(),
                contracts::Error::NoRightsToApproveBid => {
                    "You don't have the necessary rights to approve these bids".into()
                }
                contracts::Error::NoRightsToDeleteBid => {
                    "You don't have the necessary rights to delete this bid".into()
                }
                contracts::Error::FieldOutside => {
                    "NFT that you are trying to mint has to be within the bounds of parent NFT".into()
                }
                contracts::Error::FieldsOverlap => "NFT that you are trying to mint overlaps with another NFT".into(),
                contracts::Error::FieldTooLarge => "NFT that you are trying to mint is too large".into(),
                contracts::Error::Other(message) => message.clone(),
            })
        })
    });
    provide_context(error.write_only());

    let state = State {
        mandelbrot: mandelbrot.take(),
        address: Signal::derive(move || {
            ethereum
                .clone()
                .and_then(|ethereum| ethereum.connected().then(|| ethereum.address().get()))
                .flatten()
        }),
        erc1155_contract: ERC1155Contract::new(&web3, Arc::new(move |e| error.set(Some(e)))),
        explorer: Store::new(ExplorerState {
            nav_history: Vec::new(),
            children: HashMap::new(),
            bids: HashMap::new(),
        }),
        inventory: Store::new(InventoryState {
            tokens: HashMap::new(),
            bids: HashMap::new(),
        }),
        sales: Store::new(SalesState { bids: HashMap::new() }),
    };
    provide_context(LocalStorage::wrap(state.clone()));

    Effect::new(move |_| {
        if state.address.get().is_some() {
            let state = state.clone();
            spawn_local(async move {
                state.reload_inventory().await;
            });
        }
    });

    view! {
        { children() }

        <Show when=move || error_message.get().is_some()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-60">
                <div class="bg-gray-900 text-white rounded-lg shadow-lg p-6 w-full max-w-md space-y-6">

                    <div class="text-xl font-semibold border-b border-gray-700 pb-2">
                        "Error"
                    </div>

                    <div class="text-sm text-gray-300">
                        {move || error_message.get().unwrap_or("".into())}
                    </div>

                    <div class="flex justify-end pt-4 border-t border-gray-700">
                        <button
                            on:click=move |_| error.set(None)
                            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-md text-sm font-medium transition"
                        >
                            "Ok"
                        </button>
                    </div>

                </div>
            </div>
        </Show>
    }
}
