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
    let height = use_state(|| (window.inner_height().unwrap().as_f64().unwrap() + 1.0) * window.device_pixel_ratio());
    let resize_state = use_state(|| wasm_bindgen::closure::Closure::<dyn FnMut()>::new({
        let window = window.clone();
        let height = height.clone();
        move || height.set((window.inner_height().unwrap().as_f64().unwrap() + 1.0) * window.device_pixel_ratio())
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
                        <Tab label="How to Use">
                            <div style="overflow=scroll">
                                <h1>{ "Welcome to Mandelbrot NFT!" }</h1>
                                <p>{ "This decentralized application allows you to interact with the Mandelbrot NFT ecosystem and create, trade, and
                                explore unique visual representations of the Mandelbrot set. This guide will walk you through the main functionalities
                                and steps to get started." }</p>
                                <br/>
                                <h2>{ "Prerequisites:" }</h2>
                                <ol>
                                <li>{ "You will need a compatible web browser with a web3 wallet extension (such as MetaMask) installed." }</li>
                                <li>{ "Make sure you have some wFUEL tokens in your web3 wallet to pay for minting NFTs." }</li>
                                </ol>
                                <br/>
                                <h2>{ "Getting Started:" }</h2>

                                <h3>{ "Step 1: Accessing Mandelbrot NFT" }</h3>
                                <ol>
                                <li>{ "Open your web browser and navigate to https://mandelbrot-nft.onrender.com." }</li>
                                <li>{ "Ensure that your web3 wallet extension is active and connected to the Sepolia network." }</li>
                                </ol>
                                <br/>
                                <h3>{ "Step 2: Connect your Wallet" }</h3>
                                <ol>
                                <li>{ "On the dApp interface, click the 'Connect Wallet' button." }</li>
                                <li>{ "Follow the prompts from your web3 wallet extension to connect it to the dApp." }</li>
                                </ol>
                                <br/>
                                <h3>{ "Step 3: Buy wrapper FUEL" }</h3>
                                <ol>
                                <li>{ "On the dApp interface, click the 'Buy wFUEL' button." }</li>
                                <li>{ "You will be redirected to the Uniswap pair where you will have an opportunity to buy wFUEL." }</li>
                                </ol>
                                <br/>
                                <h3>{ "Step 4: Unwrap wrapper FUEL" }</h3>
                                <ol>
                                <li>{ "On the dApp interface, select the amount of wFUEL that you wish to unwrap and click the 'Unwrap' button." }</li>
                                <li>{ "Once the transaction succeeds your balance will be refreshed." }</li>
                                </ol>
                                <br/>
                                <h3>{ "Step 5: Explore the Mandelbrot Set" }</h3>
                                <ol>
                                <li>{ "You can pan and zoom on the Cartesian plane to explore different regions of the Mandelbrot set." }</li>
                                <li>{ "Each red or blue frame represents an NFT, and you can double click on any NFT to view its details." }</li>
                                </ol>
                                <br/>
                                <h3>{ "Step 6: Minting NFTs" }</h3>
                                <ol>
                                <li>{ "To mint an NFT within the coordinates of an existing NFT, double click on the NFT of interest." }</li>
                                <li>{ "On the NFT details page, click the 'Bid' button." }</li>
                                <li>{ "Set the amount of FUEL you are willing to spend on the minting process." }</li>
                                <li>{ "Set the minimum bid amount needed for others to mint NFTs inside of your's." }</li>
                                <li>{ "Afterwards submit your bid, it will be represented as a yellow frame." }</li>
                                <li>{ "The owner of the parent NFT will review the bids and decide which NFTs get minted." }</li>
                                </ol>
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
