mod bids;
mod tokens;

use leptonic::prelude::*;
use leptos::*;

use crate::state::State;
use bids::Bids;
use tokens::Tokens;


#[component]
pub fn Inventory() -> impl IntoView {
    let state = use_context::<State>().unwrap();

    let refresh = create_action({
        let state = state.clone();
        move |_| {
            let state = state.clone();
            async move {
                state.reload_inventory().await;
            }
        }
    });

    view! {
        <p>"Tokens:"</p>
        <Tokens
            tokens=state.inventory.tokens
        />
        <p>"Bids:"</p>
        <Bids
            bids=state.inventory.bids
        />
        <Button on_click=move |_| refresh.dispatch(())>"Refresh"</Button>
    }
}
