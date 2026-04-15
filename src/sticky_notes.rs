//! # Sticky Notes Module
//!
//! This module manages the creation, storage, and manipulation of sticky notes
//! in the CoCoMiro infinite canvas application.
//!
//! ## Architecture
//!
//! The module consists of three main components:
//! - `TextFormat`: Represents formatting information for text ranges
//! - `StickyNote`: Represents individual sticky notes with position, size, content, formatting, and appearance
//! - `StickyNotesState`: Manages the collection of notes and handles selection/dragging state
//!
//! ## Coordinate System
//!
//! All coordinates are in world space (not screen space), allowing notes to exist
//! anywhere on the infinite canvas regardless of viewport position or zoom level.
//!
//! ## ID Generation
//!
//! Note IDs are generated using an atomic counter to ensure uniqueness across
//! the application lifetime, even with concurrent operations.
//!
//! ## Drag Operations
//!
//! The module supports dragging notes with offset tracking to maintain smooth
//! interaction. When a drag starts, the offset between the mouse cursor and note
//! position is recorded, ensuring the note moves relative to the cursor.
//!
//! ## Rich Text Support
//!
//! Notes support rich text formatting through the `formatting` field, which contains
//! a vector of `TextFormat` spans. Each span defines formatting (bold, italic, underline)
//! for a character range in the content string. Empty formatting vector indicates plain text.

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, PartialEq)]
/// Represents formatting information for a range of text in a sticky note.
///
/// Each format span defines the start and end character positions in the content
/// string and the formatting flags to apply (bold, italic, underline).
pub struct TextFormat {
    /// Start character position (inclusive) in the content string
    pub start: usize,
    /// End character position (exclusive) in the content string
    pub end: usize,
    /// Whether this range should be rendered in bold
    pub bold: bool,
    /// Whether this range should be rendered in italic
    pub italic: bool,
    /// Whether this range should be rendered with underline
    pub underline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents the different resize handles available on a sticky note.
///
/// Each variant corresponds to a specific position on the note's border where
/// users can click and drag to resize the note. The handles allow resizing
/// from corners (diagonal resize) and edges (horizontal/vertical resize).
pub enum ResizeHandle {
    /// Top-left corner handle for diagonal resizing
    TopLeft,
    /// Top edge handle for vertical resizing
    Top,
    /// Top-right corner handle for diagonal resizing
    TopRight,
    /// Right edge handle for horizontal resizing
    Right,
    /// Bottom-right corner handle for diagonal resizing
    BottomRight,
    /// Bottom edge handle for vertical resizing
    Bottom,
    /// Bottom-left corner handle for diagonal resizing
    BottomLeft,
    /// Left edge handle for horizontal resizing
    Left,
}

impl ResizeHandle {
    /// Returns the CSS cursor style appropriate for this resize handle.
    ///
    /// The cursor indicates the type of resize operation that will occur
    /// when dragging this handle (diagonal vs horizontal/vertical).
    pub fn cursor(&self) -> &'static str {
        match self {
            ResizeHandle::TopLeft | ResizeHandle::BottomRight => "nw-resize",
            ResizeHandle::Top | ResizeHandle::Bottom => "n-resize",
            ResizeHandle::TopRight | ResizeHandle::BottomLeft => "ne-resize",
            ResizeHandle::Left | ResizeHandle::Right => "e-resize",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
/// Tracks the state of an ongoing resize operation for sticky notes.
///
/// This struct maintains information about which note is being resized,
/// which handle is being used, the original dimensions, and the starting
/// mouse position for the resize operation.
pub struct ResizingState {
    /// Whether a resize operation is currently active
    pub is_resizing: bool,
    /// ID of the note being resized (None if not resizing)
    pub note_id: Option<u32>,
    /// The handle being used for resizing (None if not resizing)
    pub handle: Option<ResizeHandle>,
    /// Mouse X position when resize started (screen coordinates)
    pub start_mouse_x: f64,
    /// Mouse Y position when resize started (screen coordinates)
    pub start_mouse_y: f64,
    /// Original width of the note before resizing started
    pub original_width: f64,
    /// Original height of the note before resizing started
    pub original_height: f64,
}

#[derive(Debug, Clone, PartialEq)]
/// Represents a single sticky note on the infinite canvas.
///
/// Each sticky note has a unique ID, position in world coordinates,
/// dimensions, text content, formatting information, and background color.
/// Notes are rendered as rectangles with text content and can be dragged around the canvas.
pub struct StickyNote {
    /// Unique identifier for this note (auto-generated)
    pub id: u32,
    /// X-coordinate of the top-left corner in world space
    pub x: f64,
    /// Y-coordinate of the top-left corner in world space
    pub y: f64,
    /// Width of the note in world units
    pub width: f64,
    /// Height of the note in world units
    pub height: f64,
    /// Text content displayed on the note
    pub content: String,
    /// Rich text formatting spans applied to the content
    pub formatting: Vec<TextFormat>,
    /// Background color as a hex string (e.g., "#ffff88")
    pub color: String,
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

impl StickyNote {
    /// Creates a new sticky note at the specified world coordinates.
    ///
    /// The note is assigned a unique ID, default dimensions (200x150),
    /// default content ("New note"), no formatting, and default color (#ffff88).
    ///
    /// # Arguments
    /// * `x` - X-coordinate of the top-left corner in world space
    /// * `y` - Y-coordinate of the top-left corner in world space
    ///
    /// # Returns
    /// A new `StickyNote` instance with default properties
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            x,
            y,
            width: 200.0,
            height: 150.0,
            content: "New note".to_string(),
            formatting: Vec::new(),
            color: "#ffff88".to_string(),
        }
    }

    /// Checks if the given point lies within this note's boundaries.
    ///
    /// This method is used for hit testing during mouse interactions,
    /// determining whether a click or drag operation should affect this note.
    ///
    /// # Arguments
    /// * `px` - X-coordinate of the test point in world space
    /// * `py` - Y-coordinate of the test point in world space
    ///
    /// # Returns
    /// `true` if the point is inside the note's rectangle, `false` otherwise
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    /// Returns the world-space position of a resize handle on this note.
    ///
    /// The handle position is calculated relative to the note's top-left corner
    /// and dimensions. The returned coordinates represent the center of the handle.
    ///
    /// # Arguments
    /// * `handle` - The resize handle to get the position for
    ///
    /// # Returns
    /// A tuple `(x, y)` representing the world coordinates of the handle center
    pub fn handle_position(&self, handle: ResizeHandle) -> (f64, f64) {
        match handle {
            ResizeHandle::TopLeft => (self.x, self.y),
            ResizeHandle::Top => (self.x + self.width / 2.0, self.y),
            ResizeHandle::TopRight => (self.x + self.width, self.y),
            ResizeHandle::Right => (self.x + self.width, self.y + self.height / 2.0),
            ResizeHandle::BottomRight => (self.x + self.width, self.y + self.height),
            ResizeHandle::Bottom => (self.x + self.width / 2.0, self.y + self.height),
            ResizeHandle::BottomLeft => (self.x, self.y + self.height),
            ResizeHandle::Left => (self.x, self.y + self.height / 2.0),
        }
    }

    /// Returns the screen-space bounding box of a resize handle.
    ///
    /// This method converts the handle's world position to screen coordinates
    /// and returns the bounding box for hit testing. The handle is treated as
    /// a square centered on the handle position.
    ///
    /// # Arguments
    /// * `handle` - The resize handle to get the bounds for
    /// * `viewport` - The current viewport state for coordinate transformation
    /// * `canvas_width` - The canvas width in pixels
    /// * `canvas_height` - The canvas height in pixels
    ///
    /// # Returns
    /// A tuple `(left, top, right, bottom)` representing the screen bounding box
    pub fn handle_bounds(
        &self,
        handle: ResizeHandle,
        viewport: &crate::viewport::ViewportState,
        canvas_width: f64,
        canvas_height: f64,
    ) -> (f64, f64, f64, f64) {
        let (world_x, world_y) = self.handle_position(handle);
        let screen_x = world_x * viewport.zoom + canvas_width / 2.0 + viewport.pan_x;
        let screen_y = world_y * viewport.zoom + canvas_height / 2.0 + viewport.pan_y;

        let half_size = RESIZE_HANDLE_SIZE / 2.0;
        (
            screen_x - half_size,
            screen_y - half_size,
            screen_x + half_size,
            screen_y + half_size,
        )
    }

    /// Returns the screen-space positions of all resize handles for this note.
    ///
    /// This method converts all handle world positions to screen coordinates
    /// for rendering and hit testing purposes.
    ///
    /// # Arguments
    /// * `viewport` - The current viewport state for coordinate transformation
    /// * `canvas_width` - The canvas width in pixels
    /// * `canvas_height` - The canvas height in pixels
    ///
    /// # Returns
    /// A vector of tuples `(handle, x, y)` where `handle` is the ResizeHandle variant
    /// and `(x, y)` are the screen coordinates of the handle center
    pub fn handle_positions(
        &self,
        viewport: &crate::viewport::ViewportState,
        canvas_width: f64,
        canvas_height: f64,
    ) -> Vec<(ResizeHandle, f64, f64)> {
        [
            ResizeHandle::TopLeft,
            ResizeHandle::Top,
            ResizeHandle::TopRight,
            ResizeHandle::Right,
            ResizeHandle::BottomRight,
            ResizeHandle::Bottom,
            ResizeHandle::BottomLeft,
            ResizeHandle::Left,
        ]
        .iter()
        .map(|&handle| {
            let (world_x, world_y) = self.handle_position(handle);
            let screen_x = world_x * viewport.zoom + canvas_width / 2.0 + viewport.pan_x;
            let screen_y = world_y * viewport.zoom + canvas_height / 2.0 + viewport.pan_y;
            (handle, screen_x, screen_y)
        })
        .collect()
    }
}

/// Size of resize handles in screen pixels (width and height)
pub const RESIZE_HANDLE_SIZE: f64 = 8.0;

#[derive(Debug, Clone, PartialEq, Default)]
/// Manages the collection of sticky notes and their interaction state.
///
/// This struct maintains the list of all notes on the canvas and tracks
/// the current selection and dragging state. It provides methods for
/// adding, finding, and manipulating notes during user interactions.
pub struct StickyNotesState {
    /// All sticky notes currently on the canvas
    pub notes: Vec<StickyNote>,
    /// ID of the currently selected note (if any)
    pub selected_note_id: Option<u32>,
    /// Whether a note is currently being dragged
    pub is_dragging: bool,
    /// Offset between mouse cursor and note position during drag operations
    pub drag_offset: Option<(f64, f64)>,
}

impl StickyNotesState {
    /// Adds a new sticky note to the collection.
    ///
    /// The note is appended to the notes vector. No validation is performed
    /// on the note's properties - it is assumed to be properly constructed.
    ///
    /// # Arguments
    /// * `note` - The `StickyNote` instance to add
    pub fn add_note(&mut self, note: StickyNote) {
        self.notes.push(note);
    }

    /// Finds the topmost note that contains the given point.
    ///
    /// Notes are checked in reverse order (last added first) so that
    /// visually topmost notes are selected when notes overlap.
    ///
    /// # Arguments
    /// * `px` - X-coordinate of the test point in world space
    /// * `py` - Y-coordinate of the test point in world space
    ///
    /// # Returns
    /// The ID of the note containing the point, or `None` if no note contains it
    pub fn find_note_at(&self, px: f64, py: f64) -> Option<u32> {
        // Check in reverse order so top notes are selected first
        for note in self.notes.iter().rev() {
            if note.contains_point(px, py) {
                return Some(note.id);
            }
        }
        None
    }

    pub fn get_note_mut(&mut self, id: u32) -> Option<&mut StickyNote> {
        self.notes.iter_mut().find(|n| n.id == id)
    }

    pub fn get_note(&self, id: u32) -> Option<&StickyNote> {
        self.notes.iter().find(|n| n.id == id)
    }

    /// Initiates a drag operation for the specified note.
    ///
    /// This method records the offset between the mouse cursor and the note's
    /// current position, marks the note as selected, and sets the dragging state.
    /// The offset ensures smooth dragging where the note moves relative to the cursor.
    ///
    /// # Arguments
    /// * `note_id` - ID of the note to start dragging
    /// * `mouse_x` - Current X-coordinate of the mouse cursor in world space
    /// * `mouse_y` - Current Y-coordinate of the mouse cursor in world space
    pub fn start_drag(&mut self, note_id: u32, mouse_x: f64, mouse_y: f64) {
        if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
            self.is_dragging = true;
            self.drag_offset = Some((mouse_x - note.x, mouse_y - note.y));
            self.selected_note_id = Some(note_id);
        }
    }

    /// Updates the position of the currently dragged note.
    ///
    /// This method moves the selected note to follow the mouse cursor,
    /// maintaining the drag offset established when dragging started.
    /// Only has effect if a drag operation is currently active.
    ///
    /// # Arguments
    /// * `mouse_x` - Current X-coordinate of the mouse cursor in world space
    /// * `mouse_y` - Current Y-coordinate of the mouse cursor in world space
    pub fn drag_to(&mut self, mouse_x: f64, mouse_y: f64) {
        if let (true, Some((offset_x, offset_y))) = (self.is_dragging, self.drag_offset)
            && let Some(note_id) = self.selected_note_id
            && let Some(note) = self.get_note_mut(note_id)
        {
            note.x = mouse_x - offset_x;
            note.y = mouse_y - offset_y;
        }
    }

    /// Terminates the current drag operation.
    ///
    /// This method resets the dragging state and clears the drag offset,
    /// allowing new drag operations to start cleanly.
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.drag_offset = None;
    }

    /// Adds a new note positioned at the center of the current viewport.
    ///
    /// This method creates a note at the viewport center in world coordinates,
    /// with a small offset based on the number of existing notes to prevent
    /// exact overlap. The note is positioned to be visible within the current
    /// viewport bounds.
    ///
    /// # Arguments
    /// * `viewport_width` - Width of the viewport in screen pixels
    /// * `viewport_height` - Height of the viewport in screen pixels
    /// * `viewport_state` - Current viewport state for coordinate transformation
    pub fn add_note_at_viewport_center(
        &mut self,
        viewport_width: f64,
        viewport_height: f64,
        viewport_state: &crate::viewport::ViewportState,
    ) {
        let (center_world_x, center_world_y) = viewport_state.world_point_at(
            viewport_width / 2.0,
            viewport_height / 2.0,
            viewport_width,
            viewport_height,
        );
        let offset = self.notes.len() as f64 * 20.0;
        let mut note_x = center_world_x + offset;
        let mut note_y = center_world_y + offset;

        // Calculate visible world bounds to ensure note stays on screen
        let top_left_world =
            viewport_state.world_point_at(0.0, 0.0, viewport_width, viewport_height);
        let bottom_right_world = viewport_state.world_point_at(
            viewport_width,
            viewport_height,
            viewport_width,
            viewport_height,
        );

        // Adjust position to keep note within visible bounds with some margin
        let margin = 50.0; // pixels in screen space, converted to world space
        let margin_world = margin / viewport_state.zoom;

        let min_x = top_left_world.0 + margin_world;
        let max_x = bottom_right_world.0 - margin_world - (200.0 / viewport_state.zoom); // account for note width in world space
        let min_y = top_left_world.1 + margin_world;
        let max_y = bottom_right_world.1 - margin_world - (150.0 / viewport_state.zoom); // account for note height in world space

        // Only clamp if the bounds are reasonable (visible area is large enough for the note)
        let visible_width = max_x - min_x;
        let visible_height = max_y - min_y;
        let note_world_width = 200.0 / viewport_state.zoom;
        let note_world_height = 150.0 / viewport_state.zoom;

        if visible_width > note_world_width + 2.0 * margin_world
            && visible_height > note_world_height + 2.0 * margin_world
        {
            // Bounds are reasonable, clamp the position
            note_x = note_x.clamp(min_x, max_x);
            note_y = note_y.clamp(min_y, max_y);
        }
        // Otherwise, don't clamp - let the note be placed at the calculated position

        let note = StickyNote::new(note_x, note_y);
        self.add_note(note);
    }

    /// Deletes the currently selected sticky note.
    ///
    /// This method removes the selected note from the collection and clears
    /// the selection. If no note is currently selected, this method does nothing.
    pub fn delete_selected(&mut self) {
        if let Some(id) = self.selected_note_id {
            self.notes.retain(|n| n.id != id);
            self.selected_note_id = None;
        }
    }

    /// Finds which resize handle (if any) is under the given screen coordinates.
    ///
    /// This method checks only the currently selected note for resize handles.
    /// Handle detection takes priority over note content area selection.
    /// The method converts screen coordinates to handle bounds for accurate hit testing.
    ///
    /// # Arguments
    /// * `screen_x` - X-coordinate in screen space (pixels)
    /// * `screen_y` - Y-coordinate in screen space (pixels)
    /// * `viewport` - Current viewport state for coordinate transformation
    /// * `canvas_width` - Canvas width in pixels
    /// * `canvas_height` - Canvas height in pixels
    ///
    /// # Returns
    /// A tuple of (note_id, handle) if a handle is found, or `None` if no handle is under the cursor
    pub fn find_resize_handle_at(
        &self,
        screen_x: f64,
        screen_y: f64,
        viewport: &crate::viewport::ViewportState,
        canvas_width: f64,
        canvas_height: f64,
    ) -> Option<(u32, ResizeHandle)> {
        // Only check handles for the selected note
        if let Some(selected_id) = self.selected_note_id {
            if let Some(selected_note) = self.notes.iter().find(|n| n.id == selected_id) {
                // Check each handle position
                for &handle in &[
                    ResizeHandle::TopLeft,
                    ResizeHandle::Top,
                    ResizeHandle::TopRight,
                    ResizeHandle::Right,
                    ResizeHandle::BottomRight,
                    ResizeHandle::Bottom,
                    ResizeHandle::BottomLeft,
                    ResizeHandle::Left,
                ] {
                    let (left, top, right, bottom) =
                        selected_note.handle_bounds(handle, viewport, canvas_width, canvas_height);

                    // Check if screen point is within handle bounds
                    if screen_x >= left
                        && screen_x <= right
                        && screen_y >= top
                        && screen_y <= bottom
                    {
                        return Some((selected_id, handle));
                    }
                }
            }
        }
        None
    }
    ///
    /// This method initializes the resize state with the original note dimensions
    /// and the handle being used for resizing. The resize operation will continue
    /// until `end_resize()` is called.
    ///
    /// # Arguments
    /// * `note_id` - ID of the note to resize
    /// * `handle` - The resize handle being used
    pub fn start_resize(&mut self, note_id: u32, _handle: ResizeHandle) {
        if let Some(_note) = self.notes.iter().find(|n| n.id == note_id) {
            // Store original dimensions for resize calculations
            self.selected_note_id = Some(note_id);
            // Note: We'll use the ResizingState in AppState to track resize state
        }
    }

    /// Updates the dimensions of the note being resized based on mouse movement.
    ///
    /// This method calculates the new width and height based on the mouse delta
    /// from the resize start position and the type of handle being used.
    /// The resize uses screen coordinates converted to world coordinates using viewport zoom
    /// to ensure consistent behavior regardless of zoom level.
    ///
    /// # Arguments
    /// * `handle` - The resize handle being used
    /// * `start_mouse_x` - Mouse X position when resize started (screen coordinates)
    /// * `start_mouse_y` - Mouse Y position when resize started (screen coordinates)
    /// * `current_mouse_x` - Current mouse X position (screen coordinates)
    /// * `current_mouse_y` - Current mouse Y position (screen coordinates)
    /// * `original_width` - Original width of the note before resizing
    /// * `original_height` - Original height of the note before resizing
    /// * `viewport` - Current viewport state for zoom-based delta conversion
    /// * `viewport_width` - Viewport width (unused, kept for API compatibility)
    /// * `viewport_height` - Viewport height (unused, kept for API compatibility)
    pub fn resize_to(
        &mut self,
        handle: ResizeHandle,
        start_mouse_x: f64,
        start_mouse_y: f64,
        current_mouse_x: f64,
        current_mouse_y: f64,
        original_width: f64,
        original_height: f64,
        viewport: &crate::viewport::ViewportState,
        _viewport_width: f64,
        _viewport_height: f64,
    ) {
        if let Some(note_id) = self.selected_note_id {
            if let Some(note) = self.get_note_mut(note_id) {
                // Convert screen coordinate deltas to world coordinate deltas using viewport zoom
                // This ensures resize speed feels consistent regardless of zoom level
                let delta_x = (current_mouse_x - start_mouse_x) / viewport.zoom;
                let delta_y = (current_mouse_y - start_mouse_y) / viewport.zoom;

                // Calculate new dimensions based on handle type and original dimensions
                match handle {
                    ResizeHandle::TopLeft => {
                        note.width = (original_width - delta_x).max(50.0); // Min width
                        note.height = (original_height - delta_y).max(40.0); // Min height
                        note.x += delta_x; // Move note to maintain bottom-right position
                        note.y += delta_y;
                    }
                    ResizeHandle::Top => {
                        note.width = original_width;
                        note.height = (original_height - delta_y).max(40.0);
                        note.y += delta_y;
                    }
                    ResizeHandle::TopRight => {
                        note.width = (original_width + delta_x).max(50.0);
                        note.height = (original_height - delta_y).max(40.0);
                        note.y += delta_y;
                    }
                    ResizeHandle::Right => {
                        note.width = (original_width + delta_x).max(50.0);
                        note.height = original_height;
                    }
                    ResizeHandle::BottomRight => {
                        note.width = (original_width + delta_x).max(50.0);
                        note.height = (original_height + delta_y).max(40.0);
                    }
                    ResizeHandle::Bottom => {
                        note.width = original_width;
                        note.height = (original_height + delta_y).max(40.0);
                    }
                    ResizeHandle::BottomLeft => {
                        note.width = (original_width - delta_x).max(50.0);
                        note.height = (original_height + delta_y).max(40.0);
                        note.x += delta_x;
                    }
                    ResizeHandle::Left => {
                        note.width = (original_width - delta_x).max(50.0);
                        note.height = original_height;
                        note.x += delta_x;
                    }
                }
            }
        }
    }

    /// Terminates the current resize operation.
    ///
    /// This method should be called when the mouse is released to end
    /// the resize operation.
    pub fn end_resize(&mut self) {
        // Currently no special cleanup needed, but method provided for consistency
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewport::ViewportState;

    #[test]
    fn sticky_note_creation() {
        let note = StickyNote::new(10.0, 20.0);
        assert_eq!(note.x, 10.0);
        assert_eq!(note.y, 20.0);
        assert_eq!(note.width, 200.0);
        assert_eq!(note.height, 150.0);
        assert_eq!(note.content, "New note");
        assert_eq!(note.color, "#ffff88");
        assert!(note.id > 0);
    }

    #[test]
    fn sticky_note_hit_testing() {
        let note = StickyNote::new(0.0, 0.0);
        assert!(note.contains_point(50.0, 50.0)); // Inside
        assert!(note.contains_point(0.0, 0.0)); // Top-left corner
        assert!(note.contains_point(200.0, 150.0)); // Bottom-right corner (inclusive)
        assert!(!note.contains_point(201.0, 151.0)); // Just outside
        assert!(!note.contains_point(-1.0, -1.0)); // Outside
    }

    #[test]
    fn add_note_at_viewport_center_no_existing_notes() {
        let mut state = StickyNotesState::default();
        let viewport = ViewportState::default();

        state.add_note_at_viewport_center(800.0, 600.0, &viewport);

        assert_eq!(state.notes.len(), 1);
        // With default viewport (pan_x=0, pan_y=0, zoom=1), center should be at world (0, 0)
        assert_eq!(state.notes[0].x, 0.0);
        assert_eq!(state.notes[0].y, 0.0);
    }

    #[test]
    fn add_note_at_viewport_center_with_offset() {
        let mut state = StickyNotesState::default();
        let viewport = ViewportState::default();

        // Add first note
        state.add_note_at_viewport_center(800.0, 600.0, &viewport);
        assert_eq!(state.notes[0].x, 0.0);
        assert_eq!(state.notes[0].y, 0.0);

        // Add second note - should be offset
        state.add_note_at_viewport_center(800.0, 600.0, &viewport);
        assert_eq!(state.notes[1].x, 20.0); // 0 + (1 * 20)
        assert_eq!(state.notes[1].y, 20.0); // 0 + (1 * 20)
    }

    #[test]
    fn add_note_at_viewport_center_with_pan_and_zoom() {
        let mut state = StickyNotesState::default();
        let mut viewport = ViewportState::default();
        viewport.pan_x = 100.0;
        viewport.pan_y = 50.0;
        viewport.zoom = 2.0;

        state.add_note_at_viewport_center(800.0, 600.0, &viewport);

        // With pan (100, 50) and zoom 2.0, the world point at screen center (400, 300)
        // should be: world_x = (400 - 400 - 100) / 2 = -50
        // world_y = (300 - 300 - 50) / 2 = -25
        assert_eq!(state.notes[0].x, -50.0);
        assert_eq!(state.notes[0].y, -25.0);
    }

    #[test]
    fn find_note_at_point() {
        let mut state = StickyNotesState::default();
        let note1 = StickyNote::new(0.0, 0.0);
        let note2 = StickyNote::new(300.0, 0.0); // Far apart
        let note1_id = note1.id;
        let note2_id = note2.id;
        state.add_note(note1);
        state.add_note(note2);

        // Should find the top note first (note2 is added last, so it's on top)
        assert_eq!(state.find_note_at(350.0, 50.0), Some(note2_id));
        assert_eq!(state.find_note_at(50.0, 50.0), Some(note1_id));
        assert_eq!(state.find_note_at(500.0, 500.0), None); // Outside both
    }

    #[test]
    fn drag_operations() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        state.add_note(note);

        // Start drag
        state.start_drag(note_id, 150.0, 150.0);
        assert!(state.is_dragging);
        assert_eq!(state.selected_note_id, Some(note_id));

        // Drag to new position
        state.drag_to(200.0, 180.0);
        assert_eq!(state.notes[0].x, 150.0); // 200 - (150 - 100) = 150
        assert_eq!(state.notes[0].y, 130.0); // 180 - (150 - 100) = 130

        // End drag
        state.end_drag();
        assert!(!state.is_dragging);
        assert!(state.drag_offset.is_none());
    }

    #[test]
    fn delete_selected_note() {
        let mut state = StickyNotesState::default();
        let note1 = StickyNote::new(0.0, 0.0);
        let note2 = StickyNote::new(100.0, 100.0);
        let note1_id = note1.id;
        let note2_id = note2.id;
        state.add_note(note1);
        state.add_note(note2);

        // Select and delete note2
        state.selected_note_id = Some(note2_id);
        state.delete_selected();
        assert_eq!(state.notes.len(), 1);
        assert_eq!(state.notes[0].id, note1_id);
        assert!(state.selected_note_id.is_none());
    }

    #[test]
    fn add_note_at_viewport_center_bounds_checking() {
        let mut state = StickyNotesState::default();
        let mut viewport = ViewportState::default();
        // Set zoom to make world coordinates smaller
        viewport.zoom = 0.5;

        // Add note when viewport is very small - should still place note
        state.add_note_at_viewport_center(400.0, 300.0, &viewport);
        assert_eq!(state.notes.len(), 1);
        // Note should be placed at center despite bounds checking
        let expected_x = 0.0; // center with no pan
        let expected_y = 0.0;
        assert_eq!(state.notes[0].x, expected_x);
        assert_eq!(state.notes[0].y, expected_y);
    }

    #[test]
    fn sticky_note_selection() {
        let mut state = StickyNotesState::default();
        let note1 = StickyNote::new(0.0, 0.0);
        let note2 = StickyNote::new(300.0, 0.0);
        let note1_id = note1.id;
        let note2_id = note2.id;
        state.add_note(note1);
        state.add_note(note2);

        // Click on note1
        let found_id = state.find_note_at(50.0, 50.0);
        assert_eq!(found_id, Some(note1_id));

        // Start drag should select the note
        state.start_drag(note1_id, 50.0, 50.0);
        assert_eq!(state.selected_note_id, Some(note1_id));

        // Click on note2 should select it
        let found_id2 = state.find_note_at(350.0, 50.0);
        assert_eq!(found_id2, Some(note2_id));
        state.start_drag(note2_id, 350.0, 50.0);
        assert_eq!(state.selected_note_id, Some(note2_id));
    }

    #[test]
    fn sticky_note_drag_with_zoom() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        state.add_note(note);

        // Simulate zoom by scaling coordinates (this is how screen coords work with zoom)
        // Start drag at screen position (200, 200) which corresponds to world (100, 100) at zoom 1
        state.start_drag(note_id, 200.0, 200.0);

        // Drag to screen position (300, 250) - should move note by (100, 50) in world space
        state.drag_to(300.0, 250.0);
        assert_eq!(state.notes[0].x, 200.0); // 300 - (200 - 100) = 200
        assert_eq!(state.notes[0].y, 150.0); // 250 - (200 - 100) = 150

        state.end_drag();
        assert!(!state.is_dragging);
    }

    #[test]
    fn sticky_note_unique_ids() {
        let note1 = StickyNote::new(0.0, 0.0);
        let note2 = StickyNote::new(10.0, 10.0);
        let note3 = StickyNote::new(20.0, 20.0);

        assert_ne!(note1.id, note2.id);
        assert_ne!(note2.id, note3.id);
        assert_ne!(note1.id, note3.id);
        assert!(note1.id > 0);
        assert!(note2.id > 0);
        assert!(note3.id > 0);
    }

    #[test]
    fn find_resize_handle_at_selected_note() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(0.0, 0.0); // Note at world (0,0) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default(); // zoom=1, pan_x=0, pan_y=0
        let canvas_width = 800.0;
        let canvas_height = 600.0;

        // Center of canvas is at world (0,0), so note top-left is at screen (400, 300)
        // Note extends 200px right and 150px down, so:
        // Top-left: screen (400, 300)
        // Top: screen (500, 300)
        // Top-right: screen (600, 300)
        // Right: screen (600, 375)
        // Bottom-right: screen (600, 450)
        // Bottom: screen (500, 450)
        // Bottom-left: screen (400, 450)
        // Left: screen (400, 375)

        // Test top-left handle
        let result =
            state.find_resize_handle_at(400.0, 300.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, Some((note_id, ResizeHandle::TopLeft)));

        // Test top handle
        let result =
            state.find_resize_handle_at(500.0, 300.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, Some((note_id, ResizeHandle::Top)));

        // Test right handle
        let result =
            state.find_resize_handle_at(600.0, 375.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, Some((note_id, ResizeHandle::Right)));

        // Test bottom-right handle
        let result =
            state.find_resize_handle_at(600.0, 450.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, Some((note_id, ResizeHandle::BottomRight)));
    }

    #[test]
    fn find_resize_handle_at_outside_bounds() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(0.0, 0.0);
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();
        let canvas_width = 800.0;
        let canvas_height = 600.0;

        // Test points far outside handle bounds
        let result =
            state.find_resize_handle_at(100.0, 100.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, None);

        let result =
            state.find_resize_handle_at(600.0, 400.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, None);
    }

    #[test]
    fn find_resize_handle_at_no_selected_note() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(0.0, 0.0);
        state.add_note(note);
        // No note selected
        state.selected_note_id = None;

        let viewport = ViewportState::default();
        let canvas_width = 800.0;
        let canvas_height = 600.0;

        // Should not find any handles when no note is selected
        let result =
            state.find_resize_handle_at(300.0, 225.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, None);
    }

    #[test]
    fn find_resize_handle_at_with_zoom_and_pan() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100)
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let mut viewport = ViewportState::default();
        viewport.zoom = 2.0; // Zoomed in
        viewport.pan_x = 50.0; // Panned right
        viewport.pan_y = 25.0; // Panned down

        let canvas_width = 800.0;
        let canvas_height = 600.0;

        // With zoom=2, pan_x=50, pan_y=25:
        // World (100,100) -> screen (100*2 + 400 + 50, 100*2 + 300 + 25) = (650, 525)
        // Top-left handle is at the note's top-left corner: screen (650, 525)

        let result =
            state.find_resize_handle_at(650.0, 525.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, Some((note_id, ResizeHandle::TopLeft)));
    }

    #[test]
    fn find_resize_handle_at_all_handles() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(0.0, 0.0);
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();
        let canvas_width = 800.0;
        let canvas_height = 600.0;

        // Test all 8 handles
        let test_cases = vec![
            (400.0, 300.0, ResizeHandle::TopLeft),     // Top-left
            (500.0, 300.0, ResizeHandle::Top),         // Top
            (600.0, 300.0, ResizeHandle::TopRight),    // Top-right
            (600.0, 375.0, ResizeHandle::Right),       // Right
            (600.0, 450.0, ResizeHandle::BottomRight), // Bottom-right
            (500.0, 450.0, ResizeHandle::Bottom),      // Bottom
            (400.0, 450.0, ResizeHandle::BottomLeft),  // Bottom-left
            (400.0, 375.0, ResizeHandle::Left),        // Left
        ];

        for (screen_x, screen_y, expected_handle) in test_cases {
            let result = state.find_resize_handle_at(
                screen_x,
                screen_y,
                &viewport,
                canvas_width,
                canvas_height,
            );
            assert_eq!(
                result,
                Some((note_id, expected_handle)),
                "Failed to find handle {:?} at ({}, {})",
                expected_handle,
                screen_x,
                screen_y
            );
        }
    }

    #[test]
    fn find_resize_handle_at_multiple_notes_only_selected() {
        let mut state = StickyNotesState::default();
        let note1 = StickyNote::new(0.0, 0.0);
        let note2 = StickyNote::new(400.0, 0.0); // Far away
        let note1_id = note1.id;
        state.add_note(note1);
        state.add_note(note2);

        // Select only note1
        state.selected_note_id = Some(note1_id);

        let viewport = ViewportState::default();
        let canvas_width = 800.0;
        let canvas_height = 600.0;

        // Should find handle for note1 at its top-left position
        let result =
            state.find_resize_handle_at(400.0, 300.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, Some((note1_id, ResizeHandle::TopLeft)));

        // Should not find handle for note2 (even though it's at a valid position relative to note2)
        // Note2's top-left would be at screen (400*1 + 400 + 0, 0*1 + 300 + 0) = (800, 300)
        let result =
            state.find_resize_handle_at(800.0, 300.0, &viewport, canvas_width, canvas_height);
        assert_eq!(result, None);
    }

    #[test]
    fn resize_to_with_zoom_consistency() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        // Test resize at zoom level 1.0 (normal)
        let mut viewport = ViewportState::default();
        viewport.zoom = 1.0;

        // Start resize from screen position (200, 200) - this is relative to start position
        // With zoom=1, screen delta of 50px should result in 50px world delta
        state.resize_to(
            ResizeHandle::BottomRight,
            200.0, // start_mouse_x
            200.0, // start_mouse_y
            250.0, // current_mouse_x (50px delta)
            230.0, // current_mouse_y (30px delta)
            200.0, // original_width
            150.0, // original_height
            &viewport,
            800.0,
            600.0,
        );

        // Note should grow by (50, 30) in world space
        assert_eq!(state.notes[0].width, 250.0); // 200 + 50
        assert_eq!(state.notes[0].height, 180.0); // 150 + 30

        // Reset note for next test
        state.notes[0].width = 200.0;
        state.notes[0].height = 150.0;

        // Test resize at zoom level 2.0 (zoomed in)
        viewport.zoom = 2.0;

        // Same screen delta (50px, 30px) should result in smaller world delta (25px, 15px)
        // because screen deltas are divided by zoom
        state.resize_to(
            ResizeHandle::BottomRight,
            200.0, // start_mouse_x
            200.0, // start_mouse_y
            250.0, // current_mouse_x (50px delta)
            230.0, // current_mouse_y (30px delta)
            200.0, // original_width
            150.0, // original_height
            &viewport,
            800.0,
            600.0,
        );

        // Note should grow by (25, 15) in world space (50/2, 30/2)
        assert_eq!(state.notes[0].width, 225.0); // 200 + 25
        assert_eq!(state.notes[0].height, 165.0); // 150 + 15

        // Reset note for next test
        state.notes[0].width = 200.0;
        state.notes[0].height = 150.0;

        // Test resize at zoom level 0.5 (zoomed out)
        viewport.zoom = 0.5;

        // Same screen delta (50px, 30px) should result in larger world delta (100px, 60px)
        // because screen deltas are divided by zoom
        state.resize_to(
            ResizeHandle::BottomRight,
            200.0, // start_mouse_x
            200.0, // start_mouse_y
            250.0, // current_mouse_x (50px delta)
            230.0, // current_mouse_y (30px delta)
            200.0, // original_width
            150.0, // original_height
            &viewport,
            800.0,
            600.0,
        );

        // Note should grow by (100, 60) in world space (50/0.5, 30/0.5)
        assert_eq!(state.notes[0].width, 300.0); // 200 + 100
        assert_eq!(state.notes[0].height, 210.0); // 150 + 60
    }
}
