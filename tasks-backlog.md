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

### 1. Add Persistence
- Implement save/load functionality for sticky notes
- Add data serialization and local storage

**Implementation Details:**
- Add serialization support for `AppState` using serde
- Implement local storage API for browser persistence
- Add save/load buttons to toolbar
- Support export/import of notes as JSON
- Add autosave functionality
- Handle data migration for future versions

### 2. Implement Undo/Redo System
- Add command pattern for reversible actions
- Implement undo/redo functionality for all user actions

**Implementation Details:**
- Design command pattern for actions (create, edit, move, delete notes)
- Implement undo/redo stack with history management
- Add keyboard shortcuts (Ctrl+Z, Ctrl+Y)
- Add undo/redo buttons to toolbar
- Handle complex operations (bulk actions, etc.)
- Add visual feedback for undo/redo state

### 3. Performance Optimizations

#### 3.1 Spatial Partitioning for Note Hit Testing

##### 3.1.1 Performance Analysis
- Profile current O(n) hit testing with benchmark tests
- Identify performance bottlenecks with 50+ notes
- Establish baseline metrics for improvement measurement

**Implementation Details:**
- Create benchmark test with 100 notes for hit testing
- Measure average hit test time per note
- Document current performance characteristics

##### 3.1.2 Design Spatial Data Structure
- Research quadtree vs R-tree vs simple grid partitioning
- Design API for spatial queries (point-in-rect, rect intersection)
- Define bounds calculation for sticky notes

**Implementation Details:**
- Evaluate data structures for 2D spatial indexing
- Consider note size variations and movement patterns
- Design for dynamic updates (add/remove/move notes)

##### 3.1.3 Implement Spatial Index Core
- Implement chosen spatial data structure (likely quadtree)
- Add insert/remove/update operations for notes
- Implement point-in-bounds queries for hit testing

**Implementation Details:**
- Create `SpatialIndex` trait and implementation
- Handle note bounding box calculations
- Support efficient bulk operations

##### 3.1.4 Integrate with Note Management
- Update `StickyNotes` struct to maintain spatial index
- Modify `add_note`, `remove_note`, `move_note` to update index
- Replace linear search in `find_note_at_point` with spatial query

**Implementation Details:**
- Add spatial index field to `StickyNotes`
- Update all note mutation methods
- Ensure index consistency during operations

##### 3.1.5 Performance Validation
- Benchmark hit testing performance improvement
- Test correctness with existing test suite
- Measure memory overhead of spatial index

**Implementation Details:**
- Compare before/after performance metrics
- Ensure no regressions in functionality
- Document performance gains

#### 3.2 Viewport Culling for Large Note Counts

##### 3.2.1 Viewport Bounds Calculation
- Implement viewport-to-world bounds conversion
- Add viewport change detection for culling updates
- Calculate expanded bounds for smooth panning

**Implementation Details:**
- Add `viewport_bounds()` method to `ViewportState`
- Handle zoom-dependent culling margins
- Support different culling strategies (conservative vs tight)

##### 3.2.2 Culling Logic Implementation
- Implement note filtering based on viewport bounds
- Add culling state management (culled vs visible notes)
- Update culling on viewport changes (pan/zoom)

**Implementation Details:**
- Create `is_note_visible(viewport_bounds, note_bounds)` function
- Add culling cache to avoid redundant calculations
- Handle note size in visibility calculations

##### 3.2.3 Rendering Pipeline Integration
- Modify canvas rendering to use culled note list
- Update rendering loop to filter notes before drawing
- Optimize render order for better performance

**Implementation Details:**
- Update `render` function to accept filtered note list
- Maintain render order for proper layering
- Add debug visualization for culling bounds

##### 3.2.4 Culling Performance Testing
- Benchmark rendering performance with 200+ notes
- Test culling accuracy and smoothness
- Measure frame rate improvements

**Implementation Details:**
- Create performance test with high note counts
- Verify no visual artifacts from culling
- Document rendering performance gains

#### 3.3 Combined Spatial + Culling Optimization

##### 3.3.1 Integration Testing
- Test spatial index + culling working together
- Verify hit testing works on culled notes
- Performance benchmark of combined optimizations

**Implementation Details:**
- End-to-end testing with large note sets
- Ensure spatial queries work with viewport bounds
- Measure overall performance improvement

##### 3.3.2 Memory and Maintenance Optimization
- Optimize spatial index memory usage
- Add index rebuilding for extreme cases
- Implement lazy culling updates

**Implementation Details:**
- Profile memory usage of spatial structures
- Add maintenance operations for index health
- Balance performance vs memory trade-offs

#### 3.3 Grid Rendering Optimization
- Optimize background grid rendering for large zoom levels
- Implement adaptive grid density

**Implementation Details:**
- Analyze grid rendering performance at different zoom levels
- Implement level-of-detail (LOD) for grid lines
- Reduce grid density at high zoom levels
- Optimize grid line calculation and drawing

#### 3.4 WebGL Acceleration (Future)
- Consider WebGL acceleration for complex rendering
- Evaluate WebGL vs Canvas 2D performance trade-offs

**Implementation Details:**
- Research WebGL rendering for 2D graphics
- Prototype WebGL-based grid and note rendering
- Compare performance with Canvas 2D
- Consider implementation if significant benefits found

## Active Medium Priority Tasks

### 4. Enhanced Accessibility
- Complete WCAG compliance and screen reader support
- Improve keyboard navigation

**Implementation Details:**
- Conduct full accessibility audit
- Add screen reader announcements for actions
- Improve keyboard navigation (tab order, focus management)
- Add high contrast mode support
- Test with screen readers

### 5. Advanced Visual Polish
- Add animations and visual effects
- Improve overall UI/UX design

**Implementation Details:**
- Add smooth animations for note creation/deletion
- Implement visual feedback for interactions
- Add gradients, shadows, and modern styling
- Improve color scheme and typography
- Add loading states and transitions

## Active Low Priority Tasks

### 6. Add Mobile Support
- Implement touch event handling for mobile devices
- Add gesture recognition for pinch-to-zoom and multi-touch interactions

**Implementation Details:**
- Add touch event listeners in `events.rs` for `touchstart`, `touchmove`, `touchend`
- Implement pinch gesture detection for zoom
- Add single-touch drag support for canvas and notes
- Test on mobile browsers and ensure responsive design
- Handle touch vs mouse event conflicts

## Recently Completed Tasks

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