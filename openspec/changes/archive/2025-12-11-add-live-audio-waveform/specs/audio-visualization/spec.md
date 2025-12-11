## ADDED Requirements

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
The system SHALL stream audio samples from the backend to the frontend with minimal latency for real-time visualization.

#### Scenario: Audio samples delivered via events
- **WHEN** audio is being captured (monitoring or recording)
- **THEN** samples are emitted to the frontend in small batches (~256 samples) via Tauri events

#### Scenario: Visualization latency is imperceptible
- **WHEN** audio input occurs (e.g., user taps microphone)
- **THEN** the waveform reflects the input within one display frame (~16ms), appearing instantaneous to the user

#### Scenario: Stop recording does not disrupt waveform
- **WHEN** the user stops recording while monitoring was active
- **THEN** the waveform continues displaying without any visual disruption or pause

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
