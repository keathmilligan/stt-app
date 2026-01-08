//! Global service state management.
//!
//! This module manages the shared state for the FlowSTT service,
//! including transcription status and audio backend state.

use flowstt_common::{RecordingMode, TranscribeStatus};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Global service state
#[derive(Default)]
pub struct ServiceState {
    /// Current transcription status
    pub transcribe_status: TranscribeStatus,
    /// Whether AEC is enabled
    pub aec_enabled: bool,
    /// Current recording mode
    pub recording_mode: RecordingMode,
    /// Primary audio source ID
    pub source1_id: Option<String>,
    /// Secondary audio source ID
    pub source2_id: Option<String>,
}

/// Thread-safe wrapper for service state
pub type SharedState = Arc<Mutex<ServiceState>>;

/// Get the global service state singleton
static SERVICE_STATE: std::sync::OnceLock<SharedState> = std::sync::OnceLock::new();

pub fn get_service_state() -> SharedState {
    SERVICE_STATE
        .get_or_init(|| Arc::new(Mutex::new(ServiceState::default())))
        .clone()
}
