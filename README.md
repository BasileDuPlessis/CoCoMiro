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

1. Install the wasm target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. Install `trunk` once if needed:
   ```bash
   cargo install trunk
   ```
3. Start the local app from the project root:
   ```bash
   trunk serve --open
   ```
4. Visit <http://127.0.0.1:8080> if the browser does not open automatically.

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
