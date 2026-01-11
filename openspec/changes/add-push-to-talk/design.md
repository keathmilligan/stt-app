# Design: Push-to-Talk Operating Mode

## Context

FlowSTT currently operates in a single automatic transcription mode where Voice Activity Detection (VAD) determines speech boundaries. This proposal adds a push-to-talk (PTT) mode as an alternative, requiring system-wide keyboard event capture across multiple platforms.

**Stakeholders**: End users who want manual control over recording, particularly in noisy environments or multi-speaker scenarios.

**Constraints**:
- macOS requires Accessibility permissions for global hotkey capture (CGEventTap)
- Different platforms have fundamentally different APIs for system-wide input monitoring
- The hotkey must work when the application window is not focused
- The service process handles audio capture, so hotkey handling should be in the service layer

## Goals / Non-Goals

**Goals**:
- Enable manual push-to-talk control as an alternative to VAD-triggered transcription
- Provide a working macOS implementation using CGEventTap
- Define clear interfaces for cross-platform hotkey abstraction
- Allow configurable hotkey binding (with sensible defaults)
- Provide visual feedback of PTT state in GUI

**Non-Goals**:
- Full Windows implementation (stub only for now)
- Full Linux implementation (stub only for now)
- Modifier key combinations (e.g., Cmd+Space) - single key only initially
- Multiple simultaneous hotkeys
- Per-application hotkey customization

## Decisions

### Decision 1: Service-Layer Hotkey Handling

**What**: Implement global hotkey capture in the service process (`flowstt-service`), not in the Tauri GUI.

**Why**: The service process owns audio capture and runs independently of the GUI. Having the service handle hotkeys ensures PTT works even when using the CLI interface, and avoids IPC round-trips for time-sensitive key events.

**Alternatives considered**:
- **Tauri GlobalShortcut plugin**: Only works when app is focused; rejected
- **Separate hotkey daemon**: Adds deployment complexity; rejected

### Decision 2: Platform Abstraction Trait

**What**: Define a `HotkeyBackend` trait similar to the existing `AudioBackend` pattern, with platform-specific implementations selected at compile time.

```rust
pub trait HotkeyBackend: Send {
    fn start(&mut self, key: KeyCode) -> Result<(), String>;
    fn stop(&mut self);
    fn try_recv(&self) -> Option<HotkeyEvent>;
}

pub enum HotkeyEvent {
    Pressed,
    Released,
}
```

**Why**: Consistent with existing architecture patterns, enables clean platform separation, and allows stub implementations without affecting other platforms.

### Decision 3: macOS CGEventTap Implementation

**What**: Use macOS CGEventTap API with passive monitoring (kCGEventTapOptionListenOnly) to detect global key events.

**Why**: CGEventTap is the standard macOS API for system-wide input monitoring. Passive mode avoids blocking other applications' key handling while still detecting press/release events.

**Permissions**: Requires Accessibility permission (kTCCServiceAccessibility). The application should detect missing permissions and guide the user to enable them.

### Decision 4: Default Hotkey: Right Option Key

**What**: Use the Right Option (Alt) key as the default PTT hotkey on macOS.

**Why**: 
- Single key avoids complexity of modifier combinations
- Right Option is rarely used in normal typing
- Ergonomically accessible for right-hand operation
- Doesn't conflict with common shortcuts

**Configuration**: Users can change to other keys via settings.

### Decision 5: Transcription Mode Enum

**What**: Add a `TranscriptionMode` enum to distinguish between automatic (VAD) and push-to-talk modes.

```rust
pub enum TranscriptionMode {
    Automatic,  // VAD-triggered (current behavior)
    PushToTalk, // Manual key-controlled
}
```

**Why**: Clean state representation that can be extended for future modes. The mode affects how the audio loop handles speech segment boundaries.

### Decision 6: PTT Audio Loop Behavior

**What**: In PTT mode, key press triggers immediate segment start (with brief lookback for onset capture), and key release triggers immediate segment end and transcription submission.

**Behavior differences from Automatic mode**:
- **Segment start**: Triggered by key press, not VAD speech-started event
- **Segment end**: Triggered by key release, not VAD speech-ended event
- **Lookback**: Small lookback buffer (50-100ms) to capture speech onset that precedes the physical key press
- **Duration limits**: No automatic segmentation at word breaks (user controls duration)
- **VAD**: Speech detection still runs for visualization but doesn't control segment boundaries

## Risks / Trade-offs

### Risk: macOS Accessibility Permission UX

**Risk**: Users may be confused or reluctant to grant Accessibility permissions.

**Mitigation**: 
- Clear in-app explanation of why permission is needed
- Direct link to System Preferences > Security & Privacy > Accessibility
- Graceful fallback with clear error message if permission denied
- PTT mode simply unavailable without permission (Automatic mode still works)

### Risk: Platform Implementation Disparity

**Risk**: Windows and Linux users will have stubs, creating inconsistent experience.

**Mitigation**:
- Clear documentation that PTT is macOS-only initially
- UI disables PTT mode selection on unsupported platforms
- Stub implementations compile cleanly and return appropriate errors

### Trade-off: Single Key vs Modifier Combinations

**Trade-off**: Supporting only single keys is simpler but less flexible.

**Decision**: Start simple with single keys. Modifier combinations can be added later if needed. Most PTT implementations (Discord, TeamSpeak, etc.) work well with single keys.

### Trade-off: Key Press Lookback Buffer

**Trade-off**: Users typically press the key slightly after starting to speak, losing initial audio.

**Decision**: Include a small lookback buffer (~100ms) when PTT activates to capture speech onset. This is shorter than VAD lookback (200ms) since the user's intent is clear.

## Migration Plan

No migration needed—this is a new feature. Default behavior remains Automatic mode, preserving existing user experience.

**Rollout**:
1. Ship with Automatic mode as default
2. PTT mode available via settings on macOS
3. PTT mode greyed out/unavailable on Windows/Linux until implemented

## Open Questions

1. **Hotkey customization UI**: Should users configure the PTT key through the GUI, CLI, or config file? 
   - *Proposed*: Start with config file + CLI, add GUI later

2. **Audio feedback**: Should there be an audible indicator when PTT activates/deactivates?
   - *Proposed*: No audio feedback initially; rely on visual indicator

3. **Hold vs Toggle**: Should there be a toggle mode where first press starts, second press stops?
   - *Proposed*: Hold-only initially; toggle mode can be added later if requested
