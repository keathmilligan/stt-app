# Change: Enhance Speech Detection Algorithm

## Why
The current speech detector relies solely on RMS amplitude thresholding, which causes false positives from non-speech audio like keyboard clicks, mouse sounds, and ambient noise. Accurate speech detection is critical to the product's core goals. The detector must reliably capture all speech including soft/whispered speech while rejecting non-speech sounds.

## What Changes
- Add Zero-Crossing Rate (ZCR) analysis to distinguish voiced speech from impulsive/transient sounds
- Add spectral centroid estimation to identify speech-like frequency content
- Implement dual-mode detection: **voiced mode** for normal speech and **whisper mode** for soft/breathy speech
- Add explicit transient rejection for keyboard clicks and similar impulsive sounds (ZCR > 0.40 AND centroid > 5500 Hz)
- Lower amplitude threshold to -50dB to capture whispered speech, with feature validation preventing noise triggers
- Use mode-specific onset times (100ms voiced, 150ms whisper) to balance responsiveness with false-positive rejection
- Retain existing hold time debouncing for speech-end detection

## Impact
- Affected specs: `audio-processing`
- Affected code: `src-tauri/src/processor.rs` (SpeechDetector implementation)
- Performance: All added features are O(n) per sample buffer with no FFT required, maintaining real-time capability
- No breaking changes to existing API or event payloads
