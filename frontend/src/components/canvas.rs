use crate::components::{FloatingToolbar, StickyNoteComponent};
use crate::constants::{
    CANVAS_BG, CANVAS_MAX_HEIGHT, CANVAS_MAX_WIDTH, TOOLBAR_INITIAL_X, TOOLBAR_INITIAL_Y,
};
use crate::error::{CanvasError, CanvasResult};
use crate::events::*;
use crate::performance::PerformanceLogger;
use crate::rendering::{draw_debug_overlay, draw_grid};
use crate::state::{
    AppAction, AppState, StickyNotesAction, StickyNotesState, ToolbarState, ViewState,
};
use crate::styles::CanvasStyle;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[function_component(InfiniteCanvas)]
pub fn infinite_canvas() -> Html {
    let styles = CanvasStyle::new();
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
            selected_note_id: None,
        },
    });

    let canvas_ref = use_node_ref();

    // Function to get canvas context
    let get_context =
        {
            let canvas_ref = canvas_ref.clone();
            move || -> CanvasResult<CanvasRenderingContext2d> {
                let canvas = canvas_ref.cast::<HtmlCanvasElement>().ok_or(
                    CanvasError::ElementCastFailed("HtmlCanvasElement".to_string()),
                )?;

                let context = canvas
                    .get_context("2d")
                    .map_err(|_| CanvasError::ContextNotAvailable)?
                    .ok_or(CanvasError::ContextNotAvailable)?;

                context.dyn_into::<CanvasRenderingContext2d>().map_err(|_| {
                    CanvasError::ElementCastFailed("CanvasRenderingContext2d".to_string())
                })
            }
        };

    let canvas_ref_clone = canvas_ref.clone();
    // Function to draw the canvas
    let draw_canvas = {
        let app_state = app_state.clone();
        let get_context = get_context.clone();
        move || {
            let start_time = js_sys::Date::now();
            match get_context() {
                Ok(ctx) => {
                    if let Some(canvas) = canvas_ref_clone.cast::<HtmlCanvasElement>() {
                        let width = canvas.width() as f64;
                        let height = canvas.height() as f64;

                        // Clear canvas with white background
                        #[allow(deprecated)]
                        ctx.set_fill_style(&JsValue::from_str(CANVAS_BG));
                        ctx.fill_rect(0.0, 0.0, width, height);

                        // Draw grid
                        draw_grid(&ctx, width, height, &app_state.view);

                        // Draw debug overlay
                        draw_debug_overlay(&ctx, width, height, &app_state.view);

                        PerformanceLogger::log_canvas_operation("canvas_render", start_time);
                    } else {
                        PerformanceLogger::log_error(
                            "canvas_render",
                            "Canvas element not found for drawing",
                        );
                    }
                }
                Err(e) => {
                    PerformanceLogger::log_error(
                        "canvas_context",
                        &format!("Failed to get canvas context: {}", e),
                    );
                }
            }
        }
    };

    // Effect to redraw canvas when view state changes
    {
        let draw_canvas = draw_canvas.clone();
        let app_state = app_state.clone();
        use_effect_with(
            (
                app_state.view.zoom,
                app_state.view.pan_x,
                app_state.view.pan_y,
                app_state.view.is_dragging,
            ),
            move |_| {
                draw_canvas();
            },
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
    let select_note = create_select_note_handler(&app_state);
    let on_mouse_down = create_mouse_down_handler(&app_state);
    let on_mouse_move = create_mouse_move_handler(&app_state);
    let on_mouse_up = create_mouse_up_handler(&app_state);
    let on_wheel = create_wheel_handler(&app_state);
    let on_key_down = create_key_down_handler(&app_state);

    // Click outside handler for saving edits and deselecting
    let on_container_click = {
        let app_state = app_state.clone();
        Callback::from(move |e: MouseEvent| {
            let has_editing = app_state.sticky_notes.editing_note_id.is_some();
            let has_selection = app_state.sticky_notes.selected_note_id.is_some();

            if has_editing {
                e.prevent_default();
                app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::SaveEdit));
                // Also deselect after saving
                app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::DeselectNote));
            } else if has_selection {
                e.prevent_default();
                app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::DeselectNote));
            }
        })
    };

    let loading_state = use_state(|| true);

    // Effect to set up canvas size and initial draw
    {
        let canvas_ref = canvas_ref.clone();
        let draw_canvas = draw_canvas.clone();
        let loading_state = loading_state.clone();
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

                    // Mark as loaded after successful setup
                    loading_state.set(false);
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
        <div class={styles.container} onclick={on_container_click}>
            <FloatingToolbar app_state={app_state.clone()} on_zoom_in={zoom_in} on_zoom_out={zoom_out} on_create_sticky_note={create_sticky_note} />
            <canvas
                ref={canvas_ref}
                class={styles.canvas}
                onmousedown={on_mouse_down}
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up}
                onwheel={on_wheel}
                onkeydown={on_key_down}
                tabindex="0"
                data-zoom={app_state.view.zoom.to_string()}
                data-pan-x={app_state.view.pan_x.to_string()}
                data-pan-y={app_state.view.pan_y.to_string()}
                data-loading={(*loading_state).to_string()}
            />
            { for app_state.sticky_notes.notes.iter().map(|note| {
                let is_editing = app_state.sticky_notes.editing_note_id.as_ref() == Some(&note.id);
                let is_selected = app_state.sticky_notes.selected_note_id.as_ref() == Some(&note.id);
                html! {
                    <StickyNoteComponent
                        note={note.clone()}
                        app_state={app_state.clone()}
                        is_editing={is_editing}
                        editing_content={app_state.sticky_notes.editing_content.clone()}
                        is_selected={is_selected}
                        on_start_edit={start_edit_note.clone()}
                        on_save_edit={save_edit_note.clone()}
                        on_cancel_edit={cancel_edit_note.clone()}
                        on_update_content={update_editing_content.clone()}
                        on_select={select_note.clone()}
                    />
                }
            })}
            { if app_state.sticky_notes.editing_note_id.is_some() {
                html! {
                    <div class={styles.overlay}
                        onclick={save_edit_note.clone()}
                        data-testid="canvas-overlay"
                    />
                }
            } else {
                html! {}
            }}
        </div>
    }
}
