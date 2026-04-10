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
            content: String::new(),
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
}
