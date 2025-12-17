## MODIFIED Requirements

### Requirement: Local Whisper Transcription
The system SHALL transcribe recorded audio to text using a local Whisper model. On Windows and macOS, transcription uses the whisper.cpp shared library loaded via FFI. On Linux, transcription uses the whisper-rs crate.

#### Scenario: Successful transcription
- **WHEN** recording stops and audio data is available
- **THEN** the audio is transcribed and the resulting text is displayed in the UI

#### Scenario: Transcription in progress
- **WHEN** transcription is processing
- **THEN** the UI displays a loading indicator

#### Scenario: Windows/macOS library loading
- **WHEN** transcription is requested on Windows or macOS
- **THEN** the whisper.cpp shared library (whisper.dll or libwhisper.dylib) is loaded from the application bundle

#### Scenario: Linux transcription
- **WHEN** transcription is requested on Linux
- **THEN** transcription is performed using the whisper-rs crate

## ADDED Requirements

### Requirement: Whisper Library Bundling
The system SHALL bundle the whisper.cpp shared library with the application on Windows and macOS. The library SHALL be downloaded from the official whisper.cpp GitHub releases during the build process.

#### Scenario: Build downloads library
- **WHEN** the application is built on Windows or macOS
- **THEN** the build process downloads the appropriate whisper.cpp binary from GitHub releases if not already cached

#### Scenario: Library bundled with application
- **WHEN** the application is packaged for distribution
- **THEN** the whisper.dll (Windows) or libwhisper.dylib (macOS) is included in the application bundle

#### Scenario: Cached binary reused
- **WHEN** building and the whisper.cpp binary for the target version already exists in the build cache
- **THEN** the cached binary is used without re-downloading

#### Scenario: Download failure handling
- **WHEN** the build process cannot download the whisper.cpp binary (network error, GitHub unavailable)
- **THEN** the build fails with a clear error message indicating the download failure

### Requirement: Platform-Specific Binary Selection
The build system SHALL select the correct whisper.cpp binary based on the target platform and architecture.

#### Scenario: Windows x64 build
- **WHEN** building for Windows x64
- **THEN** the `whisper-bin-x64.zip` binary is downloaded and whisper.dll is extracted

#### Scenario: Windows x86 build
- **WHEN** building for Windows x86
- **THEN** the `whisper-bin-Win32.zip` binary is downloaded and whisper.dll is extracted

#### Scenario: macOS build
- **WHEN** building for macOS
- **THEN** the `whisper-v{version}-xcframework.zip` is downloaded and the correct architecture dylib is extracted

#### Scenario: Linux build
- **WHEN** building for Linux
- **THEN** no binary download occurs; the whisper-rs crate builds whisper.cpp from source
