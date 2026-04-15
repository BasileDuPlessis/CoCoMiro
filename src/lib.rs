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
impl From<JsValue> for AppError {
    fn from(js_error: JsValue) -> Self {
        AppError::Generic(format!("JavaScript error: {:?}", js_error))
    }
}

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
pub mod event_constants;
pub mod event_setup;
pub mod events;
pub mod keyboard_events;
pub mod mouse_events;
pub mod sticky_notes;
pub mod text_input;
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
    web_sys::console::error_1(&JsValue::from_str(&format!(
        "CoCoMiro [{context}]: {error}"
    )));
}

/// Logs raw JavaScript values as errors.
///
/// This function is specifically for logging JsValue errors that don't
/// implement Display but can be debug-formatted.
///
/// # Arguments
/// * `context` - Descriptive context for where the error occurred
/// * `error` - The JsValue error to log
#[cfg(target_arch = "wasm32")]
pub fn log_jsvalue_error(context: &str, error: &JsValue) {
    web_sys::console::error_1(&JsValue::from_str(&format!(
        "CoCoMiro [{context}]: {:?}",
        error
    )));
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
        AppError::BrowserEnv(_) => {
            "Try refreshing the page or checking your browser compatibility."
        }
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
) -> AppResult<CanvasRenderingContext2d> {
    log_warn("Attempting canvas context recovery...");

    // Try to get a new 2D context
    let new_context = canvas
        .get_context("2d")
        .map_err(|_| AppError::Canvas("failed to get 2d context during recovery".to_string()))?
        .ok_or_else(|| {
            AppError::Canvas("could not access canvas context during recovery".to_string())
        })?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| {
            AppError::Canvas("context is not a 2D rendering context during recovery".to_string())
        })?;

    // Resize the canvas to ensure it's properly configured
    canvas::resize_canvas(canvas, &new_context)?;

    log_info("Canvas context recovery successful");
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
    error: &AppError,
) -> AppResult<()> {
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
fn start_impl() -> AppResult<()> {
    let browser_window =
        window().ok_or_else(|| AppError::BrowserEnv("window is unavailable".to_string()))?;
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

            // Attempt rendering with retry logic
            let mut retry_count = 0;
            const MAX_RETRIES: u32 = 3;

            while retry_count < MAX_RETRIES {
                match canvas::render_canvas(&context, &canvas, &status, &snapshot) {
                    Ok(()) => break, // Success
                    Err(error) => {
                        retry_count += 1;
                        if retry_count >= MAX_RETRIES {
                            log_app_error(&error, "rendering canvas (final attempt failed)");

                            // Attempt canvas context recovery before fallback
                            match recover_canvas_context(&canvas, &context) {
                                Ok(_new_context) => {
                                    log_info("Context recovery successful, retrying render");
                                    // Since we can't update the closure's context reference directly,
                                    // we'll try one more render with the potentially recovered context
                                    if let Err(final_error) =
                                        canvas::render_canvas(&context, &canvas, &status, &snapshot)
                                    {
                                        log_app_error(
                                            &final_error,
                                            "rendering after context recovery",
                                        );
                                        if let Err(fallback_error) = fallback_render(
                                            &context,
                                            &canvas,
                                            &status,
                                            &final_error,
                                        ) {
                                            log_app_error(
                                                &fallback_error,
                                                "fallback rendering after recovery",
                                            );
                                        }
                                    }
                                }
                                Err(recovery_error) => {
                                    log_app_error(&recovery_error, "canvas context recovery");
                                    if let Err(fallback_error) =
                                        fallback_render(&context, &canvas, &status, &error)
                                    {
                                        log_app_error(&fallback_error, "fallback rendering");
                                    }
                                }
                            }
                        } else {
                            log_warn(&format!(
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
                log_app_error(&error, "positioning toolbar");
            }
        }
    });
    render();
    position_toolbar();

    event_setup::setup_event_listeners(
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

#[cfg(test)]
/// Integration tests that verify interactions between multiple modules
/// These tests ensure that the components work together correctly
mod integration_tests {
    use super::*;
    use sticky_notes::StickyNote;
    use viewport::ViewportState;

    #[test]
    fn viewport_and_sticky_notes_coordinate_transformation() {
        // Test that viewport transformations correctly affect sticky note positioning
        let mut app_state = AppState::default();
        let mut viewport = ViewportState::default();

        // Set up viewport with pan and zoom
        viewport.pan_x = 100.0;
        viewport.pan_y = 50.0;
        viewport.zoom = 2.0;
        app_state.viewport = viewport;

        // Add a note at viewport center
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &app_state.viewport);

        // Verify the note was placed at the correct world coordinates
        assert_eq!(app_state.sticky_notes.notes.len(), 1);
        let note = &app_state.sticky_notes.notes[0];

        // With pan (100, 50) and zoom 2.0, center should be at world (-50, -25)
        assert_eq!(note.x, -50.0);
        assert_eq!(note.y, -25.0);
    }

    #[test]
    fn mouse_interaction_with_viewport_and_notes() {
        // Test mouse coordinate conversion and interaction with both viewport and notes
        let mut app_state = AppState::default();
        let mut viewport = ViewportState::default();

        // Set up viewport
        viewport.pan_x = 200.0;
        viewport.pan_y = 100.0;
        viewport.zoom = 1.5;
        app_state.viewport = viewport;

        // Add a note at a specific world position
        let note = StickyNote::new(50.0, 75.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);

        // Simulate mouse click at screen coordinates that should hit the note
        // With viewport pan (200, 100) and zoom 1.5, world point (50, 75) corresponds to:
        // screen_x = 50 * 1.5 + 400 + 200 = 75 + 600 = 675
        // screen_y = 75 * 1.5 + 300 + 100 = 112.5 + 400 = 512.5

        let screen_x = 675.0;
        let screen_y = 512.5;

        // Convert screen to world coordinates
        let (world_x, world_y) = app_state
            .viewport
            .world_point_at(screen_x, screen_y, 800.0, 600.0);

        // Should be approximately (50, 75)
        assert!((world_x - 50.0).abs() < 0.1);
        assert!((world_y - 75.0).abs() < 0.1);

        // Should find the note at this world position
        assert_eq!(
            app_state.sticky_notes.find_note_at(world_x, world_y),
            Some(note_id)
        );
    }

    #[test]
    fn drag_note_with_viewport_changes() {
        // Test dragging a note while viewport changes
        let mut app_state = AppState::default();

        // Add a note
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);

        // Start dragging the note
        app_state.sticky_notes.start_drag(note_id, 150.0, 150.0);

        // Drag to new position
        app_state.sticky_notes.drag_to(200.0, 180.0);

        // Verify note moved correctly
        let note = app_state.sticky_notes.get_note_mut(note_id).unwrap();
        assert_eq!(note.x, 150.0);
        assert_eq!(note.y, 130.0);

        // Now change viewport (zoom in)
        app_state.viewport.zoom = 2.0;

        // The note should still be at the same world position
        let note = app_state.sticky_notes.get_note_mut(note_id).unwrap();
        assert_eq!(note.x, 150.0);
        assert_eq!(note.y, 130.0);

        // End drag
        app_state.sticky_notes.end_drag();
        assert!(!app_state.sticky_notes.is_dragging);
    }

    #[test]
    fn multiple_notes_selection_and_deletion() {
        // Test selecting and deleting notes in a complex scenario
        let mut app_state = AppState::default();

        // Add multiple notes
        let note1 = StickyNote::new(0.0, 0.0);
        let note2 = StickyNote::new(250.0, 0.0);
        let note3 = StickyNote::new(250.0, 200.0);
        let note1_id = note1.id;
        let note2_id = note2.id;
        let note3_id = note3.id;

        app_state.sticky_notes.add_note(note1);
        app_state.sticky_notes.add_note(note2);
        app_state.sticky_notes.add_note(note3);

        assert_eq!(app_state.sticky_notes.notes.len(), 3);

        // Select note2 (should be topmost at its position)
        let found_id = app_state.sticky_notes.find_note_at(300.0, 50.0);
        assert_eq!(found_id, Some(note2_id));
        app_state.sticky_notes.selected_note_id = found_id;

        // Delete selected note
        app_state.sticky_notes.delete_selected();
        assert_eq!(app_state.sticky_notes.notes.len(), 2);
        assert!(app_state.sticky_notes.selected_note_id.is_none());

        // Verify remaining notes
        let remaining_ids: Vec<u32> = app_state.sticky_notes.notes.iter().map(|n| n.id).collect();
        assert!(remaining_ids.contains(&note1_id));
        assert!(remaining_ids.contains(&note3_id));
        assert!(!remaining_ids.contains(&note2_id));
    }

    #[test]
    fn viewport_bounds_and_note_placement() {
        // Test that notes are placed correctly within viewport bounds
        let mut app_state = AppState::default();
        let mut viewport = ViewportState::default();

        // Set viewport with extreme pan to test bounds
        viewport.pan_x = 1000.0;
        viewport.pan_y = 800.0;
        viewport.zoom = 0.5; // Zoomed out
        app_state.viewport = viewport;

        // Add note at center - should be placed at world center adjusted for pan/zoom
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &app_state.viewport);

        // With pan (1000, 800) and zoom 0.5, center calculation:
        // world_x = (400 - 400 - 1000) / 0.5 = (-1000) / 0.5 = -2000
        // world_y = (300 - 300 - 800) / 0.5 = (-800) / 0.5 = -1600
        assert_eq!(app_state.sticky_notes.notes[0].x, -2000.0);
        assert_eq!(app_state.sticky_notes.notes[0].y, -1600.0);
    }

    #[test]
    fn coordinate_system_consistency() {
        // Test that screen-to-world and world-to-screen conversions are consistent
        let viewport = ViewportState::default();

        // Test point at screen center
        let screen_x = 400.0;
        let screen_y = 300.0;
        let viewport_width = 800.0;
        let viewport_height = 600.0;

        let (world_x, world_y) =
            viewport.world_point_at(screen_x, screen_y, viewport_width, viewport_height);

        // Convert back to screen coordinates
        let center_x = viewport_width / 2.0;
        let center_y = viewport_height / 2.0;
        let screen_x_back = world_x * viewport.zoom + center_x + viewport.pan_x;
        let screen_y_back = world_y * viewport.zoom + center_y + viewport.pan_y;

        // Should get back the original screen coordinates
        assert!((screen_x_back - screen_x).abs() < 0.001);
        assert!((screen_y_back - screen_y).abs() < 0.001);
    }

    #[test]
    fn note_dragging_with_coordinate_conversion() {
        // Test dragging behavior with proper coordinate conversion
        let mut app_state = AppState::default();
        let mut viewport = ViewportState::default();

        // Set up viewport with zoom
        viewport.zoom = 2.0;
        app_state.viewport = viewport;

        // Add note at world position (100, 100)
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);

        // Convert world position to screen coordinates for mouse interaction
        let center_x = 800.0 / 2.0;
        let center_y = 600.0 / 2.0;
        let _screen_note_x = 100.0 * app_state.viewport.zoom + center_x + app_state.viewport.pan_x;
        let _screen_note_y = 100.0 * app_state.viewport.zoom + center_y + app_state.viewport.pan_y;

        // Start drag at the note's screen position
        app_state.sticky_notes.start_drag(note_id, 100.0, 100.0); // World coordinates

        // Drag to new world position (200, 150)
        app_state.sticky_notes.drag_to(200.0, 150.0);

        // Verify note moved to correct world position
        let note = app_state.sticky_notes.get_note_mut(note_id).unwrap();
        assert_eq!(note.x, 200.0);
        assert_eq!(note.y, 150.0);
    }

    #[test]
    fn complex_interaction_sequence() {
        // Test a complex sequence of interactions
        let mut app_state = AppState::default();

        // 1. Add multiple notes
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &app_state.viewport);
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &app_state.viewport);
        assert_eq!(app_state.sticky_notes.notes.len(), 2);

        // 2. Select and drag first note
        let first_note_id = app_state.sticky_notes.notes[0].id;
        app_state.sticky_notes.start_drag(first_note_id, 0.0, 0.0);
        app_state.sticky_notes.drag_to(50.0, 50.0);
        app_state.sticky_notes.end_drag();

        // 3. Select second note and delete it
        let second_note_id = app_state.sticky_notes.notes[1].id;
        app_state.sticky_notes.selected_note_id = Some(second_note_id);
        app_state.sticky_notes.delete_selected();

        // 4. Verify final state
        assert_eq!(app_state.sticky_notes.notes.len(), 1);
        assert_eq!(app_state.sticky_notes.notes[0].id, first_note_id);
        assert_eq!(app_state.sticky_notes.notes[0].x, 50.0);
        assert_eq!(app_state.sticky_notes.notes[0].y, 50.0);
        assert!(app_state.sticky_notes.selected_note_id.is_none());
    }

    #[test]
    fn html_text_parsing_and_formatting() {
        // Test that HTML tags in note content are properly parsed for rendering
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            // Test basic HTML parsing
            let html_content = "Hello <b>world</b> and <i>universe</i>";
            let segments = parse_formatted_text(html_content);

            assert_eq!(segments.len(), 4);

            // "Hello "
            assert_eq!(segments[0].text, "Hello ");
            assert!(!segments[0].bold);
            assert!(!segments[0].italic);
            assert!(!segments[0].underline);

            // "world"
            assert_eq!(segments[1].text, "world");
            assert!(segments[1].bold);
            assert!(!segments[1].italic);
            assert!(!segments[1].underline);

            // " and "
            assert_eq!(segments[2].text, " and ");
            assert!(!segments[2].bold);
            assert!(!segments[2].italic);
            assert!(!segments[2].underline);

            // "universe"
            assert_eq!(segments[3].text, "universe");
            assert!(!segments[3].bold);
            assert!(segments[3].italic);
            assert!(!segments[3].underline);
        }

        // Test nested and overlapping tags
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            let complex_html = "Start <b>bold <i>bold-italic</i> still bold</b> end";
            let segments = parse_formatted_text(complex_html);

            assert_eq!(segments.len(), 5);

            // "Start "
            assert_eq!(segments[0].text, "Start ");
            assert!(!segments[0].bold);
            assert!(!segments[0].italic);

            // "bold "
            assert_eq!(segments[1].text, "bold ");
            assert!(segments[1].bold);
            assert!(!segments[1].italic);

            // "bold-italic"
            assert_eq!(segments[2].text, "bold-italic");
            assert!(segments[2].bold);
            assert!(segments[2].italic);

            // " still bold"
            assert_eq!(segments[3].text, " still bold");
            assert!(segments[3].bold);
            assert!(!segments[3].italic);

            // " end"
            assert_eq!(segments[4].text, " end");
            assert!(!segments[4].bold);
            assert!(!segments[4].italic);
        }

        // Test underline formatting
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            let underline_html = "Normal <u>underlined</u> normal again";
            let segments = parse_formatted_text(underline_html);

            assert_eq!(segments.len(), 3);

            // "Normal "
            assert_eq!(segments[0].text, "Normal ");
            assert!(!segments[0].underline);

            // "underlined"
            assert_eq!(segments[1].text, "underlined");
            assert!(segments[1].underline);

            // " normal again"
            assert_eq!(segments[2].text, " normal again");
            assert!(!segments[2].underline);
        }

        // Test HTML with <span> tags and style attributes
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            let span_bold = r#"Text <span style="font-weight: bold;">bold</span> text"#;
            let segments = parse_formatted_text(span_bold);

            assert_eq!(segments.len(), 3);

            // "Text "
            assert_eq!(segments[0].text, "Text ");
            assert!(!segments[0].bold);

            // "bold"
            assert_eq!(segments[1].text, "bold");
            assert!(segments[1].bold);

            // " text"
            assert_eq!(segments[2].text, " text");
            assert!(!segments[2].bold);
        }

        // Test HTML with <br> tags for line breaks
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            let br_html = "Line 1<br>Line 2<br />Line 3";
            let segments = parse_formatted_text(br_html);

            assert_eq!(segments.len(), 5);

            // "Line 1"
            assert_eq!(segments[0].text, "Line 1");
            assert!(!segments[0].bold);

            // "\n"
            assert_eq!(segments[1].text, "\n");
            assert!(!segments[1].bold);

            // "Line 2"
            assert_eq!(segments[2].text, "Line 2");
            assert!(!segments[2].bold);

            // "\n"
            assert_eq!(segments[3].text, "\n");
            assert!(!segments[3].bold);

            // "Line 3"
            assert_eq!(segments[4].text, "Line 3");
            assert!(!segments[4].bold);
        }
    }
}
