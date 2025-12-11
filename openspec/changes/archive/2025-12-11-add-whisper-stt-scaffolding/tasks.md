## 1. Backend: Audio Device Enumeration
- [x] 1.1 Add `cpal` dependency to Cargo.toml
- [x] 1.2 Create Tauri command `list_audio_devices` that returns available input devices
- [x] 1.3 Test device enumeration works on host platform

## 2. Backend: Audio Recording
- [x] 2.1 Create audio recording module with start/stop control
- [x] 2.2 Implement `start_recording(device_id)` Tauri command
- [x] 2.3 Implement `stop_recording()` Tauri command that returns audio data
- [x] 2.4 Handle sample rate conversion to 16kHz mono for Whisper compatibility

## 3. Backend: Whisper Integration
- [x] 3.1 Add `whisper-rs` dependency to Cargo.toml
- [x] 3.2 Create transcription module that loads Whisper model
- [x] 3.3 Implement `transcribe(audio_data)` Tauri command
- [x] 3.4 Handle model-not-found error gracefully with user message

## 4. Frontend: UI Components
- [x] 4.1 Replace template UI in index.html with recording interface
- [x] 4.2 Add device selector dropdown
- [x] 4.3 Add record/stop toggle button
- [x] 4.4 Add transcription result display area
- [x] 4.5 Update styles.css for new UI elements

## 5. Frontend: Recording Logic
- [x] 5.1 Fetch and populate device list on load
- [x] 5.2 Implement record button click handler (start/stop toggle)
- [x] 5.3 Call transcribe after recording stops, display result
- [x] 5.4 Show loading state during transcription

## 6. Configuration
- [x] 6.1 Update tauri.conf.json if additional permissions needed
- [x] 6.2 Document model download instructions in README
