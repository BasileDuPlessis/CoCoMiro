//! # Application UI Setup
//!
//! This module handles the initialization and setup of the CoCoMiro application's
//! user interface components. It provides functions for generating HTML markup
//! and integrating the application with the browser DOM.
//!
//! ## DOM Structure
//!
//! The application creates the following DOM structure:
//! ```html
//! <main class="app-shell">
//!   <section class="canvas-panel">
//!     <p id="canvas-status">...</p>
//!     <div id="canvas-workspace">
//!       <canvas id="infinite-canvas">...</canvas>
//!       <div id="floating-toolbar">
//!         <div id="floating-toolbar-handle"></div>
//!         <button id="add-note-button">+</button>
//!       </div>
//!     </div>
//!   </section>
//! </main>
//! ```

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{Document, HtmlCanvasElement, HtmlElement};

#[cfg(target_arch = "wasm32")]
/// Generates the complete HTML markup for the CoCoMiro application.
///
/// This function returns a static string containing the application's DOM structure.
/// The markup includes semantic HTML with proper ARIA labels and accessibility features.
///
/// # Returns
/// A string containing the complete HTML structure for the application
pub fn app_markup() -> String {
    r###"
        <main class="app-shell">
            <div id="login-overlay" class="login-overlay" role="dialog" aria-labelledby="login-title" aria-describedby="login-description" aria-modal="true" aria-hidden="true">
                <div class="login-overlay__content">
                    <div class="login-overlay__header">
                        <h1 id="login-title" class="login-overlay__title">Welcome to CoCoMiro</h1>
                        <p id="login-description" class="login-overlay__description">Sign in with Google to access your infinite canvas</p>
                    </div>
                    <div class="login-overlay__body">
                        <button id="google-signin-button" class="google-signin-button" type="button" aria-label="Sign in with Google">
                            <svg class="google-signin-button__icon" viewBox="0 0 24 24" aria-hidden="true">
                                <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                            </svg>
                            <span class="google-signin-button__text">Sign in with Google</span>
                        </button>
                        <div id="login-status" class="login-status" role="status" aria-live="polite"></div>
                    </div>
                </div>
            </div>
            <section class="canvas-panel">
                <p id="canvas-status" class="canvas-status" role="status" aria-live="polite">Pan (0, 0) · Zoom 1.00× · Drag to pan, scroll to zoom, or use the arrow keys and +/-.</p>
                <div id="canvas-workspace" class="canvas-workspace">
                    <canvas id="infinite-canvas" tabindex="0" aria-label="Infinite canvas workspace" aria-describedby="canvas-status" title="Use arrow keys to pan, plus/minus to zoom, and 0 to reset the view." data-ready="false" data-pan-x="0" data-pan-y="0" data-zoom="1"></canvas>
                     <div id="floating-toolbar" class="floating-toolbar" role="toolbar" aria-orientation="vertical" aria-label="Floating canvas toolbar" title="Drag the handle to reposition the toolbar." data-x="18" data-y="18">
                         <div id="floating-toolbar-handle" class="floating-toolbar__handle" aria-label="Drag handle" title="Drag handle"></div>
                         <button id="add-note-button" class="floating-toolbar__button" aria-label="Add sticky note" title="Add a new sticky note">+</button>
                         <button id="login-button" class="floating-toolbar__button" aria-label="Sign in with Google" title="Sign in with Google" style="display: none;">🔐</button>
                         <button id="logout-button" class="floating-toolbar__button" aria-label="Sign out" title="Sign out" style="display: none;">🚪</button>
                     </div>
                </div>
            </section>
        </main>
        "###
    .to_string()
}

#[cfg(target_arch = "wasm32")]
/// Installs the CoCoMiro application into the browser DOM.
///
/// This function performs the following setup:
/// 1. Inserts the application HTML into the document body
/// 2. Retrieves references to key DOM elements (canvas, toolbar, status)
/// 3. Performs type casting and validation of DOM elements
/// 4. Sets up toolbar handle associations for drag functionality
///
/// # Arguments
/// * `document` - Reference to the browser document object
///
/// # Returns
/// * `Ok((workspace, canvas, status, toolbar))` - Successfully retrieved DOM elements
/// * `Err(AppError)` - Failed to access or cast DOM elements
pub fn install_app(
    document: &Document,
) -> crate::error::AppResult<(HtmlElement, HtmlCanvasElement, HtmlElement, HtmlElement)> {
    let body = document
        .body()
        .ok_or_else(|| crate::error::AppError::Dom("document has no body element".to_string()))?;
    body.set_inner_html(&app_markup());

    let workspace = document
        .get_element_by_id("canvas-workspace")
        .ok_or_else(|| {
            crate::error::AppError::Dom("canvas workspace element not found".to_string())
        })?
        .dyn_into::<HtmlElement>()
        .map_err(|_| {
            crate::error::AppError::Dom("canvas workspace is not an HTML element".to_string())
        })?;
    let canvas = document
        .get_element_by_id("infinite-canvas")
        .ok_or_else(|| crate::error::AppError::Dom("canvas element not found".to_string()))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| {
            crate::error::AppError::Dom("infinite-canvas is not a canvas element".to_string())
        })?;
    let status = document
        .get_element_by_id("canvas-status")
        .ok_or_else(|| crate::error::AppError::Dom("status element not found".to_string()))?
        .dyn_into::<HtmlElement>()
        .map_err(|_| {
            crate::error::AppError::Dom("canvas-status is not an HTML element".to_string())
        })?;
    let toolbar = document
        .get_element_by_id("floating-toolbar")
        .ok_or_else(|| {
            crate::error::AppError::Dom("floating toolbar element not found".to_string())
        })?
        .dyn_into::<HtmlElement>()
        .map_err(|_| {
            crate::error::AppError::Dom("floating-toolbar is not an HTML element".to_string())
        })?;
    let toolbar_handle = document
        .get_element_by_id("floating-toolbar-handle")
        .ok_or_else(|| {
            crate::error::AppError::Dom("floating toolbar handle element not found".to_string())
        })?
        .dyn_into::<HtmlElement>()
        .map_err(|_| {
            crate::error::AppError::Dom(
                "floating-toolbar-handle is not an HTML element".to_string(),
            )
        })?;
    toolbar.set_attribute("data-handle-id", &toolbar_handle.id())?;

    Ok((workspace, canvas, status, toolbar))
}
