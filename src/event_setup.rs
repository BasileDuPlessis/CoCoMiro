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

/// Shared context for event handlers to reduce complex borrow patterns.
///
/// This struct bundles all the shared state and functions that event handlers need,
/// reducing the number of individual Rc<RefCell<>> clones and simplifying closure captures.
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct EventContext {
    /// Main application state
    state: Rc<RefCell<crate::AppState>>,
    /// Toolbar state
    toolbar_state: Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
    /// Canvas re-rendering function
    render: Rc<dyn Fn()>,
    /// Toolbar positioning function
    position_toolbar: Rc<dyn Fn()>,
}

#[cfg(target_arch = "wasm32")]
impl EventContext {
    /// Creates a new event context with the provided shared resources.
    fn new(
        state: Rc<RefCell<crate::AppState>>,
        toolbar_state: Rc<RefCell<crate::toolbar::FloatingToolbarState>>,
        render: Rc<dyn Fn()>,
        position_toolbar: Rc<dyn Fn()>,
    ) -> Self {
        Self {
            state,
            toolbar_state,
            render,
            position_toolbar,
        }
    }

    /// Handles mouse down events using the shared context.
    fn handle_mouse_down(
        &self,
        event: web_sys::MouseEvent,
        canvas: &HtmlCanvasElement,
    ) -> crate::error::AppResult<()> {
        crate::mouse_events::handle_mouse_down(event, canvas, &self.state, &self.render)
    }

    /// Handles mouse move events using the shared context.
    fn handle_mouse_move(
        &self,
        event: web_sys::MouseEvent,
        canvas: &HtmlCanvasElement,
    ) -> crate::error::AppResult<()> {
        crate::mouse_events::handle_mouse_move(
            event,
            canvas,
            &self.state,
            &self.render,
            &self.toolbar_state,
            &self.position_toolbar,
        )
    }

    /// Handles mouse up events using the shared context.
    fn handle_mouse_up(&self, event: web_sys::MouseEvent) -> crate::error::AppResult<()> {
        crate::mouse_events::handle_mouse_up(
            event,
            &self.state,
            &self.render,
            &self.toolbar_state,
            &self.position_toolbar,
        )
    }

    /// Handles mouse leave events using the shared context.
    fn handle_mouse_leave(&self, event: web_sys::MouseEvent) -> crate::error::AppResult<()> {
        crate::mouse_events::handle_mouse_leave(
            event,
            &self.state,
            &self.render,
            &self.toolbar_state,
            &self.position_toolbar,
        )
    }

    /// Handles wheel events using the shared context.
    fn handle_wheel(
        &self,
        event: web_sys::WheelEvent,
        canvas: &HtmlCanvasElement,
    ) -> crate::error::AppResult<()> {
        crate::mouse_events::handle_wheel(event, canvas, &self.state, &self.render)
    }

    /// Handles key down events using the shared context.
    fn handle_key_down(
        &self,
        event: web_sys::KeyboardEvent,
        canvas: &HtmlCanvasElement,
    ) -> crate::error::AppResult<()> {
        crate::keyboard_events::handle_key_down(event, canvas, &self.state, &self.render)
    }

    /// Handles double click events using the shared context.
    fn handle_double_click(
        &self,
        event: web_sys::MouseEvent,
        canvas: &HtmlCanvasElement,
    ) -> crate::error::AppResult<()> {
        crate::mouse_events::handle_double_click(event, canvas, &self.state, &self.render)
    }

    /// Handles toolbar mouse down events using the shared context.
    fn handle_toolbar_mouse_down(
        &self,
        event: web_sys::MouseEvent,
        canvas: &HtmlCanvasElement,
    ) -> crate::error::AppResult<()> {
        crate::mouse_events::handle_toolbar_mouse_down(
            event,
            canvas,
            &self.state,
            &self.toolbar_state,
            &self.position_toolbar,
            &self.render,
        )
    }

    /// Handles add note button clicks using the shared context.
    fn handle_add_note_click(&self, canvas: &HtmlCanvasElement) {
        let viewport_width = f64::from(canvas.client_width().max(1));
        let viewport_height = f64::from(canvas.client_height().max(1));

        let viewport = self.state.borrow().viewport.clone();
        self.state
            .borrow_mut()
            .sticky_notes
            .add_note_at_viewport_center(viewport_width, viewport_height, &viewport);

        (self.render)();
        crate::logging::log_info("Added new sticky note");
    }

    /// Handles window blur events using the shared context.
    fn handle_window_blur(&self) {
        end_drag_if_needed(&self.state, &self.render);
        end_toolbar_drag_if_needed(&self.toolbar_state, &self.position_toolbar);
    }
}

/// Converts a JsValue error to an AppError with context.
///
/// # Arguments
/// * `js_error` - The JavaScript error to convert
/// * `context` - Descriptive context about where the error occurred
///
/// # Returns
/// An AppError with the provided context
#[cfg(target_arch = "wasm32")]
pub fn js_error_to_app_error(
    js_error: wasm_bindgen::JsValue,
    context: &str,
) -> crate::error::AppError {
    let message = js_error
        .as_string()
        .unwrap_or_else(|| format!("{js_error:?}"));
    crate::error::AppError::Event(format!("{context}: {message}"))
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
        if app_state.resizing.is_resizing {
            app_state.sticky_notes.end_resize();
            app_state.resizing = crate::sticky_notes::ResizingState::default();
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

/// Sets up mouse event listeners for canvas interactions
#[cfg(target_arch = "wasm32")]
fn setup_mouse_event_listeners(
    canvas: &HtmlCanvasElement,
    browser_window: &web_sys::Window,
    document: &web_sys::Document,
    context: &EventContext,
) -> crate::error::AppResult<()> {
    // Mouse down on canvas
    let on_mouse_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = context.handle_mouse_down(event, &canvas) {
                crate::logging::log_app_error(&error, "handling mouse down");
            }
        }
    }));
    canvas
        .add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach mousedown listener to canvas"))?;
    on_mouse_down.forget();

    // Mouse move
    let on_mouse_move = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = context.handle_mouse_move(event, &canvas) {
                crate::logging::log_app_error(&error, "handling mouse move");
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())?;
    on_mouse_move.forget();

    // Mouse up
    let on_mouse_up = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let context = context.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = context.handle_mouse_up(event) {
                crate::logging::log_app_error(&error, "handling mouse up");
            }
        }
    }));
    browser_window
        .add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach mouseup listener to window"))?;
    on_mouse_up.forget();

    // Mouse leave document
    let on_mouse_leave = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let context = context.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = context.handle_mouse_leave(event) {
                crate::logging::log_app_error(&error, "handling mouse leave");
            }
        }
    }));
    document
        .add_event_listener_with_callback("mouseleave", on_mouse_leave.as_ref().unchecked_ref())
        .map_err(|e| {
            js_error_to_app_error(e, "failed to attach mouseleave listener to document")
        })?;
    on_mouse_leave.forget();

    Ok(())
}

/// Sets up toolbar event listeners
#[cfg(target_arch = "wasm32")]
fn setup_toolbar_event_listeners(
    canvas: &HtmlCanvasElement,
    toolbar: &HtmlElement,
    context: &EventContext,
) -> crate::error::AppResult<()> {
    // Mouse down on toolbar handle
    let on_toolbar_mouse_down = Closure::<dyn FnMut(web_sys::MouseEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = context.handle_toolbar_mouse_down(event, &canvas) {
                crate::logging::log_app_error(&error, "handling toolbar mouse down");
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

    Ok(())
}

/// Sets up button event listeners
#[cfg(target_arch = "wasm32")]
fn setup_button_event_listeners(
    document: &web_sys::Document,
    canvas: &HtmlCanvasElement,
    context: &EventContext,
) -> crate::error::AppResult<()> {
    // Click on add note button
    let on_add_note_click = Closure::<dyn FnMut()>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        move || {
            context.handle_add_note_click(&canvas);
        }
    }));
    let add_note_button = document
        .get_element_by_id("add-note-button")
        .ok_or_else(|| {
            crate::error::AppError::Dom("add note button element not found".to_string())
        })?
        .dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| {
            crate::error::AppError::Dom("add note button is not an HTML element".to_string())
        })?;
    add_note_button
        .add_event_listener_with_callback("click", on_add_note_click.as_ref().unchecked_ref())
        .map_err(|e| {
            js_error_to_app_error(e, "failed to attach click listener to add note button")
        })?;
    on_add_note_click.forget();

    Ok(())
}

/// Sets up keyboard and wheel event listeners
#[cfg(target_arch = "wasm32")]
fn setup_keyboard_and_wheel_listeners(
    canvas: &HtmlCanvasElement,
    _browser_window: &web_sys::Window,
    context: &EventContext,
) -> crate::error::AppResult<()> {
    // Wheel
    let on_wheel = Closure::<dyn FnMut(web_sys::WheelEvent)>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        move |event: web_sys::WheelEvent| {
            if let Err(error) = context.handle_wheel(event, &canvas) {
                crate::logging::log_app_error(&error, "handling wheel");
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
        let context = context.clone();
        move |event: web_sys::KeyboardEvent| {
            if let Err(error) = context.handle_key_down(event, &canvas) {
                crate::logging::log_app_error(&error, "handling key down");
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
        let context = context.clone();
        move |event: web_sys::MouseEvent| {
            if let Err(error) = context.handle_double_click(event, &canvas) {
                crate::logging::log_app_error(&error, "handling double click");
            }
        }
    }));
    canvas
        .add_event_listener_with_callback("dblclick", on_double_click.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach dblclick listener to canvas"))?;
    on_double_click.forget();

    Ok(())
}

/// Sets up cleanup event listeners for drag operations
#[cfg(target_arch = "wasm32")]
fn setup_cleanup_listeners(
    browser_window: &web_sys::Window,
    context: &EventContext,
) -> crate::error::AppResult<()> {
    // Blur window
    let on_blur = Closure::<dyn FnMut()>::wrap(Box::new({
        let context = context.clone();
        move || {
            context.handle_window_blur();
        }
    }));
    browser_window
        .add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())
        .map_err(|e| js_error_to_app_error(e, "failed to attach blur listener to window"))?;
    on_blur.forget();

    Ok(())
}
/// Sets up all event listeners for the CoCoMiro application.
///
/// This function creates a shared event context and delegates to specialized
/// setup functions for different types of event listeners. The shared context
/// reduces complex borrow patterns by bundling all shared state together.
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
) -> crate::error::AppResult<()> {
    let browser_window = web_sys::window()
        .ok_or_else(|| crate::error::AppError::BrowserEnv("window is unavailable".to_string()))?;
    let document = browser_window.document().ok_or_else(|| {
        crate::error::AppError::BrowserEnv("could not access the browser document".to_string())
    })?;

    // Create shared event context to reduce complex borrow patterns
    let context = EventContext::new(
        state.clone(),
        toolbar_state.clone(),
        render.clone(),
        position_toolbar.clone(),
    );

    // Set up mouse event listeners
    setup_mouse_event_listeners(canvas, &browser_window, &document, &context)?;

    // Set up toolbar event listeners
    setup_toolbar_event_listeners(canvas, toolbar, &context)?;

    // Set up button event listeners
    setup_button_event_listeners(&document, canvas, &context)?;

    // Set up keyboard and wheel event listeners
    setup_keyboard_and_wheel_listeners(canvas, &browser_window, &context)?;

    // Set up cleanup event listeners
    setup_cleanup_listeners(&browser_window, &context)?;

    Ok(())
}
