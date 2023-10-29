use std::{collections::HashMap, sync::{Arc, Mutex}};

use leptonic::prelude::*;
use leptos::*;
use mandelbrot_explorer::FrameColor;

use crate::{
    components::blockchain::Address,
    evm::{types::Metadata, contracts::ERC1155Contract}
};


#[component]
pub fn Bids(
    erc1155_contract: ERC1155Contract,
    bids: RwSignal<HashMap<u128, Metadata>>,
) -> impl IntoView {
    let address = expect_context::<Address>().0;
    let mandelbrot = expect_context::<Arc<Mutex<mandelbrot_explorer::Interface>>>();

    let delete_bid = create_action({
        move |bid_id: &u128| {
            let erc1155_contract = erc1155_contract.clone();
            let bid_id = bid_id.clone();
            async move {
                if let Some(address) = address.get_untracked() {
                    if let Some(_) = erc1155_contract.delete_bid(address, bid_id).await {
                        bids.update(|bids| {
                            bids.remove(&bid_id);
                        });
                    }
                }
            }
        }
    });

    let zoom_bid = {
        move |bid_id| {
            if let Some(bid) = bids.get().get(&bid_id) {
                let frame = bid.to_frame(FrameColor::Blue);
                mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
            }
        }
    };

    view! {
        <Show when=move || {bids.get().len() > 0} fallback=|| {}>
            {
                let zoom_bid = zoom_bid.clone();
                view! {
                    <Box id="content">
                        <For
                            each=move || bids.get().into_values()
                            key=|bid| bid.token_id
                            children={
                                move |bid| view! {
                                    <p>
                                        {format!("Bid Id: {} Locked FUEL: {}", bid.token_id, bid.locked_fuel.to_string())}
                                        <Button on_click=move |_| delete_bid.dispatch(bid.token_id)>"Delete"</Button>
                                        <Button on_click={let zoom_bid = zoom_bid.clone(); move |_| zoom_bid(bid.token_id)}>"Zoom"</Button>
                                    </p>
                                }
                            }
                        />
                    </Box>
                }
            }
        </Show>
    }
}
