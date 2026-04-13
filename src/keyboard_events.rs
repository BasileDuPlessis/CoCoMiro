//! # Keyboard Event Handlers
//!
//! This module handles all keyboard-related user interactions for the CoCoMiro application.
//! It manages keyboard events for viewport navigation, sticky note deletion, and zoom controls.

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlCanvasElement, KeyboardEvent};

/// Handles keyboard events for viewport navigation and note management.
///
/// This function provides keyboard shortcuts for common operations:
/// - Arrow keys: Pan the viewport in the respective direction
/// - +/-: Zoom in/out centered on viewport center
/// - 0/Home: Reset viewport to default position and zoom
/// - Delete/Backspace: Delete the currently selected sticky note
///
/// All handled keys prevent default browser behavior to avoid conflicts
/// with page scrolling or other browser shortcuts.
///
/// # Arguments
/// * `event` - The keyboard event
/// * `canvas` - Reference to the canvas element
/// * `state` - Reference to application state
/// * `render` - Closure to trigger re-rendering
#[cfg(target_arch = "wasm32")]
pub fn handle_key_down(
    event: KeyboardEvent,
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    render: &Rc<dyn Fn()>,
) -> Result<(), crate::AppError> {
    let viewport_width = f64::from(canvas.client_width().max(1));
    let viewport_height = f64::from(canvas.client_height().max(1));
    let viewport = &mut state.borrow_mut().viewport;

    let handled = match event.key().as_str() {
        "ArrowLeft" => {
            viewport.pan_by(-crate::event_constants::KEYBOARD_PAN_STEP, 0.0);
            crate::log_info(&format!("Panned left by {}", crate::event_constants::KEYBOARD_PAN_STEP));
            true
        }
        "ArrowRight" => {
            viewport.pan_by(crate::event_constants::KEYBOARD_PAN_STEP, 0.0);
            crate::log_info(&format!("Panned right by {}", crate::event_constants::KEYBOARD_PAN_STEP));
            true
        }
        "ArrowUp" => {
            viewport.pan_by(0.0, -crate::event_constants::KEYBOARD_PAN_STEP);
            crate::log_info(&format!("Panned up by {}", crate::event_constants::KEYBOARD_PAN_STEP));
            true
        }
        "ArrowDown" => {
            viewport.pan_by(0.0, crate::event_constants::KEYBOARD_PAN_STEP);
            crate::log_info(&format!("Panned down by {}", crate::event_constants::KEYBOARD_PAN_STEP));
            true
        }
        "+" | "=" => {
            let old_zoom = viewport.zoom;
            viewport.zoom_at(
                crate::event_constants::ZOOM_STEP_FACTOR,
                viewport_width / 2.0,
                viewport_height / 2.0,
                viewport_width,
                viewport_height,
            );
            crate::log_info(&format!(
                "Zoomed in from {:.2} to {:.2}",
                old_zoom, viewport.zoom
            ));
            true
        }
        "-" | "_" => {
            let old_zoom = viewport.zoom;
            viewport.zoom_at(
                1.0 / crate::event_constants::ZOOM_STEP_FACTOR,
                viewport_width / 2.0,
                viewport_height / 2.0,
                viewport_width,
                viewport_height,
            );
            crate::log_info(&format!(
                "Zoomed out from {:.2} to {:.2}",
                old_zoom, viewport.zoom
            ));
            true
        }
        "0" | "Home" => {
            viewport.reset();
            crate::log_info("Viewport reset to default");
            true
        }
        "Delete" | "Backspace" => {
            if state.borrow().sticky_notes.selected_note_id.is_some() {
                state.borrow_mut().sticky_notes.delete_selected();
                crate::log_info("Deleted selected sticky note");
                true
            } else {
                false
            }
        }
        _ => false,
    };

    if handled {
        event.prevent_default();
        render();
    }

    Ok(())
}