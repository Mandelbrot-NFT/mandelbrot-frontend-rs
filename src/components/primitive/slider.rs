use leptos::prelude::*;

#[component]
pub fn Slider(#[prop(into)] max: Signal<f64>, value: RwSignal<f64>, class: &'static str) -> impl IntoView {
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
