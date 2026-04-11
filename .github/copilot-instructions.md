# CoCoMiro Copilot Instructions

## 🚨 CRITICAL VALIDATION PROTOCOL
**STOP AND VALIDATE: Never assume code works - always test compilation**

### Before suggesting ANY code changes:
1. **Check current compilation status** with `cargo check --target wasm32-unknown-unknown`
2. **Run existing tests** with `cargo test`
3. **Verify WASM builds** with `trunk build --release`

### After ANY code suggestion:
1. **Immediately test compilation** - Don't wait for user feedback
2. **Report actual results** - Include exact error messages if any
3. **Fix issues before proceeding** - Don't suggest more code until current code compiles
4. **Use tools proactively** - Run `cargo check` and `get_errors` after changes

### Error Reporting Protocol:
- **Include full error output** - Don't summarize, show exact compiler messages
- **Stop on first error** - Fix compilation errors before logic errors
- **Ask for clarification** - If unsure about error meaning, ask user
- **Test incrementally** - Small changes, frequent validation

## Project context
- `CoCoMiro` is a small **Rust + WebAssembly infinite canvas** project.
- Main application logic lives in `src/lib.rs`.
- The app is served locally with `trunk serve --address 127.0.0.1 --port 8080`.
- Browser E2E coverage lives in `tests/e2e_home.rs` and uses `headless_chrome`.
- **CRITICAL**: Code must compile for both host and WebAssembly targets
- When the app must run in the browser, verify the wasm build with `cargo check --target wasm32-unknown-unknown` if needed.

## Working style
- Prefer **short, focused tasks** and small diffs.
- Keep implementations simple, explicit, and easy to read.
- Refactor continuously to improve **readability**, naming, and structure.
- Split long functions into small helpers when it makes intent clearer.
- Avoid unnecessary abstraction, duplication, and overly clever code.
- **CRITICAL**: Always verify changes work for both host and WebAssembly targets

## Code quality expectations
- Write idiomatic Rust.
- Preserve existing behavior unless the task explicitly requires a change.
- Favor clear names, predictable control flow, and maintainable logic.
- Leave the codebase cleaner than you found it when possible.
- **Ensure code compiles for both host and WebAssembly targets**
- **Use correct comment types**: `///` for public API documentation, `//` for implementation comments

## Testing requirements
- **Tests are required** for bug fixes and behavior changes.
- Add or update tests when changing logic.
- Do not consider work complete without verification.
- **MANDATORY VALIDATION WORKFLOW**:
  - ✅ `cargo check` (host compilation)
  - ✅ `cargo test` (unit tests)
  - ✅ `cargo check --target wasm32-unknown-unknown` (WASM compilation)
  - ✅ `trunk build --release` (full WASM build)
  - ✅ `cargo fmt` (code formatting)
- **STOP if any step fails** - Fix compilation errors immediately
- Before finishing a meaningful change, run the relevant checks such as:
  - `cargo fmt`
  - `cargo test`
  - `cargo test -- --include-ignored` for full coverage when relevant
  - `cargo check --target wasm32-unknown-unknown` (WebAssembly compilation)

## WebAssembly-Specific Considerations
- Code must compile for both host and `wasm32-unknown-unknown` targets
- Use `#[cfg(target_arch = "wasm32")]` for browser-specific code
- Ensure all WASM imports are properly gated
- Test with `trunk serve` to catch runtime issues
- Host compilation success ≠ WebAssembly compilation success
- Keep canvas interactions stable and deterministic.
- Be careful with browser timing and E2E reliability.
- Prefer robust readiness checks over fixed sleeps in browser tests.
