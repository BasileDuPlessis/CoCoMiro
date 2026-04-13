//! # Event Setup and Shared Utilities
//!
//! This module contains the main event setup function and shared utility functions
//! used throughout the event handling system.

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlCanvasElement, HtmlElement};

/// Converts a JsValue error to an AppError with context.
///
/// # Arguments
/// * `js_error` - The JavaScript error to convert
/// * `context` - Descriptive context about where the error occurred
///
/// # Returns
/// An AppError with the provided context
#[cfg(target_arch = "wasm32")]
pub fn js_error_to_app_error(js_error: wasm_bindgen::JsValue, context: &str) -> crate::AppError {
    let message = js_error
        .as_string()
        .unwrap_or_else(|| format!("{js_error:?}"));
    crate::AppError::Event(format!("{context}: {message}"))
}

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
#[cfg(target_arch = "wasm32")]
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

/// Terminates any active toolbar drag operations.
///
/// This helper function ends toolbar dragging when cleanup is needed,
/// such as when the mouse leaves the document or the window loses focus.
/// It ensures toolbar position is updated after ending the drag.
///
/// # Arguments
/// * `state` - Reference to the toolbar state
/// * `position_toolbar` - Closure to update toolbar position after ending drag
#[cfg(target_arch = "wasm32")]
pub fn end_toolbar_drag_if_needed(
    state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    position_toolbar: &Rc<dyn Fn()>,
) {
    if state.borrow().is_dragging {
        state.borrow_mut().end_drag();
        position_toolbar();
    }
}

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
#[cfg(target_arch = "wasm32")]
pub fn setup_event_listeners(
    canvas: &HtmlCanvasElement,
    _workspace: &HtmlElement,
    toolbar: &HtmlElement,
    state: &Rc<RefCell<crate::AppState>>,
    toolbar_state: &Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    render: &Rc<dyn Fn()>,
    position_toolbar: &Rc<dyn Fn()>,
) -> crate::AppResult<()> {
    let browser_window = web_sys::window()
        .ok_or_else(|| crate::AppError::BrowserEnv("window is unavailable".to_string()))?;
    let document = browser_window.document().ok_or_else(|| {
        crate::AppError::BrowserEnv("could not access the browser document".to_string())
    })?;

    // Mouse down on canvas
    let on_mouse_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) =
                crate::mouse_events::handle_mouse_down(event, &canvas, &state, &render)
            {
                crate::log_app_error(&error, "handling mouse down");
            }
        }
    }));
    canvas
        .add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach mousedown listener to canvas"))?;
    on_mouse_down.forget();

    // Mouse down on toolbar handle
    let on_toolbar_mouse_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = crate::mouse_events::handle_toolbar_mouse_down(
                event,
                &canvas,
                &toolbar_state,
                &position_toolbar,
            ) {
                crate::log_app_error(&error, "handling toolbar mouse down");
            }
        }
    }));
    toolbar
        .add_event_listener_with_callback(
            "mousedown",
            on_toolbar_mouse_down.as_ref().unchecked_ref(),
        )
        .map_err(|e| js_error_to_app_error(e, "failed to attach mousedown listener to toolbar"))?;
    on_toolbar_mouse_down.forget();

    // Click on add note button
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
        .map_err(|e| {
            js_error_to_app_error(e, "failed to attach click listener to add note button")
        })?;
    on_add_note_click.forget();

    // Mouse move
    let on_mouse_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = crate::mouse_events::handle_mouse_move(
                event,
                &canvas,
                &state,
                &render,
                &toolbar_state,
                &position_toolbar,
            ) {
                crate::log_app_error(&error, "handling mouse move");
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())?;
    on_mouse_move.forget();

    // Mouse up
    let on_mouse_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = crate::mouse_events::handle_mouse_up(
                event,
                &state,
                &render,
                &toolbar_state,
                &position_toolbar,
            ) {
                crate::log_app_error(&error, "handling mouse up");
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach mouseup listener to window"))?;
    on_mouse_up.forget();

    // Mouse leave document
    let on_mouse_leave = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let toolbar_state = toolbar_state.clone();
        let position_toolbar = position_toolbar.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = crate::mouse_events::handle_mouse_leave(
                event,
                &state,
                &render,
                &toolbar_state,
                &position_toolbar,
            ) {
                crate::log_app_error(&error, "handling mouse leave");
            }
        }
    }));
    document
        .add_event_listener_with_callback("mouseleave", on_mouse_leave.as_ref().unchecked_ref())
        .map_err(|e| {
            js_error_to_app_error(e, "failed to attach mouseleave listener to document")
        })?;
    on_mouse_leave.forget();

    // Blur window
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
    browser_window
        .add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach blur listener to window"))?;
    on_blur.forget();

    // Wheel
    let on_wheel = Closure::<dyn FnMut(web_sys::WheelEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: web_sys::WheelEvent| {
            if let Err(error) = crate::mouse_events::handle_wheel(event, &canvas, &state, &render) {
                crate::log_app_error(&error, "handling wheel");
            }
        }
    }));
    canvas
        .add_event_listener_with_callback("wheel", on_wheel.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach wheel listener to canvas"))?;
    on_wheel.forget();

    // Key down
    let on_key_down = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: web_sys::KeyboardEvent| {
            if let Err(error) =
                crate::keyboard_events::handle_key_down(event, &canvas, &state, &render)
            {
                crate::log_app_error(&error, "handling key down");
            }
        }
    }));
    canvas
        .add_event_listener_with_callback("keydown", on_key_down.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach keydown listener to canvas"))?;
    on_key_down.forget();

    // Double-click detection for text editing
    let on_double_click = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let state = state.clone();
        let render = render.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) =
                crate::mouse_events::handle_double_click(event, &canvas, &state, &render)
            {
                crate::log_app_error(&error, "handling double click");
            }
        }
    }));
    canvas
        .add_event_listener_with_callback("dblclick", on_double_click.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach dblclick listener to canvas"))?;
    on_double_click.forget();

    Ok(())
}
