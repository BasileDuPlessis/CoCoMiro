# Sticky Notes Feature Specification

## Implementation Tasks

### Phase 1: Core Infrastructure
- [ ] **Task 1.1**: Create floating toolbar component
  - Add a floating toolbar UI element that remains visible on the canvas
  - Position it appropriately (e.g., top-right corner)
  - Ensure it doesn't interfere with canvas interactions
  - **Test Requirements**: E2E test to verify toolbar is visible and positioned correctly on canvas load

- [x] **Task 1.2**: Make floating toolbar draggable
  - Implement pointer-based drag and drop using `onpointerdown`, `onpointermove`, `onpointerup`
  - Use `setPointerCapture()` to maintain drag responsiveness even when mouse moves outside element boundaries
  - Add visual grip zone at the top with striped background pattern to indicate draggability
  - Implement proper cursor states (`grab`/`grabbing`) and `touch-action: none` for cross-device support
  - Update toolbar position state in real-time during drag operations
  - **Test Requirements**: E2E test to verify toolbar can be dragged smoothly to new positions, grip zone is visible, and drag works with fast mouse movements

- [x] **Task 1.3**: Implement basic sticky note data structure
  - Define StickyNote struct/type with properties: id, position, content, size
  - Add state management for sticky notes collection
  - Integrate with existing canvas state management
  - **Test Requirements**: E2E test to verify sticky note creation and basic state management

### Phase 2: Creation and Basic Interaction
- [x] **Task 2.1**: Add create sticky note button to toolbar
  - Add button with appropriate icon (e.g., sticky note symbol)
  - Implement click handler to create new sticky note
  - Position new sticky notes at center of current view
  - **Test Requirements**: E2E test to verify clicking toolbar button creates sticky note at center of view

- [x] **Task 2.2**: Create sticky note component
  - Build basic sticky note UI component
  - Render sticky notes on canvas at correct positions
  - Apply basic styling (yellow background, shadow, etc.)
  - **Test Requirements**: E2E test to verify sticky note renders with correct styling and position

### Phase 3: Text Editing
- [x] **Task 3.1**: Implement click-to-edit functionality
  - Add click handler to enter edit mode
  - Switch to textarea/input for text editing
  - Implement local state management for editing content to prevent re-renders on every keystroke
  - Handle focus and cursor positioning using useEffect and DOM refs (autofocus attribute alone insufficient)
  - Separate input event handling from save operations to maintain focus during typing
  - **Test Requirements**: E2E test to verify clicking sticky note enters edit mode with proper focus and maintains focus during multi-character input

- [x] **Task 3.2**: Add text editing controls
  - Implement exit edit mode on Enter or click outside
  - Auto-save changes when editing completes
  - Support basic text formatting (plain text minimum)
  - **Test Requirements**: E2E test to verify text editing, saving on Enter, and saving on outside click

### Phase 4: Drag and Drop
- [ ] **Task 4.1**: Implement drag functionality
  - Add mouse down/up/move event handlers
  - Calculate drag deltas and update position
  - Prevent text selection during drag
  - **Test Requirements**: E2E test to verify sticky note can be dragged to new position on canvas

- [ ] **Task 4.2**: Add drag visual feedback
  - Change cursor during drag operation
  - Highlight sticky note being dragged
  - Show drag preview or ghost effect
  - **Test Requirements**: E2E test to verify visual feedback during drag operations

### Phase 5: Deletion
- [ ] **Task 5.1**: Implement selection state
  - Track which sticky note is currently selected
  - Add visual indication for selected state
  - Handle click outside to deselect
  - **Test Requirements**: E2E test to verify sticky note selection and deselection behavior

- [ ] **Task 5.2**: Add keyboard deletion
  - Listen for Delete key when sticky note is selected
  - Remove sticky note from state and UI
  - Consider undo functionality
  - **Test Requirements**: E2E test to verify Delete key removes selected sticky note

### Phase 6: Persistence and Polish
- [ ] **Task 6.1**: Implement data persistence
  - Save sticky notes to local storage or backend
  - Load sticky notes on application startup
  - Handle data migration if schema changes
  - **Test Requirements**: E2E test to verify sticky notes persist across browser sessions

- [ ] **Task 6.2**: Add UI polish and feedback
  - Implement hover states for interactive elements
  - Add tooltips for toolbar buttons
  - Ensure smooth animations and transitions
  - **Test Requirements**: E2E test to verify hover states and tooltips appear correctly

### Phase 7: Testing and Optimization
- [ ] **Task 7.1**: Performance optimization
  - Optimize rendering for multiple sticky notes
  - Ensure smooth drag operations
  - Test with large numbers of sticky notes
  - **Test Requirements**: E2E performance test to verify smooth operation with 50+ sticky notes

- [ ] **Task 7.2**: Accessibility and testing
  - Add keyboard navigation support
  - Implement screen reader compatibility
  - Write unit and integration tests
  - Test across supported browsers and devices
  - **Test Requirements**: E2E tests for keyboard navigation and accessibility features

