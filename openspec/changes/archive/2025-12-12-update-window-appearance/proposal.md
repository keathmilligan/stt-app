# Change: Update Window Appearance

## Why
Improve the visual aesthetic of the application with a modern dark gray gradient background while creating a cleaner, more focused user experience by removing the title bar and preventing window resizing.

## What Changes
- Apply a dark gray gradient background to the application body
- Configure the Tauri window to be non-resizable (fixed size)
- Remove the window title bar (decorations disabled)
- Add a custom drag region to allow window dragging without a title bar

## Impact
- Affected specs: window-appearance (new capability)
- Affected code:
  - `src/styles.css` - Add gradient background styles and drag region styling
  - `src-tauri/tauri.conf.json` - Window configuration (resizable, decorations)
  - `index.html` - Add `data-tauri-drag-region` attribute to header element
