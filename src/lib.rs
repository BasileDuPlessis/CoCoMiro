//! # CoCoMiro - Infinite Canvas Application
//!
//! A WebAssembly-based infinite canvas application built with Rust, featuring:
//! - Smooth panning and zooming viewport controls
//! - Draggable sticky notes with text content
//! - Floating toolbar for quick actions
//! - Responsive design with keyboard and mouse support
//!
//! ## Architecture
//!
//! The application is structured as a modular WebAssembly module with the following components:
//!
//! - **Canvas Rendering**: Handles grid drawing, sticky note visualization, and viewport transformations
//! - **Event Handling**: Manages user interactions including mouse, keyboard, and touch events
//! - **State Management**: Maintains application state for viewport, sticky notes, and UI elements
//! - **WebAssembly Integration**: Provides the bridge between Rust logic and browser DOM
//!
//! ## Coordinate System
//!
//! The application uses a world-space coordinate system where:
//! - World coordinates are absolute positions in the infinite canvas
//! - Screen coordinates are relative to the viewport/canvas element
//! - Transformations between systems account for pan and zoom
//!
//! ## WebAssembly Considerations
//!
//! Code is conditionally compiled for WebAssembly targets using `#[cfg(target_arch = "wasm32")]`.
//! Host compilation is maintained for testing purposes.

#[cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, window};

pub mod app;
pub mod canvas;
pub mod error;
pub mod event_constants;
pub mod event_setup;
pub mod events;
pub mod keyboard_events;
pub mod logging;
pub mod mouse_events;
pub mod sticky_notes;
pub mod styling;
pub mod text_input;
pub mod toolbar;
pub mod viewport;

/// Application state containing all data for the infinite canvas.
///
/// This struct holds the complete state of the CoCoMiro application,
/// including viewport settings and sticky note data. It's used for
/// both testing and WebAssembly execution.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current viewport state (pan, zoom, drag status)
    pub viewport: viewport::ViewportState,
    /// All sticky notes in the canvas
    pub sticky_notes: sticky_notes::StickyNotesState,
    /// Current mouse position in screen coordinates
    pub mouse_x: f64,
    /// Current mouse position in screen coordinates
    pub mouse_y: f64,
    // /// Current resizing operation state
    pub resizing: sticky_notes::ResizingState,
    // /// Currently hovered resize handle (note_id, handle)
    // pub hovered_resize_handle: Option<(u32, sticky_notes::ResizeHandle)>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            viewport: viewport::ViewportState::default(),
            sticky_notes: sticky_notes::StickyNotesState::default(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            resizing: sticky_notes::ResizingState::default(),
            // hovered_resize_handle: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static APP_INITIALIZED: Cell<bool> = const { Cell::new(false) };
}

#[cfg(target_arch = "wasm32")]
/// Attempts to recover from canvas context errors by reinitializing the context.
///
/// This function provides recovery for canvas context loss or corruption by
/// attempting to recreate the rendering context and resize the canvas.
///
/// # Arguments
/// * `canvas` - The HTML canvas element that may have lost its context
/// * `context` - Reference to the current context (may be invalid)
///
/// # Returns
/// * `Ok(CanvasRenderingContext2d)` - Successfully recovered context
/// * `Err(AppError)` - Recovery failed
fn recover_canvas_context(
    canvas: &HtmlCanvasElement,
    _context: &CanvasRenderingContext2d,
) -> crate::error::AppResult<CanvasRenderingContext2d> {
    logging::log_warn("Attempting canvas context recovery...");

    // Try to get a new 2D context
    let new_context = canvas
        .get_context("2d")
        .map_err(|_| {
            crate::error::AppError::Canvas("failed to get 2d context during recovery".to_string())
        })?
        .ok_or_else(|| {
            crate::error::AppError::Canvas(
                "could not access canvas context during recovery".to_string(),
            )
        })?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| {
            crate::error::AppError::Canvas(
                "context is not a 2D rendering context during recovery".to_string(),
            )
        })?;

    // Resize the canvas to ensure it's properly configured
    canvas::resize_canvas(canvas, &new_context)?;

    logging::log_info("Canvas context recovery successful");
    Ok(new_context)
}

#[cfg(target_arch = "wasm32")]
/// Fallback rendering function that displays a basic error state when normal rendering fails.
///
/// This function provides graceful degradation by showing a minimal canvas state
/// with an error message, ensuring the application remains somewhat functional
/// even when rendering encounters critical failures.
///
/// # Arguments
/// * `ctx` - The 2D canvas rendering context
/// * `canvas` - The HTML canvas element
/// * `status` - The status text element to update
/// * `error` - The error that caused the fallback rendering
///
/// # Returns
/// * `Ok(())` - Fallback rendering completed
/// * `Err(AppError)` - Even fallback rendering failed
fn fallback_render(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    status: &HtmlElement,
    error: &crate::error::AppError,
) -> crate::error::AppResult<()> {
    // Get canvas dimensions
    let (width, height) = canvas::canvas_css_size(canvas)?;

    // Clear canvas with error color
    ctx.set_fill_style_str("#fee2e2"); // Light red background
    ctx.fill_rect(0.0, 0.0, width, height);

    // Draw error message
    ctx.set_fill_style_str("#dc2626"); // Dark red text
    ctx.set_font("16px Inter, sans-serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");

    let error_msg = "Rendering Error - Please refresh the page";
    ctx.fill_text(error_msg, width / 2.0, height / 2.0)?;

    // Update status with error information
    status.set_text_content(Some(&format!("Error: {} · Refresh page to recover", error)));

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn setup_canvas_and_context(
    canvas: &web_sys::HtmlCanvasElement,
) -> crate::error::AppResult<CanvasRenderingContext2d> {
    let context = canvas
        .get_context("2d")
        .map_err(|_| crate::error::AppError::Canvas("failed to get 2d context".to_string()))?
        .ok_or_else(|| {
            crate::error::AppError::Canvas("could not access the canvas context".to_string())
        })?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| {
            crate::error::AppError::Canvas("context is not a 2D rendering context".to_string())
        })?;
    canvas::resize_canvas(canvas, &context)?;

    Ok(context)
}

#[cfg(target_arch = "wasm32")]
fn create_render_and_position_functions(
    canvas: &web_sys::HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    status: &web_sys::HtmlElement,
    workspace: &web_sys::HtmlElement,
    toolbar: &web_sys::HtmlElement,
    state: &Rc<RefCell<AppState>>,
    toolbar_state: &Rc<RefCell<toolbar::FloatingToolbarState>>,
    is_rendering: &Rc<Cell<bool>>,
) -> (Rc<dyn Fn()>, Rc<dyn Fn()>) {
    let render: Rc<dyn Fn()> = Rc::new({
        let context = context.clone();
        let canvas = canvas.clone();
        let status = status.clone();
        let state = state.clone();
        let is_rendering = is_rendering.clone();
        move || {
            if is_rendering.replace(true) {
                logging::log_info("Render skipped: already rendering");
                return;
            }

            let snapshot = state.borrow().clone();

            // Attempt rendering with retry logic
            let mut retry_count = 0;
            const MAX_RETRIES: u32 = 3;

            while retry_count < MAX_RETRIES {
                match canvas::render_canvas(&context, &canvas, &status, &snapshot) {
                    Ok(()) => break, // Success
                    Err(error) => {
                        retry_count += 1;
                        if retry_count >= MAX_RETRIES {
                            logging::log_app_error(
                                &error,
                                "rendering canvas (final attempt failed)",
                            );

                            // Attempt canvas context recovery before fallback
                            match recover_canvas_context(&canvas, &context) {
                                Ok(_new_context) => {
                                    logging::log_info(
                                        "Context recovery successful, retrying render",
                                    );
                                    // Since we can't update the closure's context reference directly,
                                    // we'll try one more render with the potentially recovered context
                                    if let Err(final_error) =
                                        canvas::render_canvas(&context, &canvas, &status, &snapshot)
                                    {
                                        logging::log_app_error(
                                            &final_error,
                                            "rendering after context recovery",
                                        );
                                        if let Err(fallback_error) = fallback_render(
                                            &context,
                                            &canvas,
                                            &status,
                                            &final_error,
                                        ) {
                                            logging::log_app_error(
                                                &fallback_error,
                                                "fallback rendering after recovery",
                                            );
                                        }
                                    }
                                }
                                Err(recovery_error) => {
                                    logging::log_app_error(
                                        &recovery_error,
                                        "canvas context recovery",
                                    );
                                    if let Err(fallback_error) =
                                        fallback_render(&context, &canvas, &status, &error)
                                    {
                                        logging::log_app_error(
                                            &fallback_error,
                                            "fallback rendering",
                                        );
                                    }
                                }
                            }
                        } else {
                            logging::log_warn(&format!(
                                "Render attempt {} failed, retrying: {}",
                                retry_count, error
                            ));
                            // Small delay before retry (in a real implementation, you'd use setTimeout)
                        }
                    }
                }
            }

            is_rendering.set(false);
        }
    });
    let position_toolbar: Rc<dyn Fn()> = Rc::new({
        let workspace = workspace.clone();
        let toolbar = toolbar.clone();
        let toolbar_state = toolbar_state.clone();
        move || {
            if let Err(error) =
                canvas::sync_toolbar_position(&toolbar, &workspace, &mut toolbar_state.borrow_mut())
            {
                logging::log_app_error(&error, "positioning toolbar");
            }
        }
    });

    (render, position_toolbar)
}

#[cfg(target_arch = "wasm32")]
fn setup_event_system(
    canvas: &web_sys::HtmlCanvasElement,
    workspace: &web_sys::HtmlElement,
    toolbar: &web_sys::HtmlElement,
    state: &Rc<RefCell<AppState>>,
    toolbar_state: &Rc<RefCell<toolbar::FloatingToolbarState>>,
    render: &Rc<dyn Fn()>,
    position_toolbar: &Rc<dyn Fn()>,
) -> crate::error::AppResult<()> {
    event_setup::setup_event_listeners(
        canvas,
        workspace,
        toolbar,
        state,
        toolbar_state,
        render,
        position_toolbar,
    )
}

#[cfg(target_arch = "wasm32")]
fn setup_window_resize_handler(
    browser_window: &web_sys::Window,
    canvas: &web_sys::HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    render: &Rc<dyn Fn()>,
    position_toolbar: &Rc<dyn Fn()>,
) -> crate::error::AppResult<()> {
    let on_resize = Closure::<dyn FnMut()>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        let render = render.clone();
        let position_toolbar = position_toolbar.clone();
        move || {
            if let Err(error) = canvas::resize_canvas(&canvas, &context) {
                logging::log_app_error(&error, "resizing canvas");
            }
            render();
            position_toolbar();
        }
    }));
    browser_window
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())?;
    on_resize.forget();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[cfg(target_arch = "wasm32")]
fn start_impl() -> crate::error::AppResult<()> {
    let browser_window = window()
        .ok_or_else(|| crate::error::AppError::BrowserEnv("window is unavailable".to_string()))?;
    let document = browser_window.document().ok_or_else(|| {
        crate::error::AppError::BrowserEnv("could not access the browser document".to_string())
    })?;

    let (workspace, canvas, status, toolbar) = app::install_app(&document)?;

    let context = setup_canvas_and_context(&canvas)?;

    let state = Rc::new(RefCell::new(AppState::default()));
    let toolbar_state = Rc::new(RefCell::new(toolbar::FloatingToolbarState::default()));
    let is_rendering = Rc::new(Cell::new(false));

    let (render, position_toolbar) = create_render_and_position_functions(
        &canvas,
        &context,
        &status,
        &workspace,
        &toolbar,
        &state,
        &toolbar_state,
        &is_rendering,
    );

    render();
    position_toolbar();

    setup_event_system(
        &canvas,
        &workspace,
        &toolbar,
        &state,
        &toolbar_state,
        &render,
        &position_toolbar,
    )?;

    setup_window_resize_handler(
        &browser_window,
        &canvas,
        &context,
        &render,
        &position_toolbar,
    )?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
/// WebAssembly entry point for the CoCoMiro application.
///
/// This function is automatically called when the WebAssembly module loads.
/// It initializes the application by:
/// - Setting up the DOM structure
/// - Creating the canvas and rendering context
/// - Initializing application state
/// - Setting up event listeners for user interaction
/// - Starting the render loop
///
/// The function uses a thread-local flag to prevent multiple initializations
/// which could occur if the module is loaded multiple times.
///
/// # Returns
/// * `Ok(())` - Application initialized successfully
/// * `Err(JsValue)` - Initialization failed with JavaScript error
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    if APP_INITIALIZED.with(|flag| flag.replace(true)) {
        return Ok(());
    }

    if let Err(error) = start_impl() {
        APP_INITIALIZED.with(|flag| flag.set(false));
        return Err(JsValue::from(error));
    }

    Ok(())
}
