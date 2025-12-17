# Design: whisper.cpp Binary Integration

## Context

The current implementation uses the `whisper-rs` crate, which wraps `whisper-rs-sys` - a crate that compiles whisper.cpp from source. This approach has several issues:

1. **Windows build failures**: `whisper-rs-sys` requires CMake, a C++ compiler, and specific build configurations that often fail on Windows
2. **Build complexity**: Every developer needs a complete C++ toolchain
3. **Build time**: Compiling whisper.cpp from source adds significant build time

The whisper.cpp project provides prebuilt binaries for Windows and macOS in their GitHub releases, which can be downloaded and used directly.

## Goals

- Eliminate build-time compilation of whisper.cpp on Windows and macOS
- Maintain working transcription on Linux (which lacks prebuilt binaries)
- Minimize changes to the existing transcription API surface
- Keep the binary download process transparent and reproducible

## Non-Goals

- GPU/CUDA support (use CPU-only binaries for simplicity)
- Supporting multiple whisper.cpp versions simultaneously
- Automatic version updates

## Decisions

### Decision: Use dynamic linking via FFI

**What**: Load whisper.cpp as a shared library at runtime using Rust FFI bindings.

**Why**: 
- Prebuilt releases provide DLLs/dylibs, not static libraries
- Dynamic linking allows shipping the library alongside the application
- `libloading` crate provides safe cross-platform dynamic library loading

**Alternatives considered**:
- Static linking: Not possible with prebuilt release artifacts
- Keep whisper-rs: Doesn't solve the Windows build issues

### Decision: Download binaries in build.rs

**What**: The `build.rs` script downloads and extracts platform-appropriate binaries from GitHub releases at build time.

**Why**:
- Build scripts run before compilation, ensuring binaries are available
- Can cache downloads to avoid repeated network requests
- Integrates naturally with Cargo build process

**Alternatives considered**:
- Runtime download: Adds complexity, requires network at app start
- Git submodule: Still requires building from source
- Vendor binaries in repo: Bloats repository, version management issues

### Decision: Platform-specific binary selection

**What**: Select the correct binary based on `target_os` and `target_arch`:

| Platform | Binary | Library File |
|----------|--------|--------------|
| Windows x64 | `whisper-bin-x64.zip` | `whisper.dll` |
| Windows x86 | `whisper-bin-Win32.zip` | `whisper.dll` |
| macOS | `whisper-v{ver}-xcframework.zip` | `libwhisper.dylib` |
| Linux | Build from source via whisper-rs | N/A |

**Why**:
- Windows and macOS have official prebuilt binaries
- Linux lacks prebuilt binaries in releases; whisper-rs works there

### Decision: Keep whisper-rs as fallback for Linux

**What**: Use conditional compilation to:
- Windows/macOS: Use FFI with prebuilt binaries
- Linux: Continue using whisper-rs crate

**Why**:
- whisper-rs builds successfully on Linux
- Avoids maintaining a separate source build in build.rs
- Linux users typically have development tools installed

**Alternatives considered**:
- Build from source for all platforms in build.rs: Complex, duplicates whisper-rs-sys work
- Require Linux users to provide their own library: Poor UX

### Decision: Pin to specific whisper.cpp version

**What**: Hard-code the whisper.cpp version (initially v1.8.2) in build.rs.

**Why**:
- Ensures reproducible builds
- Avoids API compatibility issues between versions
- Can be updated deliberately with testing

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| GitHub rate limiting on binary downloads | Cache downloaded files in target directory; use GITHUB_TOKEN if available |
| Network unavailable during build | Fail with clear error message; document offline build process |
| API changes in new whisper.cpp versions | Pin version; update requires explicit change and testing |
| Larger binary size | Acceptable trade-off for build simplicity |
| xcframework extraction complexity | Parse Info.plist to find correct architecture slice |

## Implementation Approach

### FFI Bindings Structure

```rust
// whisper_ffi.rs - minimal bindings for required functions
pub struct WhisperContext { /* opaque pointer wrapper */ }
pub struct WhisperState { /* opaque pointer wrapper */ }
pub struct WhisperFullParams { /* opaque pointer wrapper */ }

extern "C" {
    fn whisper_init_from_file(path: *const c_char) -> *mut whisper_context;
    fn whisper_free(ctx: *mut whisper_context);
    fn whisper_full_default_params(strategy: c_int) -> whisper_full_params;
    fn whisper_full(ctx: *mut whisper_context, params: whisper_full_params, 
                    samples: *const f32, n_samples: c_int) -> c_int;
    fn whisper_full_n_segments(ctx: *mut whisper_context) -> c_int;
    fn whisper_full_get_segment_text(ctx: *mut whisper_context, i: c_int) -> *const c_char;
}
```

### Build Script Flow

1. Check if cached binary exists for current version
2. If not, download from GitHub releases
3. Extract appropriate files (DLL/dylib) to `target/{profile}/deps`
4. Set `cargo:rustc-link-search` to include library location
5. Copy library to output directory for runtime loading

## Open Questions

1. **Should we support CUDA/GPU builds?** The releases include CUDA-enabled binaries (`whisper-cublas-*`). Initial implementation will use CPU-only for simplicity, but GPU support could be added later.

2. **How to handle xcframework structure?** The macOS xcframework contains multiple architecture slices. Need to extract the correct one based on target architecture (x86_64 vs aarch64).
