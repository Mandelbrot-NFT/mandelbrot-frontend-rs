use leptonic::prelude::*;
use leptos::*;
use leptos_ethereum_provider::EthereumInterface;
use web3::transports::{eip_1193::Eip1193, Either, Http};

use crate::{
    chain::sepolia_testnet,
    evm::contracts,
};


#[derive(Clone, Debug)]
pub struct Web3(pub web3::Web3<Either<Eip1193, Http>>);


#[derive(Clone, Debug)]
pub struct Address(pub Signal<Option<web3::types::Address>>);


#[component]
pub fn Blockchain(children: Children) -> impl IntoView {
    let ethereum = expect_context::<Option<EthereumInterface>>();
    let transport = if let Some(ethereum) = &ethereum {
        Either::Left(Eip1193::new(ethereum.provider.clone()))
    } else {
        Either::Right(Http::new(&sepolia_testnet().rpc_urls[0]).unwrap())
    };
    let web3 = web3::Web3::new(transport);
    provide_context(Web3(web3));
    let address = Signal::derive(move || {
        if let Some(ethereum) = &ethereum {
            if let Some(address) = ethereum.address().get() {
                Some(address.clone())
            } else {
                None
            }
        } else {
            None
        }
    });
    provide_context(Address(address));

    let (error, set_error) = create_signal(None);
    let error_message = create_memo(move |_| {
        error.with(|error| {
            if let Some(error) = error {
                Some(match error {
                    contracts::Error::TokenNotFound => "Unable to find an NFT with this Id".into(),
                    contracts::Error::NoRightsToBurn => "You don't have the necessary rights to burn this NFT".into(),
                    contracts::Error::TokenNotEmpty => "It is not allowed to burn an NFT if it has minted NFTs inside".into(),
                    contracts::Error::BidNotFound => "Unable to find a bid with this Id".into(),
                    contracts::Error::BidTooLow => "Your bid is too low".into(),
                    contracts::Error::MinimumBidTooLow => "Minimum bid for the NFT that you wish to mint is too low".into(),
                    contracts::Error::TooManyChildTokens => "This NFT cannot contain any more NFTs".into(),
                    contracts::Error::NoRightsToApproveBid => "You don't have the necessary rights to approve these bids".into(),
                    contracts::Error::NoRightsToDeleteBid => "You don't have the necessary rights to delete this bid".into(),
                    contracts::Error::FieldOutside => "NFT that you are trying to mint has to be within the bounds of parent NFT".into(),
                    contracts::Error::FieldsOverlap => "NFT that you are trying to mint overlaps with another NFT".into(),
                    contracts::Error::FieldTooLarge => "NFT that you are trying to mint is too large".into(),
                    contracts::Error::Other(message) => message.clone(),
                })
            } else {
                None
            }
        })
    });
    provide_context(set_error);

    view! {
        { children() }
        <Modal show_when=MaybeSignal::derive(move || error_message.get().is_some())>
            <ModalHeader><ModalTitle>"Error"</ModalTitle></ModalHeader>
            <ModalBody>{move || error_message.get().unwrap_or("".into())}</ModalBody>
            <ModalFooter>
                <ButtonWrapper>
                    <Button on_click=move |_| set_error.set(None) color=ButtonColor::Secondary>"Ok"</Button>
                </ButtonWrapper>
            </ModalFooter>
        </Modal>
    }
}
