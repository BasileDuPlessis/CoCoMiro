# Sticky Note Feature Tasks

## Overview
Implement a new feature allowing users to create draggable sticky notes on the infinite canvas. Sticky notes should appear centered on screen with slight offsets for new notes.

## Tasks

### 1. Data Model Updates
- Define a `StickyNote` struct with properties: id, position (x, y), size (width, height), content (text), color/style
- Add a `Vec<StickyNote>` to the main app state in `app.rs`
- Implement unique ID generation for sticky notes

**Implementation Details:**
- Create `src/sticky_notes.rs` module
- Use `std::sync::atomic::AtomicU32` for thread-safe ID generation
- Implement `StickyNote::new(x: f64, y: f64) -> Self`
- Add `contains_point(&self, px: f64, py: f64) -> bool` for hit testing
- Update `lib.rs` to include `AppState` with `sticky_notes: StickyNotesState`
- Add `StickyNotesState` with `notes: Vec<StickyNote>`, selection state, and drag state

### 2. Toolbar Enhancement
- Add a "New Sticky Note" button to the toolbar in `toolbar.rs`
- Style the button appropriately (icon or text)
- Connect button click to sticky note creation logic

**Implementation Details:**
- Update `app.rs` HTML markup to include button: `<button id="add-note-button">+</button>`
- Add click event handler in `events.rs` `setup_event_listeners`
- Handler should call `state.sticky_notes.add_note_at_viewport_center(viewport_width, viewport_height, &state.viewport)`
- Ensure button is properly styled and accessible with ARIA labels

### 3. Sticky Note Creation Logic
- Implement function to create new sticky note at center of viewport
- Calculate center position based on current viewport transform
- Apply offset logic: shift new notes slightly right and down from existing notes
- Add the new sticky note to the app state

**Implementation Details:**
- Add `add_note_at_viewport_center(&mut self, viewport_width: f64, viewport_height: f64, viewport_state: &ViewportState)` to `StickyNotesState`
- Use `viewport_state.world_point_at(viewport_width/2.0, viewport_height/2.0, viewport_width, viewport_height)` for center calculation
- Apply offset: `let offset = self.notes.len() as f64 * 20.0; let note_x = center_world_x + offset; let note_y = center_world_y + offset;`
- Create note with `StickyNote::new(note_x, note_y)` and add to `self.notes`

### 4. Rendering Implementation
- Update canvas rendering in `canvas.rs` to draw sticky notes
- Implement sticky note visual appearance (rectangle with text area)
- Handle viewport transformations for sticky note positioning
- Ensure sticky notes render above canvas content but below UI elements

**Implementation Details:**
- Modify `render_canvas` function to iterate through `state.sticky_notes.notes`
- For each note, calculate screen position: `let screen_x = note.x - state.viewport.pan_x; let screen_y = note.y - state.viewport.pan_y;`
- Apply zoom: `screen_x *= state.viewport.zoom; screen_y *= state.viewport.zoom;`
- Draw rectangle: `ctx.fill_rect(screen_x, screen_y, note.width * zoom, note.height * zoom)`
- Draw border and text content
- Handle selection highlighting with different colors
- Ensure rendering happens after grid but before status updates

### 5. Dragging Functionality
- Implement mouse event handling for sticky note dragging in `events.rs`
- Detect when mouse is over a sticky note (hit testing)
- Track drag state (which sticky note is being dragged)
- Update sticky note position during drag operations
- Prevent canvas panning when dragging sticky notes

**Implementation Details:**
- Add drag state to `StickyNotesState`: `is_dragging: bool`, `drag_offset: Option<(f64, f64)>`
- In mousedown handler, check `state.sticky_notes.find_note_at(mouse_x, mouse_y)` first
- If note found, call `state.sticky_notes.start_drag(note_id, mouse_x, mouse_y)` and prevent canvas drag
- In mousemove handler, call `state.sticky_notes.drag_to(mouse_x, mouse_y)` if dragging
- In mouseup handler, call `state.sticky_notes.end_drag()`
- Convert screen coordinates to world coordinates for position updates

### 6. Selection and Interaction
- Add selection state for sticky notes (highlight selected note)
- Implement click-to-select behavior for sticky notes
- Handle keyboard interactions (delete selected sticky note, etc.)
- Update cursor appearance when hovering over draggable sticky notes

**Implementation Details:**
- Add `selected_note_id: Option<u32>` to `StickyNotesState`
- In mousedown handler, if clicking on note, set `selected_note_id = Some(note_id)`
- Add keyboard event handler for Delete key to remove selected note
- Update cursor in `render_canvas`: if hovering over note, set cursor to "grab"
- Add visual selection indicator (thicker border, different color) in rendering

### 7. Positioning and Layout
- Implement logic to determine center position of viewport
- Create offset calculation function (e.g., +20px right, +20px down per new note)
- Ensure sticky notes stay within reasonable bounds
- Handle viewport zoom/pan affecting sticky note positioning

**Implementation Details:**
- Center calculation already handled in task 3
- Offset logic: linear offset based on note count
- Consider bounds checking to keep notes visible
- Ensure zoom affects visual size but not world position
- Add bounds checking in `add_note_at_viewport_center` to prevent notes from being placed off-screen

### 8. Testing and Validation
- Add unit tests for sticky note creation and positioning
- Update E2E tests in `tests/e2e_home.rs` to cover sticky note interactions
- Test dragging behavior across different zoom levels
- Verify sticky notes persist correctly with canvas state

**Implementation Details:**
- Add unit tests in `sticky_notes.rs` for `add_note_at_viewport_center` with different viewport states
- Test hit detection with `contains_point`
- Add E2E test for creating sticky note via toolbar button
- Add E2E test for dragging sticky notes without panning canvas
- Test zoom behavior: notes should scale visually but maintain world positions
- Test selection and keyboard deletion
- Ensure tests use proper waiting for WASM readiness

## Implementation Notes
- Coordinate with existing canvas interaction patterns
- Ensure performance with multiple sticky notes
- Consider text editing capabilities for future enhancement
- Maintain consistent styling with existing UI elements