use std::{
    collections::HashMap,
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
            <Inner ethereum={ethereum} mandelbrot={props.mandelbrot.clone()}/>
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
    nav_history: Arc<Mutex<Vec<Metadata>>>,
    children: Arc<Mutex<HashMap<u128, Metadata>>>,
    bids: Arc<Mutex<HashMap<u128, Bid>>>,
    bid_amount: Arc<Mutex<f64>>,
    approve_amount_node_ref: NodeRef,
}

impl Inner {
    fn obtain_tokens(&self, parent_id: u128) {
        spawn_local({
            let this = self.clone();
            async move {
                if let Ok(metadata) = this.erc1155_contract.get_children_metadata(parent_id).await {
                    let children = &mut (*this.children.lock().unwrap());
                    children.clear();
                    children.extend(metadata.into_iter().map(|m: Metadata| (m.token_id, m)));
                }
                if let Ok(bids) = this.erc1155_contract.get_bids(parent_id).await {
                    let bids_ = &mut (*this.bids.lock().unwrap());
                    bids_.clear();
                    bids_.extend(bids.into_iter().map(|bid| (bid.bid_id, bid)));
                }
                this.update_frames();
                this.redraw.emit(());
            }
        });
    }

    fn update_frames(&self) {
        let frames = &mut self.mandelbrot.lock().unwrap().frames;
        frames.clear();
        frames.extend(self.children.lock().unwrap().values().map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Red)));
        frames.extend(self.nav_history.lock().unwrap().iter().rev().map(|token| token.to_frame(mandelbrot_explorer::FrameColor::Blue)));
        frames.extend(self.bids.lock().unwrap().values().map(|token| token.to_frame()));
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
        let nav_history = Arc::new(Mutex::new(Vec::new()));
        {
            let mandelbrot = mandelbrot.clone();
            let erc1155_contract = erc1155_contract.clone();
            let selected_nft_id = selected_nft_id.clone();
            let nav_history = nav_history.clone();
            spawn_local(async move {
                if let Ok(metadata) = erc1155_contract.get_metadata(*selected_nft_id.lock().unwrap()).await {
                    mandelbrot.lock().unwrap().sample_location.move_into_frame(&metadata.to_frame(mandelbrot_explorer::FrameColor::Blue));
                    nav_history.lock().unwrap().push(metadata);
                }
            });
        }

        let this = Self {
            redraw: ctx.link().callback(|_| ()),
            mandelbrot: mandelbrot.clone(),
            erc1155_contract,
            selected_nft_id: selected_nft_id.clone(),
            nav_history,
            children: Arc::new(Mutex::new(HashMap::new())),
            bids: Arc::new(Mutex::new(HashMap::new())),
            bid_amount: Arc::new(Mutex::new(0.0)),
            approve_amount_node_ref: NodeRef::default(),
        };

        let on_frame_selected = Callback::from({
            let this = this.clone();
            let mandelbrot = mandelbrot.clone();
            move |frame: mandelbrot_explorer::Frame| {
                *this.selected_nft_id.lock().unwrap() = frame.id;
                match frame.color {
                    mandelbrot_explorer::FrameColor::Red | mandelbrot_explorer::FrameColor::Blue => {
                        mandelbrot.lock().unwrap().sample_location.move_into_frame(&frame);
                        let nav_history = &mut this.nav_history.lock().unwrap();
                        for (token_id, token) in this.children.lock().unwrap().iter() {
                            if *token_id == frame.id {
                                nav_history.push(token.clone());
                                break
                            }
                        }
                        for (i, token) in nav_history.iter().enumerate().rev() {
                            if token.token_id == frame.id {
                                nav_history.truncate(i + 1);
                                break
                            }
                        }
                        this.obtain_tokens(frame.id);
                    }
                    _ => {}
                }
            }
        });

        mandelbrot.lock().unwrap().frame_selected_callback = Some(Arc::new({
            let on_frame_selected = on_frame_selected.clone();
            move |frame| on_frame_selected.emit(frame.clone())
        }));

        this.obtain_tokens(*selected_nft_id.lock().unwrap());
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

        // let on_mint_clicked = {
        //     let this = self.clone();
        //     let ethereum = ethereum.clone();
        //     move |_| {
        //         let this = this.clone();
        //         if let Some(address) = ethereum.address() {
        //             let address = address.clone();
        //             let params = this.mandelbrot.lock().unwrap().sample_location.to_mandlebrot_params(0);
        //             spawn_local(async move {
        //                 this.erc1155_contract.mint(
        //                     address,
        //                     *this.selected_nft_id.lock().unwrap(),
        //                     Field {
        //                         x_min: params.x_min as f64,
        //                         y_min: params.y_min as f64,
        //                         x_max: params.x_max as f64,
        //                         y_max: params.y_max as f64
        //                     }
        //                 ).await;
        //             });
        //         }
        //     }
        // };

        let on_bid_toggled = {
            let this = self.clone();
            move |bid_id, state| {
                {
                    let mut bids_lock = this.bids.lock().unwrap();
                    if let Some(bid) = bids_lock.get_mut(&bid_id) {
                        bid.selected = state;

                        let total_approve_amount: f64 = bids_lock.values()
                            .filter(|bid| bid.selected)
                            .map(|bid| bid.amount)
                            .sum();
                        this.approve_amount_node_ref.get().unwrap().set_text_content(Some(&total_approve_amount.to_string()));
                    }
                }
                this.update_frames();
            }
        };

        let on_approve_clicked = {
            let this = self.clone();
            let ethereum = ethereum.clone();
            move |_| {
                let this = this.clone();
                if let Some(address) = ethereum.address() {
                    let address = address.clone();
                    spawn_local(async move {
                        let selected_bids: Vec<u128> = this.bids.lock().unwrap()
                            .values()
                            .filter(|bid| bid.selected)
                            .map(|bid| bid.bid_id)
                            .collect();
                        this.erc1155_contract.batch_approve_bids(address, &selected_bids).await;
                    });
                }
            }
        };

        let bids_lock = self.bids.lock().unwrap();
        let mut bids: Vec<&Bid> = bids_lock.values().collect();
        bids.sort_by(|bid_a, bid_b| bid_a.amount.partial_cmp(&bid_b.amount).unwrap());
        let total_approve_amount: f64 = bids.iter().filter(|bid| bid.selected).map(|bid| bid.amount).sum();

        html! {
            <div>
                <Stack>
                    <StackItem>
                        <TextInputGroup>
                            <TextInputGroupMain value={self.bid_amount.lock().unwrap().to_string()} r#type="number" oninput={change_bid_amount}/>
                            <button onclick={on_bid_clicked}>{ "Bid" }</button>
                        </TextInputGroup>
                    </StackItem>
                    // <StackItem>
                    //     <button onclick={on_mint_clicked}>{ "Mint" }</button>
                    // </StackItem>
                    if bids.len() > 0 {
                        <StackItem>
                            <br/>
                            <p>{ "Bids:" }</p>
                            {
                                for bids.iter().map(|bid| {
                                    let on_bid_toggled = on_bid_toggled.clone();
                                    let bid_id = bid.bid_id;
                                    let amount = bid.amount;
                                    html_nested!{
                                        <p><Switch label={amount.to_string()} onchange={move |state| on_bid_toggled(bid_id, state)}/></p>
                                    }
                                })
                            }
                            <p>
                                <label ref={self.approve_amount_node_ref.clone()}>{ total_approve_amount }</label>
                                <button onclick={on_approve_clicked}>{ "Approve" }</button>
                            </p>
                        </StackItem>
                    }
                </Stack>
            </div>
        }
    }
}
