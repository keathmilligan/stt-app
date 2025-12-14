# Design: Migrate Visualization Processing to Backend

## Context
The application displays real-time waveform and spectrogram visualizations during audio monitoring/recording. Currently:
- Raw audio samples are emitted from Rust to the frontend via `audio-samples` events
- Frontend performs FFT (512-point), downsampling, and color mapping in JavaScript
- Frontend maintains ring buffers and handles 60fps rendering

This proposal moves the computational analysis to Rust while keeping the frontend responsible for buffering and smooth rendering.

## Goals / Non-Goals

**Goals:**
- Consolidate all audio processing in the Rust backend
- Emit render-ready data: pre-downsampled waveform amplitudes and RGB spectrogram colors
- Maintain identical visual output and 60fps rendering
- Reduce JS main thread CPU usage

**Non-Goals:**
- Change the visual appearance of waveform or spectrogram
- Move buffering/timing logic to backend (frontend handles smooth rendering)
- Add new visualization features

## Decisions

### Decision 1: Event Payload Structure

**Choice**: Single event type `visualization-data` containing both waveform and spectrogram data.

```rust
struct VisualizationPayload {
    // Pre-downsampled waveform amplitudes (e.g., 64-128 points per emit)
    waveform: Vec<f32>,
    // Spectrogram column: RGB values for each frequency bin
    spectrogram: Option<SpectrogramColumn>,
}

struct SpectrogramColumn {
    // RGB triplets for each frequency bin (256 bins = 768 bytes)
    colors: Vec<u8>,
}
```

**Rationale**: 
- Combined event reduces IPC overhead vs separate events
- Spectrogram is `Option` because FFT only produces output every 512 samples
- Waveform data flows continuously for smooth display

**Alternatives considered:**
- Separate events for waveform and spectrogram: More IPC overhead, timing complexity
- Include full frame buffer: Too much data, frontend already handles buffering well

### Decision 2: Waveform Downsampling Strategy

**Choice**: Emit ~128 amplitude samples per batch, representing peak values within each window.

**Rationale**:
- Current frontend emits ~256 raw samples, then downsamples during render
- Pre-downsampling to ~128 points reduces payload while maintaining visual fidelity
- Peak detection (max absolute value per window) preserves transients better than averaging

### Decision 3: FFT and Color Mapping in Rust

**Choice**: Use `rustfft` crate for FFT computation, implement color lookup table matching current frontend gradient.

**Rationale**:
- `rustfft` is mature, fast, and already suitable for real-time audio
- Color mapping is a simple gradient calculation; pre-computing in Rust avoids per-pixel JS overhead
- Must exactly match current colors: dark blue -> cyan -> yellow -> red

### Decision 4: Spectrogram Frequency Binning

**Choice**: Output 256 frequency bins with logarithmic scaling applied in backend.

**Rationale**:
- Current frontend uses 256 bins from 512-point FFT
- Log scaling (matching current `positionToFreqBinFloat` logic) ensures consistent appearance
- Backend computes final pixel colors; frontend just renders the column

### Decision 5: Emit Rate and Batching

**Choice**: Emit visualization data at same rate as current `audio-samples` (~256 samples worth), with spectrogram column included when FFT buffer fills.

**Rationale**:
- Maintains current low-latency behavior (~5ms batches at 48kHz)
- Frontend timing/buffering logic unchanged
- Spectrogram updates ~10-11ms (512 samples at 48kHz), matching current behavior

## Component Architecture

```
Audio Input (cpal)
       |
       v
+------------------+
| audio.rs         |
| - capture        |
| - mono conversion|
+------------------+
       |
       v
+----------------------+
| VisualizationProc    |  (NEW)
| - downsample waveform|
| - FFT + color map    |
+----------------------+
       |
       v
  Tauri Event: visualization-data
       |
       v
+------------------+
| main.ts          |
| - buffer data    |
| - render at 60fps|
+------------------+
```

## Data Flow

1. Audio callback receives ~256 samples
2. `VisualizationProcessor` accumulates samples:
   - Downsamples for waveform output (peak detection)
   - Buffers for FFT (512 samples)
3. When FFT buffer full:
   - Compute FFT magnitudes
   - Apply log frequency scaling
   - Map to RGB colors via lookup table
4. Emit `visualization-data` event with waveform + optional spectrogram column
5. Frontend receives data:
   - Push waveform samples to ring buffer
   - Push spectrogram column to scrolling image buffer
   - Render at 60fps via requestAnimationFrame

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Color mismatch between old/new | Extract exact color gradient from frontend, implement identical LUT |
| Increased event payload size | RGB data is ~768 bytes per column; acceptable for IPC |
| FFT library adds dependency | `rustfft` is lightweight, well-maintained |
| Timing changes affect smoothness | Keep emit rate identical; frontend buffering unchanged |

## Migration Path

1. Add `VisualizationProcessor` to backend (new code, no changes to existing)
2. Add `visualization-data` event emission alongside existing `audio-samples`
3. Update frontend to consume new event format
4. Remove frontend FFT/downsampling code
5. Remove `audio-samples` event (no longer needed for visualization)

## Open Questions

None - scope is well-defined.
