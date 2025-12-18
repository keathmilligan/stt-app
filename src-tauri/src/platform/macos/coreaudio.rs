//! CoreAudio backend for macOS
//!
//! This module provides basic audio capture functionality using Apple's CoreAudio:
//! - Input device enumeration (microphones)
//! - Single-source capture using AudioUnit
//!
//! System audio capture and multi-source mixing are not yet implemented.

use crate::audio::{AudioSourceType, RecordingMode};
use crate::platform::{AudioBackend, AudioSamples, PlatformAudioDevice};
use coreaudio::audio_unit::macos_helpers::{
    get_audio_device_ids, get_audio_device_supports_scope, get_default_device_id, get_device_name,
};
use coreaudio::audio_unit::Scope;
use coreaudio::sys::{
    self, kAudioOutputUnitProperty_SetInputCallback, kAudioUnitProperty_StreamFormat,
    AudioBuffer, AudioBufferList, AudioUnitRenderActionFlags,
};
use std::os::raw::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Target sample rate for output (matches Linux/Windows backends)
const TARGET_SAMPLE_RATE: f64 = 48000.0;

/// Context passed to the input callback
struct InputCallbackContext {
    audio_unit: sys::AudioUnit,
    audio_tx: mpsc::Sender<CoreAudioSamples>,
    resampler: Option<Mutex<Resampler>>,
    num_channels: usize,
    is_non_interleaved: bool,
}

/// Raw input callback procedure for CoreAudio
extern "C" fn input_callback_proc(
    in_ref_con: *mut c_void,
    io_action_flags: *mut AudioUnitRenderActionFlags,
    in_time_stamp: *const sys::AudioTimeStamp,
    in_bus_number: u32,
    in_number_frames: u32,
    _io_data: *mut AudioBufferList,
) -> sys::OSStatus {
    let context = unsafe { &*(in_ref_con as *const InputCallbackContext) };

    // Allocate buffer list for the audio data
    // For non-interleaved stereo, we need 2 buffers
    let num_buffers = if context.is_non_interleaved {
        context.num_channels
    } else {
        1
    };

    let bytes_per_frame = 4; // f32
    let frames_per_buffer = in_number_frames as usize;
    let bytes_per_buffer = frames_per_buffer * bytes_per_frame;

    // Create buffer storage
    let mut buffer_data: Vec<Vec<u8>> = (0..num_buffers)
        .map(|_| vec![0u8; bytes_per_buffer])
        .collect();

    // Create AudioBufferList
    // Note: AudioBufferList has a flexible array member, so we need to allocate extra space
    let buffer_list_size =
        std::mem::size_of::<AudioBufferList>() + (num_buffers - 1) * std::mem::size_of::<AudioBuffer>();
    let mut buffer_list_storage = vec![0u8; buffer_list_size];
    let buffer_list = buffer_list_storage.as_mut_ptr() as *mut AudioBufferList;

    unsafe {
        (*buffer_list).mNumberBuffers = num_buffers as u32;

        let buffers_ptr = (*buffer_list).mBuffers.as_mut_ptr();
        for i in 0..num_buffers {
            let buffer = &mut *buffers_ptr.add(i);
            buffer.mNumberChannels = if context.is_non_interleaved {
                1
            } else {
                context.num_channels as u32
            };
            buffer.mDataByteSize = bytes_per_buffer as u32;
            buffer.mData = buffer_data[i].as_mut_ptr() as *mut c_void;
        }
    }

    // Call AudioUnitRender to get the audio data
    let status = unsafe {
        sys::AudioUnitRender(
            context.audio_unit,
            io_action_flags,
            in_time_stamp,
            in_bus_number,
            in_number_frames,
            buffer_list,
        )
    };

    if status != 0 {
        return status;
    }

    // Process the audio data
    let num_frames = in_number_frames as usize;
    let mut samples = Vec::with_capacity(num_frames * 2);

    unsafe {
        let buffer_list_ref = &*buffer_list;

        if context.is_non_interleaved {
            // Non-interleaved: each buffer is one channel
            let buffers_ptr = buffer_list_ref.mBuffers.as_ptr();

            let mut channel_ptrs: Vec<*const f32> = Vec::with_capacity(num_buffers);
            for i in 0..num_buffers {
                let buffer = &*buffers_ptr.add(i);
                channel_ptrs.push(buffer.mData as *const f32);
            }

            // Interleave to stereo
            for i in 0..num_frames {
                let left = *channel_ptrs[0].add(i);
                let right = if num_buffers > 1 {
                    *channel_ptrs[1].add(i)
                } else {
                    left
                };
                samples.push(left);
                samples.push(right);
            }
        } else {
            // Interleaved: single buffer with all channels
            let buffer = &buffer_list_ref.mBuffers[0];
            let data_ptr = buffer.mData as *const f32;

            if context.num_channels == 1 {
                for i in 0..num_frames {
                    let sample = *data_ptr.add(i);
                    samples.push(sample);
                    samples.push(sample);
                }
            } else {
                let total_samples = num_frames * context.num_channels;
                for i in 0..total_samples {
                    samples.push(*data_ptr.add(i));
                }
            }
        }
    }

    // Resample if needed
    let samples = if let Some(ref resampler) = context.resampler {
        resampler.lock().unwrap().process(&samples, 2)
    } else {
        samples
    };

    if !samples.is_empty() {
        let _ = context.audio_tx.send(CoreAudioSamples {
            samples,
            channels: 2,
        });
    }

    0 // noErr
}

/// Internal audio samples for channel communication
struct CoreAudioSamples {
    samples: Vec<f32>,
    channels: u16,
}

/// CoreAudio backend for macOS
pub struct CoreAudioBackend {
    /// Channel to receive audio samples (wrapped in Mutex for Sync)
    audio_rx: Mutex<mpsc::Receiver<CoreAudioSamples>>,
    /// Sender for audio samples (cloned into callback)
    audio_tx: mpsc::Sender<CoreAudioSamples>,
    /// Cached input devices
    input_devices: Vec<PlatformAudioDevice>,
    /// Sample rate (always 48kHz after resampling)
    sample_rate: u32,
    /// Active audio unit instance (raw pointer)
    audio_unit: Mutex<Option<sys::AudioUnit>>,
    /// Callback context (must be kept alive while capturing)
    callback_context: Mutex<Option<*mut InputCallbackContext>>,
    /// Flag indicating if capture is active
    is_capturing: Arc<AtomicBool>,
    /// AEC enabled flag (unused in basic implementation)
    #[allow(dead_code)]
    aec_enabled: Arc<Mutex<bool>>,
    /// Recording mode (unused in basic implementation)
    #[allow(dead_code)]
    recording_mode: Arc<Mutex<RecordingMode>>,
}

// Safety: The raw pointers are only accessed while holding the mutex
unsafe impl Send for CoreAudioBackend {}
unsafe impl Sync for CoreAudioBackend {}

impl CoreAudioBackend {
    /// Create a new CoreAudio backend
    pub fn new(
        aec_enabled: Arc<Mutex<bool>>,
        recording_mode: Arc<Mutex<RecordingMode>>,
    ) -> Result<Self, String> {
        let (audio_tx, audio_rx) = mpsc::channel();

        // Enumerate input devices
        let input_devices = enumerate_input_devices()?;

        Ok(Self {
            audio_rx: Mutex::new(audio_rx),
            audio_tx,
            input_devices,
            sample_rate: TARGET_SAMPLE_RATE as u32,
            audio_unit: Mutex::new(None),
            callback_context: Mutex::new(None),
            is_capturing: Arc::new(AtomicBool::new(false)),
            aec_enabled,
            recording_mode,
        })
    }
}

impl Drop for CoreAudioBackend {
    fn drop(&mut self) {
        let _ = self.stop_capture();
    }
}

impl AudioBackend for CoreAudioBackend {
    fn list_input_devices(&self) -> Vec<PlatformAudioDevice> {
        self.input_devices.clone()
    }

    fn list_system_devices(&self) -> Vec<PlatformAudioDevice> {
        // System audio capture not yet implemented
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
        // Check if already capturing
        if self.is_capturing.load(Ordering::SeqCst) {
            return Err("Already capturing".to_string());
        }

        // Multi-source capture not yet supported
        if source2_id.is_some() {
            return Err(
                "Multi-source capture is not yet implemented for macOS. Select only one source."
                    .to_string(),
            );
        }

        let device_id_str = source1_id.ok_or("No source device specified")?;
        let device_id: u32 = device_id_str
            .parse()
            .map_err(|_| format!("Invalid device ID: {}", device_id_str))?;

        // Create audio unit using raw CoreAudio API
        let audio_unit = create_input_audio_unit(device_id)?;

        // Get the stream format
        let (sample_rate, num_channels, is_non_interleaved) = get_stream_format(audio_unit)?;

        // Create resampler if needed
        let needs_resampling = (sample_rate - TARGET_SAMPLE_RATE).abs() > 1.0;
        let resampler = if needs_resampling {
            Some(Mutex::new(Resampler::new(
                sample_rate as u32,
                TARGET_SAMPLE_RATE as u32,
            )))
        } else {
            None
        };

        let audio_tx = self.audio_tx.clone();

        // Create callback context
        let callback_context = Box::new(InputCallbackContext {
            audio_unit,
            audio_tx,
            resampler,
            num_channels,
            is_non_interleaved,
        });
        let context_ptr = Box::into_raw(callback_context);

        // Set up the render callback struct
        let render_callback = sys::AURenderCallbackStruct {
            inputProc: Some(input_callback_proc),
            inputProcRefCon: context_ptr as *mut c_void,
        };

        let status = unsafe {
            sys::AudioUnitSetProperty(
                audio_unit,
                kAudioOutputUnitProperty_SetInputCallback,
                sys::kAudioUnitScope_Global,
                0,
                &render_callback as *const _ as *const c_void,
                std::mem::size_of::<sys::AURenderCallbackStruct>() as u32,
            )
        };

        if status != 0 {
            unsafe {
                let _ = Box::from_raw(context_ptr);
                sys::AudioComponentInstanceDispose(audio_unit);
            }
            return Err(format!(
                "Failed to set input callback: OSStatus {}",
                status
            ));
        }

        // Start the audio unit
        let status = unsafe { sys::AudioOutputUnitStart(audio_unit) };
        if status != 0 {
            unsafe {
                let _ = Box::from_raw(context_ptr);
                sys::AudioComponentInstanceDispose(audio_unit);
            }
            return Err(format!("Failed to start audio unit: OSStatus {}", status));
        }

        self.is_capturing.store(true, Ordering::SeqCst);
        *self.audio_unit.lock().unwrap() = Some(audio_unit);
        *self.callback_context.lock().unwrap() = Some(context_ptr);

        Ok(())
    }

    fn stop_capture(&self) -> Result<(), String> {
        if !self.is_capturing.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Stop and dispose the audio unit
        let mut audio_unit_guard = self.audio_unit.lock().unwrap();
        if let Some(audio_unit) = audio_unit_guard.take() {
            unsafe {
                sys::AudioOutputUnitStop(audio_unit);
                sys::AudioComponentInstanceDispose(audio_unit);
            }
        }

        // Free the callback context
        let mut context_guard = self.callback_context.lock().unwrap();
        if let Some(context_ptr) = context_guard.take() {
            unsafe {
                let _ = Box::from_raw(context_ptr);
            }
        }

        self.is_capturing.store(false, Ordering::SeqCst);

        // Drain any remaining samples
        let rx = self.audio_rx.lock().unwrap();
        while rx.try_recv().is_ok() {}

        Ok(())
    }

    fn try_recv(&self) -> Option<AudioSamples> {
        let rx = self.audio_rx.lock().unwrap();
        rx.try_recv().ok().map(|s| AudioSamples {
            samples: s.samples,
            channels: s.channels,
        })
    }
}

/// Create an input audio unit for the given device ID
fn create_input_audio_unit(device_id: u32) -> Result<sys::AudioUnit, String> {
    // Find the HAL Output audio unit (used for both input and output with hardware)
    let desc = sys::AudioComponentDescription {
        componentType: sys::kAudioUnitType_Output,
        componentSubType: sys::kAudioUnitSubType_HALOutput,
        componentManufacturer: sys::kAudioUnitManufacturer_Apple,
        componentFlags: 0,
        componentFlagsMask: 0,
    };

    let component = unsafe { sys::AudioComponentFindNext(ptr::null_mut(), &desc) };
    if component.is_null() {
        return Err("Failed to find HAL Output audio component".to_string());
    }

    let mut audio_unit: sys::AudioUnit = ptr::null_mut();
    let status = unsafe { sys::AudioComponentInstanceNew(component, &mut audio_unit) };
    if status != 0 {
        return Err(format!(
            "Failed to create audio unit instance: OSStatus {}",
            status
        ));
    }

    // Enable input on element 1 (input bus)
    let enable_input: u32 = 1;
    let status = unsafe {
        sys::AudioUnitSetProperty(
            audio_unit,
            sys::kAudioOutputUnitProperty_EnableIO,
            sys::kAudioUnitScope_Input,
            1,
            &enable_input as *const _ as *const c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };
    if status != 0 {
        unsafe { sys::AudioComponentInstanceDispose(audio_unit); }
        return Err(format!("Failed to enable input: OSStatus {}", status));
    }

    // Disable output on element 0 (output bus)
    let disable_output: u32 = 0;
    let status = unsafe {
        sys::AudioUnitSetProperty(
            audio_unit,
            sys::kAudioOutputUnitProperty_EnableIO,
            sys::kAudioUnitScope_Output,
            0,
            &disable_output as *const _ as *const c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };
    if status != 0 {
        unsafe { sys::AudioComponentInstanceDispose(audio_unit); }
        return Err(format!("Failed to disable output: OSStatus {}", status));
    }

    // Set the input device
    let status = unsafe {
        sys::AudioUnitSetProperty(
            audio_unit,
            sys::kAudioOutputUnitProperty_CurrentDevice,
            sys::kAudioUnitScope_Global,
            0,
            &device_id as *const _ as *const c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };
    if status != 0 {
        unsafe { sys::AudioComponentInstanceDispose(audio_unit); }
        return Err(format!("Failed to set device: OSStatus {}", status));
    }

    // Get the input format from the device (hardware format on input scope, element 1)
    let mut device_format: sys::AudioStreamBasicDescription = unsafe { std::mem::zeroed() };
    let mut size = std::mem::size_of::<sys::AudioStreamBasicDescription>() as u32;
    let status = unsafe {
        sys::AudioUnitGetProperty(
            audio_unit,
            kAudioUnitProperty_StreamFormat,
            sys::kAudioUnitScope_Input,
            1,
            &mut device_format as *mut _ as *mut c_void,
            &mut size,
        )
    };
    if status != 0 {
        unsafe { sys::AudioComponentInstanceDispose(audio_unit); }
        return Err(format!("Failed to get device format: OSStatus {}", status));
    }

    // Set the output format (what we receive in our callback) to match the device
    let status = unsafe {
        sys::AudioUnitSetProperty(
            audio_unit,
            kAudioUnitProperty_StreamFormat,
            sys::kAudioUnitScope_Output,
            1,
            &device_format as *const _ as *const c_void,
            std::mem::size_of::<sys::AudioStreamBasicDescription>() as u32,
        )
    };
    if status != 0 {
        unsafe { sys::AudioComponentInstanceDispose(audio_unit); }
        return Err(format!("Failed to set output format: OSStatus {}", status));
    }

    // Initialize the audio unit
    let status = unsafe { sys::AudioUnitInitialize(audio_unit) };
    if status != 0 {
        unsafe { sys::AudioComponentInstanceDispose(audio_unit); }
        return Err(format!(
            "Failed to initialize audio unit: OSStatus {}",
            status
        ));
    }

    Ok(audio_unit)
}

/// Get the stream format for an audio unit's input
fn get_stream_format(audio_unit: sys::AudioUnit) -> Result<(f64, usize, bool), String> {
    let mut asbd: sys::AudioStreamBasicDescription = unsafe { std::mem::zeroed() };
    let mut size = std::mem::size_of::<sys::AudioStreamBasicDescription>() as u32;

    let status = unsafe {
        sys::AudioUnitGetProperty(
            audio_unit,
            kAudioUnitProperty_StreamFormat,
            sys::kAudioUnitScope_Output,
            1, // Element 1 = input bus
            &mut asbd as *mut _ as *mut c_void,
            &mut size,
        )
    };

    if status != 0 {
        return Err(format!("Failed to get stream format: OSStatus {}", status));
    }

    let is_non_interleaved = (asbd.mFormatFlags & sys::kAudioFormatFlagIsNonInterleaved) != 0;
    let num_channels = asbd.mChannelsPerFrame as usize;

    Ok((asbd.mSampleRate, num_channels, is_non_interleaved))
}

/// Enumerate available input devices, with default device first
fn enumerate_input_devices() -> Result<Vec<PlatformAudioDevice>, String> {
    let device_ids =
        get_audio_device_ids().map_err(|e| format!("Failed to get audio devices: {:?}", e))?;

    // Get the default input device ID
    let default_input_id = get_default_device_id(true);

    let mut input_devices = Vec::new();

    for device_id in device_ids {
        // Check if this device supports input using the coreaudio-rs helper
        let supports_input =
            get_audio_device_supports_scope(device_id, Scope::Input).unwrap_or(false);

        if supports_input {
            let name = get_device_name(device_id)
                .unwrap_or_else(|_| format!("Unknown Device {}", device_id));

            input_devices.push(PlatformAudioDevice {
                id: device_id.to_string(),
                name,
                source_type: AudioSourceType::Input,
            });
        }
    }

    // Sort so default device is first
    if let Some(default_id) = default_input_id {
        let default_id_str = default_id.to_string();
        input_devices.sort_by(|a, b| {
            let a_is_default = a.id == default_id_str;
            let b_is_default = b.id == default_id_str;
            b_is_default.cmp(&a_is_default)
        });
    }

    Ok(input_devices)
}

/// Simple linear resampler (ported from Windows implementation)
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

/// Create a macOS CoreAudio backend
pub fn create_backend(
    aec_enabled: Arc<Mutex<bool>>,
    recording_mode: Arc<Mutex<RecordingMode>>,
) -> Result<Box<dyn AudioBackend>, String> {
    let backend = CoreAudioBackend::new(aec_enabled, recording_mode)?;
    Ok(Box::new(backend))
}
