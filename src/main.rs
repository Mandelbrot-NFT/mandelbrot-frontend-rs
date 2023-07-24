mod chain;
mod components;
mod evm;

use std::sync::{Arc, Mutex};

use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_ethereum_provider::{
    AccountLabel, ConnectButton, EthereumContextProvider, UseEthereumHandle, 
};
use wasm_bindgen::JsCast;
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
    types::{Bid, Field, Metadata}
};


#[function_component]
fn App() -> Html {
    let window = web_sys::window().unwrap();
    let height = use_state(|| (window.inner_height().unwrap().as_f64().unwrap() + 1.0) * 2.0);

    let resize_callback: wasm_bindgen::closure::Closure<dyn FnMut()> = wasm_bindgen::closure::Closure::new({
        let window = window.clone();
        let height = height.clone();
        move || height.set((window.inner_height().unwrap().as_f64().unwrap() + 1.0) * 2.0)
    });
    window.set_onresize(Some(use_state(|| resize_callback).as_ref().unchecked_ref()));

    let interface = Arc::new(Mutex::new(mandelbrot_explorer::Interface {
        sample_location: mandelbrot_explorer::SampleLocation::new(*height, *height),
        red_frames: Vec::new(),
        yellow_frames: Vec::new(),
        frame_selected_callback: None,
    }));

    html! {
        <Split>
            <SplitItem fill={true}>
                <Mandelbrot ..MandelbrotProps {size: (*height, *height), interface: interface.clone()}/>
            </SplitItem>
            <SplitItem>
                <EthereumContextProvider>
                    <PageSection
                        r#type={PageSectionType::Default}
                        variant={PageSectionVariant::Light}
                        limit_width=true
                        sticky={[PageSectionSticky::Top]}
                        fill={PageSectionFill::Fill}
                    >
                        <ConnectButton/>
                        <AccountLabel/>
                    </PageSection>
                    <PageSection>
                        <Eth ..EthProps {interface}/>
                    </PageSection>
                </EthereumContextProvider>
            </SplitItem>
        </Split>
    }
}


#[derive(Properties)]
pub struct EthProps {
    interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl PartialEq for EthProps {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

#[function_component]
pub fn Eth(props: &EthProps) -> Html {
    let interface = props.interface.clone();
    let selected_nft_id = Arc::new(Mutex::new(1));

    let bid_amount = use_state(|| 0.0);

    if let Some(ethereum) = use_context::<Option<UseEthereumHandle>>().expect(
        "No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>",
    ) {
        let interface = interface.clone();
        let selected_nft_id = selected_nft_id.clone();
        let transport = Eip1193::new(ethereum.provider.clone());
        let web3 = Web3::new(transport);
        let erc1155_contract = ERC1155Contract::new(&web3);
        spawn_local({
            let interface = interface.clone();
            let selected_nft_id = selected_nft_id.clone();
            async move {
                if let Ok(metadata) = erc1155_contract.get_metadata(*selected_nft_id.lock().unwrap()).await {
                    let metadata: Metadata = metadata;
                    interface.lock().unwrap().sample_location.move_into_frame(&metadata.to_frame());
                }
            }
        });

        let transport = Eip1193::new(ethereum.provider.clone());
        let web3 = Web3::new(transport);
        let erc1155_contract = ERC1155Contract::new(&web3);

        let interface = interface.clone();
        let selected_nft_id = selected_nft_id.clone();

        let update_frames = {
            let erc1155_contract = erc1155_contract.clone();
            let interface = interface.clone();
            move |parent_id| spawn_local({
                let erc1155_contract = erc1155_contract.clone();
                let interface = interface.clone();
                async move {
                    if let Ok(metadata) = erc1155_contract.get_children_metadata(parent_id).await {
                        let metadata: Vec<Metadata> = metadata;
                        let frames = &mut interface.lock().unwrap().red_frames;
                        frames.clear();
                        frames.extend(metadata.iter().map(|m| m.to_frame()));
                    }
                    if let Ok(bids) = erc1155_contract.get_bids(parent_id).await {
                        let bids: Vec<Bid> = bids;
                        let frames = &mut interface.lock().unwrap().yellow_frames;
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
            let bid_amount = bid_amount.clone();
            move |value: String| {
                if let Ok(value) = value.parse::<f64>() {
                    bid_amount.set(value);
                }
            }
        };

        let on_bid_clicked = {
            let ethereum = ethereum.clone();
            let erc1155_contract = erc1155_contract.clone();
            let interface = interface.clone();
            let selected_nft_id = selected_nft_id.clone();
            let bid_amount = bid_amount.clone();
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
                            *bid_amount
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
                    <TextInputGroupMain value={bid_amount.to_string()} r#type="number" oninput={change_bid_amount}/>
                    <button onclick={on_bid_clicked}>{ "Bid" }</button>
                </TextInputGroup>
                <button onclick={on_mint_clicked}>{ "Mint" }</button>
                <Balance ..BalanceProps { ethereum: ethereum.clone(), erc1155_contract: erc1155_contract.clone() }/>
            </div>
        }
    } else {
        html! {}
    }
}


/// Parse the query string as returned by `web_sys::window()?.location().search()?` and get a
/// specific key out of it.
pub fn parse_url_query_string<'a>(query: &'a str, search_key: &str) -> Option<&'a str> {
    let query_string = query.strip_prefix('?')?;

    for pair in query_string.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next()?;
        let value = pair.next()?;

        if key == search_key {
            return Some(value);
        }
    }

    None
}


fn main() {
    let query_string = web_sys::window().unwrap().location().search().unwrap();
    let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
        .and_then(|x| x.parse().ok())
        .unwrap_or(log::Level::Error);
    console_log::init_with_level(level).expect("could not initialize logger");
    yew::Renderer::<App>::new().render();
}