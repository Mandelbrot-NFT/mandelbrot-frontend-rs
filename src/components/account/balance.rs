use eyre::Result;
use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::{
    context::{Context, StateStoreFields},
    evm::contracts::MandelbrotNFTContract,
};

async fn get_balance(address: web3::types::Address, contract: MandelbrotNFTContract) -> Result<f64> {
    contract.get_token_balance(address).await
}

#[component]
pub fn Balance(token_balance: RwSignal<f64>) -> impl IntoView {
    let context = use_context::<SendWrapper<Context>>().unwrap().take();

    let uniswap_link = format!(
        "https://app.uniswap.org/#/swap?inputCurrency=ETH&outputCurrency={}",
        env!("CONTRACT_ADDRESS")
    );

    let refresh_balance = Action::new_local({
        let contract = context.contract.clone();
        move |_| {
            let contract = contract.clone();
            async move {
                if let Some(address) = context.state.address().get_untracked() {
                    if let Ok(OM_balance) = get_balance(address, contract).await {
                        token_balance.set(OM_balance);
                    }
                }
            }
        }
    });

    Effect::new(move || {
        if context.state.address().get().is_some() {
            refresh_balance.dispatch(());
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

            // OM Balance Slider and Wrap Button
            <div class="grid grid-cols-[60px_1fr_min-content] gap-4 items-center min-w-0">
                <label class="text-sm font-semibold text-highlight text-right">"OM:"</label>

                <div class="flex items-center gap-2 min-w-0">
                    <span class="w-[50px] text-right text-sm font-mono text-accent1">
                        {move || format!("{:.2}", token_balance.get())}
                    </span>
                </div>
            </div>
        </div>
    }
}
