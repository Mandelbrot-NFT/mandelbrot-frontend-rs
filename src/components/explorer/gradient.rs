use leptos::prelude::*;
use leptos_use::{use_draggable, use_element_bounding, use_element_size};
use reactive_stores::{AtKeyed, Store};
use wasm_bindgen::JsCast;

#[derive(Store, Debug, Clone)]
pub struct Points {
    #[store(key: u8 = |checkpoint| checkpoint.id)]
    checkpoints: Vec<Checkpoint>,
}

#[derive(Store, Debug, Clone, PartialEq)]
struct Checkpoint {
    id: u8,
    position: u32, // 0–1000000
    color: String, // e.g. "#ff0000"
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
fn DraggableArrow(
    checkpoint: AtKeyed<Store<Points>, Points, u8, Vec<Checkpoint>>,
    active_checkpoint_id: RwSignal<Option<u8>>,
    bar_width: Signal<f64>,
    bar_left: Signal<f64>,
) -> impl IntoView {
    let el = NodeRef::<leptos::html::Div>::new();
    let draggable = use_draggable(el);

    Effect::new(move |_| {
        if draggable.is_dragging.get() {
            let x = draggable.x.get();
            if x > 0.0 {
                let new_position = (((x + 8.0 - bar_left.get()) / bar_width.get()).min(1.0) * 1_000_000.0) as u32;
                checkpoint.position().set(new_position);
            }
        }
    });

    let left_px = move || checkpoint.position().get() as f64 / 1_000_000.0 * bar_width.get();

    view! {
        <div
            node_ref=el
            on:click=move |_| active_checkpoint_id.set(Some(checkpoint.read().id))
            class=move || {
                let color_class = if Some(checkpoint.read().id) == active_checkpoint_id.get() {
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

#[component]
pub fn GradientEditor() -> impl IntoView {
    let bar_ref = NodeRef::<leptos::html::Div>::new();
    let bar_width = use_element_size(bar_ref).width;
    let bar_left = use_element_bounding(bar_ref).left;

    let points = Store::new(Points {
        checkpoints: vec![
            Checkpoint {
                id: 0,
                position: 0,
                color: "#ff0000".into(),
            },
            Checkpoint {
                id: 1,
                position: 333000,
                color: "#00ff00".into(),
            },
            Checkpoint {
                id: 2,
                position: 666000,
                color: "#0000ff".into(),
            },
        ],
    });

    let mut next_checkpoint_id = 3;

    let active_checkpoint_id = RwSignal::new(Some(0));

    let on_gradient_click = move |ev: web_sys::MouseEvent| {
        let rect = ev
            .target()
            .unwrap()
            .unchecked_into::<web_sys::HtmlElement>()
            .get_bounding_client_rect();
        let click_x = ev.client_x() as f64 - rect.left() as f64;
        let position = ((click_x / bar_width.get()).clamp(0.0, 1.0) * 1000000.0) as u32;

        let checkpoint = Checkpoint {
            id: next_checkpoint_id,
            position,
            color: "#ffffff".to_string(),
        };

        points.update(|points| points.checkpoints.push(checkpoint.clone()));

        active_checkpoint_id.set(Some(next_checkpoint_id));
        next_checkpoint_id += 1;
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
                key=|checkpoint| checkpoint.read().id
                let(checkpoint)
            >
                <DraggableArrow
                    checkpoint
                    active_checkpoint_id
                    bar_width=bar_width
                    bar_left=bar_left
                />
            </For>

            // Color Picker UI
            {move || {
                active_checkpoint_id.get().map(|active_checkpoint_id| {
                    let active_checkpoint = AtKeyed::new(points.checkpoints(), active_checkpoint_id);
                    view! {
                        <div class="mt-4 flex items-center space-x-2">
                            <label class="text-sm font-medium">"Selected Color:"</label>
                            <input
                                type="color"
                                class="w-8 h-8 rounded border shadow"
                                prop:value=active_checkpoint.read().color.clone()
                                on:input=move |ev| {
                                    let new_color = event_target_value(&ev);
                                    active_checkpoint.color().set(new_color);
                                }
                            />
                        </div>
                    }
                })
            }}
        </div>
    }
}
