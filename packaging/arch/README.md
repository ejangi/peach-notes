# Installing Peach Notes on Arch Linux / Manjaro

This package contains the pre-built release binary and desktop integration files for **Peach Notes**.

## Installation Instructions

1. Open a terminal in the extracted directory (where `PKGBUILD` is located).

2. Build and install the package using `makepkg`:
   ```bash
   makepkg -si
   ```

   - `-s` (`--syncdeps`): Automatically installs missing runtime dependencies using `pacman`.
   - `-i` (`--install`): Automatically installs the package once built.

3. Alternatively, to build without auto-installing:
   ```bash
   makepkg -s
   sudo pacman -U peach-notes-bin-*.pkg.tar.zst
   ```

## Dependencies
- `gtk4`
- `libadwaita`
- `gtksourceview5`

## Uninstalling
To remove the package:
```bash
sudo pacman -R peach-notes-bin
```
