# 5. GtkSourceView Code Blocks with Syntax Highlighting

* Status: Accepted
* Date: 2026-07-26

## Context and Problem Statement

Code blocks within notes were previously rendered as static `GtkLabel` widgets inside a styled container box. This limited functionality by preventing syntax highlighting, language tag persistence, and inline editability within code blocks.

## Decision Drivers

* Users need syntax highlighting for various programming and configuration languages.
* Users need to edit code blocks directly in the note editor.
* Users should be able to select and change the code block language via an inline UI dropdown.
* Code blocks must round-trip cleanly to standard Markdown (` ```lang `) with proper line spacing.
* The visual container styling (background, border radius, padding) must be preserved.

## Considered Options

1. Keep static `GtkLabel` widgets and apply custom Pango markup tags for syntax highlighting.
2. Use embedded `GtkSourceView` (via `sourceview5` GTK4 bindings) wrapped in a container box with a language selector dropdown.

## Decision Outcome

Chosen option: **Option 2 (GtkSourceView with language selector dropdown)**.

### Positives

* Native GNOME syntax highlighting via `GtkSourceView` supporting system-installed language definitions.
* Direct inline editing (`editable = true`) within code blocks in notes.
* Integrated language dropdown (`gtk4::DropDown` with search enabled) for selecting languages.
* Automatic Light/Dark system theme synchronization using Libadwaita `adw::StyleManager` (`Adwaita` and `Adwaita-dark` schemes).
* Standard Markdown round-trip serialization preserving language tags and enforcing 1 newline above and 1 newline below.

### Negatives

* Requires `gtksourceview-5` build and runtime system dependencies.
