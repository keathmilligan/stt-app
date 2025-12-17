## MODIFIED Requirements

### Requirement: Windows Audio Backend (Stub)
The system SHALL provide a basic audio backend for Windows using WASAPI that supports input device enumeration and single-source capture. Advanced features (system audio capture, multiple sources, mixing, echo cancellation) SHALL remain stubbed until a future update.

#### Scenario: Windows backend compiles
- **WHEN** the application is compiled on Windows
- **THEN** compilation succeeds using the WASAPI backend

#### Scenario: Windows input device enumeration
- **WHEN** device enumeration is requested on Windows
- **THEN** available input devices (microphones) are returned with their names and IDs

#### Scenario: Windows single-source capture starts
- **WHEN** the user starts capture with a single input device selected on Windows
- **THEN** audio capture begins from the selected device and samples are delivered via the backend interface

#### Scenario: Windows single-source capture stops
- **WHEN** the user stops capture on Windows
- **THEN** audio capture stops and resources are released

#### Scenario: Windows backend provides consistent sample format
- **WHEN** the Windows backend delivers audio samples
- **THEN** samples are provided as stereo f32 interleaved format at 48kHz (resampled if device uses different rate)

#### Scenario: Windows system audio enumeration returns empty
- **WHEN** system audio device enumeration is requested on Windows
- **THEN** an empty device list is returned (loopback capture not yet implemented)

#### Scenario: Windows multi-source capture returns error
- **WHEN** the user attempts to start capture with two sources on Windows
- **THEN** the system returns an error indicating multi-source capture is not yet implemented
