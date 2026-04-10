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
use web_sys::{CanvasRenderingContext2d, window};

/// Custom error types for the CoCoMiro application.
///
/// This enum provides specific error types for different failure scenarios,
/// allowing for better error handling and recovery strategies.
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    /// Browser environment errors (missing window, document, etc.)
    BrowserEnv(String),
    /// Canvas-related errors (context creation, rendering failures)
    Canvas(String),
    /// DOM manipulation errors (element access, property setting)
    Dom(String),
    /// Event handling errors (listener attachment failures)
    Event(String),
    /// State management errors (invalid state transitions)
    State(String),
    /// Rendering errors (drawing failures, context issues)
    Render(String),
    /// Generic application errors
    Generic(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BrowserEnv(msg) => write!(f, "Browser environment error: {}", msg),
            AppError::Canvas(msg) => write!(f, "Canvas error: {}", msg),
            AppError::Dom(msg) => write!(f, "DOM error: {}", msg),
            AppError::Event(msg) => write!(f, "Event error: {}", msg),
            AppError::State(msg) => write!(f, "State error: {}", msg),
            AppError::Render(msg) => write!(f, "Render error: {}", msg),
            AppError::Generic(msg) => write!(f, "Application error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

#[cfg(target_arch = "wasm32")]
impl From<AppError> for JsValue {
    fn from(error: AppError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
/// Result type alias for WebAssembly operations that may fail.
pub type AppResult<T> = Result<T, AppError>;

pub mod app;
pub mod canvas;
pub mod events;
pub mod sticky_notes;
pub mod toolbar;
pub mod viewport;

#[cfg(any(test, target_arch = "wasm32"))]
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
}

#[cfg(any(test, target_arch = "wasm32"))]
impl Default for AppState {
    fn default() -> Self {
        Self {
            viewport: viewport::ViewportState::default(),
            sticky_notes: sticky_notes::StickyNotesState::default(),
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static APP_INITIALIZED: Cell<bool> = const { Cell::new(false) };
}

#[cfg(target_arch = "wasm32")]
/// Logs JavaScript errors to the browser console with enhanced context.
///
/// This function provides consistent error logging for WebAssembly code,
/// ensuring errors are visible in browser developer tools with additional context.
///
/// # Arguments
/// * `context` - Descriptive context for where the error occurred
/// * `error` - The error to log (can be AppError or JsValue)
pub fn log_js_error(context: &str, error: &impl std::fmt::Display) {
    web_sys::console::error_1(&JsValue::from_str(&format!("CoCoMiro [{context}]: {error}")));
}

#[cfg(target_arch = "wasm32")]
/// Logs application errors with recovery suggestions.
///
/// This function logs errors and provides user-friendly recovery information
/// when possible. For critical errors, it may suggest page refresh.
///
/// # Arguments
/// * `error` - The application error to log
/// * `operation` - Description of the operation that failed
pub fn log_app_error(error: &AppError, operation: &str) {
    let recovery_hint = match error {
        AppError::BrowserEnv(_) => "Try refreshing the page or checking your browser compatibility.",
        AppError::Canvas(_) => "Try resizing the window or refreshing the page.",
        AppError::Dom(_) => "The page may have been modified. Try refreshing.",
        AppError::Event(_) => "Interaction may be limited. Try refreshing the page.",
        AppError::State(_) => "Application state may be corrupted. Try refreshing the page.",
        AppError::Render(_) => "Rendering failed. Try refreshing the page.",
        AppError::Generic(_) => "An unexpected error occurred. Try refreshing the page.",
    };

    web_sys::console::error_1(&JsValue::from_str(&format!(
        "CoCoMiro Error during '{}': {}\nRecovery: {}",
        operation, error, recovery_hint
    )));
}

#[cfg(target_arch = "wasm32")]
/// Logs informational messages to the browser console.
///
/// # Arguments
/// * `message` - The message to log
pub fn log_info(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

#[cfg(target_arch = "wasm32")]
/// Logs warning messages to the browser console.
///
/// # Arguments
/// * `message` - The warning message to log
pub fn log_warn(message: &str) {
    web_sys::console::warn_1(&JsValue::from_str(message));
}

#[cfg(target_arch = "wasm32")]
fn start_impl() -> AppResult<()> {
    let browser_window = window().ok_or_else(|| AppError::BrowserEnv("window is unavailable".to_string()))?;
    let document = browser_window
        .document()
        .ok_or_else(|| AppError::BrowserEnv("could not access the browser document".to_string()))?;

    let (workspace, canvas, status, toolbar) = app::install_app(&document)?;

    let context = canvas
        .get_context("2d")
        .map_err(|_| AppError::Canvas("failed to get 2d context".to_string()))?
        .ok_or_else(|| AppError::Canvas("could not access the canvas context".to_string()))?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| AppError::Canvas("context is not a 2D rendering context".to_string()))?;
    canvas::resize_canvas(&canvas, &context)?;

    let state = Rc::new(RefCell::new(AppState::default()));
    let toolbar_state = Rc::new(RefCell::new(toolbar::FloatingToolbarState::default()));
    let is_rendering = Rc::new(Cell::new(false));
    let render: Rc<dyn Fn()> = Rc::new({
        let context = context.clone();
        let canvas = canvas.clone();
        let status = status.clone();
        let state = state.clone();
        let is_rendering = is_rendering.clone();
        move || {
            if is_rendering.replace(true) {
                log_info("Render skipped: already rendering");
                return;
            }

            let snapshot = state.borrow().clone();
            if let Err(error) = canvas::render_canvas(&context, &canvas, &status, &snapshot) {
                log_app_error(&error, "rendering canvas");
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
                log_app_error(&error, "positioning toolbar");
            }
        }
    });
    render();
    position_toolbar();

    events::setup_event_listeners(
        &canvas,
        &workspace,
        &toolbar,
        &state,
        &toolbar_state,
        &render,
        &position_toolbar,
    )?;

    let on_resize = Closure::<dyn FnMut()>::wrap(Box::new({
        let canvas = canvas.clone();
        let context = context.clone();
        let render = render.clone();
        let position_toolbar = position_toolbar.clone();
        move || {
            if let Err(error) = canvas::resize_canvas(&canvas, &context) {
                log_app_error(&error, "resizing canvas");
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
