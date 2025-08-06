use std::sync::{Arc, Mutex};

use leptos::{prelude::*, task::spawn_local};
use leptos_ethereum_provider::EthereumInterface;
use reactive_stores::Store;
use send_wrapper::SendWrapper;
use web3::transports::{Either, Http, eip_1193::Eip1193};

use super::error_handler::ErrorHandler;
use crate::{
    chain::sepolia_testnet,
    context::{Context, State, StateStoreFields},
    evm::contracts::ERC1155Contract,
};

#[derive(Clone, Debug)]
pub struct Web3(pub web3::Web3<Either<Eip1193, Http>>);

#[component]
pub fn ContextProvider(
    mandelbrot: SendWrapper<Arc<Mutex<mandelbrot_explorer::Interface>>>,
    state: Store<State>,
    children: Children,
) -> impl IntoView {
    let ethereum = use_context::<Option<EthereumInterface>>().unwrap();
    let transport = if let Some(ethereum) = &ethereum {
        Either::Left(Eip1193::new(ethereum.provider.clone()))
    } else {
        Either::Right(Http::new(&sepolia_testnet().rpc_urls[0]).unwrap())
    };
    let web3 = web3::Web3::new(transport);

    let error = RwSignal::new(None);
    let context = Context {
        mandelbrot: mandelbrot.take(),
        erc1155_contract: ERC1155Contract::new(&web3, Arc::new(move |e| error.set(Some(e)))),
        state,
    };

    Effect::new(move || {
        state.address().set(
            ethereum
                .clone()
                .and_then(|ethereum| ethereum.connected().then(|| ethereum.address().get()))
                .flatten(),
        )
    });

    Effect::new({
        let context = context.clone();
        move || {
            if state.address().get().is_some() {
                let context = context.clone();
                spawn_local(async move {
                    context.reload_inventory().await;
                });
            }
        }
    });

    provide_context(LocalStorage::wrap(Web3(web3)));
    provide_context(error.write_only());
    provide_context(LocalStorage::wrap(context));

    view! {
        { children() }
        <ErrorHandler error/>
    }
}
