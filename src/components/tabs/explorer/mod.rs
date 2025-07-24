mod gradient;
mod visuals;

use std::collections::HashMap;

use leptos::prelude::*;
use mandelbrot_explorer::{Focus, ISample};
use send_wrapper::SendWrapper;

use visuals::Visuals;

use crate::{
    context::Context,
    util::{get_session_item, set_session_item},
};

#[component]
pub fn Explorer() -> impl IntoView {
    let context = use_context::<SendWrapper<Context>>().unwrap();
    let location_name = RwSignal::new(String::new());

    let locations: RwSignal<HashMap<String, Focus>> = RwSignal::new(
        get_session_item("locations")
            .unwrap_or_default()
            .split('\x1F')
            .filter(|s| !s.trim().is_empty())
            .map(|s| {
                let (name, focus) = s.split_once('\x1E').expect("Failed to parse location");
                (name.into(), focus.parse().expect("Failed to parse Focus"))
            })
            .collect::<HashMap<String, Focus>>(),
    );

    let save_locations = move || {
        set_session_item(
            "locations",
            &locations
                .get_untracked()
                .iter()
                .map(|(name, focus)| format!("{name}\x1E{focus}"))
                .collect::<Vec<_>>()
                .join("\x1F"),
        );
    };

    view! {
        <div class="flex flex-col">
            <Visuals/>

            <details open class="border-b w-full text-gray-700 rounded-md">
                <summary class="cursor-pointer px-2 py-2 bg-gray-100 hover:bg-gray-200">
                    Saved Locations
                </summary>
                <div class="px-4 py-2 bg-gray-100 shadow-sm">
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
                                    locations.update(|locations| { locations.insert(name, focus); });
                                    save_locations();
                                    location_name.set(String::new());
                                }
                            }
                        }
                        class="px-4 py-2 bg-green-600 hover:bg-green-500 rounded-md text-sm font-semibold transition"
                    >Save</button>
                    <For
                        each=move || locations.get()
                        key=|(name, focus)| (name.clone(), focus.to_string())
                        let((name, focus))
                    >
                        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-4 rounded-md border border-gray-700 bg-gray-900/50">
                            <div class="text-sm text-white">
                                <div class="font-semibold">{name.clone()}</div>
                            </div>

                            <div class="flex flex-wrap gap-2">
                                <button
                                    on:click={
                                        let context = context.clone();
                                        move |_| context.mandelbrot.lock().unwrap().move_into_focus(focus.clone())
                                    }
                                    class="px-3 py-1 bg-blue-600 hover:bg-blue-500 rounded-md text-white text-sm font-medium transition"
                                >
                                    Zoom
                                </button>
                                <button
                                    on:click=move |_| {
                                        locations.update(|locations| { locations.remove(&name); });
                                        save_locations();
                                    }
                                    class="px-3 py-1 bg-red-600 hover:bg-red-500 rounded-md text-white text-sm font-medium transition"
                                >
                                    Delete
                                </button>
                            </div>
                        </div>
                    </For>
                </div>
            </details>
        </div>
    }
}
