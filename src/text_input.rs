//! # Text Input Overlay System
//!
//! This module handles the creation and management of text input overlays
//! for editing sticky note content. It provides functionality to create
//! positioned HTML input elements that overlay sticky notes for text editing.

#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

/// Creates a formatting toolbar positioned above the text input overlay.
///
/// This function creates an HTML toolbar element with formatting buttons
/// (bold, italic, underline) that appears above the text input area.
/// The toolbar is styled to match the application design and handles
/// button clicks to apply text formatting.
///
/// # Arguments
/// * `document` - Reference to the browser document object
/// * `contenteditable` - The contenteditable element the toolbar controls
/// * `overlay_left` - Left position of the text input overlay
/// * `overlay_top` - Top position of the text input overlay
/// * `screen_width` - Width of the text input overlay
///
/// # Returns
/// The created toolbar HTML element
#[cfg(target_arch = "wasm32")]
fn create_formatting_toolbar(
    document: &web_sys::Document,
    contenteditable: &web_sys::HtmlElement,
    overlay_left: f64,
    overlay_top: f64,
    screen_width: f64,
) -> Result<web_sys::HtmlElement, String> {
    // Create toolbar container
    let toolbar = document
        .create_element("div")
        .map_err(|_| "Cannot create toolbar element")?;

    let toolbar: web_sys::HtmlElement = toolbar
        .dyn_into()
        .map_err(|_| "Cannot convert toolbar to HtmlElement")?;

    // Style the toolbar
    let _ = toolbar.style().set_property("position", "absolute");
    let _ = toolbar
        .style()
        .set_property("left", &format!("{}px", overlay_left));
    let _ = toolbar
        .style()
        .set_property("top", &format!("{}px", overlay_top - 40.0)); // Position above contenteditable
    let _ = toolbar
        .style()
        .set_property("width", &format!("{}px", screen_width.min(200.0))); // Limit max width
    let _ = toolbar.style().set_property("height", "32px");
    let _ = toolbar.style().set_property("background-color", "#ffffff");
    let _ = toolbar.style().set_property("border", "1px solid #e5e7eb");
    let _ = toolbar.style().set_property("border-radius", "4px");
    let _ = toolbar
        .style()
        .set_property("box-shadow", "0 1px 3px rgba(0, 0, 0, 0.1)");
    let _ = toolbar.style().set_property("display", "flex");
    let _ = toolbar.style().set_property("align-items", "center");
    let _ = toolbar.style().set_property("padding", "4px");
    let _ = toolbar.style().set_property("gap", "2px");
    let _ = toolbar.style().set_property("z-index", "1001"); // Higher than contenteditable
    let _ = toolbar.style().set_property("font-size", "12px");

    // Create bold button
    let bold_button = document
        .create_element("button")
        .map_err(|_| "Cannot create bold button")?;
    bold_button.set_text_content(Some("B"));
    let _ = bold_button.set_attribute("title", "Bold");
    let _ = bold_button.set_attribute("aria-label", "Make text bold");
    style_formatting_button(&bold_button, "font-weight: bold;")?;

    // Create italic button
    let italic_button = document
        .create_element("button")
        .map_err(|_| "Cannot create italic button")?;
    italic_button.set_text_content(Some("I"));
    let _ = italic_button.set_attribute("title", "Italic");
    let _ = italic_button.set_attribute("aria-label", "Make text italic");
    style_formatting_button(&italic_button, "font-style: italic;")?;

    // Create underline button
    let underline_button = document
        .create_element("button")
        .map_err(|_| "Cannot create underline button")?;
    underline_button.set_text_content(Some("U"));
    let _ = underline_button.set_attribute("title", "Underline");
    let _ = underline_button.set_attribute("aria-label", "Underline text");
    style_formatting_button(&underline_button, "text-decoration: underline;")?;

    // Add buttons to toolbar
    let _ = toolbar.append_child(&bold_button);
    let _ = toolbar.append_child(&italic_button);
    let _ = toolbar.append_child(&underline_button);

    // Add click handlers for formatting buttons
    add_formatting_handler(&bold_button, "bold", &contenteditable)?;
    add_formatting_handler(&italic_button, "italic", &contenteditable)?;
    add_formatting_handler(&underline_button, "underline", &contenteditable)?;

    Ok(toolbar)
}

/// Styles a formatting button with consistent appearance.
///
/// # Arguments
/// * `button` - The button element to style
/// * `font_style` - CSS font styling to apply to the button text
#[cfg(target_arch = "wasm32")]
fn style_formatting_button(button: &web_sys::Element, font_style: &str) -> Result<(), String> {
    let style = button
        .dyn_ref::<web_sys::HtmlElement>()
        .ok_or("Button is not an HtmlElement")?
        .style();

    let _ = style.set_property("width", "24px");
    let _ = style.set_property("height", "24px");
    let _ = style.set_property("border", "1px solid #d1d5db");
    let _ = style.set_property("border-radius", "3px");
    let _ = style.set_property("background-color", "#ffffff");
    let _ = style.set_property("color", "#374151");
    let _ = style.set_property("cursor", "pointer");
    let _ = style.set_property("display", "flex");
    let _ = style.set_property("align-items", "center");
    let _ = style.set_property("justify-content", "center");
    let _ = style.set_property("font-size", "12px");
    let _ = style.set_property("font-family", "Inter, sans-serif");
    let _ = style.set_property(
        &font_style.split(':').next().unwrap_or(""),
        &font_style.split(':').nth(1).unwrap_or("").trim(),
    );
    let _ = style.set_property("user-select", "none");
    let _ = style.set_property("transition", "background-color 0.1s");

    Ok(())
}

/// Adds a click handler to a formatting button.
///
/// # Arguments
/// * `button` - The button element to add handler to
/// * `format_type` - The type of formatting ("bold", "italic", "underline")
/// * `contenteditable` - The contenteditable element to apply formatting to
#[cfg(target_arch = "wasm32")]
fn add_formatting_handler(
    button: &web_sys::Element,
    format_type: &str,
    contenteditable: &web_sys::HtmlElement,
) -> Result<(), String> {
    let contenteditable = contenteditable.clone();
    let format_type = format_type.to_string();

    let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
        move |event: web_sys::Event| {
            event.prevent_default();
            event.stop_propagation();

            // Get the document to execute formatting commands
            let document = contenteditable.owner_document().unwrap();

            // Apply formatting using document.execCommand()
            let document_js = document.as_ref();
            let command = wasm_bindgen::JsValue::from_str(format_type.as_str());
            let exec_command_fn = js_sys::Function::from(
                js_sys::Reflect::get(document_js, &wasm_bindgen::JsValue::from_str("execCommand"))
                    .unwrap(),
            );
            let success = exec_command_fn
                .call1(document_js, &command)
                .ok()
                .and_then(|val| val.as_bool())
                .unwrap_or(false);

            if success {
                crate::logging::log_info(&format!(
                    "Successfully applied {} formatting",
                    format_type
                ));
            } else {
                crate::logging::log_warn(&format!("Failed to apply {} formatting", format_type));
            }

            // Focus the contenteditable to keep it active
            let _ = contenteditable.focus();
        },
    ));

    button
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .map_err(|_| "Failed to add click event listener")?;

    closure.forget();
    Ok(())
}

/// Sanitizes HTML content by extracting only plain text content.
///
/// This function creates a temporary DOM element, sets its innerHTML to the
/// provided HTML string, and then returns the textContent, which strips all
/// HTML tags and provides only the plain text content.
///
/// # Arguments
/// * `html` - The HTML string to sanitize
///
/// # Returns
/// The plain text content with all HTML tags removed
#[cfg(target_arch = "wasm32")]
fn sanitize_html_to_text(html: &str) -> String {
    // Get the document
    let document = match web_sys::window().and_then(|w| w.document()) {
        Some(doc) => doc,
        None => return html.to_string(), // Fallback to original if no document
    };

    // Create a temporary div element
    let temp_div = match document.create_element("div") {
        Ok(div) => div,
        Err(_) => return html.to_string(), // Fallback
    };

    // Set the HTML content
    temp_div.set_inner_html(html);

    // Get the text content (this strips all HTML tags)
    temp_div.text_content().unwrap_or_else(|| html.to_string())
}

/// Sanitizes HTML content for safe pasting, allowing only formatting tags.
///
/// This function removes potentially dangerous elements like scripts, styles,
/// and other non-formatting tags, while preserving basic text formatting
/// (bold, italic, underline) that the application supports.
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
            crate::logging::log_warn("Cannot create text input overlay: window unavailable");
            return;
        }
    };

    let document = match browser_window.document() {
        Some(d) => d,
        None => {
            crate::logging::log_warn("Cannot create text input overlay: document unavailable");
            return;
        }
    };

    // Get note details
    let note = match state
        .borrow()
        .sticky_notes
        .notes
        .iter()
        .find(|n| n.id == note_id)
    {
        Some(n) => n.clone(),
        None => {
            crate::logging::log_warn(&format!(
                "Cannot create input overlay for note {}: note not found",
                note_id
            ));
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

    // Create contenteditable div element for seamless text editing
    let contenteditable = match document.create_element("div") {
        Ok(el) => el,
        Err(_) => {
            crate::logging::log_warn("Cannot create contenteditable div element");
            return;
        }
    };

    let contenteditable: web_sys::HtmlElement = match contenteditable.dyn_into() {
        Ok(div) => div,
        Err(_) => {
            crate::logging::log_warn("Cannot convert element to HtmlElement");
            return;
        }
    };

    // Set contenteditable attribute
    let _ = contenteditable.set_attribute("contenteditable", "true");

    // Create formatting toolbar
    let toolbar = match create_formatting_toolbar(
        &document,
        &contenteditable,
        overlay_left,
        overlay_top,
        screen_width,
    ) {
        Ok(tb) => tb,
        Err(e) => {
            crate::logging::log_warn(&format!("Cannot create formatting toolbar: {}", e));
            return;
        }
    };

    // Style the contenteditable div to match the note exactly
    let _ = contenteditable.style().set_property("position", "absolute");
    let _ = contenteditable
        .style()
        .set_property("left", &format!("{}px", overlay_left));
    let _ = contenteditable
        .style()
        .set_property("top", &format!("{}px", overlay_top));
    let _ = contenteditable
        .style()
        .set_property("width", &format!("{}px", screen_width));
    let _ = contenteditable
        .style()
        .set_property("height", &format!("{}px", screen_height));
    let _ = contenteditable.style().set_property("font-size", "14px");
    let _ = contenteditable
        .style()
        .set_property("font-family", "Inter, sans-serif");
    let _ = contenteditable.style().set_property("border", "none");
    let _ = contenteditable.style().set_property("border-radius", "0px");
    let _ = contenteditable.style().set_property("padding", "8px");
    let _ = contenteditable
        .style()
        .set_property("background-color", &note.color);
    let _ = contenteditable.style().set_property("color", "#000000");
    let _ = contenteditable.style().set_property("outline", "none");
    let _ = contenteditable.style().set_property("z-index", "1000");
    let _ = contenteditable.style().set_property("text-align", "left");
    let _ = contenteditable
        .style()
        .set_property("box-sizing", "border-box");
    let _ = contenteditable.style().set_property("overflow", "hidden");
    let _ = contenteditable
        .style()
        .set_property("white-space", "pre-wrap");
    let _ = contenteditable
        .style()
        .set_property("word-wrap", "break-word");
    let _ = contenteditable.style().set_property("line-height", "1.2");

    // Set initial content and focus - handle both HTML and plain text content
    let initial_html = if note.content.contains('<') && note.content.contains('>') {
        // Content appears to be HTML, use as-is
        note.content.clone()
    } else {
        // Content appears to be plain text, convert line breaks to HTML
        note.content.replace("\n", "<br>")
    };
    contenteditable.set_inner_html(&initial_html);

    // Set initial height based on note height
    let initial_screen_height = screen_height;
    let _ = contenteditable
        .style()
        .set_property("height", &format!("{}px", initial_screen_height));

    let _ = contenteditable.focus();

    // Attach input event listener to handle text changes
    let on_input = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new({
        let state = state.clone();
        let render = render.clone();
        let note_id = note_id;
        let contenteditable = contenteditable.clone();
        move |event: web_sys::Event| {
            event.stop_propagation();

            // Update the note content with the current contenteditable content
            let zoom = state.borrow().viewport.zoom; // Get zoom before mutable borrow
            if let Some(note) = state.borrow_mut().sticky_notes.get_note_mut(note_id) {
                // Store HTML content directly instead of converting to plain text
                let html_content = contenteditable.inner_html();
                note.content = html_content;

                // Adjust contenteditable height to fit content
                // For contenteditable, we need to measure the scroll height
                let scroll_height = contenteditable.scroll_height() as f64;
                let _ = contenteditable
                    .style()
                    .set_property("height", &format!("{}px", scroll_height));

                // Adjust note height based on contenteditable height in world coordinates
                let min_height = 150.0; // Minimum note height
                let new_height = (scroll_height / zoom).max(min_height);
                note.height = new_height;
            }

            // Re-render the canvas to show the updated text
            render();
        }
    }));
    contenteditable
        .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach input event listener");
        });
    on_input.forget();

    // Store original content for potential cancellation
    let original_content = note.content.clone();

    // Attach keydown event listener for Enter/Escape handling
    let on_keydown =
        wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new({
            let state = state.clone();
            let render = render.clone();
            let note_id = note_id;
            let contenteditable = contenteditable.clone();
            let toolbar = toolbar.clone();
            let document = document.clone();
            let original_content = original_content.clone();
            move |event: web_sys::KeyboardEvent| {
                // Check for modifier key combinations that should be allowed to propagate
                let is_ctrl_or_cmd = event.ctrl_key() || event.meta_key();
                let key_str = event.key();
                let key = key_str.as_str();

                // Allow common text editing shortcuts to work normally
                if is_ctrl_or_cmd && matches!(key, "a" | "c" | "v" | "x" | "z" | "y") {
                    // Don't stop propagation for these shortcuts - let the browser handle them
                    return;
                }

                // Handle Tab key to insert spaces instead of navigating away
                if key == "Tab" {
                    event.prevent_default();
                    event.stop_propagation();

                    // Insert 4 spaces for tab in contenteditable
                    // For now, just prevent default and let contenteditable handle it
                    // TODO: Implement proper tab insertion

                    return;
                }

                event.stop_propagation();

                match key {
                    "Enter" => {
                        // Check if Ctrl or Shift is held for confirmation
                        if event.ctrl_key() || event.shift_key() {
                            // Confirm changes - content already updated via input handler
                            crate::logging::log_info(&format!(
                                "Text editing confirmed for note {}",
                                note_id
                            ));

                            // Remove the contenteditable overlay
                            if let Some(body) = document.body() {
                                let _ = body.remove_child(&toolbar);
                                let _ = body.remove_child(&contenteditable);
                            }
                        } else {
                            // Allow normal Enter for line breaks in contenteditable
                            // The contenteditable will handle this naturally
                        }
                    }
                    "Escape" => {
                        // Cancel editing - restore original content
                        if let Some(note) = state.borrow_mut().sticky_notes.get_note_mut(note_id) {
                            note.content = original_content.clone();
                        }
                        crate::logging::log_info(&format!(
                            "Text editing cancelled for note {}",
                            note_id
                        ));

                        // Remove the contenteditable overlay
                        if let Some(body) = document.body() {
                            let _ = body.remove_child(&toolbar);
                            let _ = body.remove_child(&contenteditable);
                        }

                        // Re-render to show restored content
                        render();
                    }
                    _ => {
                        // Allow other keys to be handled normally by the contenteditable
                    }
                }
            }
        }));
    contenteditable
        .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach keydown event listener");
        });
    on_keydown.forget();

    // Add mousedown handler to toolbar to prevent blur from removing overlay
    let toolbar_clicked = Rc::new(RefCell::new(false));
    let toolbar_clicked_clone = toolbar_clicked.clone();
    let toolbar_mousedown = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(
        Box::new(move |_event: web_sys::Event| {
            *toolbar_clicked_clone.borrow_mut() = true;
        }),
    );
    toolbar
        .add_event_listener_with_callback("mousedown", toolbar_mousedown.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach toolbar mousedown event listener");
        });
    toolbar_mousedown.forget();

    // Attach blur event listener for clicking outside
    let on_blur = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new({
        let document = document.clone();
        let contenteditable = contenteditable.clone();
        let toolbar = toolbar.clone();
        let toolbar_clicked = toolbar_clicked.clone();
        move |_event: web_sys::Event| {
            // Check if toolbar was clicked (preventing overlay removal)
            if *toolbar_clicked.borrow() {
                *toolbar_clicked.borrow_mut() = false; // Reset flag
                return;
            }

            // Confirm changes when focus is lost
            crate::logging::log_info(&format!(
                "Text editing confirmed (blur) for note {}",
                note_id
            ));

            // Remove the contenteditable overlay
            if let Some(body) = document.body() {
                let _ = body.remove_child(&toolbar);
                let _ = body.remove_child(&contenteditable);
            }
        }
    }));
    contenteditable
        .add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach blur event listener");
        });
    on_blur.forget();

    // Attach paste event listener to sanitize pasted content
    let on_paste = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
        move |event: web_sys::Event| {
            event.prevent_default();
            event.stop_propagation();

            // Get clipboard data from the event
            let clipboard_data = match js_sys::Reflect::get(&event, &"clipboardData".into()) {
                Ok(cd) => cd,
                Err(_) => {
                    crate::logging::log_warn("No clipboard data available in paste event");
                    return;
                }
            };

            // Try to get plain text first
            let pasted_text =
                if let Ok(text) = js_sys::Reflect::get(&clipboard_data, &"getData".into()) {
                    if let Ok(get_data_fn) = text.dyn_into::<js_sys::Function>() {
                        if let Ok(text_result) =
                            get_data_fn.call1(&clipboard_data, &"text/plain".into())
                        {
                            if let Some(text_str) = text_result.as_string() {
                                if !text_str.is_empty() {
                                    text_str
                                } else {
                                    // Fallback to HTML content and sanitize it
                                    if let Ok(html_result) =
                                        get_data_fn.call1(&clipboard_data, &"text/html".into())
                                    {
                                        if let Some(html_str) = html_result.as_string() {
                                            sanitize_html_to_text(&html_str)
                                        } else {
                                            String::new()
                                        }
                                    } else {
                                        String::new()
                                    }
                                }
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

            if !pasted_text.is_empty() {
                // Insert the sanitized text using execCommand
                let document = web_sys::window()
                    .and_then(|w| w.document())
                    .expect("Document should be available");

                // Use document.execCommand to insert text
                let exec_command_fn = js_sys::Function::from(
                    js_sys::Reflect::get(
                        document.as_ref(),
                        &wasm_bindgen::JsValue::from_str("execCommand"),
                    )
                    .unwrap(),
                );

                // Call execCommand("insertText", false, text)
                let _ = exec_command_fn.call3(
                    document.as_ref(),
                    &wasm_bindgen::JsValue::from_str("insertText"),
                    &wasm_bindgen::JsValue::from_bool(false),
                    &wasm_bindgen::JsValue::from_str(&pasted_text),
                );

                crate::logging::log_info("Pasted content sanitized and inserted");
            }
        },
    ));
    contenteditable
        .add_event_listener_with_callback("paste", on_paste.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach paste event listener");
        });
    on_paste.forget();

    // Add to document
    if let Some(body) = document.body() {
        let _ = body.append_child(&toolbar);
        let _ = body.append_child(&contenteditable);
    }

    crate::logging::log_info(&format!("Created text input overlay for note {}", note_id));
}
