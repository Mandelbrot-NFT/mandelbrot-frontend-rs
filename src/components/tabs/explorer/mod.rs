mod gradient;
mod visuals;

use leptos::prelude::*;

use visuals::Visuals;

#[component]
pub fn Explorer() -> impl IntoView {
    view! {
        <div class="flex flex-col">
            <Visuals/>
        </div>
    }
}
