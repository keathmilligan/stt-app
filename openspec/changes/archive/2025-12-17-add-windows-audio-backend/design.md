## Context

FlowSTT currently supports full audio capture on Linux via PipeWire. Windows and macOS have stub backends that compile but return "not implemented" errors. This change adds basic Windows audio support using WASAPI (Windows Audio Session API), following the same `AudioBackend` trait pattern.

The implementation is intentionally minimal: single-source input capture only. Advanced features (loopback capture, mixing, AEC) require significantly more complexity and are deferred to a future change.

## Goals / Non-Goals

**Goals:**
- Enable Windows users to enumerate and select input devices (microphones)
- Enable single-source audio capture from selected input device
- Provide audio samples in the same format as Linux backend (stereo f32 interleaved)
- Maintain identical `AudioBackend` trait interface

**Non-Goals:**
- System audio capture (loopback) - requires separate WASAPI loopback stream
- Multiple source capture - requires multi-stream mixing architecture
- Echo cancellation - requires two active streams
- Low-latency optimizations - basic shared mode is sufficient for now

## Decisions

### Use `windows` crate for WASAPI bindings
- **Decision**: Use the official `windows` crate with generated bindings
- **Rationale**: Well-maintained, type-safe, and supports all needed WASAPI interfaces
- **Alternatives**: 
  - `winapi` - lower-level, more manual, deprecated in favor of `windows`
  - `cpal` - higher-level abstraction but less control over device selection

### Use WASAPI shared mode
- **Decision**: Use shared mode (not exclusive mode) for audio capture
- **Rationale**: Shared mode allows other applications to use the audio device simultaneously, which is expected desktop behavior
- **Trade-off**: Slightly higher latency than exclusive mode, but acceptable for speech-to-text

### Capture in stereo f32 format
- **Decision**: Request stereo float32 format to match Linux backend output
- **Rationale**: Maintains consistent sample format across platforms, simplifying downstream processing
- **Fallback**: If f32 not supported, capture in native format and convert

### Thread-based capture loop
- **Decision**: Use a dedicated thread for WASAPI capture, communicating via channels
- **Rationale**: Matches PipeWire backend architecture; WASAPI requires COM initialization on the capture thread
- **Alternative**: Async/await - more complex, no clear benefit for this use case

### Stub advanced features gracefully
- **Decision**: `list_system_devices()` returns empty list; multi-source capture returns error
- **Rationale**: Allows incremental feature additions without API changes

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| WASAPI format negotiation complexity | Start with common formats (48kHz/44.1kHz stereo), expand if needed |
| COM threading model issues | Initialize COM as MTA on capture thread, document requirement |
| Device hotplug not handled | Acceptable for MVP; add device change notifications later |

## Migration Plan

No migration needed - this replaces a stub that had no functionality. Users on Windows will go from "not implemented" to working basic capture.

## Open Questions

- Should we support 44.1kHz native capture or always request 48kHz?
  - **Initial approach**: Accept device default rate, resample to 48kHz if different
- How to handle USB audio devices that only support 16-bit?
  - **Initial approach**: Convert to f32 in capture callback
