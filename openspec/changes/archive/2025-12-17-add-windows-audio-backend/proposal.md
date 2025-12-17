# Change: Add basic Windows audio backend using WASAPI

## Why
Windows users cannot currently use FlowSTT for audio capture - the stub backend returns "not implemented" errors. This change enables basic audio functionality on Windows by implementing device enumeration and single-source input capture via WASAPI.

## What Changes
- Replace the Windows stub backend with a working WASAPI-based implementation
- Implement input device enumeration (microphones)
- Implement single-source audio capture from input devices
- Maintain stub behavior for advanced features:
  - System audio capture (loopback) - returns empty device list
  - Multiple source capture - returns error
  - Audio mixing - not applicable (single source only)
  - Echo cancellation - not applicable (single source only)

## Impact
- Affected specs: `audio-recording`
- Affected code: `src-tauri/src/platform/windows/` (new `wasapi.rs`, modified `mod.rs`)
- Dependencies: `windows` crate for WASAPI bindings
- No breaking changes to existing API - the `AudioBackend` trait interface remains unchanged
