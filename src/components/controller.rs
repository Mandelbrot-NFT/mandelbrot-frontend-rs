use std::sync::{Arc, Mutex};

use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_ethereum_provider::UseEthereumHandle;
use wasm_bindgen_futures::spawn_local;
use web3::{
    transports::eip_1193::Eip1193,
    Web3
};

use crate::evm::{
    contracts::ERC1155Contract,
    types::{Bid, Field, Metadata}
};


#[derive(Properties)]
pub struct ControllerProps {
    pub interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl PartialEq for ControllerProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[function_component]
pub fn Controller(props: &ControllerProps) -> Html {
    let ethereum = use_context::<Option<UseEthereumHandle>>().expect(
        "No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>",
    );
    html! {
        <div>
            <Inner ..InnerProps {ethereum, interface: props.interface.clone()}/>
        </div>
    }
}


#[derive(Properties)]
struct InnerProps {
    pub ethereum: Option<UseEthereumHandle>,
    pub interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl PartialEq for InnerProps {
    fn eq(&self, other: &Self) -> bool {
        self.ethereum == other.ethereum
    }
}

struct Inner {
    selected_nft_id: Arc<Mutex<u128>>,
    bid_amount: Arc<Mutex<f64>>,
}

impl Component for Inner {
    type Message = ();
    type Properties = InnerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let selected_nft_id = Arc::new(Mutex::new(1));

        if let Some(ethereum) = ctx.props().ethereum.clone() {
            let interface = ctx.props().interface.clone();
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
            selected_nft_id,
            bid_amount: Arc::new(Mutex::new(0.0)),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(ethereum) = ctx.props().ethereum.clone() {
            let transport = Eip1193::new(ethereum.provider.clone());
            let web3 = Web3::new(transport);
            let erc1155_contract = ERC1155Contract::new(&web3);

            let interface = ctx.props().interface.clone();
            let selected_nft_id = self.selected_nft_id.clone();

            let update_frames = {
                let erc1155_contract = erc1155_contract.clone();
                let interface = interface.clone();
                move |parent_id| spawn_local({
                    let erc1155_contract = erc1155_contract.clone();
                    let interface = interface.clone();
                    async move {
                        if let Ok(metadata) = erc1155_contract.get_children_metadata(parent_id).await {
                            let metadata: Vec<Metadata> = metadata;
                            let frames = &mut interface.lock().unwrap().frames.red;
                            frames.clear();
                            frames.extend(metadata.iter().map(|m| m.to_frame()));
                        }
                        if let Ok(bids) = erc1155_contract.get_bids(parent_id).await {
                            let bids: Vec<Bid> = bids;
                            let frames = &mut interface.lock().unwrap().frames.yellow;
                            frames.clear();
                            frames.extend(bids.iter().map(|m| m.to_frame()));
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

            interface.lock().unwrap().frame_selected_callback = Some(Box::new({
                let on_frame_selected = on_frame_selected.clone();
                move |frame| on_frame_selected.emit(frame.clone())
            }));

            let change_bid_amount = {
                let bid_amount = self.bid_amount.clone();
                move |value: String| {
                    if let Ok(value) = value.parse::<f64>() {
                        *bid_amount.lock().unwrap() = value;
                    }
                }
            };

            let on_bid_clicked = {
                let ethereum = ethereum.clone();
                let erc1155_contract = erc1155_contract.clone();
                let interface = interface.clone();
                let selected_nft_id = selected_nft_id.clone();
                let bid_amount = self.bid_amount.clone();
                move |_| {
                    if let Some(address) = ethereum.address() {
                        let address = address.clone();
                        let erc1155_contract = erc1155_contract.clone();
                        let selected_nft_id = selected_nft_id.clone();
                        let bid_amount = bid_amount.clone();
                        let params = interface.lock().unwrap().sample_location.to_mandlebrot_params(0);
                        spawn_local(async move {
                            let tx = erc1155_contract.bid(
                                *selected_nft_id.lock().unwrap(),
                                address,
                                Field {
                                    x_min: params.x_min as f64,
                                    y_min: params.y_min as f64,
                                    x_max: params.x_max as f64,
                                    y_max: params.y_max as f64
                                },
                                *bid_amount.lock().unwrap()
                            ).await;
                            log::info!("{:?}", tx);
                        });
                    }
                }
            };

            let on_mint_clicked = {
                let ethereum = ethereum.clone();
                let erc1155_contract = erc1155_contract.clone();
                let interface = interface.clone();
                move |_| {
                    if let Some(address) = ethereum.address() {
                        let address = address.clone();
                        let erc1155_contract = erc1155_contract.clone();
                        let selected_nft_id = selected_nft_id.clone();
                        let params = interface.lock().unwrap().sample_location.to_mandlebrot_params(0);
                        spawn_local(async move {
                            erc1155_contract.mint(
                                *selected_nft_id.lock().unwrap(),
                                address,
                                Field {
                                    x_min: params.x_min as f64,
                                    y_min: params.y_min as f64,
                                    x_max: params.x_max as f64,
                                    y_max: params.y_max as f64
                                }
                            ).await;
                        });
                    }
                }
            };

            html! {
                <div>
                    <TextInputGroup>
                        <TextInputGroupMain value={self.bid_amount.lock().unwrap().to_string()} r#type="number" oninput={change_bid_amount}/>
                        <button onclick={on_bid_clicked}>{ "Bid" }</button>
                    </TextInputGroup>
                    <button onclick={on_mint_clicked}>{ "Mint" }</button>
                </div>
            }
        } else {
            html! {}
        }
    }
}
