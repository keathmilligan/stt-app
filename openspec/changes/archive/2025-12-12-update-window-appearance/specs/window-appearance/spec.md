## ADDED Requirements

### Requirement: Gradient Background
The application SHALL display a dark gray gradient background across the entire window.

#### Scenario: Background renders on launch
- **WHEN** the application window opens
- **THEN** the background displays a smooth dark gray gradient

### Requirement: Fixed Window Size
The application window SHALL be non-resizable with a fixed size of 800x600 pixels.

#### Scenario: User attempts to resize window
- **WHEN** the user attempts to resize the window by dragging edges or corners
- **THEN** the window remains at its fixed 800x600 size

### Requirement: No Title Bar
The application window SHALL display without a native title bar (window decorations disabled).

#### Scenario: Window renders without decorations
- **WHEN** the application window opens
- **THEN** no native title bar or window frame decorations are visible

### Requirement: Custom Drag Region
The application SHALL provide a custom drag region in the header area to allow window repositioning without a native title bar.

#### Scenario: User drags window via header
- **WHEN** the user clicks and drags on the header area
- **THEN** the window moves with the cursor to reposition on screen

#### Scenario: Drag region visual feedback
- **WHEN** the user hovers over the drag region
- **THEN** the cursor indicates the area is draggable
