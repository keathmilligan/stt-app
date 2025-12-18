# Design: Basic macOS Audio Support

## Context
FlowSTT is fully implemented on Linux (PipeWire) and Windows (WASAPI). The macOS port currently provides only a stub implementation. This design covers the basic audio infrastructure needed to make FlowSTT functional on macOS for single-source input capture.

## Goals / Non-Goals

### Goals
- Enable input device enumeration on macOS
- Enable microphone capture on macOS
- Integrate with existing whisper transcription pipeline
- Provide audio in the expected format (stereo f32 at 48kHz)

### Non-Goals
- System audio capture (requires ScreenCaptureKit complexity)
- Multi-source capture and mixing
- Echo cancellation
- Performance optimization beyond basic functionality

## Decisions

### Decision: Use coreaudio-rs crate for CoreAudio bindings
The `coreaudio-rs` crate provides high-level, safe Rust bindings to CoreAudio. It is mature and well-maintained.

**Alternatives considered:**
- `coreaudio-sys` (raw FFI) - More complex, requires manual memory management
- `cpal` (cross-platform) - Less control over CoreAudio-specific features

### Decision: Use AudioUnit HAL Output for input capture
CoreAudio's AudioUnit with `kAudioUnitSubType_HALOutput` provides direct access to hardware audio devices. This is the standard pattern for low-latency audio input on macOS.

**Implementation pattern:**
1. Create AudioUnit with HAL Output type
2. Enable input on element 1
3. Disable output on element 0
4. Set device ID on the unit
5. Set input callback for sample delivery

### Decision: Convert to f32 stereo 48kHz in the callback
To match the Windows and Linux backends, the macOS backend will:
1. Convert native format (typically Float32 or Int16/24/32) to f32
2. Convert mono to stereo by duplicating samples
3. Resample to 48kHz using linear interpolation (matching Windows)

### Decision: Thread architecture matches Windows pattern
The macOS backend will use a similar thread architecture to Windows:
- Main backend struct holds state and channels
- Audio callback runs on CoreAudio's audio thread
- Lock-free channel (crossbeam or std::sync::mpsc) for sample delivery
- Minimal work in audio callback (just push samples to channel)

## Component Structure

```
src-tauri/src/platform/macos/
├── mod.rs           # Module exports, create_backend()
├── coreaudio.rs     # CoreAudioBackend implementation
└── stub.rs          # Kept for reference, not used
```

## API Mapping

| AudioBackend method | CoreAudio implementation |
|---------------------|-------------------------|
| `list_input_devices()` | `AudioObjectGetPropertyData` with `kAudioHardwarePropertyDevices`, filter by input streams |
| `list_system_devices()` | Returns empty `Vec` (not implemented in this phase) |
| `sample_rate()` | Returns 48000 (output rate after resampling) |
| `start_capture_sources()` | Create and start AudioUnit for first source; error if second source provided |
| `stop_capture()` | Stop and dispose AudioUnit |
| `try_recv()` | Pop samples from crossbeam channel |

## Format Handling

CoreAudio devices may report various native formats:
- Float32 (most common on modern Macs)
- Int16, Int24, Int32 (some USB devices)

The backend will:
1. Query device's native format via `kAudioStreamPropertyPhysicalFormat`
2. Configure AudioUnit for native format
3. Convert to f32 in callback before sending to channel

## Sample Rate Handling

Most Mac audio devices run at 44.1kHz or 48kHz natively. The backend will:
1. Query device's native sample rate
2. If not 48kHz, apply linear interpolation resampling
3. Port the `Resampler` struct from Windows implementation

## Risks / Trade-offs

### Risk: Mono devices require duplication
- Some microphones are mono-only
- **Mitigation:** Check channel count and duplicate samples for mono sources

### Risk: Sample format variations
- Different devices may use different bit depths
- **Mitigation:** Handle common formats (Float32, Int16, Int24, Int32)

### Trade-off: No system audio in this phase
- Users cannot capture system audio until Phase 3
- **Acceptable:** Microphone input is the primary use case

## Open Questions
- None for this phase - design is straightforward port of Windows patterns
