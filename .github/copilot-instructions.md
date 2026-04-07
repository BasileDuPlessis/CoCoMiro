# CoCoMiro Copilot Instructions

## Project context
- `CoCoMiro` is a small **Rust + WebAssembly infinite canvas** project.
- Main application logic lives in `src/lib.rs`.
- The app is served locally with `trunk serve`.
- Browser E2E coverage lives in `tests/e2e_home.rs` and uses `headless_chrome`.

## Working style
- Prefer **short, focused tasks** and small diffs.
- Keep implementations simple, explicit, and easy to read.
- Refactor continuously to improve **readability**, naming, and structure.
- Split long functions into small helpers when it makes intent clearer.
- Avoid unnecessary abstraction, duplication, and overly clever code.

## Code quality expectations
- Write idiomatic Rust.
- Preserve existing behavior unless the task explicitly requires a change.
- Favor clear names, predictable control flow, and maintainable logic.
- Leave the codebase cleaner than you found it when possible.

## Testing requirements
- **Tests are required** for bug fixes and behavior changes.
- Add or update tests when changing logic.
- Do not consider work complete without verification.
- Before finishing a meaningful change, run the relevant checks such as:
  - `cargo fmt`
  - `cargo test`
  - `cargo test -- --include-ignored` for full coverage when relevant

## App-specific guidance
- Keep canvas interactions stable and deterministic.
- Be careful with browser timing and E2E reliability.
- Prefer robust readiness checks over fixed sleeps in browser tests.
