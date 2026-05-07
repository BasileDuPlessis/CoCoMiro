# Architecture Decision Records (ADRs)

This document contains the key architectural decisions made during the development of CoCoMiro, an infinite canvas application built with Rust and WebAssembly.

## ADR 001: Technology Stack - Rust + WebAssembly

### Context
CoCoMiro needed to be a high-performance web application with complex 2D graphics, coordinate transformations, and real-time interactions. Traditional web technologies (JavaScript/TypeScript + Canvas/DOM) were considered, but performance and type safety concerns led to evaluating alternatives.

### Decision
Use Rust compiled to WebAssembly as the primary implementation language, with HTML5 Canvas for rendering.

### Rationale
- **Performance**: WebAssembly provides near-native performance for computationally intensive tasks like coordinate transformations and canvas rendering
- **Type Safety**: Rust's compile-time guarantees prevent runtime errors in complex state management
- **Maintainability**: Strong typing and ownership system make the codebase more reliable and easier to refactor
- **Ecosystem**: Rich Rust ecosystem provides high-quality libraries for data structures and algorithms
- **Future-Proofing**: WebAssembly is a W3C standard with growing browser support

### Consequences
- **Positive**:
  - Excellent runtime performance for canvas operations
  - Compile-time error prevention
  - Memory safety without garbage collection overhead
  - Easy integration with existing Rust testing infrastructure
- **Negative**:
  - Larger initial bundle size compared to minified JavaScript
  - Tooling complexity (wasm-bindgen, trunk, etc.)
  - Limited access to browser APIs compared to JavaScript
  - Steeper learning curve for web developers

### Alternatives Considered
- **JavaScript/TypeScript + Canvas**: Familiar web ecosystem but performance limitations for complex coordinate math
- **JavaScript/TypeScript + WebGL**: Overkill for 2D graphics, increased complexity
- **Pure Rust native application**: Would not run in browsers, limiting accessibility

## ADR 002: Rendering Architecture - Immediate Mode Canvas

### Context
The application needs to render an infinite 2D canvas with sticky notes, grid lines, and UI elements. Performance is critical for smooth 60fps interactions.

### Decision
Use HTML5 Canvas 2D API with immediate mode rendering, redrawing the entire scene on each frame.

### Rationale
- **Simplicity**: Immediate mode eliminates complex state synchronization between retained mode objects
- **Performance**: Modern GPUs and Canvas implementations handle full redraws efficiently
- **Flexibility**: Easy to implement complex rendering logic like text formatting and coordinate transformations
- **Consistency**: Single rendering path for all visual elements
- **Debugging**: Easier to debug rendering issues with direct control over drawing operations

### Consequences
- **Positive**:
  - Straightforward implementation of complex rendering features
  - No retained mode state management complexity
  - Excellent performance on modern hardware
  - Easy to add new visual features
- **Negative**:
  - Higher CPU usage during redraws
  - Potential for overdraw on complex scenes
  - Manual optimization required for performance-critical paths

### Alternatives Considered
- **DOM-based rendering**: Would not scale to infinite canvas, poor performance for frequent updates
- **WebGL**: Overkill for 2D graphics, significantly increased complexity
- **SVG**: Limited performance for dynamic content, complex coordinate transformations

## ADR 003: Coordinate System - World vs Screen Separation

### Context
The application implements an infinite canvas where users can pan and zoom. Mouse interactions need to work consistently regardless of viewport position and zoom level.

### Decision
Maintain separate world coordinates (infinite 2D space) and screen coordinates (canvas pixels), with automatic transformations between them.

### Rationale
- **Mathematical Correctness**: Proper separation allows accurate coordinate transformations
- **User Experience**: Zoom and pan operations feel natural and predictable
- **Implementation Clarity**: Clear distinction between world state and view state
- **Extensibility**: Easy to add features like multiple viewports or collaborative editing
- **Testing**: Coordinate transformations can be unit tested independently

### Consequences
- **Positive**:
  - Accurate cursor anchoring during zoom operations
  - Consistent interaction behavior across zoom levels
  - Clean separation of concerns between model and view
  - Easy to implement features like minimaps or multiple viewports
- **Negative**:
  - Additional complexity in coordinate conversion functions
  - Need to transform coordinates for every mouse interaction
  - Potential for off-by-one errors in transformation math

### Alternatives Considered
- **Screen-only coordinates**: Would break zoom and pan functionality
- **Single coordinate system**: Would limit the "infinite" nature of the canvas

## ADR 004: State Management - Rc<RefCell<AppState>>

### Context
The application has complex state with viewport settings, multiple sticky notes, selection state, and user interactions. State needs to be shared between event handlers and the render loop.

### Decision
Use `Rc<RefCell<AppState>>` for shared mutable state across the application.

### Rationale
- **Shared Ownership**: `Rc` allows multiple references to the same state
- **Interior Mutability**: `RefCell` enables mutation through shared references
- **Performance**: Minimal overhead compared to message passing architectures
- **Simplicity**: Direct state access without complex async patterns
- **Rust Idiomatic**: Follows Rust patterns for shared mutable state

### Consequences
- **Positive**:
  - Simple and direct state access from anywhere in the application
  - No complex message passing or actor systems
  - Good performance for frequent state updates
  - Easy to reason about data flow
- **Negative**:
  - Potential for runtime borrow checker panics
  - No compile-time guarantees about mutable access patterns
  - Requires careful design to avoid circular references

### Alternatives Considered
- **Message passing with channels**: Would be overkill for a single-threaded web application
- **Redux-style state management**: Too heavy for Rust/WebAssembly context
- **Immutable state with structural sharing**: Performance overhead for frequent updates

## ADR 005: Event Handling - Custom Closure-Based System

### Context
The application needs to handle mouse, keyboard, and window events, coordinating between user input and state updates.

### Decision
Implement a custom event handling system using closures and direct DOM event listeners.

### Rationale
- **Performance**: Direct event listeners avoid framework overhead
- **Control**: Full control over event propagation and handling logic
- **Integration**: Easy integration with WebAssembly and browser APIs
- **Flexibility**: Can implement complex interaction patterns like drag operations
- **Debugging**: Clear event flow makes debugging easier

### Consequences
- **Positive**:
  - Excellent performance with minimal latency
  - Full control over event handling logic
  - Easy to implement complex interactions (drag, multi-touch, etc.)
  - No framework dependencies for core functionality
- **Negative**:
  - Manual implementation of common patterns
  - Potential for memory leaks if closures aren't properly managed
  - More boilerplate code compared to framework approaches

### Alternatives Considered
- **Web framework (React, Vue, etc.)**: Would add significant bundle size and complexity
- **Game engine event systems**: Overkill for 2D canvas application
- **Browser native event delegation**: Would require more complex routing logic

## ADR 006: Text Rendering - HTML Parsing on Canvas

### Context
Sticky notes need rich text formatting (bold, italic, underline) while maintaining high performance in a canvas-based rendering system.

### Decision
Parse HTML-formatted text and render formatted segments directly to canvas using the 2D API.

### Rationale
- **Rich Text Support**: Allows users to format text without external libraries
- **Performance**: Direct canvas rendering avoids DOM manipulation overhead
- **Consistency**: Same rendering path for all text, formatted or not
- **Security**: HTML parsing can be sanitized to prevent XSS attacks
- **Flexibility**: Easy to add new formatting options

### Consequences
- **Positive**:
  - Rich text editing without DOM elements
  - Consistent performance for all text rendering
  - No external dependencies for text formatting
  - Easy to implement text selection and editing
- **Negative**:
  - Complex HTML parsing implementation
  - Limited to canvas-supported text features
  - Manual text layout and measurement

### Alternatives Considered
- **DOM text elements**: Would break canvas-based architecture
- **External rich text libraries**: Would increase bundle size
- **Plain text only**: Would limit user functionality

## ADR 007: Testing Strategy - Headless Browser E2E

### Context
The application is WebAssembly-based with complex user interactions. Unit tests alone cannot verify end-to-end functionality.

### Decision
Use Headless Chrome for end-to-end testing, supplemented by unit tests and visual regression testing.

### Rationale
- **Real Browser Environment**: Tests run in actual browser with WebAssembly support
- **User-Centric Testing**: Verifies complete user workflows from click to render
- **Visual Regression**: Catches unintended visual changes
- **Integration Testing**: Tests component interactions in realistic conditions
- **CI/CD Friendly**: Headless mode works in automated environments

### Consequences
- **Positive**:
  - High confidence in application functionality
  - Catches integration issues missed by unit tests
  - Visual regression prevents UI bugs
  - Tests real user interactions
- **Negative**:
  - Slower test execution compared to unit tests
  - More complex test setup and maintenance
  - Potential for flaky tests due to timing issues

### Alternatives Considered
- **Unit tests only**: Would miss integration and browser-specific issues
- **Manual testing**: Not scalable or repeatable
- **Selenium WebDriver**: More complex setup than headless Chrome

## ADR 008: Error Handling - Custom Error Types with Recovery

### Context
WebAssembly applications can encounter various runtime errors including canvas context loss, network issues, and rendering failures.

### Decision
Implement custom error types with graceful recovery mechanisms and fallback rendering.

### Rationale
- **User Experience**: Application continues functioning despite errors
- **Debugging**: Structured error information aids troubleshooting
- **Robustness**: Recovery mechanisms prevent complete application failure
- **Maintainability**: Centralized error handling logic
- **Performance**: Fallback rendering provides basic functionality during errors

### Consequences
- **Positive**:
  - Application remains usable during error conditions
  - Clear error reporting for debugging
  - Graceful degradation instead of crashes
  - Structured approach to error handling
- **Negative**:
  - Additional complexity in error handling code
  - Need to design fallback behaviors for all features

### Alternatives Considered
- **Panic on errors**: Would crash the application
- **Generic error handling**: Less specific error information
- **No error recovery**: Poor user experience during failures

## ADR 009: Modular Architecture - Feature-Based Modules

### Context
The application has multiple concerns: canvas rendering, sticky notes, viewport management, event handling, etc.

### Decision
Organize code into feature-based modules with clear separation of concerns.

### Rationale
- **Maintainability**: Related code is grouped together
- **Testability**: Modules can be tested independently
- **Extensibility**: Easy to add new features without affecting existing code
- **Code Navigation**: Clear structure makes the codebase easier to understand
- **Reusability**: Modules can be reused or extracted if needed

### Consequences
- **Positive**:
  - Clear code organization and responsibilities
  - Easier testing and debugging
  - Reduced coupling between features
  - Better code reuse opportunities
- **Negative**:
  - Need to design clear module boundaries
  - Potential for circular dependencies if not careful

### Alternatives Considered
- **Single large file**: Would be difficult to maintain
- **Technical layering**: Would couple unrelated features
- **Micro-modules**: Would create too much overhead

## ADR 010: Performance Optimization - Frame Skipping and Monitoring

### Context
Canvas applications need to maintain smooth 60fps performance while handling complex rendering and interactions.

### Decision
Implement frame skipping for slow operations and performance monitoring with metrics collection.

### Rationale
- **User Experience**: Maintains responsive feel even during heavy operations
- **Debugging**: Performance metrics help identify bottlenecks
- **Scalability**: Application can handle varying complexity gracefully
- **Quality Assurance**: Performance regression detection
- **Resource Management**: Prevents excessive CPU usage during slow operations

### Consequences
- **Positive**:
  - Consistent frame rate under varying load
  - Performance visibility and monitoring
  - Graceful handling of complex scenes
  - Data-driven performance optimization
- **Negative**:
  - Additional complexity in render loop
  - Need to tune frame skipping thresholds
  - Performance monitoring overhead

### Alternatives Considered
- **No frame skipping**: Would cause stuttering during slow operations
- **Lower target frame rate**: Would feel less responsive
- **Async rendering**: Would complicate the architecture significantly