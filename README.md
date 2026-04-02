# CoCoMiro

A minimal Rust + WebAssembly example that displays **Hello world from CoCoMiro!** in the browser.

## Run it locally with Trunk

1. Install the wasm target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. Install `trunk` once if needed:
   ```bash
   cargo install trunk
   ```
3. Start the Rust web server from the project root:
   ```bash
   trunk serve --open
   ```
4. Then open <http://127.0.0.1:8080> if it does not open automatically.

## Run the Rust E2E test

This project includes a **headless browser E2E test written in Rust** that checks the homepage `<h1>`.

```bash
cargo e2e
```

You can also run the whole test suite with:

```bash
cargo test
```
