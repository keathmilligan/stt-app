# Change: Add macOS Multi-Source Audio Capture

## Why
The macOS audio backend currently supports only single-source input capture. To achieve feature parity with Linux and Windows, macOS needs system audio enumeration, multi-source capture, and audio mixing capabilities. This enables users on macOS to capture both microphone and system audio simultaneously, which is essential for transcribing conversations, meetings, and media playback.

## What Changes
- **System audio enumeration**: Enumerate available system audio outputs using ScreenCaptureKit (macOS 12.3+) as capturable sources
- **Multi-source capture**: Support concurrent capture from microphone (CoreAudio AudioUnit) and system audio (ScreenCaptureKit) simultaneously
- **Audio mixing**: Port `AudioMixer` from Windows/Linux implementations to combine and process multiple audio streams
- **Echo cancellation integration**: Enable AEC3 for macOS by adding the `aec3` crate dependency and integrating with the mixer
- **Thread-per-stream architecture**: Implement `MultiCaptureManager` pattern to coordinate multiple capture streams
- **Recording mode support**: Wire up Mixed and EchoCancel recording modes in the macOS mixer

## Impact
- Affected specs: `audio-recording`
- Affected code:
  - `src-tauri/src/platform/macos/coreaudio.rs` - Main CoreAudio backend
  - `src-tauri/src/platform/macos/mod.rs` - Module exports (may add screencapturekit.rs)
  - `src-tauri/Cargo.toml` - Add `aec3` dependency for macOS, add `objc2` and ScreenCaptureKit bindings
  - `src-tauri/build.rs` - May need entitlements for ScreenCaptureKit
- Platform-specific:
  - Requires macOS 12.3+ for ScreenCaptureKit API
  - Requires Screen Recording permission from user for system audio capture
  - Requires proper Info.plist entries for privacy permissions
