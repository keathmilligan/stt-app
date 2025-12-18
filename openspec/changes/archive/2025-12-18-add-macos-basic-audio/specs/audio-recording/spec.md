## MODIFIED Requirements

### Requirement: macOS Audio Backend (Stub)
The system SHALL provide a functional audio backend for macOS using CoreAudio that supports input device enumeration and single-source capture. System audio capture, multi-source capture, and echo cancellation are not yet supported and will return appropriate errors.

#### Scenario: macOS backend compiles
- **WHEN** the application is compiled on macOS
- **THEN** compilation succeeds using the CoreAudio backend

#### Scenario: macOS input device enumeration
- **WHEN** input device enumeration is requested on macOS
- **THEN** available input devices (microphones) are returned with their names and IDs

#### Scenario: macOS single-source capture starts
- **WHEN** the user starts capture with a single input device selected on macOS
- **THEN** audio capture begins from the selected device and samples are delivered via the backend interface

#### Scenario: macOS single-source capture stops
- **WHEN** the user stops capture on macOS
- **THEN** audio capture stops and resources are released

#### Scenario: macOS backend provides consistent sample format
- **WHEN** the macOS backend delivers audio samples
- **THEN** samples are provided as stereo f32 interleaved format at 48kHz (resampled if device uses different rate)

#### Scenario: macOS system audio returns empty list
- **WHEN** system audio device enumeration is requested on macOS
- **THEN** an empty device list is returned (not yet implemented)

#### Scenario: macOS multi-source capture returns error
- **WHEN** the user attempts to start capture with both input and system audio sources on macOS
- **THEN** the system returns an error indicating multi-source capture is not yet implemented

#### Scenario: macOS format conversion
- **WHEN** the input device provides audio in a format other than f32 stereo at 48kHz
- **THEN** the audio is converted to f32 stereo at 48kHz before delivery

#### Scenario: macOS mono input handling
- **WHEN** the input device provides mono audio
- **THEN** the audio is converted to stereo by duplicating samples
