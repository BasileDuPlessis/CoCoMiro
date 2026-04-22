# Paste Newline Issue Fix - Implementation Tasks

## Overview
This document outlines the tasks to fix the issue where copying-pasting text with newlines (e.g., "es pratiques\n\nChantiers transverses") into the contenteditable results in unwanted `<div>` elements instead of proper line breaks.

## Root Cause
When pasting plain text containing `\n` characters, the current implementation uses `document.execCommand("insertText")`, but the `contenteditable` div automatically converts `\n` to HTML `<div>` elements for paragraph structure. This creates `"es pratiques<div></div><div>Chantiers transverses"` instead of preserving line breaks.

## Proposed Solution
Modify the paste handler to treat pasted plain text as HTML with `<br>` tags for line breaks, using `execCommand("insertHTML")` instead of `insertText`.

## Implementation Tasks

### **Task 1: Modify Paste Text Insertion to Handle Newlines Properly**
**Goal**: Ensure pasted text with newlines displays correctly in contenteditable without creating unwanted `<div>` elements.

**Priority**: High

**Implementation Steps**:
- [ ] Update `insert_sanitized_text()` in `src/text_input.rs` to replace `\n` with `<br>` in the pasted text
- [ ] Change from `execCommand("insertText")` to `execCommand("insertHTML")` to properly insert HTML line breaks
- [ ] Add basic HTML escaping for the text content to prevent injection issues
- [ ] Test that single and multiple newlines are preserved as `<br>` tags in the contenteditable

**Expected Code Changes**:
```rust
// In insert_sanitized_text function
let html_text = text
    .replace("&", "&amp;")
    .replace("<", "&lt;")
    .replace(">", "&gt;")
    .replace("\n", "<br>");

let _ = exec_command_fn.call3(
    document.as_ref(),
    &wasm_bindgen::JsValue::from_str("insertHTML"),
    &wasm_bindgen::JsValue::from_bool(false),
    &wasm_bindgen::JsValue::from_str(&html_text),
);
```

### **Task 2: Add E2E Test for Paste with Newlines**
**Goal**: Verify that pasting text with newlines works correctly and doesn't create HTML divs.

**Priority**: High

**Implementation Steps**:
- [ ] Extend the existing paste sanitization test in `tests/e2e_sticky_notes.rs`
- [ ] Add a test case that pastes plain text with newlines (e.g., "line1\n\nline3")
- [ ] Verify the contenteditable contains `<br>` tags instead of `<div>` elements
- [ ] Ensure the pasted content integrates properly with existing formatting

**Expected Test Addition**:
```rust
// In assert_paste_sanitization_works
let newline_paste_script = r#"
    // Paste text with newlines
    const pasteEvent = new ClipboardEvent('paste', {
        bubbles: true,
        cancelable: true,
        clipboardData: new DataTransfer()
    });
    pasteEvent.clipboardData.setData('text/plain', 'line1\n\nline3');
    editable.dispatchEvent(pasteEvent);
    return editable.innerHTML;
"#;
```

### **Task 3: Update Content Parsing to Handle Pasted HTML**
**Goal**: Ensure that when content is saved, pasted `<br>` tags are properly converted back to `\n` in the stored text.

**Priority**: Medium

**Implementation Steps**:
- [ ] Review `parse_html_to_text_and_formatting()` in `src/canvas.rs` to ensure it converts `<br>` back to `\n`
- [ ] Test that round-trip conversion (paste → save → display) preserves newlines correctly
- [ ] Verify that mixed content (pasted text + existing formatting) works properly

### **Task 4: Validate Fix with Manual Testing**
**Goal**: Confirm the fix works in real browser scenarios.

**Priority**: Medium

**Implementation Steps**:
- [ ] Test copying from various text sources (plain text editors, web pages, etc.)
- [ ] Verify that single `\n`, double `\n\n`, and mixed content paste correctly
- [ ] Check that the fix doesn't break existing HTML paste sanitization
- [ ] Test edge cases like very long text with many newlines

## Implementation Roadmap

### **Week 1: Core Fix**
- [ ] Task 1: Modify paste insertion logic
- [ ] Task 2: Add E2E test coverage

### **Week 2: Validation & Polish**
- [ ] Task 3: Ensure proper content parsing
- [ ] Task 4: Comprehensive manual testing

## Success Criteria

- ✅ Pasting "text\n\nmore text" results in `innerHTML` containing `<br>` instead of `<div>`
- ✅ E2E tests pass for both HTML sanitization and newline handling
- ✅ Content saves and loads correctly with preserved line breaks
- ✅ No regression in existing paste functionality
- ✅ Manual testing confirms fix works across different text sources

## Testing Checklist

- [ ] Plain text with single newlines
- [ ] Plain text with multiple consecutive newlines
- [ ] Mixed content (text + existing formatting)
- [ ] HTML content sanitization (existing functionality)
- [ ] Long text with many newlines
- [ ] Copy from different applications (text editors, browsers, etc.)
- [ ] Round-trip save/load verification

## Risk Assessment

**Low Risk**: The change only affects paste behavior and uses existing sanitization logic.

**Mitigation**: 
- Thorough E2E testing before deployment
- Fallback to existing behavior if issues detected
- Gradual rollout with monitoring

## Dependencies

- Requires access to `src/text_input.rs` and `src/canvas.rs`
- E2E test framework must be functional
- Browser testing environment available