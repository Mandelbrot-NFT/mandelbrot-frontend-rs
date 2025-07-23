use leptos::prelude::*;

use crate::evm::contracts::{self, Error};

#[component]
pub fn ErrorHandler(error: RwSignal<Option<Error>>) -> impl IntoView {
    let error_message = Memo::new(move |_| {
        error.with(|error| {
            error.as_ref().map(|error| match error {
                contracts::Error::TokenNotFound => "Unable to find an NFT with this Id".into(),
                contracts::Error::NoRightsToBurn => "You don't have the necessary rights to burn this NFT".into(),
                contracts::Error::TokenNotEmpty => {
                    "It is not allowed to burn an NFT if it has minted NFTs inside".into()
                }
                contracts::Error::BidNotFound => "Unable to find a bid with this Id".into(),
                contracts::Error::BidTooLow => "Your bid is too low".into(),
                contracts::Error::MinimumBidTooLow => "Minimum bid for the NFT that you wish to mint is too low".into(),
                contracts::Error::TooManyChildTokens => "This NFT cannot contain any more NFTs".into(),
                contracts::Error::NoRightsToApproveBid => {
                    "You don't have the necessary rights to approve these bids".into()
                }
                contracts::Error::NoRightsToDeleteBid => {
                    "You don't have the necessary rights to delete this bid".into()
                }
                contracts::Error::FieldOutside => {
                    "NFT that you are trying to mint has to be within the bounds of parent NFT".into()
                }
                contracts::Error::FieldsOverlap => "NFT that you are trying to mint overlaps with another NFT".into(),
                contracts::Error::FieldTooLarge => "NFT that you are trying to mint is too large".into(),
                contracts::Error::Other(message) => message.clone(),
            })
        })
    });

    view! {
        <Show when=move || error_message.get().is_some()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-60">
                <div class="bg-gray-900 text-white rounded-lg shadow-lg p-6 w-full max-w-md space-y-6">

                    <div class="text-xl font-semibold border-b border-gray-700 pb-2">
                        "Error"
                    </div>

                    <div class="text-sm text-gray-300">
                        {move || error_message.get().unwrap_or("".into())}
                    </div>

                    <div class="flex justify-end pt-4 border-t border-gray-700">
                        <button
                            on:click=move |_| error.set(None)
                            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-md text-sm font-medium transition"
                        >
                            "Ok"
                        </button>
                    </div>

                </div>
            </div>
        </Show>
    }
}
