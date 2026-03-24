# Mori - Infinite Canvas Application

A modern infinite canvas application built with Rust, featuring WebAssembly frontend and REST API backend. Provides smooth zoom, pan, and grid functionality for creative work and note-taking.

## 🚀 Features

### Core Functionality
- **Infinite Canvas**: Pan and zoom in any direction without boundaries
- **Smooth Zoom**: Mouse wheel, keyboard shortcuts (Ctrl+Plus/Minus), and UI buttons
- **Intuitive Panning**: Click and drag to move the canvas naturally
- **Adaptive Grid**: Visual reference lines that scale with zoom level
- **Responsive Design**: Dynamic canvas sizing with hardware acceleration

### Technical Features
- **WebAssembly**: High-performance Rust frontend compiled to WASM
- **Real-time Rendering**: 60fps canvas updates with optimized drawing
- **Cross-platform**: Works on all modern browsers with WebAssembly support
- **Type-safe**: Full TypeScript and Rust type safety throughout

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Shared        │    │   Backend       │
│   (Yew + WASM)  │◄──►│   Types         │◄──►│   (Axum)        │
│                 │    │                 │    │                 │
│ • Infinite Canvas│    │ • API Models    │    │ • REST API      │
│ • Zoom/Pan UI   │    │ • Serialization  │    │ • Health Check  │
│ • Event Handling│    │ • Type Safety   │    │ • CORS Support  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

- **Frontend**: Yew WASM application with HTML5 Canvas 2D rendering
- **Backend**: Axum HTTP server with JSON API endpoints
- **Shared**: Common Rust types for type-safe frontend-backend communication

## 🛠️ Development Setup

### Prerequisites
- **Rust**: 1.70+ with WASM target (`rustup target add wasm32-unknown-unknown`)
- **Trunk**: WASM build tool (`cargo install trunk`)
- **Node.js**: 18+ for E2E testing
- **Playwright**: Browser automation for testing

### Quick Start

1. **Clone and setup**:
   ```bash
   git clone <repository-url>
   cd mori
   ```

2. **Install dependencies**:
   ```bash
   # Frontend testing dependencies
   cd frontend && npm install
   cd ..
   ```

3. **Run the application**:
   ```bash
   # Terminal 1: Start backend API
   cd backend && cargo run

   # Terminal 2: Start frontend dev server
   cd frontend && trunk serve --open
   ```

4. **Access the app**:
   - Frontend: http://localhost:8080
   - Backend API: http://localhost:3000

### Development Commands

```bash
# Build entire workspace
cargo build

# Run all tests (comprehensive test suite)
./test-all.sh

# Run E2E tests only
cd frontend && npm test

# View test reports
cd frontend && npx playwright show-report

# Debug tests interactively
cd frontend && npm run test:ui

# Build for production
cd frontend && trunk build --release
cd backend && cargo build --release
```

## 🤖 CI/CD & Quality Assurance

### Automated Testing
The project includes comprehensive automated testing:

- **Pre-commit Hooks**: Tests run automatically before every commit
- **GitHub Actions**: Full CI pipeline on every push/PR
- **Test Runner Script**: `./test-all.sh` for local comprehensive testing

### CI Pipeline
GitHub Actions runs on every push and pull request:
- Code formatting checks (`rustfmt`)
- Linting (`clippy`)
- Full workspace build
- E2E test execution
- Production builds verification

### Pre-commit Quality Gates
Before any commit, the following checks run automatically:
1. Code formatting validation
2. Clippy linting (warnings as errors)
3. Full workspace compilation
4. Complete E2E test suite

To bypass checks (not recommended): `git commit --no-verify`

## 🧪 Testing Strategy

### E2E Testing with Playwright
We use comprehensive E2E tests instead of unit tests because they verify actual user behavior:

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
```

**Test Coverage**:
- ✅ Canvas loads and displays correctly
- ✅ Zoom in/out buttons increase/decrease zoom
- ✅ Mouse drag panning moves canvas
- ✅ Keyboard zoom controls (Ctrl+Plus/Minus)
- ✅ Mouse wheel zoom functionality
- ✅ Canvas maintains functionality after multiple interactions

### Why E2E Over Unit Tests?
Unit tests only verify mathematical calculations, but E2E tests verify real user interactions like "click + drag = canvas moves". This provides much more valuable coverage for UI applications.

## 📚 API Documentation

### Health Check Endpoint
```http
GET /health
```

**Response**:
```json
{
  "status": "OK",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## 🎨 Usage Guide

### Navigation Controls
- **Pan**: Click and drag anywhere on the canvas
- **Zoom In**: Mouse wheel up, Ctrl+Plus, or "+" button
- **Zoom Out**: Mouse wheel down, Ctrl+Minus, or "-" button
- **Reset View**: Zoom and pan are maintained in memory during session

### Canvas Features
- **Infinite Space**: No boundaries - pan and zoom in any direction
- **Adaptive Grid**: Reference lines scale with zoom for consistent visual feedback
- **Smooth Performance**: Hardware-accelerated rendering at 60fps
- **Responsive**: Canvas resizes with browser window

## 🔧 Configuration

### Environment Variables
- `CI=true`: Enables CI-specific test configuration (retries, parallel execution)
- `RUST_LOG`: Set logging level for backend debugging

### Build Configuration
- **Frontend**: Configured via `Trunk.toml`
- **Backend**: Standard Cargo configuration
- **Testing**: Playwright config in `frontend/playwright.config.ts`

## 🚀 Deployment

### Production Build
```bash
# Build optimized WASM frontend
cd frontend && trunk build --release

# Build optimized backend
cd backend && cargo build --release
```

### Serving
- Frontend: Static files from `frontend/dist/`
- Backend: Run the compiled binary
- API: Ensure CORS is configured for your domain

## 🤝 Contributing

1. **Code Style**: Use `rustfmt` and `clippy` for Rust code
2. **Testing**: All changes must pass E2E tests
3. **Documentation**: Update README and specs for new features
4. **Commits**: Use conventional commit format

### Development Workflow
```bash
# 1. Create feature branch
git checkout -b feature/new-canvas-feature

# 2. Make changes with tests
# 3. Run tests
cd frontend && npm test

# 4. Build and verify
cargo build

# 5. Commit with conventional format
git commit -m "feat: add new canvas interaction"
```

## 📄 Project Structure

```
mori/
├── .github/
│   ├── copilot-instructions.md    # AI assistant guidelines
│   └── workflows/
│       └── ci.yml                 # GitHub Actions CI pipeline
├── backend/                       # Axum REST API server
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── frontend/                      # Yew WASM application
│   ├── Cargo.toml
│   ├── Trunk.toml
│   ├── package.json
│   ├── playwright.config.ts
│   ├── src/
│   │   ├── lib.rs                # Main Yew component
│   │   ├── api.rs                # API client
│   │   └── static/
│   ├── e2e/                      # E2E test specifications
│   └── index.html
├── shared/                        # Common types and models
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── specs/                         # Feature specifications
│   ├── infinite-canvas-spec.md
│   ├── sticky-notes-spec.md
│   └── workspace-specs.md
├── .git/hooks/
│   └── pre-commit               # Automated testing hook
├── test-all.sh                   # Comprehensive test runner
├── Cargo.toml                     # Workspace configuration
└── README.md
```

## 📋 Roadmap

- [ ] Sticky notes system
- [ ] Collaborative editing
- [ ] Export to image/PDF
- [ ] Dark mode theme
- [ ] Touch/mobile support
- [ ] Undo/redo functionality

## 📝 License

This project is open source. See LICENSE file for details.

## 🙏 Acknowledgments

Built with modern web technologies:
- **Yew**: Rust framework for WebAssembly
- **Axum**: Ergonomic web framework for Rust
- **Trunk**: WASM build and serve tool
- **Playwright**: Browser automation for testing

- Infinite canvas with grid
- Zoom in/out with mouse wheel or keyboard (+/-)
- Pan by dragging
- Debug overlay showing current zoom and pan values