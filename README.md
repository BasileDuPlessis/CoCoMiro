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

## Run tests

### Fast default tests

Run the regular unit suite with:

```bash
cargo test
```

### Opt-in browser E2E

The headless browser regression test is **opt-in** so normal development stays fast.
Run it explicitly with:

```bash
cargo e2e
```

Equivalent command:

```bash
cargo test --test e2e_home -- --ignored
```
