use leptonic::prelude::*;
use leptos::*;

use crate::state::State;


#[component]
pub fn Visuals() -> impl IntoView {
    let state = use_context::<State>().unwrap();
    let (max_iterations, set_max_iterations) = create_signal(40.0);
    let (offset, set_offset) = create_signal(0.0);
    let (length, set_length) = create_signal(360.0);

    create_effect({
        let mandelbrot = state.mandelbrot.clone();
        move |_| {
            let mut mandelbrot = mandelbrot.lock().unwrap();
            mandelbrot.coloring.max_iterations = (max_iterations.get() as f64).powi(2) as i32;
            mandelbrot.coloring.offset = offset.get() as f32;
            mandelbrot.coloring.length = length.get() as f32;
            if let Some(redraw) = &mandelbrot.redraw {
                redraw();
            }
        }
    });

    view! {
        "Max iterations"
        <Slider style="width: 35em" min=0.0 max=200.0
            value=max_iterations set_value=set_max_iterations
            value_display=move |v: f64| format!("{:.0}", v.powi(2))/>
        "Color offset"
        <Slider style="width: 35em" min=0.0 max=1.0
            value=offset set_value=set_offset
            value_display=move |v: f64| format!("{v:.4}")/>
        "Palette lenght"
        <Slider style="width: 35em" min=0.0 max=10000.0
            value=length set_value=set_length
            value_display=move |v: f64| format!("{v:.4}")/>
    }
}
