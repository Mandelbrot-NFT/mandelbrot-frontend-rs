use leptos::prelude::*;
use leptos_use::use_element_size;
use wasm_bindgen::JsCast;

#[derive(Clone)]
struct Wave {
    bias: f32,
    amplitude: f32,
    frequency: f32,
    phase: f32,
}

impl Wave {
    fn new(bias: f32, amplitude: f32, frequency: f32, phase: f32) -> Self {
        Self {
            bias,
            amplitude,
            frequency,
            phase,
        }
    }
}

fn gradient_wave(t: f32, dx: f32, dy: f32, waves: [Wave; 3]) -> [u8; 3] {
    let t = (1.0 - t * 2.0).abs();
    itertools::izip!(
        waves,
        [dx.sin(), (dy - dx).sin(), (3.14 * dx).cos()],
        [f32::cos, f32::sin, f32::cos]
    )
    .map(|(wave, variation, trig)| {
        let value = wave.bias + wave.amplitude * trig(t * wave.frequency + wave.phase + variation);
        (value.clamp(0.0, 1.0) * 255.0) as u8
    })
    .collect::<Vec<_>>()
    .try_into()
    .unwrap()
}

#[component]
pub fn Wave() -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let canvas_size = use_element_size(canvas_ref);

    Effect::new(move || {
        if let Some(canvas) = canvas_ref.get() {
            let width = canvas_size.width.get() as u32;
            let height = canvas_size.height.get() as u32;

            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            canvas.set_width(width);
            canvas.set_height(height);

            let waves = [
                Wave::new(1.0, 0.5, 6.28, 0.9),
                Wave::new(1.0, 0.5, 5.88, -3.14),
                Wave::new(1.0, 0.5, 3.14, -3.64),
            ];

            for x in 0..width {
                let t: f32 = x as f32 / width as f32;
                let [r, g, b] = gradient_wave(t, 0.0, 0.0, waves.clone());
                ctx.set_fill_style_str(&format!("rgb({},{},{})", r, g, b));
                ctx.fill_rect(x as f64, 0.0, 1.0, height as f64);
            }
        }
    });

    view! {
        <canvas node_ref=canvas_ref class="h-10 w-full rounded"/>
    }
}
