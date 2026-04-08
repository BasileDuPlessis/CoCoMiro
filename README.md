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
3. Start the local app:
   ```bash
   trunk serve --address 127.0.0.1 --port 8080 --open
   ```
4. If the browser does not open automatically, visit <http://127.0.0.1:8080/>.

### Troubleshooting the browser build

If `cargo test` passes but the app does not start in the browser, check the wasm build directly:

```bash
cargo check --target wasm32-unknown-unknown
```

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
