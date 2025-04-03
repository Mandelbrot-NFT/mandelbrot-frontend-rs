use leptos::*;

use crate::{components::primitive::Slider, state::State};


#[component]
pub fn Visuals() -> impl IntoView {
    let state = use_context::<State>().unwrap();
    let max_iterations = create_rw_signal(40.0);
    let offset = create_rw_signal(0.0);
    let length = create_rw_signal(360.0);

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
        <div class="flex flex-col text-white max-w-xl">
    
            <div class="space-y-2">
                <div class="flex justify-between items-center">
                    <label class="text-sm font-medium text-gray-300">"Max iterations"</label>
                    <span class="text-sm font-mono text-accent2">
                        {move || format!("{:.0}", (max_iterations.get() as f64).powi(2))}
                    </span>
                </div>
                <Slider
                    max=|| 200.0
                    value=max_iterations
                    class="w-full bg-gray-300 rounded-full focus:outline-none"
                />
            </div>
    
            <div class="space-y-2">
                <div class="flex justify-between items-center">
                    <label class="text-sm font-medium text-gray-300">"Color offset"</label>
                    <span class="text-sm font-mono text-accent2">
                        {move || format!("{:.4}", offset.get())}
                    </span>
                </div>
                <Slider
                    max=|| 1.0
                    value=offset
                    class="w-full bg-gray-300 rounded-full focus:outline-none"
                />
            </div>
    
            <div class="space-y-2">
                <div class="flex justify-between items-center">
                    <label class="text-sm font-medium text-gray-300">"Palette length"</label>
                    <span class="text-sm font-mono text-accent2">
                        {move || format!("{:.4}", length.get())}
                    </span>
                </div>
                <Slider
                    max=|| 10000.0
                    value=length
                    class="w-full bg-gray-300 rounded-full focus:outline-none"
                />
            </div>
    
        </div>
    }
}
