use std::sync::{Arc, Mutex};

use yew::prelude::*;
use web_sys::HtmlCanvasElement;


#[derive(Properties)]
pub struct MandelbrotProps {
    pub size: (f64, f64),
    pub interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
}

impl PartialEq for MandelbrotProps {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}


pub struct Mandelbrot {
    node_ref: NodeRef,
}

impl Component for Mandelbrot {
    type Message = ();
    type Properties = MandelbrotProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self { node_ref: NodeRef::default() }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let size = ctx.props().size.clone();
        let interface = ctx.props().interface.clone();
        interface.lock().unwrap().sample_location.resize(size.0 as f64, size.1 as f64);
        let style = format!(
            "width: {}px; height: {}px;",
            (size.0 / 2.0).max(1.0).to_string(),
            (size.1 / 2.0).max(1.0).to_string()
        );
        html! {
            <canvas
                ref={self.node_ref.clone()}
                width={size.0.to_string()}
                height={size.1.to_string()}
                style={style}
            />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            log::info!("FIRST RENDER");
            let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
            let interface = ctx.props().interface.clone();
            mandelbrot_explorer::start(Some(canvas), Some(interface));
        }
    }
}
