use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, PartialEq)]
pub struct StickyNote {
    pub id: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub content: String,
    pub color: String,
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

impl StickyNote {
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

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StickyNotesState {
    pub notes: Vec<StickyNote>,
    pub selected_note_id: Option<u32>,
    pub is_dragging: bool,
    pub drag_offset: Option<(f64, f64)>,
}

impl Default for StickyNotesState {
    fn default() -> Self {
        Self {
            notes: Vec::new(),
            selected_note_id: None,
            is_dragging: false,
            drag_offset: None,
        }
    }
}

impl StickyNotesState {
    pub fn add_note(&mut self, note: StickyNote) {
        self.notes.push(note);
    }

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

    pub fn start_drag(&mut self, note_id: u32, mouse_x: f64, mouse_y: f64) {
        if let Some(note) = self.notes.iter().find(|n| n.id == note_id) {
            self.is_dragging = true;
            self.drag_offset = Some((mouse_x - note.x, mouse_y - note.y));
            self.selected_note_id = Some(note_id);
        }
    }

    pub fn drag_to(&mut self, mouse_x: f64, mouse_y: f64) {
        if let (true, Some((offset_x, offset_y))) = (self.is_dragging, self.drag_offset) {
            if let Some(note_id) = self.selected_note_id {
                if let Some(note) = self.get_note_mut(note_id) {
                    note.x = mouse_x - offset_x;
                    note.y = mouse_y - offset_y;
                }
            }
        }
    }

    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.drag_offset = None;
    }

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
        let note_x = center_world_x + offset;
        let note_y = center_world_y + offset;
        let note = StickyNote::new(note_x, note_y);
        self.add_note(note);
    }

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
}
