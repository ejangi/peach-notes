# 6. Standardized Block Element Spacing for Markdown Serialization

- Status: Accepted
- Date: 2026-07-26

## Context & Problem Statement

When notes edited in the GTK `Rich Text Buffer` are serialized to disk as `.md` files by the `Note Serializer`, block-level elements (such as H1/H2/H3 headings, paragraphs, fenced code blocks, GFM tables, images, attachments, and lists) need consistent, predictable spacing.

Without explicit separation rules, block elements risk either running together without blank lines (reducing readability in raw Markdown viewers) or accumulating excess whitespace over multiple save cycles.

We need to define how block-level boundaries are formatted on disk when committing Markdown documents.

## Decision Drivers

- Clean, standard Markdown formatting across all `.md` files in the `Notes Directory`.
- Visual consistency between Peach Notes and standard Markdown previewers / Git diff tools.
- Prevention of blank-line bloat (stacking 3–4 newlines when serializing adjacent blocks).
- Clear rules for special elements like YAML frontmatter, list items inside list blocks, and POSIX EOF single-newline termination.

## Considered Options

1. **Strict Double Newline Separation (`\n\n` / 1 Blank Line)** (Selected):
   - Every top-level Block Element is separated from adjacent Block Elements by exactly two newlines (`\n\n`), resulting in one blank line between blocks.
   - List items *within* a list container use single newlines (`\n`), but the list container as a whole is separated from preceding/following blocks by `\n\n`.
   - YAML frontmatter (if present) is separated from the first block element by `\n\n`.
   - Documents terminate with a single trailing newline (`\n`).

2. **Verbatim In-Buffer Spacing Preservation**:
   - Preserve whatever arbitrary number of empty lines exist in the GTK text buffer.
   - Trade-off: Unpredictable raw Markdown documents on disk and erratic line spacing.

3. **Asymmetrical Pre-Pad / Post-Pad Stacking**:
   - Naively prepend `\n\n` before and append `\n\n` after every block.
   - Trade-off: Stacks to 4 newlines (`\n\n\n\n` / 3 blank lines) between adjacent blocks.

## Decision Outcome

Chosen Option: **1. Strict Double Newline Separation (`\n\n` / 1 Blank Line)**.

### Block Separation Rules

1. **Adjacent Block Elements**: Separated by exactly `\n\n` (one blank line).
2. **List Containers**: The list block is padded with `\n\n` before and after, but individual items inside the list remain single-newline (`\n`) separated.
3. **YAML Frontmatter**: Padded with `\n\n` before the first block element.
4. **EOF**: Ended with a single trailing newline `\n`.

### Consequences

- **Positive**: Clean, standardized `.md` files on disk that render cleanly in GitHub, GNOME tools, and external text editors.
- **Positive**: Prevents whitespace creep and redundant empty lines on save.
- **Negative / Trade-off**: In-editor trailing empty lines between blocks will be normalized to standard single-blank-line separation when saved.
