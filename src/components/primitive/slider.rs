use leptos::prelude::*;
use leptos::reactive::traits::{Get, Set};
use leptos::tachys::html::property::IntoProperty;

#[component]
pub fn Slider(
    #[prop(into)] max: Signal<f64>,
    value: impl IntoProperty + Get<Value = f64> + Set<Value = f64> + Copy + Send + 'static,
    class: &'static str,
) -> impl IntoView {
    let on_input = move |event: web_sys::Event| {
        if let Ok(input_value) = event_target_value(&event).parse::<f64>() {
            value.set(input_value);
        }
    };

    view! {
        <div>
            <input
                type="range"
                min="0.0"
                max=move || max.get()
                step="0.01"
                prop:value=value
                on:input=on_input
                class=class
            />
        </div>
    }
}
