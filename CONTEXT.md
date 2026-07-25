# Context Glossary: Peach Notes

This document defines the ubiquitous domain language for Peach Notes. It contains business/domain terminology only and is free of temporary implementation details.

## Domain Terms

### Note
A single text document created by the user containing formatted content and tracking a last-modified timestamp. In storage, each Note maps to a single Markdown file with a `.md` extension.

### Note Title
The primary heading or display name of a Note. The title is defined by the first Level 1 Heading (`# Title`) within the note's content. When updated, the Note's associated file on disk is renamed accordingly.

### Notes Directory
The designated folder on the local file system where all `.md` Note files are stored, read, and watched.

### Rich Text Buffer
The active in-memory representation of a Note's content while being edited. Formatting syntax marks (such as `#`, `**`, `*`, `` ` ``) are represented visually as styled text ranges (headings, bold, italic, code, lists) rather than raw text symbols.

### Formatting Tag
A semantic styling marker (e.g., Heading 1, Heading 2, Bold, Italic, Monospace Code, Bullet List, Strikethrough) applied to ranges of text within a Note's Rich Text Buffer.

### Note Serializer
The process that converts the text ranges and applied Formatting Tags of a Rich Text Buffer into valid Markdown content for disk storage.

### Note Parser
The process that parses Markdown content read from disk into formatted text ranges and Formatting Tags in the Rich Text Buffer.

### File Watcher
The background monitoring service that detects external additions, deletions, or external edits of `.md` files within the Notes Directory and updates the Note list accordingly.

### Note Assets Directory
A dedicated subdirectory named `<note title>.assets` stored directly alongside its corresponding `.md` Note file. Created automatically on demand when images or file attachments are inserted into a Note. It stores image files and file attachments referenced locally by the Note and is hidden from the main Note list UI.

### File Attachment
Any non-image file asset (e.g. document, archive, media) dropped into or linked within a Note, stored inside the Note Assets Directory and rendered as an interactive attachment card.

