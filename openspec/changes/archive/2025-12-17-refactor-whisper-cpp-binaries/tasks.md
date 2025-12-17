# Tasks: whisper.cpp Binary Integration

## 1. Build Infrastructure

- [x] 1.1 Update `Cargo.toml` to add `libloading` crate for dynamic library loading
- [x] 1.2 Update `Cargo.toml` to make `whisper-rs` a Linux-only dependency via target cfg
- [x] 1.3 Add `reqwest` with `blocking` feature to build-dependencies for binary download
- [x] 1.4 Add `zip` crate to build-dependencies for archive extraction

## 2. Build Script Binary Download

- [x] 2.1 Create binary download function in `build.rs` that fetches from GitHub releases
- [x] 2.2 Implement platform detection (Windows x64/x86, macOS, Linux)
- [x] 2.3 Implement download caching to avoid repeated downloads
- [x] 2.4 Implement ZIP extraction for Windows binaries
- [x] 2.5 Implement xcframework extraction for macOS (locate correct architecture dylib)
- [x] 2.6 Copy shared library to appropriate output location
- [x] 2.7 Set cargo link directives for library discovery

## 3. FFI Bindings

- [x] 3.1 Create `src/whisper_ffi.rs` module with C function declarations
- [x] 3.2 Implement safe Rust wrappers for whisper context creation/destruction
- [x] 3.3 Implement safe wrapper for `whisper_full` transcription function
- [x] 3.4 Implement safe wrapper for segment iteration and text extraction
- [x] 3.5 Implement `Drop` for automatic resource cleanup

## 4. Transcriber Refactor

- [x] 4.1 Update `transcribe.rs` to use conditional compilation (`cfg(target_os)`)
- [x] 4.2 Implement FFI-based `Transcriber` for Windows/macOS
- [x] 4.3 Keep existing whisper-rs `Transcriber` for Linux
- [x] 4.4 Ensure both implementations expose identical public API
- [x] 4.5 Update library loading to find bundled DLL/dylib at runtime

## 5. Distribution Setup

- [x] 5.1 Update `tauri.conf.json` to bundle whisper shared library with application
- [x] 5.2 Configure library search path for runtime loading
- [x] 5.3 Test that bundled library is found on application launch

## 6. Validation

- [x] 6.1 Test build on Windows (primary target platform)
- [ ] 6.2 Test transcription functionality on Windows
- [ ] 6.3 Verify Linux build still works with whisper-rs
- [ ] 6.4 Verify Linux transcription functionality
- [ ] 6.5 Document build requirements and offline build process
