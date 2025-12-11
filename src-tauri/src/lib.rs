mod audio;
mod transcribe;

use audio::{AudioDevice, RecordingState};
use std::env;
use std::sync::Mutex;
use tauri::State;
use transcribe::Transcriber;

/// Detect if running on Wayland and set workaround env vars
fn configure_wayland_workarounds() {
    // Check for Wayland session
    let is_wayland = env::var("WAYLAND_DISPLAY").is_ok()
        || env::var("XDG_SESSION_TYPE")
            .map(|v| v.to_lowercase() == "wayland")
            .unwrap_or(false);

    if is_wayland {
        // WebKitGTK has compositing issues on Wayland
        env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }
}

struct AppState {
    recording: RecordingState,
    transcriber: Mutex<Transcriber>,
}

// Implement Send + Sync for AppState since RecordingState only contains Arc<Mutex<_>>
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

#[tauri::command]
fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    audio::list_devices()
}

#[tauri::command]
fn start_recording(device_id: String, state: State<AppState>) -> Result<(), String> {
    audio::start_recording(&device_id, &state.recording)
}

#[tauri::command]
fn stop_recording(state: State<AppState>) -> Result<Vec<f32>, String> {
    audio::stop_recording(&state.recording)
}

#[tauri::command]
fn is_recording(state: State<AppState>) -> bool {
    state.recording.is_recording()
}

#[tauri::command]
fn transcribe(audio_data: Vec<f32>, state: State<AppState>) -> Result<String, String> {
    let mut transcriber = state.transcriber.lock().unwrap();
    transcriber.transcribe(&audio_data)
}

#[tauri::command]
fn check_model_status(state: State<AppState>) -> Result<ModelStatus, String> {
    let transcriber = state.transcriber.lock().unwrap();
    Ok(ModelStatus {
        available: transcriber.is_model_available(),
        path: transcriber.get_model_path().to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn download_model(state: State<AppState>) -> Result<(), String> {
    let transcriber = state.transcriber.lock().unwrap();
    let model_path = transcriber.get_model_path().clone();
    drop(transcriber); // Release lock during download
    
    transcribe::download_model(&model_path)
}

#[derive(serde::Serialize)]
struct ModelStatus {
    available: bool,
    path: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    configure_wayland_workarounds();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            recording: RecordingState::new(),
            transcriber: Mutex::new(Transcriber::new()),
        })
        .invoke_handler(tauri::generate_handler![
            list_audio_devices,
            start_recording,
            stop_recording,
            is_recording,
            transcribe,
            check_model_status,
            download_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
