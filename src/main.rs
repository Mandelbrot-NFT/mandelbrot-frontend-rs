mod chain;

use std::sync::{Arc, Mutex};

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
    use_ethereum, AccountLabel, ConnectButton, EthereumContextProvider, SwitchNetworkButton, UseEthereumHandle, 
};


#[function_component]
fn App() -> Html {
    // let counter = use_state(|| 0);
    // let onclick = {
    //     let counter = counter.clone();
    //     move |_| {
    //         let value = *counter + 1;
    //         counter.set(value);
    //     }
    // };

    html! {
        <div>
            <EthereumContextProvider>
                <ConnectButton />
                <SwitchNetworkButton chain={chain::ethereum()}/>
                <SwitchNetworkButton chain={chain::sepolia_testnet()}/>
                <SwitchNetworkButton chain={yew_ethereum_provider::chain::avalanche_testnet()}/>
                <AccountLabel />
                <Eth />
            </EthereumContextProvider>
            // <button {onclick}>{ "+1" }</button>
            // <p>{ *counter }</p>
        </div>
    }
}

#[function_component]
pub fn Eth() -> Html {
    let ethereum = use_context::<UseEthereumHandle>().expect(
        "No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>",
    );
    let transport = Eip1193::new(ethereum.provider.clone());
    let web3 = Web3::new(transport);
    let contract = Contract::from_json(
        web3.eth(),
        "6FFCf0b2D7cEC89d4D73419ca60B43b6793f8526".parse().unwrap(),
        include_bytes!("../resources/MandelbrotNFT.json"),
    ).unwrap();

    html! {
        <Mandelbrot ..MandelbrotProps { ethereum, contract } />
    }
}

#[derive(Properties)]
pub struct MandelbrotProps {
    pub ethereum: UseEthereumHandle,
    contract: Contract<Eip1193>,
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
    location: Arc<Mutex<mandelbrot_explorer::SampleLocation>>,
    fields: Arc<Mutex<Vec<Field>>>,
}

impl Component for Mandelbrot {
    type Message = ();
    type Properties = MandelbrotProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            location: Arc::new(Mutex::new(mandelbrot_explorer::SampleLocation::new(1000.0, 1000.0))),
            fields: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ethereum = ctx.props().ethereum.clone();
        let contract = ctx.props().contract.clone();
        let location = self.location.clone();
        let fields = self.fields.clone();
        let onclick = move |_| {
            info!("onclick");
            log::info!("SENDER BALANCE {:?}", fields.lock());
            let ethereum = ethereum.clone();
            let contract = contract.clone();
            let params = location.lock().unwrap().to_mandlebrot_params(0);
            log::info!("{:?}", params);

            spawn_local(async move {
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
                // let qwe = result.await;
                // let result = contract.query(
                //     "balanceOf",
                //     (*address, U256::from(0)),
                //     None,
                //     Options::default(),
                //     None
                // );
                // let qwe = result.await;
                // log::info!("ZXC {:?}", qwe);
                // if let Ok(balance_of) = qwe {
                //     let balance_of: U256 = balance_of;
                //     log::info!("SENDER BALANCE {:?}", balance_of);
                // } else {
                //     log::info!("BALANCE OF ERROR");
                // }

                // let result = contract.query(
                //     "balanceOf",
                //     (Address::from(hex!("106EbfED93c3E174F798438D39f718D227b01906")), U256::from(1)),
                //     None,
                //     Options::default(),
                //     None
                // );
                // if let Ok(balance_of) = result.await {
                //     let balance_of: U256 = balance_of;
                //     log::info!("RECEIVER BALANCE {:?}", balance_of);
                // }
            });
        };

        html! {
            <>
                <button {onclick}>{ "Get coords" }</button>
                <canvas ref={self.node_ref.clone()} width="1000" height="1000"></canvas>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            log::info!("FIRST RENDER");
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
            let contract = ctx.props().contract.clone();
            let location = self.location.clone();
            let fields = self.fields.clone();
            spawn_local(async move {
                let result = contract.query(
                    "fields",
                    (),
                    None,
                    Options::default(),
                    None
                );
                if let Ok(balance_of) = result.await {
                    let mut balance_of: Vec<Field> = balance_of;
                    log::info!("FIELDS: {:?}", balance_of);
                    fields.lock().unwrap().append(&mut balance_of);
                    mandelbrot_explorer::start(Some(canvas), Some(location));
                }
            });
        }
    }
}


fn consume(qwe: &String) {

}


fn mutate(qwe: &mut String) {

}


fn main() {
    let mut qwe = "qwe".to_owned();
    consume(&qwe);
    mutate(&mut qwe);

    println!("HELLO {}", qwe);
    yew::Renderer::<App>::new().render();
}