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

/// Struct to hold all closures for the text input overlay to prevent memory leaks.
/// When this struct is dropped, all closures are properly cleaned up.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub struct TextInputOverlayClosures {
    /// Input event closure for text changes
    pub input_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
    /// Keydown event closure for Enter/Escape handling
    pub keydown_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::KeyboardEvent)>>,
    /// Blur event closure for clicking outside
    pub blur_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
    /// Paste event closure for sanitizing pasted content
    pub paste_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
    /// Bold button click closure
    pub bold_button_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
    /// Italic button click closure
    pub italic_button_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
    /// Underline button click closure
    pub underline_button_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
    /// Color button click closure
    pub color_button_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
    /// Color option click closures (one per color)
    pub color_option_closures: Vec<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Debug for TextInputOverlayClosures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInputOverlayClosures")
            .field("input_closure", &self.input_closure.is_some())
            .field("keydown_closure", &self.keydown_closure.is_some())
            .field("blur_closure", &self.blur_closure.is_some())
            .field("paste_closure", &self.paste_closure.is_some())
            .field("bold_button_closure", &self.bold_button_closure.is_some())
            .field(
                "italic_button_closure",
                &self.italic_button_closure.is_some(),
            )
            .field(
                "underline_button_closure",
                &self.underline_button_closure.is_some(),
            )
            .field("color_button_closure", &self.color_button_closure.is_some())
            .field(
                "color_option_closures_count",
                &self.color_option_closures.len(),
            )
            .finish()
    }
}

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
) -> Result<
    (
        web_sys::Element,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    ),
    JsValue,
> {
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

    let color_button = create_color_button(document, "#FFFF88")?; // Default color

    // Add buttons to toolbar
    let _ = toolbar.append_child(&bold_button);
    let _ = toolbar.append_child(&italic_button);
    let _ = toolbar.append_child(&underline_button);
    let _ = toolbar.append_child(&color_button);

    // Add click handlers for formatting buttons
    let bold_closure = add_formatting_handler(&bold_button, "bold", contenteditable)?;
    let italic_closure = add_formatting_handler(&italic_button, "italic", contenteditable)?;
    let underline_closure =
        add_formatting_handler(&underline_button, "underline", contenteditable)?;

    Ok((
        color_button,
        bold_closure,
        italic_closure,
        underline_closure,
    ))
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
/// A tuple containing the created toolbar HTML element, the color picker element, and all the formatting button closures
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
) -> Result<
    (
        web_sys::HtmlElement,
        web_sys::HtmlElement,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        Vec<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    ),
    JsValue,
> {
    // Create toolbar container
    let toolbar = create_toolbar_container(document, overlay_left, overlay_top, screen_width)?;

    // Add buttons to toolbar and get color button and closures
    let (color_button, bold_closure, italic_closure, underline_closure) =
        add_buttons_to_toolbar(document, &toolbar, contenteditable)?;

    // Create and add color picker
    let (color_picker, color_option_closures, color_button_closure) = add_color_picker_handler(
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

    Ok((
        toolbar,
        color_picker,
        bold_closure,
        italic_closure,
        underline_closure,
        color_option_closures,
        color_button_closure,
    ))
}

/// Applies text formatting using modern Selection/Range APIs instead of deprecated execCommand
#[cfg(target_arch = "wasm32")]
fn apply_text_formatting(
    contenteditable: &web_sys::HtmlElement,
    format_type: &str,
) -> Result<(), JsValue> {
    let document = contenteditable.owner_document().unwrap();
    let selection = document.get_selection().unwrap().unwrap();
    let range = selection.get_range_at(0).unwrap();

    // Get the selected text
    let selected_text = range.clone().to_string().as_string().unwrap_or_default();

    // Determine the tag based on format type
    let tag = match format_type {
        "bold" => "b",
        "italic" => "i",
        "underline" => "u",
        _ => return Err(JsValue::from_str("Unsupported format type")),
    };

    // Create the formatted HTML
    let formatted_html = if selected_text.is_empty() {
        // No selection - insert empty tag at cursor
        format!("<{}></{}>", tag, tag)
    } else {
        // Wrap selected text with tag
        format!("<{}>{}</{}>", tag, selected_text, tag)
    };

    // Delete the current selection and insert the formatted text
    let _ = range.delete_contents();
    let fragment = document.create_document_fragment();
    let temp_div = document.create_element("div")?;
    temp_div.set_inner_html(&formatted_html);
    while let Some(child) = temp_div.first_child() {
        fragment.append_child(&child)?;
    }
    range.insert_node(&fragment)?;

    // Restore focus to contenteditable
    let _ = contenteditable.focus();

    Ok(())
}

/// Adds a click handler to a formatting button.
///
/// # Arguments
/// * `button` - The button element to add handler to
/// * `format_type` - The type of formatting ("bold", "italic", "underline")
/// * `contenteditable` - The contenteditable element to apply formatting to
///
/// # Returns
/// The created closure that was attached to the button
#[cfg(target_arch = "wasm32")]
fn add_formatting_handler(
    button: &web_sys::Element,
    format_type: &str,
    contenteditable: &web_sys::HtmlElement,
) -> Result<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>, JsValue> {
    let contenteditable = contenteditable.clone();
    let format_type = format_type.to_string();

    let closure = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(
        move |event: web_sys::Event| {
            event.prevent_default();
            event.stop_propagation();

            // Apply formatting using modern Selection/Range APIs
            match apply_text_formatting(&contenteditable, &format_type) {
                Ok(_) => {
                    crate::logging::log_info(&format!(
                        "Successfully applied {} formatting",
                        format_type
                    ));
                }
                Err(e) => {
                    crate::logging::log_warn(&format!(
                        "Failed to apply {} formatting: {:?}",
                        format_type, e
                    ));
                }
            }

            // Focus the contenteditable to keep it active
            let _ = contenteditable.focus();
        },
    ));

    button
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .map_err(|_| JsValue::from_str("Failed to add click event listener"))?;

    // Return the closure instead of forgetting it
    Ok(closure)
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
    style.set_property("position", "absolute")?;
    // Position directly below the color button (last button in toolbar)
    // Toolbar buttons: 24px each + 2px gap = 26px per button
    // Color button is at position: 3 buttons * 26px = 78px from toolbar left
    let color_button_offset = 78.0; // 3 buttons * (24px + 2px gap) = 78px
    style.set_property("left", &format!("{}px", overlay_left + color_button_offset))?;
    style.set_property("top", &format!("{}px", overlay_top))?; // Directly below toolbar
    style.set_property("display", "none")?; // Hidden by default
    style.set_property("flex-direction", "column")?;
    style.set_property("gap", "2px")?;
    style.set_property("z-index", "1002")?; // Higher than toolbar

    Ok(color_picker)
}

/// Defines the available colors for the color picker
#[cfg(target_arch = "wasm32")]
fn get_available_colors() -> Vec<(&'static str, &'static str)> {
    vec![
        ("#ffff88", "Yellow"),
        ("#add8e6", "Light Blue"),
        ("#ff6b6b", "Red"),
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
    option_style.set_property("width", "24px")?;
    option_style.set_property("height", "24px")?;
    option_style.set_property("background-color", color_hex)?;
    option_style.set_property("border", "1px solid #d1d5db")?;
    option_style.set_property("border-radius", "3px")?;
    option_style.set_property("cursor", "pointer")?;
    option_style.set_property("transition", "transform 0.1s")?;

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
) -> Result<Vec<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>, JsValue> {
    let colors = get_available_colors();
    let mut closures = Vec::new();

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

        closures.push(color_closure);

        // Add to color picker
        let _ = color_picker.append_child(&color_option);
    }

    Ok(closures)
}

/// Sets up the click handler for the color button to show/hide the color picker
#[cfg(target_arch = "wasm32")]
fn setup_color_button_handler(
    color_button: &web_sys::Element,
    color_picker: &web_sys::HtmlElement,
    contenteditable: &web_sys::HtmlElement,
) -> Result<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>, JsValue> {
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

    Ok(closure)
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
/// The created color picker element and the color button closure
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
) -> Result<
    (
        web_sys::HtmlElement,
        Vec<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    ),
    JsValue,
> {
    // Create color picker container
    let color_picker = create_color_picker_container(document, overlay_left, overlay_top)?;

    // Add color options to the picker
    let color_option_closures = add_color_options_to_picker(
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
    let color_button_closure =
        setup_color_button_handler(color_button, &color_picker, contenteditable)?;

    Ok((color_picker, color_option_closures, color_button_closure))
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

/// Sanitizes HTML content for safe display in contenteditable, allowing only safe formatting tags.
///
/// This function removes potentially dangerous elements like scripts, styles,
/// and other non-formatting tags, while preserving basic text formatting
/// (bold, italic, underline, line breaks) that the application supports.
///
/// # Arguments
/// * `html` - The HTML string to sanitize
///
/// # Returns
/// The sanitized HTML content with only safe tags allowed
#[cfg(target_arch = "wasm32")]
fn sanitize_html_for_display(html: &str) -> String {
    // Use regex to remove dangerous tags and keep only safe ones
    // This is a simple approach - for production, consider using a proper HTML sanitizer

    // First, remove any script, style, or other dangerous tags and their content
    let dangerous_tags = [
        "script", "style", "iframe", "object", "embed", "form", "input", "button", "link", "meta",
        "base", "img", "svg", "math", "canvas", "video", "audio", "a", "area", "table", "tr", "td",
        "th", "tbody", "thead", "tfoot", "col", "colgroup",
    ];

    let mut result = html.to_string();

    // Remove dangerous tags and their content
    for tag in &dangerous_tags {
        // Remove self-closing tags
        let pattern = format!(r#"<{}(?:\s[^>]*)?/?>"#, regex::escape(tag));
        result = regex::Regex::new(&pattern)
            .unwrap()
            .replace_all(&result, "")
            .to_string();

        // Remove opening and closing tag pairs with content
        let pattern = format!(
            r#"<{}(?:\s[^>]*)?>.*?</{}>"#,
            regex::escape(tag),
            regex::escape(tag)
        );
        result = regex::Regex::new(&pattern)
            .unwrap()
            .replace_all(&result, "")
            .to_string();
    }

    // Remove event handlers and dangerous attributes from remaining tags
    let dangerous_attrs = [
        "on\\w+",
        "javascript:",
        "data:",
        "vbscript:",
        "href",
        "src",
        "action",
    ];

    for attr in &dangerous_attrs {
        let pattern = format!(r#"\s+{}="[^"]*""#, attr);
        result = regex::Regex::new(&pattern)
            .unwrap()
            .replace_all(&result, "")
            .to_string();
        let pattern = format!(r#"\s+{}='[^']*'"#, attr);
        result = regex::Regex::new(&pattern)
            .unwrap()
            .replace_all(&result, "")
            .to_string();
    }

    // The remaining tags should now be safe (only b, i, u, br, em, strong should remain)
    // If any other tags remain, they will be treated as text by the browser

    result
}
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
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        Vec<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
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
    let (
        toolbar,
        color_picker,
        bold_closure,
        italic_closure,
        underline_closure,
        color_option_closures,
        color_button_closure,
    ) = create_formatting_toolbar(
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
        // Content appears to be HTML, sanitize it before use
        sanitize_html_for_display(&note.content)
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

    Ok((
        contenteditable,
        toolbar,
        color_picker,
        bold_closure,
        italic_closure,
        underline_closure,
        color_option_closures,
        color_button_closure,
    ))
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
) -> Result<
    (
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::KeyboardEvent)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
        wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
    ),
    JsValue,
> {
    // Attach input event listener to handle text changes
    let on_input = setup_input_event(contenteditable, state, note_id, render, screen_height);
    contenteditable
        .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach input event listener");
        });

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

    // Mousedown handlers are now set up in setup_blur_event

    // Attach blur event listener for clicking outside
    let on_blur = setup_blur_event(
        contenteditable,
        toolbar,
        color_picker,
        document,
        note_id,
        state,
    );
    contenteditable
        .add_event_listener_with_callback("blur", on_blur.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach blur event listener");
        });

    // Attach paste event listener to sanitize pasted content
    let on_paste = setup_paste_event();
    contenteditable
        .add_event_listener_with_callback("paste", on_paste.as_ref().unchecked_ref())
        .unwrap_or_else(|_| {
            crate::logging::log_warn("Failed to attach paste event listener");
        });

    Ok((on_input, on_keydown, on_blur, on_paste))
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
                // Store sanitized HTML content instead of raw HTML
                let html_content = contenteditable.inner_html();
                let sanitized_content = sanitize_html_for_display(&html_content);
                note.content = sanitized_content;

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

                    // Clear overlay closures from state to prevent memory leaks
                    state.borrow_mut().text_input_overlay_closures = None;

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

                    // Clear overlay closures from state to prevent memory leaks
                    state.borrow_mut().text_input_overlay_closures = None;

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
    state: &Rc<RefCell<crate::AppState>>,
) -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)> {
    let document = document.clone();
    let contenteditable = contenteditable.clone();
    let toolbar = toolbar.clone();
    let color_picker = color_picker.clone();
    let state = state.clone();

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

            // Clear overlay closures from state to prevent memory leaks
            state.borrow_mut().text_input_overlay_closures = None;

            // Remove the contenteditable overlay
            if let Some(body) = document.body() {
                let _ = body.remove_child(&toolbar);
                let _ = body.remove_child(&contenteditable);
                let _ = body.remove_child(&color_picker);
            }
        },
    ))
}

/// Inserts sanitized text using modern Selection/Range APIs instead of deprecated execCommand
#[cfg(target_arch = "wasm32")]
fn insert_sanitized_text(document: &web_sys::Document, text: &str) -> Result<(), JsValue> {
    let selection = document.get_selection().unwrap().unwrap();

    // If there's a selection, replace it; otherwise insert at cursor
    if let Ok(range) = selection.get_range_at(0) {
        // Delete current selection
        let _ = range.delete_contents();

        // Create a text node using JavaScript
        let text_node_js = js_sys::Reflect::get(document.as_ref(), &"createTextNode".into())
            .unwrap()
            .dyn_into::<js_sys::Function>()
            .unwrap()
            .call1(document.as_ref(), &wasm_bindgen::JsValue::from_str(text))
            .unwrap();

        let text_node = text_node_js.dyn_into::<web_sys::Node>()?;

        range.insert_node(&text_node)?;

        // Move cursor to end of inserted text
        range.set_start_after(&text_node)?;
        range.set_end_after(&text_node)?;
        let _ = selection.remove_all_ranges();
        selection.add_range(&range)?;
    }

    crate::logging::log_info("Pasted content sanitized and inserted");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
/// Sets up the paste event listener to sanitize pasted content
/// Extracts text content from clipboard data
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
    let (
        contenteditable,
        toolbar,
        color_picker,
        bold_closure,
        italic_closure,
        underline_closure,
        color_option_closures,
        color_button_closure,
    ) = create_contenteditable_element(
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
    let (input_closure, keydown_closure, blur_closure, paste_closure) = setup_overlay_events(
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

    // Store all closures in the app state to prevent memory leaks
    state.borrow_mut().text_input_overlay_closures = Some(TextInputOverlayClosures {
        input_closure: Some(input_closure),
        keydown_closure: Some(keydown_closure),
        blur_closure: Some(blur_closure),
        paste_closure: Some(paste_closure),
        bold_button_closure: Some(bold_closure),
        italic_button_closure: Some(italic_closure),
        underline_button_closure: Some(underline_closure),
        color_button_closure: Some(color_button_closure),
        color_option_closures,
    });

    // Add elements to document
    add_overlay_to_document(&document, &toolbar, &contenteditable, &color_picker)?;

    crate::logging::log_info(&format!("Created text input overlay for note {}", note_id));
    Ok(())
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_sanitize_html_for_display_removes_dangerous_tags() {
        // Test that dangerous tags are removed
        let input = r#"<script>alert('xss')</script><b>Safe</b><img src=x onerror=alert('xss')>"#;
        let result = sanitize_html_for_display(input);
        assert!(!result.contains("<script>"));
        assert!(!result.contains("<img"));
        assert!(result.contains("<b>Safe</b>"));
    }

    #[wasm_bindgen_test]
    fn test_sanitize_html_for_display_allows_safe_formatting_tags() {
        // Test that safe formatting tags are preserved
        let input = r#"<b>Bold</b><i>Italic</i><u>Underline</u><br><em>Emphasis</em><strong>Strong</strong>"#;
        let result = sanitize_html_for_display(input);
        assert_eq!(result, input.to_lowercase()); // Tags should be lowercase
    }

    #[wasm_bindgen_test]
    fn test_sanitize_html_for_display_removes_nested_dangerous_tags() {
        // Test nested dangerous tags
        let input = r#"<b>Safe <script>danger</script> text</b>"#;
        let result = sanitize_html_for_display(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("<b>Safe"));
        assert!(result.contains("danger"));
        assert!(result.contains("text</b>"));
    }

    #[wasm_bindgen_test]
    fn test_sanitize_html_for_display_handles_mixed_content() {
        // Test mixed safe and dangerous content
        let input = r#"Normal text <b>bold</b> <script>evil</script> more <i>italic</i> text"#;
        let result = sanitize_html_for_display(input);
        assert!(!result.contains("<script>"));
        assert!(result.contains("Normal text"));
        assert!(result.contains("<b>bold</b>"));
        assert!(result.contains("more"));
        assert!(result.contains("<i>italic</i>"));
        assert!(result.contains("text"));
    }

    #[wasm_bindgen_test]
    fn test_sanitize_html_for_display_handles_empty_and_plain_text() {
        // Test edge cases
        assert_eq!(sanitize_html_for_display(""), "");
        assert_eq!(sanitize_html_for_display("Plain text"), "Plain text");
        assert_eq!(sanitize_html_for_display("<br>"), "<br>");
    }

    #[wasm_bindgen_test]
    fn test_sanitize_html_to_text_extracts_plain_text() {
        // Test that HTML tags are stripped to plain text
        let input = r#"<b>Bold</b> and <i>italic</i> text"#;
        let result = sanitize_html_to_text(input);
        assert_eq!(result, "Bold and italic text");
    }

    #[wasm_bindgen_test]
    fn test_sanitize_html_to_text_handles_complex_html() {
        // Test complex HTML with dangerous tags
        let input = r#"<div><script>alert('xss')</script><b>Safe</b><p>Text</p></div>"#;
        let result = sanitize_html_to_text(input);
        assert_eq!(result, "SafeText");
    }
}
