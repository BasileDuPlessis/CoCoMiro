//! # Canvas Rendering Engine
//!
//! This module handles all canvas-based rendering for the CoCoMiro application.
//! It provides functions for drawing the grid, sticky notes, and managing the
//! canvas viewport with proper HiDPI support.
//!
//! ## Rendering Pipeline
//!
//! The rendering process follows this order:
//! 1. Clear canvas with background color
//! 2. Draw grid lines based on zoom level
//! 3. Render all sticky notes with selection highlighting
//! 4. Update status text and canvas attributes
//!
//! ## Coordinate Transformations
//!
//! The module handles transformations between world coordinates (sticky note positions)
//! and screen coordinates (canvas pixels) accounting for:
//! - Viewport pan offset
//! - Zoom scaling
//! - Canvas device pixel ratio for HiDPI displays
//!
//! ## Performance Considerations
//!
//! - Grid spacing adapts to zoom level for optimal visual density
//! - Canvas is resized efficiently with proper device pixel ratio handling
//! - Rendering uses immediate mode for simplicity and performance

#[cfg(target_arch = "wasm32")]
use crate::toolbar::TOOLBAR_EDGE_PADDING;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, window};

#[cfg(target_arch = "wasm32")]
/// Performance metrics for monitoring rendering performance
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    /// Timestamp of the last frame (in milliseconds)
    last_frame_time: f64,
    /// Current FPS calculation
    fps: f64,
    /// Last measured render time (in milliseconds)
    last_render_time: f64,
    /// Frame count for averaging
    frame_count: u32,
    /// Accumulated render times for averaging
    render_time_accumulator: f64,
}

#[cfg(target_arch = "wasm32")]
impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            last_frame_time: 0.0,
            fps: 0.0,
            last_render_time: 0.0,
            frame_count: 0,
            render_time_accumulator: 0.0,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl PerformanceMetrics {
    /// Updates FPS calculation based on current timestamp
    fn update_fps(&mut self, current_time: f64) {
        if self.last_frame_time > 0.0 {
            let delta_time = current_time - self.last_frame_time;
            if delta_time > 0.0 {
                self.fps = 1000.0 / delta_time;
            }
        }
        self.last_frame_time = current_time;
    }

    /// Records render time and updates averages
    fn record_render_time(&mut self, render_time: f64) {
        self.last_render_time = render_time;
        self.render_time_accumulator += render_time;
        self.frame_count += 1;

        // Reset accumulator every 60 frames to keep averages fresh
        if self.frame_count >= 60 {
            self.frame_count = 0;
            self.render_time_accumulator = 0.0;
        }
    }

    /// Gets the average render time over recent frames
    fn average_render_time(&self) -> f64 {
        if self.frame_count > 0 {
            self.render_time_accumulator / self.frame_count as f64
        } else {
            self.last_render_time
        }
    }
}

/// Wraps text to fit within a specified width using estimated character width.
///
/// This function splits text into lines that fit within the given pixel width,
/// breaking at word boundaries when possible. Uses an estimated average character
/// width for simplicity.
///
/// # Arguments
/// * `text` - The text to wrap
/// * `max_width` - Maximum width in pixels for each line
/// * `font_size` - Font size in pixels (default 14)
///
/// # Returns
/// A vector of strings, each representing a wrapped line
#[cfg(target_arch = "wasm32")]
fn wrap_text(text: &str, max_width: f64, font_size: f64) -> Vec<String> {
    let mut lines = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return lines;
    }

    // Estimate average character width (rough approximation)
    let avg_char_width = font_size * 0.6; // Approximate for most fonts
    let max_chars_per_line = (max_width / avg_char_width) as usize;

    let mut current_line = String::new();

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if test_line.len() <= max_chars_per_line {
            current_line = test_line;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static PERFORMANCE_METRICS: RefCell<PerformanceMetrics> = RefCell::new(PerformanceMetrics::default());
}

#[cfg(target_arch = "wasm32")]
/// Fallback viewport width when window dimensions are unavailable
const FALLBACK_VIEWPORT_WIDTH: f64 = 1280.0;
#[cfg(target_arch = "wasm32")]
/// Fallback viewport height when window dimensions are unavailable
const FALLBACK_VIEWPORT_HEIGHT: f64 = 840.0;
#[cfg(target_arch = "wasm32")]
/// Horizontal margin around canvas for layout
const CANVAS_HORIZONTAL_MARGIN: f64 = 32.0;
#[cfg(target_arch = "wasm32")]
/// Vertical margin around canvas for layout
const CANVAS_VERTICAL_MARGIN: f64 = 96.0;
#[cfg(target_arch = "wasm32")]
/// Minimum canvas edge length to ensure usability
const MIN_CANVAS_EDGE: f64 = 320.0;
#[cfg(target_arch = "wasm32")]
/// Base spacing for grid lines in world coordinates
const GRID_BASE_SPACING: f64 = 48.0;
#[cfg(target_arch = "wasm32")]
/// Minimum grid spacing to prevent overcrowding
const GRID_MIN_SPACING: f64 = 24.0;
#[cfg(target_arch = "wasm32")]
/// Maximum grid spacing to maintain visual reference
const GRID_MAX_SPACING: f64 = 120.0;
#[cfg(target_arch = "wasm32")]
/// Default help text shown in status area
const STATUS_HELP_TEXT: &str = "Drag to pan, scroll to zoom, or use the arrow keys and +/-.";

#[cfg(target_arch = "wasm32")]
/// Calculates the CSS size for the canvas element based on viewport dimensions.
///
/// This function determines the appropriate canvas size considering:
/// - Browser window inner dimensions
/// - Configured margins for UI elements
/// - Minimum size constraints for usability
///
/// # Arguments
/// * `canvas` - The HTML canvas element
///
/// # Returns
/// * `Ok((width, height))` - CSS dimensions for the canvas
/// * `Err(AppError)` - Failed to access window or canvas properties
pub fn canvas_css_size(canvas: &HtmlCanvasElement) -> crate::AppResult<(f64, f64)> {
    let browser_window =
        window().ok_or_else(|| crate::AppError::BrowserEnv("window is unavailable".to_string()))?;
    let viewport_width = browser_window
        .inner_width()
        .map_err(|_| crate::AppError::BrowserEnv("failed to get window inner width".to_string()))?
        .as_f64()
        .unwrap_or(FALLBACK_VIEWPORT_WIDTH);
    let viewport_height = browser_window
        .inner_height()
        .map_err(|_| crate::AppError::BrowserEnv("failed to get window inner height".to_string()))?
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
/// Resizes the canvas element and updates its rendering context for HiDPI displays.
///
/// This function handles the complex process of canvas resizing by:
/// 1. Calculating appropriate CSS size for layout
/// 2. Setting CSS properties for proper layout
/// 3. Setting actual canvas bitmap size accounting for device pixel ratio
/// 4. Configuring the rendering context transform for crisp rendering
///
/// # Arguments
/// * `canvas` - The HTML canvas element to resize
/// * `ctx` - The 2D rendering context for the canvas
///
/// # Returns
/// * `Ok(())` - Canvas resized successfully
/// * `Err(AppError)` - Failed to resize or configure canvas
pub fn resize_canvas(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
) -> crate::AppResult<()> {
    let browser_window =
        window().ok_or_else(|| crate::AppError::BrowserEnv("window is unavailable".to_string()))?;
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
/// Renders the complete canvas scene including grid, sticky notes, and UI elements.
///
/// This is the main rendering function that draws the entire application state.
/// The rendering order ensures proper layering and visual hierarchy.
///
/// # Arguments
/// * `ctx` - The 2D canvas rendering context
/// * `canvas` - The HTML canvas element being rendered to
/// * `status` - The status text element to update
/// * `state` - The complete application state to render
///
/// # Returns
/// * `Ok(())` - Rendering completed successfully
/// * `Err(AppError)` - Rendering failed
pub fn render_canvas(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    status: &HtmlElement,
    state: &crate::AppState,
) -> crate::AppResult<()> {
    // Start performance timing
    let performance = window()
        .and_then(|w| w.performance())
        .ok_or_else(|| crate::AppError::BrowserEnv("performance API unavailable".to_string()))?;
    let start_time = performance.now();

    // Update FPS
    PERFORMANCE_METRICS.with(|metrics| {
        let mut metrics = metrics.borrow_mut();
        metrics.update_fps(start_time);
    });

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

        // Draw note content text with wrapping
        if !note.content.is_empty() {
            ctx.set_fill_style_str("#000000");
            ctx.set_font("14px Inter, sans-serif");
            ctx.set_text_align("left");
            ctx.set_text_baseline("top");
            // Add some padding
            let text_x = screen_x + 8.0;
            let text_y = screen_y + 8.0;
            let max_text_width = screen_width - 16.0; // Account for padding

            // Process text with line breaks and wrapping
            let mut all_lines = Vec::new();
            for line in note.content.lines() {
                if line.is_empty() {
                    all_lines.push(String::new());
                } else {
                    let wrapped_lines = wrap_text(line, max_text_width, 14.0);
                    all_lines.extend(wrapped_lines);
                }
            }

            // Render all lines
            let mut y_offset = 0.0;
            for line in all_lines {
                if !line.is_empty() {
                    ctx.fill_text(&line, text_x, text_y + y_offset)?;
                }
                y_offset += 18.0; // Line height
            }
        }
    }

    canvas.set_attribute("data-ready", "true")?;
    canvas.set_attribute("data-pan-x", &format!("{:.2}", state.viewport.pan_x))?;
    canvas.set_attribute("data-pan-y", &format!("{:.2}", state.viewport.pan_y))?;
    canvas.set_attribute("data-zoom", &format!("{:.2}", state.viewport.zoom))?;

    // Determine cursor based on interaction state
    let cursor = if state.sticky_notes.is_dragging {
        "grabbing"
    } else {
        // Check if hovering over a sticky note
        let world_pos = state
            .viewport
            .world_point_at(state.mouse_x, state.mouse_y, width, height);
        if state
            .sticky_notes
            .find_note_at(world_pos.0, world_pos.1)
            .is_some()
        {
            "grab"
        } else if state.viewport.is_dragging {
            "grabbing"
        } else {
            "grab"
        }
    };

    canvas.style().set_property("cursor", cursor)?;

    // Record render time
    let end_time = performance.now();
    let render_time = end_time - start_time;
    PERFORMANCE_METRICS.with(|metrics| {
        let mut metrics = metrics.borrow_mut();
        metrics.record_render_time(render_time);
    });

    // Get performance metrics for display
    let (fps, avg_render_time) = PERFORMANCE_METRICS.with(|metrics| {
        let metrics = metrics.borrow();
        (metrics.fps, metrics.average_render_time())
    });

    status.set_text_content(Some(&format!(
        "Pan ({:.0}, {:.0}) · Zoom {:.2}× · {:.0} FPS · {:.1}ms · {} notes · {}",
        state.viewport.pan_x,
        state.viewport.pan_y,
        state.viewport.zoom,
        fps,
        avg_render_time,
        state.sticky_notes.notes.len(),
        STATUS_HELP_TEXT
    )));

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn sync_toolbar_position(
    toolbar: &HtmlElement,
    workspace: &HtmlElement,
    state: &mut crate::toolbar::FloatingToolbarState,
) -> crate::AppResult<()> {
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
