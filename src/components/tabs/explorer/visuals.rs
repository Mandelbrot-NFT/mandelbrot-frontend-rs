use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    str::FromStr,
};

use eyre::{Report, Result};
use leptos::prelude::*;
use reactive_stores::Store;
use send_wrapper::SendWrapper;

use crate::{
    color::{Gradient, StepGradient},
    components::primitive::Slider,
    context::Context,
    util::{get_session_item, set_session_item},
};

use super::gradient::{step, wave};

#[derive(Clone, Store)]
pub struct Palette {
    gradient: Gradient,
    max_iterations: f64,
    offset: f64,
    length: f64,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            gradient: StepGradient::default().into(),
            max_iterations: 40.0,
            offset: 0.0,
            length: 100.0,
        }
    }
}

impl Display for Palette {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{:?}/{:?}/{:?}",
            self.gradient, self.max_iterations, self.offset, self.length
        )
    }
}

impl FromStr for Palette {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self> {
        let parts = s.splitn(4, '/').collect::<Vec<_>>();
        Ok(Self {
            gradient: parts[0].parse()?,
            max_iterations: parts[1].parse()?,
            offset: parts[2].parse()?,
            length: parts[3].parse()?,
        })
    }
}

#[component]
pub fn Visuals() -> impl IntoView {
    let context = use_context::<SendWrapper<Context>>().unwrap();
    let selected_palette = RwSignal::new(Palette::default());
    let active_palette = Store::new(Palette::default());
    let palette_name = RwSignal::new(String::new());

    let palettes: RwSignal<HashMap<String, Palette>> = RwSignal::new(
        get_session_item("palettes")
            .unwrap_or_default()
            .split('\x1F')
            .filter(|s| !s.trim().is_empty())
            .map(|s| {
                let (name, palette) = s.split_once('\x1E').expect("Failed to parse palette");
                (name.into(), palette.parse().expect("Failed to parse Palette"))
            })
            .collect::<HashMap<String, Palette>>(),
    );

    Effect::new(move || active_palette.set(selected_palette.get()));
    Effect::new({
        let mandelbrot = context.mandelbrot.clone();
        move || {
            let mut mandelbrot = mandelbrot.lock().unwrap();
            mandelbrot.palette.max_iterations = (active_palette.max_iterations().get() as f64).powi(2) as i32;
            mandelbrot.palette.offset = active_palette.offset().get() as f32;
            mandelbrot.palette.length = active_palette.length().get() as f32;
            if let Some(redraw) = &mandelbrot.redraw {
                redraw();
            }
        }
    });

    let set_gradient = move |value| {
        let gradient = Gradient::from(value);
        {
            let mut mandelbrot = context.mandelbrot.lock().unwrap();
            mandelbrot.palette.gradient = gradient.clone().into();
            if let Some(redraw) = &mandelbrot.redraw {
                redraw();
            }
        }

        if active_palette.gradient().get_untracked() != gradient {
            active_palette.gradient().set(gradient);
        }
    };

    let save_palettes = move || {
        set_session_item(
            "palettes",
            &palettes
                .get_untracked()
                .iter()
                .map(|(name, palette)| format!("{name}\x1E{palette}"))
                .collect::<Vec<_>>()
                .join("\x1F"),
        );
    };

    view! {
        <div class="flex flex-col">
            {
                move || {
                    match selected_palette.get().gradient {
                        Gradient::Wave(_) => view! { <wave::Wave/> }.into_any(),
                        Gradient::Step(gradient) => view! { <step::Editor gradient on_update=set_gradient.clone()/> }.into_any(),
                    }
                }
            }

            <div class="space-y-2">
                <div class="flex justify-between items-center">
                    <label class="text-sm font-medium text-gray-300">"Max iterations"</label>
                    <span class="text-sm font-mono text-accent2">
                        {move || format!("{:.0}", active_palette.max_iterations().get().powi(2))}
                    </span>
                </div>
                <Slider
                    max=200.0
                    value=active_palette.max_iterations()
                    class="w-full bg-gray-300 rounded-full focus:outline-none"
                />
            </div>

            <div class="space-y-2">
                <div class="flex justify-between items-center">
                    <label class="text-sm font-medium text-gray-300">"Color offset"</label>
                    <span class="text-sm font-mono text-accent2">
                        {move || format!("{:.4}", active_palette.offset().get())}
                    </span>
                </div>
                <Slider
                    max=1.0
                    value=active_palette.offset()
                    class="w-full bg-gray-300 rounded-full focus:outline-none"
                />
            </div>

            <div class="space-y-2">
                <div class="flex justify-between items-center">
                    <label class="text-sm font-medium text-gray-300">"Palette length"</label>
                    <span class="text-sm font-mono text-accent2">
                        {move || format!("{:.4}", active_palette.length().get())}
                    </span>
                </div>
                <Slider
                    max=500.0
                    value=active_palette.length()
                    class="w-full bg-gray-300 rounded-full focus:outline-none"
                />
            </div>


            <details open class="border-b w-full text-gray-700 rounded-md">
                <summary class="cursor-pointer px-2 py-2 bg-gray-100 hover:bg-gray-200">
                    Saved Palettes
                </summary>
                <div class="px-4 py-2 bg-gray-100 shadow-sm">
                    <input
                        type="text"
                        placeholder="Enter palette name"
                        prop:value=move || palette_name.get()
                        on:input=move |ev| palette_name.set(event_target_value(&ev)) />
                    <button
                        on:click={
                            move |_| {
                                let name = palette_name.get_untracked().trim().to_string();
                                if !name.is_empty() {
                                    palettes.update(|palettes| { palettes.insert(name, active_palette.get()); });
                                    save_palettes();
                                    palette_name.set(String::new());
                                }
                            }
                        }
                        class="px-4 py-2 bg-green-600 hover:bg-green-500 rounded-md text-sm font-semibold transition"
                    >Save</button>
                    <For
                        each=move || palettes.get()
                        key=|(name, palette)| (name.clone(), palette.to_string())
                        let((name, palette))
                    >
                        {
                            match palette.gradient.clone() {
                                Gradient::Wave(_) => todo!(),
                                Gradient::Step(gradient) => view! {
                                    <step::Bar
                                        position=RwSignal::new(0.0).write_only()
                                        width=RwSignal::new(0.0).write_only()
                                        points=Store::new(gradient.into())
                                        on_click=|_| {}
                                    >
                                        <div class="w-full flex flex-row items-center justify-between gap-4 p-4">
                                            <div class="text-sm text-white">
                                                <div class="font-semibold">{name.clone()}</div>
                                            </div>
                                            <div class="flex flex-wrap gap-2">
                                                <button
                                                    on:click=move |_| selected_palette.set(palette.clone())
                                                    class="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-medium transition"
                                                >
                                                    Load
                                                </button>
                                                <button
                                                    on:click=move |_| {
                                                        palettes.update(|palettes| { palettes.remove(&name); });
                                                        save_palettes();
                                                    }
                                                    class="px-3 py-1 bg-red-600 hover:bg-red-500 rounded-md text-white text-sm font-medium transition"
                                                >
                                                    Delete
                                                </button>
                                            </div>
                                        </div>
                                    </step::Bar>
                                }.into_any(),
                            }
                        }
                    </For>
                </div>
            </details>
        </div>
    }
}
