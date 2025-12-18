//! macOS audio backend module
//!
//! Provides audio capture for macOS, supporting:
//! - Input device enumeration and capture (CoreAudio)
//! - System audio enumeration and capture via ScreenCaptureKit (macOS 12.3+)
//! - Multi-source capture with mixing
//! - Echo cancellation (AEC3)

mod coreaudio;
mod screencapturekit;

pub use coreaudio::create_backend;
