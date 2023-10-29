use std::sync::{Arc, Mutex};

use leptonic::prelude::*;
use leptos::*;

use crate::{
    components::blockchain::Address,
    evm::{
        contracts::ERC1155Contract,
        types::{Field, Metadata},
    }
};


#[component]
pub fn Auction(
    erc1155_contract: ERC1155Contract,
    token: Signal<Option<Metadata>>,
) -> impl IntoView {
    let address = expect_context::<Address>().0;
    let mandelbrot = expect_context::<Arc<Mutex<mandelbrot_explorer::Interface>>>();
    let (max_iterations, set_max_iterations) = create_signal(40.0);
    let (offset, set_offset) = create_signal(0.0);
    let (length, set_length) = create_signal(360.0);

    create_effect({
        let mandelbrot = mandelbrot.clone();
        move |_| {
            let mut mandelbrot = mandelbrot.lock().unwrap();
            mandelbrot.coloring.max_iterations = (max_iterations.get() as f64).powi(2) as i32;
            mandelbrot.coloring.offset = offset.get() as f32;
            mandelbrot.coloring.length = length.get() as f32;
            if let Some(redraw) = &mandelbrot.redraw {
                redraw();
            }
        }
    });

    let (bid_amount, set_bid_amount) = create_signal(0.0);
    let (bids_minimum_price, set_bids_minimum_price) = create_signal(0.0);

    let create_bid = create_action({
        move |token_id| {
            let erc1155_contract = erc1155_contract.clone();
            let mandelbrot = mandelbrot.clone();
            let token_id = *token_id;
            async move {
                if let Some(address) = address.get_untracked() {
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
        "Max iterations"
        <Slider style="width: 35em" min=0.0 max=200.0
            value=max_iterations set_value=set_max_iterations
            value_display=move |v: f64| format!("{:.0}", v.powi(2))/>
        "Color offset"
        <Slider style="width: 35em" min=0.0 max=1.0
            value=offset set_value=set_offset
            value_display=move |v: f64| format!("{v:.4}")/>
        "Palette lenght"
        <Slider style="width: 35em" min=0.0 max=10000.0
            value=length set_value=set_length
            value_display=move |v: f64| format!("{v:.4}")/>
        {
            move || {
                if let Some(token) = token.get() {
                    set_bid_amount.set(token.minimum_price);
                    set_bids_minimum_price.set(token.minimum_price);
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
                } else {
                    Default::default()
                }
            }
        }
    }
}
