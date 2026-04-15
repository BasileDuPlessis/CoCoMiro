//! # Text Input Overlay System
//!
//! This module handles the creation and management of text input overlays
//! for editing sticky note content. It provides functionality to create
//! positioned HTML input elements that overlay sticky notes for text editing.

use regex::Regex;

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
                crate::log_info(&format!("Successfully applied {} formatting", format_type));
            } else {
                crate::log_warn(&format!("Failed to apply {} formatting", format_type));
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

/// Sanitizes HTML content for safe pasting, allowing only formatting tags.
///
/// This function removes potentially dangerous elements like scripts, styles,
/// and other non-formatting tags, while preserving basic text formatting
/// (bold, italic, underline) that the application supports.
///
/// # Arguments
/// * `html` - The raw HTML content from clipboard
///
/// # Returns
/// Sanitized HTML content safe for insertion
fn sanitize_pasted_html(html: &str) -> String {
    let mut result = html.to_string();

    // Remove script and style tags completely (including their content)
    let script_re = Regex::new(r"<script[^>]*>.*?</script>").unwrap();
    result = script_re.replace_all(&result, "").to_string();

    let style_re = Regex::new(r"<style[^>]*>.*?</style>").unwrap();
    result = style_re.replace_all(&result, "").to_string();

    // Remove other dangerous tags
    let dangerous_tags = [
        "link", "meta", "iframe", "object", "embed", "form", "input", "button", "select",
        "textarea", "canvas", "svg",
    ];
    for tag in &dangerous_tags {
        let re = Regex::new(&format!(r"<{tag}[^>]*>.*?</{tag}>")).unwrap();
        result = re.replace_all(&result, "").to_string();
        // Self-closing
        let re_self = Regex::new(&format!(r"<{tag}[^>]*/>")).unwrap();
        result = re_self.replace_all(&result, "").to_string();
    }

    // Strip attributes from allowed inline tags
    let allowed_inline = ["b", "i", "u", "span", "strong", "em", "mark"];
    for tag in &allowed_inline {
        let re = Regex::new(&format!(r"<{tag}[^>]*>")).unwrap();
        result = re.replace_all(&result, &format!("<{tag}>")).to_string();
    }

    // Convert block elements to br
    let block_tags = [
        "p",
        "div",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "li",
        "ul",
        "ol",
        "blockquote",
    ];
    for tag in &block_tags {
        let re_open = Regex::new(&format!(r"<{tag}[^>]*>")).unwrap();
        result = re_open.replace_all(&result, "<br>").to_string();
        let re_close = Regex::new(&format!(r"</{tag}>")).unwrap();
        result = re_close.replace_all(&result, "<br>").to_string();
    }

    // Normalize br
    let br_re = Regex::new(r"<br[^>]*>").unwrap();
    result = br_re.replace_all(&result, "<br>").to_string();

    // Trim trailing <br>
    if result.ends_with("<br>") {
        result = result[..result.len() - 4].to_string();
    }

    result
}

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
    let note = match state
        .borrow()
        .sticky_notes
        .notes
        .iter()
        .find(|n| n.id == note_id)
    {
        Some(n) => n.clone(),
        None => {
            crate::log_warn(&format!(
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
            crate::log_warn("Cannot create contenteditable div element");
            return;
        }
    };

    let contenteditable: web_sys::HtmlElement = match contenteditable.dyn_into() {
        Ok(div) => div,
        Err(_) => {
            crate::log_warn("Cannot convert element to HtmlElement");
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
            crate::log_warn(&format!("Cannot create formatting toolbar: {}", e));
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
            crate::log_warn("Failed to attach input event listener");
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
                            crate::log_info(&format!(
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
                        crate::log_info(&format!("Text editing cancelled for note {}", note_id));

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
            crate::log_warn("Failed to attach keydown event listener");
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
            crate::log_warn("Failed to attach toolbar mousedown event listener");
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
            crate::log_info(&format!(
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
            crate::log_warn("Failed to attach blur event listener");
        });
    on_blur.forget();

    // Attach paste event listener for rich text paste handling
    let on_paste = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new({
        let contenteditable = contenteditable.clone();
        move |event: web_sys::Event| {
            crate::log_info("Paste event triggered");
            event.prevent_default();
            event.stop_propagation();

            // Try to get clipboard data using JavaScript interop
            let event_js = event.as_ref();
            let clipboard_data_js = js_sys::Reflect::get(event_js, &"clipboardData".into());

            if !clipboard_data_js.is_err() {
                let clipboard_data = clipboard_data_js.unwrap();

                // Try to get HTML content first
                let html_result = js_sys::Reflect::get(&clipboard_data, &"getData".into());
                if !html_result.is_err() {
                    let get_data_fn = html_result.unwrap();
                    if get_data_fn.is_function() {
                        let get_data_fn = js_sys::Function::from(get_data_fn);
                        let html_content = get_data_fn.call1(&clipboard_data, &"text/html".into());

                        if !html_content.is_err() {
                            let html_content = html_content.unwrap();
                            if html_content.is_string() {
                                let html_str = html_content.as_string().unwrap();
                                if !html_str.is_empty() {
                                    crate::log_info(&format!(
                                        "Raw HTML from clipboard: {}",
                                        html_str
                                    ));
                                    let sanitized_html = sanitize_pasted_html(&html_str);
                                    crate::log_info(&format!("Sanitized HTML: {}", sanitized_html));

                                    // Use execCommand to insert HTML
                                    let document = contenteditable.owner_document().unwrap();
                                    let document_js = document.as_ref();

                                    let exec_command_fn = js_sys::Function::from(
                                        js_sys::Reflect::get(document_js, &"execCommand".into())
                                            .unwrap(),
                                    );
                                    let _ = exec_command_fn.call3(
                                        document_js,
                                        &"insertHTML".into(),
                                        &false.into(),
                                        &sanitized_html.into(),
                                    );

                                    crate::log_info("Pasted HTML content (sanitized)");
                                    return;
                                }
                            }
                        }
                    }
                }

                // Fall back to plain text
                let text_result = js_sys::Reflect::get(&clipboard_data, &"getData".into());
                if !text_result.is_err() {
                    let get_data_fn = text_result.unwrap();
                    if get_data_fn.is_function() {
                        let get_data_fn = js_sys::Function::from(get_data_fn);
                        let text_content = get_data_fn.call1(&clipboard_data, &"text/plain".into());

                        if !text_content.is_err() {
                            let text_content = text_content.unwrap();
                            if text_content.is_string() {
                                let text_str = text_content.as_string().unwrap();
                                if !text_str.is_empty() {
                                    crate::log_info(&format!(
                                        "Plain text from clipboard: {}",
                                        text_str
                                    ));
                                    // Use execCommand to insert text
                                    let document = contenteditable.owner_document().unwrap();
                                    let document_js = document.as_ref();

                                    let exec_command_fn = js_sys::Function::from(
                                        js_sys::Reflect::get(document_js, &"execCommand".into())
                                            .unwrap(),
                                    );
                                    let _ = exec_command_fn.call3(
                                        document_js,
                                        &"insertText".into(),
                                        &false.into(),
                                        &text_str.into(),
                                    );

                                    crate::log_info("Pasted plain text content");
                                    return;
                                }
                            }
                        }
                    }
                }
            }

            crate::log_warn("Failed to access clipboard data during paste");
        }
    }));
    contenteditable
        .add_event_listener_with_callback("paste", on_paste.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::log_warn("Failed to attach paste event listener");
        });
    on_paste.forget();

    // Add to document
    if let Some(body) = document.body() {
        let _ = body.append_child(&toolbar);
        let _ = body.append_child(&contenteditable);
    }

    crate::log_info(&format!("Created text input overlay for note {}", note_id));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_pasted_html_simple_text() {
        // Test that simple text is preserved
        let input = "Hello world";
        let result = sanitize_pasted_html(input);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_sanitize_pasted_html_simple_formatted() {
        // Test simple formatted text
        let input = "Hello <b>world</b>";
        let result = sanitize_pasted_html(input);
        assert_eq!(result, "Hello <b>world</b>");
    }

    #[test]
    fn test_sanitize_pasted_html_removes_dangerous_tags() {
        // Test that dangerous tags are removed
        let input = r#"<p>Hello <script>alert('xss')</script> world</p>"#;
        let result = sanitize_pasted_html(input);
        assert_eq!(result, "<br>Hello  world");
    }

    #[test]
    fn test_sanitize_pasted_html_converts_block_elements() {
        // Test that block elements are converted to line breaks
        let input = r#"<p>First paragraph</p><p>Second paragraph</p>"#;
        let result = sanitize_pasted_html(input);
        assert_eq!(result, "<br>First paragraph<br><br>Second paragraph");
    }

    #[test]
    fn test_sanitize_pasted_html_google_slides_like() {
        // Test with Google Slides-like HTML (simplified)
        let input = r#"<p style="line-height: 1.2; margin: 0;"><span style="font-weight: 700;">Bold text</span> and normal text</p>"#;
        let result = sanitize_pasted_html(input);
        // Should convert p to br and keep span (though span attributes will be stripped)
        assert!(result.contains("<br>"));
        assert!(result.contains("<span>"));
        assert!(!result.contains("style="));
    }

    #[test]
    fn test_sanitize_pasted_html_strips_attributes() {
        // Test that attributes are stripped from allowed tags
        let input = r#"<b style="color: red;">Bold</b><i class="italic">Italic</i>"#;
        let result = sanitize_pasted_html(input);
        assert_eq!(result, "<b>Bold</b><i>Italic</i>");
    }

    #[test]
    fn test_sanitize_pasted_html_plain_text() {
        // Test that plain text is preserved
        let input = "Just plain text";
        let result = sanitize_pasted_html(input);
        assert_eq!(result, "Just plain text");
    }

    #[test]
    fn test_sanitize_pasted_html_mixed_content() {
        // Test mixed content with allowed and disallowed elements
        let input = r#"<p>Hello <b>world</b></p><script>bad</script><i>good</i>"#;
        let result = sanitize_pasted_html(input);
        assert!(result.contains("<br>Hello <b>world</b><br>"));
        assert!(result.contains("<i>good</i>"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_sanitize_pasted_html_google_slides_realistic() {
        // Test with realistic Google Slides HTML
        let input = r#"<p style="line-height: 1.15; margin-top: 0pt; margin-bottom: 0pt;"><span style="font-size: 11pt; font-family: Arial; color: #000000; background-color: transparent; font-weight: 400; font-style: normal; font-variant: normal; text-decoration: none; vertical-align: baseline; white-space: pre-wrap;">This is </span><span style="font-size: 11pt; font-family: Arial; color: #000000; background-color: transparent; font-weight: 700; font-style: normal; font-variant: normal; text-decoration: none; vertical-align: baseline; white-space: pre-wrap;">bold</span><span style="font-size: 11pt; font-family: Arial; color: #000000; background-color: transparent; font-weight: 400; font-style: normal; font-variant: normal; text-decoration: none; vertical-align: baseline; white-space: pre-wrap;"> text</span></p>"#;
        let result = sanitize_pasted_html(input);
        // Should convert p to br and keep spans (though attributes will be stripped)
        assert!(result.contains("<br>"));
        assert!(result.contains("<span>"));
        assert!(!result.contains("style="));
        assert!(!result.contains("font-size"));
        assert!(!result.contains("font-family"));
    }

    #[test]
    fn test_sanitize_pasted_html_complex_nested() {
        // Test with complex nested HTML
        let input = r#"<div><p>Hello <b>world</b></p><p>Second <i>paragraph</i></p></div>"#;
        let result = sanitize_pasted_html(input);
        // Should convert div and p to br, keep b and i
        assert!(result.contains("<br>Hello <b>world</b><br>"));
        assert!(result.contains("<br>Second <i>paragraph</i>"));
        assert!(!result.contains("<div>"));
        assert!(!result.contains("</div>"));
    }

    #[test]
    fn test_sanitize_pasted_html_empty_and_whitespace() {
        // Test with empty tags and whitespace
        let input = r#"<p>   </p><p>Hello</p><br><br>"#;
        let result = sanitize_pasted_html(input);
        // Should clean up empty paragraphs and multiple br tags
        assert!(result.contains("<br>Hello"));
        assert!(!result.contains("<p>   </p>"));
    }
}
