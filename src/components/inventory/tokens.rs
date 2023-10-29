use std::collections::HashMap;

use leptonic::prelude::*;
use leptos::*;
use leptos_router::*;
use mandelbrot_explorer::FrameColor;

use crate::{
    evm::types::Metadata,
    state::State,
};


#[component]
pub fn Tokens(
    tokens: RwSignal<HashMap<u128, Metadata>>,
) -> impl IntoView {
    let state = use_context::<State>().unwrap();

    let burn_token = create_action({
        let erc1155_contract = state.erc1155_contract.clone();
        move |token_id: &u128| {
            let erc1155_contract = erc1155_contract.clone();
            let token_id = token_id.clone();
            async move {
                if let Some(address) = state.address.get_untracked() {
                    if let Some(_) = erc1155_contract.burn(address, token_id).await {
                        tokens.update(|tokens| {
                            tokens.remove(&token_id);
                        });
                    }
                }
            }
        }
    });

    let query = use_query_map();
    let preserve_log_level = move |uri| {
        if let Some(log_level) = query.get_untracked().get("RUST_LOG") {
            format!("{uri}?RUST_LOG={log_level}")
        } else {
            uri
        }
    };

    let zoom_token = move |token_id| {
        if let Some(token) = tokens.get().get(&token_id) {
            use_navigate()(&preserve_log_level(format!("/tokens/{}", token_id)), Default::default());
            let frame = token.to_frame(FrameColor::Blue);
            state.mandelbrot.lock().unwrap().move_into_bounds(&frame.bounds)
        }
    };

    let edited_token = create_rw_signal(None);
    let (bids_minimum_price, set_bids_minimum_price) = create_signal(0.0);
    let edit_token = move |token: Metadata| {
        set_bids_minimum_price.set(token.minimum_price);
        edited_token.set(Some(token))
    };
    let edit_token_submit = create_action({
        let erc1155_contract = state.erc1155_contract.clone();
        move |_| {
            let erc1155_contract = erc1155_contract.clone();
            async move {
                if let (Some(address), Some(token)) = (state.address.get_untracked(), edited_token.get_untracked()) {
                    erc1155_contract.set_minimum_bid(address, token.token_id, bids_minimum_price.get_untracked()).await;
                }
                edited_token.set(None);
            }
        }
    });

    view! {
        <Show when=move || {tokens.get().len() > 0} fallback=|| {}>
            {
                let zoom_token = zoom_token.clone();
                view! {
                    <Box id="content">
                        <For
                            each=move || tokens.get().into_values()
                            key=|token| token.token_id
                            children={
                                move |token| view! {
                                    <p>
                                        <Button on_click={let zoom_token = zoom_token.clone(); move |_| zoom_token(token.token_id)}>"Zoom"</Button>
                                        {format!("Token Id: {} Locked FUEL: {}", token.token_id, token.locked_fuel.to_string())}
                                        <Button on_click={let token = token.clone(); move |_| edit_token(token.clone())}>"Edit"</Button>
                                        <Button on_click=move |_| burn_token.dispatch(token.token_id)>"Burn"</Button>
                                    </p>
                                }
                            }
                        />
                    </Box>
                }
            }
        </Show>

        <Modal show_when=MaybeSignal::derive(move || edited_token.get().is_some())>
            <ModalHeader><ModalTitle>
                "Token Id "{move || edited_token.get().map_or("".into(), |token| token.token_id.to_string())}
            </ModalTitle></ModalHeader>
            <ModalBody>
                {
                    move || {
                        if let Some(token) = edited_token.get() {
                            view! {
                                <Stack orientation=StackOrientation::Horizontal spacing=Size::Em(0.6)>
                                    "Minimum bid price:"
                                    <NumberInput min=token.minimum_price get=bids_minimum_price set=set_bids_minimum_price/>
                                </Stack>
                            }
                        } else {
                            view! {}.into_view()
                        }
                    }
                }
            </ModalBody>
            <ModalFooter>
                <ButtonWrapper>
                    <Button on_click=move |_| edit_token_submit.dispatch(()) color=ButtonColor::Primary>"Save"</Button>
                    <Button on_click=move |_| edited_token.set(None) color=ButtonColor::Secondary>"Cancel"</Button>
                </ButtonWrapper>
            </ModalFooter>
        </Modal>
    }
}
