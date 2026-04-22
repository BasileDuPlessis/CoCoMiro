# Text Workflow Analysis: Copy-Paste to Sticky ContentEditable

## Overview

This document analyzes the text handling workflow in CoCoMiro, from user copy-paste operations through text editing in contenteditable elements to final storage in sticky notes. It covers how text formatting is managed initially, how line breaks are preserved, and proposes improvements for better text handling.

## Current Implementation

### Text Storage Architecture

CoCoMiro uses a dual-storage approach for text content:

- **Plain Text**: Stored in `note.content` (String)
- **Formatting Spans**: Stored in `note.formatting` (Vec<TextFormat>)

```rust
pub struct TextFormat {
    pub start: usize,  // Character position (inclusive)
    pub end: usize,    // Character position (exclusive)
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}
```

### Copy-Paste Workflow

1. **Paste Event Interception**: The `setup_paste_event` function in `text_input.rs` intercepts paste events
2. **Content Sanitization**: Extracts text content from clipboard data, stripping HTML formatting
3. **Text Insertion**: Uses `document.execCommand("insertText")` to insert sanitized plain text

```rust
fn setup_paste_event() -> wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)> {
    // Extracts text/plain or sanitizes text/html to plain text
    let pasted_text = extract_clipboard_text(&clipboard_data)?;
    insert_sanitized_text(&document, &pasted_text);
}
```

### Text Formatting Management

#### Initial Formatting State
- New notes start with plain text "New note" and empty formatting vector
- No initial formatting is applied

#### Formatting Application
- Uses modern Selection/Range APIs instead of deprecated `execCommand`
- Formatting buttons wrap selected text in HTML tags: `<b>`, `<i>`, `<u>`

```rust
fn apply_formatting(contenteditable: &web_sys::HtmlElement, format_type: &str) {
    // Get current selection and apply formatting to ranges
    for range in selection_ranges {
        apply_formatting_to_range(window, &range, format_type)?;
    }
}
```

#### HTML Generation and Parsing
- **To HTML**: `format_text_with_spans_to_html()` converts plain text + spans to HTML
- **From HTML**: `parse_html_to_text_and_formatting()` parses HTML back to plain text + spans

### Line Break Preservation

#### Input Handling
- Contenteditable naturally preserves line breaks as `\n` characters
- Multi-line pasted content maintains line breaks through sanitization

#### HTML Conversion
- Line breaks (`\n`) are converted to `<br>` tags for HTML display
- Reverse conversion happens during parsing

```rust
// In format_text_with_spans_to_html
let formatted_text = span_text.replace("\n", "<br>");

// In parse_formatted_text
if tag_type == "br" {
    segments.push(TextSegment {
        text: "\n".to_string(),
        bold: false, italic: false, underline: false,
    });
}
```

#### Canvas Rendering
- `render_note_text_content()` processes text line-by-line
- Splits on `\n` characters to create separate lines
- Applies text wrapping within each line

## Issues Identified

### 1. Rich Text Loss on Paste
**Problem**: All HTML formatting is stripped during paste operations
**Impact**: Users lose formatting when copying from rich text sources
**Current Code**:
```rust
// Strips all HTML, keeps only plain text
let pasted_text = sanitize_html_to_text(&html_str);
```

### 2. Formatting Inconsistency
**Problem**: Round-trip formatting (HTML ↔ plain text + spans) may lose precision
**Impact**: Formatting spans may not exactly match original selections
**Example**: Adjacent formatting spans with identical properties should merge

### 3. Line Break Handling Edge Cases
**Problem**: Mixed line break sources (pasted `\r\n`, typed `\n`, `<br>`) not normalized
**Impact**: Inconsistent line break rendering across different input methods

### 4. Backward Compatibility Burden
**Problem**: Legacy notes with HTML content require special handling
**Impact**: Code complexity and potential parsing errors

### 5. Performance Concerns
**Problem**: HTML parsing/generation on every keystroke during editing
**Impact**: Potential lag during text input in large notes

## Proposed Improvements

### 1. Enhanced Paste Handling

**Allow Selective HTML Preservation**:
```rust
fn sanitize_html_for_paste(html: &str) -> String {
    // Allow only safe formatting tags: <b>, <i>, <u>, <br>
    // Strip scripts, styles, and other potentially dangerous elements
    // Preserve line breaks and basic formatting
}
```

**Smart Paste Detection**:
- Detect if pasted content is from CoCoMiro (preserve formatting spans)
- Detect if content is from external rich text sources (preserve safe HTML)
- Fallback to plain text for unknown sources

### 2. Improved Formatting System

**Formatting Span Optimization**:
```rust
impl TextFormat {
    fn merge_adjacent(&mut self, other: &TextFormat) -> bool {
        // Merge if adjacent and identical formatting
        if self.end == other.start && self.formatting_matches(other) {
            self.end = other.end;
            return true;
        }
        false
    }
}
```

**Real-time Formatting Updates**:
- Update formatting spans incrementally during typing
- Avoid full HTML re-parsing on every keystroke
- Use DOM mutation observers for efficient change detection

### 3. Robust Line Break Management

**Universal Line Break Normalization**:
```rust
fn normalize_line_breaks(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}
```

**Line Break Type Preservation**:
- Track whether line breaks came from user input vs. word wrapping
- Distinguish between paragraph breaks and line continuations

### 4. Enhanced ContentEditable Integration

**ContentEditable State Synchronization**:
```rust
struct ContentEditableState {
    plain_text: String,
    formatting_spans: Vec<TextFormat>,
    selection: Option<(usize, usize)>, // Character positions
}

impl ContentEditableState {
    fn from_html(html: &str) -> Self { /* ... */ }
    fn to_html(&self) -> String { /* ... */ }
}
```

**Bidirectional Sync**:
- Sync contenteditable changes to note state in real-time
- Preserve cursor position during formatting operations
- Handle undo/redo operations properly

### 5. Performance Optimizations

**Lazy HTML Generation**:
- Generate HTML only when needed for display
- Cache formatted HTML and invalidate on changes
- Use virtual DOM diffing for efficient updates

**Incremental Parsing**:
```rust
fn update_formatting_spans(
    current_spans: &mut Vec<TextFormat>,
    change_start: usize,
    change_end: usize,
    new_formatting: Option<TextFormat>
) {
    // Update only affected spans instead of re-parsing everything
}
```

### 6. Accessibility Improvements

**Screen Reader Support**:
- Ensure formatting is properly announced
- Provide keyboard shortcuts for formatting
- Maintain focus management during text operations

**Keyboard Navigation**:
- Arrow key navigation respects line breaks
- Tab insertion (4 spaces) for code-like content
- Proper handling of multi-line selections

## Implementation Roadmap

### Phase 1: Core Fixes (High Priority)
1. Fix paste handling to preserve safe HTML formatting
2. Implement line break normalization
3. Add formatting span merging and optimization

### Phase 2: Performance (Medium Priority)
1. Implement incremental formatting updates
2. Add HTML caching and lazy generation
3. Optimize canvas text rendering for large notes

### Phase 3: Advanced Features (Low Priority)
1. Rich paste detection and handling
2. Collaborative editing support
3. Advanced formatting (colors, fonts, lists)

## Testing Strategy

### Unit Tests
- Test HTML parsing and generation round-trips
- Test formatting span merging logic
- Test line break normalization

### Integration Tests
- Test copy-paste from various sources (Word, Google Docs, etc.)
- Test formatting preservation during editing
- Test line break handling in different scenarios

### E2E Tests
- Visual regression tests for text rendering
- Cross-browser compatibility testing
- Performance benchmarks for large notes

## Migration Strategy

### Backward Compatibility
- Maintain support for existing HTML content in notes
- Provide migration path for old format notes
- Ensure no data loss during upgrades

### Gradual Rollout
- Feature flags for new text handling features
- A/B testing for performance improvements
- Fallback mechanisms for error cases

## Success Metrics

- **Formatting Preservation**: >95% of rich text formatting preserved during copy-paste
- **Performance**: <50ms latency for text input operations
- **Compatibility**: Support for all major browsers and rich text sources
- **User Experience**: Seamless text editing experience matching modern editors</content>
<parameter name="filePath">/Users/basile.du.plessis/Documents/cocomiro/text-workflow-analysis.md