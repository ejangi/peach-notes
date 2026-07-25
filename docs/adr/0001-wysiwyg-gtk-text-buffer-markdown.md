# 1. WYSIWYG GtkTextBuffer Serialization for Markdown Notes

- Status: Accepted
- Date: 2026-07-25

## Context & Problem Statement

Peach Notes requires a native GNOME text editor experience with Apple Notes aesthetics while using local Markdown (`.md`) files as the storage mechanism. We needed to choose how Markdown content is rendered and edited in `gtk::TextView`.

## Decision Drivers

- Native GNOME look and feel with rich formatting (headings, bold, italic, lists, code blocks).
- Clean visual display without cluttered raw syntax symbols (`**`, `#`, `` ` ``) during editing.
- Standard Markdown files stored on disk for interoperability with external tools.

## Considered Options

1. **Full WYSIWYG Rich Text Buffer** (Selected): Strip raw syntax tokens on load, apply `GtkTextTag` styles to buffer text ranges, and serialize tags back to Markdown syntax on save.
2. **Inline Syntax Highlighting**: Keep raw Markdown tokens in buffer text and colorize them with tags.
3. **Live Preview Hiding**: Keep raw tokens in buffer text, but hide them visually with invisible tags except on the active cursor line.

## Decision Outcome

Chosen Option: **1. Full WYSIWYG Rich Text Buffer**.

### Consequences

- **Positive**: Clean, modern Apple Notes aesthetic without visible syntax marks.
- **Positive**: Direct rich text experience for non-technical users while retaining `.md` file storage.
- **Negative / Trade-off**: Requires precise tag serialization logic to accurately reconstruct Markdown syntax from `GtkTextBuffer` tags without losing structure.
