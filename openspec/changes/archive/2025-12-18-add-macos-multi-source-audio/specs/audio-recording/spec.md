## MODIFIED Requirements

### Requirement: macOS Audio Backend (Stub)
The system SHALL provide a fully functional audio backend for macOS using CoreAudio for input capture and ScreenCaptureKit for system audio capture. The backend supports input device enumeration, system audio enumeration, single-source capture, multi-source capture with mixing, and echo cancellation, achieving feature parity with Linux and Windows backends.

#### Scenario: macOS backend compiles
- **WHEN** the application is compiled on macOS
- **THEN** compilation succeeds using the CoreAudio and ScreenCaptureKit backend

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

#### Scenario: macOS format conversion
- **WHEN** the input device provides audio in a format other than f32 stereo at 48kHz
- **THEN** the audio is converted to f32 stereo at 48kHz before delivery

#### Scenario: macOS mono input handling
- **WHEN** the input device provides mono audio
- **THEN** the audio is converted to stereo by duplicating samples

#### Scenario: macOS system audio enumeration
- **WHEN** system audio device enumeration is requested on macOS 12.3+
- **THEN** available system audio outputs are returned via ScreenCaptureKit with user-friendly names

#### Scenario: macOS system audio enumeration on older macOS
- **WHEN** system audio device enumeration is requested on macOS versions prior to 12.3
- **THEN** an empty device list is returned (ScreenCaptureKit not available)

#### Scenario: macOS system audio capture starts
- **WHEN** the user starts capture with a system audio source selected on macOS
- **THEN** audio capture begins from the system audio output using ScreenCaptureKit

#### Scenario: macOS system audio requires permission
- **WHEN** the user attempts to capture system audio without Screen Recording permission
- **THEN** the system returns an error indicating Screen Recording permission is required

#### Scenario: macOS multi-source capture starts
- **WHEN** the user starts capture with both an input device and system audio source on macOS
- **THEN** audio capture begins from both sources simultaneously using separate capture threads

#### Scenario: macOS multi-source audio mixing
- **WHEN** capturing from both input and system sources on macOS
- **THEN** samples from both sources are mixed using frame-based processing (10ms frames at 48kHz)

#### Scenario: macOS echo cancellation applied
- **WHEN** capturing from both sources on macOS with echo cancellation enabled
- **THEN** the AEC3 algorithm is applied to the microphone signal using system audio as reference

#### Scenario: macOS recording mode Mixed
- **WHEN** capturing from both sources on macOS in Mixed recording mode
- **THEN** echo-cancelled microphone and system audio are combined with soft clipping to prevent distortion

#### Scenario: macOS recording mode EchoCancel
- **WHEN** capturing from both sources on macOS in EchoCancel recording mode
- **THEN** only the echo-cancelled microphone signal is output (no system audio in output)

#### Scenario: macOS excludes app audio from system capture
- **WHEN** capturing system audio on macOS
- **THEN** the application's own audio output is excluded from the captured audio to prevent feedback loops

## ADDED Requirements

### Requirement: ScreenCaptureKit Permission Handling
The system SHALL handle ScreenCaptureKit permission requirements on macOS gracefully, providing clear feedback to users when permission is needed or denied.

#### Scenario: Permission check before system audio capture
- **WHEN** the user attempts to capture system audio on macOS
- **THEN** the system verifies Screen Recording permission status before starting capture

#### Scenario: Permission denied feedback
- **WHEN** Screen Recording permission is denied or not granted
- **THEN** the system returns a clear error message indicating permission is required and where to enable it

#### Scenario: Permission granted allows capture
- **WHEN** Screen Recording permission has been granted
- **THEN** system audio capture proceeds normally via ScreenCaptureKit

#### Scenario: No permission prompt on input-only capture
- **WHEN** the user captures only from an input device (microphone)
- **THEN** no Screen Recording permission is requested (only Microphone permission applies)

### Requirement: macOS Minimum Version for System Audio
The system SHALL require macOS 12.3 or later for system audio capture functionality, as ScreenCaptureKit is not available on earlier versions.

#### Scenario: macOS version check for system audio
- **WHEN** the application starts on macOS
- **THEN** system audio features are enabled only if running on macOS 12.3 or later

#### Scenario: Older macOS shows no system devices
- **WHEN** running on macOS versions prior to 12.3
- **THEN** the system audio device list is empty and Mixed source type shows only input device

#### Scenario: Feature availability indication
- **WHEN** system audio features are unavailable due to macOS version
- **THEN** the UI indicates that system audio requires macOS 12.3 or later
