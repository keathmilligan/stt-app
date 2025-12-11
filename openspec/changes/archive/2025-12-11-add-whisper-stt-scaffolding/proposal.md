# Change: Add Whisper Speech-to-Text Scaffolding

## Why
The app needs core speech-to-text functionality using local Whisper inference. This establishes the foundational audio capture and transcription pipeline that future features will build upon.

## What Changes
- Add audio device enumeration and selection in the frontend
- Add audio recording capability via Tauri commands (Rust backend)
- Integrate whisper-rs for local speech transcription
- Create minimal UI: device selector, record button, transcription display

## Impact
- Affected specs: `audio-recording` (new), `speech-transcription` (new)
- Affected code:
  - `src-tauri/Cargo.toml` - new dependencies (cpal, whisper-rs)
  - `src-tauri/src/lib.rs` - new Tauri commands
  - `src/main.ts` - recording UI and logic
  - `index.html` - UI elements
  - `src-tauri/tauri.conf.json` - permissions for audio access
