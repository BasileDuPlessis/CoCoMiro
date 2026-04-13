//! # Text Input Overlay System
//!
//! This module handles the creation and management of text input overlays
//! for editing sticky note content. It provides functionality to create
//! positioned HTML input elements that overlay sticky notes for text editing.

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

/// Creates a text input overlay positioned over a sticky note for editing.
///
/// This function creates an HTML textarea element that overlays the specified sticky note,
/// allowing the user to edit the note's text content. The textarea is styled to match the
/// note's appearance and includes event handling for text input.
///
/// # Arguments
/// * `canvas` - The canvas element for coordinate calculations
/// * `state` - Reference to application state containing the note
/// * `note_id` - ID of the note to create input overlay for
/// * `render` - Closure to trigger canvas re-rendering when content changes
#[cfg(target_arch = "wasm32")]
pub fn create_text_input_overlay(
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
) {
    let browser_window = match web_sys::window() {
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
    let canvas_js: &wasm_bindgen::JsValue = canvas.as_ref();
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
    let on_input = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new({
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

    // Store original content for potential cancellation
    let original_content = note.content.clone();

    // Attach keydown event listener for Enter/Escape handling
    let on_keydown = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let note_id = note_id;
        let textarea = textarea.clone();
        let document = document.clone();
        let original_content = original_content.clone();
        move |event: web_sys::KeyboardEvent| {
            event.stop_propagation();

            match event.key().as_str() {
                "Enter" => {
                    // Confirm changes - content already updated via input handler
                    crate::log_info(&format!("Text editing confirmed for note {}", note_id));

                    // Remove the textarea overlay
                    if let Some(body) = document.body() {
                        let _ = body.remove_child(&textarea);
                    }
                }
                "Escape" => {
                    // Cancel editing - restore original content
                    if let Some(note) = state.borrow_mut().sticky_notes.get_note_mut(note_id) {
                        note.content = original_content.clone();
                    }
                    crate::log_info(&format!("Text editing cancelled for note {}", note_id));

                    // Remove the textarea overlay
                    if let Some(body) = document.body() {
                        let _ = body.remove_child(&textarea);
                    }

                    // Re-render to show restored content
                    render();
                }
                _ => {
                    // Allow other keys to be handled normally
                }
            }
        }
    }));
    textarea.add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::log_warn("Failed to attach keydown event listener");
        });
    on_keydown.forget();

    // Attach blur event listener for clicking outside
    let on_blur = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new({
        let document = document.clone();
        let textarea = textarea.clone();
        move |_event: web_sys::Event| {
            // Confirm changes when focus is lost
            crate::log_info(&format!("Text editing confirmed (blur) for note {}", note_id));

            // Remove the textarea overlay
            if let Some(body) = document.body() {
                let _ = body.remove_child(&textarea);
            }
        }
    }));
    textarea.add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::log_warn("Failed to attach blur event listener");
        });
    on_blur.forget();

    // Add to document
    if let Some(body) = document.body() {
        let _ = body.append_child(&textarea);
    }

    crate::log_info(&format!("Created text input overlay for note {}", note_id));
}