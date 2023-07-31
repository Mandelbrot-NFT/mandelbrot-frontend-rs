use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

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
    pub mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl PartialEq for ControllerProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[function_component]
pub fn Controller(props: &ControllerProps) -> Html {
    if let Some(ethereum) = use_context::<Option<UseEthereumHandle>>().expect(
        "No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>",
    ) {
        html! {
            <Inner ..InnerProps {ethereum, mandelbrot: props.mandelbrot.clone()}/>
        }
    } else {
        html! {}
    }
}


#[derive(Properties)]
struct InnerProps {
    pub ethereum: UseEthereumHandle,
    pub mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl PartialEq for InnerProps {
    fn eq(&self, other: &Self) -> bool {
        self.ethereum == other.ethereum
    }
}

#[derive(Clone)]
struct Inner {
    redraw: Callback<()>,
    mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
    erc1155_contract: ERC1155Contract,
    selected_nft_id: Arc<Mutex<u128>>,
    bid_amount: Arc<Mutex<f64>>,
    selected_bids: Arc<Mutex<HashSet<u128>>>,
}

impl Inner {
    fn update_frames(&self, parent_id: u128) {
        spawn_local({
            let redraw = self.redraw.clone();
            let mandelbrot = self.mandelbrot.clone();
            let erc1155_contract = self.erc1155_contract.clone();
            async move {
                if let Ok(metadata) = erc1155_contract.get_children_metadata(parent_id).await {
                    let metadata: Vec<Metadata> = metadata;
                    let frames = &mut mandelbrot.lock().unwrap().frames.red;
                    frames.clear();
                    frames.extend(metadata.iter().map(|m| m.to_frame()));
                }
                if let Ok(bids) = erc1155_contract.get_bids(parent_id).await {
                    let bids: Vec<Bid> = bids;
                    let frames = &mut mandelbrot.lock().unwrap().frames.yellow;
                    frames.clear();
                    frames.extend(bids.iter().map(|m| m.to_frame()));
                }
                redraw.emit(());
            }
        });
    }
}

impl Component for Inner {
    type Message = ();
    type Properties = InnerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mandelbrot = ctx.props().mandelbrot.clone();
        let transport = Eip1193::new(ctx.props().ethereum.provider.clone());
        let web3 = Web3::new(transport);
        let erc1155_contract = ERC1155Contract::new(&web3);
        let selected_nft_id = Arc::new(Mutex::new(1));
        {
            let mandelbrot = mandelbrot.clone();
            let erc1155_contract = erc1155_contract.clone();
            let selected_nft_id = selected_nft_id.clone();
            spawn_local(async move {
                if let Ok(metadata) = erc1155_contract.get_metadata(*selected_nft_id.lock().unwrap()).await {
                    let metadata: Metadata = metadata;
                    mandelbrot.lock().unwrap().sample_location.move_into_frame(&metadata.to_frame());
                }
            });
        }

        let this = Self {
            redraw: ctx.link().callback(|_| ()),
            mandelbrot: mandelbrot.clone(),
            erc1155_contract,
            selected_nft_id: selected_nft_id.clone(),
            bid_amount: Arc::new(Mutex::new(0.0)),
            selected_bids: Arc::new(Mutex::new(HashSet::new())),
        };

        let on_frame_selected = Callback::from({
            let this = this.clone();
            move |frame: mandelbrot_explorer::Frame| {
                *this.selected_nft_id.lock().unwrap() = frame.id;
                this.update_frames(frame.id);
            }
        });

        mandelbrot.lock().unwrap().frame_selected_callback = Some(Box::new({
            let on_frame_selected = on_frame_selected.clone();
            move |frame| on_frame_selected.emit(frame.clone())
        }));

        this.update_frames(*selected_nft_id.lock().unwrap());
        this
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ethereum = ctx.props().ethereum.clone();

        let change_bid_amount = {
            let bid_amount = self.bid_amount.clone();
            move |value: String| {
                if let Ok(value) = value.parse::<f64>() {
                    *bid_amount.lock().unwrap() = value;
                }
            }
        };

        let on_bid_clicked = {
            let this = self.clone();
            let ethereum = ethereum.clone();
            move |_| {
                let this = this.clone();
                if let Some(address) = ethereum.address() {
                    let address = address.clone();
                    let params = this.mandelbrot.lock().unwrap().sample_location.to_mandlebrot_params(0);
                    spawn_local(async move {
                        let tx = this.erc1155_contract.bid(
                            address,
                            *this.selected_nft_id.lock().unwrap(),
                            Field {
                                x_min: params.x_min as f64,
                                y_min: params.y_min as f64,
                                x_max: params.x_max as f64,
                                y_max: params.y_max as f64
                            },
                            *this.bid_amount.lock().unwrap()
                        ).await;
                        log::info!("{:?}", tx);
                    });
                }
            }
        };

        let on_mint_clicked = {
            let this = self.clone();
            let ethereum = ethereum.clone();
            move |_| {
                let this = this.clone();
                if let Some(address) = ethereum.address() {
                    let address = address.clone();
                    let params = this.mandelbrot.lock().unwrap().sample_location.to_mandlebrot_params(0);
                    spawn_local(async move {
                        this.erc1155_contract.mint(
                            address,
                            *this.selected_nft_id.lock().unwrap(),
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

        let on_bid_toggled = {
            let this = self.clone();
            move |bid_id, state| {
                if state {
                    this.selected_bids.lock().unwrap().insert(bid_id);
                } else {
                    this.selected_bids.lock().unwrap().remove(&bid_id);
                }
            }
        };

        let on_approve_clicked = {
            let this = self.clone();
            let ethereum = ethereum.clone();
            move |_| {
                log::info!("BIDS {:?}", *this.selected_bids.lock().unwrap());
                let this = this.clone();
                if let Some(address) = ethereum.address() {
                    let address = address.clone();
                    let params = this.mandelbrot.lock().unwrap().sample_location.to_mandlebrot_params(0);
                    spawn_local(async move {
                        for bid_id in &*this.selected_bids.lock().unwrap() {
                            log::info!("approve_bid {}", bid_id);
                            this.erc1155_contract.approve_bid(address, *bid_id).await;
                        }
                    });
                }
            }
        };

        html! {
            <div>
                <Stack>
                    <StackItem>
                        <TextInputGroup>
                            <TextInputGroupMain value={self.bid_amount.lock().unwrap().to_string()} r#type="number" oninput={change_bid_amount}/>
                            <button onclick={on_bid_clicked}>{ "Bid" }</button>
                        </TextInputGroup>
                    </StackItem>
                    <StackItem>
                        <button onclick={on_mint_clicked}>{ "Mint" }</button>
                    </StackItem>
                    if self.mandelbrot.lock().unwrap().frames.yellow.len() > 0 {
                        <StackItem>
                            <br/>
                            <p>{ "Bids:" }</p>
                            {
                                for self.mandelbrot.lock().unwrap().frames.yellow.iter().map(|frame| {
                                    let on_bid_toggled = on_bid_toggled.clone();
                                    let bid_id = frame.id; 
                                    html_nested!{
                                        <p><Switch label={bid_id.to_string()} onchange={move |state| on_bid_toggled(bid_id, state)}/></p>
                                    }
                                })
                            }
                            <p><button onclick={on_approve_clicked}>{ "Approve" }</button></p>
                        </StackItem>
                    }
                </Stack>
            </div>
        }
    }
}
