# Mori - Infinite Canvas Application Development Guidelines

## Project Overview
Mori is a modern infinite canvas application built with Rust, featuring:
- **Frontend**: Yew WebAssembly application with HTML5 Canvas 2D rendering
- **Backend**: Axum REST API server with CORS support
- **Shared**: Type-safe Rust models for frontend-backend communication
- **Testing**: E2E tests with Playwright (no unit tests - they only tested math, not user behavior)

## Code Style & Best Practices
- Use `rustfmt` and `clippy` for consistent formatting and linting
- Explicit error handling - no `unwrap()` or `expect()` in production code
- Meaningful variable/function names following Rust conventions
- Prefer immutable data structures and functional programming patterns
- Use `#[allow(deprecated)]` for Canvas API methods that are deprecated but still widely supported

## Architecture Guidelines

### Frontend (Yew + WASM)
- **State Management**: Use Yew's `use_state` hook for component state
- **Event Handling**: Implement mouse, keyboard, and canvas events properly
- **Performance**: Minimize allocations in render loops, cache calculations
- **Canvas API**: Use web-sys bindings for DOM and Canvas 2D context
- **Async Operations**: Handle WASM async properly with proper error boundaries

### Backend (Axum)
- **API Design**: RESTful endpoints with JSON responses
- **CORS**: Enable CORS for frontend communication
- **Error Handling**: Return proper HTTP status codes and error messages
- **Health Checks**: Provide `/health` endpoint for monitoring

### Shared Types
- **Serialization**: Use Serde for JSON serialization/deserialization
- **Type Safety**: Ensure all API communication is type-safe
- **Common Models**: Define shared structs for API requests/responses

## Testing Strategy (IMPORTANT)

### E2E Testing with Playwright (PRIMARY)
We use E2E tests instead of unit tests because they verify ACTUAL user behavior:

```bash
cd frontend

# Run all tests
npm test

# Run with browser visible
npm run test:headed

# Interactive debugging
npm run test:ui

# Step-by-step debugging
npm run test:debug

# View test reports
npx playwright show-report
```

**Test Coverage Focus**:
- Canvas loads and displays correctly
- Zoom buttons increase/decrease zoom level
- Mouse drag panning moves canvas
- Keyboard shortcuts work (Ctrl+Plus/Minus)
- Mouse wheel zoom functions
- Canvas maintains functionality after multiple interactions

### Why E2E Over Unit Tests?
Unit tests only verify mathematical calculations (e.g., "zoom * 1.2 = new_zoom"). E2E tests verify real user interactions (e.g., "click + drag = canvas moves"). This provides much more valuable coverage for UI applications.

### Compilation Testing
```bash
# Test that everything compiles
cargo build

# Test workspace compilation
cargo build --workspace
```

## Development Workflow

### Running the Application
```bash
# Backend API server (port 3000)
cd backend && cargo run

# Frontend dev server (port 8080)
# IMPORTANT: Build frontend package first due to workspace dependencies
cargo build --package hello-world-frontend
cd frontend && trunk serve --open
```

### Building for Production
```bash
# Frontend WASM build
cd frontend && trunk build --release

# Backend optimized build
cd backend && cargo build --release
```

## Browser Compatibility
- **WebAssembly**: Required for WASM execution
- **Canvas 2D**: Required for rendering
- **Modern Browsers**: Chrome 90+, Firefox 88+, Safari 14+, Edge 90+
- **Hardware Acceleration**: Recommended for smooth 60fps performance

## Debugging Guidelines
- **WASM Issues**: Check browser console for loading errors
- **Canvas Problems**: Verify web-sys bindings and context creation
- **API Issues**: Check CORS configuration and network requests
- **Performance**: Use browser dev tools for frame rate analysis
- **Test Failures**: Use `npm run test:ui` for interactive debugging

## Trunk Development Server
- `trunk serve --port 8080` for development with hot reload
- Never hardcode asset paths - use Trunk's asset handling
- `trunk build --release` for production optimization
- Check `Trunk.toml` for build configuration

## Port Configuration
- **Backend API**: `http://localhost:3000`
- **Frontend App**: `http://localhost:8080`
- **Test Server**: Automatically managed by Playwright webServer config

## Code Organization
- **Components**: Keep Yew components focused and reusable
- **State**: Use appropriate state management for component complexity
- **Events**: Handle user interactions efficiently without blocking render
- **API Calls**: Use proper async patterns for backend communication
- **Error Boundaries**: Implement error handling for WASM panics

## Performance Considerations
- **Canvas Rendering**: Minimize draw operations, use efficient algorithms
- **Memory Management**: Avoid memory leaks in long-running WASM applications
- **Event Handling**: Debounce rapid events (mouse move, scroll)
- **State Updates**: Batch state changes to reduce re-renders
- **Bundle Size**: Keep WASM bundle optimized for web delivery

## Security Notes
- **CORS**: Properly configured for frontend-backend communication
- **Input Validation**: Validate all user inputs on backend
- **API Keys**: Never commit secrets to version control
- **HTTPS**: Use HTTPS in production deployments

## Documentation Updates
When making changes:
1. Update this instruction file if development practices change
2. Update README.md for user-facing changes
3. Update specs/ for new features or requirements
4. Ensure tests cover new functionality
5. Run full test suite before committing

## Common Issues & Solutions

### Port Conflicts
- Backend runs on port 3000, frontend on 8080
- If ports are in use, check for running processes
- Use `lsof -i :3000` to find conflicting processes

### WASM Build Issues
- Ensure `wasm32-unknown-unknown` target is installed
- Clear `target/` directory if build cache issues
- Check Rust version compatibility

### Test Failures
- Run `npm run test:ui` for interactive debugging
- Check browser console for runtime errors
- Verify Playwright browsers are installed
- Ensure backend is not running on frontend port

### Canvas Rendering Issues
- Check browser WebAssembly support
- Verify Canvas 2D context creation
- Test with different browsers
- Check for deprecated API usage (use `#[allow(deprecated)]`)