use leptonic::prelude::*;
use leptos::*;

use crate::{
    evm::types::{Field, Metadata},
    state::State,
};


#[component]
pub fn Auction(
    token: Metadata,
) -> impl IntoView {
    let state = use_context::<State>().unwrap();

    let (bid_amount, set_bid_amount) = create_signal(token.minimum_price);
    let (bids_minimum_price, set_bids_minimum_price) = create_signal(token.minimum_price);

    let create_bid = create_action({
        move |token_id| {
            let erc1155_contract = state.erc1155_contract.clone();
            let mandelbrot = state.mandelbrot.clone();
            let token_id = *token_id;
            async move {
                if let Some(address) = state.address.get_untracked() {
                    let bounds = mandelbrot.lock().unwrap().sample.borrow().get_bounds();
                    erc1155_contract.bid(
                        address,
                        token_id,
                        Field {
                            x_min: bounds.x_min,
                            y_min: bounds.y_min,
                            x_max: bounds.x_max,
                            y_max: bounds.y_max,
                        },
                        bid_amount.get_untracked(),
                        bids_minimum_price.get_untracked(),
                    ).await;
                };
            }
        }
    });

    view! {
        <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
            <Stack orientation=StackOrientation::Vertical spacing=Size::Em(0.6)>
                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                    "Bid amount:"
                    <NumberInput min=token.minimum_price get=bid_amount set=set_bid_amount placeholder="Bid amount"/>
                </Stack>
                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                    "Minimum bid price:"
                    <NumberInput min=token.minimum_price get=bids_minimum_price set=set_bids_minimum_price placeholder="Minimum bid price"/>
                </Stack>
            </Stack>
            <Button on_click=move |_| create_bid.dispatch(token.token_id)>"Bid"</Button>
        </Stack>
    }
}
