use leptonic::prelude::*;
use leptos::*;
use leptos_ethereum_provider::EthereumInterface;

use super::balance::Balance;


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
            <button on:click=disconnect class="btn btn-primary connected">
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
    fuel_balance: RwSignal<f64>,
) -> impl IntoView {
    let ethereum = expect_context::<Option<EthereumInterface>>();

    let disconnect = {
        move |_| {
            open.set(false);
            if let Some(ethereum) = &ethereum {
                ethereum.disconnect();
            }
        }
    };

    view! {
        <Drawer side=DrawerSide::Right shown=Signal::derive(move || open.get()) style="padding: 0.5em; height: 19.5em; overflow: scroll; position: absolute; top: 3em; right: 0; background-color: var(--brand-color); border-left: 1px solid gray;">
            <Balance fuel_balance/>
            <button on:click=disconnect class="btn btn-primary connected">
                "Disconnect"
            </button>
        </Drawer>
    }
}
