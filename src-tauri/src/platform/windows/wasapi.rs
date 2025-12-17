//! WASAPI audio backend for Windows
//!
//! This module provides audio capture from input devices (microphones) using
//! Windows Audio Session API (WASAPI). Currently supports single-source capture only.
//! System audio capture (loopback) and multi-source mixing are stubbed for future implementation.

use crate::audio::{AudioSourceType, RecordingMode};
use crate::platform::{AudioBackend, AudioSamples, PlatformAudioDevice};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use windows::core::{GUID, PCWSTR, PWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{
    eCapture, IAudioCaptureClient, IAudioClient, IMMDevice, IMMDeviceCollection,
    IMMDeviceEnumerator, MMDeviceEnumerator, AUDCLNT_SHAREMODE_SHARED,
    AUDCLNT_STREAMFLAGS_EVENTCALLBACK, WAVEFORMATEX, WAVEFORMATEXTENSIBLE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
    STGM_READ,
};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;

/// WAVE_FORMAT_EXTENSIBLE constant (0xFFFE)
const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;

/// WAVE_FORMAT_PCM constant (1)
const WAVE_FORMAT_PCM: u16 = 1;

/// WAVE_FORMAT_IEEE_FLOAT constant (3)
const WAVE_FORMAT_IEEE_FLOAT: u16 = 3;

/// KSDATAFORMAT_SUBTYPE_IEEE_FLOAT GUID
const KSDATAFORMAT_SUBTYPE_IEEE_FLOAT: GUID = GUID::from_u128(0x00000003_0000_0010_8000_00aa00389b71);

/// Target sample rate for output (matches Linux backend)
const TARGET_SAMPLE_RATE: u32 = 48000;

/// Internal audio samples for channel communication
struct WasapiAudioSamples {
    samples: Vec<f32>,
    channels: u16,
}

/// Commands sent to the capture thread
enum CaptureCommand {
    Start {
        device_id: String,
        result_tx: mpsc::Sender<Result<(), String>>,
    },
    Stop,
    Shutdown,
}

/// WASAPI audio backend for Windows
pub struct WasapiBackend {
    /// Channel to send commands to capture thread
    cmd_tx: mpsc::Sender<CaptureCommand>,
    /// Channel to receive audio samples from capture thread (wrapped in Mutex for Sync)
    audio_rx: Mutex<mpsc::Receiver<WasapiAudioSamples>>,
    /// Cached input devices
    input_devices: Arc<Mutex<Vec<PlatformAudioDevice>>>,
    /// Sample rate (always 48kHz after resampling)
    sample_rate: u32,
    /// Capture thread handle
    _thread_handle: JoinHandle<()>,
    /// Flag indicating if capture is active
    is_capturing: Arc<AtomicBool>,
    /// AEC enabled flag (unused for now, kept for API compatibility)
    #[allow(dead_code)]
    aec_enabled: Arc<Mutex<bool>>,
    /// Recording mode (unused for now, kept for API compatibility)
    #[allow(dead_code)]
    recording_mode: Arc<Mutex<RecordingMode>>,
}

impl WasapiBackend {
    /// Create a new WASAPI backend
    pub fn new(
        aec_enabled: Arc<Mutex<bool>>,
        recording_mode: Arc<Mutex<RecordingMode>>,
    ) -> Result<Self, String> {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (audio_tx, audio_rx) = mpsc::channel();
        let input_devices = Arc::new(Mutex::new(Vec::new()));
        let is_capturing = Arc::new(AtomicBool::new(false));

        // Initialize COM on this thread if not already initialized
        // We use COINIT_MULTITHREADED for compatibility with capture thread
        let com_initialized = unsafe {
            // CoInitializeEx returns S_OK (0) if successful, S_FALSE (1) if already initialized,
            // or an error code. We only need to uninitialize if we initialized it ourselves.
            let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
            hr.is_ok()
        };

        // Enumerate devices
        let devices = enumerate_input_devices();

        // Uninitialize COM if we initialized it (balance the call)
        if com_initialized {
            unsafe {
                CoUninitialize();
            }
        }

        // Now handle the result
        let devices = devices?;
        *input_devices.lock().unwrap() = devices;

        let input_devices_clone = Arc::clone(&input_devices);
        let is_capturing_clone = Arc::clone(&is_capturing);

        let thread_handle = thread::spawn(move || {
            run_capture_thread(cmd_rx, audio_tx, input_devices_clone, is_capturing_clone);
        });

        Ok(Self {
            cmd_tx,
            audio_rx: Mutex::new(audio_rx),
            input_devices,
            sample_rate: TARGET_SAMPLE_RATE,
            _thread_handle: thread_handle,
            is_capturing,
            aec_enabled,
            recording_mode,
        })
    }
}

impl Drop for WasapiBackend {
    fn drop(&mut self) {
        // Signal the capture thread to shutdown
        let _ = self.cmd_tx.send(CaptureCommand::Shutdown);
    }
}

impl AudioBackend for WasapiBackend {
    fn list_input_devices(&self) -> Vec<PlatformAudioDevice> {
        self.input_devices.lock().unwrap().clone()
    }

    fn list_system_devices(&self) -> Vec<PlatformAudioDevice> {
        // System audio capture (loopback) not yet implemented
        Vec::new()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn start_capture_sources(
        &self,
        source1_id: Option<String>,
        source2_id: Option<String>,
    ) -> Result<(), String> {
        // Check for two-source capture (not supported)
        if source1_id.is_some() && source2_id.is_some() {
            return Err(
                "Multi-source capture is not yet implemented for Windows. Please select only one source."
                    .to_string(),
            );
        }

        // Get the device ID to capture from
        let device_id = source1_id
            .or(source2_id)
            .ok_or("No audio source specified")?;

        // Create a channel to receive the result
        let (result_tx, result_rx) = mpsc::channel();

        self.cmd_tx
            .send(CaptureCommand::Start { device_id, result_tx })
            .map_err(|e| format!("Failed to send start command: {}", e))?;

        // Wait for the result from the capture thread (with timeout)
        match result_rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err("Timeout waiting for audio capture to start".to_string())
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err("Capture thread disconnected".to_string())
            }
        }
    }

    fn stop_capture(&self) -> Result<(), String> {
        self.cmd_tx
            .send(CaptureCommand::Stop)
            .map_err(|e| format!("Failed to send stop command: {}", e))?;
        Ok(())
    }

    fn try_recv(&self) -> Option<AudioSamples> {
        self.audio_rx
            .lock()
            .unwrap()
            .try_recv()
            .ok()
            .map(|samples| AudioSamples {
                samples: samples.samples,
                channels: samples.channels,
            })
    }
}

/// Create a Windows audio backend using WASAPI
pub fn create_backend(
    aec_enabled: Arc<Mutex<bool>>,
    recording_mode: Arc<Mutex<RecordingMode>>,
) -> Result<Box<dyn AudioBackend>, String> {
    let backend = WasapiBackend::new(aec_enabled, recording_mode)?;
    Ok(Box::new(backend))
}

/// Enumerate available input devices (microphones)
fn enumerate_input_devices() -> Result<Vec<PlatformAudioDevice>, String> {
    unsafe {
        // Create device enumerator
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| format!("Failed to create device enumerator: {}", e))?;

        // Enumerate capture devices
        let collection: IMMDeviceCollection = enumerator
            .EnumAudioEndpoints(eCapture, windows::Win32::Media::Audio::DEVICE_STATE_ACTIVE)
            .map_err(|e| format!("Failed to enumerate audio endpoints: {}", e))?;

        let count = collection
            .GetCount()
            .map_err(|e| format!("Failed to get device count: {}", e))?;

        let mut devices = Vec::new();

        for i in 0..count {
            if let Ok(device) = collection.Item(i) {
                if let Some(platform_device) = device_to_platform_device(&device) {
                    devices.push(platform_device);
                }
            }
        }

        Ok(devices)
    }
}

/// Convert an IMMDevice to a PlatformAudioDevice
fn device_to_platform_device(device: &IMMDevice) -> Option<PlatformAudioDevice> {
    unsafe {
        // Get device ID
        let id_ptr: PWSTR = device.GetId().ok()?;
        let id = pwstr_to_string(id_ptr);
        windows::Win32::System::Com::CoTaskMemFree(Some(id_ptr.0 as *const _));

        // Get friendly name from property store
        let props: IPropertyStore = device.OpenPropertyStore(STGM_READ).ok()?;
        let prop_variant = props.GetValue(&PKEY_Device_FriendlyName).ok()?;

        let name = {
            // Try to get string value from PROPVARIANT
            let name_str = prop_variant.to_string();
            if name_str.is_empty() {
                "Unknown Device".to_string()
            } else {
                name_str
            }
        };

        Some(PlatformAudioDevice {
            id,
            name,
            source_type: AudioSourceType::Input,
        })
    }
}

/// Convert a PWSTR to a Rust String
fn pwstr_to_string(pwstr: PWSTR) -> String {
    unsafe {
        if pwstr.0.is_null() {
            return String::new();
        }
        let len = (0..).take_while(|&i| *pwstr.0.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(pwstr.0, len);
        String::from_utf16_lossy(slice)
    }
}

/// Run the capture thread
fn run_capture_thread(
    cmd_rx: mpsc::Receiver<CaptureCommand>,
    audio_tx: mpsc::Sender<WasapiAudioSamples>,
    _input_devices: Arc<Mutex<Vec<PlatformAudioDevice>>>,
    is_capturing: Arc<AtomicBool>,
) {
    println!("WASAPI: Capture thread started");
    
    unsafe {
        // Initialize COM for this thread (MTA)
        let com_result = CoInitializeEx(None, COINIT_MULTITHREADED);
        if com_result.is_err() {
            eprintln!("WASAPI: Failed to initialize COM on capture thread: {:?}", com_result);
            // Drain any pending commands and respond with errors
            while let Ok(cmd) = cmd_rx.try_recv() {
                if let CaptureCommand::Start { result_tx, .. } = cmd {
                    let _ = result_tx.send(Err(format!("COM initialization failed: {:?}", com_result)));
                }
            }
            return;
        }
        println!("WASAPI: COM initialized on capture thread");

        let mut capture_state: Option<CaptureState> = None;

        loop {
            // Check for commands (non-blocking when capturing)
            let timeout = if capture_state.is_some() {
                std::time::Duration::from_millis(1)
            } else {
                std::time::Duration::from_secs(1)
            };

            match cmd_rx.recv_timeout(timeout) {
                Ok(CaptureCommand::Start { device_id, result_tx }) => {
                    // Stop any existing capture
                    if let Some(state) = capture_state.take() {
                        drop(state);
                    }

                    // Start new capture
                    match start_capture(&device_id) {
                        Ok(state) => {
                            println!("WASAPI: Started capture from device {}", device_id);
                            is_capturing.store(true, Ordering::SeqCst);
                            capture_state = Some(state);
                            let _ = result_tx.send(Ok(()));
                        }
                        Err(e) => {
                            eprintln!("WASAPI: Failed to start capture: {}", e);
                            is_capturing.store(false, Ordering::SeqCst);
                            let _ = result_tx.send(Err(e));
                        }
                    }
                }
                Ok(CaptureCommand::Stop) => {
                    if let Some(state) = capture_state.take() {
                        println!("WASAPI: Stopping capture");
                        drop(state);
                    }
                    is_capturing.store(false, Ordering::SeqCst);
                }
                Ok(CaptureCommand::Shutdown) => {
                    if let Some(state) = capture_state.take() {
                        drop(state);
                    }
                    is_capturing.store(false, Ordering::SeqCst);
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Continue processing audio if capturing
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }

            // Process audio if capturing
            if let Some(ref mut state) = capture_state {
                if let Err(e) = process_capture(state, &audio_tx) {
                    eprintln!("WASAPI: Capture error: {}", e);
                    capture_state = None;
                    is_capturing.store(false, Ordering::SeqCst);
                }
            }
        }

        CoUninitialize();
    }
}

/// State for an active capture session
struct CaptureState {
    audio_client: IAudioClient,
    capture_client: IAudioCaptureClient,
    format: CaptureFormat,
    event_handle: windows::Win32::Foundation::HANDLE,
    resampler: Option<Resampler>,
}

impl Drop for CaptureState {
    fn drop(&mut self) {
        unsafe {
            let _ = self.audio_client.Stop();
            if !self.event_handle.is_invalid() {
                let _ = windows::Win32::Foundation::CloseHandle(self.event_handle);
            }
        }
    }
}

/// Format information for captured audio
struct CaptureFormat {
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    is_float: bool,
}

/// Start capturing from a device
unsafe fn start_capture(device_id: &str) -> Result<CaptureState, String> {
    // Create device enumerator
    let enumerator: IMMDeviceEnumerator =
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| format!("Failed to create device enumerator: {}", e))?;

    // Get the device by ID
    let device_id_wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
    let device: IMMDevice = enumerator
        .GetDevice(PCWSTR(device_id_wide.as_ptr()))
        .map_err(|e| format!("Failed to get device {}: {}", device_id, e))?;

    // Activate audio client
    let audio_client: IAudioClient = device
        .Activate(CLSCTX_ALL, None)
        .map_err(|e| format!("Failed to activate audio client: {}", e))?;

    // Get the mix format (device's native format)
    let mix_format_ptr = audio_client
        .GetMixFormat()
        .map_err(|e| format!("Failed to get mix format: {}", e))?;

    let mix_format = &*mix_format_ptr;
    let format = parse_wave_format(mix_format)?;

    println!(
        "WASAPI: Device format: {}Hz, {} channels, {} bits, float={}",
        format.sample_rate, format.channels, format.bits_per_sample, format.is_float
    );

    // Create event for buffer notifications
    let event_handle = CreateEventW(None, false, false, None)
        .map_err(|e| format!("Failed to create event: {}", e))?;

    // Calculate buffer duration (100ms in 100-nanosecond units)
    let buffer_duration: i64 = 1_000_000; // 100ms

    // Initialize audio client in shared mode with event callback
    audio_client
        .Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
            buffer_duration,
            0,
            mix_format_ptr,
            None,
        )
        .map_err(|e| format!("Failed to initialize audio client: {}", e))?;

    // Set the event handle
    audio_client
        .SetEventHandle(event_handle)
        .map_err(|e| format!("Failed to set event handle: {}", e))?;

    // Get capture client
    let capture_client: IAudioCaptureClient = audio_client
        .GetService()
        .map_err(|e| format!("Failed to get capture client: {}", e))?;

    // Create resampler if needed
    let resampler = if format.sample_rate != TARGET_SAMPLE_RATE {
        Some(Resampler::new(format.sample_rate, TARGET_SAMPLE_RATE))
    } else {
        None
    };

    // Start capture
    audio_client
        .Start()
        .map_err(|e| format!("Failed to start capture: {}", e))?;

    // Free the format pointer
    windows::Win32::System::Com::CoTaskMemFree(Some(mix_format_ptr as *const _ as *const _));

    Ok(CaptureState {
        audio_client,
        capture_client,
        format,
        event_handle,
        resampler,
    })
}

/// Parse WAVEFORMATEX into CaptureFormat
fn parse_wave_format(format: &WAVEFORMATEX) -> Result<CaptureFormat, String> {
    let is_float;
    let bits_per_sample;

    // Copy values from potentially packed struct to avoid alignment issues
    let format_tag = format.wFormatTag;
    let sample_rate = format.nSamplesPerSec;
    let channels = format.nChannels;
    let bits = format.wBitsPerSample;

    if format_tag == WAVE_FORMAT_EXTENSIBLE {
        let ext = unsafe { &*(format as *const WAVEFORMATEX as *const WAVEFORMATEXTENSIBLE) };
        // Read packed fields safely using ptr::read_unaligned
        let sub_format = unsafe {
            std::ptr::read_unaligned(std::ptr::addr_of!(ext.SubFormat))
        };
        let valid_bits = unsafe {
            std::ptr::read_unaligned(std::ptr::addr_of!(ext.Samples.wValidBitsPerSample))
        };
        is_float = sub_format == KSDATAFORMAT_SUBTYPE_IEEE_FLOAT;
        bits_per_sample = valid_bits;
    } else if format_tag == WAVE_FORMAT_IEEE_FLOAT {
        is_float = true;
        bits_per_sample = bits;
    } else if format_tag == WAVE_FORMAT_PCM {
        is_float = false;
        bits_per_sample = bits;
    } else {
        return Err(format!("Unsupported audio format tag: {}", format_tag));
    }

    Ok(CaptureFormat {
        sample_rate,
        channels,
        bits_per_sample,
        is_float,
    })
}

/// Process captured audio data
unsafe fn process_capture(
    state: &mut CaptureState,
    audio_tx: &mpsc::Sender<WasapiAudioSamples>,
) -> Result<(), String> {
    // Wait for buffer event (10ms timeout)
    let wait_result = WaitForSingleObject(state.event_handle, 10);
    if wait_result.0 != 0 {
        // Timeout or error - that's OK, just no data yet
        return Ok(());
    }

    loop {
        let mut buffer_ptr: *mut u8 = std::ptr::null_mut();
        let mut num_frames: u32 = 0;
        let mut flags: u32 = 0;

        // Get the buffer
        let result = state.capture_client.GetBuffer(
            &mut buffer_ptr,
            &mut num_frames,
            &mut flags,
            None,
            None,
        );

        if result.is_err() || num_frames == 0 {
            break;
        }

        // Convert to f32 samples
        let samples = convert_to_f32(
            buffer_ptr,
            num_frames as usize,
            &state.format,
        );

        // Release the buffer
        let _ = state.capture_client.ReleaseBuffer(num_frames);

        if samples.is_empty() {
            continue;
        }

        // Resample if needed
        let final_samples = if let Some(ref mut resampler) = state.resampler {
            resampler.process(&samples, state.format.channels as usize)
        } else {
            samples
        };

        // Convert mono to stereo if needed
        let stereo_samples = if state.format.channels == 1 {
            mono_to_stereo(&final_samples)
        } else {
            final_samples
        };

        // Send samples
        let _ = audio_tx.send(WasapiAudioSamples {
            samples: stereo_samples,
            channels: 2,
        });
    }

    Ok(())
}

/// Convert raw audio buffer to f32 samples
unsafe fn convert_to_f32(
    buffer: *const u8,
    num_frames: usize,
    format: &CaptureFormat,
) -> Vec<f32> {
    let num_samples = num_frames * format.channels as usize;

    if format.is_float && format.bits_per_sample == 32 {
        // Already f32
        let f32_ptr = buffer as *const f32;
        std::slice::from_raw_parts(f32_ptr, num_samples).to_vec()
    } else if !format.is_float && format.bits_per_sample == 16 {
        // 16-bit signed integer
        let i16_ptr = buffer as *const i16;
        let i16_slice = std::slice::from_raw_parts(i16_ptr, num_samples);
        i16_slice.iter().map(|&s| s as f32 / 32768.0).collect()
    } else if !format.is_float && format.bits_per_sample == 24 {
        // 24-bit signed integer (packed)
        let mut samples = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let offset = i * 3;
            let b0 = *buffer.add(offset) as i32;
            let b1 = *buffer.add(offset + 1) as i32;
            let b2 = *buffer.add(offset + 2) as i32;
            // Sign extend from 24-bit
            let value = (b0 | (b1 << 8) | (b2 << 16)) << 8 >> 8;
            samples.push(value as f32 / 8388608.0);
        }
        samples
    } else if !format.is_float && format.bits_per_sample == 32 {
        // 32-bit signed integer
        let i32_ptr = buffer as *const i32;
        let i32_slice = std::slice::from_raw_parts(i32_ptr, num_samples);
        i32_slice.iter().map(|&s| s as f32 / 2147483648.0).collect()
    } else {
        eprintln!(
            "WASAPI: Unsupported format: float={}, bits={}",
            format.is_float, format.bits_per_sample
        );
        Vec::new()
    }
}

/// Convert mono audio to stereo by duplicating channels
fn mono_to_stereo(mono: &[f32]) -> Vec<f32> {
    let mut stereo = Vec::with_capacity(mono.len() * 2);
    for &sample in mono {
        stereo.push(sample);
        stereo.push(sample);
    }
    stereo
}

/// Simple linear resampler
struct Resampler {
    source_rate: u32,
    target_rate: u32,
    buffer: Vec<f32>,
    position: f64,
}

impl Resampler {
    fn new(source_rate: u32, target_rate: u32) -> Self {
        Self {
            source_rate,
            target_rate,
            buffer: Vec::new(),
            position: 0.0,
        }
    }

    fn process(&mut self, samples: &[f32], channels: usize) -> Vec<f32> {
        // Append new samples to buffer
        self.buffer.extend_from_slice(samples);

        let ratio = self.source_rate as f64 / self.target_rate as f64;
        let input_frames = self.buffer.len() / channels;
        let output_frames = ((input_frames as f64 - self.position) / ratio) as usize;

        if output_frames == 0 {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(output_frames * channels);

        for _ in 0..output_frames {
            let src_frame = self.position as usize;
            let frac = self.position - src_frame as f64;

            for ch in 0..channels {
                let idx0 = src_frame * channels + ch;
                let idx1 = (src_frame + 1) * channels + ch;

                let sample = if idx1 < self.buffer.len() {
                    // Linear interpolation
                    self.buffer[idx0] * (1.0 - frac as f32) + self.buffer[idx1] * frac as f32
                } else if idx0 < self.buffer.len() {
                    self.buffer[idx0]
                } else {
                    0.0
                };
                output.push(sample);
            }

            self.position += ratio;
        }

        // Remove consumed samples from buffer
        let consumed_frames = self.position as usize;
        if consumed_frames > 0 {
            let consumed_samples = consumed_frames * channels;
            if consumed_samples < self.buffer.len() {
                self.buffer.drain(0..consumed_samples);
                self.position -= consumed_frames as f64;
            } else {
                self.buffer.clear();
                self.position = 0.0;
            }
        }

        output
    }
}
