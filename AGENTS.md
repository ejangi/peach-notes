# AGENTS.md

## Overview & Architecture

**Peach Notes** is a native GNOME desktop notes application built with **Rust**, **GTK4** (`gtk4-rs`), and **Libadwaita** (`libadwaita-rs`). It features an Apple Notes-inspired user interface backed by local Markdown (`.md`) files stored in a user-selected directory on disk.

### Key Architecture Components

1. **User Interface (`ui/`)**:
   - Uses `adw::NavigationSplitView` for a dual-pane layout: Note List on the left (with search filtering) and Note Editor pane on the right.
   - `adw::HeaderBar` with Search, New Note, Formatting Toolbar toggle, and Preferences button.
   - `gtk::TextView` with custom `GtkTextBuffer` tags for WYSIWYG rich text rendering.
   - `AdwPreferencesWindow` for configuring the Notes Directory path.

2. **Markdown Parser & Serializer (`markdown/`)**:
   - Uses `pulldown-cmark` to parse Markdown files into structured events and populate `GtkTextBuffer` with styling tags (`heading-1`, `heading-2`, `bold`, `italic`, `monospace`, `bullet-list`, `strikethrough`).
   - Serializes `GtkTextBuffer` contents and active tags back into standard Markdown string representation on disk.

3. **Storage & Disk Sync (`storage/`)**:
   - Manages reading/writing `.md` files in the configured Notes Directory (defaults to `~/Documents/Notes`).
   - Derives note titles from the first H1 header in content (`# Title`), syncing file renames automatically on disk.
   - When an asset (image/file) is attached or dropped into a note, stores it inside `<note title>.assets/` directory alongside the note file and references it via relative path.
   - Filters out `.assets` folders and non-`.md` files from the sidebar note list.
   - Monitors the Notes Directory using `notify` / GLib timeouts for real-time external file change detection.

4. **Preferences (`config/`)**:
   - Persists app settings (Notes Directory path, window state) in `~/.config/peach-notes/config.json`.

## Build & Test Commands

- **Build**: `cargo build`
- **Run**: `cargo run`
- **Check / Lint**: `cargo check`, `cargo clippy`
- **Format**: `cargo fmt`
- **Test**: `cargo test`

## Code Guidelines & Conventions

- Strictly follow idiomatic Rust and GNOME GTK4 / Libadwaita best practices.
- Keep UI thread non-blocking; use GLib channels / async channels for filesystem tasks.
- Respect domain terminology defined in `CONTEXT.md`.
- Maintain clean separation between Markdown buffer parsing/serialization logic and GTK widget code.
