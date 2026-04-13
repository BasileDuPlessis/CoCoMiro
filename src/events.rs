//! # Event Handling System
//!
//! This module manages all user interactions for the CoCoMiro application.
//! It handles mouse, keyboard, and window events to provide a responsive
//! user experience across different input methods.
//!
//! ## Event Types
//!
//! The system handles these event categories:
//! - **Mouse Events**: Canvas panning, sticky note interaction, toolbar dragging
//! - **Keyboard Events**: Viewport navigation, sticky note deletion, zoom controls
//! - **Wheel Events**: Smooth zooming with cursor anchoring
//! - **Window Events**: Resize handling, focus management, drag cleanup
//!
//! ## Interaction Priority
//!
//! Events are processed with this priority order:
//! 1. Toolbar dragging (highest priority)
//! 2. Sticky note interactions (selection, dragging)
//! 3. Canvas panning (default behavior)
//!
//! ## Coordinate Systems
//!
//! The module handles conversions between:
//! - Screen coordinates (mouse events)
//! - Canvas coordinates (relative to canvas element)
//! - World coordinates (absolute positions in infinite space)
//!
//! ## State Management
//!
//! Event handlers update application state and trigger rendering.
//! Drag operations use offset tracking to maintain smooth interaction.

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent, WheelEvent, window};

#[cfg(target_arch = "wasm32")]
/// Zoom factor applied per wheel event (1.1 = 10% zoom per step)
const ZOOM_STEP_FACTOR: f64 = 1.1;
#[cfg(target_arch = "wasm32")]
/// Distance moved per keyboard pan event (in screen pixels)
const KEYBOARD_PAN_STEP: f64 = 40.0;
#[cfg(target_arch = "wasm32")]
/// Converts a JsValue error to an AppError with context.
///
/// # Arguments
/// * `js_error` - The JavaScript error to convert
/// * `context` - Descriptive context about where the error occurred
///
/// # Returns
/// An AppError with the provided context
fn js_error_to_app_error(js_error: JsValue, context: &str) -> crate::AppError {
    let message = js_error.as_string().unwrap_or_else(|| format!("{js_error:?}"));
    crate::AppError::Event(format!("{context}: {message}"))
}

#[cfg(target_arch = "wasm32")]
/// Terminates any active drag operations for viewport and sticky notes.
///
/// This helper function is called when drag operations should be forcibly ended,
/// such as when the mouse leaves the document or the window loses focus.
/// It ensures that drag state is properly cleaned up and triggers a re-render
/// if any drag operations were active.
///
/// # Arguments
/// * `state` - Reference to the application state containing viewport and sticky notes
/// * `render` - Closure to trigger canvas re-rendering after ending drags
pub fn end_drag_if_needed(state: &Rc<RefCell<crate::AppState>>, render: &Rc<dyn Fn()>) {
    let mut should_render = false;
    {
        let mut app_state = state.borrow_mut();
        if app_state.viewport.is_dragging {
            app_state.viewport.end_drag();
            should_render = true;
        }
        if app_state.sticky_notes.is_dragging {
            app_state.sticky_notes.end_drag();
            should_render = true;
        }
    }
    if should_render {
        render();
    }
}

#[cfg(target_arch = "wasm32")]
/// Terminates any active toolbar drag operations.
///
/// This helper function ends toolbar dragging when cleanup is needed,
/// such as when the mouse leaves the document or the window loses focus.
/// It ensures toolbar position is updated after ending the drag.
///
/// # Arguments
/// * `state` - Reference to the toolbar state
/// * `position_toolbar` - Closure to update toolbar position after ending drag
pub fn end_toolbar_drag_if_needed(
    state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    position_toolbar: &Rc<dyn Fn()>,
) {
    if state.borrow().is_dragging {
        state.borrow_mut().end_drag();
        position_toolbar();
    }
}

#[cfg(target_arch = "wasm32")]
/// Creates a text input overlay positioned over a sticky note for editing.
///
/// This function creates an HTML input element that overlays the specified sticky note,
/// allowing the user to edit the note's text content. The input is styled to match the
/// note's appearance and includes event handling for text input.
///
/// # Arguments
/// * `canvas` - The canvas element for coordinate calculations
/// * `state` - Reference to application state containing the note
/// * `note_id` - ID of the note to create input overlay for
/// * `render` - Closure to trigger canvas re-rendering when content changes
fn create_text_input_overlay(
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
) {
    let browser_window = match window() {
        Some(w) => w,
        None => {
            crate::log_warn("Cannot create text input overlay: window unavailable");
            return;
        }
    };

    let document = match browser_window.document() {
        Some(d) => d,
        None => {
            crate::log_warn("Cannot create text input overlay: document unavailable");
            return;
        }
    };

    // Get note details
    let note = match state.borrow().sticky_notes.notes.iter().find(|n| n.id == note_id) {
        Some(n) => n.clone(),
        None => {
            crate::log_warn(&format!("Cannot create input overlay for note {}: note not found", note_id));
            return;
        }
    };

    // Calculate screen position from world coordinates
    let viewport_width = f64::from(canvas.client_width().max(1));
    let viewport_height = f64::from(canvas.client_height().max(1));
    let zoom = state.borrow().viewport.zoom;
    let pan_x = state.borrow().viewport.pan_x;
    let pan_y = state.borrow().viewport.pan_y;

    let screen_x = note.x * zoom + viewport_width / 2.0 + pan_x;
    let screen_y = note.y * zoom + viewport_height / 2.0 + pan_y;
    let screen_width = note.width * zoom;
    let screen_height = note.height * zoom;

    // Get canvas position relative to the document using getBoundingClientRect
    let canvas_js: &JsValue = canvas.as_ref();
    let rect_js = js_sys::Reflect::get(canvas_js, &"getBoundingClientRect".into())
        .unwrap()
        .dyn_into::<js_sys::Function>()
        .unwrap()
        .call0(canvas_js)
        .unwrap();
    
    let canvas_left = js_sys::Reflect::get(&rect_js, &"left".into())
        .unwrap()
        .as_f64()
        .unwrap();
    let canvas_top = js_sys::Reflect::get(&rect_js, &"top".into())
        .unwrap()
        .as_f64()
        .unwrap();

    // Calculate document-relative position for the overlay
    // Position textarea to align with canvas text (which starts 8px from top)
    let overlay_left = canvas_left + screen_x;
    let overlay_top = canvas_top + screen_y;

    // Create textarea element for multiline text editing
    let textarea = match document.create_element("textarea") {
        Ok(el) => el,
        Err(_) => {
            crate::log_warn("Cannot create text textarea element");
            return;
        }
    };

    let textarea: web_sys::HtmlTextAreaElement = match textarea.dyn_into() {
        Ok(ta) => ta,
        Err(_) => {
            crate::log_warn("Cannot convert element to textarea");
            return;
        }
    };

    // Style the textarea to match the note
    let _ = textarea.style().set_property("position", "absolute");
    let _ = textarea.style().set_property("left", &format!("{}px", overlay_left));
    let _ = textarea.style().set_property("top", &format!("{}px", overlay_top));
    let _ = textarea.style().set_property("width", &format!("{}px", screen_width));
    let _ = textarea.style().set_property("height", &format!("{}px", screen_height));
    let _ = textarea.style().set_property("font-size", "14px");
    let _ = textarea.style().set_property("font-family", "Inter, sans-serif");
    let _ = textarea.style().set_property("border", "2px solid #2563eb");
    let _ = textarea.style().set_property("border-radius", "4px");
    let _ = textarea.style().set_property("padding", "8px");
    let _ = textarea.style().set_property("background-color", &note.color);
    let _ = textarea.style().set_property("color", "#000000");
    let _ = textarea.style().set_property("outline", "none");
    let _ = textarea.style().set_property("z-index", "1000");
    let _ = textarea.style().set_property("text-align", "left");
    let _ = textarea.style().set_property("box-sizing", "border-box");
    let _ = textarea.style().set_property("resize", "none");
    let _ = textarea.style().set_property("overflow", "hidden");

    // Set initial value and focus
    textarea.set_value(&note.content);
    let _ = textarea.focus();
    let _ = textarea.select();

    // Attach input event listener to handle text changes
    let on_input = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let note_id = note_id;
        let textarea = textarea.clone();
        move |event: web_sys::Event| {
            event.stop_propagation();
            
            // Update the note content with the current textarea value
            if let Some(note) = state.borrow_mut().sticky_notes.get_note_mut(note_id) {
                note.content = textarea.value();
            }
            
            // Re-render the canvas to show the updated text
            render();
        }
    }));
    textarea.add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::log_warn("Failed to attach input event listener");
        });
    on_input.forget();

    // Add to document
    if let Some(body) = document.body() {
        let _ = body.append_child(&textarea);
    }

    crate::log_info(&format!("Created text input overlay for note {}", note_id));
}

#[cfg(target_arch = "wasm32")]
/// Sets up all event listeners for the CoCoMiro application.
///
/// This function establishes comprehensive event handling by:
/// 1. Creating closures for each event type with proper state capture
/// 2. Attaching event listeners to DOM elements and window
/// 3. Configuring event propagation and default behavior prevention
/// 4. Setting up cleanup handlers for drag operations
///
/// Event listeners are attached to:
/// - Canvas: mouse, wheel, keyboard events
/// - Toolbar: mouse events for dragging
/// - Window: resize, blur, mouse events for drag cleanup
/// - Add button: click events for note creation
///
/// # Arguments
/// * `canvas` - The main canvas element for drawing interactions
/// * `workspace` - The workspace container element
/// * `toolbar` - The floating toolbar element
/// * `state` - Reference to the main application state
/// * `toolbar_state` - Reference to the toolbar state
/// * `render` - Closure to trigger canvas re-rendering
/// * `position_toolbar` - Closure to update toolbar position
///
/// # Returns
/// * `Ok(())` - All event listeners set up successfully
/// * `Err(AppError)` - Failed to set up event listeners
pub fn setup_event_listeners(
    canvas: &HtmlCanvasElement,
    _workspace: &HtmlElement,
    toolbar: &HtmlElement,
    state: &Rc<RefCell<crate::AppState>>,
    toolbar_state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    render: &Rc<dyn Fn()>,
    position_toolbar: &Rc<dyn Fn()>,
) -> crate::AppResult<()> {
    let browser_window = window().ok_or_else(|| crate::AppError::BrowserEnv("window is unavailable".to_string()))?;
    let document = browser_window
        .document()
        .ok_or_else(|| crate::AppError::BrowserEnv("could not access the browser document".to_string()))?;

    // Mouse down on canvas
    // Handles mouse down events on the canvas to initiate drag operations.
    //
    // This function determines the type of interaction based on:
    // 1. Which mouse button was pressed (left for panning/dragging, right for context menu)
    // 2. Whether the target element is a sticky note or the canvas background
    // 3. Current application state (tool selection, existing drag state)
    //
    // For sticky notes, it initiates note dragging. For canvas background,
    // it starts viewport panning. The function prevents default browser behavior
    // and captures the mouse to ensure smooth drag operations.
    //
    // Arguments:
    // * event - The mouse down event
    // * state - Reference to application state
    // * render - Closure to trigger re-rendering
    let on_mouse_down = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: MouseEvent| {
            if event.button() != 0 {
                return;
            }

            event.prevent_default();
            if let Err(error) = canvas.focus() {
                crate::log_jsvalue_error("canvas focus failed", &error);
            }

            let mouse_x = event.offset_x() as f64;
            let mouse_y = event.offset_y() as f64;

            // Check for sticky note first
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
                crate::log_info(&format!("Sticky note {} drag started", note_id));
                render();
                return;
            }

            // If no sticky note hit, start canvas drag
            state.borrow_mut().viewport.start_drag(mouse_x, mouse_y);
            crate::log_info(&format!(
                "Canvas drag started at ({}, {})",
                mouse_x, mouse_y
            ));
            render();
        }
    }));
    canvas.add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach mousedown listener to canvas"))?;
    on_mouse_down.forget();

    // Mouse down on toolbar handle
    // Handles mouse down events on the toolbar to initiate toolbar dragging.
    //
    // This function allows users to reposition the floating toolbar by dragging
    // its handle. It checks that the event target is the toolbar handle element
    // and initiates the drag operation by updating the toolbar state.
    //
    // The toolbar drag is independent of canvas interactions and allows users
    // to move the toolbar to their preferred position on screen.
    //
    // Arguments:
    // * event - The mouse down event on the toolbar
    // * toolbar_state - Reference to toolbar state for drag tracking
    // * position_toolbar - Closure to update toolbar position during drag
    let on_toolbar_mouse_down = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |event: MouseEvent| {
            if event.button() != 0 {
                return;
            }

            let Some(target) = event.target() else {
                return;
            };
            let Ok(target_element) = target.dyn_into::<HtmlElement>() else {
                return;
            };
            if target_element.id() != "floating-toolbar-handle" {
                return;
            }

            event.prevent_default();
            event.stop_propagation();
            if let Err(error) = canvas.focus() {
                crate::log_jsvalue_error("canvas focus failed", &error);
            }
            toolbar_state
                .borrow_mut()
                .start_drag(event.client_x() as f64, event.client_y() as f64);
            position_toolbar();
        }
    }));
    toolbar.add_event_listener_with_callback(
        "mousedown",
        on_toolbar_mouse_down.as_ref().unchecked_ref(),
    ).map_err(|e| js_error_to_app_error(e, "failed to attach mousedown listener to toolbar"))?;
    on_toolbar_mouse_down.forget();

    // Click on add note button
    // Handles click events on the "Add Note" button to create new sticky notes.
    // Creates a new sticky note positioned at the center of the current viewport.
    let on_add_note_click = Closure::<dyn FnMut()>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move || {
            let viewport_width = f64::from(canvas.client_width().max(1));
            let viewport_height = f64::from(canvas.client_height().max(1));

            let viewport = state.borrow().viewport.clone();
            state.borrow_mut().sticky_notes.add_note_at_viewport_center(
                viewport_width,
                viewport_height,
                &viewport,
            );

            render();
            crate::log_info("Added new sticky note");
        }
    }));
    let add_note_button = document
        .get_element_by_id("add-note-button")
        .ok_or_else(|| crate::AppError::Dom("add note button element not found".to_string()))?
        .dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| crate::AppError::Dom("add note button is not an HTML element".to_string()))?;
    add_note_button
        .add_event_listener_with_callback("click", on_add_note_click.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach click listener to add note button"))?;
    on_add_note_click.forget();

    // Mouse move
    // Handles mouse move events for continuous drag operations and hover effects.
    //
    // This function updates the mouse position in the application state and handles
    // ongoing drag operations for both sticky notes and the viewport. It prioritizes
    // toolbar dragging over canvas interactions.
    //
    // The function performs the following operations in order:
    // 1. Updates current mouse coordinates in app state
    // 2. Handles toolbar dragging if active
    // 3. Handles sticky note dragging if active
    // 4. Handles viewport panning if active
    //
    // Arguments:
    // * event - The mouse move event
    // * state - Reference to application state
    // * render - Closure to trigger re-rendering
    // * toolbar_state - Reference to toolbar state
    // * position_toolbar - Closure to update toolbar position
    let on_mouse_move = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |event: MouseEvent| {
            let mouse_x = event.offset_x() as f64;
            let mouse_y = event.offset_y() as f64;

            // Update mouse position in app state
            state.borrow_mut().mouse_x = mouse_x;
            state.borrow_mut().mouse_y = mouse_y;

            // Handle toolbar dragging first
            let did_toolbar_move = toolbar_state.borrow_mut().drag_to(event.client_x() as f64, event.client_y() as f64);
            if did_toolbar_move {
                position_toolbar();
                return;
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
                return;
            }

            // Handle canvas dragging
            let did_move = { state.borrow_mut().viewport.drag_to(mouse_x, mouse_y) };

            if did_move {
                render();
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())?;
    on_mouse_move.forget();

    // Mouse up
    // Handles mouse up events to complete drag operations.
    //
    // This function terminates any active drag operations (viewport panning,
    // sticky note dragging, or toolbar dragging) and logs the completion.
    // It ensures that drag state is properly cleaned up and the final position
    // is rendered.
    //
    // Arguments:
    // * event - The mouse up event (unused)
    // * state - Reference to application state
    // * render - Closure to trigger final re-rendering
    // * toolbar_state - Reference to toolbar state
    // * position_toolbar - Closure to update final toolbar position
    let on_mouse_up = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |_event: MouseEvent| {
            let was_dragging = state.borrow().viewport.is_dragging;
            let toolbar_was_dragging = toolbar_state.borrow().is_dragging;
            let sticky_note_was_dragging = state.borrow().sticky_notes.is_dragging;
            end_drag_if_needed(&state, &render);
            end_toolbar_drag_if_needed(&toolbar_state, &position_toolbar);
            if was_dragging {
                crate::log_info("Canvas drag ended");
            }
            if toolbar_was_dragging {
                crate::log_info("Toolbar drag ended");
            }
            if sticky_note_was_dragging {
                crate::log_info("Sticky note drag ended");
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach mouseup listener to window"))?;
    on_mouse_up.forget();

    // Mouse leave document
    // Handles mouse leave events to clean up drag operations when the mouse exits the document.
    //
    // This function ensures that any active drag operations are properly terminated
    // when the mouse leaves the document area, preventing stuck drag states.
    // It's important for maintaining consistent application state during edge cases.
    //
    // Arguments:
    // * event - The mouse leave event (unused)
    // * state - Reference to application state
    // * render - Closure to trigger re-rendering
    // * toolbar_state - Reference to toolbar state
    // * position_toolbar - Closure to update toolbar position
    let on_mouse_leave = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |_event: MouseEvent| {
            end_drag_if_needed(&state, &render);
            end_toolbar_drag_if_needed(&toolbar_state, &position_toolbar);
        }
    }));
    document
        .add_event_listener_with_callback("mouseleave", on_mouse_leave.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach mouseleave listener to document"))?;
    on_mouse_leave.forget();

    // Blur window
    // Handles window blur events to clean up drag operations when the window loses focus.
    //
    // This function ensures that any active drag operations are properly terminated
    // when the browser window loses focus, preventing stuck drag states that could
    // occur if the user switches tabs or applications during a drag operation.
    //
    // Arguments:
    // * state - Reference to application state
    // * render - Closure to trigger re-rendering
    // * toolbar_state - Reference to toolbar state
    // * position_toolbar - Closure to update toolbar position
    let on_blur = Closure::<dyn FnMut()>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move || {
            end_drag_if_needed(&state, &render);
            end_toolbar_drag_if_needed(&toolbar_state, &position_toolbar);
        }
    }));
    browser_window.add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach blur listener to window"))?;
    on_blur.forget();

    // Wheel
    // Handles mouse wheel events for zoom functionality.
    //
    // This function implements smooth zooming centered on the mouse cursor position.
    // It calculates the zoom factor based on wheel direction and applies it to the
    // viewport, maintaining the world point under the cursor stationary during zoom.
    //
    // Zoom behavior:
    // - Wheel up (negative delta_y): Zoom in by ZOOM_STEP_FACTOR
    // - Wheel down (positive delta_y): Zoom out by 1/ZOOM_STEP_FACTOR
    //
    // The zoom is centered on the current mouse position to provide intuitive
    // navigation experience.
    //
    // Arguments:
    // * event - The wheel event
    // * canvas - Reference to the canvas element
    // * state - Reference to application state
    // * render - Closure to trigger re-rendering
    let on_wheel = Closure::<dyn FnMut(WheelEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: WheelEvent| {
            event.prevent_default();
            let factor = if event.delta_y() < 0.0 {
                ZOOM_STEP_FACTOR
            } else {
                1.0 / ZOOM_STEP_FACTOR
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
            crate::log_info(&format!(
                "Zoom changed from {:.2} to {:.2} at ({}, {})",
                old_zoom,
                new_zoom,
                event.offset_x(),
                event.offset_y()
            ));
            render();
        }
    }));
    canvas.add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach wheel listener to canvas"))?;
    on_wheel.forget();

    // Key down
    // Handles keyboard events for viewport navigation and note management.
    //
    // This function provides keyboard shortcuts for common operations:
    // - Arrow keys: Pan the viewport in the respective direction
    // - +/-: Zoom in/out centered on viewport center
    // - 0/Home: Reset viewport to default position and zoom
    // - Delete/Backspace: Delete the currently selected sticky note
    //
    // All handled keys prevent default browser behavior to avoid conflicts
    // with page scrolling or other browser shortcuts.
    //
    // # Arguments
    // * `event` - The keyboard event
    // * `canvas` - Reference to the canvas element
    // * `state` - Reference to application state
    // * `render` - Closure to trigger re-rendering
    let on_key_down = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: KeyboardEvent| {
            let viewport_width = f64::from(canvas.client_width().max(1));
            let viewport_height = f64::from(canvas.client_height().max(1));
            let viewport = &mut state.borrow_mut().viewport;

            let handled = match event.key().as_str() {
                "ArrowLeft" => {
                    viewport.pan_by(-KEYBOARD_PAN_STEP, 0.0);
                    crate::log_info(&format!("Panned left by {}", KEYBOARD_PAN_STEP));
                    true
                }
                "ArrowRight" => {
                    viewport.pan_by(KEYBOARD_PAN_STEP, 0.0);
                    crate::log_info(&format!("Panned right by {}", KEYBOARD_PAN_STEP));
                    true
                }
                "ArrowUp" => {
                    viewport.pan_by(0.0, -KEYBOARD_PAN_STEP);
                    crate::log_info(&format!("Panned up by {}", KEYBOARD_PAN_STEP));
                    true
                }
                "ArrowDown" => {
                    viewport.pan_by(0.0, KEYBOARD_PAN_STEP);
                    crate::log_info(&format!("Panned down by {}", KEYBOARD_PAN_STEP));
                    true
                }
                "+" | "=" => {
                    let old_zoom = viewport.zoom;
                    viewport.zoom_at(
                        ZOOM_STEP_FACTOR,
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
                        1.0 / ZOOM_STEP_FACTOR,
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
        }
    }));
    canvas.add_event_listener_with_callback("keydown", on_key_down.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach keydown listener to canvas"))?;
    on_key_down.forget();

    // Double-click detection for text editing (Task 1.1.1 & 1.1.2)
    // Handles double-click events on the canvas to detect sticky note selection
    // and create text input overlay for editing.
    //
    // This function checks if a double-click occurred on a sticky note and creates
    // a positioned input overlay for text editing. It prevents the default double-click
    // behavior to avoid text selection or other browser actions.
    //
    // Arguments:
    // * event - The double-click event
    // * state - Reference to application state for coordinate conversion
    // * canvas - Reference to canvas element for viewport dimensions
    let on_double_click = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let canvas = canvas.clone();
        let render = render.clone();
        move |event: MouseEvent| {
            event.prevent_default();

            let mouse_x = event.offset_x() as f64;
            let mouse_y = event.offset_y() as f64;

            // Convert screen coordinates to world coordinates
            let viewport_width = f64::from(canvas.client_width().max(1));
            let viewport_height = f64::from(canvas.client_height().max(1));

            let world_pos = state.borrow().viewport.world_point_at(
                mouse_x,
                mouse_y,
                viewport_width,
                viewport_height,
            );

            // Check if double-click is on a sticky note
            if let Some(note_id) = state
                .borrow()
                .sticky_notes
                .find_note_at(world_pos.0, world_pos.1)
            {
                crate::log_info(&format!(
                    "Double-click detected on sticky note {} at world position ({:.1}, {:.1}) - creating input overlay",
                    note_id, world_pos.0, world_pos.1
                ));

                // Create text input overlay for the selected note
                create_text_input_overlay(&canvas, &state, note_id, &render);
            } else {
                crate::log_info(&format!(
                    "Double-click detected on canvas at world position ({:.1}, {:.1}) - no note selected",
                    world_pos.0, world_pos.1
                ));
            }
        }
    }));
    canvas.add_event_listener_with_callback("dblclick", on_double_click.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach dblclick listener to canvas"))?;
    on_double_click.forget();

    Ok(())
}
