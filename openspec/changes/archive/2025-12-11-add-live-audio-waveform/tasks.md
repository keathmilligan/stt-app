# Tasks: Add Live Audio Waveform Visualization

## 1. Backend Audio Streaming
- [x] 1.1 Add event emission infrastructure to `audio.rs` (AppHandle storage, event types)
- [x] 1.2 Implement sample buffering in audio callback with chunk emission (~16ms batches)
- [x] 1.3 Add `start_monitor` command that streams audio without accumulating samples
- [x] 1.4 Add `stop_monitor` command to stop monitoring
- [x] 1.5 Modify `start_recording` to also emit visualization events
- [x] 1.6 Register new commands in `lib.rs`

## 2. Frontend Waveform Rendering
- [x] 2.1 Create ring buffer utility class for storing waveform samples
- [x] 2.2 Implement canvas waveform renderer with requestAnimationFrame loop
- [x] 2.3 Add Tauri event listener for audio sample events
- [x] 2.4 Connect event data to ring buffer and trigger renders

## 3. UI Integration
- [x] 3.1 Add canvas element and monitor button to `index.html`
- [x] 3.2 Add styles for waveform display area in `styles.css`
- [x] 3.3 Implement monitor button click handler with state management
- [x] 3.4 Update record button to work alongside monitor state (auto-stop monitor on record, or transition)
- [x] 3.5 Handle cleanup on stop (clear canvas or show idle state)

## 4. Validation
- [ ] 4.1 Test monitor mode starts/stops correctly without affecting recording
- [ ] 4.2 Test recording mode shows visualization while accumulating audio
- [ ] 4.3 Verify waveform updates smoothly at 60fps without dropped frames
- [ ] 4.4 Verify latency is perceptually instant (tap test - visual response to audio input)
- [ ] 4.5 Test with different audio devices and sample rates
