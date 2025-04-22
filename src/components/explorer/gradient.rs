use leptos::prelude::*;
use leptos_use::{use_draggable, use_element_bounding, use_element_size};
use reactive_stores::Store;
use wasm_bindgen::JsCast;

#[derive(Store, Debug, Clone)]
pub struct Points {
    #[store(key: u32 = |checkpoint| checkpoint.position.clone())]
    checkpoints: Vec<Checkpoint>,
}

#[derive(Store, Debug, Clone, PartialEq)]
struct Checkpoint {
    position: u32,   // 0–1000000
    color: String,   // e.g. "#ff0000"
}


fn generate_linear_gradient(checkpoints: &[Checkpoint]) -> String {
    let mut sorted = checkpoints.to_vec();
    sorted.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());

    let mut stops: Vec<String> = sorted
        .iter()
        .map(|cp| format!("{} {}%", cp.color, cp.position / 10000))
        .collect();

    // Add virtual stop at 100% to close the loop with the first color
    if let (Some(first), Some(last)) = (sorted.first(), sorted.last()) {
        if last.position < 1000000 {
            stops.push(format!("{} 100%", first.color));
        }
    }

    format!("linear-gradient(to right, {})", stops.join(", "))
}

#[component]
pub fn GradientEditor() -> impl IntoView {
    let bar_ref = NodeRef::<leptos::html::Div>::new();
    let bar_width = use_element_size(bar_ref).width;
    let bar_left = use_element_bounding(bar_ref).left;

    let points = Store::new(Points {
        checkpoints: vec![
            Checkpoint { position: 0, color: "#ff0000".into() },
            // Checkpoint { position: 50.0, color: "#00ff00".into() },
            Checkpoint { position: 500000, color: "#0000ff".into() },
        ],
    });

    let active_checkpoint = RwSignal::new(points.get_untracked().checkpoints.first().cloned());

    let on_gradient_click = move |ev: web_sys::MouseEvent| {
        let rect = ev
            .target()
            .unwrap()
            .unchecked_into::<web_sys::HtmlElement>()
            .get_bounding_client_rect();
        let click_x = ev.client_x() as f64 - rect.left() as f64;
        let position = ((click_x / bar_width.get()).clamp(0.0, 1.0) * 1000000.0) as u32;

        let checkpoint = Checkpoint {
            position,
            color: "#ffffff".to_string(),
        };

        points.update(|points| points.checkpoints.push(checkpoint.clone()));

        active_checkpoint.set(Some(checkpoint));
    };


    view! {
        <div class="relative w-full">
            // Gradient bar with dynamic background
            <div
                node_ref=bar_ref
                class="h-10 rounded cursor-crosshair"
                on:click=on_gradient_click
                style=move || format!("background: {};", generate_linear_gradient(&points.get().checkpoints))
            ></div>

            // Draggable arrows
            <For
                each=move || points.checkpoints()
                key=|checkpoint| checkpoint.read().position
                children=move |checkpoint| {
                    let left_px = move || checkpoint.position().get() as f64 / 1000000.0 * bar_width.get();
                    let el = NodeRef::<leptos::html::Div>::new();
                    let draggable = use_draggable(el);
                    let current_drag_position = RwSignal::new(None::<u32>);

                    Effect::new(move |_| {
                        if draggable.is_dragging.get() && draggable.x.get() != 0.0 {
                            let new_position = (((draggable.x.get() - bar_left.get()) / bar_width.get()) * 1000000.0) as u32;
                            // checkpoint.position().set(new_position);
                            current_drag_position.set(Some(new_position));
                        } else if let Some(pos) = current_drag_position.get() {
                            log::info!("not dragging {}", pos);
                            current_drag_position.set(None);
                            checkpoint.position().set(pos);
                        }
                    });

                    log::info!("RENDER");

                    view! {
                        <div
                            node_ref=el
                            on:click=move |_| active_checkpoint.set(Some(checkpoint.get()))
                            class=move || {
                                let color_class = if Some(checkpoint.get()) == active_checkpoint.get() {
                                    "border-t-pink-500"
                                } else {
                                    "border-t-black"
                                };
                    
                                format!(
                                    "absolute top-0 w-0 h-0 border-l-8 border-r-8 border-transparent border-t-[12px] {} cursor-pointer",
                                    color_class
                                )
                            }
                            style=move || format!("left: {}px; transform: translateX(-8px) translateY(-12px);", left_px())
                        ></div>
                    }
                }
            />

            // Color Picker UI
            {move || {
                active_checkpoint.get().map(|active_checkpoint| view! {
                    <div class="mt-4 flex items-center space-x-2">
                        <label class="text-sm font-medium">"Selected Color:"</label>
                        <input
                            type="color"
                            class="w-8 h-8 rounded border shadow"
                            prop:value=active_checkpoint.color.clone()
                            on:input=move |ev| {
                                let new_color = event_target_value(&ev);
                                points.update(|points| {
                                    for checkpoint in &mut points.checkpoints {
                                        if checkpoint.position == active_checkpoint.position {
                                            checkpoint.color = new_color;
                                            break
                                        }
                                    }
                                });
                            }
                        />
                    </div>
                })
            }}
        </div>
    }
}