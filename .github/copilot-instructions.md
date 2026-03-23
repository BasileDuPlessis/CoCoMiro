# Hello World - Rust/WASM Development Guidelines

## Code Style
- Use `rustfmt` and `clippy`
- Explicit error handling, no unwrap/expect
- Meaningful names, Rust conventions

## WASM Guidelines
- Minimize allocations in hot paths
- Cache operations, handle async properly
- Check browser compatibility for web-sys

## Testing & TDD
- Write tests BEFORE code (TDD)
- Use `wasm-bindgen-test` for WASM tests
- Run with `wasm-pack test --node`
- Target >80% coverage

## Debugging
- Check browser console for WASM loading errors
- Verify CORS for ES modules

## Trunk Development
- `trunk serve` for hot reload
- Never hardcode asset paths
- `trunk build --release` for production

## Running the App
- Frontend: `trunk serve --open` from frontend directory
- Backend: `cd backend && cargo run`

## Architecture
- Yew components with immutable state
- Workspace: frontend (WASM) + backend (Axum) + shared types