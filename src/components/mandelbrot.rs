use std::sync::{Arc, Mutex};

use leptos::*;


#[component]
pub fn Mandelbrot(
    interface: Arc<Mutex<mandelbrot_explorer::Interface>>,
) -> impl IntoView {
    let canvas = create_node_ref::<html::Canvas>();
    let window = web_sys::window().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap() + 1.0;
    let (get_height, set_height) = create_signal(height);

    let resize_callback = Arc::new({
        let window = window.clone();
        move || {
            let height = window.inner_height().unwrap().as_f64().unwrap() + 1.0;
            set_height.set(height);
        }
    });

    create_effect({
        let window = window.clone();
        let interface = interface.clone();
        move |_| interface.lock().unwrap().sample.borrow_mut().resize(
            (get_height.get() * window.device_pixel_ratio()) as u32,
            (get_height.get() * window.device_pixel_ratio()) as u32
        )
    });
    canvas.on_load(|canvas| mandelbrot_explorer::start(Some((*canvas).clone()), interface, resize_callback));

    view! {
        <canvas
            _ref=canvas
            width={let window = window.clone(); move || get_height.get() * window.device_pixel_ratio()}
            height={let window = window.clone(); move || get_height.get() * window.device_pixel_ratio()}
            style:width=move || format!("{}px", get_height.get().max(1.0).to_string())
            style:height=move || format!("{}px", get_height.get().max(1.0).to_string())
        />
    }
}
