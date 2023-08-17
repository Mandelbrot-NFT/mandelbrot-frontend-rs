use std::sync::{Arc, Mutex};

use patternfly_yew::prelude::*;
use web3::transports::{eip_1193::Eip1193, Either, Http};
use yew::prelude::*;
use yew_router::prelude::{BrowserRouter, Routable, Switch};
use yew_ethereum_provider::UseEthereumHandle;

use crate::{
    chain::sepolia_testnet,
    evm::contracts,
    components::{
        balance::Balance,
        controller::{Controller, ControllerProps},
    }
};


#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Main,
    #[at("/nodes/:id")]
    Node { id: u128 },
    #[at("/*")]
    Default,
}


#[derive(Properties)]
pub struct BlockchainProps {
    pub mandelbrot: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl PartialEq for BlockchainProps {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[function_component]
pub fn Blockchain(props: &BlockchainProps) -> Html {
    let backdropper = use_backdrop().expect("Must be nested under a BackdropViewer component");
    let ethereum = use_context::<Option<UseEthereumHandle>>().expect("No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>");
    let (transport, address) = if let Some(ethereum) = ethereum {
        (
            Either::Left(Eip1193::new(ethereum.provider.clone())),
            if let Some(address) = ethereum.address() {
                Some(address.clone())
            } else {
                None
            }
        )
    } else {
        (Either::Right(Http::new(&sepolia_testnet().rpc_urls[0]).unwrap()), None)
    };

    let handle_error = Callback::from({
        let backdropper = backdropper.clone();
        move |error: contracts::Error| {
            let message = match error {
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
                contracts::Error::Other(message) => message,
            };

            backdropper.open(Backdrop::new(
                html! {
                    <Bullseye>
                        <Modal
                            title = {"Error"}
                            variant = { ModalVariant::Medium }
                            description = { message }
                        />
                    </Bullseye>
                }
            ));
        }
    });

    let switch = {
        let handle_error = handle_error.clone();
        let mandelbrot = props.mandelbrot.clone();
        move |route| {
            let mut props = ControllerProps {
                handle_error: handle_error.clone(),
                transport: transport.clone(),
                address,
                mandelbrot: mandelbrot.clone(),
                token_id: 1,
            };
            match route {
                Route::Node { id } => { props.token_id = id; },
                _ => {},
            }
            html! {
                <Controller ..props/>
            }
        }
    };

    html! {
        <>
            if address.is_some() {
                <PageSection>
                    <Balance {handle_error}/>
                </PageSection>
            }
            <PageSection
                variant={PageSectionVariant::Light}
            >
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </PageSection>
        </>
    }
}
