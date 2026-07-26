# 4. GFM Task List Items Specification

- **Status**: Accepted
- **Date**: 2026-07-26
- **Context**: Implementation of GFM Section 5.3 (Task List Items) in Peach Notes native GTK4 / Libadwaita Markdown editor.

## Context & Problem Statement
Peach Notes requires interactive Task List Items (`- [ ] ` / `- [x] `) within the WYSIWYG GTK4 `TextView` editor according to GitHub Flavored Markdown Specification Section 5.3.

## Decision Drivers
- Seamless native GNOME UI/UX matching GTK4 / Libadwaita aesthetic.
- Interactive checkboxes in the note editor buffer that toggle Markdown source state on disk (`[ ]` $\leftrightarrow$ `[x]`).
- Fluid keyboard input (<kbd>Enter</kbd> key task item continuation and auto-exit).

## Agreed Architecture & Design

### 1. Interactive GTK4 Checkbox Widget Anchor
- Task list item markers (`- [ ] `, `* [ ] `, `+ [ ] `, `- [x] `, `* [x] `, `+ [x] `) are rendered using native `GtkCheckButton` widgets embedded at `GtkTextChildAnchor` locations inside `GtkTextBuffer`.
- Each `GtkCheckButton` is given widget metadata (`TASK|[ ]` or `TASK|[x]`).
- Toggling the checkbox in the UI instantly updates the underlying `GtkTextBuffer` payload, applies/removes dimmed text styling on the item line, and emits the buffer changed signal to trigger autosave.

### 2. Completed Item Styling
- When checked (`- [x] `), the text content of the task line receives dimmed opacity styling (`alpha(@window_fg_color, 0.55)`).
- No strikethrough is applied (per design decision).
- When unchecked (`- [ ] `), normal text color and full opacity are restored.

### 3. Keyboard Editing & Continuation
- **Typing Syntax**: Typing `- [ ] `, `* [ ] `, or `+ [ ] ` followed by a space automatically converts the line into a task list item anchor.
- **Return Key on Non-Empty Task Line**: Pressing <kbd>Return</kbd> creates a new unchecked task list item (`- [ ] `) on the next line with matching indentation.
- **Return Key on Empty Task Line**: Pressing <kbd>Return</kbd> on an empty task item (`- [ ] ` with no text) clears the marker and exits list editing back to standard paragraph entry.

### 4. Selection Toolbar Button
- A `☑` button is added to the floating selection formatting popover.
- Clicking `☑` toggles selected lines into task list items.

## Roundtrip Markdown Serialization
- `parse_markdown_to_buffer()` identifies GFM task list item events (`pulldown_cmark::TaskItemMarker`) and inserts `GtkCheckButton` child anchors.
- `serialize_buffer_to_markdown()` converts `TASK|[ ]` and `TASK|[x]` child anchors back to canonical GFM pipe markdown (`- [ ] ` and `- [x] `).
