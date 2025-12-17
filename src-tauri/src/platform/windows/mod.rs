//! Windows audio backend module
//!
//! Provides audio capture using WASAPI (Windows Audio Session API).
//! Currently supports single-source input capture (microphones).
//! System audio capture and multi-source mixing are stubbed for future implementation.

#[allow(dead_code)]
mod stub;
mod wasapi;

pub use wasapi::create_backend;
