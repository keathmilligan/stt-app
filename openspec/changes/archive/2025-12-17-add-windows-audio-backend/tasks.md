## 1. Dependencies and Project Setup
- [x] 1.1 Add `windows` crate dependency with required WASAPI features to `Cargo.toml` (Windows-only target)
- [x] 1.2 Verify project compiles on Windows with new dependency

## 2. WASAPI Backend Module Structure
- [x] 2.1 Create `src-tauri/src/platform/windows/wasapi.rs` with module structure
- [x] 2.2 Update `src-tauri/src/platform/windows/mod.rs` to use `wasapi` instead of `stub`
- [x] 2.3 Implement `WasapiBackend` struct with required state (device enumerator, capture thread handle, channels)

## 3. Device Enumeration
- [x] 3.1 Implement COM initialization helper for device enumeration
- [x] 3.2 Implement `list_input_devices()` using `IMMDeviceEnumerator` to list capture endpoints
- [x] 3.3 Map WASAPI device IDs and friendly names to `PlatformAudioDevice` format
- [x] 3.4 Implement `list_system_devices()` as stub returning empty list
- [x] 3.5 Test device enumeration shows available microphones

## 4. Audio Capture Implementation
- [x] 4.1 Implement `start_capture_sources()` for single-source case
- [x] 4.2 Create capture thread with COM initialization (MTA)
- [x] 4.3 Implement WASAPI shared mode stream initialization with format negotiation
- [x] 4.4 Implement capture loop reading from `IAudioCaptureClient`
- [x] 4.5 Convert captured samples to stereo f32 format (handle 16-bit int, mono, etc.)
- [x] 4.6 Resample to 48kHz if device uses different sample rate
- [x] 4.7 Send samples via channel to main thread
- [x] 4.8 Implement `start_capture_sources()` error for two-source case

## 5. Stop and Cleanup
- [x] 5.1 Implement `stop_capture()` to signal capture thread termination
- [x] 5.2 Ensure proper cleanup of WASAPI resources (client, device, enumerator)
- [x] 5.3 Implement `try_recv()` to receive samples from capture thread

## 6. Integration and Testing
- [x] 6.1 Verify backend compiles and links on Windows
- [x] 6.2 Test device enumeration in application UI
- [x] 6.3 Test start/stop recording with single microphone
- [x] 6.4 Verify audio visualization displays captured audio
- [x] 6.5 Verify transcription works with captured audio
- [x] 6.6 Test error handling for unavailable device
- [x] 6.7 Test graceful behavior when selecting "System" source type (empty list shown)
