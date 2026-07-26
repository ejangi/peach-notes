<p align="center">
  <img src="assets/icons/peach-notes-icon.png" alt="Peach Notes Logo" width="128" height="128">
</p>

<h1 align="center">Peach Notes</h1>

<p align="center">
  <strong>A modern, native GNOME desktop notes application for Linux.</strong>
</p>

<p align="center">
  <a href="#key-features">Features</a> •
  <a href="#getting-started">Getting Started</a> •
  <a href="#built-with">Built With</a>
</p>

---

## 🍑 Overview

**Peach Notes** combines the elegance and simplicity of Apple Notes with the open power of local Markdown files. Built from the ground up for Linux desktops, it features a native Libadwaita user interface that adapts seamlessly to your system theme.

All your notes are stored locally as plain `.md` files in a folder of your choice, ensuring your data remains private, portable, and accessible anytime.

---

## ✨ Key Features

- 🎨 **Native GNOME & Libadwaita Interface**: Sleek dual-pane layout with light and dark mode auto-detection.
- 📝 **Live WYSIWYG Markdown Editing**: Rich visual text rendering for headings, bold, italics, strikethrough, code blocks, and links.
- 📌 **Smart Bullet Lists**: Auto-converts `* ` or `- ` into bullet points on the fly. Hitting <kbd>Enter</kbd> automatically starts the next item, while hitting <kbd>Enter</kbd> on an empty bullet exits list mode.
- 🖼️ **Drag & Drop Media & Files**: Drag images or documents directly into notes. Images render with optional captions, and non-image files are displayed as interactive attachment cards.
- 📐 **Dynamic Image Resizing**: Images scale smoothly in real-time as you resize the application window or sidebar pane while maintaining their original proportions.
- 🔍 **Instant Search & Fast Sorting**: Find notes instantly with real-time filtering, sorted automatically by your most recently modified note.
- 🔒 **100% Local & Private**: Your notes are standard Markdown files stored on disk. No lock-in, no tracking, no cloud requirement.

---

## 🚀 Getting Started

### Prerequisites

- A Linux environment with **GTK4** and **Libadwaita** installed.
- **Rust** compiler (version 1.70+ recommended).

### Running Peach Notes

1. **Clone the repository**:
   ```bash
   git clone https://github.com/ejangi/peach-notes.git
   cd peach-notes
   ```

2. **Build & Run**:
   ```bash
   cargo run --release
   ```

---

## 🤖 Built With

- **Rust** — Fast, safe, and efficient systems language.
- **GTK4 & Libadwaita** — Modern GNOME desktop design language.
- **pulldown-cmark** — Fast CommonMark/GFM Markdown parser.

<p align="center">
  ✨ <em>Vibe-coded using <strong>Google Antigravity</strong></em> ✨
</p>
