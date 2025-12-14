## MODIFIED Requirements

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

## ADDED Requirements

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
