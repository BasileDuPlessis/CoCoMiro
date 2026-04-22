//! Centralized styling utilities for dynamic CSS properties
//!
//! This module provides helper functions for setting dynamic inline styles
//! that cannot be expressed as static CSS classes.

use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

/// Extension trait for common styling operations on HTML elements
pub trait ElementStyling {
    /// Set the cursor style for an element
    fn set_cursor(&self, cursor: &str) -> Result<(), JsValue>;

    /// Set transform property for positioning
    fn set_transform(&self, transform: &str) -> Result<(), JsValue>;

    /// Set position and size for overlay elements
    fn set_overlay_position(
        &self,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
    ) -> Result<(), JsValue>;

    /// Set background color
    fn set_background_color(&self, color: &str) -> Result<(), JsValue>;
}

impl ElementStyling for HtmlElement {
    fn set_cursor(&self, cursor: &str) -> Result<(), JsValue> {
        self.style().set_property("cursor", cursor)
    }

    fn set_transform(&self, transform: &str) -> Result<(), JsValue> {
        self.style().set_property("transform", transform)
    }

    fn set_overlay_position(
        &self,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
    ) -> Result<(), JsValue> {
        let style = self.style();
        style.set_property("left", &format!("{}px", left))?;
        style.set_property("top", &format!("{}px", top))?;
        style.set_property("width", &format!("{}px", width))?;
        style.set_property("height", &format!("{}px", height))?;
        Ok(())
    }

    fn set_background_color(&self, color: &str) -> Result<(), JsValue> {
        self.style().set_property("background-color", color)
    }
}

/// Styling functions for specific UI components
#[cfg(target_arch = "wasm32")]
pub mod components {
    use crate::AppState;
    use wasm_bindgen::JsValue;
    use web_sys::HtmlElement;

    /// Vertical offset for positioning formatting toolbar above text input (in pixels)
    pub const FORMATTING_TOOLBAR_VERTICAL_OFFSET: f64 = 40.0;
    /// Maximum width limit for text input overlays (in pixels)
    pub const TEXT_INPUT_MAX_WIDTH: f64 = 200.0;
    /// Default test viewport width for styling calculations (in pixels)
    pub const TEST_VIEWPORT_WIDTH: f64 = 800.0;
    /// Default test viewport height for styling calculations (in pixels)
    pub const TEST_VIEWPORT_HEIGHT: f64 = 600.0;

    /// Style the text input toolbar with dynamic positioning
    #[cfg(target_arch = "wasm32")]
    pub fn style_text_input_toolbar(
        toolbar: &HtmlElement,
        left: f64,
        top: f64,
        width: f64,
    ) -> Result<(), JsValue> {
        // Set CSS class for static styles
        toolbar.set_attribute("class", "text-input-toolbar")?;

        // Set dynamic positioning styles
        let style = toolbar.style();
        style.set_property("left", &format!("{}px", left))?;
        style.set_property(
            "top",
            &format!("{}px", top - FORMATTING_TOOLBAR_VERTICAL_OFFSET),
        )?; // Position above contenteditable
        style.set_property("width", &format!("{}px", width.min(TEXT_INPUT_MAX_WIDTH)))?; // Limit max width

        Ok(())
    }

    /// Style the contenteditable overlay to match a sticky note
    #[cfg(target_arch = "wasm32")]
    pub fn style_contenteditable_overlay(
        overlay: &HtmlElement,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
        background_color: &str,
    ) -> Result<(), JsValue> {
        // Set CSS class for static styles
        overlay.set_attribute("class", "contenteditable-overlay")?;

        // Set dynamic positioning and sizing
        let style = overlay.style();
        style.set_property("left", &format!("{}px", left))?;
        style.set_property("top", &format!("{}px", top))?;
        style.set_property("width", &format!("{}px", width))?;
        style.set_property("height", &format!("{}px", height))?;
        style.set_property("background-color", background_color)?;

        Ok(())
    }

    /// Update canvas cursor based on interaction state
    #[cfg(target_arch = "wasm32")]
    pub fn update_canvas_cursor(canvas: &HtmlElement, state: &AppState) -> Result<(), JsValue> {
        let canvas_width = canvas.client_width() as f64;
        let canvas_height = canvas.client_height() as f64;

        let cursor = if let Some((_note_id, handle)) = state.sticky_notes.find_resize_handle_at(
            state.mouse_x,
            state.mouse_y,
            &state.viewport,
            canvas_width,
            canvas_height,
        ) {
            handle.cursor()
        } else if state
            .sticky_notes
            .find_note_at(
                state
                    .viewport
                    .world_point_at(state.mouse_x, state.mouse_y, canvas_width, canvas_height)
                    .0,
                state
                    .viewport
                    .world_point_at(state.mouse_x, state.mouse_y, canvas_width, canvas_height)
                    .1,
            )
            .is_some()
        {
            "grab"
        } else if state.viewport.is_dragging {
            "grabbing"
        } else {
            "grab"
        };

        canvas.style().set_property("cursor", cursor)
    }

    /// Update toolbar position and cursor
    #[cfg(target_arch = "wasm32")]
    pub fn update_toolbar_position(
        toolbar: &HtmlElement,
        state: &mut crate::toolbar::FloatingToolbarState,
        max_x: f64,
        max_y: f64,
    ) -> Result<(), JsValue> {
        state.clamp_within(max_x, max_y);

        // Update data attributes for debugging/inspection
        toolbar.set_attribute("data-x", &format!("{:.2}", state.x))?;
        toolbar.set_attribute("data-y", &format!("{:.2}", state.y))?;

        // Set dynamic positioning
        let style = toolbar.style();
        style.set_property(
            "transform",
            &format!("translate({:.2}px, {:.2}px)", state.x, state.y),
        )?;

        // Set cursor based on drag state
        let cursor = if state.is_dragging {
            "grabbing"
        } else {
            "grab"
        };
        style.set_property("cursor", cursor)?;

        Ok(())
    }
}

/// Utility functions for dynamic sizing
#[cfg(target_arch = "wasm32")]
pub mod sizing {
    use wasm_bindgen::JsValue;
    use web_sys::HtmlElement;

    /// Set element dimensions
    pub fn set_dimensions(element: &HtmlElement, width: f64, height: f64) -> Result<(), JsValue> {
        let style = element.style();
        style.set_property("width", &format!("{}px", width.round()))?;
        style.set_property("height", &format!("{}px", height.round()))?;
        Ok(())
    }

    /// Update contenteditable height based on scroll height
    pub fn update_contenteditable_height(
        contenteditable: &HtmlElement,
        scroll_height: f64,
        initial_screen_height: f64,
    ) -> Result<(), JsValue> {
        let new_height = scroll_height.max(initial_screen_height);
        contenteditable
            .style()
            .set_property("height", &format!("{}px", new_height))?;
        Ok(())
    }
}
