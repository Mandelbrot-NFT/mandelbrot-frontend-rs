use std::sync::{Arc, Mutex};

use log::info;
use yew::prelude::*;
use yew_ethereum_provider::UseEthereumHandle;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;

use crate::evm::{
    contracts::ERC1155Contract,
    types::{Field, Metadata}
};


#[derive(Properties)]
pub struct MandelbrotProps {
    pub interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl PartialEq for MandelbrotProps {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}


pub struct Mandelbrot {
    node_ref: NodeRef,
    interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl Component for Mandelbrot {
    type Message = ();
    type Properties = MandelbrotProps;

    fn create(ctx: &Context<Self>) -> Self {
        let interface = ctx.props().interface.clone();
        Self {
            node_ref: NodeRef::default(),
            interface,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sample_location = &self.interface.lock().unwrap().sample_location;
        html! {
            <div>
                <canvas
                    ref={self.node_ref.clone()}
                    width={sample_location.width.to_string()}
                    height={sample_location.height.to_string()}
                />
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            log::info!("FIRST RENDER");
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
            let interface = self.interface.clone();
            mandelbrot_explorer::start(Some(canvas), Some(interface));
        }
    }
}
