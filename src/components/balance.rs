use eyre::Result;
use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_ethereum_provider::UseEthereumHandle;
use wasm_bindgen_futures::spawn_local;
use web3::{
    transports::eip_1193::Eip1193,
    types::Address,
    Web3
};

use crate::evm::contracts::{
    ERC1155Contract,
    Wrapped1155FactoryContract,
    ERC20Contract
};


#[derive(Properties)]
pub struct BalanceProps {
    pub ethereum: UseEthereumHandle,
    pub erc1155_contract: ERC1155Contract,
}

impl PartialEq for BalanceProps {
    fn eq(&self, other: &Self) -> bool {
        self.ethereum == other.ethereum
    }
}

async fn get_balance(
    address: Address,
    erc1155_contract: ERC1155Contract,
    erc20_contract: ERC20Contract,
) -> Result<(f64, f64)> {
    Ok((erc1155_contract.get_fuel_balance(address).await?, erc20_contract.get_balance(address).await?))
}

#[function_component]
pub fn Balance(props: &BalanceProps) -> Html {
    let erc1155_contract = props.erc1155_contract.clone();

    let transport = Eip1193::new(props.ethereum.provider.clone());
    let web3 = Web3::new(transport);
    let wrapper_contract = Wrapped1155FactoryContract::new(&web3, erc1155_contract.address());
    let erc20_contract = ERC20Contract::new(&web3);

    let fuel_balance = use_state(|| 0.0);
    let wfuel_balance = use_state(|| 0.0);
    let refresh_balance = {
        let ethereum = props.ethereum.clone();
        let erc1155_contract = erc1155_contract.clone();
        let erc20_contract = erc20_contract.clone();
        let fuel_balance = fuel_balance.clone();
        let wfuel_balance = wfuel_balance.clone();
        move || {
            let fuel_balance = fuel_balance.clone();
            let wfuel_balance = wfuel_balance.clone();
            let erc1155_contract = erc1155_contract.clone();
            let erc20_contract = erc20_contract.clone();
            if let Some(address) = ethereum.address() {
                let address = address.clone();
                spawn_local(async move {
                    if let Ok((fuel_balance_, wfuel_balance_)) = get_balance(address, erc1155_contract, erc20_contract).await {
                        fuel_balance.set(fuel_balance_);
                        wfuel_balance.set(wfuel_balance_);
                    }
                });
            }
        }
    };
    let refresh_balance_onclick = {
        let refresh_balance = refresh_balance.clone();
        move |_| refresh_balance()
    };

    let wrap_amount = use_state(|| 0.0);
    let wrap_amount_str = use_state(|| "0.0".to_owned());
    let change_wrap_amount = {
        let wrap_amount = wrap_amount.clone();
        let wrap_amount_str = wrap_amount_str.clone();
        move |value: f64| {
            wrap_amount.set(value);
            wrap_amount_str.set(format!("{value:.2}"));
        }
    };

    let unwrap_amount = use_state(|| 0.0);
    let unwrap_amount_str = use_state(|| "0.0".to_owned());
    let change_unwrap_amount = {
        let unwrap_amount = unwrap_amount.clone();
        let unwrap_amount_str = unwrap_amount_str.clone();
        move |value: f64| {
            unwrap_amount.set(value);
            unwrap_amount_str.set(format!("{value:.2}"));
        }
    };

    let wrap = {
        let ethereum = props.ethereum.clone();
        let erc1155_contract = erc1155_contract.clone();
        let wrapper_contract = wrapper_contract.clone();
        let erc20_contract = erc20_contract.clone();
        let fuel_balance = fuel_balance.clone();
        let wfuel_balance = wfuel_balance.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let wrapper_contract = wrapper_contract.clone();
            let erc20_contract = erc20_contract.clone();
            let fuel_balance = fuel_balance.clone();
            let wfuel_balance = wfuel_balance.clone();
            let wrap_amount = wrap_amount.clone();
            if let Some(address) = ethereum.address() {
                let address = address.clone();
                spawn_local(async move {
                    erc1155_contract.transfer_fuel(address, wrapper_contract.address(), *wrap_amount).await;
                    if let Ok((fuel_balance_, wfuel_balance_)) = get_balance(address, erc1155_contract, erc20_contract).await {
                        fuel_balance.set(fuel_balance_);
                        wfuel_balance.set(wfuel_balance_);
                    }
                });
            }
        }
    };

    let unwrap = {
        let ethereum = props.ethereum.clone();
        let fuel_balance = fuel_balance.clone();
        let wfuel_balance = wfuel_balance.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let wrapper_contract = wrapper_contract.clone();
            let erc20_contract = erc20_contract.clone();
            let fuel_balance = fuel_balance.clone();
            let wfuel_balance = wfuel_balance.clone();
            let unwrap_amount = unwrap_amount.clone();
            if let Some(address) = ethereum.address() {
                let address = address.clone();
                spawn_local(async move {
                    wrapper_contract.unwrap(address, *unwrap_amount).await;
                    if let Ok((fuel_balance_, wfuel_balance_)) = get_balance(address, erc1155_contract, erc20_contract).await {
                        fuel_balance.set(fuel_balance_);
                        wfuel_balance.set(wfuel_balance_);
                    }
                });
            }
        }
    };

    refresh_balance();

    html! {
        <Grid>
            <GridItem cols={[2]} rows={[1]}><Button variant={ButtonVariant::Primary} onclick={refresh_balance_onclick}>{ "Refresh balance" }</Button></GridItem>
            <GridItem cols={[10]} rows={[1]}/>

            <GridItem cols={[3]} rows={[1]}><strong>{ "FUEL: " }</strong> {*fuel_balance} </GridItem>
            <GridItem cols={[7]} rows={[1]}><Slider min=0f64 max={*fuel_balance} onchange={change_wrap_amount}/></GridItem>
            <GridItem cols={[1]} rows={[1]}> { (*wrap_amount_str).clone() } </GridItem>
            <GridItem cols={[1]} rows={[1]}><Button variant={ButtonVariant::Primary} onclick={wrap}>{ "Wrap" }</Button></GridItem>

            <GridItem cols={[3]} rows={[1]}><strong>{ "wFUEL: " }</strong> {*wfuel_balance} </GridItem>
            <GridItem cols={[7]} rows={[1]}><Slider min=0f64 max={*wfuel_balance} onchange={change_unwrap_amount}/></GridItem>
            <GridItem cols={[1]} rows={[1]}> { (*unwrap_amount_str).clone() } </GridItem>
            <GridItem cols={[1]} rows={[1]}><Button variant={ButtonVariant::Primary} onclick={unwrap}>{ "Unwrap" }</Button></GridItem>
        </Grid>
    }
}
