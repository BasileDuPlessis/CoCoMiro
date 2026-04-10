# Tasks Backlog

## Overview
This backlog contains tasks to improve the CoCoMiro infinite canvas application based on code review findings.

## High Priority Tasks

### 1. Enhance Text Editing
- Implement proper text editing for sticky notes
- Add text input, selection, and editing capabilities
- Improve text rendering and layout

**Implementation Details:**
- Add text input mode for sticky notes (double-click to edit)
- Implement text selection and cursor positioning
- Add keyboard shortcuts for text editing (Ctrl+A, Ctrl+C, Ctrl+V, etc.)
- Support multi-line text with proper line breaks
- Add text formatting options (bold, italic, etc.)
- Improve text rendering quality and font handling

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

#### 4.1 Spatial Indexing for Hit Testing
- Implement efficient data structure for note hit testing
- Replace linear search with spatial indexing (quadtree or similar)

**Implementation Details:**
- Analyze current O(n) hit testing performance
- Implement quadtree or R-tree data structure
- Update note addition/removal to maintain spatial index
- Benchmark performance improvement with many notes

#### 4.2 View Culling for Rendering
- Only render sticky notes visible in the current viewport
- Implement frustum culling for canvas elements

**Implementation Details:**
- Calculate viewport bounds in world coordinates
- Filter notes before rendering based on visibility
- Update culling when viewport changes (pan/zoom)
- Measure rendering performance with 100+ notes

#### 4.3 Grid Rendering Optimization
- Optimize background grid rendering for large zoom levels
- Implement adaptive grid density

**Implementation Details:**
- Analyze grid rendering performance at different zoom levels
- Implement level-of-detail (LOD) for grid lines
- Reduce grid density at high zoom levels
- Optimize grid line calculation and drawing

#### 4.4 Performance Monitoring
- Add frame rate monitoring and performance profiling
- Implement performance metrics collection

**Implementation Details:**
- Add FPS counter to status display
- Track rendering time per frame
- Monitor memory usage and note count
- Add performance logging for debugging

#### 4.5 WebGL Acceleration (Future)
- Consider WebGL acceleration for complex rendering
- Evaluate WebGL vs Canvas 2D performance trade-offs

**Implementation Details:**
- Research WebGL rendering for 2D graphics
- Prototype WebGL-based grid and note rendering
- Compare performance with Canvas 2D
- Consider implementation if significant benefits found

## Medium Priority Tasks

### 5. Visual Polish
- Add visual enhancements like shadows, rounded corners, and animations
- Improve overall UI aesthetics

**Implementation Details:**
- Add CSS-like styling to canvas rendering (shadows, gradients)
- Implement smooth animations for toolbar and note interactions
- Add visual feedback for all interactions
- Improve color scheme and visual hierarchy
- Add loading states and transitions

### 6. Accessibility Improvements
- Enhance ARIA labels and keyboard navigation
- Ensure WCAG compliance

**Implementation Details:**
- Add comprehensive ARIA labels to all interactive elements
- Implement full keyboard navigation (tab order, focus management)
- Add screen reader support for canvas content
- Ensure sufficient color contrast
- Add keyboard shortcuts documentation

## Low Priority Tasks

### 7. Add Mobile Support
- Implement touch event handling for mobile devices
- Add gesture recognition for pinch-to-zoom and multi-touch interactions

**Implementation Details:**
- Add touch event listeners in `events.rs` for `touchstart`, `touchmove`, `touchend`
- Implement pinch gesture detection for zoom
- Add single-touch drag support for canvas and notes
- Test on mobile browsers and ensure responsive design
- Handle touch vs mouse event conflicts

## Implementation Notes
- All changes must maintain compatibility with both host and WebAssembly targets
- Run full test suite after each change
- Update documentation as features are implemented
- Consider backward compatibility for any breaking changes