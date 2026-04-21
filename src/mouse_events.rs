//! # Mouse Event Handlers
//!
//! This module handles all mouse-related user interactions for the CoCoMiro application.
//! It manages mouse down, move, up, wheel, and double-click events for canvas panning,
//! sticky note interaction, toolbar dragging, and zooming.

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlCanvasElement, HtmlElement, MouseEvent, WheelEvent};

// Import functions from other event modules
#[cfg(target_arch = "wasm32")]
use crate::event_setup::{end_drag_if_needed, end_toolbar_drag_if_needed};
#[cfg(target_arch = "wasm32")]
use crate::sticky_notes::{DEFAULT_NOTE_HEIGHT, DEFAULT_NOTE_WIDTH};
#[cfg(target_arch = "wasm32")]
use crate::text_input::create_text_input_overlay;

/// Handles mouse down events on the canvas.
///
/// This function determines the type of interaction based on:
/// 1. Which mouse button was pressed (left for panning/dragging, right for context menu)
/// 2. Whether the target element is a sticky note or the canvas background
/// 3. Current application state (tool selection, existing drag state)
///
/// For sticky notes, it initiates note dragging. For canvas background,
/// it starts viewport panning. The function prevents default browser behavior
/// and captures the mouse to ensure smooth drag operations.
///
/// # Arguments
/// * `event` - The mouse down event
/// * `canvas` - The canvas element
/// * `state` - Reference to application state
/// * `render` - Closure to trigger re-rendering
#[cfg(target_arch = "wasm32")]
pub fn handle_mouse_down(
    event: MouseEvent,
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    render: &Rc<dyn Fn()>,
) -> Result<(), crate::error::AppError> {
    if event.button() != 0 {
        return Ok(());
    }

    event.prevent_default();
    if let Err(error) = canvas.focus() {
        crate::logging::log_jsvalue_error("canvas focus failed", &error);
    }

    let mouse_x = event.offset_x() as f64;
    let mouse_y = event.offset_y() as f64;

    // Check for resize handle first (highest priority)
    let viewport_width = f64::from(canvas.client_width().max(1));
    let viewport_height = f64::from(canvas.client_height().max(1));
    let resize_handle_hit = {
        let viewport = &state.borrow().viewport;
        let sticky_notes = &state.borrow().sticky_notes;
        sticky_notes.find_resize_handle_at(
            mouse_x,
            mouse_y,
            viewport,
            viewport_width,
            viewport_height,
        )
    };

    if let Some((note_id, handle)) = resize_handle_hit {
        // Get note dimensions before mutable borrow
        let (original_width, original_height) = {
            let sticky_notes = &state.borrow().sticky_notes;
            let note = sticky_notes.get_note(note_id);
            (
                note.map(|n| n.width).unwrap_or(DEFAULT_NOTE_WIDTH),
                note.map(|n| n.height).unwrap_or(DEFAULT_NOTE_HEIGHT),
            )
        };

        // Start resizing the sticky note
        state
            .borrow_mut()
            .sticky_notes
            .start_resize(note_id, handle);
        state.borrow_mut().resizing = crate::sticky_notes::ResizingState {
            is_resizing: true,
            note_id: Some(note_id),
            handle: Some(handle),
            start_mouse_x: mouse_x,
            start_mouse_y: mouse_y,
            original_width,
            original_height,
        };
        crate::logging::log_info(&format!(
            "Resize started for note {} with handle {:?}",
            note_id, handle
        ));
        render();
        return Ok(());
    }
    // Check for sticky note next
    let note_hit = {
        let viewport_width = f64::from(canvas.client_width().max(1));
        let viewport_height = f64::from(canvas.client_height().max(1));
        let world_pos = state.borrow().viewport.world_point_at(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
        );
        state
            .borrow()
            .sticky_notes
            .find_note_at(world_pos.0, world_pos.1)
    };

    if let Some(note_id) = note_hit {
        // Start dragging the sticky note
        let world_pos = {
            let viewport_width = f64::from(canvas.client_width().max(1));
            let viewport_height = f64::from(canvas.client_height().max(1));
            state.borrow().viewport.world_point_at(
                mouse_x,
                mouse_y,
                viewport_width,
                viewport_height,
            )
        };
        state
            .borrow_mut()
            .sticky_notes
            .start_drag(note_id, world_pos.0, world_pos.1);
        crate::logging::log_info(&format!("Sticky note {} drag started", note_id));
        render();
        return Ok(());
    }

    // If no sticky note hit, clear selection and start canvas drag
    state.borrow_mut().sticky_notes.clear_selection();
    state.borrow_mut().viewport.start_drag(mouse_x, mouse_y);
    crate::logging::log_info(&format!(
        "Canvas drag started at ({}, {}), selection cleared",
        mouse_x, mouse_y
    ));
    render();
    Ok(())
}

/// Handles mouse move events for continuous drag operations and hover effects.
///
/// This function updates the mouse position in the application state and handles
/// ongoing drag operations for both sticky notes and the viewport. It prioritizes
/// toolbar dragging over canvas interactions.
///
/// The function performs the following operations in order:
/// 1. Updates current mouse coordinates in app state
/// 2. Handles toolbar dragging if active
/// 3. Handles sticky note dragging if active
/// 4. Handles viewport panning if active
///
/// # Arguments
/// * `event` - The mouse move event
/// * `canvas` - The canvas element
/// * `state` - Reference to application state
/// * `render` - Closure to trigger re-rendering
/// * `toolbar_state` - Reference to toolbar state
/// * `position_toolbar` - Closure to update toolbar position
#[cfg(target_arch = "wasm32")]
pub fn handle_mouse_move(
    event: MouseEvent,
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    render: &Rc<dyn Fn()>,
    toolbar_state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    position_toolbar: &Rc<dyn Fn()>,
) -> Result<(), crate::error::AppError> {
    let mouse_x = event.offset_x() as f64;
    let mouse_y = event.offset_y() as f64;

    // Update mouse position in app state
    state.borrow_mut().mouse_x = mouse_x;
    state.borrow_mut().mouse_y = mouse_y;

    // Handle toolbar dragging first
    let did_toolbar_move = toolbar_state
        .borrow_mut()
        .drag_to(event.client_x() as f64, event.client_y() as f64);
    if did_toolbar_move {
        position_toolbar();
        return Ok(());
    }

    // Handle sticky note dragging
    let did_note_move = {
        let viewport_width = f64::from(canvas.client_width().max(1));
        let viewport_height = f64::from(canvas.client_height().max(1));
        let world_pos = state.borrow().viewport.world_point_at(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
        );
        let sticky_notes = &mut state.borrow_mut().sticky_notes;
        if sticky_notes.is_dragging {
            sticky_notes.drag_to(world_pos.0, world_pos.1);
            true
        } else {
            false
        }
    };

    if did_note_move {
        render();
        return Ok(());
    }

    // Handle sticky note resizing
    let did_resize = {
        let viewport_width = f64::from(canvas.client_width().max(1));
        let viewport_height = f64::from(canvas.client_height().max(1));

        // Extract resizing state before mutable borrow
        let (
            is_resizing,
            note_id,
            handle,
            start_mouse_x,
            start_mouse_y,
            original_width,
            original_height,
        ) = {
            let resizing = &state.borrow().resizing;
            (
                resizing.is_resizing,
                resizing.note_id,
                resizing.handle,
                resizing.start_mouse_x,
                resizing.start_mouse_y,
                resizing.original_width,
                resizing.original_height,
            )
        };

        if is_resizing {
            if let (Some(_note_id), Some(handle)) = (note_id, handle) {
                // Extract viewport before mutable borrow
                let viewport = state.borrow().viewport.clone();
                state.borrow_mut().sticky_notes.resize_to(
                    handle,
                    start_mouse_x,
                    start_mouse_y,
                    mouse_x,
                    mouse_y,
                    original_width,
                    original_height,
                    &viewport,
                    viewport_width,
                    viewport_height,
                );
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if did_resize {
        render();
        return Ok(());
    }

    // Handle canvas dragging
    let did_move = { state.borrow_mut().viewport.drag_to(mouse_x, mouse_y) };

    if did_move {
        render();
    }

    // Update hovered resize handle (after all dragging logic)
    {
        let viewport_width = f64::from(canvas.client_width().max(1));
        let viewport_height = f64::from(canvas.client_height().max(1));
        let viewport = &state.borrow().viewport;
        let _hovered_handle = state.borrow().sticky_notes.find_resize_handle_at(
            mouse_x,
            mouse_y,
            viewport,
            viewport_width,
            viewport_height,
        );
        // state.borrow_mut().hovered_resize_handle = hovered_handle;
    }

    Ok(())
}

/// Handles mouse up events to complete drag operations.
///
/// This function terminates any active drag operations (viewport panning,
/// sticky note dragging, or toolbar dragging) and logs the completion.
/// It ensures that drag state is properly cleaned up and the final position
/// is rendered.
///
/// # Arguments
/// * `event` - The mouse up event (unused)
/// * `state` - Reference to application state
/// * `render` - Closure to trigger final re-rendering
/// * `toolbar_state` - Reference to toolbar state
/// * `position_toolbar` - Closure to update final toolbar position
#[cfg(target_arch = "wasm32")]
pub fn handle_mouse_up(
    _event: MouseEvent,
    state: &Rc<RefCell<crate::AppState>>,
    render: &Rc<dyn Fn()>,
    toolbar_state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    position_toolbar: &Rc<dyn Fn()>,
) -> Result<(), crate::error::AppError> {
    let was_dragging = state.borrow().viewport.is_dragging;
    let toolbar_was_dragging = toolbar_state.borrow().is_dragging;
    let sticky_note_was_dragging = state.borrow().sticky_notes.is_dragging;
    let sticky_note_was_resizing = state.borrow().resizing.is_resizing;
    end_drag_if_needed(&state, &render);
    end_toolbar_drag_if_needed(&toolbar_state, &position_toolbar);
    if was_dragging {
        crate::logging::log_info("Canvas drag ended");
    }
    if toolbar_was_dragging {
        crate::logging::log_info("Toolbar drag ended");
    }
    if sticky_note_was_dragging {
        crate::logging::log_info("Sticky note drag ended");
    }
    if sticky_note_was_resizing {
        crate::logging::log_info("Sticky note resize ended");
    }
    Ok(())
}

/// Handles mouse leave events to clean up drag operations when the mouse exits the document.
///
/// This function ensures that any active drag operations are properly terminated
/// when the mouse leaves the document area, preventing stuck drag states.
/// It's important for maintaining consistent application state during edge cases.
///
/// # Arguments
/// * `event` - The mouse leave event (unused)
/// * `state` - Reference to application state
/// * `render` - Closure to trigger re-rendering
/// * `toolbar_state` - Reference to toolbar state
/// * `position_toolbar` - Closure to update toolbar position
#[cfg(target_arch = "wasm32")]
pub fn handle_mouse_leave(
    _event: MouseEvent,
    state: &Rc<RefCell<crate::AppState>>,
    render: &Rc<dyn Fn()>,
    toolbar_state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    position_toolbar: &Rc<dyn Fn()>,
) -> Result<(), crate::error::AppError> {
    end_drag_if_needed(&state, &render);
    end_toolbar_drag_if_needed(&toolbar_state, &position_toolbar);
    Ok(())
}

/// Handles mouse wheel events for zoom functionality.
///
/// This function implements smooth zooming centered on the mouse cursor position.
/// It calculates the zoom factor based on wheel direction and applies it to the
/// viewport, maintaining the world point under the cursor stationary during zoom.
///
/// Zoom behavior:
/// - Wheel up (negative delta_y): Zoom in by ZOOM_STEP_FACTOR
/// - Wheel down (positive delta_y): Zoom out by 1/ZOOM_STEP_FACTOR
///
/// The zoom is centered on the current mouse position to provide intuitive
/// navigation experience.
///
/// # Arguments
/// * `event` - The wheel event
/// * `canvas` - Reference to the canvas element
/// * `state` - Reference to application state
/// * `render` - Closure to trigger re-rendering
#[cfg(target_arch = "wasm32")]
pub fn handle_wheel(
    event: WheelEvent,
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    render: &Rc<dyn Fn()>,
) -> Result<(), crate::error::AppError> {
    event.prevent_default();
    let factor = if event.delta_y() < 0.0 {
        crate::event_constants::ZOOM_STEP_FACTOR
    } else {
        1.0 / crate::event_constants::ZOOM_STEP_FACTOR
    };
    let viewport_width = f64::from(canvas.client_width().max(1));
    let viewport_height = f64::from(canvas.client_height().max(1));

    let old_zoom = state.borrow().viewport.zoom;
    state.borrow_mut().viewport.zoom_at(
        factor,
        event.offset_x() as f64,
        event.offset_y() as f64,
        viewport_width,
        viewport_height,
    );
    let new_zoom = state.borrow().viewport.zoom;
    crate::logging::log_info(&format!(
        "Zoom changed from {:.2} to {:.2} at ({}, {})",
        old_zoom,
        new_zoom,
        event.offset_x(),
        event.offset_y()
    ));
    render();
    Ok(())
}

/// Handles double-click events on the canvas to detect sticky note selection
/// and create text input overlay for editing.
///
/// This function checks if a double-click occurred on a sticky note and creates
/// a positioned input overlay for text editing. It prevents the default double-click
/// behavior to avoid text selection or other browser actions.
///
/// # Arguments
/// * `event` - The double-click event
/// * `canvas` - The canvas element
/// * `state` - Reference to application state for coordinate conversion
/// * `render` - Closure to trigger canvas re-rendering when content changes
#[cfg(target_arch = "wasm32")]
pub fn handle_double_click(
    event: MouseEvent,
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    render: &Rc<dyn Fn()>,
) -> Result<(), crate::error::AppError> {
    event.prevent_default();

    let mouse_x = event.offset_x() as f64;
    let mouse_y = event.offset_y() as f64;

    // Convert screen coordinates to world coordinates
    let viewport_width = f64::from(canvas.client_width().max(1));
    let viewport_height = f64::from(canvas.client_height().max(1));

    let world_pos =
        state
            .borrow()
            .viewport
            .world_point_at(mouse_x, mouse_y, viewport_width, viewport_height);

    // Check if double-click is on a sticky note
    if let Some(note_id) = state
        .borrow()
        .sticky_notes
        .find_note_at(world_pos.0, world_pos.1)
    {
        crate::logging::log_info(&format!(
            "Double-click detected on sticky note {} at world position ({:.1}, {:.1}) - creating input overlay",
            note_id, world_pos.0, world_pos.1
        ));

        // Create text input overlay for the selected note
        if let Err(e) = create_text_input_overlay(&canvas, &state, note_id, &render) {
            crate::logging::log_warn(&format!("Failed to create text input overlay: {:?}", e));
        }
    } else {
        crate::logging::log_info(&format!(
            "Double-click detected on canvas at world position ({:.1}, {:.1}) - no note selected",
            world_pos.0, world_pos.1
        ));
    }
    Ok(())
}

/// Handles mouse down events on the toolbar to initiate toolbar dragging.
///
/// This function allows users to reposition the floating toolbar by dragging
/// its handle. It checks that the event target is the toolbar handle element
/// and initiates the drag operation by updating the toolbar state.
///
/// The toolbar drag is independent of canvas interactions and allows users
/// to move the toolbar to their preferred position on screen.
///
/// # Arguments
/// * `event` - The mouse down event on the toolbar
/// * `canvas` - The canvas element for focus management
/// * `toolbar_state` - Reference to toolbar state for drag tracking
/// * `position_toolbar` - Closure to update toolbar position during drag
#[cfg(target_arch = "wasm32")]
pub fn handle_toolbar_mouse_down(
    event: MouseEvent,
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    toolbar_state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    position_toolbar: &Rc<dyn Fn()>,
    render: &Rc<dyn Fn()>,
) -> Result<(), crate::error::AppError> {
    if event.button() != 0 {
        return Ok(());
    }

    let Some(target) = event.target() else {
        return Ok(());
    };
    let Ok(target_element) = target.dyn_into::<HtmlElement>() else {
        return Ok(());
    };

    let target_id = target_element.id();
    if target_id == "floating-toolbar-handle" {
        // Handle toolbar drag - also clear selection
        event.prevent_default();
        event.stop_propagation();
        if let Err(error) = canvas.focus() {
            crate::logging::log_jsvalue_error("canvas focus failed", &error);
        }
        state.borrow_mut().sticky_notes.clear_selection();
        crate::logging::log_info("Selection cleared by toolbar handle click");
        render();
        toolbar_state
            .borrow_mut()
            .start_drag(event.client_x() as f64, event.client_y() as f64);
        position_toolbar();
        Ok(())
    } else if target_id == "floating-toolbar" {
        // Handle toolbar background click - clear selection
        event.prevent_default();
        event.stop_propagation();
        state.borrow_mut().sticky_notes.clear_selection();
        crate::logging::log_info("Selection cleared by toolbar background click");
        render();
        Ok(())
    } else {
        // Click on toolbar button or other element - let it bubble
        Ok(())
    }
}
