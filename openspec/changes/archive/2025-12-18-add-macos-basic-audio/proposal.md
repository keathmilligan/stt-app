# Change: Add Basic macOS Audio Support

## Why
macOS currently has only a stub audio backend that returns "not implemented" errors for all operations. Users on macOS cannot enumerate audio devices, capture audio, or transcribe speech. This change implements the foundational audio functionality for macOS to enable device enumeration, single-source capture, and whisper transcription.

## What Changes
- Replace the macOS stub backend with a functional CoreAudio implementation
- Implement input device enumeration using CoreAudio APIs
- Implement single-source audio capture from microphones using AudioUnit
- Add format conversion (native format to f32 stereo)
- Add sample rate conversion (native rate to 48kHz)
- Add mono-to-stereo conversion for mono input devices
- Wire through to whisper transcription (already functional on macOS)

## Scope Exclusions (Future Work)
The following features are **not** included in this change and will be addressed in subsequent proposals:
- System audio capture (ScreenCaptureKit or aggregate devices)
- Multi-source capture (microphone + system audio)
- Audio mixing
- Echo cancellation integration
- Recording mode support (Mixed/EchoCancel)

## Impact
- Affected specs: `audio-recording`
- Affected code:
  - `src-tauri/src/platform/macos/` - Replace stub with CoreAudio implementation
  - `src-tauri/Cargo.toml` - Add CoreAudio dependencies
- No breaking changes to other platforms
