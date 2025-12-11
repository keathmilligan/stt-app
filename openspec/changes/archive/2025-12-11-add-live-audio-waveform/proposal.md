# Change: Add Live Audio Waveform Visualization

## Why
Users need real-time visual feedback of audio input to verify their microphone is working correctly before and during recording. A live waveform display helps users troubleshoot audio issues and provides confidence that audio is being captured.

## What Changes
- Add a new audio visualization capability with a side-scrolling waveform display
- Stream audio samples from Rust backend to frontend via Tauri events for minimal latency
- Add a "Monitor" button to preview audio levels without recording
- Render waveform using Canvas API with requestAnimationFrame for smooth 60fps updates
- New dedicated UI section below controls for the waveform display

## Impact
- Affected specs: New `audio-visualization` capability
- Affected code:
  - `src-tauri/src/audio.rs` - Add audio sample streaming via events
  - `src-tauri/src/lib.rs` - Register new monitor commands and event emitter
  - `src/main.ts` - Add monitor button handling and waveform rendering
  - `index.html` - Add canvas element and monitor button
  - `src/styles.css` - Style the waveform display area
