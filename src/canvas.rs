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

/// Represents a segment of text with formatting information
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
struct TextSegment {
    text: String,
    bold: bool,
    italic: bool,
    underline: bool,
}

/// Parses markdown-style formatting from text and returns formatted segments
///
/// Supports:
/// - **text** for bold
/// - *text* for italic
/// - __text__ for underline
///
/// # Arguments
/// * `text` - The text to parse for formatting
///
/// # Returns
/// A vector of TextSegment objects representing the parsed text
#[cfg(target_arch = "wasm32")]
fn parse_formatted_text(text: &str) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Find the next HTML formatting tag (support various formats including spans)
        let patterns = [
            ("<b>", "bold", "</b>"),
            ("<b ", "bold", "</b>"),
            ("<strong>", "bold", "</strong>"),
            ("<strong ", "bold", "</strong>"),
            ("<i>", "italic", "</i>"),
            ("<i ", "italic", "</i>"),
            ("<em>", "italic", "</em>"),
            ("<em ", "italic", "</em>"),
            ("<u>", "underline", "</u>"),
            ("<u ", "underline", "</u>"),
            ("<span ", "span", "</span>"),
        ];

        // Helper to decode HTML entities in text
        fn decode_html_entities(s: &str) -> String {
            s.replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&")
        }

        // Special handling for <br>, <br/>, <br /> (self-closing)
        let br_pos = remaining.find("<br");
        let mut earliest_pos = None;
        let mut tag_info = None;

        for (pattern, tag_type, closing) in &patterns {
            if let Some(pos) = remaining.find(pattern) {
                if earliest_pos.is_none() || pos < earliest_pos.unwrap() {
                    earliest_pos = Some(pos);
                    tag_info = Some((pos, *tag_type, *closing));
                }
            }
        }

        // Check if <br>, <br/>, or <br /> is the earliest tag
        if let Some(pos) = br_pos {
            // Only treat as <br> if it's the earliest tag
            if earliest_pos.is_none() || pos < earliest_pos.unwrap() {
                // Add text before the <br> as plain text
                if pos > 0 {
                    segments.push(TextSegment {
                        text: decode_html_entities(&remaining[..pos]),
                        bold: false,
                        italic: false,
                        underline: false,
                    });
                }
                // Find the end of the <br> tag (handle <br>, <br/>, <br />)
                let after = &remaining[pos..];
                let br_end = if after.starts_with("<br>") {
                    pos + 4
                } else if after.starts_with("<br/>") {
                    pos + 5
                } else if after.starts_with("<br />") {
                    pos + 6
                } else {
                    // Not a recognized <br> tag, treat as plain text
                    segments.push(TextSegment {
                        text: decode_html_entities(&remaining[pos..]),
                        bold: false,
                        italic: false,
                        underline: false,
                    });
                    break;
                };
                // Insert a line break
                segments.push(TextSegment {
                    text: "\n".to_string(),
                    bold: false,
                    italic: false,
                    underline: false,
                });
                remaining = &remaining[br_end..];
                continue;
            }
        }

        if let Some((pos, tag_type, closing_tag)) = tag_info {
            // Add text before the tag as plain text
            if pos > 0 {
                segments.push(TextSegment {
                    text: decode_html_entities(&remaining[..pos]),
                    bold: false,
                    italic: false,
                    underline: false,
                });
            }

            // Find the end of the opening tag
            let tag_end = if let Some(gt_pos) = remaining[pos..].find('>') {
                pos + gt_pos + 1
            } else {
                // Malformed tag, treat as plain text
                segments.push(TextSegment {
                    text: decode_html_entities(&remaining[pos..]),
                    bold: false,
                    italic: false,
                    underline: false,
                });
                break;
            };

            // For <span> tags, check if they have formatting styles
            let (is_bold, is_italic, is_underline) = if tag_type == "span" {
                let tag_content = &remaining[pos..tag_end];
                let has_bold = tag_content.contains("font-weight:")
                    && (tag_content.contains("bold") || tag_content.contains("700"));
                let has_italic =
                    tag_content.contains("font-style:") && tag_content.contains("italic");
                let has_underline =
                    tag_content.contains("text-decoration:") && tag_content.contains("underline");
                (has_bold, has_italic, has_underline)
            } else {
                (
                    tag_type == "bold",
                    tag_type == "italic",
                    tag_type == "underline",
                )
            };

            if let Some(end_pos) = remaining[tag_end..].find(closing_tag) {
                let end_pos = tag_end + end_pos;
                let formatted_text = &remaining[tag_end..end_pos];

                // Recursively parse the content inside the tag
                let sub_segments = parse_formatted_text(formatted_text);
                for mut sub_segment in sub_segments {
                    // Merge formatting flags
                    sub_segment.bold |= is_bold;
                    sub_segment.italic |= is_italic;
                    sub_segment.underline |= is_underline;
                    segments.push(sub_segment);
                }

                remaining = &remaining[end_pos + closing_tag.len()..];
            } else {
                // No closing tag found, treat as plain text
                segments.push(TextSegment {
                    text: remaining[pos..].to_string(),
                    bold: false,
                    italic: false,
                    underline: false,
                });
                break;
            }
        } else {
            // No more tags found, add remaining text as plain text (decode HTML entities)
            segments.push(TextSegment {
                text: decode_html_entities(remaining),
                bold: false,
                italic: false,
                underline: false,
            });
            break;
        }
    }

    segments
}

/// Parses HTML content and returns plain text and TextFormat spans
///
/// # Arguments
/// * `html` - The HTML content to parse
///
/// # Returns
/// A tuple of (plain_text, formatting_spans)
#[cfg(target_arch = "wasm32")]
pub fn parse_html_to_text_and_formatting(
    html: &str,
) -> (String, Vec<super::sticky_notes::TextFormat>) {
    use super::sticky_notes::TextFormat;

    let segments = parse_formatted_text(html);
    let mut plain_text = String::new();
    let mut formatting_spans: Vec<super::sticky_notes::TextFormat> = Vec::new();
    let mut current_pos = 0;

    for segment in segments {
        let start_pos = current_pos;
        plain_text.push_str(&segment.text);
        let end_pos = current_pos + segment.text.len();

        // If this segment has formatting, add or merge with previous span
        if segment.bold || segment.italic || segment.underline {
            if let Some(last_span) = formatting_spans.last_mut() {
                // Check if we can merge with the previous span
                if last_span.end == start_pos
                    && last_span.bold == segment.bold
                    && last_span.italic == segment.italic
                    && last_span.underline == segment.underline
                {
                    // Merge with previous span
                    last_span.end = end_pos;
                } else {
                    // Add new span
                    formatting_spans.push(TextFormat {
                        start: start_pos,
                        end: end_pos,
                        bold: segment.bold,
                        italic: segment.italic,
                        underline: segment.underline,
                    });
                }
            } else {
                // First span
                formatting_spans.push(TextFormat {
                    start: start_pos,
                    end: end_pos,
                    bold: segment.bold,
                    italic: segment.italic,
                    underline: segment.underline,
                });
            }
        }

        current_pos = end_pos;
    }

    (plain_text, formatting_spans)
}

/// Converts plain text and TextFormat spans to HTML
///
/// # Arguments
/// * `text` - The plain text content
/// * `formatting` - The formatting spans
///
/// # Returns
/// HTML representation of the formatted text
#[cfg(target_arch = "wasm32")]
pub fn format_text_with_spans_to_html(
    text: &str,
    formatting: &[super::sticky_notes::TextFormat],
) -> String {
    if formatting.is_empty() {
        // No formatting, return plain text with line breaks converted to <br>
        return text.replace("\n", "<br>");
    }

    let mut result = String::new();
    let mut last_end = 0;

    for span in formatting {
        // Add text before this span
        if span.start > last_end {
            let before_text = &text[last_end..span.start];
            result.push_str(&before_text.replace("\n", "<br>"));
        }

        // Add the formatted text
        let span_text = &text[span.start..span.end];
        let formatted_text = format_span_text(span_text, span);
        result.push_str(&formatted_text);

        last_end = span.end;
    }

    // Add remaining text after the last span
    if last_end < text.len() {
        let after_text = &text[last_end..];
        result.push_str(&after_text.replace("\n", "<br>"));
    }

    result
}

/// Formats a text span with the given formatting
///
/// # Arguments
/// * `text` - The text to format
/// * `span` - The formatting information
///
/// # Returns
/// HTML formatted text
#[cfg(target_arch = "wasm32")]
fn format_span_text(text: &str, span: &super::sticky_notes::TextFormat) -> String {
    let mut result = text.replace("\n", "<br>");

    if span.underline {
        result = format!("<u>{}</u>", result);
    }
    if span.italic {
        result = format!("<i>{}</i>", result);
    }
    if span.bold {
        result = format!("<b>{}</b>", result);
    }

    result
}

/// Creates a CSS font string based on text formatting
///
/// # Arguments
/// * `segment` - The text segment with formatting information
/// * `base_size` - Base font size in pixels
///
/// # Returns
/// A CSS font property string
#[cfg(target_arch = "wasm32")]
fn format_font(segment: &TextSegment, base_size: f64) -> String {
    let mut styles = Vec::new();

    if segment.italic {
        styles.push("italic".to_string());
    }

    if segment.bold {
        styles.push("bold".to_string());
    }

    styles.push(format!("{}px", base_size));
    styles.push("Inter, sans-serif".to_string());

    styles.join(" ")
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
/// Renders the grid background on the canvas
///
/// # Arguments
/// * `ctx` - The 2D canvas rendering context
/// * `width` - Canvas width in CSS pixels
/// * `height` - Canvas height in CSS pixels
/// * `zoom` - Current zoom level
/// * `pan_x` - Horizontal pan offset
/// * `pan_y` - Vertical pan offset
#[cfg(target_arch = "wasm32")]
fn render_grid_background(
    ctx: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) -> crate::error::AppResult<()> {
    let grid_spacing = (GRID_BASE_SPACING * zoom).clamp(GRID_MIN_SPACING, GRID_MAX_SPACING);
    let offset_x = pan_x.rem_euclid(grid_spacing);
    let offset_y = pan_y.rem_euclid(grid_spacing);

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

    Ok(())
}

/// Renders all sticky notes on the canvas
///
/// # Arguments
/// * `ctx` - The 2D canvas rendering context
/// * `state` - The complete application state
/// * `width` - Canvas width in CSS pixels
/// * `height` - Canvas height in CSS pixels
#[cfg(target_arch = "wasm32")]
/// Renders the background and border for a single sticky note
fn render_note_background_and_border(
    ctx: &CanvasRenderingContext2d,
    note: &crate::sticky_notes::StickyNote,
    screen_x: f64,
    screen_y: f64,
    screen_width: f64,
    screen_height: f64,
    is_selected: bool,
) -> crate::error::AppResult<()> {
    // Draw note background
    ctx.set_fill_style_str(&note.color);
    ctx.fill_rect(screen_x, screen_y, screen_width, screen_height);

    // Draw note border
    ctx.set_stroke_style_str(if is_selected {
        "#2563eb" // Blue border for selected notes
    } else {
        "#374151" // Gray border for unselected notes
    });
    ctx.set_line_width(2.0);
    ctx.stroke_rect(screen_x, screen_y, screen_width, screen_height);

    Ok(())
}

#[cfg(target_arch = "wasm32")]
/// Renders resize handles for a selected sticky note
fn render_note_resize_handles(
    ctx: &CanvasRenderingContext2d,
    note: &crate::sticky_notes::StickyNote,
    state: &crate::AppState,
    width: f64,
    height: f64,
) -> crate::error::AppResult<()> {
    let handle_positions = note.handle_positions(&state.viewport, width, height);
    for (_handle, hx, hy) in handle_positions {
        // Determine handle color based on state
        let fill_color = if false {
            // if state.resizing.is_resizing
            //     && state.resizing.note_id == Some(note.id)
            //     && state.resizing.handle == Some(handle)
            // {
            // Active handle (being resized)
            "#2563eb"
        } else {
            // } else if state.hovered_resize_handle == Some((note.id, handle)) {
            // Hovered handle
            // "#6b7280"
            // } else {
            // Normal handle
            "#9ca3af"
        };

        // Draw handle as a filled square
        ctx.set_fill_style_str(fill_color);
        let handle_size = crate::sticky_notes::RESIZE_HANDLE_SIZE;
        ctx.fill_rect(
            hx - handle_size / 2.0,
            hy - handle_size / 2.0,
            handle_size,
            handle_size,
        );

        // Draw handle border
        ctx.set_stroke_style_str("#ffffff");
        ctx.set_line_width(1.0);
        ctx.stroke_rect(
            hx - handle_size / 2.0,
            hy - handle_size / 2.0,
            handle_size,
            handle_size,
        );
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
/// Renders the text content of a sticky note with rich formatting and wrapping
fn render_note_text_content(
    ctx: &CanvasRenderingContext2d,
    note: &crate::sticky_notes::StickyNote,
    screen_x: f64,
    screen_y: f64,
    screen_width: f64,
    screen_height: f64,
) -> crate::error::AppResult<()> {
    if note.content.is_empty() {
        return Ok(());
    }

    ctx.set_text_align("left");
    ctx.set_text_baseline("top");
    // Add some padding
    let text_x = screen_x + 8.0;
    let text_y = screen_y + 8.0;
    let max_text_width = screen_width - 16.0; // Account for padding
    let max_text_height = screen_height - 16.0; // Account for padding

    // Save canvas state and set clipping region to prevent text overflow
    ctx.save();
    ctx.begin_path();
    ctx.rect(screen_x + 8.0, screen_y + 8.0, max_text_width, max_text_height);
    ctx.clip();

    // If there is no formatting, split plain text on '\n', then wrap each line to fit within the note
    if note.formatting.is_empty() && !(note.content.contains('<') && note.content.contains('>')) {
        let lines: Vec<&str> = note.content.split('\n').collect();
        let mut y_offset = 0.0;
        ctx.set_font("14px sans-serif");
        for line in lines {
            let mut current_line = String::new();
            for word in line.split_whitespace() {
                let test_line = if current_line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", current_line, word)
                };
                let test_width = ctx.measure_text(&test_line)?.width();
                if test_width <= max_text_width {
                    current_line = test_line;
                } else {
                    // Draw the current line and start a new one
                    ctx.set_fill_style_str("#000000");
                    ctx.fill_text(&current_line, text_x, text_y + y_offset)?;
                    y_offset += 18.0;
                    current_line = word.to_string();
                }
            }
            // Draw any remaining text in the line
            if !current_line.is_empty() {
                ctx.set_fill_style_str("#000000");
                ctx.fill_text(&current_line, text_x, text_y + y_offset)?;
                y_offset += 18.0;
            }
            // If the line was empty, still advance y_offset for blank lines
            if line.trim().is_empty() {
                y_offset += 18.0;
            }
        }

        // Restore canvas state to remove clipping
        ctx.restore();
        return Ok(());
    }

    // Otherwise, use the existing rich text/HTML logic
    let html_content = if note.content.contains('<') && note.content.contains('>') {
        note.content.clone()
    } else {
        format_text_with_spans_to_html(&note.content, &note.formatting)
    };
    let formatted_segments = parse_formatted_text(&html_content);

    let mut all_lines = Vec::new();
    let mut current_line_segments = Vec::new();
    let mut current_line_width = 0.0;

    for segment in formatted_segments {
        // Special handling for line break segments created by <br> tags
        if segment.text == "\n" {
            // Always start a new line for line breaks, even if current line is empty
            // This ensures consecutive line breaks create multiple blank lines
            all_lines.push(current_line_segments);
            current_line_segments = Vec::new();
            current_line_width = 0.0;
            continue;
        }

        let lines_in_segment: Vec<&str> = segment.text.lines().collect();
        for (i, line_part) in lines_in_segment.iter().enumerate() {
            if i > 0 {
                if !current_line_segments.is_empty() {
                    all_lines.push(current_line_segments);
                    current_line_segments = Vec::new();
                    current_line_width = 0.0;
                }
            }
            if line_part.is_empty() {
                continue;
            }
            let chars: Vec<char> = line_part.chars().collect();
            let mut current_word = String::new();
            for (_char_idx, &ch) in chars.iter().enumerate() {
                if ch.is_whitespace() {
                    if !current_word.is_empty() {
                        let word_segment = TextSegment {
                            text: current_word.clone(),
                            bold: segment.bold,
                            italic: segment.italic,
                            underline: segment.underline,
                        };
                        let font = format_font(&word_segment, 14.0);
                        ctx.set_font(&font);
                        let word_width = ctx.measure_text(&word_segment.text)?.width();
                        if current_line_width + word_width <= max_text_width
                            || current_line_segments.is_empty()
                        {
                            current_line_segments.push(word_segment);
                            current_line_width += word_width;
                        } else {
                            if !current_line_segments.is_empty() {
                                all_lines.push(current_line_segments);
                            }
                            current_line_segments = vec![word_segment];
                            current_line_width = word_width;
                        }
                        current_word.clear();
                    }
                    let space_segment = TextSegment {
                        text: ch.to_string(),
                        bold: segment.bold,
                        italic: segment.italic,
                        underline: segment.underline,
                    };
                    ctx.set_font(&format_font(&space_segment, 14.0));
                    let space_width = ctx.measure_text(&space_segment.text)?.width();
                    if current_line_width + space_width <= max_text_width
                        || current_line_segments.is_empty()
                    {
                        current_line_segments.push(space_segment);
                        current_line_width += space_width;
                    } else {
                        if !current_line_segments.is_empty() {
                            all_lines.push(current_line_segments);
                        }
                        current_line_segments = vec![space_segment];
                        current_line_width = space_width;
                    }
                } else {
                    current_word.push(ch);
                }
            }
            if !current_word.is_empty() {
                let word_segment = TextSegment {
                    text: current_word,
                    bold: segment.bold,
                    italic: segment.italic,
                    underline: segment.underline,
                };
                let font = format_font(&word_segment, 14.0);
                ctx.set_font(&font);
                let word_width = ctx.measure_text(&word_segment.text)?.width();
                if current_line_width + word_width <= max_text_width
                    || current_line_segments.is_empty()
                {
                    current_line_segments.push(word_segment);
                    current_line_width += word_width;
                } else {
                    if !current_line_segments.is_empty() {
                        all_lines.push(current_line_segments);
                    }
                    current_line_segments = vec![word_segment];
                    current_line_width = word_width;
                }
            }
        }
    }
    if !current_line_segments.is_empty() {
        all_lines.push(current_line_segments);
    }

    // Render all lines
    let mut y_offset = 0.0;
    for line_segments in all_lines {
        let mut x_offset = 0.0;
        for segment in line_segments {
            let font = format_font(&segment, 14.0);
            ctx.set_font(&font);

            // Set text decoration for underline
            if segment.underline {
                ctx.set_stroke_style_str("#000000");
                ctx.set_line_width(1.0);
                let text_width = ctx.measure_text(&segment.text)?.width();
                let underline_y = text_y + y_offset + 14.0 + 1.0; // Below baseline
                ctx.begin_path();
                ctx.move_to(text_x + x_offset, underline_y);
                ctx.line_to(text_x + x_offset + text_width, underline_y);
                ctx.stroke();
            }

            // Draw the text
            ctx.set_fill_style_str("#000000");
            ctx.fill_text(&segment.text, text_x + x_offset, text_y + y_offset)?;

            // Update x offset for next segment
            let segment_width = ctx.measure_text(&segment.text)?.width();
            x_offset += segment_width;
        }
        y_offset += 18.0; // Line height
    }

    // Restore canvas state to remove clipping
    ctx.restore();

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn render_sticky_notes(
    ctx: &CanvasRenderingContext2d,
    state: &crate::AppState,
    width: f64,
    height: f64,
) -> crate::error::AppResult<()> {
    // Render sticky notes
    for note in &state.sticky_notes.notes {
        // Calculate screen position from world coordinates
        // This matches the center cross positioning: screen = world * zoom + center + pan
        let screen_x = note.x * state.viewport.zoom + width / 2.0 + state.viewport.pan_x;
        let screen_y = note.y * state.viewport.zoom + height / 2.0 + state.viewport.pan_y;
        let screen_width = note.width * state.viewport.zoom;
        let screen_height = note.height * state.viewport.zoom;

        let is_selected = Some(note.id) == state.sticky_notes.selected_note_id;

        // Render note background and border
        render_note_background_and_border(
            ctx,
            note,
            screen_x,
            screen_y,
            screen_width,
            screen_height,
            is_selected,
        )?;

        // Draw resize handles for selected notes
        if is_selected {
            render_note_resize_handles(ctx, note, state, width, height)?;
        }

        // Draw note content text with rich formatting and wrapping
        render_note_text_content(ctx, note, screen_x, screen_y, screen_width, screen_height)?;
    }

    Ok(())
}

/// Updates canvas attributes and cursor based on application state
///
/// # Arguments
/// * `canvas` - The HTML canvas element
/// * `state` - The complete application state
/// * `width` - Canvas width in CSS pixels
/// * `height` - Canvas height in CSS pixels
#[cfg(target_arch = "wasm32")]
fn update_canvas_attributes(
    canvas: &HtmlCanvasElement,
    state: &crate::AppState,
    _width: f64,
    _height: f64,
) -> crate::error::AppResult<()> {
    canvas.set_attribute("data-ready", "true")?;
    canvas.set_attribute("data-pan-x", &format!("{:.2}", state.viewport.pan_x))?;
    canvas.set_attribute("data-pan-y", &format!("{:.2}", state.viewport.pan_y))?;
    canvas.set_attribute("data-zoom", &format!("{:.2}", state.viewport.zoom))?;

    // Determine cursor based on interaction state
    // Note: Cursor is set by the centralized styling function below

    crate::styling::components::update_canvas_cursor(canvas, state)?;

    Ok(())
}

/// Updates the status display element with current application metrics
///
/// # Arguments
/// * `status` - The status HTML element to update
/// * `state` - The complete application state
/// * `fps` - Current frames per second
/// * `avg_render_time` - Average render time in milliseconds
#[cfg(target_arch = "wasm32")]
fn update_status_display(
    status: &HtmlElement,
    state: &crate::AppState,
    fps: f64,
    avg_render_time: f64,
) -> crate::error::AppResult<()> {
    // Get selected note dimensions if any
    let selected_note_info = if let Some(note_id) = state.sticky_notes.selected_note_id {
        if let Some(note) = state.sticky_notes.get_note(note_id) {
            format!("Note {}: {:.0}×{:.0}", note_id, note.width, note.height)
        } else {
            "Note: N/A".to_string()
        }
    } else {
        "No selection".to_string()
    };

    status.set_text_content(Some(&format!(
        "Mouse ({:.0}, {:.0}) · {} · Pan ({:.0}, {:.0}) · Zoom {:.2}× · {:.0} FPS · {:.1}ms · {} notes · {}",
        state.mouse_x,
        state.mouse_y,
        selected_note_info,
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
pub fn canvas_css_size(canvas: &HtmlCanvasElement) -> crate::error::AppResult<(f64, f64)> {
    let browser_window = window()
        .ok_or_else(|| crate::error::AppError::BrowserEnv("window is unavailable".to_string()))?;
    let viewport_width = browser_window
        .inner_width()
        .map_err(|_| {
            crate::error::AppError::BrowserEnv("failed to get window inner width".to_string())
        })?
        .as_f64()
        .unwrap_or(FALLBACK_VIEWPORT_WIDTH);
    let viewport_height = browser_window
        .inner_height()
        .map_err(|_| {
            crate::error::AppError::BrowserEnv("failed to get window inner height".to_string())
        })?
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
) -> crate::error::AppResult<()> {
    let browser_window = window()
        .ok_or_else(|| crate::error::AppError::BrowserEnv("window is unavailable".to_string()))?;
    let (width, height) = canvas_css_size(canvas)?;
    // Keep CSS size stable while allocating a denser backing store for Retina/HiDPI displays.
    let device_pixel_ratio = browser_window.device_pixel_ratio().max(1.0);

    crate::styling::sizing::set_dimensions(canvas, width, height)?;
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
/// Renders the complete canvas scene including grid, sticky notes, and UI updates
///
/// This is the main rendering function that orchestrates the entire canvas drawing process.
/// It handles performance monitoring, clears and redraws the canvas, renders all visual
/// elements, and updates UI state.
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
) -> crate::error::AppResult<()> {
    // Start performance timing
    let performance = window().and_then(|w| w.performance()).ok_or_else(|| {
        crate::error::AppError::BrowserEnv("performance API unavailable".to_string())
    })?;
    let start_time = performance.now();

    // Update FPS
    PERFORMANCE_METRICS.with(|metrics| {
        let mut metrics = metrics.borrow_mut();
        metrics.update_fps(start_time);
    });

    let (width, height) = canvas_css_size(canvas)?;

    // Render grid background
    render_grid_background(
        ctx,
        width,
        height,
        state.viewport.zoom,
        state.viewport.pan_x,
        state.viewport.pan_y,
    )?;

    // Render sticky notes
    render_sticky_notes(ctx, state, width, height)?;

    // Update canvas attributes and cursor
    update_canvas_attributes(canvas, state, width, height)?;

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

    // Update status display
    update_status_display(status, state, fps, avg_render_time)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn sync_toolbar_position(
    toolbar: &HtmlElement,
    workspace: &HtmlElement,
    state: &mut crate::toolbar::FloatingToolbarState,
) -> crate::error::AppResult<()> {
    let max_x = (f64::from(workspace.client_width() - toolbar.offset_width())
        - TOOLBAR_EDGE_PADDING)
        .max(TOOLBAR_EDGE_PADDING);
    let max_y = (f64::from(workspace.client_height() - toolbar.offset_height())
        - TOOLBAR_EDGE_PADDING)
        .max(TOOLBAR_EDGE_PADDING);

    crate::styling::components::update_toolbar_position(toolbar, state, max_x, max_y)?;

    Ok(())
}
