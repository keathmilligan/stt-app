use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Audio processor trait for extensible audio analysis.
/// Processors must be fast and non-blocking as they run in the audio callback.
pub trait AudioProcessor: Send {
    /// Process a batch of audio samples.
    /// Samples are mono f32 values, typically in the range [-1.0, 1.0].
    /// The AppHandle can be used to emit events to the frontend.
    fn process(&mut self, samples: &[f32], app_handle: &AppHandle);

    /// Return the processor's name for identification.
    fn name(&self) -> &str;
}

/// Silence detector that logs state transitions to console.
/// Uses RMS (root mean square) amplitude with a configurable dB threshold.
pub struct SilenceDetector {
    /// Threshold in dB below which audio is considered silent (default: -40.0)
    threshold_db: f32,
    /// Current silence state
    is_silent: bool,
    /// Whether we've logged the initial state
    initialized: bool,
}

impl SilenceDetector {
    /// Create a new silence detector with default threshold (-40 dB)
    pub fn new() -> Self {
        Self {
            threshold_db: -40.0,
            is_silent: true,
            initialized: false,
        }
    }

    /// Calculate RMS amplitude of samples
    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Convert linear amplitude to decibels
    fn amplitude_to_db(amplitude: f32) -> f32 {
        if amplitude <= 0.0 {
            return f32::NEG_INFINITY;
        }
        20.0 * amplitude.log10()
    }
}

impl Default for SilenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioProcessor for SilenceDetector {
    fn process(&mut self, samples: &[f32], _app_handle: &AppHandle) {
        let rms = Self::calculate_rms(samples);
        let db = Self::amplitude_to_db(rms);
        let now_silent = db < self.threshold_db;

        // Only log on state transitions (or first detection)
        if !self.initialized {
            self.initialized = true;
            self.is_silent = now_silent;
            if now_silent {
                println!("[SilenceDetector] Silence detected (initial state)");
            } else {
                println!("[SilenceDetector] Sound detected (initial state)");
            }
        } else if now_silent != self.is_silent {
            self.is_silent = now_silent;
            if now_silent {
                println!("[SilenceDetector] Silence detected");
            } else {
                println!("[SilenceDetector] Sound detected");
            }
        }
    }

    fn name(&self) -> &str {
        "SilenceDetector"
    }
}

/// Event payload for speech detection events
#[derive(Clone, Serialize)]
pub struct SpeechEventPayload {
    /// Duration in milliseconds (for speech-ended: how long the speech lasted)
    pub duration_ms: Option<u64>,
}

/// Speech detector that emits events when speech starts and ends.
/// Uses RMS amplitude with configurable threshold, hold time for debouncing,
/// and onset time to filter out brief non-speech sounds.
pub struct SpeechDetector {
    /// Threshold in dB below which audio is considered silent (default: -30.0)
    threshold_db: f32,
    /// Hold time in samples before emitting speech-ended event
    hold_samples: u32,
    /// Onset time in samples - sound must persist this long before considered speech
    onset_samples: u32,
    /// Sample rate for time calculations
    sample_rate: u32,
    /// Current speech state (true = speaking, false = silent)
    is_speaking: bool,
    /// Whether we're in the "maybe speaking" state (sound detected but not yet confirmed)
    is_pending_speech: bool,
    /// Counter for onset time during potential speech
    onset_sample_count: u32,
    /// Counter for hold time during silence
    silence_sample_count: u32,
    /// Counter for speech duration (from confirmed start)
    speech_sample_count: u64,
    /// Whether we've initialized (first sample processed)
    initialized: bool,
}

impl SpeechDetector {
    /// Create a new speech detector with specified sample rate.
    /// Uses default threshold (-30 dB), hold time (300ms), and onset time (100ms).
    pub fn new(sample_rate: u32) -> Self {
        Self::with_config(sample_rate, -30.0, 300, 100)
    }

    /// Create a new speech detector with custom configuration.
    /// 
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `threshold_db` - Threshold in dB below which audio is considered silent
    /// * `hold_time_ms` - Time in milliseconds to wait before emitting speech-ended
    /// * `onset_time_ms` - Time in milliseconds sound must persist before considered speech
    pub fn with_config(sample_rate: u32, threshold_db: f32, hold_time_ms: u32, onset_time_ms: u32) -> Self {
        let hold_samples = (sample_rate as u64 * hold_time_ms as u64 / 1000) as u32;
        let onset_samples = (sample_rate as u64 * onset_time_ms as u64 / 1000) as u32;
        Self {
            threshold_db,
            hold_samples,
            onset_samples,
            sample_rate,
            is_speaking: false,
            is_pending_speech: false,
            onset_sample_count: 0,
            silence_sample_count: 0,
            speech_sample_count: 0,
            initialized: false,
        }
    }

    /// Calculate RMS amplitude of samples
    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Convert linear amplitude to decibels
    fn amplitude_to_db(amplitude: f32) -> f32 {
        if amplitude <= 0.0 {
            return f32::NEG_INFINITY;
        }
        20.0 * amplitude.log10()
    }

    /// Convert sample count to milliseconds
    fn samples_to_ms(&self, samples: u64) -> u64 {
        samples * 1000 / self.sample_rate as u64
    }
}

impl AudioProcessor for SpeechDetector {
    fn process(&mut self, samples: &[f32], app_handle: &AppHandle) {
        let rms = Self::calculate_rms(samples);
        let db = Self::amplitude_to_db(rms);
        let is_sound = db >= self.threshold_db;

        if !self.initialized {
            self.initialized = true;
            // Don't emit on first sample - wait for proper onset
            return;
        }

        if is_sound {
            // Sound detected
            self.silence_sample_count = 0;
            
            if self.is_speaking {
                // Continue confirmed speech
                self.speech_sample_count += samples.len() as u64;
            } else if self.is_pending_speech {
                // Continue pending speech, check if onset time reached
                self.onset_sample_count += samples.len() as u32;
                if self.onset_sample_count >= self.onset_samples {
                    // Confirm speech started
                    self.is_speaking = true;
                    self.is_pending_speech = false;
                    self.speech_sample_count = self.onset_sample_count as u64 + samples.len() as u64;
                    let _ = app_handle.emit("speech-started", SpeechEventPayload { duration_ms: None });
                    println!("[SpeechDetector] Speech started");
                }
            } else {
                // Start pending speech
                self.is_pending_speech = true;
                self.onset_sample_count = samples.len() as u32;
            }
        } else {
            // Silence detected
            if self.is_pending_speech {
                // Cancel pending speech - it was just a brief sound
                self.is_pending_speech = false;
                self.onset_sample_count = 0;
            } else if self.is_speaking {
                self.silence_sample_count += samples.len() as u32;
                
                // Check if hold time has elapsed
                if self.silence_sample_count >= self.hold_samples {
                    // Emit speech-ended with duration
                    let duration_ms = self.samples_to_ms(self.speech_sample_count);
                    self.is_speaking = false;
                    self.speech_sample_count = 0;
                    let _ = app_handle.emit("speech-ended", SpeechEventPayload { duration_ms: Some(duration_ms) });
                    println!("[SpeechDetector] Speech ended (duration: {}ms)", duration_ms);
                }
            }
        }
    }

    fn name(&self) -> &str {
        "SpeechDetector"
    }
}
