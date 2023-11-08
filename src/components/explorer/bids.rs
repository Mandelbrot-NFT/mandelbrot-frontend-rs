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

    let zoom_bid = {
        move |bid_id| {
            if let Some(bid) = bids.get().get(&bid_id) {
                let frame = bid.to_frame(FrameColor::Blue);
                state.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
            }
        }
    };

    let sorted_bids = create_memo(move |_| {
        let mut bids: Vec<Metadata> = bids.get().values().map(|bid| bid.clone()).collect();
        bids.sort_by(|bid_a, bid_b| bid_b.locked_fuel.partial_cmp(&bid_a.locked_fuel).unwrap());
        bids
    });

    view! {
        <p>"Bids:"</p>
        <Box id="content">
            <For
                each=move || sorted_bids.get()
                key=|bid| bid.token_id
                children={
                    move |bid| view! {
                        <p>
                            {format!("{} {:?}", bid.locked_fuel.to_string(), bid.owner)}
                            <Button on_click={let zoom_bid = zoom_bid.clone(); move |_| zoom_bid(bid.token_id)}>"Zoom"</Button>
                        </p>
                    }
                }
            />
        </Box>
    }
}
