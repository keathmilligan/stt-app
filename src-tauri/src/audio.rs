use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig};
use rubato::{FftFixedIn, Resampler};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

/// Shared buffer for recording samples
struct RecordingBuffer {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    is_recording: bool,
}

/// Thread-safe recording state that can be shared with Tauri
pub struct RecordingState {
    buffer: Arc<Mutex<RecordingBuffer>>,
    stop_signal: Arc<Mutex<bool>>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(RecordingBuffer {
                samples: Vec::new(),
                sample_rate: 0,
                channels: 0,
                is_recording: false,
            })),
            stop_signal: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_recording(&self) -> bool {
        self.buffer.lock().unwrap().is_recording
    }
}

pub fn list_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    let mut result = Vec::new();
    for (index, device) in devices.enumerate() {
        let name = device
            .name()
            .unwrap_or_else(|_| format!("Unknown Device {}", index));
        result.push(AudioDevice {
            id: index.to_string(),
            name,
        });
    }

    Ok(result)
}

fn get_device_by_id(device_id: &str) -> Result<Device, String> {
    let host = cpal::default_host();
    let index: usize = device_id
        .parse()
        .map_err(|_| "Invalid device ID".to_string())?;

    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

    devices
        .enumerate()
        .find(|(i, _)| *i == index)
        .map(|(_, d)| d)
        .ok_or_else(|| "Device not found".to_string())
}

pub fn start_recording(device_id: &str, state: &RecordingState) -> Result<(), String> {
    {
        let buffer = state.buffer.lock().unwrap();
        if buffer.is_recording {
            return Err("Already recording".to_string());
        }
    }

    let device = get_device_by_id(device_id)?;
    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get default config: {}", e))?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let sample_format = config.sample_format();

    // Initialize recording state
    {
        let mut buffer = state.buffer.lock().unwrap();
        buffer.samples.clear();
        buffer.sample_rate = sample_rate;
        buffer.channels = channels;
        buffer.is_recording = true;
    }

    // Reset stop signal
    {
        let mut stop = state.stop_signal.lock().unwrap();
        *stop = false;
    }

    let buffer_clone = Arc::clone(&state.buffer);
    let stop_signal = Arc::clone(&state.stop_signal);

    // Spawn recording thread - the stream lives entirely in this thread
    thread::spawn(move || {
        let stream_config: StreamConfig = config.into();
        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        let buffer_for_callback = Arc::clone(&buffer_clone);

        let stream_result = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buffer) = buffer_for_callback.lock() {
                        if buffer.is_recording {
                            buffer.samples.extend_from_slice(data);
                        }
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => {
                let buffer_for_i16 = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buffer) = buffer_for_i16.lock() {
                            if buffer.is_recording {
                                let float_samples: Vec<f32> =
                                    data.iter().map(|&s| s as f32 / 32768.0).collect();
                                buffer.samples.extend(float_samples);
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            SampleFormat::U16 => {
                let buffer_for_u16 = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buffer) = buffer_for_u16.lock() {
                            if buffer.is_recording {
                                let float_samples: Vec<f32> = data
                                    .iter()
                                    .map(|&s| (s as f32 - 32768.0) / 32768.0)
                                    .collect();
                                buffer.samples.extend(float_samples);
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => {
                eprintln!("Unsupported sample format: {:?}", sample_format);
                return;
            }
        };

        let stream = match stream_result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to build stream: {}", e);
                if let Ok(mut buffer) = buffer_clone.lock() {
                    buffer.is_recording = false;
                }
                return;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("Failed to start stream: {}", e);
            if let Ok(mut buffer) = buffer_clone.lock() {
                buffer.is_recording = false;
            }
            return;
        }

        // Wait for stop signal
        loop {
            thread::sleep(std::time::Duration::from_millis(50));
            if *stop_signal.lock().unwrap() {
                break;
            }
        }

        // Stream is dropped here when thread ends, which stops recording
    });

    Ok(())
}

pub fn stop_recording(state: &RecordingState) -> Result<Vec<f32>, String> {
    // Signal the recording thread to stop
    {
        let mut stop = state.stop_signal.lock().unwrap();
        *stop = true;
    }

    // Give the thread a moment to stop
    thread::sleep(std::time::Duration::from_millis(100));

    // Extract recorded audio
    let (samples, sample_rate, channels) = {
        let mut buffer = state.buffer.lock().unwrap();
        buffer.is_recording = false;
        let samples = std::mem::take(&mut buffer.samples);
        (samples, buffer.sample_rate, buffer.channels)
    };

    if samples.is_empty() {
        return Err("No audio recorded".to_string());
    }

    // Convert to mono if stereo
    let mono_samples = if channels > 1 {
        convert_to_mono(&samples, channels as usize)
    } else {
        samples
    };

    // Resample to 16kHz for Whisper
    let resampled = resample_to_16khz(&mono_samples, sample_rate)?;

    Ok(resampled)
}

fn convert_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    samples
        .chunks(channels)
        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
        .collect()
}

fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Result<Vec<f32>, String> {
    const TARGET_RATE: u32 = 16000;

    if source_rate == TARGET_RATE {
        return Ok(samples.to_vec());
    }

    let chunk_size = 1024;
    let mut resampler = FftFixedIn::<f32>::new(
        source_rate as usize,
        TARGET_RATE as usize,
        chunk_size,
        2,
        1, // mono
    )
    .map_err(|e| format!("Failed to create resampler: {}", e))?;

    let mut output = Vec::new();

    for chunk in samples.chunks(chunk_size) {
        let mut padded_chunk = chunk.to_vec();
        // Pad last chunk if needed
        if padded_chunk.len() < chunk_size {
            padded_chunk.resize(chunk_size, 0.0);
        }

        let input = vec![padded_chunk];
        let result = resampler
            .process(&input, None)
            .map_err(|e| format!("Resample error: {}", e))?;

        if !result.is_empty() {
            output.extend(&result[0]);
        }
    }

    Ok(output)
}
