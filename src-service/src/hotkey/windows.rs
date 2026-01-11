//! Windows hotkey backend stub.
//!
//! This is a placeholder implementation. Full Windows support using
//! RegisterHotKey API will be implemented in a future release.

use super::backend::{HotkeyBackend, HotkeyEvent};
use flowstt_common::KeyCode;

/// Windows hotkey backend (stub implementation)
pub struct WindowsHotkeyBackend {
    _private: (),
}

impl WindowsHotkeyBackend {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl HotkeyBackend for WindowsHotkeyBackend {
    fn start(&mut self, _key: KeyCode) -> Result<(), String> {
        Err("Push-to-talk is not yet available on Windows. This feature will be implemented in a future release.".to_string())
    }

    fn stop(&mut self) {
        // No-op for stub
    }

    fn try_recv(&self) -> Option<HotkeyEvent> {
        None
    }

    fn is_running(&self) -> bool {
        false
    }

    fn is_available(&self) -> bool {
        false
    }

    fn unavailable_reason(&self) -> Option<String> {
        Some("Push-to-talk is not yet available on Windows".to_string())
    }
}
