#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MASTER_ICON="$PROJECT_ROOT/assets/icons/peach-notes-icon.png"
OUTPUT_BASE="$PROJECT_ROOT/assets/icons/hicolor"

if [ ! -f "$MASTER_ICON" ]; then
    echo "Error: Master icon not found at $MASTER_ICON" >&2
    exit 1
fi

IM_CMD="magick"
if ! command -v magick &>/dev/null; then
    IM_CMD="convert"
fi

SIZES=(16 32 48 64 128 256 512)

echo "Generating icons from $MASTER_ICON..."

for SIZE in "${SIZES[@]}"; do
    TARGET_DIR="$OUTPUT_BASE/${SIZE}x${SIZE}/apps"
    mkdir -p "$TARGET_DIR"
    TARGET_FILE="$TARGET_DIR/org.gnome.PeachNotes.png"
    
    echo "Creating ${SIZE}x${SIZE} icon at $TARGET_FILE"
    "$IM_CMD" "$MASTER_ICON" \
        -resize "${SIZE}x${SIZE}" \
        -background transparent \
        -gravity center \
        -extent "${SIZE}x${SIZE}" \
        "$TARGET_FILE"
done

echo "Icon generation complete!"
