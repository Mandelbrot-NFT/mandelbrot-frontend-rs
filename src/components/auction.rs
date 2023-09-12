use std::sync::{Arc, Mutex};

use leptonic::prelude::*;
use leptos::*;
use web3::types::Address;

use crate::evm::{
    contracts::ERC1155Contract,
    types::{Field, Metadata},
};


#[component]
pub fn Auction(
    cx: Scope,
    erc1155_contract: ERC1155Contract,
    address: Signal<Option<Address>>,
    token: Signal<Metadata>,
) -> impl IntoView {
    let mandelbrot = expect_context::<Arc<Mutex<mandelbrot_explorer::Interface>>>(cx);
    let (max_iterations, set_max_iterations) = create_signal(cx, 40.0);

    create_effect(cx, {
        let mandelbrot = mandelbrot.clone();
        move |_| {
            let mut mandelbrot = mandelbrot.lock().unwrap();
            mandelbrot.coloring.max_iterations = (max_iterations() as f64).powi(2) as i32;
            if let Some(redraw) = &mandelbrot.redraw {
                redraw();
            }
        }
    });

    let (bid_amount, set_bid_amount) = create_signal(cx, 0.0);
    let (bids_minimum_price, set_bids_minimum_price) = create_signal(cx, 0.0);

    let create_bid = create_action(cx, {
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            let mandelbrot = mandelbrot.clone();
            async move {
                if let Some(address) = address.get_untracked() {
                    let params = mandelbrot.lock().unwrap().to_mandlebrot_params();
                    erc1155_contract.bid(
                        address,
                        token.get_untracked().token_id,
                        Field {
                            x_min: params.x_min as f64,
                            y_min: params.y_min as f64,
                            x_max: params.x_max as f64,
                            y_max: params.y_max as f64
                        },
                        bid_amount.get_untracked(),
                        bids_minimum_price.get_untracked(),
                    ).await;
                };
            }
        }
    });

    move || {
        set_bid_amount(token.get().minimum_price);
        set_bids_minimum_price(token.get().minimum_price);
        view! { cx,
            <Slider style="width: 35em" min=0.0 max=200.0
                value=max_iterations set_value=set_max_iterations
                value_display=create_callback(cx, move |v: f64| format!("{:.0}", v.powi(2))) />
            <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                <Stack orientation=StackOrientation::Vertical spacing=Size::Em(0.6)>
                    <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                        "Bid amount:"
                        <NumberInput min=token.get().minimum_price get=bid_amount set=set_bid_amount placeholder="Bid amount"/>
                    </Stack>
                    <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                        "Minimum bid price:"
                        <NumberInput min=token.get().minimum_price get=bids_minimum_price set=set_bids_minimum_price placeholder="Minimum bid price"/>
                    </Stack>
                </Stack>
                <Button on_click=move |_| create_bid.dispatch(())>"Bid"</Button>
            </Stack>
        }
    }
}
