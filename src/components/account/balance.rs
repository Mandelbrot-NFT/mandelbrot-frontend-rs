use std::sync::Arc;

use eyre::Result;
use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::{
    components::{primitive::Slider, state::Web3},
    evm::contracts::{self, ERC1155Contract, ERC20Contract, Wrapped1155FactoryContract},
    state::State,
};

async fn get_balance(
    address: web3::types::Address,
    erc1155_contract: ERC1155Contract,
    erc20_contract: ERC20Contract,
) -> Result<(f64, f64)> {
    Ok((
        erc1155_contract.get_OM_balance(address).await?,
        erc20_contract.get_balance(address).await?,
    ))
}

#[component]
pub fn Balance(OM_balance: RwSignal<f64>) -> impl IntoView {
    let state = use_context::<SendWrapper<State>>().unwrap().take();
    let web3 = use_context::<SendWrapper<Web3>>().unwrap().take().0;
    let handle_error = use_context::<WriteSignal<Option<contracts::Error>>>().unwrap();

    let wOM_balance = RwSignal::new(0.0);
    let wrap_amount = RwSignal::new(0.0);
    let unwrap_amount = RwSignal::new(0.0);

    let uniswap_link = format!(
        "https://app.uniswap.org/#/swap?inputCurrency=ETH&outputCurrency={}",
        env!("ERC20_CONTRACT_ADDRESS")
    );

    let handle_error = Arc::new(move |error| handle_error.set(Some(error)));
    let wrapper_contract = Wrapped1155FactoryContract::new(&web3, state.erc1155_contract.address(), handle_error);
    let erc20_contract = ERC20Contract::new(&web3);

    let refresh_balance = Action::new_local({
        let erc1155_contract = state.erc1155_contract.clone();
        let erc20_contract = erc20_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let erc20_contract = erc20_contract.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    if let Ok((OM_balance_, wOM_balance_)) =
                        get_balance(address, erc1155_contract, erc20_contract).await
                    {
                        OM_balance.set(OM_balance_);
                        wOM_balance.set(wOM_balance_);
                    }
                }
            }
        }
    });

    Effect::new(move |_| {
        if state.address.get().is_some() {
            refresh_balance.dispatch(());
        }
    });

    let unwrap = Action::new_local({
        let wrapper_contract = wrapper_contract.clone();
        move |_| {
            let wrapper_contract = wrapper_contract.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    wrapper_contract.unwrap(address, unwrap_amount.get_untracked()).await;
                    refresh_balance.dispatch(());
                }
            }
        }
    });

    let wrap = Action::new_local({
        let erc1155_contract = state.erc1155_contract.clone();
        let wrapper_contract = wrapper_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let wrapper_contract = wrapper_contract.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    erc1155_contract
                        .transfer_OM(address, wrapper_contract.address(), wrap_amount.get_untracked())
                        .await;
                    refresh_balance.dispatch(());
                }
            }
        }
    });

    view! {
        <div class="grid gap-6 p-4 text-white w-full max-w-[400px]">
            // Refresh balance and Buy wOM Buttons
            <div class="grid grid-cols-2 gap-4">
                <button
                    on:click=move |_| { refresh_balance.dispatch(()); }
                    class="py-2 bg-gray-700 hover:bg-gray-600 rounded-md font-semibold transition"
                >
                    "Refresh balance"
                </button>
                <a href={uniswap_link} target="_blank">
                    <button class="py-2 bg-purple-700 hover:bg-purple-600 rounded-md font-semibold transition">
                        "Buy wOM"
                    </button>
                </a>
            </div>

            // wOM Balance Slider and Unwrap Button
            <div class="grid grid-cols-[60px_1fr_min-content] gap-4 items-center min-w-0">
                <label class="text-sm font-semibold text-highlight text-right">"wOM:"</label>

                <div class="flex items-center gap-2 min-w-0">
                    <span class="w-[50px] text-right text-sm font-mono text-accent1">
                        {move || format!("{:.2}", wOM_balance.get())}
                    </span>
                    <Slider
                        max=wOM_balance.read_only()
                        value=unwrap_amount
                        class="w-full h-2 bg-gray-400 rounded-full focus:outline-none focus:ring-2 focus:ring-accent1"
                    />
                    <span class="w-[50px] text-left text-sm font-mono text-accent2">
                        {move || format!("{:.2}", unwrap_amount.get())}
                    </span>
                </div>

                <button
                    on:click=move |_| { unwrap.dispatch(()); }
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-semibold transition"
                >
                    "Unwrap"
                </button>
            </div>

            // OM Balance Slider and Wrap Button
            <div class="grid grid-cols-[60px_1fr_min-content] gap-4 items-center min-w-0">
                <label class="text-sm font-semibold text-highlight text-right">"OM:"</label>

                <div class="flex items-center gap-2 min-w-0">
                    <span class="w-[50px] text-right text-sm font-mono text-accent1">
                        {move || format!("{:.2}", OM_balance.get())}
                    </span>
                    <Slider
                        max=OM_balance.read_only()
                        value=wrap_amount
                        class="w-full h-2 bg-gray-400 rounded-full focus:outline-none focus:ring-2 focus:ring-accent1"
                    />
                    <span class="w-[50px] text-left text-sm font-mono text-accent2">
                        {move || format!("{:.2}", wrap_amount.get())}
                    </span>
                </div>

                <button
                    on:click=move |_| { wrap.dispatch(()); }
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-semibold transition"
                >
                    "Wrap"
                </button>
            </div>
        </div>
    }
}
