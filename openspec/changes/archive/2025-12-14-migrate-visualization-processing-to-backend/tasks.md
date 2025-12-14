## 1. Backend: Add Visualization Processor

- [x] 1.1 Add `rustfft` dependency to `Cargo.toml`
- [x] 1.2 Create `VisualizationProcessor` struct in `processor.rs` implementing `AudioProcessor` trait
- [x] 1.3 Implement waveform downsampling with peak detection (~128 output samples per batch)
- [x] 1.4 Implement 512-point FFT with Hanning window
- [x] 1.5 Implement logarithmic frequency bin mapping (matching frontend's 20Hz-24kHz range)
- [x] 1.6 Implement color lookup table matching frontend gradient (dark blue -> cyan -> yellow -> red)
- [x] 1.7 Define `VisualizationPayload` and `SpectrogramColumn` structs with Serde serialization

## 2. Backend: Integrate Visualization Events

- [x] 2.1 Add `VisualizationProcessor` to audio stream callback alongside existing processors
- [x] 2.2 Emit `visualization-data` events with computed payload
- [x] 2.3 Ensure visualization processor runs when monitoring is active (independent of processing toggle)

## 3. Frontend: Update Renderers

- [x] 3.1 Add TypeScript types for `VisualizationPayload` and `SpectrogramColumn`
- [x] 3.2 Add event listener for `visualization-data` events
- [x] 3.3 Update `WaveformRenderer` to accept pre-downsampled amplitudes (remove internal downsampling)
- [x] 3.4 Update `SpectrogramRenderer` to accept RGB color data (remove FFT processing)
- [x] 3.5 Ensure ring buffer and scrolling behavior unchanged for smooth 60fps rendering

## 4. Cleanup

- [x] 4.1 Remove `FFTProcessor` class from `main.ts`
- [x] 4.2 Remove waveform downsampling logic from `WaveformRenderer.draw()`
- [x] 4.3 Remove `audio-samples` event emission from backend (replaced by `visualization-data`)
- [x] 4.4 Remove `audio-samples` event listener from frontend

## 5. Validation

- [x] 5.1 Verify waveform appearance matches previous implementation
- [x] 5.2 Verify spectrogram colors match previous implementation
- [x] 5.3 Verify 60fps rendering performance maintained
- [x] 5.4 Verify latency remains imperceptible (<16ms)
- [x] 5.5 Test monitoring start/stop behavior
- [x] 5.6 Test recording with monitoring active/inactive transitions
