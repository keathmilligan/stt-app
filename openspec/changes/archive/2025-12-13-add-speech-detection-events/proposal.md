# Change: Add Speech Detection with Start/End Events

## Why

The SuperFlow vision describes a multi-phase audio processing system where "cadence analysis" happens before transcription to detect natural speech pauses and determine *when* to respond. Currently, the `SilenceDetector` only logs to the console and doesn't emit events to the frontend. To enable the adaptive timeout and acknowledgment feedback loops described in the vision, the system needs to emit structured events when speech starts and ends that the frontend (and future cadence analyzer) can consume.

## What Changes

- **ADDED**: Speech detection processor that emits Tauri events (`speech-started`, `speech-ended`) when voice activity transitions occur
- **ADDED**: Configurable detection parameters (threshold, hold time) to tune sensitivity
- **MODIFIED**: Audio processor architecture to support event emission via `AppHandle`

## Impact

- Affected specs: `audio-processing`
- Affected code: `src-tauri/src/processor.rs`, `src-tauri/src/audio.rs`, `src/main.ts`
- Dependencies: None (builds on existing `AudioProcessor` trait)
