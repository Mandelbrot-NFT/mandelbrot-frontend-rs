mod chain;
mod components;
mod evm;

use std::sync::{Arc, Mutex};

use patternfly_yew::prelude::*;
use yew::prelude::*;
use yew_ethereum_provider::{AccountLabel, ConnectButton, EthereumContextProvider};
use wasm_bindgen::JsCast;

use components::{
    blockchain::Blockchain,
    mandelbrot::Mandelbrot,
};


#[function_component]
fn App() -> Html {
    let window = web_sys::window().unwrap();
    let height = use_state(|| (window.inner_height().unwrap().as_f64().unwrap() + 1.0) * 2.0);
    let resize_state = use_state(|| wasm_bindgen::closure::Closure::<dyn FnMut()>::new({
        let window = window.clone();
        let height = height.clone();
        move || height.set((window.inner_height().unwrap().as_f64().unwrap() + 1.0) * 2.0)
    }));
    if window.onresize().is_none() {
        window.set_onresize(Some(resize_state.as_ref().unchecked_ref()));
    }

    let interface = Arc::new(Mutex::new(mandelbrot_explorer::Interface {
        sample_location: mandelbrot_explorer::SampleLocation::new(*height, *height),
        frames: Vec::new(),
        frame_selected_callback: None,
        redraw: None,
    }));

    html! {
        <BackdropViewer>
            <Split>
                <SplitItem fill={true}>
                    <Mandelbrot size={(*height, *height)} interface={interface.clone()}/>
                </SplitItem>
                <SplitItem>
                    <Tabs>
                        <Tab label="DApp">
                            <EthereumContextProvider>
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
                                <Blockchain mandelbrot={interface}/>
                            </EthereumContextProvider>
                        </Tab>
                        <Tab label="Description">
                            <iframe
                                width="100%"
                                src="https://www.youtube.com/embed/OlD2rcm971U"
                                title="YouTube video player"
                                frameborder="0"
                                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                                allowfullscreen=true
                            />
                            <div style="overflow=scroll">

                                <p>{ "The Mandelbrot set is a mathematical concept that can be visualized on a Cartesian plane with coordinates ranging from -2 to 2 on both the x and y axes." }</p>
                                <br/>
                                <p>{ "In our system, each NFT (Non-Fungible Token) represents a specific region within these coordinates, defined by a rectangular shape. We have developed custom software that generates a graphical representation for each NFT." }</p>
                                <br/>
                                <p>{ "The original NFT, called the Origin NFT, is owned by the project DAO (Decentralized Autonomous Organisation) and covers the entire coordinate range of -2 to 2 on both axes." }</p>
                                <br/>
                                <p>{ "Every NFT has the ability to create a fixed number (20 in this case) of child NFTs within its own boundaries. This hierarchical structure allows us to trace back each NFT to the Origin NFT." }</p>
                                <br/>
                                <p>{ "Creating a new NFT requires the use of a cryptocurrency token called FUEL. The number of NFTs that can be minted within a parent NFT is limited." }</p>
                                <br/>
                                <p>{ "To determine which NFTs get minted, users submit bids, and the owner of the parent NFT selects the winning bids." }</p>
                                <br/>
                                <p>{ "When an NFT is successfully minted, the FUEL used for minting is distributed among all the parent NFTs, and a portion of it is locked within a newly minted NFT." }</p>
                                <br/>
                                <p>{ "The owner of a parent NFT can set a minimum amount of FUEL that must be used in a mint bid. All child NFTs created within that parent NFT must adhere to this minimum requirement." }</p>
                                <br/>
                                <p>{ "Owner of an NFT can burn it, given that it doesn't have any child NFTs in it. When an NFT is burned, FUEL that was locked inside of it is transferred to its owner." }</p>
                            </div>
                        </Tab>
                    </Tabs>
                </SplitItem>
            </Split>
        </BackdropViewer>
    }
}


/// Parse the query string as returned by `web_sys::window()?.location().search()?` and get a
/// specific key out of it.
pub fn parse_url_query_string<'a>(query: &'a str, search_key: &str) -> Option<&'a str> {
    let query_string = query.strip_prefix('?')?;

    for pair in query_string.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next()?;
        let value = pair.next()?;

        if key == search_key {
            return Some(value);
        }
    }

    None
}


fn main() {
    let query_string = web_sys::window().unwrap().location().search().unwrap();
    let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
        .and_then(|x| x.parse().ok())
        .unwrap_or(log::Level::Error);
    console_log::init_with_level(level).expect("could not initialize logger");
    yew::Renderer::<App>::new().render();
}
