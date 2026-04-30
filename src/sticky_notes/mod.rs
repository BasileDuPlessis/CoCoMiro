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

mod note;
mod state;
mod types;

pub use note::*;
pub use state::*;
pub use types::*;

// Re-export ResizeParams for public API
pub use state::ResizeParams;

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
    fn resize_to_with_screen_delta_consistency() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        // Test resize at zoom level 1.0 (normal)
        let mut viewport = ViewportState::default();
        viewport.zoom = 1.0;

        // Start resize from screen position (200, 200) - this is relative to start position
        // Screen delta of 50px should result in 50px world delta regardless of zoom
        let params = ResizeParams {
            start_mouse_x: 200.0,
            start_mouse_y: 200.0,
            current_mouse_x: 250.0, // 50px delta
            current_mouse_y: 230.0, // 30px delta
            original_width: 200.0,
            original_height: 150.0,
        };
        state.resize_to(ResizeHandle::BottomRight, params);

        // Note should grow by (50, 30) in world space
        assert_eq!(state.notes[0].width, 250.0); // 200 + 50
        assert_eq!(state.notes[0].height, 180.0); // 150 + 30

        // Reset note for next test
        state.notes[0].width = 200.0;
        state.notes[0].height = 150.0;

        // Test resize at zoom level 2.0 (zoomed in)
        viewport.zoom = 2.0;

        // Same screen delta (50px, 30px) should result in same world delta (50px, 30px)
        // Screen deltas are not divided by zoom for consistent feel
        let params = ResizeParams {
            start_mouse_x: 200.0,
            start_mouse_y: 200.0,
            current_mouse_x: 250.0, // 50px delta
            current_mouse_y: 230.0, // 30px delta
            original_width: 200.0,
            original_height: 150.0,
        };
        state.resize_to(ResizeHandle::BottomRight, params);

        // Note should grow by (50, 30) in world space (same as zoom=1.0)
        assert_eq!(state.notes[0].width, 250.0); // 200 + 50
        assert_eq!(state.notes[0].height, 180.0); // 150 + 30

        // Reset note for next test
        state.notes[0].width = 200.0;
        state.notes[0].height = 150.0;

        // Test resize at zoom level 0.5 (zoomed out)
        viewport.zoom = 0.5;

        // Same screen delta (50px, 30px) should result in same world delta (50px, 30px)
        // Screen deltas are not divided by zoom for consistent feel
        let params = ResizeParams {
            start_mouse_x: 200.0,
            start_mouse_y: 200.0,
            current_mouse_x: 250.0, // 50px delta
            current_mouse_y: 230.0, // 30px delta
            original_width: 200.0,
            original_height: 150.0,
        };
        state.resize_to(ResizeHandle::BottomRight, params);

        // Note should grow by (50, 30) in world space (same as other zoom levels)
        assert_eq!(state.notes[0].width, 250.0); // 200 + 50
        assert_eq!(state.notes[0].height, 180.0); // 150 + 30
    }

    #[test]
    fn resize_left_handle_keeps_right_edge_fixed() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();

        // Drag left handle 50px to the left (delta_x = -50)
        let params = ResizeParams {
            start_mouse_x: 200.0,
            start_mouse_y: 200.0,
            current_mouse_x: 150.0, // -50px delta
            current_mouse_y: 200.0, // 0px delta
            original_width: 200.0,
            original_height: 150.0,
        };
        state.resize_to(ResizeHandle::Left, params);

        // Width should increase by 50px (from 200 to 250)
        assert_eq!(state.notes[0].width, 250.0);
        // Height should stay the same
        assert_eq!(state.notes[0].height, 150.0);
        // X position should move left by 50px to keep right edge fixed
        // Right edge was at 100 + 200 = 300, should stay at 300
        // New x = 100 - 50 = 50, new width = 250, so right edge = 50 + 250 = 300 ✓
        assert_eq!(state.notes[0].x, 50.0);
        assert_eq!(state.notes[0].y, 100.0); // Y unchanged
    }

    #[test]
    fn resize_top_handle_keeps_bottom_edge_fixed() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();

        // Drag top handle 30px up (delta_y = -30)
        let params = ResizeParams {
            start_mouse_x: 200.0,
            start_mouse_y: 200.0,
            current_mouse_x: 200.0, // 0px delta
            current_mouse_y: 170.0, // -30px delta
            original_width: 200.0,
            original_height: 150.0,
        };
        state.resize_to(ResizeHandle::Top, params);

        // Width should stay the same
        assert_eq!(state.notes[0].width, 200.0);
        // Height should increase by 30px (from 150 to 180)
        assert_eq!(state.notes[0].height, 180.0);
        // Y position should move up by 30px to keep bottom edge fixed
        // Bottom edge was at 100 + 150 = 250, should stay at 250
        // New y = 100 - 30 = 70, new height = 180, so bottom edge = 70 + 180 = 250 ✓
        assert_eq!(state.notes[0].x, 100.0); // X unchanged
        assert_eq!(state.notes[0].y, 70.0);
    }

    #[test]
    fn resize_top_left_handle_keeps_bottom_right_corner_fixed() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();

        // Drag top-left handle 40px left and 25px up (delta_x = -40, delta_y = -25)
        let params = ResizeParams {
            start_mouse_x: 200.0,
            start_mouse_y: 200.0,
            current_mouse_x: 160.0, // -40px delta
            current_mouse_y: 175.0, // -25px delta
            original_width: 200.0,
            original_height: 150.0,
        };
        state.resize_to(ResizeHandle::TopLeft, params);

        // Width should increase by 40px (from 200 to 240)
        assert_eq!(state.notes[0].width, 240.0);
        // Height should increase by 25px (from 150 to 175)
        assert_eq!(state.notes[0].height, 175.0);
        // Bottom-right corner should stay fixed at (100+200, 100+150) = (300, 250)
        // New position: x moves right by 40, y moves down by 25
        // New x = 100 + 40 = 140, new y = 100 + 25 = 125
        // Bottom-right = (140 + 240, 125 + 175) = (380, 300) Wait, that's not right!

        // Let me recalculate:
        // Original bottom-right: (100 + 200, 100 + 150) = (300, 250)
        // When width increases by 40, to keep bottom-right fixed, x should move left by 40
        // When height increases by 25, to keep bottom-right fixed, y should move up by 25
        // So new x = 100 - 40 = 60, new y = 100 - 25 = 75
        // Then bottom-right = (60 + 240, 75 + 175) = (300, 250) ✓

        assert_eq!(state.notes[0].x, 60.0);
        assert_eq!(state.notes[0].y, 75.0);
    }

    #[test]
    fn resize_bottom_left_handle_keeps_top_right_corner_fixed() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();

        // Drag bottom-left handle 35px left and 20px down (delta_x = -35, delta_y = 20)
        let params = ResizeParams {
            start_mouse_x: 200.0,
            start_mouse_y: 200.0,
            current_mouse_x: 165.0, // -35px delta
            current_mouse_y: 220.0, // 20px delta
            original_width: 200.0,
            original_height: 150.0,
        };
        state.resize_to(ResizeHandle::BottomLeft, params);

        // Width should increase by 35px (from 200 to 235)
        assert_eq!(state.notes[0].width, 235.0);
        // Height should increase by 20px (from 150 to 170)
        assert_eq!(state.notes[0].height, 170.0);
        // Top-right corner should stay fixed at (100+200, 100) = (300, 100)
        // When width increases by 35, to keep top-right x fixed, x should move left by 35
        // When height increases by 20, top edge stays fixed (y unchanged)
        // New x = 100 - 35 = 65, y = 100
        // Top-right = (65 + 235, 100) = (300, 100) ✓

        assert_eq!(state.notes[0].x, 65.0);
        assert_eq!(state.notes[0].y, 100.0);
    }

    #[test]
    fn resize_top_right_handle_keeps_bottom_left_corner_fixed() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();

        // Drag top-right handle 45px right and 15px up (delta_x = 45, delta_y = -15)
        state.resize_to(
            ResizeHandle::TopRight,
            ResizeParams {
                start_mouse_x: 200.0,
                start_mouse_y: 200.0,
                current_mouse_x: 245.0, // 45px delta
                current_mouse_y: 185.0, // -15px delta
                original_width: 200.0,
                original_height: 150.0,
            },
        );

        // Width should increase by 45px (from 200 to 245)
        assert_eq!(state.notes[0].width, 245.0);
        // Height should increase by 15px (from 150 to 165)
        assert_eq!(state.notes[0].height, 165.0);
        // Bottom-left corner should stay fixed at (100, 100+150) = (100, 250)
        // When width increases by 45, left edge stays fixed (x unchanged)
        // When height increases by 15, to keep bottom-left y fixed, y should move up by 15
        // New x = 100, new y = 100 - 15 = 85
        // Bottom-left = (100, 85 + 165) = (100, 250) ✓

        assert_eq!(state.notes[0].x, 100.0);
        assert_eq!(state.notes[0].y, 85.0);
    }

    #[test]
    fn resize_right_handle_resizes_from_center() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();

        // Drag right handle 60px to the right (delta_x = 60)
        state.resize_to(
            ResizeHandle::Right,
            ResizeParams {
                start_mouse_x: 200.0,
                start_mouse_y: 200.0,
                current_mouse_x: 260.0, // 60px delta
                current_mouse_y: 200.0, // 0px delta
                original_width: 200.0,
                original_height: 150.0,
            },
        );

        // Width should increase by 60px (from 200 to 260)
        assert_eq!(state.notes[0].width, 260.0);
        // Height should stay the same
        assert_eq!(state.notes[0].height, 150.0);
        // Position should stay the same (resize from center)
        assert_eq!(state.notes[0].x, 100.0);
        assert_eq!(state.notes[0].y, 100.0);
    }

    #[test]
    fn resize_bottom_handle_resizes_from_center() {
        let mut state = StickyNotesState::default();
        let note = StickyNote::new(100.0, 100.0); // Note at world (100,100) with size 200x150
        let note_id = note.id;
        state.add_note(note);
        state.selected_note_id = Some(note_id);

        let viewport = ViewportState::default();

        // Drag bottom handle 40px down (delta_y = 40)
        state.resize_to(
            ResizeHandle::Bottom,
            ResizeParams {
                start_mouse_x: 200.0,
                start_mouse_y: 200.0,
                current_mouse_x: 200.0, // 0px delta
                current_mouse_y: 240.0, // 40px delta
                original_width: 200.0,
                original_height: 150.0,
            },
        );

        // Width should stay the same
        assert_eq!(state.notes[0].width, 200.0);
        // Height should increase by 40px (from 150 to 190)
        assert_eq!(state.notes[0].height, 190.0);
        // Position should stay the same (resize from center)
        assert_eq!(state.notes[0].x, 100.0);
        assert_eq!(state.notes[0].y, 100.0);
    }

    #[test]
    fn text_clipping_with_long_content() {
        // Test that notes with very long text content can be created and handled properly
        // This verifies that the text clipping implementation doesn't break basic note functionality
        let mut state = StickyNotesState::default();

        // Create a note with extremely long text that would definitely overflow
        let long_text = format!(
            "{}{}{}{}{}",
            "This is a very long line of text that should exceed the width of a standard sticky note and wrap to multiple lines. ".repeat(20),
            "\n\n",
            "This is another paragraph with more text that continues the long content test. ".repeat(15),
            "\n\n",
            "<b>Bold text</b> and <i>italic text</i> should also work properly with text clipping enabled. ".repeat(10)
        );

        let note = StickyNote {
            id: 1,
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 150.0,
            content: long_text.clone(),
            formatting: Vec::new(),
            color: "#ffff88".to_string(),
        };

        state.add_note(note);

        // Verify the note was added successfully
        assert_eq!(state.notes.len(), 1);
        assert_eq!(state.notes[0].content, long_text); // Verify content matches exactly
        assert_eq!(state.notes[0].width, 200.0);
        assert_eq!(state.notes[0].height, 150.0);

        // Test that the note can still be found and selected
        let found_id = state.find_note_at(50.0, 50.0);
        assert_eq!(found_id, Some(1));

        // Test selection and dragging still work
        state.start_drag(1, 50.0, 50.0);
        assert_eq!(state.selected_note_id, Some(1));
        assert!(state.is_dragging);

        state.drag_to(100.0, 100.0);
        assert_eq!(state.notes[0].x, 50.0); // 100.0 - 50.0 offset
        assert_eq!(state.notes[0].y, 50.0); // 100.0 - 50.0 offset

        state.end_drag();
        assert!(!state.is_dragging);
    }
}
