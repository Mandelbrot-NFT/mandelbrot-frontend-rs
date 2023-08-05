use std::sync::{Arc, Mutex};

use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_router::prelude::{BrowserRouter, Routable, Switch};
use yew_ethereum_provider::{AccountLabel, ConnectButton, UseEthereumHandle};

use crate::components::{
    balance::Balance,
    controller::{Controller, ControllerProps},
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
    if let (backdropper, Some(ethereum)) = (
        use_backdrop().expect("Must be nested under a BackdropViewer component"),
        use_context::<Option<UseEthereumHandle>>().expect("No ethereum provider found. You must wrap your components in an <EthereumContextProvider/>"),
    ) {
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
                    ethereum: ethereum.clone(),
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
                <PageSection
                    r#type={PageSectionType::Default}
                    variant={PageSectionVariant::Light}
                    limit_width=true
                    sticky={[PageSectionSticky::Top]}
                    fill={PageSectionFill::Fill}
                >
                    <ConnectButton/>
                    <AccountLabel/>
                </PageSection>
                <PageSection>
                    <Balance {handle_error}/>
                </PageSection>
                <PageSection
                    variant={PageSectionVariant::Light}
                >
                    <BrowserRouter>
                        <Switch<Route> render={switch} />
                    </BrowserRouter>
                </PageSection>
            </>
        }
    } else {
        html! {}
    }

}
