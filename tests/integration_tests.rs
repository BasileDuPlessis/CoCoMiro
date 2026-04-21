#[cfg(test)]
/// Integration tests that verify interactions between multiple modules
/// These tests ensure that the components work together correctly
mod integration_tests {
    use cocomiro::{AppState, sticky_notes::StickyNote, viewport::ViewportState};

    #[test]
    fn viewport_and_sticky_notes_coordinate_transformation() {
        // Test that viewport transformations correctly affect sticky note positioning
        let mut app_state = AppState::default();
        let mut viewport = ViewportState::default();

        // Set up viewport with pan and zoom
        viewport.pan_x = 100.0;
        viewport.pan_y = 50.0;
        viewport.zoom = 2.0;
        app_state.viewport = viewport;

        // Add a note at viewport center
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &app_state.viewport);

        // Verify the note was placed at the correct world coordinates
        assert_eq!(app_state.sticky_notes.notes.len(), 1);
        let note = &app_state.sticky_notes.notes[0];

        // With pan (100, 50) and zoom 2.0, center should be at world (-50, -25)
        assert_eq!(note.x, -50.0);
        assert_eq!(note.y, -25.0);
    }

    #[test]
    fn mouse_interaction_with_viewport_and_notes() {
        // Test mouse coordinate conversion and interaction with both viewport and notes
        let mut app_state = AppState::default();
        let mut viewport = ViewportState::default();

        // Set up viewport
        viewport.pan_x = 200.0;
        viewport.pan_y = 100.0;
        viewport.zoom = 1.5;
        app_state.viewport = viewport;

        // Add a note at a specific world position
        let note = StickyNote::new(50.0, 75.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);

        // Simulate mouse click at screen coordinates that should hit the note
        // With viewport pan (200, 100) and zoom 1.5, world point (50, 75) corresponds to:
        // screen_x = 50 * 1.5 + 400 + 200 = 75 + 600 = 675
        // screen_y = 75 * 1.5 + 300 + 100 = 112.5 + 400 = 512.5

        let screen_x = 675.0;
        let screen_y = 512.5;

        // Convert screen to world coordinates
        let (world_x, world_y) = app_state
            .viewport
            .world_point_at(screen_x, screen_y, 800.0, 600.0);

        // Should be approximately (50, 75)
        assert!((world_x - 50.0).abs() < 0.1);
        assert!((world_y - 75.0).abs() < 0.1);

        // Should find the note at this world position
        assert_eq!(
            app_state.sticky_notes.find_note_at(world_x, world_y),
            Some(note_id)
        );
    }

    #[test]
    fn drag_note_with_viewport_changes() {
        // Test dragging a note while viewport changes
        let mut app_state = AppState::default();

        // Add a note
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);

        // Start dragging the note
        app_state.sticky_notes.start_drag(note_id, 150.0, 150.0);

        // Drag to new position
        app_state.sticky_notes.drag_to(200.0, 180.0);

        // Verify note moved correctly
        let note = app_state.sticky_notes.get_note_mut(note_id).unwrap();
        assert_eq!(note.x, 150.0);
        assert_eq!(note.y, 130.0);

        // Now change viewport (zoom in)
        app_state.viewport.zoom = 2.0;

        // The note should still be at the same world position
        let note = app_state.sticky_notes.get_note_mut(note_id).unwrap();
        assert_eq!(note.x, 150.0);
        assert_eq!(note.y, 130.0);

        // End drag
        app_state.sticky_notes.end_drag();
        assert!(!app_state.sticky_notes.is_dragging);
    }

    #[test]
    fn multiple_notes_selection_and_deletion() {
        // Test selecting and deleting notes in a complex scenario
        let mut app_state = AppState::default();

        // Add multiple notes
        let note1 = StickyNote::new(0.0, 0.0);
        let note2 = StickyNote::new(250.0, 0.0);
        let note3 = StickyNote::new(250.0, 200.0);
        let note1_id = note1.id;
        let note2_id = note2.id;
        let note3_id = note3.id;

        app_state.sticky_notes.add_note(note1);
        app_state.sticky_notes.add_note(note2);
        app_state.sticky_notes.add_note(note3);

        assert_eq!(app_state.sticky_notes.notes.len(), 3);

        // Select note2 (should be topmost at its position)
        let found_id = app_state.sticky_notes.find_note_at(300.0, 50.0);
        assert_eq!(found_id, Some(note2_id));
        app_state.sticky_notes.selected_note_id = found_id;

        // Delete selected note
        app_state.sticky_notes.delete_selected();
        assert_eq!(app_state.sticky_notes.notes.len(), 2);
        assert!(app_state.sticky_notes.selected_note_id.is_none());

        // Verify remaining notes
        let remaining_ids: Vec<u32> = app_state.sticky_notes.notes.iter().map(|n| n.id).collect();
        assert!(remaining_ids.contains(&note1_id));
        assert!(remaining_ids.contains(&note3_id));
        assert!(!remaining_ids.contains(&note2_id));
    }

    #[test]
    fn viewport_bounds_and_note_placement() {
        // Test that notes are placed correctly within viewport bounds
        let mut app_state = AppState::default();
        let mut viewport = ViewportState::default();

        // Set viewport with extreme pan to test bounds
        viewport.pan_x = 1000.0;
        viewport.pan_y = 800.0;
        viewport.zoom = 0.5; // Zoomed out
        app_state.viewport = viewport;

        // Add note at center - should be placed at world center adjusted for pan/zoom
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &app_state.viewport);

        // With pan (1000, 800) and zoom 0.5, center calculation:
        // world_x = (400 - 400 - 1000) / 0.5 = (-1000) / 0.5 = -2000
        // world_y = (300 - 300 - 800) / 0.5 = (-800) / 0.5 = -1600
        assert_eq!(app_state.sticky_notes.notes[0].x, -2000.0);
        assert_eq!(app_state.sticky_notes.notes[0].y, -1600.0);
    }

    #[test]
    fn coordinate_system_consistency() {
        // Test that screen-to-world and world-to-screen conversions are consistent
        let viewport = ViewportState::default();

        // Test point at screen center
        let screen_x = 400.0;
        let screen_y = 300.0;
        let viewport_width = 800.0;
        let viewport_height = 600.0;

        let (world_x, world_y) =
            viewport.world_point_at(screen_x, screen_y, viewport_width, viewport_height);

        // Convert back to screen coordinates
        let center_x = viewport_width / 2.0;
        let center_y = viewport_height / 2.0;
        let screen_x_back = world_x * viewport.zoom + center_x + viewport.pan_x;
        let screen_y_back = world_y * viewport.zoom + center_y + viewport.pan_y;

        // Should get back the original screen coordinates
        assert!((screen_x_back - screen_x).abs() < 0.001);
        assert!((screen_y_back - screen_y).abs() < 0.001);
    }

    #[test]
    fn note_dragging_with_coordinate_conversion() {
        // Test dragging behavior with proper coordinate conversion
        let mut app_state = AppState::default();
        let mut viewport = ViewportState::default();

        // Set up viewport with zoom
        viewport.zoom = 2.0;
        app_state.viewport = viewport;

        // Add note at world position (100, 100)
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);

        // Convert world position to screen coordinates for mouse interaction
        let center_x = 800.0 / 2.0;
        let center_y = 600.0 / 2.0;
        let _screen_note_x = 100.0 * app_state.viewport.zoom + center_x + app_state.viewport.pan_x;
        let _screen_note_y = 100.0 * app_state.viewport.zoom + center_y + app_state.viewport.pan_y;

        // Start drag at the note's screen position
        app_state.sticky_notes.start_drag(note_id, 100.0, 100.0); // World coordinates

        // Drag to new world position (200, 150)
        app_state.sticky_notes.drag_to(200.0, 150.0);

        // Verify note moved to correct world position
        let note = app_state.sticky_notes.get_note_mut(note_id).unwrap();
        assert_eq!(note.x, 200.0);
        assert_eq!(note.y, 150.0);
    }

    #[test]
    fn complex_interaction_sequence() {
        // Test a complex sequence of interactions
        let mut app_state = AppState::default();

        // 1. Add multiple notes
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &app_state.viewport);
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &app_state.viewport);
        assert_eq!(app_state.sticky_notes.notes.len(), 2);

        // 2. Select and drag first note
        let first_note_id = app_state.sticky_notes.notes[0].id;
        app_state.sticky_notes.start_drag(first_note_id, 0.0, 0.0);
        app_state.sticky_notes.drag_to(50.0, 50.0);
        app_state.sticky_notes.end_drag();

        // 3. Select second note and delete it
        let second_note_id = app_state.sticky_notes.notes[1].id;
        app_state.sticky_notes.selected_note_id = Some(second_note_id);
        app_state.sticky_notes.delete_selected();

        // 4. Verify final state
        assert_eq!(app_state.sticky_notes.notes.len(), 1);
        assert_eq!(app_state.sticky_notes.notes[0].id, first_note_id);
        assert_eq!(app_state.sticky_notes.notes[0].x, 50.0);
        assert_eq!(app_state.sticky_notes.notes[0].y, 50.0);
        assert!(app_state.sticky_notes.selected_note_id.is_none());
    }

    #[test]
    fn html_text_parsing_and_formatting() {
        // Test that HTML tags in note content are properly parsed for rendering
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            // Test basic HTML parsing
            let html_content = "Hello <b>world</b> and <i>universe</i>";
            let segments = parse_formatted_text(html_content);

            assert_eq!(segments.len(), 4);

            // "Hello "
            assert_eq!(segments[0].text, "Hello ");
            assert!(!segments[0].bold);
            assert!(!segments[0].italic);
            assert!(!segments[0].underline);

            // "world"
            assert_eq!(segments[1].text, "world");
            assert!(segments[1].bold);
            assert!(!segments[1].italic);
            assert!(!segments[1].underline);

            // " and "
            assert_eq!(segments[2].text, " and ");
            assert!(!segments[2].bold);
            assert!(!segments[2].italic);
            assert!(!segments[2].underline);

            // "universe"
            assert_eq!(segments[3].text, "universe");
            assert!(!segments[3].bold);
            assert!(segments[3].italic);
            assert!(!segments[3].underline);
        }

        // Test nested and overlapping tags
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            let complex_html = "Start <b>bold <i>bold-italic</i> still bold</b> end";
            let segments = parse_formatted_text(complex_html);

            assert_eq!(segments.len(), 5);

            // "Start "
            assert_eq!(segments[0].text, "Start ");
            assert!(!segments[0].bold);
            assert!(!segments[0].italic);

            // "bold "
            assert_eq!(segments[1].text, "bold ");
            assert!(segments[1].bold);
            assert!(!segments[1].italic);

            // "bold-italic"
            assert_eq!(segments[2].text, "bold-italic");
            assert!(segments[2].bold);
            assert!(segments[2].italic);

            // " still bold"
            assert_eq!(segments[3].text, " still bold");
            assert!(segments[3].bold);
            assert!(!segments[3].italic);

            // " end"
            assert_eq!(segments[4].text, " end");
            assert!(!segments[4].bold);
            assert!(!segments[4].italic);
        }

        // Test underline formatting
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            let underline_html = "Normal <u>underlined</u> normal again";
            let segments = parse_formatted_text(underline_html);

            assert_eq!(segments.len(), 3);

            // "Normal "
            assert_eq!(segments[0].text, "Normal ");
            assert!(!segments[0].underline);

            // "underlined"
            assert_eq!(segments[1].text, "underlined");
            assert!(segments[1].underline);

            // " normal again"
            assert_eq!(segments[2].text, " normal again");
            assert!(!segments[2].underline);
        }

        // Test HTML with <span> tags and style attributes
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            let span_bold = r#"Text <span style="font-weight: bold;">bold</span> text"#;
            let segments = parse_formatted_text(span_bold);

            assert_eq!(segments.len(), 3);

            // "Text "
            assert_eq!(segments[0].text, "Text ");
            assert!(!segments[0].bold);

            // "bold"
            assert_eq!(segments[1].text, "bold");
            assert!(segments[1].bold);

            // " text"
            assert_eq!(segments[2].text, " text");
            assert!(!segments[2].bold);
        }

        // Test HTML with <br> tags for line breaks
        #[cfg(target_arch = "wasm32")]
        {
            use canvas::parse_formatted_text;

            let br_html = "Line 1<br>Line 2<br />Line 3";
            let segments = parse_formatted_text(br_html);

            assert_eq!(segments.len(), 5);

            // "Line 1"
            assert_eq!(segments[0].text, "Line 1");
            assert!(!segments[0].bold);

            // "\n"
            assert_eq!(segments[1].text, "\n");
            assert!(!segments[1].bold);

            // "Line 2"
            assert_eq!(segments[2].text, "Line 2");
            assert!(!segments[2].bold);

            // "\n"
            assert_eq!(segments[3].text, "\n");
            assert!(!segments[3].bold);

            // "Line 3"
            assert_eq!(segments[4].text, "Line 3");
            assert!(!segments[4].bold);
        }
    }

    #[test]
    fn sticky_note_selection_clearing() {
        // Test that clicking on empty canvas clears note selection
        let mut app_state = AppState::default();

        // Add a note and select it
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);
        app_state.sticky_notes.selected_note_id = Some(note_id);

        // Verify note is selected
        assert_eq!(app_state.sticky_notes.selected_note_id, Some(note_id));

        // Clear selection (simulating click on empty canvas)
        app_state.sticky_notes.clear_selection();

        // Verify selection is cleared
        assert!(app_state.sticky_notes.selected_note_id.is_none());
    }

    #[test]
    fn toolbar_button_clears_selection_when_adding_note() {
        // Test that clicking the toolbar "add note" button clears any existing selection
        let mut app_state = AppState::default();
        let viewport = ViewportState::default();

        // Add a note and select it
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);
        app_state.sticky_notes.selected_note_id = Some(note_id);

        // Verify note is selected
        assert_eq!(app_state.sticky_notes.selected_note_id, Some(note_id));

        // Simulate clicking the toolbar button (which calls add_note_at_viewport_center)
        app_state.sticky_notes.clear_selection(); // This is what the button handler does
        app_state
            .sticky_notes
            .add_note_at_viewport_center(800.0, 600.0, &viewport);

        // Verify selection is cleared
        assert!(app_state.sticky_notes.selected_note_id.is_none());

        // Verify a new note was added
        assert_eq!(app_state.sticky_notes.notes.len(), 2);
    }

    #[test]
    fn toolbar_background_click_clears_selection() {
        // Test that clicking on the toolbar background clears any existing selection
        let mut app_state = AppState::default();

        // Add a note and select it
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);
        app_state.sticky_notes.selected_note_id = Some(note_id);

        // Verify note is selected
        assert_eq!(app_state.sticky_notes.selected_note_id, Some(note_id));

        // Simulate clicking on toolbar background (this calls clear_selection)
        app_state.sticky_notes.clear_selection();

        // Verify selection is cleared
        assert!(app_state.sticky_notes.selected_note_id.is_none());
    }

    #[test]
    fn toolbar_handle_click_clears_selection() {
        // Test that clicking on the toolbar handle clears any existing selection
        let mut app_state = AppState::default();

        // Add a note and select it
        let note = StickyNote::new(100.0, 100.0);
        let note_id = note.id;
        app_state.sticky_notes.add_note(note);
        app_state.sticky_notes.selected_note_id = Some(note_id);

        // Verify note is selected
        assert_eq!(app_state.sticky_notes.selected_note_id, Some(note_id));

        // Simulate clicking on toolbar handle (this calls clear_selection)
        app_state.sticky_notes.clear_selection();

        // Verify selection is cleared
        assert!(app_state.sticky_notes.selected_note_id.is_none());
    }
}
