use leptos::*;

use crate::evm::types::Metadata;


#[component]
pub fn Info(
    token: Metadata,
) -> impl IntoView {

    view! {
        <p>{format!("NFT id: {}", token.token_id)}</p>
        <p>{format!("Owner: {}", token.owner)}</p>
        <p>{format!("Locked FUEL: {}", token.locked_fuel)}</p>
        <p>{format!("Minimum bid: {}", token.minimum_price)}</p>
    }
}
