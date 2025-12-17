# Change: Replace whisper-rs with whisper.cpp prebuilt binaries

## Why

The `whisper-rs` crate depends on `whisper-rs-sys`, which requires compiling whisper.cpp from source. This fails on Windows due to build toolchain issues and adds complexity to the build process. Using prebuilt binaries from the official whisper.cpp GitHub releases eliminates these issues and simplifies cross-platform distribution.

## What Changes

- **BREAKING**: Remove `whisper-rs` crate dependency
- Add build-time binary download from whisper.cpp GitHub releases
- Create Rust FFI bindings to whisper.cpp shared library (DLL/dylib/so)
- Update `transcribe.rs` to use FFI bindings instead of whisper-rs API
- Platform-specific binary selection:
  - Windows x64: `whisper-bin-x64.zip`
  - Windows x86: `whisper-bin-Win32.zip`
  - macOS: `whisper-v{version}-xcframework.zip` (contains dylib)
  - Linux: Build from source (no prebuilt binaries available in releases)
- Bundle whisper.cpp shared library with application at build time

## Impact

- Affected specs: `speech-transcription`
- Affected code:
  - `src-tauri/Cargo.toml` - Remove whisper-rs, add libloading
  - `src-tauri/build.rs` - Add binary download logic
  - `src-tauri/src/transcribe.rs` - Rewrite to use FFI
  - New: `src-tauri/src/whisper_ffi.rs` - FFI bindings module
- Build process: Requires internet connection on first build to download binaries
- Distribution: whisper.dll/libwhisper.dylib bundled with app
