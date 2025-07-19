use leptos::prelude::*;

use crate::evm::types::Metadata;

#[component]
pub fn Info(token: Metadata) -> impl IntoView {
    view! {
        <p>{format!("NFT id: {}", token.token_id)}</p>
        <p>{format!("Owner: {}", token.owner)}</p>
        <p>{format!("Locked OM: {}", token.locked_tokens)}</p>
        <p>{format!("Minimum bid: {}", token.minimum_price)}</p>
    }
}
