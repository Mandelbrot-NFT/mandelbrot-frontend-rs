use std::sync::{Arc, Mutex};

use patternfly_yew::prelude::*;
use web3::transports::{eip_1193::Eip1193, Either, Http};
use yew::prelude::*;
use yew_router::prelude::{BrowserRouter, Routable, Switch};
use yew_ethereum_provider::UseEthereumHandle;

use crate::{
    chain::sepolia_testnet,
    components::{
        balance::Balance,
        controller::{Controller, ControllerProps},
    }
};


#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Main,
    #[at("/node/:id")]
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
        move |error: eyre::Report| {
            backdropper.open(Backdrop::new(
                html! {
                    <Bullseye>
                        <Modal
                            title = {"Error"}
                            variant = { ModalVariant::Medium }
                            description = { error.root_cause().to_string() }
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
