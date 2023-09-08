use std::sync::Arc;

use eyre::Result;
use leptonic::prelude::*;
use leptos::*;
use web3::{
    transports::{eip_1193::Eip1193, Either, Http},
    types::Address,
    Web3
};

use crate::evm::contracts::{
    self,
    ERC1155Contract,
    Wrapped1155FactoryContract,
    ERC20Contract
};


async fn get_balance(
    address: Address,
    erc1155_contract: ERC1155Contract,
    erc20_contract: ERC20Contract,
) -> Result<(f64, f64)> {
    Ok((erc1155_contract.get_fuel_balance(address).await?, erc20_contract.get_balance(address).await?))
}

#[component]
pub fn Balance(
    cx: Scope,
    address: Signal<Option<Address>>,
) -> impl IntoView {
    let web3 = expect_context::<Web3<Either<Eip1193, Http>>>(cx);
    let handle_error = expect_context::<WriteSignal<Option<contracts::Error>>>(cx);

    let (fuel_balance, set_fuel_balance) = create_signal(cx, 0.0);
    let (wfuel_balance, set_wfuel_balance) = create_signal(cx, 0.0);
    let (wrap_amount, set_wrap_amount) = create_signal(cx, 0.0);
    let (unwrap_amount, set_unwrap_amount) = create_signal(cx, 0.0);

    let uniswap_link = format!("https://app.uniswap.org/#/swap?inputCurrency=ETH&outputCurrency={}", env!("ERC20_CONTRACT_ADDRESS"));

    let handle_error = Arc::new(move |error| handle_error.set(Some(error)));
    let erc1155_contract = ERC1155Contract::new(&web3, handle_error.clone());
    let wrapper_contract = Wrapped1155FactoryContract::new(&web3, erc1155_contract.address(), handle_error);
    let erc20_contract = ERC20Contract::new(&web3);

    let refresh_balance = create_action(cx, {
        let erc1155_contract = erc1155_contract.clone();
        let erc20_contract = erc20_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let erc20_contract = erc20_contract.clone();
            async move {
                if let Some(address) = address.get_untracked() {
                    if let Ok((fuel_balance, wfuel_balance)) = get_balance(address, erc1155_contract, erc20_contract).await {
                        set_fuel_balance(fuel_balance);
                        set_wfuel_balance(wfuel_balance);
                    }
                }
            }
        }
    });

    create_effect(cx, move |_| {
        if address.get().is_some() {
            refresh_balance.dispatch(());
        }
    });

    let unwrap = create_action(cx, {
        let wrapper_contract = wrapper_contract.clone();
        move |_| {
            let wrapper_contract = wrapper_contract.clone();
            async move {
                if let Some(address) = address.get_untracked() {
                    wrapper_contract.unwrap(address, unwrap_amount.get_untracked()).await;
                    refresh_balance.dispatch(());
                }
            }
        }
    });

    let wrap = create_action(cx, {
        let erc1155_contract = erc1155_contract.clone();
        let wrapper_contract = wrapper_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let wrapper_contract = wrapper_contract.clone();
            async move {
                if let Some(address) = address.get_untracked() {
                    erc1155_contract.transfer_fuel(address, wrapper_contract.address(), wrap_amount.get_untracked()).await;
                    refresh_balance.dispatch(());
                }
            }
        }
    });

    view! { cx,
        <div>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                <Button on_click=move |_| refresh_balance.dispatch(())>"Refresh balance"</Button>
                <a href={uniswap_link} target="_blank">
                    <Button on_click=move |_| ()>"Buy wFUEL"</Button>
                </a>
            </Stack>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                <strong>"wFUEL: "</strong>
                {move || view! { cx,
                    {format!("{:.2}", wfuel_balance())}
                    <Slider style="width: 20em" min=0.0 max=wfuel_balance() step=0.01
                        value=unwrap_amount set_value=set_unwrap_amount
                        value_display=create_callback(cx, move |v| format!("{v:.2}")) />
                }}
                <Button on_click=move |_| unwrap.dispatch(())>"Unwrap"</Button>
            </Stack>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                <strong>"FUEL: "</strong>
                {move || view! { cx,
                    {format!("{:.2}", fuel_balance())}
                    <Slider style="width: 20em" min=0.0 max=fuel_balance() step=0.01
                        value=wrap_amount set_value=set_wrap_amount
                        value_display=create_callback(cx, move |v| format!("{v:.2}")) />
                }}
                <Button on_click=move |_| wrap.dispatch(())>"Wrap"</Button>
            </Stack>
        </div>
    }
}
