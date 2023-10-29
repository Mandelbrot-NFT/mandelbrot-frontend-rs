use std::collections::HashMap;

use leptonic::prelude::*;
use leptos::*;
use mandelbrot_explorer::FrameColor;

use crate::{
    evm::types::Metadata,
    state::State,
};


#[component]
pub fn Bids(
    bids: RwSignal<HashMap<u128, Metadata>>,
) -> impl IntoView {
    let state = use_context::<State>().unwrap();

    let delete_bid = create_action({
        move |bid_id: &u128| {
            let erc1155_contract = state.erc1155_contract.clone();
            let bid_id = bid_id.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
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
                state.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
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
                                        <Button on_click={let zoom_bid = zoom_bid.clone(); move |_| zoom_bid(bid.token_id)}>"Zoom"</Button>
                                        {format!("Bid Id: {} Locked FUEL: {}", bid.token_id, bid.locked_fuel.to_string())}
                                        <Button on_click=move |_| delete_bid.dispatch(bid.token_id)>"Delete"</Button>
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
