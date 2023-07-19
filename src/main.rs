mod chain;

use std::{
    env,
    sync::{Arc, Mutex}
};

use ethabi::token::Token;
use hex_literal::hex;
use log::info;
use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_ethereum_provider::{
    AccountLabel, ConnectButton, EthereumContextProvider, SwitchNetworkButton, UseEthereumHandle, 
};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use web3::{
    contract::tokens::Tokenizable,
    // futures::StreamExt,
    transports::eip_1193::{Eip1193, Provider},
    // transports::WebSocket,
    types::{FilterBuilder, Address, BlockNumber, BlockId, Log, Bytes, U256}, contract::{Contract, Options},
    Web3
};


#[function_component]
fn App() -> Html {
    html! {
        <PageSectionGroup>
            <EthereumContextProvider>
                <Split>
                    <SplitItem>
                        <PageSection>
                            <Eth/>
                        </PageSection>
                    </SplitItem>
                    <SplitItem>
                        <PageSection
                            r#type={PageSectionType::Default}
                            variant={PageSectionVariant::Light}
                            limit_width=true
                            sticky={[PageSectionSticky::Top]}
                        >
                            <ConnectButton/>
                            <SwitchNetworkButton chain={chain::ethereum()}/>
                            <SwitchNetworkButton chain={chain::sepolia_testnet()}/>
                            <AccountLabel/>
                        </PageSection>
                    </SplitItem>
                </Split>
            </EthereumContextProvider>
        </PageSectionGroup>
    }
}

#[function_component]
pub fn Eth() -> Html {
    if let Some(ethereum) = use_context::<Option<UseEthereumHandle>>().expect(
        "No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>",
    ) {
        let transport = Eip1193::new(ethereum.provider.clone());
        let web3 = Web3::new(transport);
        let nft_contract = Contract::from_json(
            web3.eth(),
            env!("NFT_CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
            include_bytes!("../resources/MandelbrotNFT.json"),
        ).unwrap();

        html! {
            <div>
                <Mandelbrot ..MandelbrotProps { ethereum: Some(ethereum.clone()), contract: Some(nft_contract.clone()) }/>
                <Balance ..BalanceProps { ethereum: ethereum.clone(), nft_contract: nft_contract.clone() }/>
            </div>
        }
    } else {
        html! {
            <div>
                <Mandelbrot ..MandelbrotProps { ethereum: None, contract: None }/>
            </div>
        }
    }
}

#[derive(Properties)]
pub struct BalanceProps {
    pub ethereum: UseEthereumHandle,
    nft_contract: Contract<Eip1193>,
}

impl PartialEq for BalanceProps {
    fn eq(&self, other: &Self) -> bool {
        self.ethereum == other.ethereum
    }
}

#[function_component]
pub fn Balance(props: &BalanceProps) -> Html {
    let nft_contract = props.nft_contract.clone();

    let transport = Eip1193::new(props.ethereum.provider.clone());
    let web3 = Web3::new(transport);
    let wrapper_contract = Contract::from_json(
        web3.eth(),
        env!("WRAPPER_CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
        include_bytes!("../resources/Wrapped1155Factory.json"),
    ).unwrap();
    let erc20_contract = Contract::from_json(
        web3.eth(),
        env!("ERC20_CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
        include_bytes!("../resources/Wrapped1155.json"),
    ).unwrap();

    let fuel_balance = use_state(|| 0.0);
    let wfuel_balance = use_state(|| 0.0);
    let refresh_balance = {
        let ethereum = props.ethereum.clone();
        let nft_contract = nft_contract.clone();
        let fuel_balance = fuel_balance.clone();
        let wfuel_balance = wfuel_balance.clone();
        move |_| {
            let fuel_balance = fuel_balance.clone();
            let wfuel_balance = wfuel_balance.clone();
            let nft_contract = nft_contract.clone();
            let erc20_contract = erc20_contract.clone();
            if let Some(address) = ethereum.address() {
                let address = address.clone();
                spawn_local(async move {
                    let result = nft_contract.query(
                        "balanceOf",
                        (address, U256::from(0),),
                        None,
                        Options::default(),
                        None
                    );
                    if let Ok(balance) = result.await {
                        let balance: U256 = balance;
                        fuel_balance.set(balance.as_u128() as f64 / 10_f64.powi(18));
                    }

                    let result = erc20_contract.query(
                        "balanceOf",
                        (address,),
                        None,
                        Options::default(),
                        None
                    );
                    if let Ok(balance) = result.await {
                        let balance: U256 = balance;
                        wfuel_balance.set(balance.as_u128() as f64 / 10_f64.powi(18));
                    }
                });
            }
        }
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
        let nft_contract = nft_contract.clone();
        let wrapper_contract = wrapper_contract.clone();
        move |_| {
            let nft_contract = nft_contract.clone();
            let wrapper_contract = wrapper_contract.clone();
            let wrap_amount = wrap_amount.clone();
            if let Some(address) = ethereum.address() {
                let address = address.clone();
                spawn_local(async move {
                    let tx = nft_contract.call("safeTransferFrom", (
                        address,
                        wrapper_contract.address(),
                        U256::from(0),
                        U256::from((*wrap_amount * 10_f64.powi(18)) as u128),
                        hex::decode("57726170706564204d616e64656c62726f74204655454c00000000000000002e774655454c00000000000000000000000000000000000000000000000000000a12").unwrap(),
                    ), address, Options::default()).await;
                    log::info!("{:?}", tx);
                });
            }
        }
    };

    let unwrap = {
        let ethereum = props.ethereum.clone();
        move |_| {
            let nft_contract = nft_contract.clone();
            let wrapper_contract = wrapper_contract.clone();
            let unwrap_amount = unwrap_amount.clone();
            if let Some(address) = ethereum.address() {
                let address = address.clone();
                spawn_local(async move {
                    let tx = wrapper_contract.call("unwrap", (
                        nft_contract.address(),
                        U256::from(0),
                        U256::from((*unwrap_amount * 10_f64.powi(18)) as u128),
                        address,
                        hex::decode("57726170706564204d616e64656c62726f74204655454c00000000000000002e774655454c00000000000000000000000000000000000000000000000000000a12").unwrap(),
                    ), address, Options::default()).await;
                    log::info!("{:?}", tx);
                });
            }
        }
    };

    html! {
        <Grid>
            <GridItem cols={[2]} rows={[1]}><Button variant={ButtonVariant::Primary} onclick={refresh_balance}>{ "Refresh balance" }</Button></GridItem>
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

#[derive(Properties)]
pub struct MandelbrotProps {
    pub ethereum: Option<UseEthereumHandle>,
    contract: Option<Contract<Eip1193>>,
}

impl PartialEq for MandelbrotProps {
    fn eq(&self, other: &Self) -> bool {
        self.ethereum == other.ethereum
    }
}

#[derive(Debug)]
pub struct Field {
    x_min: f64,
    y_min: f64,
    x_max: f64,
    y_max: f64,
}

impl Tokenizable for Field {
    fn from_token(token: Token) -> Result<Self, web3::contract::Error> {
        match token {
            Token::Tuple(tokens) => {
                Ok(Self { 
                    x_min: U256::from_token(tokens[0].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.0,
                    y_min: U256::from_token(tokens[1].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.0,
                    x_max: U256::from_token(tokens[2].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.0,
                    y_max: U256::from_token(tokens[3].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.0
                })
            }
            _ => Err(web3::contract::Error::Abi(ethabi::Error::InvalidData)),
        }
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            U256::from(((self.x_min + 2.0) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.y_min + 2.0) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.x_max + 2.0) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.y_max + 2.0) * 10_f64.powi(18)) as u128).into_token(),
        ])
    }
}

impl web3::contract::tokens::TokenizableItem for Field {}


#[derive(Debug)]
pub struct Metadata {
    token_id: U256,
    parent_id: U256,
    field: Field,
}

impl Tokenizable for Metadata {
    fn from_token(token: Token) -> Result<Self, web3::contract::Error> {
        match token {
            Token::Tuple(tokens) => {
                Ok(Self { 
                    token_id: U256::from_token(tokens[0].clone())?,
                    parent_id: U256::from_token(tokens[1].clone())?,
                    field: Field::from_token(tokens[2].clone())?,
                })
            }
            _ => Err(web3::contract::Error::Abi(ethabi::Error::InvalidData)),
        }
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            self.token_id.into_token(),
            self.parent_id.into_token(),
            self.field.into_token(),
        ])
    }
}

impl web3::contract::tokens::TokenizableItem for Metadata {}


pub struct Mandelbrot {
    node_ref: NodeRef,
    interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
    selected_nft_id: Arc<Mutex<U256>>,
}

impl Mandelbrot {
    async fn nft_selected(
        interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
        contract: Option<Contract<Eip1193>>,
        selected_nft_id: Arc<Mutex<U256>>,
        frame: mandelbrot_explorer::Frame
    ) {
        *selected_nft_id.lock().unwrap() = U256::from(frame.id);
        if let Some(contract) = contract {
            let result = contract.query(
                "getChildrenMetadata",
                (U256::from(frame.id),),
                None,
                Options::default(),
                None
            );
            if let Ok(metadata) = result.await {
                let metadata: Vec<Metadata> = metadata;
                let frames = &mut interface.lock().unwrap().frames;
                frames.clear();
                frames.extend(metadata.iter().map(|m| mandelbrot_explorer::Frame {
                    id: m.token_id.as_u128(),
                    x_min: m.field.x_min,
                    x_max: m.field.x_max,
                    y_min: m.field.y_min,
                    y_max: m.field.y_max,
                }));
            }
        }
    }
}

impl Component for Mandelbrot {
    type Message = ();
    type Properties = MandelbrotProps;

    fn create(ctx: &Context<Self>) -> Self {
        let contract = ctx.props().contract.clone();
        let interface = Arc::new(Mutex::new(mandelbrot_explorer::Interface {
            sample_location: mandelbrot_explorer::SampleLocation::new(1500.0, 1500.0),
            frames: Vec::new(),
            frame_selected_callback: None,
        }));
        let selected_nft_id = Arc::new(Mutex::new(U256::from(1)));
        {
            let interface = interface.clone();
            let interface_clone = interface.clone();
            let selected_nft_id = selected_nft_id.clone();
            interface_clone.clone().lock().unwrap().frame_selected_callback = Some(Box::new(move |frame| {
                let interface = interface.clone();
                let contract = contract.clone();
                let selected_nft_id = selected_nft_id.clone();
                let frame = frame.clone();
                spawn_local(async move {
                    Mandelbrot::nft_selected(interface, contract, selected_nft_id, frame).await
                })
            }));
        }
        Self {
            node_ref: NodeRef::default(),
            interface,
            selected_nft_id,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ethereum = ctx.props().ethereum.clone();
        let contract = ctx.props().contract.clone();
        let interface = self.interface.clone();
        let selected_nft_id = self.selected_nft_id.clone();
        let onclick = move |_| {
            info!("onclick");
            let ethereum = ethereum.clone();
            let contract = contract.clone();
            let selected_nft_id = selected_nft_id.clone();
            let params = interface.lock().unwrap().sample_location.to_mandlebrot_params(0);
            log::info!("{:?}", params);

            spawn_local(async move {
                if let (Some(ethereum), Some(contract)) = (ethereum, contract) {
                    let chain_id = ethereum.request("eth_chainId", vec![]).await;
                    log::info!("CHAIN ID {:?}", chain_id);

                    if let Some(address) = ethereum.address() {
                        log::info!("ADDRESS {:?}", address);

                        let tx = contract.call("mintNFT", (
                            *selected_nft_id.lock().unwrap(),
                            *address,
                            Field {
                                x_min: params.x_min as f64,
                                y_min: params.y_min as f64,
                                x_max: params.x_max as f64,
                                y_max: params.y_max as f64
                            }
                        ), *address, Options::default()).await;

                        log::info!("TRANSACTION {:?}", tx);
                    }
                }
            });
        };

        html! {
            <div>
                <canvas ref={self.node_ref.clone()} width="1500" height="1500"/>
                <button {onclick}>{ "Mint" }</button>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            log::info!("FIRST RENDER");
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
            let interface = self.interface.clone();
            let selected_nft_id = self.selected_nft_id.clone();
            let contract = ctx.props().contract.clone();
            spawn_local(async move {
                if let Some(contract) = contract {
                    let result = contract.query(
                        "getChildrenMetadata",
                        (*selected_nft_id.lock().unwrap(),),
                        None,
                        Options::default(),
                        None
                    );
                    if let Ok(metadata) = result.await {
                        let metadata: Vec<Metadata> = metadata;
                        let frames = &mut interface.lock().unwrap().frames;
                        frames.clear();
                        frames.extend(metadata.iter().map(|m| mandelbrot_explorer::Frame {
                            id: m.token_id.as_u128(),
                            x_min: m.field.x_min,
                            x_max: m.field.x_max,
                            y_min: m.field.y_min,
                            y_max: m.field.y_max,
                        }));
                    }
                }
                mandelbrot_explorer::start(Some(canvas), Some(interface));
            });
        }
    }
}


fn main() {
    yew::Renderer::<App>::new().render();
}