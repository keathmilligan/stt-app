## Context
This is a Tauri 2.0 desktop app (Rust backend + TypeScript frontend). We need to capture audio from user-selected input devices and transcribe using local Whisper models via whisper-rs (bindings to whisper.cpp).

## Goals / Non-Goals
- **Goals:**
  - Enumerate all recordable audio sources (input devices)
  - Record audio from selected source
  - Transcribe audio locally using Whisper
  - Display transcription results in the UI
- **Non-Goals:**
  - System audio capture (loopback) - platform-specific complexity, defer to future
  - Streaming/real-time transcription - batch processing first
  - Model selection UI - use a sensible default (base.en)
  - Transcription history or persistence

## Decisions

### Audio Capture: cpal
- **Decision:** Use `cpal` crate for cross-platform audio device enumeration and recording
- **Alternatives considered:**
  - `rodio` - higher-level, but less control over device selection
  - Platform-specific APIs - not portable
- **Rationale:** cpal is the standard Rust audio I/O library, supports device enumeration, works on Windows/macOS/Linux

### Transcription: whisper-rs
- **Decision:** Use `whisper-rs` crate (whisper.cpp bindings)
- **Alternatives considered:**
  - Pure Rust implementations - less mature
  - OpenAI API - requires internet, costs money
- **Rationale:** whisper.cpp is battle-tested, whisper-rs provides safe Rust bindings, runs fully offline

### Audio Format
- **Decision:** Record as f32 samples, convert to 16kHz mono WAV for Whisper
- **Rationale:** Whisper expects 16kHz mono audio; cpal provides flexible sample formats

### Model Management
- **Decision:** Expect model file at a known location, download separately (not bundled)
- **Rationale:** Models are large (75MB-1.5GB); bundling bloats the app. User downloads once.
- **Default model path:** `~/.cache/whisper/ggml-base.en.bin` (or platform equivalent)

### Recording State
- **Decision:** Simple start/stop toggle managed in Rust, UI reflects state
- **Rationale:** Minimal complexity for scaffolding phase

## Risks / Trade-offs
- **Model not found:** App should gracefully error if model missing, with clear message
- **Long recordings:** Large audio buffers in memory; acceptable for scaffolding, optimize later
- **Platform audio permissions:** macOS/Windows may require permission grants; Tauri handles some, may need user action

## Open Questions
- Should we auto-download the model on first run? (Suggest: no, keep scaffolding simple)
