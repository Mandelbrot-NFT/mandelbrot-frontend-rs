mod gradient;
mod visuals;

use std::{collections::HashMap, time::Duration};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use mandelbrot_explorer::{Focus, ISample};
use reactive_stores::Store;
use send_wrapper::SendWrapper;

use url::Url;
use visuals::Visuals;

use crate::{
    color::Gradient,
    context::Context,
    util::{load_item, store_item},
};
use visuals::Palette;

#[component]
pub fn Explorer() -> impl IntoView {
    let query_map = use_query_map();
    let context = use_context::<SendWrapper<Context>>().unwrap();
    let selected_palette = RwSignal::new({
        if let Some(palette) = query_map.get_untracked().get("palette") {
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&palette).unwrap()).unwrap()
        } else {
            load_item::<Palette>("active_palette").unwrap_or_default()
        }
    });
    let active_palette = RwSignal::new(Palette::default());
    let show_toast = RwSignal::new(false);

    let locations =
        RwSignal::new(load_item::<HashMap<String, (Focus, Option<Palette>)>>("locations").unwrap_or_default());
    let location_name = RwSignal::new(String::new());
    let preserve_color = RwSignal::new(true);
    let store_locations = move || store_item("locations", &locations.get_untracked());

    view! {
        <div class="flex flex-col">
            <Visuals palette=selected_palette on_update=move |palette| active_palette.set(palette)/>

            <details class="border-b w-full text-gray-700 rounded-md">
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
                        {
                            let context = context.clone();
                            view! {
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
                                                        length=palette.length
                                                        offset=palette.offset
                                                    >
                                                        <Location
                                                            name=name.clone()
                                                            on_select=move || {
                                                                selected_palette.set(palette.clone());
                                                                context.mandelbrot.lock().unwrap().move_into_focus(focus.clone());
                                                            }
                                                            on_delete=move || {
                                                                locations.update(|locations| { locations.remove(&name); });
                                                                store_locations();
                                                            }
                                                        />
                                                    </gradient::step::Bar>
                                                }.into_any(),
                                            }
                                        } else {
                                            view! {
                                                <div class="flex flex-row items-center h-12 rounded-md border border-gray-700 bg-gray-900/50">
                                                    <Location
                                                        name=name.clone()
                                                        on_select=move || {
                                                            palette.clone().map(|palette| selected_palette.set(palette));
                                                            context.mandelbrot.lock().unwrap().move_into_focus(focus.clone());
                                                        }
                                                        on_delete=move || {
                                                            locations.update(|locations| { locations.remove(&name); });
                                                            store_locations();
                                                        }
                                                    />
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                </For>
                            }
                        }
                </div>
            </details>

            <div class="flex flex-row my-1">
                <button
                    on:click={
                        let context = context.clone();
                        move |_| {
                            if let Ok(href) = web_sys::window().unwrap().location().href() {
                                let mut url = Url::parse(&href).ok().unwrap();
                                url.set_query(Some(&format!(
                                    "focus={}&palette={}",
                                    context.mandelbrot.lock().unwrap().engine.borrow().focus(),
                                    URL_SAFE_NO_PAD.encode(serde_json::to_string(&active_palette.get_untracked()).unwrap()),
                                )));
                                let clipboard = web_sys::window()
                                    .unwrap()
                                    .navigator()
                                    .clipboard();

                                let _ = clipboard.write_text(&url.to_string());

                                show_toast.set(true);
                                let show_toast = show_toast.clone();
                                set_timeout(move || show_toast.set(false), Duration::from_millis(1500));
                            }
                        }
                    }
                    class="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-medium transition"
                >
                    Share
                </button>

                {move || show_toast.get().then(|| view! {
                    <div class="mx-10 px-3 text-gray-500">
                        URL copied to clipboard
                    </div>
                })}
            </div>
        </div>
    }
}

#[component]
pub fn Location(name: String, on_select: impl Fn() + 'static, on_delete: impl Fn() + 'static) -> impl IntoView {
    view! {
        <div class="w-full flex flex-row items-center justify-between gap-4 p-4">
            <div class="text-sm text-white">
                <div class="font-semibold">{name}</div>
            </div>
            <div class="flex flex-wrap gap-2">
                <button
                    on:click=move |_| on_select()
                    class="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-medium transition"
                >
                    Zoom
                </button>
                <button
                    on:click=move |_| on_delete()
                    class="px-3 py-1 bg-red-600 hover:bg-red-500 rounded-md text-white text-sm font-medium transition"
                >
                    Delete
                </button>
            </div>
        </div>
    }
}
