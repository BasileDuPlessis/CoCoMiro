//! # Text Input Overlay System
//!
//! This module handles the creation and management of text input overlays
//! for editing sticky note content. It provides functionality to create
//! positioned HTML input elements that overlay sticky notes for text editing.

#[cfg(target_arch = "wasm32")]
use crate::sticky_notes::DEFAULT_NOTE_HEIGHT;
#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

/// Creates a formatting button with specified properties
#[cfg(target_arch = "wasm32")]
fn create_formatting_button(
    document: &web_sys::Document,
    text: &str,
    title: &str,
    aria_label: &str,
    class: &str,
) -> Result<web_sys::Element, JsValue> {
    let button = document
        .create_element("button")
        .map_err(|_| JsValue::from_str("Cannot create formatting button"))?;
    button.set_text_content(Some(text));
    let _ = button.set_attribute("title", title);
    let _ = button.set_attribute("aria-label", aria_label);
    let _ = button.set_attribute("class", class);
    Ok(button)
}

/// Creates the color button with current note color styling
#[cfg(target_arch = "wasm32")]
fn create_color_button(
    document: &web_sys::Document,
    current_color: &str,
) -> Result<web_sys::Element, JsValue> {
    let color_button = document
        .create_element("button")
        .map_err(|_| JsValue::from_str("Cannot create color button"))?;
    let _ = color_button.set_attribute("title", "Change note color");
    let _ = color_button.set_attribute("aria-label", "Change note background color");
    let _ = color_button.set_attribute("class", "formatting-button formatting-button--color");

    // Set button background to current note color
    let color_button_element: &web_sys::HtmlElement = color_button
        .dyn_ref()
        .ok_or_else(|| JsValue::from_str("Cannot cast color button to HtmlElement"))?;
    let _ = color_button_element
        .style()
        .set_property("background-color", current_color);

    Ok(color_button)
}

/// Creates the toolbar container and adds all buttons
#[cfg(target_arch = "wasm32")]
fn create_toolbar_container(
    document: &web_sys::Document,
    overlay_left: f64,
    overlay_top: f64,
    screen_width: f64,
) -> Result<web_sys::HtmlElement, JsValue> {
    let toolbar = document
        .create_element("div")
        .map_err(|_| JsValue::from_str("Cannot create toolbar element"))?;

    let toolbar: web_sys::HtmlElement = toolbar
        .dyn_into()
        .map_err(|_| JsValue::from_str("Cannot convert toolbar to HtmlElement"))?;

    // Style the toolbar with centralized styling function
    crate::styling::components::style_text_input_toolbar(
        &toolbar,
        overlay_left,
        overlay_top,
        screen_width,
    )?;

    Ok(toolbar)
}

/// Adds formatting buttons to the toolbar
#[cfg(target_arch = "wasm32")]
fn add_buttons_to_toolbar(
    document: &web_sys::Document,
    toolbar: &web_sys::HtmlElement,
    contenteditable: &web_sys::HtmlElement,
    current_color: &str,
) -> Result<web_sys::Element, JsValue> {
    // Create buttons
    let bold_button = create_formatting_button(
        document,
        "B",
        "Bold",
        "Make text bold",
        "formatting-button formatting-button--bold",
    )?;

    let italic_button = create_formatting_button(
        document,
        "I",
        "Italic",
        "Make text italic",
        "formatting-button formatting-button--italic",
    )?;

    let underline_button = create_formatting_button(
        document,
        "U",
        "Underline",
        "Underline text",
        "formatting-button formatting-button--underline",
    )?;

    let color_button = create_color_button(document, current_color)?;

    // Add buttons to toolbar
    let _ = toolbar.append_child(&bold_button);
    let _ = toolbar.append_child(&italic_button);
    let _ = toolbar.append_child(&underline_button);
    let _ = toolbar.append_child(&color_button);

    // Add click handlers for formatting buttons
    add_formatting_handler(&bold_button, "bold", contenteditable)?;
    add_formatting_handler(&italic_button, "italic", contenteditable)?;
    add_formatting_handler(&underline_button, "underline", contenteditable)?;

    Ok(color_button)
}

/// Creates a formatting toolbar positioned above the text input overlay.
///
/// This function creates an HTML toolbar element with formatting buttons
/// (bold, italic, underline, color) that appears above the text input area.
/// The toolbar is styled to match the application design and handles
/// button clicks to apply text formatting and change note colors.
///
/// # Arguments
/// * `document` - Reference to the browser document object
/// * `contenteditable` - The contenteditable element the toolbar controls
/// * `overlay_left` - Left position of the text input overlay
/// * `overlay_top` - Top position of the text input overlay
/// * `screen_width` - Width of the text input overlay
/// * `state` - Application state for updating note colors
/// * `note_id` - ID of the note being edited
/// * `render` - Render callback to update the display
/// * `current_color` - Current background color of the note
///
/// # Returns
/// A tuple containing the created toolbar HTML element and the color picker element
#[cfg(target_arch = "wasm32")]
fn create_formatting_toolbar(
    document: &web_sys::Document,
    contenteditable: &web_sys::HtmlElement,
    overlay_left: f64,
    overlay_top: f64,
    screen_width: f64,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
    current_color: &str,
) -> Result<(web_sys::HtmlElement, web_sys::HtmlElement), JsValue> {
    // Create toolbar container
    let toolbar = create_toolbar_container(document, overlay_left, overlay_top, screen_width)?;

    // Add buttons to toolbar and get color button
    let color_button = add_buttons_to_toolbar(document, &toolbar, contenteditable, current_color)?;

    // Create and add color picker
    let color_picker = add_color_picker_handler(
        &color_button,
        document,
        contenteditable,
        state,
        note_id,
        render,
        overlay_left,
        overlay_top,
        current_color,
    )?;

    Ok((toolbar, color_picker))
}

/// Applies text formatting using modern Selection/Range APIs instead of deprecated execCommand.
///
/// # Arguments
/// * `contenteditable` - The contenteditable element to apply formatting to
/// * `format_type` - The type of formatting ("bold", "italic", "underline")
///
/// # Returns
/// Result indicating success or failure
#[cfg(target_arch = "wasm32")]
fn apply_formatting(
    _contenteditable: &web_sys::HtmlElement,
    format_type: &str,
) -> Result<(), JsValue> {
    // Get the window
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window available"))?;

    // Get the current selection
    let selection = window
        .get_selection()
        .map_err(|_| JsValue::from_str("Failed to get selection"))?
        .ok_or_else(|| JsValue::from_str("No selection available"))?;

    let range_count = selection.range_count();

    if range_count == 0 {
        // No selection, insert empty formatting tags
        insert_empty_formatting_tags(&window, format_type)?;
    } else {
        // Apply formatting to each selected range
        for i in 0..range_count {
            let range = selection
                .get_range_at(i)
                .map_err(|_| JsValue::from_str("Failed to get range"))?;
            apply_formatting_to_range(&window, &range, format_type)?;
        }
    }

    Ok(())
}

/// Inserts empty formatting tags at the current cursor position when no text is selected.
#[cfg(target_arch = "wasm32")]
fn insert_empty_formatting_tags(
    window: &web_sys::Window,
    format_type: &str,
) -> Result<(), JsValue> {
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document available"))?;
    let selection = window
        .get_selection()
        .map_err(|_| JsValue::from_str("Failed to get selection"))?
        .ok_or_else(|| JsValue::from_str("No selection available"))?;

    if selection.range_count() == 0 {
        return Ok(()); // No cursor position
    }

    let range = selection
        .get_range_at(0)
        .map_err(|_| JsValue::from_str("Failed to get range"))?;

    // Create formatting element
    let tag_name = match format_type {
        "bold" => "b",
        "italic" => "i",
        "underline" => "u",
        _ => return Err(JsValue::from_str("Unknown format type")),
    };

    let formatting_element = document.create_element(tag_name)?;

    // Insert the element
    range.insert_node(&formatting_element)?;

    // Position cursor inside the element
    let new_range = document.create_range()?;
    new_range.select_node_contents(&formatting_element)?;
    new_range.collapse(); // Collapse to start
    selection.remove_all_ranges()?;
    selection.add_range(&new_range)?;

    Ok(())
}

/// Applies formatting to a selected range of text.
#[cfg(target_arch = "wasm32")]
fn apply_formatting_to_range(
    window: &web_sys::Window,
    range: &web_sys::Range,
    format_type: &str,
) -> Result<(), JsValue> {
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document available"))?;

    // Check if the range is collapsed (no selection)
    if range.collapsed() {
        return insert_empty_formatting_tags(window, format_type);
    }

    // Extract the selected content
    let selected_content = range.extract_contents()?;

    // Create the formatting element
    let tag_name = match format_type {
        "bold" => "b",
        "italic" => "i",
        "underline" => "u",
        _ => return Err(JsValue::from_str("Unknown format type")),
    };

    let formatting_element = document.create_element(tag_name)?;
    formatting_element.append_child(&selected_content)?;

    // Insert the formatted element back
    range.insert_node(&formatting_element)?;

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
) -> Result<(), JsValue> {
    let contenteditable = contenteditable.clone();
    let format_type = format_type.to_string();

    let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
        move |event: web_sys::Event| {
            event.prevent_default();
            event.stop_propagation();

            // Apply formatting using modern Selection/Range APIs
            if let Err(e) = apply_formatting(&contenteditable, &format_type) {
                crate::logging::log_warn(&format!(
                    "Failed to apply {} formatting: {:?}",
                    format_type, e
                ));
            } else {
                crate::logging::log_info(&format!(
                    "Successfully applied {} formatting",
                    format_type
                ));
            }

            // Focus the contenteditable to keep it active
            let _ = contenteditable.focus();
        },
    ));

    button
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .map_err(|_| JsValue::from_str("Failed to add click event listener"))?;

    closure.forget();
    Ok(())
}

/// Creates the color picker dropdown container
#[cfg(target_arch = "wasm32")]
fn create_color_picker_container(
    document: &web_sys::Document,
    overlay_left: f64,
    overlay_top: f64,
) -> Result<web_sys::HtmlElement, JsValue> {
    // Create color picker dropdown
    let color_picker = document
        .create_element("div")
        .map_err(|_| JsValue::from_str("Cannot create color picker"))?;
    let color_picker: web_sys::HtmlElement = color_picker
        .dyn_into()
        .map_err(|_| JsValue::from_str("Cannot convert color picker to HtmlElement"))?;

    // Style the color picker
    color_picker.set_attribute("class", "color-picker")?;
    let style = color_picker.style();
    // Position directly below the color button (last button in toolbar)
    // Toolbar buttons: 24px each + 2px gap = 26px per button
    // Color button is at position: 3 buttons * 26px = 78px from toolbar left
    let color_button_offset = 78.0; // 3 buttons * (24px + 2px gap) = 78px
    style.set_property("left", &format!("{}px", overlay_left + color_button_offset))?;
    style.set_property("top", &format!("{}px", overlay_top))?; // Directly below toolbar
    style.set_property("display", "none")?; // Hidden by default

    Ok(color_picker)
}

/// Defines the available colors for the color picker
#[cfg(target_arch = "wasm32")]
fn get_available_colors() -> Vec<(&'static str, &'static str)> {
    vec![
        ("#ffff88", "Yellow"),
        ("#add8e6", "Light Blue"),
        ("#ffb6c1", "Light Red"),
        ("#ffb6d9", "Light Pink"),
        ("#d3d3d3", "Light Grey"),
    ]
}

/// Creates a color option button for the color picker
#[cfg(target_arch = "wasm32")]
fn create_color_option(
    document: &web_sys::Document,
    color_hex: &str,
    color_name: &str,
) -> Result<web_sys::HtmlElement, JsValue> {
    let color_option = document
        .create_element("button")
        .map_err(|_| JsValue::from_str("Cannot create color option"))?;
    let color_option: web_sys::HtmlElement = color_option
        .dyn_into()
        .map_err(|_| JsValue::from_str("Cannot convert color option to HtmlElement"))?;

    color_option.set_attribute("class", "color-picker-option")?;
    color_option.set_attribute("title", color_name)?;
    color_option.set_attribute("aria-label", &format!("Set note color to {}", color_name))?;

    let option_style = color_option.style();
    option_style.set_property("background-color", color_hex)?;

    Ok(color_option)
}

/// Adds color option buttons to the color picker
#[cfg(target_arch = "wasm32")]
fn add_color_options_to_picker(
    document: &web_sys::Document,
    color_picker: &web_sys::HtmlElement,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
    contenteditable: &web_sys::HtmlElement,
    color_button: &web_sys::Element,
) -> Result<(), JsValue> {
    let colors = get_available_colors();

    for (color_hex, color_name) in colors {
        let color_option = create_color_option(document, color_hex, color_name)?;

        // Add click handler for color selection
        let state_clone = state.clone();
        let render_clone = render.clone();
        let contenteditable_clone = contenteditable.clone();
        let color_picker_clone = color_picker.clone();
        let color_button_clone = color_button.clone();
        let color_hex_clone = color_hex.to_string();

        let color_closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(
            Box::new(move |event: web_sys::Event| {
                event.prevent_default();
                event.stop_propagation();

                // Update note color
                if let Some(note) = state_clone.borrow_mut().sticky_notes.get_note_mut(note_id) {
                    note.color = color_hex_clone.clone();
                }

                // Update contenteditable background color
                let _ = contenteditable_clone
                    .style()
                    .set_property("background-color", &color_hex_clone);

                // Update color button background color
                if let Some(color_button_element) =
                    color_button_clone.dyn_ref::<web_sys::HtmlElement>()
                {
                    let _ = color_button_element
                        .style()
                        .set_property("background-color", &color_hex_clone);
                }

                // Hide color picker
                let _ = color_picker_clone.style().set_property("display", "none");

                // Trigger render
                render_clone();

                // Focus back to contenteditable
                let _ = contenteditable_clone.focus();
            }),
        );

        color_option
            .add_event_listener_with_callback("click", color_closure.as_ref().unchecked_ref())
            .map_err(|_| JsValue::from_str("Failed to add color click event listener"))?;

        color_closure.forget();

        // Add to color picker
        let _ = color_picker.append_child(&color_option);
    }

    Ok(())
}

/// Sets up the click handler for the color button to show/hide the color picker
#[cfg(target_arch = "wasm32")]
fn setup_color_button_handler(
    color_button: &web_sys::Element,
    color_picker: &web_sys::HtmlElement,
    contenteditable: &web_sys::HtmlElement,
) -> Result<(), JsValue> {
    let color_picker_clone = color_picker.clone();
    let contenteditable_clone = contenteditable.clone();

    let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
        move |event: web_sys::Event| {
            event.prevent_default();
            event.stop_propagation();

            // Show color picker (always show, don't toggle)
            let _ = color_picker_clone.style().set_property("display", "flex");

            // Focus back to contenteditable
            let _ = contenteditable_clone.focus();
        },
    ));

    color_button
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .map_err(|_| JsValue::from_str("Failed to add color button click event listener"))?;

    closure.forget();

    Ok(())
}

/// Adds a click handler to the color button that shows/hides a color picker.
///
/// # Arguments
/// * `color_button` - The color button element
/// * `document` - The browser document object
/// * `contenteditable` - The contenteditable element being edited
/// * `state` - Application state for updating note colors
/// * `note_id` - ID of the note being edited
/// * `render` - Render callback to update the display
/// * `overlay_left` - Left position of the overlay
/// * `overlay_top` - Top position of the overlay
/// * `current_color` - Current background color of the note
///
/// # Returns
/// The created color picker element
#[cfg(target_arch = "wasm32")]
fn add_color_picker_handler(
    color_button: &web_sys::Element,
    document: &web_sys::Document,
    contenteditable: &web_sys::HtmlElement,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
    overlay_left: f64,
    overlay_top: f64,
    _current_color: &str,
) -> Result<web_sys::HtmlElement, JsValue> {
    // Create color picker container
    let color_picker = create_color_picker_container(document, overlay_left, overlay_top)?;

    // Add color options to the picker
    add_color_options_to_picker(
        document,
        &color_picker,
        state,
        note_id,
        render,
        contenteditable,
        color_button,
    )?;

    // Add color picker to document body
    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("Cannot get document body"))?;
    let _ = body.append_child(&color_picker);

    // Set up color button handler
    setup_color_button_handler(color_button, &color_picker, contenteditable)?;

    Ok(color_picker)
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
/// Calculates the screen position for the text input overlay
fn calculate_overlay_position(
    canvas: &HtmlCanvasElement,
    note: &crate::sticky_notes::StickyNote,
    state: &Rc<RefCell<crate::AppState>>,
) -> Result<(f64, f64, f64, f64, f64, f64), JsValue> {
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

    Ok((
        overlay_left,
        overlay_top,
        screen_width,
        screen_height,
        viewport_width,
        viewport_height,
    ))
}

#[cfg(target_arch = "wasm32")]
/// Creates and configures the contenteditable element for text input
fn create_contenteditable_element(
    document: &web_sys::Document,
    overlay_left: f64,
    overlay_top: f64,
    screen_width: f64,
    screen_height: f64,
    note: &crate::sticky_notes::StickyNote,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
) -> Result<
    (
        web_sys::HtmlElement,
        web_sys::HtmlElement,
        web_sys::HtmlElement,
    ),
    JsValue,
> {
    // Create contenteditable div element for seamless text editing
    let contenteditable = document
        .create_element("div")
        .map_err(|_| JsValue::from_str("Cannot create contenteditable element"))?;

    let contenteditable: web_sys::HtmlElement = contenteditable
        .dyn_into()
        .map_err(|_| JsValue::from_str("Cannot convert to HtmlElement"))?;

    // Set contenteditable attribute
    contenteditable.set_attribute("contenteditable", "true")?;

    // Create formatting toolbar
    let (toolbar, color_picker) = create_formatting_toolbar(
        document,
        &contenteditable,
        overlay_left,
        overlay_top,
        screen_width,
        state,
        note_id,
        render,
        &note.color,
    )?;

    // Style the contenteditable div with centralized styling function
    crate::styling::components::style_contenteditable_overlay(
        &contenteditable,
        overlay_left,
        overlay_top,
        screen_width,
        screen_height,
        &note.color,
    )?;

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
    crate::styling::sizing::update_contenteditable_height(
        &contenteditable,
        screen_height,
        screen_height,
    )?;

    contenteditable.focus()?;

    Ok((contenteditable, toolbar, color_picker))
}

#[cfg(target_arch = "wasm32")]
/// Sets up all event listeners for the text input overlay
fn setup_overlay_events(
    contenteditable: &web_sys::HtmlElement,
    toolbar: &web_sys::HtmlElement,
    color_picker: &web_sys::HtmlElement,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
    screen_height: f64,
    original_content: &str,
    document: &web_sys::Document,
) -> Result<(), JsValue> {
    // Attach input event listener to handle text changes
    let on_input = setup_input_event(contenteditable, state, note_id, render, screen_height);
    contenteditable
        .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach input event listener");
        });
    on_input.forget();

    // Attach keydown event listener for Enter/Escape handling
    let on_keydown = setup_keydown_event(
        contenteditable,
        toolbar,
        color_picker,
        state,
        note_id,
        render,
        original_content,
        document,
    );
    contenteditable
        .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach keydown event listener");
        });
    on_keydown.forget();

    // Mousedown handlers are now set up in setup_blur_event

    // Attach blur event listener for clicking outside
    let on_blur = setup_blur_event(contenteditable, toolbar, color_picker, document, note_id);
    contenteditable
        .add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach blur event listener");
        });
    on_blur.forget();

    // Attach paste event listener to sanitize pasted content
    let on_paste = setup_paste_event();
    contenteditable
        .add_event_listener_with_callback("paste", on_paste.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach paste event listener");
        });
    on_paste.forget();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
/// Sets up the input event listener for text changes
fn setup_input_event(
    contenteditable: &web_sys::HtmlElement,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
    screen_height: f64,
) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)> {
    let state = state.clone();
    let render = render.clone();
    let contenteditable = contenteditable.clone();

    wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
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
                let _ = crate::styling::sizing::update_contenteditable_height(
                    &contenteditable,
                    scroll_height,
                    screen_height,
                );

                // Adjust note height based on contenteditable height in world coordinates
                let min_height = DEFAULT_NOTE_HEIGHT; // Minimum note height
                let new_height = (scroll_height / zoom).max(min_height);
                note.height = new_height;
            }

            // Re-render the canvas to show the updated text
            render();
        },
    ))
}

#[cfg(target_arch = "wasm32")]
/// Sets up the keydown event listener for keyboard shortcuts
fn setup_keydown_event(
    contenteditable: &web_sys::HtmlElement,
    toolbar: &web_sys::HtmlElement,
    color_picker: &web_sys::HtmlElement,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
    original_content: &str,
    document: &web_sys::Document,
) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::KeyboardEvent)> {
    let state = state.clone();
    let render = render.clone();
    let contenteditable = contenteditable.clone();
    let toolbar = toolbar.clone();
    let color_picker = color_picker.clone();
    let document = document.clone();
    let original_content = original_content.to_string();

    wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new(
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
                    // Confirm changes on Enter (for test compatibility) or Ctrl/Shift+Enter
                    // Confirm changes - content already updated via input handler
                    crate::logging::log_info(&format!(
                        "Text editing confirmed for note {}",
                        note_id
                    ));

                    // Remove the contenteditable overlay
                    if let Some(body) = document.body() {
                        let _ = body.remove_child(&toolbar);
                        let _ = body.remove_child(&contenteditable);
                        let _ = body.remove_child(&color_picker);
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
                        let _ = body.remove_child(&color_picker);
                    }

                    // Re-render to show restored content
                    render();
                }
                _ => {
                    // Allow other keys to be handled normally by the contenteditable
                }
            }
        },
    ))
}

#[cfg(target_arch = "wasm32")]
/// Sets up the blur event listener for clicking outside the overlay
fn setup_blur_event(
    contenteditable: &web_sys::HtmlElement,
    toolbar: &web_sys::HtmlElement,
    color_picker: &web_sys::HtmlElement,
    document: &web_sys::Document,
    note_id: u32,
) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)> {
    let document = document.clone();
    let contenteditable = contenteditable.clone();
    let toolbar = toolbar.clone();
    let color_picker = color_picker.clone();

    // Shared flag to track if toolbar/color picker was clicked
    let overlay_clicked = Rc::new(RefCell::new(false));

    // Set up mousedown handlers to track clicks on overlay elements
    let overlay_clicked_clone = overlay_clicked.clone();
    let toolbar_mousedown = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(
        Box::new(move |_event: web_sys::Event| {
            *overlay_clicked_clone.borrow_mut() = true;
        }),
    );
    toolbar
        .add_event_listener_with_callback("mousedown", toolbar_mousedown.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach toolbar mousedown event listener");
        });

    let overlay_clicked_clone2 = overlay_clicked.clone();
    let color_picker_mousedown = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(
        Box::new(move |_event: web_sys::Event| {
            *overlay_clicked_clone2.borrow_mut() = true;
        }),
    );
    color_picker
        .add_event_listener_with_callback(
            "mousedown",
            color_picker_mousedown.as_ref().unchecked_ref(),
        )
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach color picker mousedown event listener");
        });

    toolbar_mousedown.forget();
    color_picker_mousedown.forget();

    wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
        move |_event: web_sys::Event| {
            // Check if toolbar or color picker was clicked - if so, don't remove overlay
            if *overlay_clicked.borrow() {
                *overlay_clicked.borrow_mut() = false; // Reset flag
                return; // Don't remove overlay
            }

            // Confirm changes when focus is lost (clicked outside)
            crate::logging::log_info(&format!(
                "Text editing confirmed (blur) for note {}",
                note_id
            ));

            // Remove the contenteditable overlay
            if let Some(body) = document.body() {
                let _ = body.remove_child(&toolbar);
                let _ = body.remove_child(&contenteditable);
                let _ = body.remove_child(&color_picker);
            }
        },
    ))
}

#[cfg(target_arch = "wasm32")]
fn insert_sanitized_text(document: &web_sys::Document, text: &str) -> Result<(), JsValue> {
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
        &wasm_bindgen::JsValue::from_str(text),
    );

    crate::logging::log_info("Pasted content sanitized and inserted");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
/// Sets up the paste event listener to sanitize pasted content
/// Extracts text content from clipboard data
#[cfg(target_arch = "wasm32")]
fn extract_clipboard_text(clipboard_data: &wasm_bindgen::JsValue) -> Result<String, JsValue> {
    // Try to get plain text first
    if let Ok(text) = js_sys::Reflect::get(clipboard_data, &"getData".into()) {
        if let Ok(get_data_fn) = text.dyn_into::<js_sys::Function>() {
            if let Ok(text_result) = get_data_fn.call1(clipboard_data, &"text/plain".into()) {
                if let Some(text_str) = text_result.as_string() {
                    if !text_str.is_empty() {
                        return Ok(text_str);
                    }
                }
            }

            // Fallback to HTML content and sanitize it
            if let Ok(html_result) = get_data_fn.call1(clipboard_data, &"text/html".into()) {
                if let Some(html_str) = html_result.as_string() {
                    return Ok(sanitize_html_to_text(&html_str));
                }
            }
        }
    }

    Ok(String::new())
}

#[cfg(target_arch = "wasm32")]
fn setup_paste_event() -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)> {
    wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
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

            // Extract text from clipboard
            let pasted_text = match extract_clipboard_text(&clipboard_data) {
                Ok(text) => text,
                Err(_) => {
                    crate::logging::log_warn("Failed to extract clipboard text");
                    return;
                }
            };

            if !pasted_text.is_empty() {
                // Get document and insert sanitized text
                let document = match web_sys::window().and_then(|w| w.document()) {
                    Some(doc) => doc,
                    None => {
                        crate::logging::log_warn("Document not available for paste insertion");
                        return;
                    }
                };

                let _ = insert_sanitized_text(&document, &pasted_text);
            }
        },
    ))
}

#[cfg(target_arch = "wasm32")]
/// Adds the overlay elements to the document body
fn add_overlay_to_document(
    document: &web_sys::Document,
    toolbar: &web_sys::HtmlElement,
    contenteditable: &web_sys::HtmlElement,
    color_picker: &web_sys::HtmlElement,
) -> Result<(), JsValue> {
    if let Some(body) = document.body() {
        body.append_child(toolbar)?;
        body.append_child(contenteditable)?;
        body.append_child(color_picker)?;
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn create_text_input_overlay(
    canvas: &HtmlCanvasElement,
    state: &Rc<RefCell<crate::AppState>>,
    note_id: u32,
    render: &Rc<dyn Fn()>,
) -> Result<(), JsValue> {
    let browser_window = match web_sys::window() {
        Some(w) => w,
        None => {
            crate::logging::log_warn("Cannot create text input overlay: window unavailable");
            return Err(JsValue::from_str("Window unavailable"));
        }
    };

    let document = match browser_window.document() {
        Some(d) => d,
        None => {
            crate::logging::log_warn("Cannot create text input overlay: document unavailable");
            return Err(JsValue::from_str("Document unavailable"));
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
            return Err(JsValue::from_str("Note not found"));
        }
    };

    // Calculate overlay position
    let (overlay_left, overlay_top, screen_width, screen_height, _viewport_width, _viewport_height) =
        calculate_overlay_position(canvas, &note, state)?;

    // Create contenteditable element
    let (contenteditable, toolbar, color_picker) = create_contenteditable_element(
        &document,
        overlay_left,
        overlay_top,
        screen_width,
        screen_height,
        &note,
        state,
        note_id,
        render,
    )?;

    // Store original content for potential cancellation
    let original_content = note.content.clone();

    // Set up all event listeners
    setup_overlay_events(
        &contenteditable,
        &toolbar,
        &color_picker,
        state,
        note_id,
        render,
        screen_height,
        &original_content,
        &document,
    )?;

    // Add elements to document
    add_overlay_to_document(&document, &toolbar, &contenteditable, &color_picker)?;

    crate::logging::log_info(&format!("Created text input overlay for note {}", note_id));
    Ok(())
}
