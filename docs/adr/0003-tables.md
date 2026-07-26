# 3. GFM Table Rendering and Markdown Module Refactoring

* Status: Accepted
* Date: 2026-07-26

## Context

Peach Notes relies on `pulldown-cmark` for parsing Markdown files into GTK text buffers and `GtkTextChildAnchor` widgets. As the application grows to support the full GitHub Flavored Markdown (GFM) specification (Section 4.10 Tables extension), the single monolithic `src/markdown/parser.rs` file becomes difficult to maintain.

Furthermore, GFM tables require rich visual representation (headers, alignments, cell borders) and interactive editing capabilities (direct cell text entry, inserting/deleting rows and columns mid-table) while preserving 100% roundtrip fidelity when saving back to standard `.md` pipe tables on disk.

## Decision

1. **Refactor `src/markdown/` into Modular Renderers**:
   - `src/markdown/parser.rs`: AST event loop orchestrator.
   - `src/markdown/serializer.rs`: Serializes text buffer iterators and table anchors back to GFM Markdown strings.
   - `src/markdown/renderers/mod.rs`: Dispatcher for element renderers.
   - `src/markdown/renderers/table.rs`: GFM Table AST parsing, `GtkGrid` widget builder, interactive cell entry bindings, mid-table row/column management controls.
   - `src/markdown/renderers/image.rs`: Image anchor rendering & dynamic width aspect ratio scaling.
   - `src/markdown/renderers/attachment.rs`: File attachment card widgets.
   - `src/markdown/renderers/inline.rs`: Formatting tags (headings, bold, italic, strikethrough, monospace, links, bullet lists).

2. **GFM Table Widget Architecture**:
   - **Anchor Embedding**: Tables are rendered as `GtkGrid` widgets embedded into `GtkTextBuffer` via `GtkTextChildAnchor`.
   - **Cell Editing**: Each cell contains a `GtkEntry` bound to the cell data so users can click and edit text directly.
   - **Alignment**: Column alignments (`left`, `center`, `right`) defined in GFM delimiter rows (`| :--- | :---: | ---: |`) map directly to GTK alignment (`Align::Start`, `Align::Center`, `Align::End`).
   - **Mid-Table Controls**: Hover/header control buttons per row and column allowing insertion and deletion of rows and columns anywhere mid-table.
   - **Serialization**: Tables are serialized back to standard GFM pipe table format on disk save.

## Consequences

- Clean separation of concerns between AST parsing, individual widget renderers, and Markdown serialization.
- High-fidelity visual tables in the GTK editor that blend seamlessly into the GNOME desktop design system.
- Full roundtrip compatibility with standard GFM pipe table syntax across external Markdown editors.
