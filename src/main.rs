mod chain;
mod components;
mod evm;

use std::sync::{Arc, Mutex};

use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_ethereum_provider::{
    AccountLabel, ConnectButton, EthereumContextProvider, SwitchNetworkButton, UseEthereumHandle, 
};
use wasm_bindgen_futures::spawn_local;
use web3::{
    transports::eip_1193::Eip1193,
    Web3
};

use components::{
    balance::{Balance, BalanceProps},
    mandelbrot::{Mandelbrot, MandelbrotProps}
};
use evm::{
    contracts::ERC1155Contract,
    types::{Field, Metadata}
};


#[function_component]
fn App() -> Html {
    html! {
        <PageSectionGroup>
            <EthereumContextProvider>
                <Split>
                    <SplitItem>
                        <PageSection>
                            <EthWrapper/>
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
pub fn EthWrapper() -> Html {
    let ethereum = use_context::<Option<UseEthereumHandle>>().expect(
        "No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>",
    );
    html! {
        <div>
            <Eth ..EthProps {ethereum}/>
        </div>
    }
}


#[derive(Properties)]
pub struct EthProps {
    pub ethereum: Option<UseEthereumHandle>,
}

impl PartialEq for EthProps {
    fn eq(&self, other: &Self) -> bool {
        self.ethereum == other.ethereum
    }
}

pub struct Eth {
    interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
    selected_nft_id: Arc<Mutex<u128>>,
}

impl Component for Eth {
    type Message = ();
    type Properties = EthProps;

    fn create(ctx: &Context<Self>) -> Self {
        let interface = Arc::new(Mutex::new(mandelbrot_explorer::Interface {
            sample_location: mandelbrot_explorer::SampleLocation::new(1500.0, 1500.0),
            frames: Vec::new(),
            frame_selected_callback: None,
        }));
        let selected_nft_id = Arc::new(Mutex::new(1));

        if let Some(ethereum) = ctx.props().ethereum.clone() {
            let interface = interface.clone();
            let selected_nft_id = selected_nft_id.clone();
            let transport = Eip1193::new(ethereum.provider.clone());
            let web3 = Web3::new(transport);
            let erc1155_contract = ERC1155Contract::new(&web3);
            spawn_local(async move {
                if let Ok(metadata) = erc1155_contract.get_metadata(*selected_nft_id.lock().unwrap()).await {
                    let metadata: Metadata = metadata;
                    interface.lock().unwrap().sample_location.move_into_frame(&metadata.to_frame());
                }
            });
        }

        Self {
            interface,
            selected_nft_id,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(ethereum) = ctx.props().ethereum.clone() {
            let transport = Eip1193::new(ethereum.provider.clone());
            let web3 = Web3::new(transport);
            let erc1155_contract = ERC1155Contract::new(&web3);

            let interface = self.interface.clone();
            let selected_nft_id = self.selected_nft_id.clone();
            // let frames = self.interface.lock().unwrap().frames.clone();

            let update_frames = {
                let erc1155_contract = erc1155_contract.clone();
                let interface = interface.clone();
                move |parent_id| spawn_local({
                    let erc1155_contract = erc1155_contract.clone();
                    let interface = interface.clone();
                    async move {
                        if let Ok(metadata) = erc1155_contract.get_children_metadata(parent_id).await {
                            let metadata: Vec<Metadata> = metadata;
                            let frames = &mut interface.lock().unwrap().frames;
                            frames.clear();
                            frames.extend(metadata.iter().map(|m| m.to_frame()));
                        }
                    }
                })
            };

            update_frames(*selected_nft_id.lock().unwrap());

            let on_frame_selected = Callback::from({
                let selected_nft_id = selected_nft_id.clone();
                move |frame: mandelbrot_explorer::Frame| {
                    *selected_nft_id.lock().unwrap() = frame.id;
                    update_frames(frame.id);
                }
            });

            self.interface.lock().unwrap().frame_selected_callback = Some(Box::new({
                let on_frame_selected = on_frame_selected.clone();
                move |frame| on_frame_selected.emit(frame.clone())
            }));

            let on_mint_clicked = {
                let ethereum = ethereum.clone();
                let erc1155_contract = erc1155_contract.clone();
                let interface = self.interface.clone();
                move |_| {
                    log::info!("onclick");
                    let ethereum = ethereum.clone();
                    let erc1155_contract = erc1155_contract.clone();
                    let selected_nft_id = selected_nft_id.clone();
                    let params = interface.lock().unwrap().sample_location.to_mandlebrot_params(0);
                    log::info!("{:?}", params);

                    spawn_local(async move {
                        let chain_id = ethereum.request("eth_chainId", vec![]).await;
                        log::info!("CHAIN ID {:?}", chain_id);

                        if let Some(address) = ethereum.address() {
                            log::info!("ADDRESS {:?}", address);

                            let tx = erc1155_contract.mint(
                                *selected_nft_id.lock().unwrap(),
                                *address,
                                Field {
                                    x_min: params.x_min as f64,
                                    y_min: params.y_min as f64,
                                    x_max: params.x_max as f64,
                                    y_max: params.y_max as f64
                                }
                            ).await;

                            log::info!("TRANSACTION {:?}", tx);
                        }
                    });
                }
            };

            html! {
                <div>
                    <Mandelbrot ..MandelbrotProps {interface: self.interface.clone()}/>
                    <button onclick={on_mint_clicked}>{ "Mint" }</button>
                    <Balance ..BalanceProps { ethereum: ethereum.clone(), erc1155_contract: erc1155_contract.clone() }/>
                </div>
            }
        } else {
            html! {
                <div>
                    <Mandelbrot ..MandelbrotProps {
                        interface: self.interface.clone(),
                    }/>
                </div>
            }
        }
    }
}


fn main() {
    console_log::init_with_level(log::Level::Info).expect("could not initialize logger");
    yew::Renderer::<App>::new().render();
}