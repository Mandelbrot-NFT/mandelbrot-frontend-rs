mod chain;

use std::{
    env,
    sync::{Arc, Mutex}
};

use ethabi::token::Token;
use hex_literal::hex;
use log::info;
use yew::prelude::*;
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
use yew_ethereum_provider::{
    AccountLabel, ConnectButton, EthereumContextProvider, SwitchNetworkButton, UseEthereumHandle, 
};


#[function_component]
fn App() -> Html {
    html! {
        <div>
            <EthereumContextProvider>
                <ConnectButton/>
                <SwitchNetworkButton chain={chain::ethereum()}/>
                <SwitchNetworkButton chain={chain::sepolia_testnet()}/>
                <SwitchNetworkButton chain={yew_ethereum_provider::chain::avalanche_testnet()}/>
                <AccountLabel/>
                <Eth/>
            </EthereumContextProvider>
        </div>
    }
}

#[function_component]
pub fn Eth() -> Html {
    let (ethereum, contract) = if let Some(ethereum) = use_context::<Option<UseEthereumHandle>>().expect(
        "No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>",
    ) {
        let transport = Eip1193::new(ethereum.provider.clone());
        let web3 = Web3::new(transport);
        let contract = Contract::from_json(
            web3.eth(),
            env!("CONTRACT_ADDRESS")
                .trim_start_matches("0x")
                .parse()
                .unwrap(),
            include_bytes!("../resources/MandelbrotNFT.json"),
        ).unwrap();

        (Some(ethereum), Some(contract))
    } else {
        (None, None)
    };

    html! {
        <Mandelbrot ..MandelbrotProps { ethereum, contract }/>
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

pub struct Mandelbrot {
    node_ref: NodeRef,
    interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
    fields: Arc<Mutex<Vec<Field>>>,
}

impl Component for Mandelbrot {
    type Message = ();
    type Properties = MandelbrotProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            interface: Arc::new(Mutex::new(mandelbrot_explorer::Interface {
                sample_location: mandelbrot_explorer::SampleLocation::new(1500.0, 1500.0),
                frames: Vec::new()
            })),
            fields: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ethereum = ctx.props().ethereum.clone();
        let contract = ctx.props().contract.clone();
        let interface = self.interface.clone();
        let fields = self.fields.clone();
        let onclick = move |_| {
            info!("onclick");
            log::info!("SENDER BALANCE {:?}", fields.lock());
            let ethereum = ethereum.clone();
            let contract = contract.clone();
            let params = interface.lock().unwrap().sample_location.to_mandlebrot_params(0);
            log::info!("{:?}", params);

            spawn_local(async move {
                if let (Some(ethereum), Some(contract)) = (ethereum, contract) {
                    let chain_id = ethereum.request("eth_chainId", vec![]).await;
                    log::info!("CHAIN ID {:?}", chain_id);
                    let address = ethereum.address().unwrap();
                    log::info!("ADDRESS {:?}", address);

                    let tx = contract.call("mintNFT", (
                        *address,
                        Field { 
                            x_min: params.x_min as f64,
                            y_min: params.y_min as f64,
                            x_max: params.x_max as f64,
                            y_max: params.y_max as f64
                        }
                    ), *address, Options::default()).await;

                    log::info!("TRANSACTION {:?}", tx);

                    let result = contract.query(
                        "fields",
                        (),
                        None,
                        Options::default(),
                        None
                    );
                    if let Ok(balance_of) = result.await {
                        let balance_of: Vec<Field> = balance_of;
                        log::info!("SENDER BALANCE {:?}", balance_of);
                    } else {
                        log::info!("BALANCE OF ERROR");
                    }
                }
            });
        };

        html! {
            <>
                <p><canvas ref={self.node_ref.clone()} width="1500" height="1500"/></p>
                <p><button {onclick}>{ "Mint" }</button></p>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            log::info!("FIRST RENDER");
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
            let interface = self.interface.clone();
            let contract = ctx.props().contract.clone();
            spawn_local(async move {
                if let Some(contract) = contract {
                    let result = contract.query(
                        "fields",
                        (),
                        None,
                        Options::default(),
                        None
                    );
                    if let Ok(fields) = result.await {
                        let mut fields: Vec<Field> = fields;
                        let frames = &mut interface.lock().unwrap().frames;
                        frames.clear();
                        frames.extend(fields.iter().map(|field| mandelbrot_explorer::Frame {
                            x_min: field.x_min,
                            x_max: field.x_max,
                            y_min: field.y_min,
                            y_max: field.y_max,
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