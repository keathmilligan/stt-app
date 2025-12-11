# Design: Live Audio Waveform Visualization

## Context
The application currently captures audio for transcription but provides no visual feedback during recording. Users cannot verify their microphone is working until after transcription completes. A live waveform addresses this by showing real-time audio input.

**Key constraint**: The user requires "very low latency" - targeting the best achievable performance, ideally <16ms visual latency for 60fps rendering.

## Goals / Non-Goals
- **Goals**:
  - Real-time waveform visualization with minimal perceived latency
  - Monitor mode to preview audio without recording
  - Smooth scrolling display (right-to-left) at 60fps
  - Simple, maintainable implementation

- **Non-Goals**:
  - Frequency/spectrogram visualization (waveform only)
  - Audio level meters or peak indicators
  - Waveform customization (colors, scale, etc.)

## Decisions

### 1. Data Flow: Tauri Events for Streaming
**Decision**: Use Tauri's event system to stream audio samples from Rust to the frontend.

**Rationale**: 
- Tauri commands are request/response - unsuitable for continuous streaming
- Events allow push-based delivery with minimal overhead
- Events can be emitted from the audio callback thread with buffering

**Alternatives considered**:
- Polling via commands: Higher latency, wastes CPU cycles
- WebSocket: Unnecessary complexity for same-process communication
- Shared memory: Complex, not supported by Tauri directly

### 2. Buffering Strategy: Small Chunks at High Frequency
**Decision**: Emit audio chunks every ~16ms (matching 60fps) containing ~256-512 samples.

**Rationale**:
- Balances latency vs event overhead
- 16ms chunks align with display refresh rate
- Small enough to maintain responsive feel

**Trade-offs**:
- More events than larger buffers, but necessary for low latency
- May need adjustment based on audio callback buffer sizes

### 3. Rendering: Canvas 2D with requestAnimationFrame
**Decision**: Use HTML Canvas 2D API with requestAnimationFrame for rendering.

**Rationale**:
- Canvas 2D is sufficient for waveform rendering (simple line drawing)
- requestAnimationFrame ensures vsync-aligned updates
- No external dependencies required
- Well-supported across all platforms

**Alternatives considered**:
- WebGL: Overkill for simple waveform, adds complexity
- SVG: Poor performance for real-time updates with many points
- CSS animations: Not suitable for dynamic data visualization

### 4. Waveform Buffer: Ring Buffer in Frontend
**Decision**: Maintain a fixed-size ring buffer in TypeScript to store visible waveform data.

**Rationale**:
- Constant memory usage regardless of recording duration
- O(1) append and implicit "scroll" by overwriting old data
- Buffer size determines visible time window (e.g., 2-3 seconds)

### 5. Monitor vs Recording Mode
**Decision**: Separate "Monitor" button that starts audio streaming without accumulating for transcription.

**Rationale**:
- Clear user intent separation
- Monitor can run indefinitely without memory growth
- Recording automatically enables visualization (no separate toggle needed)

**State transitions**:
```
Idle -> [Monitor] -> Monitoring (visualization active, no accumulation)
Idle -> [Record] -> Recording (visualization active, accumulating)
Monitoring -> [Stop Monitor] -> Idle
Monitoring -> [Record] -> Recording (stream continues, now also accumulating)
Recording -> [Stop] -> Monitoring (if was monitoring before) or Idle
Recording -> [Stop] -> Background transcription starts (non-blocking)
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Audio callback may block on event emission | Use try_lock(); drop samples if lock unavailable rather than block |
| High event frequency may impact performance | Batch samples into chunks; monitor and adjust chunk size if needed |
| Canvas rendering may lag on slow devices | Keep draw operations simple; consider reducing point density if needed |
| Stopping recording blocks audio stream | Split stop_recording into fast extraction + async processing; transcription runs in background thread |

## Implementation Notes

### Rust Side
- Single `AudioStreamState` manages both recording and monitoring flags
- Stream starts once and continues as long as either flag is true
- `stop_recording` extracts samples quickly without blocking, returns `RawRecordedAudio`
- `process_recorded_audio` handles CPU-intensive resampling (called in background thread)
- Transcription runs in spawned thread, emits events on completion/error
- Use `try_lock()` in audio callback to avoid blocking

### TypeScript Side
- Ring buffer class with fixed capacity (e.g., 8192 samples)
- On event: push samples to ring buffer
- On requestAnimationFrame: draw ring buffer contents to canvas
- Downsample for drawing if buffer has more samples than canvas pixels
- Listen for `transcription-complete` and `transcription-error` events
- `stop_recording` returns immediately; UI updates via events

## Open Questions
- None - ready for implementation
