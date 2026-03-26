use crate::constants::*;
use crate::state::{AppAction, AppState, StickyNotesAction, ViewAction};
use cocomiro_shared::{Position, Size};
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent, WheelEvent};
use yew::prelude::*;

pub fn create_zoom_in_handler(app_state: &UseReducerHandle<AppState>) -> Callback<MouseEvent> {
    let app_state = app_state.clone();
    Callback::from(move |_: MouseEvent| {
        app_state.dispatch(AppAction::View(ViewAction::ZoomIn));
    })
}

pub fn create_zoom_out_handler(app_state: &UseReducerHandle<AppState>) -> Callback<MouseEvent> {
    let app_state = app_state.clone();
    Callback::from(move |_: MouseEvent| {
        app_state.dispatch(AppAction::View(ViewAction::ZoomOut));
    })
}

pub fn create_sticky_note_handler(
    app_state: &UseReducerHandle<AppState>,
    canvas_ref: &NodeRef,
) -> Callback<MouseEvent> {
    let app_state = app_state.clone();
    let canvas_ref = canvas_ref.clone();
    Callback::from(move |_: MouseEvent| {
        if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
            let canvas_width = canvas.width() as f64;
            let canvas_height = canvas.height() as f64;

            // Calculate center of current view in world coordinates
            let center_x = (-app_state.view.pan_x + canvas_width / 2.0) / app_state.view.zoom;
            let center_y = (-app_state.view.pan_y + canvas_height / 2.0) / app_state.view.zoom;

            app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::CreateNote(
                Position {
                    x: center_x - STICKY_NOTE_CENTER_OFFSET_X,
                    y: center_y - STICKY_NOTE_CENTER_OFFSET_Y,
                },
                Size {
                    width: STICKY_NOTE_DEFAULT_WIDTH,
                    height: STICKY_NOTE_DEFAULT_HEIGHT,
                },
            )));
        }
    })
}

pub fn create_start_edit_handler(app_state: &UseReducerHandle<AppState>) -> Callback<String> {
    let app_state = app_state.clone();
    Callback::from(move |note_id: String| {
        app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::StartEdit(
            note_id,
        )));
    })
}

pub fn create_save_edit_handler(app_state: &UseReducerHandle<AppState>) -> Callback<MouseEvent> {
    let app_state = app_state.clone();
    Callback::from(move |_| {
        app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::SaveEdit));
    })
}

pub fn create_cancel_edit_handler(app_state: &UseReducerHandle<AppState>) -> Callback<MouseEvent> {
    let app_state = app_state.clone();
    Callback::from(move |_: MouseEvent| {
        app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::CancelEdit));
    })
}

pub fn create_update_content_handler(app_state: &UseReducerHandle<AppState>) -> Callback<String> {
    let app_state = app_state.clone();
    Callback::from(move |content: String| {
        app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::UpdateContent(
            content,
        )));
    })
}

pub fn create_select_note_handler(app_state: &UseReducerHandle<AppState>) -> Callback<String> {
    let app_state = app_state.clone();
    Callback::from(move |note_id: String| {
        app_state.dispatch(AppAction::StickyNotes(StickyNotesAction::SelectNote(
            note_id,
        )));
    })
}

pub fn create_mouse_down_handler(app_state: &UseReducerHandle<AppState>) -> Callback<MouseEvent> {
    let app_state = app_state.clone();
    Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        app_state.dispatch(AppAction::View(ViewAction::StartDrag(
            e.client_x() as f64,
            e.client_y() as f64,
        )));
    })
}

pub fn create_mouse_move_handler(app_state: &UseReducerHandle<AppState>) -> Callback<MouseEvent> {
    let app_state = app_state.clone();
    Callback::from(move |e: MouseEvent| {
        if app_state.view.is_dragging {
            e.prevent_default();
            app_state.dispatch(AppAction::View(ViewAction::UpdateDrag(
                e.client_x() as f64,
                e.client_y() as f64,
            )));
        }
    })
}

pub fn create_mouse_up_handler(app_state: &UseReducerHandle<AppState>) -> Callback<MouseEvent> {
    let app_state = app_state.clone();
    Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        app_state.dispatch(AppAction::View(ViewAction::EndDrag));
    })
}

pub fn create_wheel_handler(app_state: &UseReducerHandle<AppState>) -> Callback<WheelEvent> {
    let app_state = app_state.clone();
    Callback::from(move |e: WheelEvent| {
        e.prevent_default();
        let delta = e.delta_y();
        let zoom_factor = if delta > 0.0 {
            ZOOM_FACTOR_OUT
        } else {
            ZOOM_FACTOR_IN
        };
        app_state.dispatch(AppAction::View(ViewAction::ZoomBy(zoom_factor)));
    })
}

pub fn create_key_down_handler(app_state: &UseReducerHandle<AppState>) -> Callback<KeyboardEvent> {
    let app_state = app_state.clone();
    Callback::from(move |e: KeyboardEvent| {
        if e.ctrl_key() || e.meta_key() {
            match e.key().as_str() {
                "+" | "=" => {
                    e.prevent_default();
                    app_state.dispatch(AppAction::View(ViewAction::ZoomIn));
                }
                "-" => {
                    e.prevent_default();
                    app_state.dispatch(AppAction::View(ViewAction::ZoomOut));
                }
                _ => {}
            }
        }
    })
}
