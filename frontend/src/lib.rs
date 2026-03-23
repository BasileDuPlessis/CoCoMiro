use yew::prelude::*;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d, MouseEvent, WheelEvent, KeyboardEvent};
use wasm_bindgen::JsCast;

mod api;

#[derive(Clone, PartialEq)]
struct ViewState {
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
    is_dragging: bool,
    last_mouse_pos: Option<(f64, f64)>,
}

#[function_component(InfiniteCanvas)]
fn infinite_canvas() -> Html {
    let view_state = use_state(|| ViewState {
        zoom: 1.0,
        pan_x: 0.0,
        pan_y: 0.0,
        is_dragging: false,
        last_mouse_pos: None,
    });

    let canvas_ref = use_node_ref();

    // Function to get canvas context
    let get_context = {
        let canvas_ref = canvas_ref.clone();
        move || -> Option<CanvasRenderingContext2d> {
            canvas_ref
                .cast::<HtmlCanvasElement>()
                .and_then(|canvas| canvas.get_context("2d").ok())
                .flatten()
                .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
        }
    };

    let canvas_ref_clone = canvas_ref.clone();
    // Function to draw the canvas
    let draw_canvas = {
        let view_state = view_state.clone();
        let get_context = get_context.clone();
        move || {
            if let Some(ctx) = get_context() {
                let canvas = canvas_ref_clone.cast::<HtmlCanvasElement>().unwrap();
                let width = canvas.width() as f64;
                let height = canvas.height() as f64;

                // Clear canvas with white background
                ctx.set_fill_style(&wasm_bindgen::JsValue::from("#FFFFFF"));
                ctx.fill_rect(0.0, 0.0, width, height);

                // Draw grid
                draw_grid(&ctx, width, height, &view_state);

                // Draw debug overlay
                draw_debug_overlay(&ctx, width, height, &view_state);
            }
        }
    };

    // Draw grid function
    fn draw_grid(ctx: &CanvasRenderingContext2d, width: f64, height: f64, view_state: &ViewState) {
        let grid_spacing = 50.0 * view_state.zoom;
        let line_width = (1.0 / view_state.zoom).max(0.5).min(2.0);

        ctx.set_stroke_style(&wasm_bindgen::JsValue::from("#E0E0E0"));
        ctx.set_line_width(line_width);

        // Calculate grid offset based on pan
        let offset_x = view_state.pan_x % grid_spacing;
        let offset_y = view_state.pan_y % grid_spacing;

        // Draw vertical lines
        let mut x = offset_x;
        while x < width {
            ctx.begin_path();
            ctx.move_to(x, 0.0);
            ctx.line_to(x, height);
            ctx.stroke();
            x += grid_spacing;
        }

        // Draw horizontal lines
        let mut y = offset_y;
        while y < height {
            ctx.begin_path();
            ctx.move_to(0.0, y);
            ctx.line_to(width, y);
            ctx.stroke();
            y += grid_spacing;
        }
    }

    // Draw debug overlay
    fn draw_debug_overlay(ctx: &CanvasRenderingContext2d, width: f64, _height: f64, view_state: &ViewState) {
        let debug_text = format!(
            "Zoom: {:.2}\nPan: ({:.1}, {:.1})\nDragging: {}",
            view_state.zoom, view_state.pan_x, view_state.pan_y, view_state.is_dragging
        );

        ctx.set_fill_style(&wasm_bindgen::JsValue::from("rgba(0, 0, 0, 0.7)"));
        ctx.set_font("14px monospace");
        ctx.fill_rect(width - 200.0, 10.0, 190.0, 60.0);

        ctx.set_fill_style(&wasm_bindgen::JsValue::from("#FFFFFF"));
        for (i, line) in debug_text.lines().enumerate() {
            ctx.fill_text(line, width - 190.0, 30.0 + (i as f64 * 16.0)).unwrap();
        }
    }

    // Effect to redraw canvas when view state changes
    {
        let draw_canvas = draw_canvas.clone();
        let view_state = view_state.clone();
        use_effect_with_deps(
            move |_| {
                draw_canvas();
            },
            (view_state.zoom, view_state.pan_x, view_state.pan_y, view_state.is_dragging),
        );
    }

    // Zoom in handler
    let zoom_in = {
        let view_state = view_state.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_state = (*view_state).clone();
            new_state.zoom = (new_state.zoom * 1.2).min(10.0); // Max zoom 1000%
            view_state.set(new_state);
        })
    };

    // Zoom out handler
    let zoom_out = {
        let view_state = view_state.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_state = (*view_state).clone();
            new_state.zoom = (new_state.zoom / 1.2).max(0.1);
            view_state.set(new_state);
        })
    };

    // Mouse down handler
    let on_mouse_down = {
        let view_state = view_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let mut new_state = (*view_state).clone();
            new_state.is_dragging = true;
            new_state.last_mouse_pos = Some((e.client_x() as f64, e.client_y() as f64));
            view_state.set(new_state);
        })
    };

    // Mouse move handler
    let on_mouse_move = {
        let view_state = view_state.clone();
        Callback::from(move |e: MouseEvent| {
            if view_state.is_dragging {
                e.prevent_default();
                if let Some((last_x, last_y)) = view_state.last_mouse_pos {
                    let current_x = e.client_x() as f64;
                    let current_y = e.client_y() as f64;
                    let delta_x = current_x - last_x;
                    let delta_y = current_y - last_y;

                    let mut new_state = (*view_state).clone();
                    new_state.pan_x += delta_x;
                    new_state.pan_y += delta_y;
                    new_state.last_mouse_pos = Some((current_x, current_y));
                    view_state.set(new_state);
                }
            }
        })
    };

    // Mouse up handler
    let on_mouse_up = {
        let view_state = view_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let mut new_state = (*view_state).clone();
            new_state.is_dragging = false;
            new_state.last_mouse_pos = None;
            view_state.set(new_state);
        })
    };

    // Wheel handler for zoom
    let on_wheel = {
        let view_state = view_state.clone();
        Callback::from(move |e: WheelEvent| {
            e.prevent_default();
            let delta = e.delta_y();
            let zoom_factor = if delta > 0.0 { 0.8333 } else { 1.2 }; // ~16.7% out, +20% in

            let mut new_state = (*view_state).clone();
            new_state.zoom = (new_state.zoom * zoom_factor).max(0.1).min(10.0);
            view_state.set(new_state);
        })
    };

    // Keyboard handler
    let on_key_down = {
        let view_state = view_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.ctrl_key() || e.meta_key() {
                match e.key().as_str() {
                    "+" | "=" => {
                        e.prevent_default();
                        let mut new_state = (*view_state).clone();
                        new_state.zoom = (new_state.zoom * 1.2).min(10.0);
                        view_state.set(new_state);
                    }
                    "-" => {
                        e.prevent_default();
                        let mut new_state = (*view_state).clone();
                        new_state.zoom = (new_state.zoom / 1.2).max(0.1);
                        view_state.set(new_state);
                    }
                    _ => {}
                }
            }
        })
    };

    // Effect to set up canvas size and initial draw
    {
        let canvas_ref = canvas_ref.clone();
        let draw_canvas = draw_canvas.clone();
        use_effect(move || {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Some(window) = window() {
                    let width = (window.inner_width().unwrap().as_f64().unwrap() as u32).min(3000);
                    let height = (window.inner_height().unwrap().as_f64().unwrap() as u32).min(2000);
                    canvas.set_width(width);
                    canvas.set_height(height);
                    draw_canvas();
                }
            }
            || ()
        });
    }

    html! {
        <div style="position: relative; width: 100vw; height: 100vh; overflow: hidden;">
            <button onclick={zoom_in} style="position: absolute; top: 10px; left: 10px; z-index: 10;">{"+"}</button>
            <button onclick={zoom_out} style="position: absolute; top: 40px; left: 10px; z-index: 10;">{"-"}</button>
            <canvas
                ref={canvas_ref}
                style="cursor: grab; display: block;"
                onmousedown={on_mouse_down}
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up}
                onwheel={on_wheel}
                onkeydown={on_key_down}
                tabindex="0"
            />
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <InfiniteCanvas />
    }
}

#[cfg(not(test))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    yew::Renderer::<App>::new().render();
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_zoom_limits() {
        let view_state = ViewState {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            is_dragging: false,
            last_mouse_pos: None,
        };

        // Test zoom in
        let zoomed_in = ViewState {
            zoom: (view_state.zoom * 1.2).min(10.0),
            ..view_state
        };
        assert!(zoomed_in.zoom >= 1.0);

        // Test zoom out
        let zoomed_out = ViewState {
            zoom: (view_state.zoom / 1.2).max(0.1),
            ..view_state
        };
        assert!(zoomed_out.zoom >= 0.1);
    }
}