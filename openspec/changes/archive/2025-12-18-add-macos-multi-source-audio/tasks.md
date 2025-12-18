## 1. Dependencies and Build Setup
- [x] 1.1 Add `aec3` crate to macOS dependencies in `Cargo.toml`
- [x] 1.2 Add `objc2` and related crates for Objective-C interop
- [x] 1.3 Add ScreenCaptureKit framework linking in build configuration
- [x] 1.4 Add Screen Recording entitlement to app entitlements (if not already present)
- [x] 1.5 Verify build compiles on macOS with new dependencies

## 2. System Audio Capture Module
- [x] 2.1 Create `screencapturekit.rs` module for system audio capture (fallback)
- [x] 2.2 Create `coreaudio_taps.rs` module for Core Audio Taps (macOS 14.2+, preferred)
- [x] 2.3 Implement ScreenCaptureKit permission check function
- [x] 2.4 Implement system audio enumeration
- [x] 2.5 Implement automatic backend selection (Core Audio Taps on 14.2+, ScreenCaptureKit on 12.3-14.1)
- [x] 2.6 Implement `try_recv()` pattern for non-blocking sample retrieval
- [ ] 2.7 Test system audio enumeration returns available outputs

## 3. Multi-Source Capture Infrastructure
- [x] 3.1 Create `StreamSamples` struct for inter-thread communication
- [x] 3.2 Create `MultiCaptureManager` struct to coordinate capture streams
- [x] 3.3 Implement thread-per-stream architecture for CoreAudio input
- [x] 3.4 Implement thread-per-stream architecture for system audio capture
- [x] 3.5 Wire up mpsc channels between stream threads and main mixer thread
- [ ] 3.6 Test concurrent capture from both sources delivers samples

## 4. Audio Mixer Port
- [x] 4.1 Port `AudioMixer` struct from Windows implementation
- [x] 4.2 Implement `push_samples()` with source routing (loopback vs mic)
- [x] 4.3 Implement render-first AEC pattern (feed system audio first)
- [x] 4.4 Implement `process_capture()` for AEC and mixing
- [x] 4.5 Implement Mixed recording mode (combine streams with soft clipping)
- [x] 4.6 Implement EchoCancel recording mode (mic-only with echo removed)
- [x] 4.7 Wire `aec_enabled` and `recording_mode` flags to mixer

## 5. Backend Integration
- [x] 5.1 Update `list_system_devices()` to return system audio sources
- [x] 5.2 Update `start_capture_sources()` to handle dual-source mode
- [x] 5.3 Update `stop_capture()` to stop all stream threads cleanly
- [x] 5.4 Handle permission denied error with clear user-facing message
- [x] 5.5 Remove `stub.rs` (no longer needed)

## 6. Testing and Validation
- [x] 6.1 Test input device enumeration still works
- [ ] 6.2 Test system device enumeration shows available outputs
- [x] 6.3 Test single-source input capture works
- [ ] 6.4 Test single-source system audio capture works
- [ ] 6.5 Test dual-source capture delivers mixed audio
- [ ] 6.6 Test echo cancellation toggle affects output
- [ ] 6.7 Test recording mode switch between Mixed and EchoCancel
- [ ] 6.8 Test on Apple Silicon Mac
- [ ] 6.9 Test on Intel Mac (if available)
- [ ] 6.10 Test permission handling when Screen Recording is denied (for ScreenCaptureKit fallback)

## 7. Spec Updates
- [ ] 7.1 Archive this change and update `audio-recording` spec to reflect full macOS support

## Implementation Notes

### What Was Implemented

1. **Dependencies**: Added `aec3`, `objc2`, `objc2-foundation`, `objc2-app-kit`, `block2`, and `dispatch` crates to Cargo.toml for macOS support.

2. **Build Configuration**: Updated `build.rs` to link ScreenCaptureKit, CoreMedia, and AVFoundation frameworks on macOS.

3. **Entitlements**: Created `entitlements.plist` with `com.apple.security.screen-capture` and `com.apple.security.device.audio-input` entitlements. Updated `tauri.conf.json` to reference the entitlements file and set minimum macOS version to 12.3.

4. **Core Audio Taps Module** (`coreaudio_taps.rs`) - **NEW, PREFERRED**:
   - macOS 14.2+ system audio capture via `AudioHardwareCreateProcessTap`
   - Does NOT require Screen Recording permission
   - Lower latency than ScreenCaptureKit
   - Excludes own process from tap to prevent feedback
   - Uses CATapDescription Objective-C class via objc2

5. **ScreenCaptureKit Module** (`screencapturekit.rs`) - **FALLBACK**:
   - Version check using `sw_vers` command
   - Permission check via `CGPreflightScreenCaptureAccess`
   - Permission request via `CGRequestScreenCaptureAccess`
   - System audio device enumeration (returns single "System Audio" device)
   - `SCKAudioCapture` struct with automatic backend selection
   - Placeholder implementation for actual ScreenCaptureKit audio capture

6. **CoreAudio Backend** (`coreaudio.rs`):
   - Full rewrite to support multi-source capture
   - `AudioMixer` struct ported from Windows implementation with AEC3 integration
   - `MultiCaptureManager` for coordinating multiple capture streams
   - Thread-per-stream architecture for CoreAudio input
   - System audio polling via Core Audio Taps or ScreenCaptureKit
   - Render-first AEC pattern for echo cancellation
   - Mixed and EchoCancel recording modes with soft clipping

### System Audio Backend Selection

The system automatically selects the best backend:
- **macOS 14.2+**: Core Audio Taps (no permission required, lower latency)
- **macOS 12.3-14.1**: ScreenCaptureKit (requires Screen Recording permission)

### Known Limitations

1. **ScreenCaptureKit Fallback**: The ScreenCaptureKit audio capture is still a placeholder implementation. Full audio capture requires additional Objective-C binding work. This affects only macOS 12.3-14.1 users.

2. **Testing Required**: Manual testing is needed on actual macOS hardware to verify:
   - System audio capture via Core Audio Taps
   - Dual-source mixing
   - Echo cancellation effectiveness

3. **Permission UX**: For the ScreenCaptureKit fallback path, permission handling needs real-world testing.
