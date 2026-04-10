# Tasks Backlog

## Overview
This backlog contains tasks to improve the CoCoMiro infinite canvas application based on code review findings.

## High Priority Tasks

### 1. Enhance Text Rendering
- Implement text wrapping for sticky notes
- Improve typography and text layout
- Add text editing capabilities

**Implementation Details:**
- Modify `canvas.rs` render function to handle multi-line text
- Implement word wrapping algorithm for note content
- Add text metrics calculation for proper line height
- Consider adding text selection and editing UI
- Support different fonts and text styling

### 2. Add Mobile Support
- Implement touch event handling for mobile devices
- Add gesture recognition for pinch-to-zoom and multi-touch interactions

**Implementation Details:**
- Add touch event listeners in `events.rs` for `touchstart`, `touchmove`, `touchend`
- Implement pinch gesture detection for zoom
- Add single-touch drag support for canvas and notes
- Test on mobile browsers and ensure responsive design
- Handle touch vs mouse event conflicts

### 3. Performance Optimizations

#### 3.1 Spatial Indexing for Hit Testing
- Implement efficient data structure for note hit testing
- Replace linear search with spatial indexing (quadtree or similar)

**Implementation Details:**
- Analyze current O(n) hit testing performance
- Implement quadtree or R-tree data structure
- Update note addition/removal to maintain spatial index
- Benchmark performance improvement with many notes

#### 3.2 View Culling for Rendering
- Only render sticky notes visible in the current viewport
- Implement frustum culling for canvas elements

**Implementation Details:**
- Calculate viewport bounds in world coordinates
- Filter notes before rendering based on visibility
- Update culling when viewport changes (pan/zoom)
- Measure rendering performance with 100+ notes

#### 3.3 Grid Rendering Optimization
- Optimize background grid rendering for large zoom levels
- Implement adaptive grid density

**Implementation Details:**
- Analyze grid rendering performance at different zoom levels
- Implement level-of-detail (LOD) for grid lines
- Reduce grid density at high zoom levels
- Optimize grid line calculation and drawing

#### 3.4 Performance Monitoring
- Add frame rate monitoring and performance profiling
- Implement performance metrics collection

**Implementation Details:**
- Add FPS counter to status display
- Track rendering time per frame
- Monitor memory usage and note count
- Add performance logging for debugging

#### 3.5 WebGL Acceleration (Future)
- Consider WebGL acceleration for complex rendering
- Evaluate WebGL vs Canvas 2D performance trade-offs

**Implementation Details:**
- Research WebGL rendering for 2D graphics
- Prototype WebGL-based grid and note rendering
- Compare performance with Canvas 2D
- Consider implementation if significant benefits found

## Medium Priority Tasks

### 4. Visual Polish
- Add visual enhancements like shadows, rounded corners, and animations
- Improve overall UI aesthetics

**Implementation Details:**
- Add CSS-like styling to canvas rendering (shadows, gradients)
- Implement smooth animations for toolbar and note interactions
- Add visual feedback for all interactions
- Improve color scheme and visual hierarchy
- Add loading states and transitions

### 5. Accessibility Improvements
- Enhance ARIA labels and keyboard navigation
- Ensure WCAG compliance

**Implementation Details:**
- Add comprehensive ARIA labels to all interactive elements
- Implement full keyboard navigation (tab order, focus management)
- Add screen reader support for canvas content
- Ensure sufficient color contrast
- Add keyboard shortcuts documentation

## Low Priority Tasks

### 6. Persistence
- Add save/load functionality for sticky notes
- Implement data serialization

**Implementation Details:**
- Add serialization support for `AppState` (serde)
- Implement local storage or file-based persistence
- Add export/import functionality
- Handle data migration for future versions
- Add autosave functionality

### 7. Undo/Redo System
- Implement command pattern for reversible actions
- Add undo/redo functionality for all user actions

**Implementation Details:**
- Design command pattern for actions (create note, move note, delete note, etc.)
- Implement undo/redo stack with history management
- Add keyboard shortcuts (Ctrl+Z, Ctrl+Y)
- Handle complex operations (bulk actions, etc.)
- Add visual feedback for undo/redo state

## Implementation Notes
- All changes must maintain compatibility with both host and WebAssembly targets
- Run full test suite after each change
- Update documentation as features are implemented
- Consider backward compatibility for any breaking changes