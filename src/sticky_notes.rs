//! # Sticky Notes Module
//!
//! This module manages the creation, storage, and manipulation of sticky notes
//! in the CoCoMiro infinite canvas application.
//!
//! ## Architecture
//!
//! The module consists of two main components:
//! - `StickyNote`: Represents individual sticky notes with position, size, content, and appearance
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

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, PartialEq)]
/// Represents a single sticky note on the infinite canvas.
///
/// Each sticky note has a unique ID, position in world coordinates,
/// dimensions, text content, and background color. Notes are rendered
/// as rectangles with text content and can be dragged around the canvas.
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
    /// Background color as a hex string (e.g., "#ffff88")
    pub color: String,
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

impl StickyNote {
    /// Creates a new sticky note at the specified world coordinates.
    ///
    /// The note is assigned a unique ID, default dimensions (200x150),
    /// default content ("New note"), and default color (#ffff88).
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
}

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
            && let Some(note) = self.get_note_mut(note_id) {
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
    #[cfg(any(test, target_arch = "wasm32"))]
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
}
