# Tasks Backlog

## Overview
This backlog contains tasks to improve the CoCoMiro infinite canvas application based on code review findings.

## High Priority Tasks

### 1. Enhance Text Editing

#### 1.1 Basic Text Input Mode
- Implement double-click to edit sticky notes
- Add simple text input functionality

##### 1.1.1 Double-Click Detection ✅ COMPLETED
- Add double-click event handler on canvas
- Detect which sticky note was double-clicked
- Prevent default double-click behavior

**Implementation Details:**
- Attach `dblclick` event listener to canvas element
- Calculate world coordinates from mouse position
- Find which sticky note contains the click point
- Log the double-click event for debugging

##### 1.1.2 Text Input Overlay Creation ✅ COMPLETED
- Create HTML input element overlay
- Position input element over the selected sticky note
- Style input to match note appearance
- Focus and select text in input

**Implementation Details:**
- Create `HtmlInputElement` programmatically with proper styling
- Calculate screen position from world coordinates and zoom level
- Apply note-matching styles (background color, border, font, padding)
- Position overlay with absolute positioning and high z-index
- Focus input and select all text for immediate editing

##### 1.1.3 Basic Text Editing ✅ COMPLETED
- Handle text input and basic editing operations
- Support typing, backspace, and basic navigation
- Update input value as user types

**Implementation Details:**
- Attach `input` event listener to handle text changes
- Allow standard text editing operations
- Prevent event propagation to avoid canvas interactions
- Maintain input focus during editing

##### 1.1.4 Input Confirmation and Cleanup ✅ COMPLETED
- Handle Enter key to confirm changes
- Handle Escape key to cancel editing
- Update note content and re-render canvas
- Remove input overlay and restore normal interaction

**Implementation Details:**
- Attach `keydown` event listener for Enter/Escape handling
- Attach `blur` event listener for clicking outside
- Update `StickyNote.content` with new text
- Remove input element from DOM
- Trigger canvas re-rendering

#### 1.2 Advanced Text Editing ✅ COMPLETED
- Implement text selection and cursor positioning
- Add keyboard shortcuts for text editing

**Implementation Details:**
- Add cursor positioning and text selection
- Implement keyboard shortcuts (Ctrl+A, Ctrl+C, Ctrl+V, etc.)
- Handle arrow keys for cursor movement
- Support text deletion and insertion at cursor

#### 1.3 Multi-line Text Support ✅ COMPLETED
- Support multi-line text with proper line breaks
- Implement text wrapping for long lines

**Implementation Details:**
- Handle Enter key for line breaks
- Implement text wrapping algorithm
- Update text rendering for multi-line display
- Adjust note height based on content

#### 1.4 Text Formatting Options
- Add text formatting capabilities (bold, italic, etc.)
- Implement rich text editing

##### 1.4.1 Add Formatting Toolbar ✅ COMPLETED
- Create a formatting toolbar with bold, italic, underline buttons
- Position toolbar near text input overlay
- Style toolbar to match application design

**Implementation Details:**
- Add HTML toolbar element with formatting buttons
- Position toolbar above or below text input area
- Include icons or text labels for each formatting option
- Handle button clicks to apply formatting

##### 1.4.2 Implement Rich Text Storage ✅ COMPLETED
- Extend note data structure to store HTML formatting
- Support HTML content in note text field
- Maintain backward compatibility with plain text

**Implementation Details:**
- Store HTML content directly in `StickyNote.content` field
- Plain text notes remain as plain text, rich text notes contain HTML
- HTML content gets converted to markdown for canvas rendering
- Update module documentation to describe HTML-based rich text architecture

##### 1.4.3 Implement Rich Text Rendering ✅ COMPLETED
- Render formatted text on canvas with appropriate styles
- Support bold, italic, underline rendering
- Handle overlapping formatting ranges

**Implementation Details:**
- Extended canvas text rendering for rich text with HTML parsing
- Added `TextSegment` struct to represent formatted text segments
- Implemented `parse_formatted_text()` to parse HTML tags (<b>, <i>, <u>)
- Added `format_font()` to create CSS font strings with appropriate styles
- Modified rendering to handle text wrapping while preserving formatting
- Underline implemented with canvas stroke operations
- Maintains backward compatibility with plain text

##### 1.4.4 Replace Textarea with ContentEditable Div ✅ COMPLETED
- Replace textarea overlay with contenteditable div for seamless editing
- Style contenteditable div to match sticky note appearance exactly
- Ensure no visible overlay distinction between edit and view modes

**Implementation Details:**
- Created contenteditable div instead of textarea element with `contenteditable="true"`
- Applied identical CSS styling (background, border, font, padding) to match note exactly
- Positioned contenteditable div precisely over the sticky note using absolute positioning
- Updated event handlers for contenteditable behavior (input, blur, keydown)
- Maintained existing toolbar integration and formatting functionality
- Added proper HTML content handling (innerHTML with <br> tags for line breaks)
- Implemented toolbar click prevention to avoid premature overlay removal
- Removed border and border-radius for perfect visual blending with rendered note
- Simplified formatting buttons to maintain focus (full formatting implementation deferred)

##### 1.4.5 Update ContentEditable Event Handling ✅ COMPLETED
- Adapt event handlers for contenteditable div behavior
- Handle input, blur, and keyboard events appropriately
- Prevent canvas interactions during editing

**Implementation Details:**
- Replace textarea-specific event listeners with contenteditable equivalents
- Handle contenteditable input events for real-time updates
- Implement blur handling to confirm/cancel edits
- Maintain keyboard shortcuts and navigation
- Prevent event propagation to canvas during editing

##### 1.4.6 Implement Toolbar Button Formatting
- Make toolbar buttons apply real HTML formatting using document.execCommand()
- Replace logging-only handlers with actual formatting functionality

**Implementation Details:**
- Use document.execCommand('bold'), document.execCommand('italic'), document.execCommand('underline')
- Handle button clicks to apply formatting to selected text or at cursor position
- Maintain cursor position and text selection during formatting operations
- Test formatting works correctly with contenteditable div

##### 1.4.7 Store HTML Content in Notes
- Modify note storage to handle HTML content instead of plain text
- Update text input handling to store HTML from contenteditable

**Implementation Details:**
- Store HTML content directly in StickyNote.content field
- Update input event handler to preserve HTML formatting
- Handle conversion between HTML and plain text for backward compatibility
- Ensure existing plain text notes continue to work

##### 1.4.8 Implement HTML-to-Markdown Conversion
- Create HTML parsing function to convert HTML to markdown for canvas rendering
- Update parse_formatted_text() to handle HTML tags instead of markdown

**Implementation Details:**
- Parse <b>, <i>, <u> tags and convert to markdown equivalents
- Handle nested formatting and overlapping tags
- Update canvas rendering to use HTML-based parsing
- Maintain backward compatibility with existing markdown parsing

##### 1.4.9 Handle Rich Text Paste Operations
- Implement paste event handling to preserve HTML formatting
- Clean and sanitize pasted content

**Implementation Details:**
- Add paste event listener to contenteditable div
- Preserve HTML formatting from clipboard
- Strip unwanted elements (scripts, styles) for security
- Handle plain text paste operations

##### 1.4.10 Add ContentEditable Undo/Redo Support
- Implement undo/redo functionality within contenteditable context
- Handle browser's native undo/redo behavior

**Implementation Details:**
- Support Ctrl+Z/Ctrl+Y keyboard shortcuts
- Handle undo/redo state management
- Preserve formatting during undo/redo operations
- Integrate with browser's native contenteditable undo/redo

##### 1.4.11 Add ContentEditable Accessibility
- Ensure contenteditable div is accessible to screen readers
- Add proper ARIA labels and keyboard navigation
- Handle focus management between canvas and editing mode

**Implementation Details:**
- Add ARIA attributes for screen reader support
- Implement proper focus indicators and keyboard navigation
- Handle tab order and focus trapping during editing
- Add visual focus indicators that match application design
- Test with screen readers and accessibility tools

##### 1.4.12 Add Keyboard Shortcuts for Formatting
- Implement keyboard shortcuts for text formatting
- Support common shortcuts (Ctrl+B, Ctrl+I, Ctrl+U)

**Implementation Details:**
- Add keyboard event handling for formatting shortcuts
- Integrate with toolbar button functionality
- Handle shortcuts within contenteditable context
- Integrate with contenteditable div
- Provide visual feedback for active formatting
- Document available shortcuts

#### 1.5 Text Rendering Improvements
- Improve text rendering quality and font handling
- Optimize text display performance

**Implementation Details:**
- Upgrade font rendering quality
- Add better font choices and sizing
- Optimize text rendering performance
- Handle different text sizes and styles

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