# CoCoMiro Product Specification

## Overview

CoCoMiro is a WebAssembly-based infinite canvas application built with Rust, providing a smooth, interactive drawing and note-taking experience in the browser. The application features an infinite-feeling canvas with pannable and zoomable viewport, draggable sticky notes with rich text editing capabilities, and a floating toolbar for quick actions.

## Core Features

### 1. Infinite Canvas System

#### Viewport Management
- **Pan and Zoom**: Smooth cursor-anchored zooming with mouse wheel
- **Keyboard Navigation**:
  - Arrow keys for panning
  - `+`/`-` keys for zooming
  - `0` key to reset zoom to 1.0 and pan to (0,0)
- **Coordinate System**:
  - World coordinates: Absolute positions in the infinite canvas space
  - Screen coordinates: Relative to the viewport/canvas element
  - Automatic transformation between coordinate systems accounting for pan and zoom
- **HiDPI Support**: Sharp rendering on Retina and high-DPI displays

#### Canvas Rendering
- **Grid Display**: Movable grid background that scales with zoom
- **Real-time Updates**: Canvas re-renders on viewport changes, note movements, and state updates
- **Error Recovery**: Graceful handling of canvas context loss with automatic recovery attempts

### 2. Sticky Notes System

#### Note Creation and Management
- **Add Notes**: Button in floating toolbar creates new notes at viewport center
- **Default Properties**: New notes are 200x150 pixels with default positioning
- **Unique IDs**: Each note has a unique 32-bit identifier
- **World Positioning**: Notes exist in world coordinates, unaffected by viewport changes

#### Note Interaction
- **Dragging**: Click and drag to move notes around the canvas
- **Selection**: Click on a note to select it (visual feedback provided)
- **Multi-note Support**: Multiple notes can exist simultaneously
- **Selection Clearing**: Clicking empty canvas space deselects all notes
- **Delete Functionality**: Press `Delete` key to remove selected notes

#### Text Editing
- **In-place Editing**: Double-click a note to enter text editing mode
- **Rich Text Support**:
  - Bold (`<b>` tags)
  - Italic (`<i>` tags)
  - Underline (`<u>` tags)
  - Nested formatting (e.g., bold + italic)
- **Formatting Toolbar**: Buttons for applying text formatting to selected text
- **HTML Parsing**: Proper rendering of HTML-formatted text content
- **Line Breaks**: Support for multi-line text with `<br>` tags

#### Note Resizing
- **Resize Handles**: Eight handles (corners and midpoints) for resizing
- **Visual Feedback**: Handles appear when note is selected
- **Drag-to-Resize**: Click and drag handles to change note dimensions
- **Maintain Aspect Ratio**: Resizing preserves note proportions appropriately

### 3. Floating Toolbar

#### Toolbar Properties
- **Positioning**: Freely draggable around the canvas
- **Orientation**: Vertical layout (height > width)
- **Persistence**: Position data exposed via `data-x` and `data-y` attributes
- **Handle**: Dedicated drag handle for repositioning

#### Toolbar Actions
- **Add Note Button**: Creates new sticky note at viewport center
- **Formatting Buttons**: Bold, italic, underline for text editing
- **Selection Clearing**: Clicking toolbar background clears note selection

### 4. User Interface

#### Visual Design
- **Clean Interface**: No header copy or marketing content on initial load
- **Grid Background**: Subtle grid pattern that moves with panning
- **Note Styling**: Distinctive appearance for sticky notes
- **Toolbar Styling**: Floating design with clear button states

#### Responsive Behavior
- **Window Resize**: Automatic canvas resizing and toolbar repositioning
- **HiDPI Rendering**: Crisp visuals on high-resolution displays
- **Cross-browser Compatibility**: Works in modern browsers with WebAssembly support

### 5. Data Management

#### State Structure
- **AppState**: Central state container with viewport, notes, mouse position, and auth data
- **ViewportState**: Pan coordinates, zoom level, drag status
- **StickyNotesState**: Collection of notes with selection management
- **AuthManager**: User authentication state (placeholder for future features)

#### Persistence
- **In-memory State**: Current implementation maintains state in memory
- **Future Extensibility**: Architecture supports adding persistence layers

### 6. Security and Safety

#### Input Sanitization
- **HTML Paste Protection**: Strips malicious HTML tags from pasted content
- **Safe Text Extraction**: Converts HTML to plain text with line break preservation
- **XSS Prevention**: Removes script tags and other dangerous elements

#### Error Handling
- **Graceful Degradation**: Fallback rendering when canvas fails
- **Context Recovery**: Automatic recovery from WebGL/canvas context loss
- **Logging**: Comprehensive error logging for debugging

## Technical Architecture

### Technology Stack
- **Language**: Rust with WebAssembly compilation
- **Rendering**: HTML5 Canvas 2D API
- **Build System**: Trunk for WASM bundling
- **Testing**: Headless Chrome for E2E tests, Rust unit tests
- **Styling**: CSS with responsive design

### Module Structure
- **Canvas**: Rendering logic and coordinate transformations
- **Events**: Mouse, keyboard, and touch event handling
- **Viewport**: Pan/zoom state management
- **Sticky Notes**: Note creation, editing, and interaction
- **Toolbar**: Floating UI element management
- **Text Input**: Rich text editing functionality
- **Auth**: User authentication (placeholder)

### Performance Characteristics
- **Smooth Rendering**: 60fps target with frame skipping for slow operations
- **Efficient Updates**: Only re-renders when state changes
- **Memory Management**: Proper cleanup and resource management
- **WebAssembly Optimization**: Minimal bundle size and fast execution

## Testing and Quality Assurance

### Test Coverage
- **Unit Tests**: Individual module functionality
- **Integration Tests**: Component interactions and coordinate systems
- **E2E Tests**: Full browser automation with Headless Chrome
- **Visual Regression**: Screenshot comparison for UI consistency

### Test Scenarios
- **Basic Functionality**: Canvas panning, toolbar dragging, clean initial state
- **Note Lifecycle**: Creation, dragging, selection, deletion
- **Text Editing**: Formatting, paste sanitization, multi-line support
- **Viewport Interaction**: Zoom with notes, coordinate transformations
- **Multi-note Scenarios**: Multiple notes, selection management
- **Error Conditions**: Canvas recovery, fallback rendering

## Browser Compatibility

### Supported Browsers
- Chrome/Chromium (recommended for testing)
- Firefox
- Safari
- Edge

### Requirements
- WebAssembly support
- HTML5 Canvas API
- Modern JavaScript features
- CSS Grid and Flexbox support

## Development and Deployment

### Build Process
- **Dual Compilation**: Host and WebAssembly targets
- **Trunk Integration**: Automatic bundling and serving
- **Asset Processing**: CSS compilation and optimization

### Development Workflow
- **Local Testing**: `trunk serve` for development
- **Cross-target Verification**: Both host and WASM compilation checks
- **E2E Testing**: Automated browser tests with `cargo e2e`

## Future Extensibility

### Planned Features
- **User Authentication**: Login/logout functionality
- **Data Persistence**: Save/load canvas state
- **Collaboration**: Real-time multi-user editing
- **Export**: Canvas export to various formats
- **Templates**: Pre-built note layouts and themes

### Architecture Benefits
- **Modular Design**: Easy to add new features
- **Type Safety**: Rust's type system prevents runtime errors
- **Performance**: WebAssembly provides near-native speed
- **Maintainability**: Clean separation of concerns

## Success Criteria

### Functional Requirements
- ✅ Infinite canvas with smooth pan and zoom
- ✅ Sticky note creation, editing, and deletion
- ✅ Rich text formatting in notes
- ✅ Floating toolbar with all required actions
- ✅ Proper coordinate system transformations
- ✅ Cross-browser compatibility

### Quality Requirements
- ✅ Comprehensive test coverage (unit, integration, E2E)
- ✅ Visual regression testing
- ✅ Error recovery and graceful degradation
- ✅ Security through input sanitization
- ✅ Performance meeting 60fps target

### User Experience Requirements
- ✅ Intuitive mouse and keyboard controls
- ✅ Responsive design for different screen sizes
- ✅ Clear visual feedback for interactions
- ✅ Accessible keyboard navigation
- ✅ Professional, clean interface design