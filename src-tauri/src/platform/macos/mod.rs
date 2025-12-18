//! macOS audio backend module
//!
//! Provides CoreAudio-based audio capture for macOS, supporting:
//! - Input device enumeration
//! - Single-source capture from microphones
//!
//! System audio capture and multi-source mixing are not yet implemented.

mod coreaudio;
mod stub;

pub use coreaudio::create_backend;
