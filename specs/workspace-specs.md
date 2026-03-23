# Hello World - Workspace Setup Specifications

## Overview
Setup specifications for a simple Hello World application: WASM frontend with Trunk + Rust backend communication. Follows TDD principles.

## Prerequisites

### 1. .gitignore Setup
Create a comprehensive `.gitignore` file for Rust/WASM development:

```
/target/
/Cargo.lock
**/*.rs.bk
*.pdb

# Trunk specific
/dist/
/.wasm-target/

# WASM specific
/pkg/
/static/pkg/
/node_modules/
/build/

# IDE and editor files
.vscode/
.idea/
*.swp
*.swo
*~

# OS specific
.DS_Store
Thumbs.db

# Logs
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Environment variables
.env
.env.local
.env.development.local
.env.test.local
.env.production.local

# Temporary files
*.tmp
*.temp
```

### 2. .copilot-instructions.md Setup
Create `.copilot-instructions.md` with development guidelines:

```markdown
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

## Architecture
- Yew components with immutable state
- Workspace: frontend (WASM) + backend (Axum) + shared types
```

## 3. Project Structure 

### Workspace Architecture
For full-stack applications with backend communication:

```
hello-world/
├── Cargo.toml                    # Workspace configuration
├── Cargo.lock
├── frontend/
│   ├── Cargo.toml               # WASM crate
│   ├── Trunk.toml               # Trunk build config
│   ├── index.html               # Main HTML file
│   ├── src/
│   │   ├── lib.rs              # WASM library
│   │   └── api.rs              # API communication
│   └── static/                 # Static assets
├── backend/
│   ├── Cargo.toml              # Server crate
│   └── src/
│       └── main.rs             # Server entry point with health check
└── shared/
    ├── Cargo.toml             # Shared types/models
    └── src/
        └── lib.rs             # Common types
```

### Frontend Cargo.toml (Workspace)
```toml
[package]
name = "hello-world-frontend"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
yew = { version = "0.20", features = ["csr", "ssr"] }
web-sys = { version = "0.3", features = [
  "console", "Window", "Document", "Element", "HtmlElement", "Node", "Text",
  "Request", "RequestInit", "RequestMode", "Response", "Headers"
] }
js-sys = "0.3"
wasm-bindgen-test = "0.3"
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
hello-world-shared = { path = "../shared" }  # Shared types

[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-O", "--enable-reference-types", "shrink-level=1"]
```

### Backend Cargo.toml (Workspace)
```toml
[package]
name = "hello-world-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
hello-world-shared = { path = "../shared" }  # Shared types

[[bin]]
name = "hello-world-backend"
path = "src/main.rs"
```

### Shared Cargo.toml (Workspace)
```toml
[package]
name = "hello-world-shared"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

### Root Cargo.toml (Workspace)
```toml
[workspace]
members = ["frontend", "backend", "shared"]
```

### Trunk.toml Configuration
```toml
[build]
target = "index.html"
dist = "dist"

[[hooks]]
stage = "pre_build"
command = "echo"
command_arguments = ["Building Hello World..."]

[[hooks]]
stage = "post_build"
command = "cp"
command_arguments = ["-r", "static/.", "dist/"]
```

### Development Tools Setup
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Install WASM target: `rustup target add wasm32-unknown-unknown`
3. Install Trunk: `cargo install trunk`
4. Install cargo-watch: `cargo install cargo-watch`

## 4. Full-Stack Development Workflow

### Running the Full-Stack Application
```bash
# Terminal 1: Start the backend server
cd backend
cargo run

# Terminal 2: Start the frontend development server
cd frontend
trunk serve --port 8080
```

### API Communication
Frontend makes HTTP requests to backend for health check:

```rust
// frontend/src/api.rs
use wasm_bindgen::JsValue;
use web_sys::{Request, RequestInit, RequestMode, Response};

pub async fn health_check() -> Result<String, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("http://localhost:3000/health", &opts)?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().map_err(|_| JsValue::from_str("Failed to cast response"))?;

    match resp.status() {
        200 => {
            let text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
            Ok(text.as_string().unwrap_or_default())
        },
        _ => Err(JsValue::from_str("Health check failed")),
    }
}
```

### CORS Configuration
Backend must allow requests from frontend:

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

let app = Router::new()
    .route("/health", get(health_check))
    .layer(cors);
```

## 4. Testing Tooling Setup

### Rust Unit Testing
- Built-in `#[test]` attribute
- Run with `cargo test`
- Use `assert!`, `assert_eq!`, `assert_ne!` macros

### WASM Testing with Trunk
- `wasm-bindgen-test` for browser-compatible tests
- Run headless: `wasm-pack test --node`
- Run in browser: `wasm-pack test --chrome --firefox --safari`
- Trunk serves test files automatically

### Integration Testing
- Test WASM module loading
- Test DOM interactions
- Test API communication

### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(function(), expected);
    }
}

#[cfg(test)]
mod wasm_tests {
    use wasm_bindgen_test::*;
    use super::*;

    #[wasm_bindgen_test]
    fn test_dom() {
        // DOM interaction tests
    }
}
```

## 5. Test Driven Development (TDD) Workflow

### TDD Cycle with Trunk
1. **RED**: Write a failing test first
2. **GREEN**: Implement minimal code to pass the test
3. **REFACTOR**: Improve code while keeping tests passing
4. **VERIFY**: Use `trunk serve` to test in browser

### Example TDD for "HELLO WORLD"
1. Write test in `frontend/src/lib.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use wasm_bindgen_test::*;

       #[wasm_bindgen_test(async)]
       async fn test_hello_world_display() {
           // Test that hello world is displayed
           // This is a basic test to ensure the component renders
           let rendered = yew::ServerRenderer::<App>::new().render().await;
           assert!(rendered.contains("HELLO WORLD"));
       }
   }
   ```
2. Implement minimal code in `frontend/src/lib.rs`
3. Run `cd frontend && trunk serve` to verify in browser
4. Refactor for better structure

### Continuous Testing
- Auto-run tests: `cargo watch -x test`
- Debug output: `cargo test -- --nocapture`
- Hot reload with Trunk for rapid iteration

## 6. Trunk Development Workflow

### Development Server
```bash
# Start development server with hot reload
trunk serve

# Serve on specific port
trunk serve --port 8080

# Open browser automatically
trunk serve --open

# Serve with release optimizations
trunk serve --release
```

### Building for Production
```bash
# Build optimized version
trunk build --release

# Output goes to ./dist/ directory
# Ready for deployment to any static hosting
```

### Asset Management
- Place static assets in `static/` directory
- Trunk automatically copies them to `dist/`
- Reference assets using relative paths from HTML root
- Trunk handles cache busting automatically

## 7. "HELLO WORLD" Implementation Goal

### Workspace Structure Setup (REQUIRED)
Before implementing the hello world example, establish the full workspace structure as defined in section 3. This ensures scalability and proper separation of concerns from the start.

1. Create the workspace directories: `frontend/`, `backend/`, `shared/`
2. Set up root `Cargo.toml` as workspace with members
3. Create individual `Cargo.toml` files for each crate
4. Move frontend files to `frontend/` directory
5. Create basic backend and shared crates

### Minimal Working Example with Workspace
1. Create `frontend/src/lib.rs`:
   ```rust
   use yew::prelude::*;

   #[function_component(App)]
   pub fn app() -> Html {
       html! { <h1>{ "HELLO WORLD" }</h1> }
   }

   #[cfg(not(test))]
   #[wasm_bindgen::prelude::wasm_bindgen(start)]
   pub fn main() {
       yew::Renderer::<App>::new().render();
   }

   #[cfg(test)]
   mod tests {
       use super::*;
       use wasm_bindgen_test::*;

       #[wasm_bindgen_test(async)]
       async fn test_hello_world_display() {
           let rendered = yew::ServerRenderer::<App>::new().render().await;
           assert!(rendered.contains("HELLO WORLD"));
       }
   }
   ```

2. Create `frontend/index.html`:
   ```html
   <!DOCTYPE html>
   <html>
     <head>
       <meta charset="utf-8" />
       <title>Hello World</title>
       <style>
         body {
           margin: 0;
           padding: 20px;
           font-family: Arial, sans-serif;
           background: #f5f5f5;
         }
         #app {
           max-width: 800px;
           margin: 0 auto;
         }
         h1 {
           color: #333;
           text-align: center;
         }
       </style>
     </head>
     <body>
       <div id="app"></div>
     </body>
   </html>
   ```

3. Start development from frontend directory:
   ```bash
   cd frontend
   trunk serve --open
   ```

4. Open browser: Trunk automatically opens `http://localhost:8080`

### Verification Steps
- Page loads without errors
- "HELLO WORLD" text displays
- Browser console shows no WASM errors
- Hot reload works when code changes
- Workspace structure is properly established for future development

## 8. Health Check Implementation

### Backend Health Check Endpoint
Create a health check endpoint in the backend to verify server connectivity:

```rust
// backend/src/main.rs
use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;
use std::io;

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Frontend Health Check Integration
Add health check functionality to the frontend:

```rust
// frontend/src/lib.rs
use yew::prelude::*;
use wasm_bindgen::JsValue;

mod api;

#[function_component(App)]
pub fn app() -> Html {
    let health_status = use_state(|| "Checking...".to_string());

    {
        let health_status = health_status.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::health_check().await {
                    Ok(status) => health_status.set(format!("Backend: {}", status)),
                    Err(_) => health_status.set("Backend: Unavailable".to_string()),
                }
            });
            || ()
        });
    }

    html! {
        <div>
            <h1>{ "HELLO WORLD" }</h1>
            <p>{ (*health_status).clone() }</p>
        </div>
    }
}
```

### API Module for Health Check
```rust
// frontend/src/api.rs
use wasm_bindgen::JsValue;
use web_sys::{Request, RequestInit, RequestMode, Response};

pub async fn health_check() -> Result<String, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init("http://localhost:3000/health", &opts)?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().map_err(|_| JsValue::from_str("Failed to cast response"))?;

    match resp.status() {
        200 => {
            let text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
            Ok(text.as_string().unwrap_or_default())
        },
        _ => Err(JsValue::from_str("Health check failed")),
    }
}
```

### Running with Health Check
```bash
# Terminal 1: Start backend server
cd backend
cargo run

# Terminal 2: Start frontend with health check
cd frontend
trunk serve --port 8080
```

### Health Check Verification
- Frontend displays "Backend: OK" when server is running
- Frontend displays "Backend: Unavailable" when server is down
- CORS allows cross-origin requests from frontend to backend
- Health check runs automatically on page load

<!-- How to implement with a GEN AI Agent: "Following workspace-specs.md exactly, implement the Hello World application" -->