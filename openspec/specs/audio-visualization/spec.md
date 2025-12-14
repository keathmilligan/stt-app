# audio-visualization Specification

## Purpose
TBD - created by archiving change add-live-audio-waveform. Update Purpose after archive.
## Requirements
### Requirement: Live Waveform Display
The system SHALL display a real-time waveform visualization of audio input in a dedicated canvas area.

#### Scenario: Waveform renders during monitoring
- **WHEN** the user starts audio monitoring
- **THEN** a scrolling waveform appears showing audio amplitude over time, updating at 60fps

#### Scenario: Waveform scrolls right to left
- **WHEN** new audio samples arrive
- **THEN** they appear on the right edge of the display and scroll leftward as newer samples arrive

#### Scenario: Waveform renders during recording
- **WHEN** the user is recording audio
- **THEN** the waveform visualization is active, showing the audio being captured

#### Scenario: Waveform clears on stop when not monitoring
- **WHEN** recording stops and monitoring was not active before recording started
- **THEN** the waveform display shows an idle state (flat line or cleared canvas)

#### Scenario: Waveform continues on stop when monitoring was active
- **WHEN** recording stops and monitoring was active before recording started
- **THEN** the waveform continues displaying live audio input

### Requirement: Audio Monitor Mode
The system SHALL allow the user to monitor audio input without recording, for verifying microphone function.

#### Scenario: Start monitoring
- **WHEN** the user clicks the Monitor button while idle
- **THEN** audio streaming begins, the waveform displays live input, and no audio is accumulated for transcription

#### Scenario: Stop monitoring
- **WHEN** the user clicks the Monitor button while monitoring
- **THEN** audio streaming stops and the waveform returns to idle state

#### Scenario: Recording starts while monitoring
- **WHEN** the user clicks Record while monitoring
- **THEN** recording begins seamlessly without disrupting the audio stream, and the waveform continues uninterrupted

#### Scenario: Recording stops while monitoring was active
- **WHEN** the user stops recording and monitoring was active before recording started
- **THEN** monitoring continues uninterrupted and the waveform keeps displaying live audio

### Requirement: Low-Latency Audio Streaming
The system SHALL stream render-ready visualization data from the backend to the frontend with minimal latency for real-time display.

#### Scenario: Visualization data delivered via events
- **WHEN** audio is being captured (monitoring or recording)
- **THEN** pre-computed visualization data is emitted to the frontend in batches via Tauri events containing waveform amplitudes and spectrogram colors

#### Scenario: Visualization latency is imperceptible
- **WHEN** audio input occurs (e.g., user taps microphone)
- **THEN** the waveform and spectrogram reflect the input within one display frame (~16ms), appearing instantaneous to the user

#### Scenario: Stop recording does not disrupt waveform
- **WHEN** the user stops recording while monitoring was active
- **THEN** the waveform and spectrogram continue displaying without any visual disruption or pause

### Requirement: Non-Blocking Transcription
The system SHALL process and transcribe recorded audio in the background without blocking the UI or audio stream.

#### Scenario: Recording stops immediately
- **WHEN** the user clicks stop recording
- **THEN** the recording stops immediately and UI is responsive

#### Scenario: Transcription runs in background
- **WHEN** recording stops
- **THEN** audio processing and transcription run in a background thread

#### Scenario: Transcription results delivered via events
- **WHEN** transcription completes
- **THEN** the result is delivered to the frontend via a Tauri event

### Requirement: Spectrogram Display
The system SHALL display a real-time spectrogram visualization below the waveform, showing frequency content of audio input over time.

#### Scenario: Spectrogram renders during monitoring
- **WHEN** the user starts audio monitoring
- **THEN** a scrolling spectrogram appears showing frequency content over time, updating at 60fps

#### Scenario: Spectrogram scrolls right to left
- **WHEN** new audio samples arrive and are analyzed
- **THEN** new frequency data appears on the right edge of the display and scrolls leftward as newer data arrives

#### Scenario: Spectrogram renders during recording
- **WHEN** the user is recording audio
- **THEN** the spectrogram visualization is active, showing the frequency content of audio being captured

#### Scenario: Spectrogram clears on stop when not monitoring
- **WHEN** recording stops and monitoring was not active before recording started
- **THEN** the spectrogram display shows an idle state (cleared canvas with background color)

#### Scenario: Spectrogram continues on stop when monitoring was active
- **WHEN** recording stops and monitoring was active before recording started
- **THEN** the spectrogram continues displaying live frequency content

### Requirement: FFT-Based Frequency Analysis
The system SHALL compute frequency content of audio samples using Fast Fourier Transform in the backend for spectrogram visualization.

#### Scenario: FFT window processing
- **WHEN** sufficient audio samples are buffered (512 samples)
- **THEN** the backend performs FFT analysis and extracts magnitude for each frequency bin

#### Scenario: Frequency bins mapped to colors
- **WHEN** FFT analysis completes
- **THEN** the backend maps frequency magnitudes to RGB colors and emits them as a spectrogram column ready for direct rendering

### Requirement: Spectrogram Color Mapping
The system SHALL map frequency magnitude values to colors using a heat map gradient in the backend for visual clarity.

#### Scenario: Low energy displayed as cool colors
- **WHEN** a frequency bin has low magnitude
- **THEN** the backend emits dark blue or black RGB values

#### Scenario: High energy displayed as warm colors
- **WHEN** a frequency bin has high magnitude
- **THEN** the backend emits yellow, orange, or red RGB values

#### Scenario: Color gradient is continuous
- **WHEN** frequency magnitudes span the range from low to high
- **THEN** colors transition smoothly through the gradient (blue -> cyan -> green -> yellow -> red)

### Requirement: Backend Waveform Processing
The system SHALL compute pre-downsampled waveform amplitude values in the backend, ready for direct rendering by the frontend.

#### Scenario: Waveform downsampling
- **WHEN** audio samples are captured
- **THEN** the backend downsamples them using peak detection to produce render-ready amplitude values

#### Scenario: Waveform data emitted with visualization events
- **WHEN** visualization data is emitted
- **THEN** waveform amplitudes are included in every event for continuous display updates

### Requirement: Unified Visualization Event
The system SHALL emit a single event type containing both waveform and spectrogram data to minimize IPC overhead.

#### Scenario: Combined payload structure
- **WHEN** visualization data is ready to emit
- **THEN** the system sends a `visualization-data` event containing waveform amplitudes and an optional spectrogram column

#### Scenario: Spectrogram column included when ready
- **WHEN** the FFT buffer fills (every 512 samples)
- **THEN** the visualization event includes a spectrogram column with RGB color data

#### Scenario: Waveform-only events between FFT frames
- **WHEN** visualization data is emitted before the FFT buffer is full
- **THEN** the event contains waveform data but no spectrogram column

