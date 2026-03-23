# Infinite Canvas Implementation Tasks

This document breaks down the infinite canvas feature specification into manageable implementation tasks.

## Task 1: Project Setup and Dependencies
- Set up Yew framework with WebAssembly support
- Configure Trunk build tool for compilation and serving
- Add web-sys crate for DOM and Canvas API interaction
- Ensure modern browser compatibility (WebAssembly, Canvas 2D)
- Verify hardware acceleration support

## Task 2: Basic Canvas Component Structure
- Create Yew component for the infinite canvas
- Implement HTML5 Canvas element with 2D context
- Set canvas size dynamically to window inner dimensions (capped at 3000x2000 pixels)
- Add solid white background fill to prevent flickering
- Set cursor style to "grab" for pan indication

## Task 3: View State Management
- Implement state management using Yew's use_state hook
- Define zoom level as f64 (1.0 = 100% magnification, minimum 0.1)
- Define pan position as (pan_x, pan_y) f64 values in screen coordinates
- Add drag state boolean flag and last mouse position tracking
- Ensure state persists only in memory (no session persistence)

## Task 4: Zoom Functionality Implementation
- Add zoom in (+) and zoom out (-) button controls
- Position zoom buttons in top-left corner (10px from top-left edge)
- Implement mouse wheel scrolling for zoom (centered on mouse position)
- Add keyboard shortcuts: Ctrl+Plus/Ctrl+Equals to zoom in, Ctrl+Minus to zoom out
- Enforce zoom limits (minimum 0.1)
- Calculate zoom factors: +20% for zoom in, ~16.7% for zoom out
- Update canvas rendering on zoom changes

## Task 5: Pan Functionality Implementation
- Implement click and drag panning on canvas
- Track mouse down, move, and up events
- Update pan_x and pan_y based on mouse delta movement
- Ensure natural pan behavior (canvas moves opposite to drag direction)
- Support infinite movement in all directions (no boundaries)
- Maintain smooth 60fps performance during drag operations

## Task 6: Grid System Implementation
- Draw horizontal and vertical grid lines as visual reference points
- Space grid lines 50 world units apart
- Implement adaptive grid line width based on zoom level (thinner at higher zoom)
- Use screen-space coordinate calculations for precise rendering
- Draw lines directly to canvas without external libraries
- Ensure grid translates correctly with pan operations

## Task 7: Event Handling Integration
- Attach direct event listeners to canvas element for mouse and keyboard events
- Handle mouse wheel events for zoom
- Handle mouse down/move/up for panning
- Handle keyboard events for shortcuts
- Prevent default browser behaviors where necessary (e.g., page scroll on wheel)
- Ensure events update view state correctly
- **CRITICAL**: Implement canvas redrawing effects that trigger on state changes (zoom, pan)
- Use Yew's use_effect_with_deps to redraw canvas when view state updates
- Ensure canvas redraws immediately after state changes for responsive interaction
- **CRITICAL**: Use proper dependency cloning in use_effect_with_deps - clone state values instead of dereferencing to avoid compilation errors

## Task 8: Debug Overlay Implementation
- Create semi-transparent overlay in top-left corner
- Display real-time zoom level, pan coordinates (pan_x, pan_y), and drag state
- Update overlay on every render cycle
- Position overlay to not interfere with canvas interaction
- Moved debug overlay from top-left to top-right corner to avoid overlapping zoom buttons
- Use simple text rendering for debug information

## Task 9: Performance and Rendering Optimization
- Implement imperative canvas drawing with screen-space coordinates
- Avoid transform matrices for precision
- Ensure 60fps animation during zoom and pan operations
- Optimize grid drawing for large canvas areas
- Limit canvas size to 3000x2000 pixels for performance
- Manage memory efficiently for infinite space concept
- **CRITICAL**: Ensure reactive rendering - canvas must redraw immediately when view state changes
- Implement proper dependency tracking for canvas redraw effects
- Test that all state changes (pan, zoom) trigger immediate visual updates

## Task 10: Accessibility Features
- Ensure keyboard shortcuts work (Ctrl+Plus, Ctrl+Minus)
- Implement tab navigation support for zoom buttons
- Add proper ARIA labels and roles where applicable
- Test keyboard-only navigation
- Verify screen reader compatibility

## Task 11: Testing and Validation
- Implement unit tests for zoom calculations and limits
- Test pan functionality with mouse simulation
- Validate grid drawing at different zoom levels
- Performance test for 60fps during operations
- Test boundary handling for infinite movement
- Cross-browser compatibility testing
- **CRITICAL**: Test reactive canvas updates - verify canvas redraws immediately on state changes
- Test that pan operations update visual grid position in real-time
- Test that zoom operations update visual grid scale immediately
- Validate that no state changes are "silent" (don't trigger visual updates)
- **CRITICAL**: Validate compilation succeeds - ensure no "cannot move out of dereference" errors in use_effect_with_deps

## Task 12: Integration and Final Polish
- Integrate all features into cohesive application
- Test complete user workflows (zoom + pan + grid)
- Optimize UI layout: Position zoom buttons in top-left, debug overlay in top-right to prevent overlap
- Add any missing error handling
- Optimize build and serve process with Trunk
- Document usage and deployment instructions
- Final validation against specification requirements
- **CRITICAL**: Perform manual testing of reactive canvas updates - drag to pan and verify grid moves immediately
- **CRITICAL**: Test zoom operations and verify immediate visual feedback
- **CRITICAL**: Ensure no "static canvas" bugs where state changes don't trigger redraws
- **CRITICAL**: Validate compilation - ensure use_effect_with_deps dependencies are properly cloned (not dereferenced) to avoid "cannot move out of dereference" errors