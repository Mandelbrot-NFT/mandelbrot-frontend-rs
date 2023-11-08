use std::sync::Arc;

use eyre::Result;
use leptonic::prelude::*;
use leptos::*;
use leptos_ethereum_provider::AccountLabel;

use crate::{
    components::state::Web3,
    evm::contracts::{
        self,
        ERC1155Contract,
        Wrapped1155FactoryContract,
        ERC20Contract
    },
    state::State,
};


async fn get_balance(
    address: web3::types::Address,
    erc1155_contract: ERC1155Contract,
    erc20_contract: ERC20Contract,
) -> Result<(f64, f64)> {
    Ok((erc1155_contract.get_OM_balance(address).await?, erc20_contract.get_balance(address).await?))
}

#[component]
pub fn Balance(
    OM_balance: RwSignal<f64>,
) -> impl IntoView {
    let state = use_context::<State>().unwrap();
    let web3 = use_context::<Web3>().unwrap().0;
    let handle_error = use_context::<WriteSignal<Option<contracts::Error>>>().unwrap();

    let (wOM_balance, set_wOM_balance) = create_signal(0.0);
    let (wrap_amount, set_wrap_amount) = create_signal(0.0);
    let (unwrap_amount, set_unwrap_amount) = create_signal(0.0);

    let uniswap_link = format!("https://app.uniswap.org/#/swap?inputCurrency=ETH&outputCurrency={}", env!("ERC20_CONTRACT_ADDRESS"));

    let handle_error = Arc::new(move |error| handle_error.set(Some(error)));
    let wrapper_contract = Wrapped1155FactoryContract::new(&web3, state.erc1155_contract.address(), handle_error);
    let erc20_contract = ERC20Contract::new(&web3);

    let refresh_balance = create_action({
        let erc1155_contract = state.erc1155_contract.clone();
        let erc20_contract = erc20_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let erc20_contract = erc20_contract.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    if let Ok((OM_balance_, wOM_balance)) = get_balance(address, erc1155_contract, erc20_contract).await {
                        OM_balance.set(OM_balance_);
                        set_wOM_balance.set(wOM_balance);
                    }
                }
            }
        }
    });

    create_effect(move |_| {
        if state.address.get().is_some() {
            refresh_balance.dispatch(());
        }
    });

    let unwrap = create_action({
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

    let wrap = create_action({
        let erc1155_contract = state.erc1155_contract.clone();
        let wrapper_contract = wrapper_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let wrapper_contract = wrapper_contract.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    erc1155_contract.transfer_OM(address, wrapper_contract.address(), wrap_amount.get_untracked()).await;
                    refresh_balance.dispatch(());
                }
            }
        }
    });

    view! {
        <div>
            <AccountLabel/>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                <Button on_click=move |_| refresh_balance.dispatch(())>"Refresh balance"</Button>
                <a href={uniswap_link} target="_blank">
                    <Button on_click=move |_| ()>"Buy wOM"</Button>
                </a>
            </Stack>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                <strong>"wOM: "</strong>
                {move || view! {
                    {format!("{:.2}", wOM_balance.get())}
                    <Slider style="width: 20em" min=0.0 max=wOM_balance.get() step=0.01
                        value=unwrap_amount set_value=set_unwrap_amount
                        value_display=move |v| format!("{v:.2}") />
                }}
                <Button on_click=move |_| unwrap.dispatch(())>"Unwrap"</Button>
            </Stack>
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                <strong>"OM: "</strong>
                {move || view! {
                    {format!("{:.2}", OM_balance.get())}
                    <Slider style="width: 20em" min=0.0 max=OM_balance.get() step=0.01
                        value=wrap_amount set_value=set_wrap_amount
                        value_display=move |v| format!("{v:.2}") />
                }}
                <Button on_click=move |_| wrap.dispatch(())>"Wrap"</Button>
            </Stack>
        </div>
    }
}
