# Change: Migrate Visualization Processing to Rust Backend

## Why
Audio analysis for waveform and spectrogram visualizations is currently performed in the TypeScript frontend (FFT, downsampling, color mapping). This creates architectural inconsistency where audio processing is split between backend (speech detection) and frontend (visualization), and places computational load on the JS main thread.

## What Changes
- **Backend**: Add visualization processor that computes render-ready waveform amplitudes and spectrogram colors
- **Backend**: Emit new event type with pre-computed visualization data instead of raw samples
- **Frontend**: Simplify renderers to consume pre-computed data; remove FFT and downsampling logic
- **Frontend**: Maintain existing buffering, timing, and rendering behavior for smooth 60fps display

## Impact
- Affected specs: `audio-visualization`
- Affected code:
  - `src-tauri/src/processor.rs` - New visualization processor
  - `src-tauri/src/audio.rs` - Emit visualization data events
  - `src/main.ts` - Simplify renderers, remove FFT/downsampling
