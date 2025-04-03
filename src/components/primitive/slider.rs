use leptos::*;

#[component]
pub fn Slider(
    max: impl Fn() -> f64 + 'static + Clone,
    value: RwSignal<f64>,
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
                max=move || max()
                step="0.01"
                prop:value=value
                on:input=on_input
                class=class
            />
        </div>
    }
}
