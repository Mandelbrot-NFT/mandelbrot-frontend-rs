use std::sync::{Arc, Mutex};

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use web_sys::HtmlCanvasElement;

#[component]
pub fn Mandelbrot(interface: SendWrapper<Arc<Mutex<mandelbrot_explorer::Interface>>>) -> impl IntoView {
    let canvas = NodeRef::new();
    let window = web_sys::window().unwrap();
    let height = RwSignal::new(window.inner_height().unwrap().as_f64().unwrap() + 1.0);
    let device_pixel_ratio = RwSignal::new(window.device_pixel_ratio());

    let resize_callback = Arc::new({
        let window = window.clone();
        move || height.set(window.inner_height().unwrap().as_f64().unwrap() + 1.0)
    });

    Effect::new({
        let interface = interface.clone();
        move |_| {
            let pixel_ratio = window.device_pixel_ratio();
            device_pixel_ratio.set(pixel_ratio);
            interface
                .lock()
                .unwrap()
                .sample
                .borrow_mut()
                .resize((height.get() * pixel_ratio) as u32, (height.get() * pixel_ratio) as u32);
        }
    });
    canvas.on_load(|canvas: HtmlCanvasElement| {
        mandelbrot_explorer::start(Some(canvas.clone()), interface.take(), resize_callback)
    });

    view! {
        <canvas
            class="outline-none"
            node_ref=canvas
            width=move || height.get() * device_pixel_ratio.get()
            height=move || height.get() * device_pixel_ratio.get()
            style:width=move || format!("{}px", height.get().max(1.0).to_string())
            style:height=move || format!("{}px", height.get().max(1.0).to_string())
        />
    }
}
