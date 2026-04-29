---
name: rust-best-practices
user-invocable: true
description: >
  Enforce Rust best practices, formatting, and unit testing before marking a task as done or committing code.
---

# Rust Best Practices Skill

## Purpose
This skill ensures that all Rust code changes follow best practices, are properly formatted, and are covered by unit tests before tasks are considered complete or code is committed.

## Workflow Steps
1. **Code Review**
   - Review the code for idiomatic Rust style, clear naming, and maintainable structure.
   - Refactor for readability and simplicity where possible.
2. **Formatting**
   - Run `cargo fmt` to auto-format the codebase.
   - Ensure no formatting errors remain.
3. **Compilation**
   - Run `cargo check` for host compilation.
   - Run `cargo check --target wasm32-unknown-unknown` for WASM compatibility.
   - Fix all compilation errors before proceeding.
4. **Testing**
   - Run `cargo test` to execute all unit and integration tests.
   - Add or update tests for any new or changed logic.
   - Ensure all tests pass.
5. **WASM Build Validation**
   - Run `trunk build --release` to verify the WASM build.
   - Fix any build errors.
6. **Completion Criteria**
   - All steps above must succeed with no errors or test failures.
   - Code must be idiomatic, formatted, tested, and WASM-compatible.

## Quality Criteria
- No compilation or formatting errors
- All tests pass
- WASM build succeeds
- Code is idiomatic and maintainable
- New/changed logic is covered by tests

## Example Prompts
- "Check if my Rust code follows best practices before commit."
- "Validate formatting, compilation, and tests for this Rust change."
- "Ensure WASM compatibility and test coverage for my update."

## Related Customizations
- Pre-commit hooks for `cargo fmt` and `cargo test`
- E2E test skills for browser workflows
- WASM-specific validation skills
