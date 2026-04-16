# Code Review Backlog

## Overview
This backlog contains tasks identified during the comprehensive code review of the CoCoMiro infinite canvas application. Tasks are organized by severity level with actionable fixes.

**Review Date:** April 2026
**Total Issues:** 19
- 🔴 Critical: 2 issues (security & compatibility)
- 🟠 High: 4 issues (memory leaks, API design)
- 🟡 Medium: 7 issues (performance, maintainability)
- 🟢 Low: 6 issues (code quality, tests)

## Critical Priority Tasks

### 1. ✅ Fix XSS Vulnerability in Text Input
**Severity:** Critical
**Files:** `src/text_input.rs`, `src/app.rs`
**Issue:** User-controlled HTML content is injected unsafely via `set_inner_html()` without sanitization.

**Subtasks:**
- [x] Create HTML sanitization function that allows only safe tags (`<b>`, `<i>`, `<u>`, `<br>`, `<em>`, `<strong>`)
- [x] Sanitize note content before calling `set_inner_html()` in `create_contenteditable_element`
- [x] Update input handler to sanitize content before storing in note.content
- [x] Add tests for HTML sanitization edge cases

### 2. Replace Deprecated document.execCommand()
**Severity:** Critical
**Files:** `src/text_input.rs`
**Issue:** Uses deprecated `execCommand` API which may be removed from browsers.

**Subtasks:**
- [ ] Replace `execCommand` with modern Selection/Range APIs for text formatting
- [ ] Implement bold/italic/underline using `document.queryCommandState()` and `execCommand('bold')` alternatives
- [ ] Handle `unwrap()` failures gracefully in JS reflection calls
- [ ] Test formatting works across different browsers

## High Priority Tasks

### 3. Fix Memory Leaks in Event Listeners
**Severity:** High
**Files:** `src/event_setup.rs`, `src/text_input.rs`, `src/mouse_events.rs`
**Issue:** All event listeners use `closure.forget()` causing memory leaks, especially for text input overlays.

**Subtasks:**
- [ ] Store `Closure` references in overlay state instead of calling `forget()`
- [ ] Implement proper cleanup when text input overlays are removed
- [ ] Track and drop formatting button closures on overlay destruction
- [ ] Add memory leak tests or monitoring

### 4. Remove Duplicate #[cfg] Attributes
**Severity:** High
**Files:** `src/lib.rs`, `src/canvas.rs`
**Issue:** Duplicate `#[cfg(target_arch = "wasm32")]` attributes on functions.

**Subtasks:**
- [ ] Remove duplicate `#[cfg(target_arch = "wasm32")]` from `start_impl()` in `src/lib.rs`
- [ ] Remove duplicate `#[cfg(target_arch = "wasm32")]` from render functions in `src/canvas.rs`
- [ ] Verify compilation still works for both targets

### 5. Fix AppResult Type Availability
**Severity:** High
**Files:** `src/error.rs`
**Issue:** `AppResult` type alias is only available on WASM target despite `AppError` being available everywhere.

**Subtasks:**
- [ ] Remove `#[cfg(target_arch = "wasm32")]` gate from `AppResult` type alias
- [ ] Verify host compilation still works

### 6. Refactor resize_to Function Signature
**Severity:** High
**Files:** `src/sticky_notes/state.rs`
**Issue:** `resize_to` function has 11 parameters (clippy limit is 7), with 3 unused parameters.

**Subtasks:**
- [ ] Remove unused `_viewport`, `_viewport_width`, `_viewport_height` parameters
- [ ] Consider bundling remaining parameters into a struct if still too many
- [ ] Update all call sites to match new signature

## Medium Priority Tasks

### 7. Fix Hardcoded Viewport Dimensions in Cursor Detection
**Severity:** Medium
**Files:** `src/styling.rs`
**Issue:** `update_canvas_cursor` uses hardcoded 800x600 instead of actual canvas dimensions.

**Subtasks:**
- [ ] Pass actual canvas width/height to `update_canvas_cursor` function
- [ ] Update function signature to accept viewport dimensions
- [ ] Test cursor behavior with different canvas sizes

### 8. Optimize Redundant world_point_at Calls
**Severity:** Medium
**Files:** `src/styling.rs`
**Issue:** Same `world_point_at` call made twice to get `.0` and `.1` coordinates.

**Subtasks:**
- [ ] Destructure the tuple once and reuse the values
- [ ] Verify cursor detection still works correctly

### 9. Remove Commented-Out Code
**Severity:** Medium
**Files:** `src/lib.rs`, `src/canvas.rs`
**Issue:** Multiple instances of commented-out code adding noise to codebase.

**Subtasks:**
- [ ] Remove commented-out `hovered_resize_handle` field and related code
- [ ] Remove commented-out handle color logic in canvas rendering
- [ ] Ensure no functionality is lost

### 10. Add Recursion Depth Limit to HTML Parser
**Severity:** Medium
**Files:** `src/canvas.rs`
**Issue:** `parse_formatted_text` can recurse infinitely with deeply nested HTML.

**Subtasks:**
- [ ] Add max recursion depth parameter (e.g., 10 levels)
- [ ] Return error or flatten content when depth exceeded
- [ ] Add tests for deeply nested HTML parsing

### 11. Implement Actual Delay in Render Retry Loop
**Severity:** Medium
**Files:** `src/lib.rs`
**Issue:** Render retry loop has no actual delay between attempts.

**Subtasks:**
- [ ] Implement proper delay using `setTimeout` in WASM context
- [ ] Consider exponential backoff for retries
- [ ] Test retry behavior with artificial failures

### 12. Remove Unused regex Dependency
**Severity:** Medium
**Files:** `Cargo.toml`
**Issue:** `regex` crate is listed but never used in codebase.

**Subtasks:**
- [ ] Search codebase for any regex usage (including comments)
- [ ] Remove `regex` dependency from `Cargo.toml` if truly unused
- [ ] Verify compilation and WASM build still work

### 13. Remove Inline Style Duplication in Color Picker
**Severity:** Medium
**Files:** `src/text_input.rs`
**Issue:** Color picker functions set inline styles that duplicate CSS classes.

**Subtasks:**
- [ ] Remove inline style setting in `create_color_picker_container` that duplicates `.color-picker` CSS
- [ ] Remove inline style setting in `create_color_option` that duplicates `.color-picker-option` CSS
- [ ] Test that styling still works correctly

## Low Priority Tasks

### 14. Fix ElementStyling Trait Gating
**Severity:** Low
**Files:** `src/styling.rs`
**Issue:** `ElementStyling` trait not gated with `#[cfg(target_arch = "wasm32")]` despite using WASM types.

**Subtasks:**
- [ ] Add `#[cfg(target_arch = "wasm32")]` to `ElementStyling` trait and impl
- [ ] Verify host compilation still works

### 15. Make Test IDs Deterministic
**Severity:** Low
**Files:** `src/sticky_notes/types.rs`
**Issue:** Global atomic ID counter makes test IDs non-deterministic.

**Subtasks:**
- [ ] Consider making ID generation deterministic for tests
- [ ] Or document that ID assertions should avoid exact matching
- [ ] Update test assertions if needed

### 16. Fix Enter Key Behavior in Text Input
**Severity:** Low
**Files:** `src/text_input.rs`
**Issue:** Enter key closes overlay instead of inserting newlines.

**Subtasks:**
- [ ] Change Enter key to insert newlines in contenteditable
- [ ] Use Shift+Enter or other key combination to confirm edits
- [ ] Update user documentation if needed

### 17. Add DOM Element Cleanup Error Handling
**Severity:** Low
**Files:** `src/text_input.rs`
**Issue:** Silent failure when removing DOM elements from overlays.

**Subtasks:**
- [ ] Handle `remove_child()` errors properly instead of ignoring with `let _ =`
- [ ] Log cleanup failures or implement retry logic
- [ ] Test overlay cleanup in error conditions

### 18. Fix Clippy Warnings in Tests
**Severity:** Low
**Files:** `tests/e2e_home.rs`, `tests/integration_tests.rs`, `tests/visual_regression.rs`
**Issue:** Various clippy warnings in test code.

**Subtasks:**
- [ ] Remove redundant `base64` import in visual regression tests
- [ ] Replace `map_or(false, ...)` with `is_some_and(...)`
- [ ] Fix needless borrows on string references
- [ ] Remove or implement unused method `assert_sticky_note_color_picker_behavior`
- [ ] Fix field assignments outside Default::default() initializers

### 19. Make Event Constants Available on All Targets
**Severity:** Low
**Files:** `src/event_constants.rs`
**Issue:** Constants only available on WASM target.

**Subtasks:**
- [ ] Remove `#[cfg(target_arch = "wasm32")]` from event constants
- [ ] Verify host compilation still works
- [ ] Consider if tests need access to these constants

## Implementation Notes

**Testing Strategy:**
- All fixes should maintain existing test suite (28 tests currently passing)
- Add new tests for security fixes (XSS prevention, HTML sanitization)
- Test WASM compilation after each change
- Run clippy and fix any new warnings introduced

**Priority Guidelines:**
- 🔴 Critical: Fix immediately (security/deprecation issues)
- 🟠 High: Fix in next sprint (memory leaks, API design)
- 🟡 Medium: Fix when convenient (performance, maintainability)
- 🟢 Low: Fix during code cleanup sessions (code quality)

**Risk Assessment:**
- Critical fixes have highest risk but also highest benefit
- Memory leak fixes may require significant refactoring of event system
- HTML sanitization changes need careful testing to avoid breaking existing functionality</content>
<parameter name="filePath">/Users/basile.du.plessis/Documents/cocomiro/code-review-backlog.md