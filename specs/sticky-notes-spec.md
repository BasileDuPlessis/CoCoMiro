# Sticky Notes Feature Specification

## Implementation Tasks

### Phase 1: Core Infrastructure
- [ ] **Task 1.1**: Create floating toolbar component
  - Add a floating toolbar UI element that remains visible on the canvas
  - Position it appropriately (e.g., top-right corner)
  - Ensure it doesn't interfere with canvas interactions

- [ ] **Task 1.2**: Implement basic sticky note data structure
  - Define StickyNote struct/type with properties: id, position, content, size
  - Add state management for sticky notes collection
  - Integrate with existing canvas state management

### Phase 2: Creation and Basic Interaction
- [ ] **Task 2.1**: Add create sticky note button to toolbar
  - Add button with appropriate icon (e.g., sticky note symbol)
  - Implement click handler to create new sticky note
  - Position new sticky notes at center of current view

- [ ] **Task 2.2**: Create sticky note component
  - Build basic sticky note UI component
  - Render sticky notes on canvas at correct positions
  - Apply basic styling (yellow background, shadow, etc.)

### Phase 3: Text Editing
- [ ] **Task 3.1**: Implement click-to-edit functionality
  - Add click handler to enter edit mode
  - Switch to textarea/input for text editing
  - Handle focus and cursor positioning

- [ ] **Task 3.2**: Add text editing controls
  - Implement exit edit mode on Enter or click outside
  - Auto-save changes when editing completes
  - Support basic text formatting (plain text minimum)

### Phase 4: Drag and Drop
- [ ] **Task 4.1**: Implement drag functionality
  - Add mouse down/up/move event handlers
  - Calculate drag deltas and update position
  - Prevent text selection during drag

- [ ] **Task 4.2**: Add drag visual feedback
  - Change cursor during drag operation
  - Highlight sticky note being dragged
  - Show drag preview or ghost effect

### Phase 5: Deletion
- [ ] **Task 5.1**: Implement selection state
  - Track which sticky note is currently selected
  - Add visual indication for selected state
  - Handle click outside to deselect

- [ ] **Task 5.2**: Add keyboard deletion
  - Listen for Delete key when sticky note is selected
  - Remove sticky note from state and UI
  - Consider undo functionality

### Phase 6: Persistence and Polish
- [ ] **Task 6.1**: Implement data persistence
  - Save sticky notes to local storage or backend
  - Load sticky notes on application startup
  - Handle data migration if schema changes

- [ ] **Task 6.2**: Add UI polish and feedback
  - Implement hover states for interactive elements
  - Add tooltips for toolbar buttons
  - Ensure smooth animations and transitions

### Phase 7: Testing and Optimization
- [ ] **Task 7.1**: Performance optimization
  - Optimize rendering for multiple sticky notes
  - Ensure smooth drag operations
  - Test with large numbers of sticky notes

- [ ] **Task 7.2**: Accessibility and testing
  - Add keyboard navigation support
  - Implement screen reader compatibility
  - Write unit and integration tests
  - Test across supported browsers and devices

