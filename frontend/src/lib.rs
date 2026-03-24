use wasm_bindgen::JsCast;
use web_sys::{
    window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent, PointerEvent, WheelEvent,
};
use yew::prelude::*;

mod api;
use hello_world_shared::{Position, Size, StickyNote};

#[derive(Clone, PartialEq)]
struct ViewState {
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
    is_dragging: bool,
    last_mouse_pos: Option<(f64, f64)>,
}

#[derive(Clone, PartialEq)]
struct ToolbarState {
    x: f64,
    y: f64,
    is_dragging: bool,
    drag_offset: Option<(f64, f64)>,
}

#[derive(Clone, PartialEq)]
struct StickyNotesState {
    notes: Vec<StickyNote>,
    editing_note_id: Option<String>,
    editing_content: Option<String>,
}

#[derive(Properties, PartialEq)]
struct FloatingToolbarProps {
    on_zoom_in: Callback<MouseEvent>,
    on_zoom_out: Callback<MouseEvent>,
    on_create_sticky_note: Callback<MouseEvent>,
}

#[function_component(FloatingToolbar)]
fn floating_toolbar(props: &FloatingToolbarProps) -> Html {
    let toolbar_state = use_state(|| ToolbarState {
        x: 10.0,
        y: 10.0,
        is_dragging: false,
        drag_offset: None,
    });

    let on_pointer_down = {
        let toolbar_state = toolbar_state.clone();
        Callback::from(move |e: PointerEvent| {
            e.prevent_default();
            e.stop_propagation();
            
            // Capture the pointer to receive events even when outside the element
            if let Ok(target) = e.target().unwrap().dyn_into::<HtmlElement>() {
                let _ = target.set_pointer_capture(e.pointer_id());
            }
            
            let offset_x = e.client_x() as f64 - toolbar_state.x;
            let offset_y = e.client_y() as f64 - toolbar_state.y;

            let mut new_state = (*toolbar_state).clone();
            new_state.is_dragging = true;
            new_state.drag_offset = Some((offset_x, offset_y));
            toolbar_state.set(new_state);
        })
    };

    let on_pointer_move = {
        let toolbar_state = toolbar_state.clone();
        Callback::from(move |e: PointerEvent| {
            if toolbar_state.is_dragging {
                e.prevent_default();
                if let Some((offset_x, offset_y)) = toolbar_state.drag_offset {
                    let mut new_state = (*toolbar_state).clone();
                    new_state.x = e.client_x() as f64 - offset_x;
                    new_state.y = e.client_y() as f64 - offset_y;
                    toolbar_state.set(new_state);
                }
            }
        })
    };

    let on_pointer_up = {
        let toolbar_state = toolbar_state.clone();
        Callback::from(move |e: PointerEvent| {
            e.prevent_default();
            let mut new_state = (*toolbar_state).clone();
            new_state.is_dragging = false;
            new_state.drag_offset = None;
            toolbar_state.set(new_state);
        })
    };

    let style = format!(
        "position: absolute; left: {}px; top: {}px; z-index: 10; display: flex; flex-direction: column; gap: 5px; background: rgba(255, 255, 255, 0.9); border-radius: 8px; padding: 8px; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15); backdrop-filter: blur(10px); cursor: {}; user-select: none; touch-action: none;",
        toolbar_state.x,
        toolbar_state.y,
        if toolbar_state.is_dragging { "grabbing" } else { "grab" }
    );

    html! {
        <div
            {style}
            onpointerdown={on_pointer_down}
            onpointermove={on_pointer_move}
            onpointerup={on_pointer_up}
        >
            <div style="
                width: 100%;
                height: 8px;
                background: linear-gradient(90deg, #ccc 25%, transparent 25%, transparent 75%, #ccc 75%);
                background-size: 4px 4px;
                border-radius: 4px 4px 0 0;
                margin: -8px -8px 8px -8px;
                cursor: grab;
            " title="Drag to move toolbar"></div>
            <button
                onclick={&props.on_zoom_in}
                style="
                    width: 32px;
                    height: 32px;
                    border: 1px solid #ccc;
                    border-radius: 4px;
                    background: white;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    font-weight: bold;
                "
                title="Zoom In"
            >
                {"+"}
            </button>
            <button
                onclick={&props.on_zoom_out}
                style="
                    width: 32px;
                    height: 32px;
                    border: 1px solid #ccc;
                    border-radius: 4px;
                    background: white;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    font-weight: bold;
                "
                title="Zoom Out"
            >
                {"-"}
            </button>
            <button
                onclick={&props.on_create_sticky_note}
                style="
                    width: 32px;
                    height: 32px;
                    border: 1px solid #ccc;
                    border-radius: 4px;
                    background: #FFFF88;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-size: 16px;
                    font-weight: bold;
                "
                title="Create Sticky Note"
            >
                {"📝"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct StickyNoteProps {
    note: StickyNote,
    view_state: ViewState,
    is_editing: bool,
    editing_content: Option<String>,
    on_start_edit: Callback<String>,
    on_save_edit: Callback<MouseEvent>,
    on_cancel_edit: Callback<MouseEvent>,
    on_update_content: Callback<String>,
}

#[function_component(StickyNoteComponent)]
fn sticky_note_component(props: &StickyNoteProps) -> Html {
    let note = &props.note;
    let view_state = &props.view_state;

    // Ref for the textarea
    let textarea_ref = use_node_ref();

    // Focus textarea when entering edit mode
    {
        let textarea_ref = textarea_ref.clone();
        let is_editing = props.is_editing;
        use_effect_with_deps(
            move |_| {
                if is_editing {
                    if let Some(textarea) = textarea_ref.cast::<web_sys::HtmlTextAreaElement>() {
                        let _ = textarea.focus();
                    }
                }
            },
            is_editing,
        );
    }

    // Transform note position based on view state
    let screen_x = note.position.x * view_state.zoom + view_state.pan_x;
    let screen_y = note.position.y * view_state.zoom + view_state.pan_y;
    let screen_width = note.size.width * view_state.zoom;
    let screen_height = note.size.height * view_state.zoom;

    let style = format!(
        "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; background: #FFFF88; border: 2px solid #CCCC00; padding: 8px; box-sizing: border-box; font-family: Arial, sans-serif; font-size: {}px; cursor: pointer; user-select: none; z-index: 5;",
        screen_x,
        screen_y,
        screen_width,
        screen_height,
        16.0 * view_state.zoom.max(0.5)
    );

    if props.is_editing {
        // Edit mode - show textarea
        let on_input = {
            let on_update_content = props.on_update_content.clone();
            Callback::from(move |e: web_sys::InputEvent| {
                if let Ok(target) = e.target().unwrap().dyn_into::<web_sys::HtmlTextAreaElement>() {
                    let value = target.value();
                    on_update_content.emit(value);
                }
            })
        };

        let on_keydown = {
            let on_save_edit = props.on_save_edit.clone();
            let on_cancel_edit = props.on_cancel_edit.clone();
            Callback::from(move |e: KeyboardEvent| {
                if e.key() == "Enter" && !e.shift_key() {
                    e.prevent_default();
                    on_save_edit.emit(web_sys::MouseEvent::new("click").unwrap());
                } else if e.key() == "Escape" {
                    e.prevent_default();
                    on_cancel_edit.emit(web_sys::MouseEvent::new("click").unwrap());
                }
            })
        };

        html! {
            <textarea
                ref={textarea_ref}
                style={format!("{} resize: none; border: none; outline: none; background: #FFFF88;", style)}
                value={props.editing_content.clone().unwrap_or_default()}
                oninput={on_input}
                onkeydown={on_keydown}
                autofocus={true}
            />
        }
    } else {
        // Display mode - show text
        let on_click = {
            let note_id = note.id.clone();
            let on_start_edit = props.on_start_edit.clone();
            Callback::from(move |_| {
                on_start_edit.emit(note_id.clone());
            })
        };

        html! {
            <div {style} onclick={on_click}>
                { for note.content.split('\n').enumerate().map(|(_, line)| {
                    html! {
                        <div style={format!("margin-bottom: {}px;", 4.0 * view_state.zoom.max(0.5))}>
                            {line}
                        </div>
                    }
                })}
            </div>
        }
    }
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

    let sticky_notes_state = use_state(|| StickyNotesState {
        notes: Vec::new(),
        editing_note_id: None,
        editing_content: None,
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
        let _sticky_notes_state = sticky_notes_state.clone();
        let get_context = get_context.clone();
        #[allow(deprecated)]
        move || {
            if let Some(ctx) = get_context() {
                let canvas = canvas_ref_clone.cast::<HtmlCanvasElement>().unwrap();
                let width = canvas.width() as f64;
                let height = canvas.height() as f64;

                // Clear canvas with white background
                ctx.set_fill_style(&"#FFFFFF".into());
                ctx.fill_rect(0.0, 0.0, width, height);

                // Draw grid
                draw_grid(&ctx, width, height, &view_state);

                // Draw debug overlay
                draw_debug_overlay(&ctx, width, height, &view_state);
            }
        }
    };

    // Draw grid function
    #[allow(deprecated)]
    fn draw_grid(ctx: &CanvasRenderingContext2d, width: f64, height: f64, view_state: &ViewState) {
        let grid_spacing = 50.0 * view_state.zoom;
        let line_width = (1.0 / view_state.zoom).clamp(0.5, 2.0);

        ctx.set_stroke_style(&"#E0E0E0".into());
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
    #[allow(deprecated)]
    fn draw_debug_overlay(
        ctx: &CanvasRenderingContext2d,
        width: f64,
        _height: f64,
        view_state: &ViewState,
    ) {
        let debug_text = format!(
            "Zoom: {:.2}\nPan: ({:.1}, {:.1})\nDragging: {}",
            view_state.zoom, view_state.pan_x, view_state.pan_y, view_state.is_dragging
        );

        ctx.set_fill_style(&"rgba(0, 0, 0, 0.7)".into());
        ctx.set_font("14px monospace");
        ctx.fill_rect(width - 200.0, 10.0, 190.0, 60.0);

        ctx.set_fill_style(&"#FFFFFF".into());
        for (i, line) in debug_text.lines().enumerate() {
            ctx.fill_text(line, width - 190.0, 30.0 + (i as f64 * 16.0))
                .unwrap();
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
            (
                view_state.zoom,
                view_state.pan_x,
                view_state.pan_y,
                view_state.is_dragging,
            ),
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

    // Create sticky note handler
    let create_sticky_note = {
        let sticky_notes_state = sticky_notes_state.clone();
        let view_state = view_state.clone();
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |_: MouseEvent| {
            let canvas = canvas_ref.cast::<HtmlCanvasElement>().unwrap();
            let canvas_width = canvas.width() as f64;
            let canvas_height = canvas.height() as f64;

            // Calculate center of current view in world coordinates
            let center_x = (-view_state.pan_x + canvas_width / 2.0) / view_state.zoom;
            let center_y = (-view_state.pan_y + canvas_height / 2.0) / view_state.zoom;

            let new_note = StickyNote {
                id: format!("note-{}", js_sys::Date::now()),
                position: Position {
                    x: center_x - 100.0, // Center the 200px wide note
                    y: center_y - 75.0,  // Center the 150px tall note
                },
                content: "New sticky note".to_string(),
                size: Size {
                    width: 200.0,
                    height: 150.0,
                },
            };

            let mut new_state = (*sticky_notes_state).clone();
            new_state.notes.push(new_note);
            sticky_notes_state.set(new_state);
        })
    };

    // Start editing sticky note handler
    let start_edit_note = {
        let sticky_notes_state = sticky_notes_state.clone();
        Callback::from(move |note_id: String| {
            let mut new_state = (*sticky_notes_state).clone();
            new_state.editing_note_id = Some(note_id.clone());
            if let Some(note) = new_state.notes.iter().find(|n| n.id == note_id) {
                new_state.editing_content = Some(note.content.clone());
            }
            sticky_notes_state.set(new_state);
        })
    };

    // Save edited sticky note handler
    let save_edit_note = {
        let sticky_notes_state = sticky_notes_state.clone();
        Callback::from(move |_| {
            let mut new_state = (*sticky_notes_state).clone();
            if let Some(note_id) = new_state.editing_note_id.clone() {
                if let Some(content) = new_state.editing_content.clone() {
                    if let Some(note) = new_state.notes.iter_mut().find(|n| n.id == note_id) {
                        note.content = content;
                    }
                }
            }
            new_state.editing_note_id = None;
            new_state.editing_content = None;
            sticky_notes_state.set(new_state);
        })
    };

    // Cancel editing sticky note handler
    let cancel_edit_note = {
        let sticky_notes_state_clone = sticky_notes_state.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_state = (*sticky_notes_state_clone).clone();
            new_state.editing_note_id = None;            new_state.editing_content = None;            sticky_notes_state_clone.set(new_state);
        })
    };

    // Update editing content handler
    let update_editing_content = {
        let sticky_notes_state = sticky_notes_state.clone();
        Callback::from(move |content: String| {
            let mut new_state = (*sticky_notes_state).clone();
            new_state.editing_content = Some(content);
            sticky_notes_state.set(new_state);
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
            new_state.zoom = (new_state.zoom * zoom_factor).clamp(0.1, 10.0);
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
                    let height =
                        (window.inner_height().unwrap().as_f64().unwrap() as u32).min(2000);
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
            <FloatingToolbar on_zoom_in={zoom_in} on_zoom_out={zoom_out} on_create_sticky_note={create_sticky_note} />
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
            { for sticky_notes_state.notes.iter().map(|note| {
                let is_editing = sticky_notes_state.editing_note_id.as_ref() == Some(&note.id);
                html! {
                    <StickyNoteComponent
                        note={note.clone()}
                        view_state={(*view_state).clone()}
                        is_editing={is_editing}
                        editing_content={sticky_notes_state.editing_content.clone()}
                        on_start_edit={start_edit_note.clone()}
                        on_save_edit={save_edit_note.clone()}
                        on_cancel_edit={cancel_edit_note.clone()}
                        on_update_content={update_editing_content.clone()}
                    />
                }
            })}
            { if sticky_notes_state.editing_note_id.is_some() {
                html! {
                    <div 
                        style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 4; cursor: default;"
                        onclick={save_edit_note.clone()}
                    />
                }
            } else {
                html! {}
            }}
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
