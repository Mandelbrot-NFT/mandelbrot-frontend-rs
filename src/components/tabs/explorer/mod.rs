mod gradient;
mod visuals;

use std::collections::HashMap;

use leptos::prelude::*;
use mandelbrot_explorer::{Focus, ISample};
use reactive_stores::Store;
use send_wrapper::SendWrapper;

use visuals::Visuals;

use crate::{
    color::Gradient,
    context::Context,
    util::{get_session_item, set_session_item},
};
use visuals::Palette;

#[component]
pub fn Explorer() -> impl IntoView {
    let context = use_context::<SendWrapper<Context>>().unwrap();
    let selected_palette = RwSignal::new(get_session_item::<Palette>("active_palette").unwrap_or_default());
    let active_palette = RwSignal::new(Palette::default());

    let locations =
        RwSignal::new(get_session_item::<HashMap<String, (Focus, Option<Palette>)>>("locations").unwrap_or_default());
    let location_name = RwSignal::new(String::new());
    let preserve_color = RwSignal::new(true);
    let store_locations = move || set_session_item("locations", &locations.get_untracked());

    view! {
        <div class="flex flex-col">
            <Visuals palette=selected_palette on_update=move |palette| active_palette.set(palette)/>

            <details open class="border-b w-full text-gray-700 rounded-md">
                <summary class="cursor-pointer px-2 py-2 bg-gray-100 hover:bg-gray-200">
                    Saved Locations
                </summary>
                <div class="px-4 py-2 bg-gray-100 shadow-sm">
                    <div class="flex flex-row gap-2 items-center">
                        <input
                            type="text"
                            placeholder="Enter localion name"
                            prop:value=move || location_name.get()
                            on:input=move |ev| location_name.set(event_target_value(&ev)) />
                        <button
                            on:click={
                                let context = context.clone();
                                move |_| {
                                    let name = location_name.get_untracked().trim().to_string();
                                    if !name.is_empty() {
                                        let focus = context.mandelbrot.lock().unwrap().engine.borrow().focus();
                                        locations.update(|locations| {
                                            locations.insert(name, (focus, preserve_color.get().then(|| {
                                                active_palette.get()
                                            })));
                                        });
                                        store_locations();
                                        location_name.set(String::new());
                                    }
                                }
                            }
                            class="px-4 py-2 bg-green-600 hover:bg-green-500 rounded-md text-sm font-semibold transition"
                        >Save</button>
                        <input
                            type="checkbox"
                            bind:checked=preserve_color
                            class="accent-accent1 w-4 h-4"
                        />
                        <label class="text-sm text-gray-700">preserve color</label>
                    </div>
                    <For
                        each=move || locations.get()
                        key=|(name, (focus, _))| (name.clone(), focus.to_string())
                        let((name, (focus, palette)))
                    >
                        {
                            let context = context.clone();
                            if let Some(palette) = palette.clone() {
                                match palette.gradient.clone() {
                                    Gradient::Wave(_) => todo!(),
                                    Gradient::Step(gradient) => view! {
                                        <gradient::step::Bar
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
                                                        on:click=move |_| {
                                                            selected_palette.set(palette.clone());
                                                            context.mandelbrot.lock().unwrap().move_into_focus(focus.clone());
                                                        }
                                                        class="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-medium transition"
                                                    >
                                                        Zoom
                                                    </button>
                                                    <button
                                                        on:click=move |_| {
                                                            locations.update(|locations| { locations.remove(&name); });
                                                            store_locations();
                                                        }
                                                        class="px-3 py-1 bg-red-600 hover:bg-red-500 rounded-md text-white text-sm font-medium transition"
                                                    >
                                                        Delete
                                                    </button>
                                                </div>
                                            </div>
                                        </gradient::step::Bar>
                                    }.into_any(),
                                }
                            } else {
                                view! {
                                    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-4 rounded-md border border-gray-700 bg-gray-900/50">
                                        <div class="text-sm text-white">
                                            <div class="font-semibold">{name.clone()}</div>
                                        </div>

                                        <div class="flex flex-wrap gap-2">
                                            <button
                                                on:click=move |_| {
                                                    palette.clone().map(|palette| selected_palette.set(palette));
                                                    context.mandelbrot.lock().unwrap().move_into_focus(focus.clone());
                                                }
                                                class="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-medium transition"
                                            >
                                                Zoom
                                            </button>
                                            <button
                                                on:click=move |_| {
                                                    locations.update(|locations| { locations.remove(&name); });
                                                    store_locations();
                                                }
                                                class="px-3 py-1 bg-red-600 hover:bg-red-500 rounded-md text-white text-sm font-medium transition"
                                            >
                                                Delete
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    </For>
                </div>
            </details>
        </div>
    }
}
