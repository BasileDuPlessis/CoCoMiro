# Tasks Backlog

## Overview
This backlog contains tasks to improve the CoCoMiro infinite canvas application.

**Current Status (April 2026):**
- ✅ Basic text editing with contenteditable overlay
- ✅ HTML sanitization (strips all formatting on paste)
- ✅ Rich text toolbar with formatting buttons
- ✅ Paste handling with content sanitization
- ✅ Basic accessibility features (ARIA labels)
- ✅ Performance monitoring (FPS tracking)
- ✅ Basic visual polish (box shadows)
- ❌ No persistence, undo/redo system, or advanced performance optimizations
- ❌ No mobile support

**Implemented Features:**
- Double-click to edit sticky notes
- ContentEditable text input with toolbar
- HTML formatting during editing (bold, italic, underline)
- Paste sanitization (strips all HTML to plain text)
- Basic accessibility (ARIA labels)
- Performance metrics collection
- Multi-line text support
- Text selection and cursor positioning

**Not Yet Implemented:**
- Persistence (save/load)
- Undo/Redo system
- Spatial indexing for performance
- View culling
- Mobile/touch support
- Advanced accessibility
- Visual animations and effects

## Active High Priority Tasks

### 1. Implement Sticky Note Resizing
- Add resize handles to sticky notes for drag-and-drop resizing
- Support corner and edge handles with appropriate cursors
- Maintain minimum size constraints and smooth interactions

**Subtasks:**

#### ✅ 1.1 Verify StickyNote Structure
- Confirm `StickyNote` struct already has `width` and `height` fields
- Ensure existing code uses these fields properly
- Add any missing documentation for size fields

**Implementation Details:**
- Check `src/sticky_notes.rs` for current `StickyNote` implementation
- Verify width/height are used in rendering and hit testing
- Update documentation if needed

**Verification Results:**
- ✅ `StickyNote` struct has `width: f64` and `height: f64` fields with proper documentation
- ✅ `contains_point()` method correctly uses `width` and `height` for hit testing
- ✅ `render_sticky_notes()` in `canvas.rs` uses `note.width * zoom` and `note.height * zoom` for rendering
- ✅ Tests verify default dimensions (200.0 x 150.0) and hit testing behavior
- ✅ Code compiles for both host and WebAssembly targets
- ✅ All existing tests pass

#### ✅ 1.2 Define Resize Handle Types and Positions
- Create `ResizeHandle` enum for 8 handle positions (corners + midpoints)
- Implement handle position calculation methods
- Add handle size and visual properties

**Implementation Details:**
- Define enum: `TopLeft`, `Top`, `TopRight`, `Right`, `BottomRight`, `Bottom`, `BottomLeft`, `Left`
- Add `handle_positions()` method to `StickyNote` returning screen coordinates
- Define handle size constants (e.g., 8x8 pixels)

**Implementation Results:**
- ✅ Created `ResizeHandle` enum with 8 variants for all handle positions
- ✅ Added `cursor()` method to `ResizeHandle` returning appropriate CSS cursor styles
- ✅ Added `handle_position()` method to `StickyNote` for individual handle world coordinates
- ✅ Added `handle_bounds()` method for screen-space bounding boxes (for hit testing)
- ✅ Added `handle_positions()` method returning all handle screen coordinates for rendering
- ✅ Defined `RESIZE_HANDLE_SIZE` constant (8.0 pixels)
- ✅ Code compiles for both host and WebAssembly targets
- ✅ All existing tests pass

#### ✅ 1.3 Add Resizing State to AppState
- Extend `AppState` with `ResizingState` for tracking resize operations
- Add fields for active resize handle, original dimensions, drag start position
- Integrate with existing drag state management

**Implementation Details:**
- Create `ResizingState` struct with: `is_resizing: bool`, `note_id: Option<u32>`, `handle: Option<ResizeHandle>`, `start_mouse_x/y`, `original_width/height`
- Add to `AppState` struct
- Update `AppState::default()` to include default resizing state

**Implementation Results:**
- ✅ Created `ResizingState` struct in `src/sticky_notes.rs` with all required fields
- ✅ Added `ResizingState` to `AppState` struct in `src/lib.rs`
- ✅ Updated `AppState::default()` to include default resizing state
- ✅ Code compiles for both host and WebAssembly targets
- ✅ All existing tests pass
- ✅ Full WASM build succeeds

#### ✅ 1.4 Implement Handle Hit Detection
- Add method to detect which resize handle (if any) is under the mouse cursor
- Calculate handle bounds in screen coordinates
- Prioritize handle detection over note content area

**Implementation Details:**
- Add `find_resize_handle_at()` method to `StickyNotesState`
- Convert world coordinates to screen coordinates for handle bounds
- Return `Option<(u32, ResizeHandle)>` for note ID and handle type

**Implementation Results:**
- ✅ Added `find_resize_handle_at()` method to `StickyNotesState` that checks only selected notes
- ✅ Method uses existing `handle_bounds()` for accurate screen-space hit testing
- ✅ Prioritizes handle detection over note content area by checking handles first
- ✅ Handles coordinate transformation with viewport zoom and pan
- ✅ Added comprehensive unit tests covering all 8 handles, zoom/pan scenarios, and edge cases
- ✅ Code compiles for both host and WebAssembly targets
- ✅ All existing tests pass
- ✅ Full WASM build succeeds

#### ✅ 1.5 Add Handle Rendering
- Update rendering pipeline to draw resize handles on selected notes
- Implement visual states (normal, hover, active) for handles
- Ensure handles scale properly with viewport zoom

**Implementation Details:**
- Modify `render_sticky_notes()` in `canvas.rs` to draw handles
- Draw small squares/circles at handle positions
- Use different colors/styles for different states
- Handle zoom scaling for consistent handle sizes

**Implementation Results:**
- ✅ Added `hovered_resize_handle` field to `AppState` to track currently hovered handle
- ✅ Updated `handle_mouse_move()` to detect resize handle hovering using `find_resize_handle_at()`
- ✅ Modified `render_sticky_notes()` to draw 8 resize handles on selected notes
- ✅ Implemented visual states: normal (gray), hover (darker gray), active (blue)
- ✅ Handles are drawn as 8x8 pixel squares with white borders
- ✅ Handle positions automatically scale with viewport zoom and pan
- ✅ Made `RESIZE_HANDLE_SIZE` constant public for use in canvas rendering
- ✅ Code compiles for both host and WebAssembly targets
- ✅ All existing tests pass
- ✅ Full WASM build succeeds
- ✅ Added E2E test `resize_handle_click_and_drag_does_not_panic` to verify clicking and dragging handles doesn't cause panics

#### ✅ 1.6 Implement Basic Resize Logic
- Add mouse event handling for resize operations
- Calculate new dimensions based on mouse movement and handle type
- Update note dimensions during drag operations

**Implementation Details:**
- Modify `handle_mouse_down()` to detect resize handle clicks
- Add `start_resize()` and `resize_to()` methods to `StickyNotesState`
- Implement dimension calculation based on handle type and mouse delta
- Update mouse move handler to call resize logic

**Implementation Results:**
- ✅ Added `start_resize()`, `resize_to()`, and `end_resize()` methods to `StickyNotesState`
- ✅ Implemented handle-specific dimension calculations (corners change both dimensions, edges change one dimension)
- ✅ Added minimum size constraints (50px width × 40px height) to prevent unusable notes
- ✅ Modified `handle_mouse_down()` to prioritize resize handle detection over note selection
- ✅ Updated `handle_mouse_move()` to call resize logic during active resize operations
- ✅ Modified `handle_mouse_up()` and `end_drag_if_needed()` to properly end resize operations
- ✅ Added proper coordinate transformation from screen to world space for accurate resizing
- ✅ Added `get_note()` method to `StickyNotesState` for immutable note access
- ✅ Code compiles for both host and WebAssembly targets
- ✅ All existing tests pass
- ✅ Full WASM build succeeds
- ✅ All E2E tests pass, including resize handle functionality verification

#### 1.7 Add Cursor Changes for Handles
- Implement dynamic cursor changes when hovering over resize handles
- Use appropriate cursors (nw-resize, n-resize, etc.) for each handle type
- Update cursor in `update_canvas_attributes()`

**Implementation Details:**
- Add cursor detection logic in mouse move handler
- Map `ResizeHandle` variants to CSS cursor values
- Update canvas style cursor property dynamically

#### 1.8 Support Proportional Resizing (Shift Modifier)
- Detect Shift key during resize operations
- Maintain aspect ratio when Shift is held
- Provide visual feedback for proportional mode

**Implementation Details:**
- Track Shift key state in resize operations
- Calculate proportional dimensions using original aspect ratio
- Add visual indicator (different cursor or handle styling) for proportional mode

#### 1.9 Add Minimum Size Constraints
- Implement minimum width/height limits for sticky notes
- Prevent notes from becoming too small during resize
- Provide smooth constraint enforcement

**Implementation Details:**
- Define `MIN_NOTE_WIDTH` and `MIN_NOTE_HEIGHT` constants (e.g., 50.0, 40.0)
- Clamp dimensions in resize logic to minimum values
- Ensure constraints work with proportional resizing

#### 1.10 Test Resize with Viewport Zoom and Pan
- Verify resize handles work correctly at different zoom levels
- Ensure handle positions update properly during pan operations
- Test edge cases with extreme zoom levels

**Implementation Details:**
- Test handle hit detection at various zoom levels
- Verify cursor changes work with viewport transformations
- Ensure minimum size constraints scale appropriately with zoom

### 2. Add Persistence
- Implement save/load functionality for sticky notes
- Add data serialization and local storage

**Implementation Details:**
- Add serialization support for `AppState` using serde
- Implement local storage API for browser persistence
- Add save/load buttons to toolbar
- Support export/import of notes as JSON
- Add autosave functionality
- Handle data migration for future versions

### 3. Implement Undo/Redo System
- Add command pattern for reversible actions
- Implement undo/redo functionality for all user actions

**Implementation Details:**
- Design command pattern for actions (create, edit, move, delete notes)
- Implement undo/redo stack with history management
- Add keyboard shortcuts (Ctrl+Z, Ctrl+Y)
- Add undo/redo buttons to toolbar
- Handle complex operations (bulk actions, etc.)
- Add visual feedback for undo/redo state

### 4. Performance Optimizations

#### 4.1 Spatial Partitioning for Note Hit Testing

##### 4.1.1 Performance Analysis
- Profile current O(n) hit testing with benchmark tests
- Identify performance bottlenecks with 50+ notes
- Establish baseline metrics for improvement measurement

**Implementation Details:**
- Create benchmark test with 100 notes for hit testing
- Measure average hit test time per note
- Document current performance characteristics

##### 4.1.2 Design Spatial Data Structure
- Research quadtree vs R-tree vs simple grid partitioning
- Design API for spatial queries (point-in-rect, rect intersection)
- Define bounds calculation for sticky notes

**Implementation Details:**
- Evaluate data structures for 2D spatial indexing
- Consider note size variations and movement patterns
- Design for dynamic updates (add/remove/move notes)

##### 4.1.3 Implement Spatial Index Core
- Implement chosen spatial data structure (likely quadtree)
- Add insert/remove/update operations for notes
- Implement point-in-bounds queries for hit testing

**Implementation Details:**
- Create `SpatialIndex` trait and implementation
- Handle note bounding box calculations
- Support efficient bulk operations

##### 4.1.4 Integrate with Note Management
- Update `StickyNotes` struct to maintain spatial index
- Modify `add_note`, `remove_note`, `move_note` to update index
- Replace linear search in `find_note_at_point` with spatial query

**Implementation Details:**
- Add spatial index field to `StickyNotes`
- Update all note mutation methods
- Ensure index consistency during operations

##### 4.1.5 Performance Validation
- Benchmark hit testing performance improvement
- Test correctness with existing test suite
- Measure memory overhead of spatial index

**Implementation Details:**
- Compare before/after performance metrics
- Ensure no regressions in functionality
- Document performance gains

#### 4.2 Viewport Culling for Large Note Counts

##### 4.2.1 Viewport Bounds Calculation
- Implement viewport-to-world bounds conversion
- Add viewport change detection for culling updates
- Calculate expanded bounds for smooth panning

**Implementation Details:**
- Add `viewport_bounds()` method to `ViewportState`
- Handle zoom-dependent culling margins
- Support different culling strategies (conservative vs tight)

##### 4.2.2 Culling Logic Implementation
- Implement note filtering based on viewport bounds
- Add culling state management (culled vs visible notes)
- Update culling on viewport changes (pan/zoom)

**Implementation Details:**
- Create `is_note_visible(viewport_bounds, note_bounds)` function
- Add culling cache to avoid redundant calculations
- Handle note size in visibility calculations

##### 4.2.3 Rendering Pipeline Integration
- Modify canvas rendering to use culled note list
- Update rendering loop to filter notes before drawing
- Optimize render order for better performance

**Implementation Details:**
- Update `render` function to accept filtered note list
- Maintain render order for proper layering
- Add debug visualization for culling bounds

##### 4.2.4 Culling Performance Testing
- Benchmark rendering performance with 200+ notes
- Test culling accuracy and smoothness
- Measure frame rate improvements

**Implementation Details:**
- Create performance test with high note counts
- Verify no visual artifacts from culling
- Document rendering performance gains

#### 4.3 Combined Spatial + Culling Optimization

##### 4.3.1 Integration Testing
- Test spatial index + culling working together
- Verify hit testing works on culled notes
- Performance benchmark of combined optimizations

**Implementation Details:**
- End-to-end testing with large note sets
- Ensure spatial queries work with viewport bounds
- Measure overall performance improvement

##### 4.3.2 Memory and Maintenance Optimization
- Optimize spatial index memory usage
- Add index rebuilding for extreme cases
- Implement lazy culling updates

**Implementation Details:**
- Profile memory usage of spatial structures
- Add maintenance operations for index health
- Balance performance vs memory trade-offs

#### 4.3 Grid Rendering Optimization
- Optimize background grid rendering for large zoom levels
- Implement adaptive grid density

**Implementation Details:**
- Analyze grid rendering performance at different zoom levels
- Implement level-of-detail (LOD) for grid lines
- Reduce grid density at high zoom levels
- Optimize grid line calculation and drawing

#### 4.4 WebGL Acceleration (Future)
- Consider WebGL acceleration for complex rendering
- Evaluate WebGL vs Canvas 2D performance trade-offs

**Implementation Details:**
- Research WebGL rendering for 2D graphics
- Prototype WebGL-based grid and note rendering
- Compare performance with Canvas 2D
- Consider implementation if significant benefits found

## Active Medium Priority Tasks

### 5. Enhanced Accessibility
- Complete WCAG compliance and screen reader support
- Improve keyboard navigation

**Implementation Details:**
- Conduct full accessibility audit
- Add screen reader announcements for actions
- Improve keyboard navigation (tab order, focus management)
- Add high contrast mode support
- Test with screen readers

### 6. Advanced Visual Polish
- Add animations and visual effects
- Improve overall UI/UX design

**Implementation Details:**
- Add smooth animations for note creation/deletion
- Implement visual feedback for interactions
- Add gradients, shadows, and modern styling
- Improve color scheme and typography
- Add loading states and transitions

## Active Low Priority Tasks

### 7. Add Mobile Support
- Implement touch event handling for mobile devices
- Add gesture recognition for pinch-to-zoom and multi-touch interactions

**Implementation Details:**
- Add touch event listeners in `events.rs` for `touchstart`, `touchmove`, `touchend`
- Implement pinch gesture detection for zoom
- Add single-touch drag support for canvas and notes
- Test on mobile browsers and ensure responsive design
- Handle touch vs mouse event conflicts

## Recently Completed Tasks

### ✅ 1.3 Add Resizing State to AppState
- Extended `AppState` with `ResizingState` struct for tracking resize operations
- Added fields for active resize handle, original dimensions, drag start position
- Integrated with existing drag state management

**Implementation Details:**
- Created `ResizingState` struct with: `is_resizing: bool`, `note_id: Option<u32>`, `handle: Option<ResizeHandle>`, `start_mouse_x/y`, `original_width/height`
- Added `ResizingState` to `AppState` struct in `src/lib.rs`
- Updated `AppState::default()` to include default resizing state
- Code compiles for both host and WebAssembly targets
- All existing tests pass
- Full WASM build succeeds

### ✅ 1.4 Implement Handle Hit Detection
- Added method to detect which resize handle is under the mouse cursor
- Implemented screen coordinate hit testing with viewport transformations
- Prioritized handle detection over note content area

**Implementation Details:**
- Added `find_resize_handle_at()` method to `StickyNotesState` with screen coordinate parameters
- Used existing `handle_bounds()` method for accurate bounding box calculations
- Implemented proper coordinate transformation accounting for zoom and pan
- Added comprehensive unit tests covering all handle types, viewport transformations, and edge cases
- Code compiles for both host and WebAssembly targets
- All existing tests pass
- Full WASM build succeeds

### ✅ Externalize CSS Styles
- Moved all inline CSS from `index.html` to external `styles.css` file
- Replaced programmatic style setting in Rust code with CSS classes for static styles
- Defined CSS custom properties for colors and reusable values
- Maintained all existing visual styling and behavior
- Created CSS classes for dynamic elements (.text-input-toolbar, .contenteditable-overlay, .formatting-button variants)
- Kept only truly dynamic properties (positions, dimensions, background-color) in Rust code
- Ensured trunk build includes the external CSS file
- All tests pass and WASM build succeeds

### ✅ Create Visual Regression Test Suite
- Set up automated screenshot comparison for key UI components
- Capture baseline images of toolbar, text input overlay, canvas states
- Implement pixel-perfect comparison or perceptual diff testing
- Add to CI/CD pipeline for future changes

**Implementation Details:**
- Use headless_chrome for screenshot capture in browser automation tests
- Store baseline images in `tests/baselines/` directory (committed to version control)
- Implement pixel-by-pixel image comparison with configurable threshold (1% difference allowed)
- Created `tests/visual_regression.rs` with automated test suite
- Tests for: toolbar initial state, canvas initial state, text input overlay
- Utility test to update baselines when UI changes are intentional
- Failing screenshots saved with `_fail.png` suffix for debugging (not committed)
- All tests pass and WASM build succeeds

### ✅ Split render_canvas Function
- Broke down the ~290-line `render_canvas` function in `canvas.rs` into smaller, focused functions
- Extracted `render_grid_background()`, `render_sticky_notes()`, `update_canvas_attributes()`, and `update_status_display()`
- Maintained the same external API and behavior with proper error handling
- All tests pass and WASM build succeeds

### ✅ Extract Error Types Module
- Created new `src/error.rs` module with comprehensive error type documentation
- Moved `AppError` enum and `Display`/`Error` trait implementations from `lib.rs`
- Moved `AppResult<T>` type alias for WebAssembly operations
- Updated all imports across dependent modules (app.rs, canvas.rs, event_setup.rs, logging.rs, mouse_events.rs, keyboard_events.rs)
- Preserved WebAssembly conditional compilation for all error conversions
- Verified compilation for both host and WASM targets
- All tests pass and full WASM build succeeds

### ✅ Extract Logging Module
- Created new `src/logging.rs` module with proper documentation
- Moved all logging functions (`log_info`, `log_warn`, `log_app_error`, `log_js_error`, `log_jsvalue_error`) from `lib.rs`
- Updated imports across all modules that use logging (mouse_events.rs, text_input.rs, event_setup.rs, keyboard_events.rs, lib.rs)
- Preserved WebAssembly conditional compilation for all functions
- Verified compilation for both host and WASM targets
- All tests pass and full WASM build succeeds

### ✅ Move Integration Tests
- Create new `tests/integration_tests.rs` file
- Move all integration tests from `lib.rs` to the new file
- Update test module structure and imports
- Ensure all tests still pass

**Implementation Details:**
- Create `tests/` directory if it doesn't exist
- Move the 9 integration test functions from `lib.rs`
- Update `#[cfg(test)]` module declarations
- Made AppState, ViewportState, and related methods available for host compilation to support integration tests
- Run full test suite to verify no regressions

### ✅ Break Down start_impl Function
- Split the ~150-line `start_impl` function in `lib.rs` into smaller initialization phases
- Extracted `setup_canvas_and_context()`, `create_render_and_position_functions()`, `setup_event_system()`, and `setup_window_resize_handler()` helper functions
- Improved readability and maintainability while preserving error handling and resource management
- All tests pass and WASM build succeeds

### ✅ Text Editing Features (All Completed)
- Double-click to edit sticky notes
- ContentEditable text input with toolbar
- HTML formatting during editing (bold, italic, underline)
- Paste sanitization (strips all HTML to plain text)
- Multi-line text support
- Text selection and cursor positioning
- Rich text storage and rendering
- Toolbar button formatting
- HTML-to-text conversion for display

### ✅ Basic Accessibility Features
- ARIA labels for toolbar buttons
- Canvas accessibility attributes
- Basic keyboard navigation support

### ✅ Performance Monitoring
- FPS counter and performance metrics
- Frame rate tracking
- Rendering performance monitoring

### ✅ Basic Visual Polish
- Box shadows for text input overlay
- Consistent styling for toolbar and inputs

## Implementation Notes
- All changes must maintain compatibility with both host and WebAssembly targets
- Run full test suite after each change (12 unit tests + 8 E2E tests currently passing)
- Update documentation as features are implemented
- Consider backward compatibility for any breaking changes
- HTML sanitization strips all formatting on paste for simplicity