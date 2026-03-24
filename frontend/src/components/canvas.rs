use crate::components::{FloatingToolbar, StickyNoteComponent};
use crate::constants::*;
use crate::events::*;
use crate::rendering::{draw_debug_overlay, draw_grid};
use crate::state::{AppState, StickyNotesState, ToolbarState, ViewState};
use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[function_component(InfiniteCanvas)]
pub fn infinite_canvas() -> Html {
    let app_state = use_reducer(|| AppState {
        view: ViewState {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            is_dragging: false,
            last_mouse_pos: None,
        },
        toolbar: ToolbarState {
            x: TOOLBAR_INITIAL_X,
            y: TOOLBAR_INITIAL_Y,
            is_dragging: false,
            drag_offset: None,
        },
        sticky_notes: StickyNotesState {
            notes: Vec::new(),
            editing_note_id: None,
            editing_content: None,
        },
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
        let app_state = app_state.clone();
        let get_context = get_context.clone();
        #[allow(deprecated)]
        move || {
            if let Some(ctx) = get_context() {
                if let Some(canvas) = canvas_ref_clone.cast::<HtmlCanvasElement>() {
                    let width = canvas.width() as f64;
                    let height = canvas.height() as f64;

                    // Clear canvas with white background
                    ctx.set_fill_style(&CANVAS_BG.into());
                    ctx.fill_rect(0.0, 0.0, width, height);

                    // Draw grid
                    draw_grid(&ctx, width, height, &app_state.view);

                    // Draw debug overlay
                    draw_debug_overlay(&ctx, width, height, &app_state.view);
                }
            }
        }
    };

    // Effect to redraw canvas when view state changes
    {
        let draw_canvas = draw_canvas.clone();
        let app_state = app_state.clone();
        use_effect_with_deps(
            move |_| {
                draw_canvas();
            },
            (
                app_state.view.zoom,
                app_state.view.pan_x,
                app_state.view.pan_y,
                app_state.view.is_dragging,
            ),
        );
    }

    // Create event handlers using the events module
    let zoom_in = create_zoom_in_handler(&app_state);
    let zoom_out = create_zoom_out_handler(&app_state);
    let create_sticky_note = create_sticky_note_handler(&app_state, &canvas_ref);
    let start_edit_note = create_start_edit_handler(&app_state);
    let save_edit_note = create_save_edit_handler(&app_state);
    let cancel_edit_note = create_cancel_edit_handler(&app_state);
    let update_editing_content = create_update_content_handler(&app_state);
    let on_mouse_down = create_mouse_down_handler(&app_state);
    let on_mouse_move = create_mouse_move_handler(&app_state);
    let on_mouse_up = create_mouse_up_handler(&app_state);
    let on_wheel = create_wheel_handler(&app_state);
    let on_key_down = create_key_down_handler(&app_state);

    // Effect to set up canvas size and initial draw
    {
        let canvas_ref = canvas_ref.clone();
        let draw_canvas = draw_canvas.clone();
        use_effect(move || {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Some(window) = window() {
                    let width = window
                        .inner_width()
                        .ok()
                        .and_then(|w| w.as_f64())
                        .map(|w| w as u32)
                        .unwrap_or(CANVAS_MAX_WIDTH)
                        .min(CANVAS_MAX_WIDTH);
                    let height = window
                        .inner_height()
                        .ok()
                        .and_then(|h| h.as_f64())
                        .map(|h| h as u32)
                        .unwrap_or(CANVAS_MAX_HEIGHT)
                        .min(CANVAS_MAX_HEIGHT);
                    canvas.set_width(width);
                    canvas.set_height(height);
                    draw_canvas();
                } else {
                    log::error!("Window object not available for canvas setup");
                }
            } else {
                log::error!("Canvas element not found for setup");
            }
            || ()
        });
    }

    html! {
        <div style="position: relative; width: 100vw; height: 100vh; overflow: hidden;">
            <FloatingToolbar app_state={app_state.clone()} on_zoom_in={zoom_in} on_zoom_out={zoom_out} on_create_sticky_note={create_sticky_note} />
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
            { for app_state.sticky_notes.notes.iter().map(|note| {
                let is_editing = app_state.sticky_notes.editing_note_id.as_ref() == Some(&note.id);
                html! {
                    <StickyNoteComponent
                        note={note.clone()}
                        app_state={app_state.clone()}
                        is_editing={is_editing}
                        editing_content={app_state.sticky_notes.editing_content.clone()}
                        on_start_edit={start_edit_note.clone()}
                        on_save_edit={save_edit_note.clone()}
                        on_cancel_edit={cancel_edit_note.clone()}
                        on_update_content={update_editing_content.clone()}
                    />
                }
            })}
            { if app_state.sticky_notes.editing_note_id.is_some() {
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
