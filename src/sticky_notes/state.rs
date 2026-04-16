use super::note::StickyNote;
use super::types::ResizeHandle;

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

    /// Initializes the resize state with the original note dimensions
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
        _viewport: &crate::viewport::ViewportState,
        _viewport_width: f64,
        _viewport_height: f64,
    ) {
        if let Some(note_id) = self.selected_note_id {
            if let Some(note) = self.get_note_mut(note_id) {
                // Convert screen coordinate deltas to world coordinate deltas using viewport zoom
                // This ensures resize speed feels consistent regardless of zoom level
                let delta_x = current_mouse_x - start_mouse_x;
                let delta_y = current_mouse_y - start_mouse_y;

                // Calculate new dimensions based on handle type and original dimensions
                match handle {
                    ResizeHandle::TopLeft => Self::resize_top_left(
                        note,
                        delta_x,
                        delta_y,
                        original_width,
                        original_height,
                    ),
                    ResizeHandle::Top => {
                        Self::resize_top(note, delta_x, delta_y, original_width, original_height)
                    }
                    ResizeHandle::TopRight => Self::resize_top_right(
                        note,
                        delta_x,
                        delta_y,
                        original_width,
                        original_height,
                    ),
                    ResizeHandle::Right => {
                        Self::resize_right(note, delta_x, delta_y, original_width, original_height)
                    }
                    ResizeHandle::BottomRight => Self::resize_bottom_right(
                        note,
                        delta_x,
                        delta_y,
                        original_width,
                        original_height,
                    ),
                    ResizeHandle::Bottom => {
                        Self::resize_bottom(note, delta_x, delta_y, original_width, original_height)
                    }
                    ResizeHandle::BottomLeft => Self::resize_bottom_left(
                        note,
                        delta_x,
                        delta_y,
                        original_width,
                        original_height,
                    ),
                    ResizeHandle::Left => {
                        Self::resize_left(note, delta_x, delta_y, original_width, original_height)
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

    /// Resizes the note using the top-left handle, keeping the bottom-right corner fixed.
    fn resize_top_left(
        note: &mut StickyNote,
        delta_x: f64,
        delta_y: f64,
        original_width: f64,
        original_height: f64,
    ) {
        let old_width = note.width;
        let old_height = note.height;
        note.width = (original_width - delta_x).max(50.0);
        note.height = (original_height - delta_y).max(40.0);
        note.x += old_width - note.width; // Move note to keep right edge fixed
        note.y += old_height - note.height; // Move note to keep bottom edge fixed
    }

    /// Resizes the note using the top handle, keeping the bottom edge fixed.
    fn resize_top(
        note: &mut StickyNote,
        _delta_x: f64,
        delta_y: f64,
        original_width: f64,
        original_height: f64,
    ) {
        let old_height = note.height;
        note.width = original_width;
        note.height = (original_height - delta_y).max(40.0);
        note.y += old_height - note.height; // Move note to keep bottom edge fixed
    }

    /// Resizes the note using the top-right handle, keeping the bottom-left corner fixed.
    fn resize_top_right(
        note: &mut StickyNote,
        delta_x: f64,
        delta_y: f64,
        original_width: f64,
        original_height: f64,
    ) {
        note.width = (original_width + delta_x).max(50.0);
        note.height = (original_height - delta_y).max(40.0);
        note.y += delta_y; // Move note to keep bottom-left corner fixed
    }

    /// Resizes the note using the right handle, keeping the left edge fixed.
    fn resize_right(
        note: &mut StickyNote,
        delta_x: f64,
        _delta_y: f64,
        original_width: f64,
        original_height: f64,
    ) {
        note.width = (original_width + delta_x).max(50.0);
        note.height = original_height;
    }

    /// Resizes the note using the bottom-right handle, keeping the top-left corner fixed.
    fn resize_bottom_right(
        note: &mut StickyNote,
        delta_x: f64,
        delta_y: f64,
        original_width: f64,
        original_height: f64,
    ) {
        note.width = (original_width + delta_x).max(50.0);
        note.height = (original_height + delta_y).max(40.0);
    }

    /// Resizes the note using the bottom handle, keeping the top edge fixed.
    fn resize_bottom(
        note: &mut StickyNote,
        _delta_x: f64,
        delta_y: f64,
        original_width: f64,
        original_height: f64,
    ) {
        note.width = original_width;
        note.height = (original_height + delta_y).max(40.0);
    }

    /// Resizes the note using the bottom-left handle, keeping the top-right corner fixed.
    fn resize_bottom_left(
        note: &mut StickyNote,
        delta_x: f64,
        delta_y: f64,
        original_width: f64,
        original_height: f64,
    ) {
        let old_width = note.width;
        note.width = (original_width - delta_x).max(50.0);
        note.height = (original_height + delta_y).max(40.0);
        note.x += old_width - note.width; // Move note to keep right edge fixed
        // note.y stays fixed
    }

    /// Resizes the note using the left handle, keeping the right edge fixed.
    fn resize_left(
        note: &mut StickyNote,
        delta_x: f64,
        _delta_y: f64,
        original_width: f64,
        original_height: f64,
    ) {
        let old_width = note.width;
        note.width = (original_width - delta_x).max(50.0);
        note.height = original_height;
        note.x += old_width - note.width; // Move note to keep right edge fixed
    }
}
