use itertools::Itertools;
use leptos::prelude::*;
use leptos_use::{use_draggable, use_element_bounding};
use reactive_stores::{AtKeyed, Store};
use wasm_bindgen::JsCast;

use super::smoothstep;

fn hex_to_rgb(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some([r, g, b])
}

#[derive(Store, Debug, Default, Clone)]
pub struct Points {
    #[store(key: u8 = |checkpoint| checkpoint.id)]
    checkpoints: Vec<Checkpoint>,
    next_id: u8,
}

impl Points {
    fn add_checkpoint(&mut self, position: f64, color: [u8; 3]) -> u8 {
        let id = self.next_id;
        self.checkpoints.push(Checkpoint { id, position, color });
        self.next_id += 1;
        id
    }

    fn export(&self) -> Vec<(f64, [u8; 3])> {
        if self.checkpoints.is_empty() {
            vec![(0.0, [255, 255, 255])]
        } else {
            self.checkpoints
                .iter()
                .sorted_by(|a, b| a.position.partial_cmp(&b.position).unwrap())
                .map(|c| (c.position, c.color))
                .collect()
        }
    }
}

#[derive(Store, Debug, Clone, PartialEq)]
struct Checkpoint {
    id: u8,
    position: f64,
    color: [u8; 3],
}

impl Checkpoint {
    fn _color_rgb(&self) -> String {
        let [r, g, b] = self.color;
        format!("rgb({},{},{})", r, g, b)
    }

    fn color_hex(&self) -> String {
        let [r, g, b] = self.color;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

#[component]
fn DraggableArrow(
    checkpoint: AtKeyed<Store<Points>, Points, u8, Vec<Checkpoint>>,
    active_checkpoint_id: RwSignal<Option<u8>>,
    #[prop(into)] bar_width: Signal<f64>,
    #[prop(into)] bar_left: Signal<f64>,
) -> impl IntoView {
    let el = NodeRef::<leptos::html::Div>::new();
    let draggable = use_draggable(el);

    Effect::new(move |_| {
        if draggable.is_dragging.get() {
            let x = draggable.x.get();
            if x > 0.0 {
                let new_position = ((x + 8.0 - bar_left.get()) / bar_width.get()).max(0.0).min(1.0);
                checkpoint.position().set(new_position);
            }
        }
    });

    let left_px = move || checkpoint.position().get() * bar_width.get();

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
fn Bar(
    position: WriteSignal<f64>,
    width: WriteSignal<f64>,
    points: Store<Points>,
    mut on_click: impl FnMut(f64) -> () + 'static,
) -> impl IntoView {
    let node_ref = NodeRef::<leptos::html::Canvas>::new();
    let bounding = use_element_bounding(node_ref);

    Effect::new(move || position.set(bounding.left.get()));
    Effect::new(move || width.set(bounding.width.get()));

    let emit_click = move |ev: web_sys::MouseEvent| {
        let rect = ev
            .target()
            .unwrap()
            .unchecked_into::<web_sys::HtmlElement>()
            .get_bounding_client_rect();
        let click_x = ev.client_x() as f64 - rect.left() as f64;
        let position = (click_x / bounding.width.get()).clamp(0.0, 1.0);
        on_click(position)
    };

    let step_gradient = move |t: f64| {
        let points = points.read();
        let checkpoints = points
            .checkpoints
            .iter()
            .sorted_by(|a, b| a.position.partial_cmp(&b.position).unwrap())
            .collect::<Vec<_>>();

        if checkpoints.is_empty() {
            return [255, 255, 255];
        }

        let (a, b, local_t) = checkpoints
            .windows(2)
            .find(|&pair| {
                let [a, b] = *pair else { unreachable!() };
                t >= a.position && t < b.position
            })
            .map(|pair| {
                let [a, b] = *pair else { unreachable!() };
                (a, b, (t - a.position) / (b.position - a.position))
            })
            .unwrap_or_else(|| {
                let first = checkpoints[0];
                let last = checkpoints.last().unwrap();
                let range = 1.0 - last.position + first.position;
                (last, first, ((t - last.position + 1.0) % 1.0) / range)
            });
        smoothstep(&a.color, &b.color, local_t)
    };

    Effect::new(move |_| {
        if let Some(canvas) = node_ref.get() {
            let width = bounding.width.get() as u32;
            let height = bounding.height.get() as u32;

            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            canvas.set_width(width);
            canvas.set_height(height);

            for x in 0..width {
                let t = x as f64 / width as f64;
                let [r, g, b] = step_gradient(t);
                ctx.set_fill_style_str(&format!("rgb({},{},{})", r, g, b));
                ctx.fill_rect(x as f64, 0.0, 1.0, height as f64);
            }
        }
    });

    view! {
        <canvas node_ref=node_ref on:click=emit_click class="h-10 w-full rounded cursor-crosshair"/>
    }
}

#[component]
pub fn Editor(on_change: impl Fn(Vec<(f64, [u8; 3])>) + 'static) -> impl IntoView {
    let bar_width = RwSignal::new(0.0);
    let bar_left = RwSignal::new(0.0);

    let points = Store::new(Points::default());
    points.update(|points| {
        points.add_checkpoint(0.0, hex_to_rgb("#ff0000").unwrap());
        points.add_checkpoint(0.333, hex_to_rgb("#00ff00").unwrap());
        points.add_checkpoint(0.666, hex_to_rgb("#0000ff").unwrap());
    });

    let active_checkpoint_id = RwSignal::new(Some(0));

    Effect::new(move || on_change(points.read().export()));

    let on_gradient_click = move |position| {
        let mut checkpoint_id = 0;
        points.update(|points| {
            checkpoint_id = points.add_checkpoint(position, hex_to_rgb("#ffffff").unwrap());
        });
        active_checkpoint_id.set(Some(checkpoint_id));
    };

    view! {
        <div class="relative w-full">
            <Bar
                position=bar_left.write_only()
                width=bar_width.write_only()
                points
                on_click=on_gradient_click
            />

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
                active_checkpoint_id.get().map(|id| {
                    let active_checkpoint = AtKeyed::new(points.checkpoints(), id);
                    view! {
                        <div class="mt-4 flex items-center space-x-4">
                            <div class="flex items-center space-x-2">
                                <label class="text-sm font-medium">"Selected Color:"</label>
                                <input
                                    type="color"
                                    class="w-8 h-8 rounded border shadow"
                                    prop:value=active_checkpoint.read().color_hex()
                                    on:input=move |ev| {
                                        let new_color = hex_to_rgb(&event_target_value(&ev)).unwrap();
                                        active_checkpoint.color().set(new_color);
                                    }
                                />
                            </div>
                            <button
                                class="px-4 py-1 bg-red-500 text-white rounded hover:bg-red-600"
                                on:click=move |_| {
                                    points.update(|points| {
                                        points.checkpoints.retain(|checkpoint| checkpoint.id != id);
                                    });
                                    points.checkpoints().update_keys();
                                    active_checkpoint_id.set(None);
                                }
                            >
                                "Delete Checkpoint"
                            </button>
                        </div>
                    }
                })
            }}
        </div>
    }
}
