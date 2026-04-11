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
    r#"
        <main class="app-shell">
            <section class="canvas-panel">
                <p id="canvas-status" class="canvas-status" role="status" aria-live="polite">Pan (0, 0) · Zoom 1.00× · Drag to pan, scroll to zoom, or use the arrow keys and +/-.</p>
                <div id="canvas-workspace" class="canvas-workspace">
                    <canvas id="infinite-canvas" tabindex="0" aria-label="Infinite canvas workspace" aria-describedby="canvas-status" title="Use arrow keys to pan, plus/minus to zoom, and 0 to reset the view." data-ready="false" data-pan-x="0" data-pan-y="0" data-zoom="1"></canvas>
                    <div id="floating-toolbar" class="floating-toolbar" role="toolbar" aria-orientation="vertical" aria-label="Floating canvas toolbar" title="Drag the handle to reposition the toolbar." data-x="18" data-y="18">
                        <div id="floating-toolbar-handle" class="floating-toolbar__handle" aria-label="Drag handle" title="Drag handle"></div>
                        <button id="add-note-button" class="floating-toolbar__button" aria-label="Add sticky note" title="Add a new sticky note">+</button>
                    </div>
                </div>
            </section>
        </main>
        "#
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
) -> crate::AppResult<(HtmlElement, HtmlCanvasElement, HtmlElement, HtmlElement)> {
    let body = document
        .body()
        .ok_or_else(|| crate::AppError::Dom("document has no body element".to_string()))?;
    body.set_inner_html(&app_markup());

    let workspace = document
        .get_element_by_id("canvas-workspace")
        .ok_or_else(|| crate::AppError::Dom("canvas workspace element not found".to_string()))?
        .dyn_into::<HtmlElement>()
        .map_err(|_| crate::AppError::Dom("canvas workspace is not an HTML element".to_string()))?;
    let canvas = document
        .get_element_by_id("infinite-canvas")
        .ok_or_else(|| crate::AppError::Dom("canvas element not found".to_string()))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| crate::AppError::Dom("infinite-canvas is not a canvas element".to_string()))?;
    let status = document
        .get_element_by_id("canvas-status")
        .ok_or_else(|| crate::AppError::Dom("status element not found".to_string()))?
        .dyn_into::<HtmlElement>()
        .map_err(|_| crate::AppError::Dom("canvas-status is not an HTML element".to_string()))?;
    let toolbar = document
        .get_element_by_id("floating-toolbar")
        .ok_or_else(|| crate::AppError::Dom("floating toolbar element not found".to_string()))?
        .dyn_into::<HtmlElement>()
        .map_err(|_| crate::AppError::Dom("floating-toolbar is not an HTML element".to_string()))?;
    let toolbar_handle = document
        .get_element_by_id("floating-toolbar-handle")
        .ok_or_else(|| crate::AppError::Dom("floating toolbar handle element not found".to_string()))?
        .dyn_into::<HtmlElement>()
        .map_err(|_| crate::AppError::Dom("floating-toolbar-handle is not an HTML element".to_string()))?;
    toolbar.set_attribute("data-handle-id", &toolbar_handle.id())?;

    Ok((workspace, canvas, status, toolbar))
}
