use std::sync::{Arc, Mutex};

use leptos::*;
use mandelbrot_explorer::ISample;


#[component]
pub fn Mandelbrot(
    cx: Scope,
    size: Signal<(f64, f64)>,
) -> impl IntoView {
    let interface = expect_context::<Arc<Mutex<mandelbrot_explorer::Interface>>>(cx);
    let canvas = create_node_ref::<html::Canvas>(cx);
    let window = web_sys::window().unwrap();
    create_effect(cx, {
        let window = window.clone();
        let interface = interface.clone();
        move |_| interface.lock().unwrap().sample.resize(
            (size().0 * window.device_pixel_ratio()) as u32,
            (size().1 * window.device_pixel_ratio()) as u32
        )
    });
    canvas.on_load(cx, |canvas| mandelbrot_explorer::start(Some((*canvas).clone()), Some(interface)));

    view! { cx,
        <canvas
            _ref=canvas
            width={let window = window.clone(); move || size().0 * window.device_pixel_ratio()}
            height={let window = window.clone(); move || size().1 * window.device_pixel_ratio()}
            style:width=move || format!("{}px", size().0.max(1.0).to_string())
            style:height=move || format!("{}px", size().1.max(1.0).to_string())
        />
    }
}
