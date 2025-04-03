mod balance;

use leptos::*;
use leptos_ethereum_provider::{AccountLabel, EthereumInterface};

use balance::Balance;


#[component]
pub fn AccountButton(
    balance: ReadSignal<f64>,
    #[prop(into)] on_click: Callback<()>,
) -> impl IntoView {
    let disconnect = {
        move |_| on_click.call(())
    };

    view! {
        <div>
            <button on:click=disconnect class="btn-primary connected">
                <strong>"Balance: "</strong>
                {
                    move || format!("{:.2}", balance.get())
                }
            </button>
        </div>
    }
}


#[component]
pub fn Account(
    open: RwSignal<bool>,
    OM_balance: RwSignal<f64>,
) -> impl IntoView {
    let ethereum = use_context::<Option<EthereumInterface>>().unwrap();

    let disconnect = {
        move |_| {
            open.set(false);
            if let Some(ethereum) = &ethereum {
                ethereum.disconnect();
            }
        }
    };

    view! {
        <div
            class=move || if open.get() {
                "fixed top-12 right-0 max-h-[calc(100vh-3rem)] overflow-y-auto z-50 bg-gradient-to-b from-gray-900 to-black p-4 transform transition-transform duration-300 ease-in-out translate-x-0"
            } else {
                "fixed top-12 right-0 max-h-[calc(100vh-3rem)] overflow-y-auto z-50 bg-gradient-to-b from-gray-900 to-black p-4 transform transition-transform duration-300 ease-in-out translate-x-full"
            }
        >
            <div class="space-y-6 p-6 rounded-lg shadow-xl max-w-lg w-[400px] text-white bg-black bg-opacity-70 backdrop-blur-sm">
                <div class="text-sm font-mono text-center text-gray-400 break-all truncate max-w-full">
                    <AccountLabel/>
                </div>
    
                <Balance OM_balance/>
    
                <div class="flex justify-center">
                    <button on:click=disconnect class="w-full py-2 bg-red-600 hover:bg-red-700 text-white rounded-md font-semibold transition">
                        "Disconnect"
                    </button>
                </div>
            </div>
        </div>
    }
}
