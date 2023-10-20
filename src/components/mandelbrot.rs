use std::sync::{Arc, Mutex};

use leptos::*;


#[component]
pub fn Mandelbrot(size: Signal<(f64, f64)>) -> impl IntoView {
    let interface = expect_context::<Arc<Mutex<mandelbrot_explorer::Interface>>>();
    let canvas = create_node_ref::<html::Canvas>();
    let window = web_sys::window().unwrap();
    create_effect({
        let window = window.clone();
        let interface = interface.clone();
        move |_| interface.lock().unwrap().sample.borrow_mut().resize(
            (size.get().0 * window.device_pixel_ratio()) as u32,
            (size.get().1 * window.device_pixel_ratio()) as u32
        )
    });
    canvas.on_load(|canvas| mandelbrot_explorer::start(Some((*canvas).clone()), interface));

    view! {
        <canvas
            _ref=canvas
            width={let window = window.clone(); move || size.get().0 * window.device_pixel_ratio()}
            height={let window = window.clone(); move || size.get().1 * window.device_pixel_ratio()}
            style:width=move || format!("{}px", size.get().0.max(1.0).to_string())
            style:height=move || format!("{}px", size.get().1.max(1.0).to_string())
        />
    }
}
