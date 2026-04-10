#[cfg(target_arch = "wasm32")]
use crate::toolbar::TOOLBAR_EDGE_PADDING;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, window};

#[cfg(target_arch = "wasm32")]
const FALLBACK_VIEWPORT_WIDTH: f64 = 1280.0;
#[cfg(target_arch = "wasm32")]
const FALLBACK_VIEWPORT_HEIGHT: f64 = 840.0;
#[cfg(target_arch = "wasm32")]
const CANVAS_HORIZONTAL_MARGIN: f64 = 32.0;
#[cfg(target_arch = "wasm32")]
const CANVAS_VERTICAL_MARGIN: f64 = 96.0;
#[cfg(target_arch = "wasm32")]
const MIN_CANVAS_EDGE: f64 = 320.0;
#[cfg(target_arch = "wasm32")]
const GRID_BASE_SPACING: f64 = 48.0;
#[cfg(target_arch = "wasm32")]
const GRID_MIN_SPACING: f64 = 24.0;
#[cfg(target_arch = "wasm32")]
const GRID_MAX_SPACING: f64 = 120.0;
#[cfg(target_arch = "wasm32")]
const STATUS_HELP_TEXT: &str = "Drag to pan, scroll to zoom, or use the arrow keys and +/-.";

#[cfg(target_arch = "wasm32")]
pub fn canvas_css_size(canvas: &HtmlCanvasElement) -> Result<(f64, f64), JsValue> {
    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let viewport_width = browser_window
        .inner_width()?
        .as_f64()
        .unwrap_or(FALLBACK_VIEWPORT_WIDTH);
    let viewport_height = browser_window
        .inner_height()?
        .as_f64()
        .unwrap_or(FALLBACK_VIEWPORT_HEIGHT);

    let width = match canvas.client_width() {
        0 => (viewport_width - CANVAS_HORIZONTAL_MARGIN).max(MIN_CANVAS_EDGE),
        value => f64::from(value),
    };
    let height = match canvas.client_height() {
        0 => (viewport_height - CANVAS_VERTICAL_MARGIN).max(MIN_CANVAS_EDGE),
        value => f64::from(value),
    };

    Ok((width, height))
}

#[cfg(target_arch = "wasm32")]
pub fn resize_canvas(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
) -> Result<(), JsValue> {
    let browser_window = window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
    let (width, height) = canvas_css_size(canvas)?;
    // Keep CSS size stable while allocating a denser backing store for Retina/HiDPI displays.
    let device_pixel_ratio = browser_window.device_pixel_ratio().max(1.0);

    canvas
        .style()
        .set_property("width", &format!("{}px", width.round()))?;
    canvas
        .style()
        .set_property("height", &format!("{}px", height.round()))?;
    canvas.set_width((width * device_pixel_ratio).round() as u32);
    canvas.set_height((height * device_pixel_ratio).round() as u32);

    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    ctx.scale(device_pixel_ratio, device_pixel_ratio)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn render_canvas(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    status: &HtmlElement,
    state: &crate::AppState,
) -> Result<(), JsValue> {
    let (width, height) = canvas_css_size(canvas)?;
    let zoom = state.viewport.zoom;
    let grid_spacing = (GRID_BASE_SPACING * zoom).clamp(GRID_MIN_SPACING, GRID_MAX_SPACING);
    let offset_x = state.viewport.pan_x.rem_euclid(grid_spacing);
    let offset_y = state.viewport.pan_y.rem_euclid(grid_spacing);

    ctx.set_fill_style_str("#f8fafc");
    ctx.fill_rect(0.0, 0.0, width, height);

    ctx.begin_path();
    ctx.set_stroke_style_str("#d7e3f4");
    ctx.set_line_width(1.0);

    let mut x = offset_x - grid_spacing;
    while x <= width + grid_spacing {
        ctx.move_to(x, 0.0);
        ctx.line_to(x, height);
        x += grid_spacing;
    }

    let mut y = offset_y - grid_spacing;
    while y <= height + grid_spacing {
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
        y += grid_spacing;
    }
    ctx.stroke();

    // Render sticky notes
    for note in &state.sticky_notes.notes {
        // Calculate screen position from world coordinates
        // This matches the center cross positioning: screen = world * zoom + center + pan
        let screen_x = note.x * state.viewport.zoom + width / 2.0 + state.viewport.pan_x;
        let screen_y = note.y * state.viewport.zoom + height / 2.0 + state.viewport.pan_y;
        let screen_width = note.width * state.viewport.zoom;
        let screen_height = note.height * state.viewport.zoom;

        // Draw note background
        ctx.set_fill_style_str(&note.color);
        ctx.fill_rect(screen_x, screen_y, screen_width, screen_height);

        // Draw note border
        ctx.set_stroke_style_str(if Some(note.id) == state.sticky_notes.selected_note_id {
            "#2563eb" // Blue border for selected notes
        } else {
            "#374151" // Gray border for unselected notes
        });
        ctx.set_line_width(2.0);
        ctx.stroke_rect(screen_x, screen_y, screen_width, screen_height);

        // Draw note content text (placeholder for now)
        if !note.content.is_empty() {
            ctx.set_fill_style_str("#000000");
            ctx.set_font("14px Inter, sans-serif");
            ctx.set_text_align("left");
            ctx.set_text_baseline("top");
            // Add some padding
            let text_x = screen_x + 8.0;
            let text_y = screen_y + 8.0;
            // Simple text rendering (no wrapping for now)
            let mut y_offset = 0.0;
            for line in note.content.lines() {
                if !line.is_empty() {
                    ctx.fill_text(line, text_x, text_y + y_offset)?;
                    y_offset += 18.0; // Line height
                }
            }
        }
    }

    let center_x = (width / 2.0) + state.viewport.pan_x;
    let center_y = (height / 2.0) + state.viewport.pan_y;

    ctx.begin_path();
    ctx.set_stroke_style_str("#2563eb");
    ctx.set_line_width(2.0);
    ctx.move_to(center_x - (20.0 * zoom), center_y);
    ctx.line_to(center_x + (20.0 * zoom), center_y);
    ctx.move_to(center_x, center_y - (20.0 * zoom));
    ctx.line_to(center_x, center_y + (20.0 * zoom));
    ctx.stroke();

    canvas.set_attribute("data-ready", "true")?;
    canvas.set_attribute("data-pan-x", &format!("{:.2}", state.viewport.pan_x))?;
    canvas.set_attribute("data-pan-y", &format!("{:.2}", state.viewport.pan_y))?;
    canvas.set_attribute("data-zoom", &format!("{:.2}", state.viewport.zoom))?;
    canvas.style().set_property(
        "cursor",
        if state.viewport.is_dragging {
            "grabbing"
        } else {
            "grab"
        },
    )?;

    status.set_text_content(Some(&format!(
        "Pan ({:.0}, {:.0}) · Zoom {:.2}× · {}",
        state.viewport.pan_x, state.viewport.pan_y, state.viewport.zoom, STATUS_HELP_TEXT
    )));

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn sync_toolbar_position(
    toolbar: &HtmlElement,
    workspace: &HtmlElement,
    state: &mut crate::toolbar::FloatingToolbarState,
) -> Result<(), JsValue> {
    let max_x = (f64::from(workspace.client_width() - toolbar.offset_width())
        - TOOLBAR_EDGE_PADDING)
        .max(TOOLBAR_EDGE_PADDING);
    let max_y = (f64::from(workspace.client_height() - toolbar.offset_height())
        - TOOLBAR_EDGE_PADDING)
        .max(TOOLBAR_EDGE_PADDING);

    state.clamp_within(max_x, max_y);
    toolbar.set_attribute("data-x", &format!("{:.2}", state.x))?;
    toolbar.set_attribute("data-y", &format!("{:.2}", state.y))?;
    toolbar.style().set_property(
        "transform",
        &format!("translate({:.2}px, {:.2}px)", state.x, state.y),
    )?;
    toolbar.style().set_property(
        "cursor",
        if state.is_dragging {
            "grabbing"
        } else {
            "grab"
        },
    )?;

    Ok(())
}
