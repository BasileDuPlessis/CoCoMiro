use crate::constants::*;
use crate::performance::PerformanceLogger;
use crate::state::ViewState;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

#[allow(deprecated)]
pub fn draw_grid(ctx: &CanvasRenderingContext2d, width: f64, height: f64, view_state: &ViewState) {
    let start_time = js_sys::Date::now();
    let grid_spacing = GRID_BASE_SPACING * view_state.zoom;
    let line_width = (1.0 / view_state.zoom).clamp(GRID_LINE_WIDTH_MIN, GRID_LINE_WIDTH_MAX);

    ctx.set_stroke_style(&JsValue::from_str(GRID_COLOR));
    ctx.set_line_width(line_width);

    // Calculate grid offset based on pan
    let offset_x = view_state.pan_x % grid_spacing;
    let offset_y = view_state.pan_y % grid_spacing;

    // Draw vertical lines
    let mut x = offset_x;
    while x < width {
        ctx.begin_path();
        ctx.move_to(x, 0.0);
        ctx.line_to(x, height);
        ctx.stroke();
        x += grid_spacing;
    }

    // Draw horizontal lines
    let mut y = offset_y;
    while y < height {
        ctx.begin_path();
        ctx.move_to(0.0, y);
        ctx.line_to(width, y);
        ctx.stroke();
        y += grid_spacing;
    }

    PerformanceLogger::log_canvas_operation("draw_grid", start_time);
}

#[allow(deprecated)]
pub fn draw_debug_overlay(
    ctx: &CanvasRenderingContext2d,
    width: f64,
    _height: f64,
    view_state: &ViewState,
) {
    let start_time = js_sys::Date::now();
    let debug_text = format!(
        "Zoom: {:.2}\nPan: ({:.1}, {:.1})\nDragging: {}",
        view_state.zoom, view_state.pan_x, view_state.pan_y, view_state.is_dragging
    );

    ctx.set_fill_style(&JsValue::from_str(DEBUG_OVERLAY_BG));
    ctx.set_font("14px monospace");
    ctx.fill_rect(width - 200.0, 10.0, 190.0, 60.0);

    ctx.set_fill_style(&JsValue::from_str(DEBUG_OVERLAY_TEXT));
    for (i, line) in debug_text.lines().enumerate() {
        let _ = ctx.fill_text(line, width - 190.0, 30.0 + (i as f64 * 16.0));
    }

    PerformanceLogger::log_canvas_operation("draw_debug_overlay", start_time);
}
