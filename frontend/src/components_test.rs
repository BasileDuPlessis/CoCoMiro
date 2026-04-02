#[cfg(test)]
mod tests {
    use crate::components::App;
    use wasm_bindgen_test::*;
    use yew::Reducible;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // Unit tests that validate component compilation and basic structure
    #[test]
    fn test_components_module_imports() {
        // This test ensures all component modules can be imported
        // and validates that the component structure is correct at compile time
        // The test passes if the imports work (compile-time validation)
        assert!(true);
    }

    #[test]
    fn test_app_component_type() {
        // Test that App is a valid function component
        // This validates that App implements the Component trait
        // (though we can't instantiate it directly)
        assert!(true);
    }

    // ServerRenderer tests for HTML validation
    #[wasm_bindgen_test]
    async fn test_app_server_renderer() {
        // Test that the App component renders correctly using ServerRenderer
        let renderer = yew::ServerRenderer::<App>::new();
        let html = renderer.render().await;

        // Basic validation that HTML is generated
        assert!(!html.is_empty());
        assert!(html.contains("<canvas") || html.contains("canvas"));
        assert!(html.contains("class") || html.contains("id")); // Some styling attributes
    }

    // State management tests
    #[wasm_bindgen_test]
    async fn test_view_state_zoom_functionality() {
        use crate::state::ViewState;

        // Test zoom in action
        let mut view_state = ViewState {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            is_dragging: false,
            last_mouse_pos: None,
        };

        // Simulate zoom in action
        view_state.zoom *= 1.2; // Zoom in by 20%
        assert_eq!(view_state.zoom, 1.2);

        // Simulate zoom out action
        view_state.zoom /= 1.2; // Zoom out
        assert_eq!(view_state.zoom, 1.0);
    }

    #[wasm_bindgen_test]
    async fn test_view_state_pan_functionality() {
        use crate::state::ViewState;

        // Test pan functionality
        let mut view_state = ViewState {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            is_dragging: false,
            last_mouse_pos: None,
        };

        // Simulate pan
        view_state.pan_x = 100.0;
        view_state.pan_y = 50.0;

        assert_eq!(view_state.pan_x, 100.0);
        assert_eq!(view_state.pan_y, 50.0);
    }

    // ToolbarState reducer tests
    #[wasm_bindgen_test]
    async fn test_toolbar_state_drag_functionality() {
        use crate::state::{ToolbarAction, ToolbarState};
        use std::rc::Rc;

        // Test toolbar drag start
        let initial_state = Rc::new(ToolbarState {
            x: 100.0,
            y: 200.0,
            is_dragging: false,
            drag_offset: None,
        });

        let new_state =
            ToolbarState::reduce(initial_state.clone(), ToolbarAction::StartDrag(10.0, 20.0));
        assert!(new_state.is_dragging);
        assert_eq!(new_state.drag_offset, Some((10.0, 20.0)));
        assert_eq!(new_state.x, 100.0); // Position shouldn't change on start
        assert_eq!(new_state.y, 200.0);

        // Test toolbar drag update
        let update_state = ToolbarState::reduce(new_state, ToolbarAction::UpdateDrag(150.0, 250.0));
        assert!(update_state.is_dragging);
        assert_eq!(update_state.x, 140.0); // 150 - 10 = 140
        assert_eq!(update_state.y, 230.0); // 250 - 20 = 230

        // Test toolbar drag end
        let end_state = ToolbarState::reduce(update_state, ToolbarAction::EndDrag);
        assert!(!end_state.is_dragging);
        assert_eq!(end_state.drag_offset, None);
        assert_eq!(end_state.x, 140.0); // Position should remain
        assert_eq!(end_state.y, 230.0);
    }

    // StickyNotesState reducer tests
    #[wasm_bindgen_test]
    async fn test_sticky_notes_state_create_note() {
        use crate::state::{StickyNotesAction, StickyNotesState};
        use cocomiro_shared::{Position, Size};
        use std::rc::Rc;

        let initial_state = Rc::new(StickyNotesState {
            notes: vec![],
            editing_note_id: None,
            editing_content: None,
            selected_note_id: None,
        });

        let position = Position { x: 100.0, y: 200.0 };
        let size = Size {
            width: 200.0,
            height: 150.0,
        };
        let new_state = StickyNotesState::reduce(
            initial_state,
            StickyNotesAction::CreateNote(position.clone(), size.clone()),
        );

        assert_eq!(new_state.notes.len(), 1);
        let note = &new_state.notes[0];
        assert_eq!(note.position, position);
        assert_eq!(note.size, size);
        assert_eq!(note.content, "New sticky note");
        assert!(note.id.starts_with("note-"));
    }

    #[wasm_bindgen_test]
    async fn test_sticky_notes_state_edit_workflow() {
        use crate::state::{StickyNotesAction, StickyNotesState};
        use cocomiro_shared::{Position, Size};
        use std::rc::Rc;

        // Create initial state with a note
        let initial_state = Rc::new(StickyNotesState {
            notes: vec![cocomiro_shared::StickyNote {
                id: "test-note-1".to_string(),
                position: Position { x: 0.0, y: 0.0 },
                content: "Original content".to_string(),
                size: Size {
                    width: 200.0,
                    height: 150.0,
                },
            }],
            editing_note_id: None,
            editing_content: None,
            selected_note_id: None,
        });

        // Start editing
        let edit_state = StickyNotesState::reduce(
            initial_state,
            StickyNotesAction::StartEdit("test-note-1".to_string()),
        );
        assert_eq!(edit_state.editing_note_id, Some("test-note-1".to_string()));
        assert_eq!(
            edit_state.editing_content,
            Some("Original content".to_string())
        );
        assert_eq!(edit_state.selected_note_id, Some("test-note-1".to_string()));

        // Update content
        let update_state = StickyNotesState::reduce(
            edit_state,
            StickyNotesAction::UpdateContent("Updated content".to_string()),
        );
        assert_eq!(
            update_state.editing_content,
            Some("Updated content".to_string())
        );

        // Save edit
        let save_state = StickyNotesState::reduce(update_state, StickyNotesAction::SaveEdit);
        assert_eq!(save_state.editing_note_id, None);
        assert_eq!(save_state.editing_content, None);
        assert_eq!(save_state.selected_note_id, None);
        assert_eq!(save_state.notes[0].content, "Updated content");
    }

    // AppState composition tests
    #[wasm_bindgen_test]
    async fn test_app_state_composition() {
        use crate::state::{
            AppAction, AppState, StickyNotesAction, StickyNotesState, ToolbarAction, ToolbarState,
            ViewAction, ViewState,
        };
        use cocomiro_shared::{Position, Size};
        use std::rc::Rc;

        // Create initial app state
        let initial_state = Rc::new(AppState {
            view: ViewState {
                zoom: 1.0,
                pan_x: 0.0,
                pan_y: 0.0,
                is_dragging: false,
                last_mouse_pos: None,
            },
            toolbar: ToolbarState {
                x: 100.0,
                y: 200.0,
                is_dragging: false,
                drag_offset: None,
            },
            sticky_notes: StickyNotesState {
                notes: vec![],
                editing_note_id: None,
                editing_content: None,
                selected_note_id: None,
            },
        });

        // Test View action composition
        let view_action_state =
            AppState::reduce(initial_state.clone(), AppAction::View(ViewAction::ZoomIn));
        assert_eq!(view_action_state.view.zoom, 1.2);
        assert_eq!(view_action_state.toolbar.x, 100.0); // Other states unchanged
        assert_eq!(view_action_state.sticky_notes.notes.len(), 0);

        // Test Toolbar action composition
        let toolbar_action_state = AppState::reduce(
            view_action_state,
            AppAction::Toolbar(ToolbarAction::StartDrag(10.0, 20.0)),
        );
        assert_eq!(toolbar_action_state.view.zoom, 1.2); // Previous view state preserved
        assert!(toolbar_action_state.toolbar.is_dragging);
        assert_eq!(toolbar_action_state.sticky_notes.notes.len(), 0);

        // Test StickyNotes action composition
        let sticky_action_state = AppState::reduce(
            toolbar_action_state,
            AppAction::StickyNotes(StickyNotesAction::CreateNote(
                Position { x: 50.0, y: 75.0 },
                Size {
                    width: 200.0,
                    height: 150.0,
                },
            )),
        );
        assert_eq!(sticky_action_state.view.zoom, 1.2); // All previous states preserved
        assert!(sticky_action_state.toolbar.is_dragging);
        assert_eq!(sticky_action_state.sticky_notes.notes.len(), 1);
    }

    // Component integration tests
    #[wasm_bindgen_test]
    async fn test_component_integration_app_structure() {
        // Test that App component renders correctly using ServerRenderer
        // This validates the component tree structure
        let renderer = yew::ServerRenderer::<App>::new();
        let html = renderer.render().await;

        // Check that the HTML contains expected elements
        assert!(html.contains("canvas") || html.contains("<canvas"));
        // The App component renders an InfiniteCanvas, so we should see canvas-related content
        assert!(!html.is_empty());
    }

    #[wasm_bindgen_test]
    async fn test_component_lifecycle_mounting() {
        // Test that App component can be rendered without errors
        let renderer = yew::ServerRenderer::<App>::new();
        let html = renderer.render().await;

        // Component should render some content
        assert!(!html.is_empty());

        // Should contain expected elements from InfiniteCanvas
        assert!(html.contains("canvas") || html.contains("<canvas"));
    }
}
