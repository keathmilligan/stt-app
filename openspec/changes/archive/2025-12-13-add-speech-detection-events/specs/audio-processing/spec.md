## ADDED Requirements

### Requirement: Speech Detection Events
The system SHALL emit events when speech activity transitions occur, indicating when the user starts and stops speaking.

#### Scenario: Speech starts
- **WHEN** audio amplitude rises above the detection threshold after a period of silence
- **THEN** the system emits a `speech-started` event to the frontend

#### Scenario: Speech ends after hold time
- **WHEN** audio amplitude falls below the detection threshold and remains below for the configured hold time (default 300ms)
- **THEN** the system emits a `speech-ended` event to the frontend

#### Scenario: Brief pause during speech
- **WHEN** audio amplitude briefly falls below threshold but returns above threshold before hold time elapses
- **THEN** no `speech-ended` event is emitted (debouncing prevents false triggers)

#### Scenario: Processing disabled
- **WHEN** voice processing is disabled via toggle
- **THEN** no speech detection events are emitted

### Requirement: Configurable Speech Detection Parameters
The system SHALL allow configuration of speech detection sensitivity through threshold and hold time parameters.

#### Scenario: Default parameters
- **WHEN** the speech detector is created without explicit configuration
- **THEN** it uses default threshold (-40dB) and hold time (300ms)

#### Scenario: Custom threshold
- **WHEN** a custom threshold is configured
- **THEN** speech detection uses the specified threshold for amplitude comparison

## MODIFIED Requirements

### Requirement: Extensible Audio Processor Architecture
The system SHALL provide a trait-based architecture for audio processors, allowing new processor types to be added without modifying the core audio pipeline. Processors MAY emit events to the frontend via an `AppHandle` parameter.

#### Scenario: Processor receives samples during monitoring
- **WHEN** monitoring is active and voice processing is enabled
- **THEN** the active processor receives audio samples and an AppHandle reference in the audio callback

#### Scenario: Processor executes without blocking
- **WHEN** a processor processes samples
- **THEN** processing completes within the audio callback without causing audio dropouts

#### Scenario: Processor emits event
- **WHEN** a processor determines an event should be emitted
- **THEN** the processor uses the provided AppHandle to emit the event to the frontend
