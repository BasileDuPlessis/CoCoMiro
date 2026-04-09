# CoCoMiro

A small **Rust + WebAssembly infinite canvas** prototype with drag-to-pan, wheel zoom, and an opt-in headless browser regression test.

> Requires a current Rust toolchain with **edition 2024** support.

## Features

- **Infinite-feeling canvas** with a movable grid and note card
- **Cursor-anchored zoom** for smoother navigation
- **Keyboard-accessible pan/zoom** with arrow keys, `+/-`, and `0` to reset
- **HiDPI-aware rendering** for sharper output on Retina displays
- **Rust E2E coverage** using `headless_chrome`

## Run it locally with Trunk

From the project root:

1. Install the wasm target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. Install `trunk` once if needed:
   ```bash
   cargo install trunk
   ```
3. **Verify both compilation targets** before running:
   ```bash
   # Test host compilation (your local machine)
   cargo check
   cargo test

   # Test WebAssembly compilation (required for browser)
   cargo check --target wasm32-unknown-unknown
   ```
4. Start the local app:
   ```bash
   trunk serve --address 127.0.0.1 --port 8080 --open
   ```
5. If the browser does not open automatically, visit <http://127.0.0.1:8080/>.

### ⚠️ Important: Test Both Compilation Targets

**Always test both host and WebAssembly targets** - they catch different issues:

- `cargo check` - Tests compilation for your local machine
- `cargo check --target wasm32-unknown-unknown` - Tests compilation for the browser

Code that compiles for your host machine may fail in WebAssembly due to:
- Missing WASM-specific imports
- Conditional compilation (`#[cfg(target_arch = "wasm32")]`) issues
- WebAssembly API limitations

## Development Workflow

When making changes:

1. **Test host compilation**: `cargo check && cargo test`
2. **Test WebAssembly compilation**: `cargo check --target wasm32-unknown-unknown`
3. **Run the app**: `trunk serve --address 127.0.0.1 --port 8080`

Never skip the WebAssembly check - it catches issues that host compilation misses!

### Troubleshooting the browser build

If `cargo test` passes but the app does not start in the browser:

## Run tests

### Unit tests

```bash
cargo test --lib
```

### Opt-in browser E2E

```bash
cargo e2e
```

Equivalent command:

```bash
cargo test --test e2e_home -- --ignored
```
