# Code Review Backlog

## Overview
This backlog contains tasks identified during the comprehensive code review of the CoCoMiro infinite canvas application. Tasks are organized by severity level with actionable fixes.

**Review Date:** April 2026
**Total Issues:** 20
**Completed:** 7 issues ✅
- 🔴 Critical: 2 issues ✅ (security & compatibility)
- 🟠 High: 8 issues (memory leaks ✅, API design ✅)
- 🟡 Medium: 7 issues (2 ✅ performance, maintainability)
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

### 2. ✅ Replace Deprecated document.execCommand()
**Severity:** Critical
**Files:** `src/text_input.rs`
**Issue:** Uses deprecated `execCommand` API which may be removed from browsers.

**Subtasks:**
- [x] Replace `execCommand` with modern Selection/Range APIs for text formatting
- [x] Implement bold/italic/underline using `document.queryCommandState()` and `execCommand('bold')` alternatives
- [x] Handle `unwrap()` failures gracefully in JS reflection calls
- [x] Test formatting works across different browsers

## High Priority Tasks

### 3. ✅ Fix Memory Leaks in Text Input Overlays
**Severity:** High
**Files:** `src/text_input.rs`
**Issue:** Text input overlay event listeners use `closure.forget()` causing memory leaks when overlays are created and destroyed.

**Subtasks:**
- [x] **3.1.1 Add Missing Closure Fields to TextInputOverlayState**: Cancelled - alternative approach doesn't require state struct
- [x] **3.1.2 Modify setup_blur_event Signature**: Alternative implemented - replaced individual mousedown handlers with single document-level listener for better memory management  
- [x] **3.1.3 Update setup_blur_event Call Site**: Cancelled - no state parameter needed with document-level listener approach
- [x] **3.1.4 Verify Overlay Creation Still Works**: Verified - all E2E tests pass, overlays create and function correctly
- [x] **3.1.5 Verify Memory Leak Fix**: Verified - closures stored in thread-local and cleared on overlay destruction
- [x] **3.2 Verify Overlay Cleanup**: Verified - setting `text_input_overlay = None` properly triggers cleanup via blur handler
- [x] **3.3 Add Memory Leak Detection**: Implemented - thread-local storage with automatic cleanup on overlay destruction

### 4. Optimize Main Canvas Event Listeners
**Severity:** Medium
**Files:** `src/event_setup.rs`
**Issue:** Main canvas event listeners are set up once at app initialization and may benefit from proper storage for future cleanup.

**Subtasks:**
- [ ] **4.1 Analyze Main Canvas Listener Lifetime**: Document that main canvas listeners don't leak (set once, persist for app lifetime)
- [ ] **4.2 Consider Event Listener Storage**: If needed for future cleanup, add `EventListenerState` to `AppState` and store main canvas closures


### 5. Remove Duplicate #[cfg] Attributes
**Severity:** High
**Files:** `src/lib.rs`, `src/canvas.rs`
**Issue:** Duplicate `#[cfg(target_arch = "wasm32")]` attributes on functions.

**Subtasks:**
- [x] Remove duplicate `#[cfg(target_arch = "wasm32")]` from `start_impl()` in `src/lib.rs`
- [x] Remove duplicate `#[cfg(target_arch = "wasm32")]` from render functions in `src/canvas.rs`
- [x] Verify compilation still works for both targets

### 6. ✅ Fix AppResult Type Availability
**Severity:** High
**Files:** `src/error.rs`
**Issue:** `AppResult` type alias is only available on WASM target despite `AppError` being available everywhere.

**Subtasks:**
- [x] Remove `#[cfg(target_arch = "wasm32")]` gate from `AppResult` type alias
- [x] Verify host compilation still works

### 7. Refactor resize_to Function Signature
**Severity:** High
**Files:** `src/sticky_notes/state.rs`
**Issue:** `resize_to` function has 11 parameters (clippy limit is 7), with 3 unused parameters.

**Subtasks:**
- [ ] Remove unused `_viewport`, `_viewport_width`, `_viewport_height` parameters
- [ ] Consider bundling remaining parameters into a struct if still too many
- [ ] Update all call sites to match new signature

**Subtasks:**
- [ ] Remove unused `_viewport`, `_viewport_width`, `_viewport_height` parameters
- [ ] Consider bundling remaining parameters into a struct if still too many
- [ ] Update all call sites to match new signature

## Medium Priority Tasks

### 8. Fix Hardcoded Viewport Dimensions in Cursor Detection
**Severity:** Medium
**Files:** `src/styling.rs`
**Issue:** `update_canvas_cursor` uses hardcoded 800x600 instead of actual canvas dimensions.

**Subtasks:**
- [ ] Pass actual canvas width/height to `update_canvas_cursor` function
- [ ] Update function signature to accept viewport dimensions
- [ ] Test cursor behavior with different canvas sizes

### 9. Optimize Redundant world_point_at Calls
**Severity:** Medium
**Files:** `src/styling.rs`
### 8. Fix Hardcoded Viewport Dimensions in Cursor Detection
**Severity:** Medium
**Files:** `src/styling.rs`
**Issue:** `update_canvas_cursor` uses hardcoded 800x600 instead of actual canvas dimensions.

**Subtasks:**
- [ ] Pass actual canvas width/height to `update_canvas_cursor` function
- [ ] Update function signature to accept viewport dimensions
- [ ] Test cursor behavior with different canvas sizes

### 9. Optimize Redundant world_point_at Calls
**Severity:** Medium
**Files:** `src/styling.rs`
**Issue:** Same `world_point_at` call made twice to get `.0` and `.1` coordinates.

**Subtasks:**
- [ ] Destructure the tuple once and reuse the values
- [ ] Verify cursor detection still works correctly

### 10. Remove Commented-Out Code
**Severity:** Medium
**Files:** `src/lib.rs`, `src/canvas.rs`
**Issue:** Multiple instances of commented-out code adding noise to codebase.

**Subtasks:**
- [ ] Remove commented-out `hovered_resize_handle` field and related code
- [ ] Remove commented-out handle color logic in canvas rendering
- [ ] Ensure no functionality is lost

### 11. Add Recursion Depth Limit to HTML Parser
**Severity:** Medium
**Files:** `src/canvas.rs`
**Issue:** `parse_formatted_text` can recurse infinitely with deeply nested HTML.

**Subtasks:**
- [ ] Add max recursion depth parameter (e.g., 10 levels)
- [ ] Return error or flatten content when depth exceeded
- [ ] Add tests for deeply nested HTML parsing

### 12. Implement Actual Delay in Render Retry Loop
**Severity:** Medium
**Files:** `src/lib.rs`
**Issue:** Render retry loop has no actual delay between attempts.

**Subtasks:**
- [ ] Implement proper delay using `setTimeout` in WASM context
- [ ] Consider exponential backoff for retries
- [ ] Test retry behavior with artificial failures

### 13. ✅ Remove Unused regex Dependency
**Severity:** Medium
**Files:** `Cargo.toml`
**Issue:** `regex` crate is listed but never used in codebase.

**Subtasks:**
- [x] Search codebase for any regex usage (including comments)
- [x] Remove `regex` dependency from `Cargo.toml` if truly unused
- [x] Verify compilation and WASM build still work

### 14. ✅ Remove Inline Style Duplication in Color Picker
**Severity:** Medium
**Files:** `src/text_input.rs`, `styles.css`
**Issue:** Color picker functions set inline styles that duplicate CSS classes.

**Subtasks:**
- [x] Remove inline style setting in `create_color_picker_container` that duplicates `.color-picker` CSS
- [x] Remove inline style setting in `create_color_option` that duplicates `.color-picker-option` CSS
- [x] Test that styling still works correctly
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

### 12. ✅ Remove Unused regex Dependency
**Severity:** Medium
**Files:** `Cargo.toml`
**Issue:** `regex` crate is listed but never used in codebase.

**Subtasks:**
- [x] Search codebase for any regex usage (including comments)
- [x] Remove `regex` dependency from `Cargo.toml` if truly unused
- [x] Verify compilation and WASM build still work

### 13. ✅ Remove Inline Style Duplication in Color Picker
**Severity:** Medium
**Files:** `src/text_input.rs`, `styles.css`
**Issue:** Color picker functions set inline styles that duplicate CSS classes.

**Subtasks:**
- [x] Remove inline style setting in `create_color_picker_container` that duplicates `.color-picker` CSS
- [x] Remove inline style setting in `create_color_option` that duplicates `.color-picker-option` CSS
- [x] Test that styling still works correctly

## Low Priority Tasks

### 15. Fix ElementStyling Trait Gating
**Severity:** Low
**Files:** `src/styling.rs`
**Issue:** `ElementStyling` trait not gated with `#[cfg(target_arch = "wasm32")]` despite using WASM types.

**Subtasks:**
- [ ] Add `#[cfg(target_arch = "wasm32")]` to `ElementStyling` trait and impl
- [ ] Verify host compilation still works

### 16. Make Test IDs Deterministic
**Severity:** Low
**Files:** `src/sticky_notes/types.rs`
**Issue:** Global atomic ID counter makes test IDs non-deterministic.

**Subtasks:**
- [ ] Consider making ID generation deterministic for tests
- [ ] Or document that ID assertions should avoid exact matching
- [ ] Update test assertions if needed

### 17. Fix Enter Key Behavior in Text Input
**Severity:** Low
**Files:** `src/text_input.rs`
**Issue:** Enter key closes overlay instead of inserting newlines.

**Subtasks:**
- [ ] Change Enter key to insert newlines in contenteditable
- [ ] Use Shift+Enter or other key combination to confirm edits
- [ ] Update user documentation if needed

### 18. Add DOM Element Cleanup Error Handling
**Severity:** Low
**Files:** `src/text_input.rs`
**Issue:** Silent failure when removing DOM elements from overlays.

**Subtasks:**
- [ ] Handle `remove_child()` errors properly instead of ignoring with `let _ =`
- [ ] Log cleanup failures or implement retry logic
- [ ] Test overlay cleanup in error conditions

### 19. Fix Clippy Warnings in Tests
**Severity:** Low
**Files:** `tests/e2e_home.rs`, `tests/integration_tests.rs`, `tests/visual_regression.rs`
**Issue:** Various clippy warnings in test code.

**Subtasks:**
- [ ] Remove redundant `base64` import in visual regression tests
- [ ] Replace `map_or(false, ...)` with `is_some_and(...)`
- [ ] Fix needless borrows on string references
- [ ] Remove or implement unused method `assert_sticky_note_color_picker_behavior`
- [ ] Fix field assignments outside Default::default() initializers

### 20. Make Event Constants Available on All Targets
**Severity:** Low
**Files:** `src/event_constants.rs`
**Issue:** Constants only available on WASM target.

**Subtasks:**
- [ ] Remove `#[cfg(target_arch = "wasm32")]` from event constants
- [ ] Verify host compilation still works
- [ ] Consider if tests need access to these constants
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