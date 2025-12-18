## Context

The macOS audio backend currently provides input device enumeration and single-source capture via CoreAudio. System audio capture on macOS is fundamentally different from Windows (WASAPI loopback) and Linux (PipeWire sink monitors) because macOS does not provide native loopback capture. 

Apple introduced ScreenCaptureKit in macOS 12.3, which provides an official API for capturing system audio. This API requires Screen Recording permission, which is displayed in System Preferences > Privacy & Security.

Reference implementations exist in `src-tauri/src/platform/windows/wasapi.rs` (1120 lines) and `src-tauri/src/platform/linux/pipewire.rs` (709 lines).

## Goals / Non-Goals

**Goals:**
- Enable system audio enumeration on macOS via ScreenCaptureKit
- Support concurrent capture from input and system audio sources
- Port `AudioMixer` and `MultiCaptureManager` patterns from Windows implementation
- Integrate AEC3 echo cancellation for macOS
- Maintain the same API contract as Linux/Windows backends

**Non-Goals:**
- Supporting macOS versions prior to 12.3 (Monterey)
- Bundling virtual audio drivers (e.g., BlackHole)
- Per-application audio capture (capture all system audio only)
- Hardware-accelerated resampling via AudioConverter (linear interpolation is sufficient)

## Decisions

### Decision 1: Use ScreenCaptureKit for System Audio
**What:** Use Apple's ScreenCaptureKit API to capture system audio.
**Why:** 
- Official Apple API, future-proof and stable
- Does not require third-party virtual audio drivers
- Can exclude the app's own audio output to prevent feedback loops
- Works on both Intel and Apple Silicon

**Alternatives considered:**
- Aggregate Device approach: Requires user to manually configure in Audio MIDI Setup, poor UX
- Virtual audio driver (BlackHole): Complex installation, security concerns, maintenance burden

### Decision 2: Thread-per-Stream Architecture
**What:** Create separate threads for CoreAudio input capture and ScreenCaptureKit system audio capture.
**Why:**
- CoreAudio uses callbacks on the audio thread with strict timing requirements
- ScreenCaptureKit uses its own callback mechanism
- Matches the Windows WASAPI architecture which works well
- `mpsc` channels provide safe communication between threads

### Decision 3: Port AudioMixer from Windows
**What:** Port the `AudioMixer` struct and `MultiCaptureManager` pattern from the Windows implementation.
**Why:**
- Windows implementation is battle-tested and handles all edge cases
- Render-first AEC processing pattern is correct for echo cancellation
- Frame-based mixing (10ms at 48kHz) matches AEC3 requirements

### Decision 4: Objective-C Interop via objc2
**What:** Use the `objc2` crate for ScreenCaptureKit interop from Rust.
**Why:**
- Pure Rust solution, no Swift bridge library needed
- `objc2` provides safe and ergonomic Objective-C bindings
- Avoids build complexity of mixed Swift/Rust projects

**Alternatives considered:**
- Swift bridge library: Additional build complexity, cross-language debugging challenges
- `objc` crate: Older, less safe than `objc2`

### Decision 5: Graceful Permission Handling
**What:** Check for Screen Recording permission before attempting system audio capture and provide clear feedback to users.
**Why:**
- ScreenCaptureKit requires explicit user permission
- Attempting capture without permission results in silent failure or empty audio
- Clear messaging improves user experience

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| ScreenCaptureKit Rust bindings may be incomplete | Use `objc2` for direct Objective-C interop; create minimal bindings for required APIs only |
| Screen Recording permission is intrusive | Clear messaging explaining why permission is needed; graceful degradation when denied |
| Different behavior between Intel and Apple Silicon | Test on both architectures; ScreenCaptureKit abstracts hardware differences |
| Thread synchronization complexity | Use established patterns from Windows implementation; mpsc channels for safety |
| AEC3 may have different performance on macOS | AEC3 is pure Rust, should work identically; test with real-world scenarios |

## Module Structure

```
src-tauri/src/platform/macos/
├── mod.rs                  # Module exports
├── coreaudio.rs           # CoreAudioBackend (updated with mixer and multi-source)
├── screencapturekit.rs    # ScreenCaptureKit system audio capture (new)
└── stub.rs                # Legacy stub (can be removed)
```

## Data Flow

```
┌─────────────────┐      ┌─────────────────┐
│   CoreAudio     │      │ScreenCaptureKit │
│   AudioUnit     │      │    Stream       │
│  (Input)        │      │  (System Audio) │
└────────┬────────┘      └────────┬────────┘
         │                        │
         ▼                        ▼
    ┌────────┐              ┌────────┐
    │ Stream │              │ Stream │
    │ Thread │              │ Thread │
    └────┬───┘              └────┬───┘
         │                        │
         │  StreamSamples         │  StreamSamples
         │  (is_loopback=false)   │  (is_loopback=true)
         ▼                        ▼
      ┌──────────────────────────────┐
      │        Main Thread           │
      │     ┌────────────────┐       │
      │     │   AudioMixer   │       │
      │     │  (AEC + Mix)   │       │
      │     └───────┬────────┘       │
      └─────────────┼────────────────┘
                    │
                    ▼
              AudioSamples
                    │
                    ▼
           ┌───────────────┐
           │   Processor   │
           │   Pipeline    │
           └───────────────┘
```

## Migration Plan

1. Add dependencies: `objc2`, ScreenCaptureKit framework bindings, `aec3` for macOS
2. Create `screencapturekit.rs` module for system audio capture
3. Create `MultiCaptureManager` and `AudioMixer` in `coreaudio.rs`
4. Update `list_system_devices()` to enumerate via ScreenCaptureKit
5. Update `start_capture_sources()` to handle dual-source capture
6. Test on macOS 12.3+ with various hardware configurations
7. Update spec to reflect full macOS support

No rollback needed as this is additive functionality.

## Open Questions

1. **ScreenCaptureKit permission flow**: Should the app proactively request permission on first launch, or wait until user selects a system audio source?
   - Proposed: Request when user first selects "System" or "Mixed" source type

2. **Minimum macOS version**: Should we support older macOS with fallback (empty system devices) or require 12.3+?
   - Proposed: Require 12.3+ for system audio; gracefully show no system devices on older versions

3. **Audio exclusion**: Should the app exclude its own audio output from system capture?
   - Proposed: Yes, use `excludesCurrentProcessAudio` to prevent feedback loops
