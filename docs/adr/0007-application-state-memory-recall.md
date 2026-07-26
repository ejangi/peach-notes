# 7. Application State Memory & Recall

- Status: Accepted
- Date: 2026-07-26

## Context & Problem Statement

Peach Notes needs to restore user session context across application restarts to provide a seamless, native desktop experience. Specifically, the application should remember:
1. Which note was open last time and automatically reopen it upon launch.
2. Window dimensions (width and height) and maximized window state.
3. The X/Y location on screen from the previous session.

## Decision Drivers

- Seamless session resumption for the user without requiring manual note search on startup.
- Consistent window size and layout across sessions.
- Compliance with GNOME / GTK4 architecture and Wayland display server security boundaries.

## Considered Options

1. **Full State Persistence (Dimensions, Maximized State, & Last Opened Note)** (Selected):
   - Store un-maximized `window_width`, `window_height`, `is_maximized`, and `last_opened_note` (Note ID relative file path) in `~/.config/peach-notes/config.json`.
   - Update window dimensions via debounced GTK signal listeners without overwriting un-maximized dimensions when maximized.
   - Reopen `last_opened_note` on launch if it exists, falling back to the first available note if missing or deleted.
   - Exclude absolute X/Y screen coordinates due to GTK4 and Wayland compositor restrictions.

2. **Include Platform-Specific X11 Window Positioning**:
   - Attempt X11-specific positioning calls or low-level window manager hints for X/Y coordinate restoration.

## Decision Outcome

Chosen Option: **1. Full State Persistence (Dimensions, Maximized State, & Last Opened Note)**.

### Consequences & Technical Nuances

- **Window Location (X/Y)**: Documented as a platform-restricted limitation. GTK4 deprecated/removed `move()` and `set_position()` APIs because Wayland compositors strictly govern window placement.
- **Window Dimensions & Maximized State**: Window dimensions are continuously updated with debounced saves during resize events. When the window is maximized, stored `window_width` and `window_height` reflect the last un-maximized dimensions.
- **Last Opened Note Persistence**: Stored as a relative Note ID (`last_opened_note`). If the persisted note file is deleted or renamed externally while closed, the app gracefully falls back to selecting the top note in the sidebar.
