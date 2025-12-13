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

/// Configuration for a speech detection mode (voiced or whisper)
#[derive(Clone)]
struct SpeechModeConfig {
    /// Minimum amplitude threshold in dB
    threshold_db: f32,
    /// ZCR range (min, max) - normalized as crossings per sample
    zcr_range: (f32, f32),
    /// Spectral centroid range in Hz (min, max)
    centroid_range: (f32, f32),
    /// Onset time in samples before confirming speech
    onset_samples: u32,
}

/// Speech detector that emits events when speech starts and ends.
/// 
/// Uses multi-feature analysis for robust speech detection:
/// - RMS amplitude for basic energy detection
/// - Zero-Crossing Rate (ZCR) to distinguish voiced speech from transients
/// - Spectral centroid approximation to identify speech-band frequency content
/// 
/// Implements dual-mode detection:
/// - **Voiced mode**: For normal speech (lower ZCR, speech-band centroid)
/// - **Whisper mode**: For soft/breathy speech (higher ZCR, broader centroid range)
/// 
/// Explicit transient rejection filters keyboard clicks and similar impulsive sounds.
pub struct SpeechDetector {
    /// Sample rate for time/frequency calculations
    sample_rate: u32,
    /// Voiced speech detection configuration
    voiced_config: SpeechModeConfig,
    /// Whisper speech detection configuration  
    whisper_config: SpeechModeConfig,
    /// Transient rejection: ZCR threshold (reject if above)
    transient_zcr_threshold: f32,
    /// Transient rejection: centroid threshold in Hz (reject if above, combined with ZCR)
    transient_centroid_threshold: f32,
    /// Hold time in samples before emitting speech-ended event
    hold_samples: u32,
    /// Current speech state (true = speaking, false = silent)
    is_speaking: bool,
    /// Whether we're in "pending voiced" state
    is_pending_voiced: bool,
    /// Whether we're in "pending whisper" state
    is_pending_whisper: bool,
    /// Counter for voiced onset time
    voiced_onset_count: u32,
    /// Counter for whisper onset time
    whisper_onset_count: u32,
    /// Counter for hold time during silence
    silence_sample_count: u32,
    /// Counter for speech duration (from confirmed start)
    speech_sample_count: u64,
    /// Whether we've initialized (first sample processed)
    initialized: bool,
}

impl SpeechDetector {
    /// Create a new speech detector with specified sample rate.
    /// Uses default dual-mode configuration optimized for speech detection.
    pub fn new(sample_rate: u32) -> Self {
        Self::with_defaults(sample_rate)
    }

    /// Create a speech detector with default dual-mode configuration.
    /// 
    /// Default parameters:
    /// - Voiced mode: -40dB threshold, ZCR 0.01-0.20, centroid 250-4000Hz, 100ms onset
    /// - Whisper mode: -50dB threshold, ZCR 0.10-0.40, centroid 400-6000Hz, 150ms onset
    /// - Transient rejection: ZCR > 0.40 AND centroid > 5500Hz
    /// - Hold time: 300ms
    pub fn with_defaults(sample_rate: u32) -> Self {
        let hold_samples = (sample_rate as u64 * 300 / 1000) as u32;
        
        Self {
            sample_rate,
            voiced_config: SpeechModeConfig {
                threshold_db: -40.0,
                zcr_range: (0.01, 0.20),
                centroid_range: (250.0, 4000.0),
                onset_samples: (sample_rate as u64 * 100 / 1000) as u32,
            },
            whisper_config: SpeechModeConfig {
                threshold_db: -50.0,
                zcr_range: (0.10, 0.40),
                centroid_range: (400.0, 6000.0),
                onset_samples: (sample_rate as u64 * 150 / 1000) as u32,
            },
            transient_zcr_threshold: 0.40,
            transient_centroid_threshold: 5500.0,
            hold_samples,
            is_speaking: false,
            is_pending_voiced: false,
            is_pending_whisper: false,
            voiced_onset_count: 0,
            whisper_onset_count: 0,
            silence_sample_count: 0,
            speech_sample_count: 0,
            initialized: false,
        }
    }

    /// Create a new speech detector with custom configuration.
    /// 
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `voiced_threshold_db` - Threshold for voiced speech detection
    /// * `whisper_threshold_db` - Threshold for whisper speech detection (should be lower)
    /// * `hold_time_ms` - Time in milliseconds to wait before emitting speech-ended
    pub fn with_config(sample_rate: u32, voiced_threshold_db: f32, whisper_threshold_db: f32, hold_time_ms: u32) -> Self {
        let mut detector = Self::with_defaults(sample_rate);
        detector.voiced_config.threshold_db = voiced_threshold_db;
        detector.whisper_config.threshold_db = whisper_threshold_db;
        detector.hold_samples = (sample_rate as u64 * hold_time_ms as u64 / 1000) as u32;
        detector
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

    /// Calculate Zero-Crossing Rate (ZCR) of samples.
    /// 
    /// ZCR is the rate at which the signal changes sign (crosses zero).
    /// Returns normalized value: crossings per sample (0.0 to 0.5 max).
    /// 
    /// Acoustic characteristics:
    /// - Voiced speech: ~0.02-0.15 (periodic vocal cord vibration)
    /// - Whispered speech: ~0.15-0.35 (breathy, more noise-like)
    /// - Clicks/transients: >0.35 (impulsive, high-frequency content)
    /// - Low rumble: <0.01 (slow oscillation)
    fn calculate_zcr(samples: &[f32]) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }
        
        let mut crossings = 0u32;
        for i in 1..samples.len() {
            // Count sign changes (one sample positive, next negative or vice versa)
            if (samples[i] >= 0.0) != (samples[i - 1] >= 0.0) {
                crossings += 1;
            }
        }
        
        // Normalize: crossings per sample
        crossings as f32 / (samples.len() - 1) as f32
    }

    /// Estimate spectral centroid using first-difference approximation.
    /// 
    /// This provides a frequency estimate without FFT by using:
    /// centroid ≈ sample_rate * mean(|diff(samples)|) / (2 * mean(|samples|))
    /// 
    /// The intuition: higher frequency signals have larger sample-to-sample
    /// differences relative to their amplitude.
    /// 
    /// Returns frequency estimate in Hz.
    /// 
    /// Acoustic characteristics:
    /// - Voiced speech: ~300-3500 Hz (fundamental + harmonics)
    /// - Whispered speech: ~500-5000 Hz (shifted up, more fricative)
    /// - Keyboard clicks: >5000 Hz (high-frequency transient)
    /// - Low rumble: <200 Hz
    fn estimate_spectral_centroid(&self, samples: &[f32]) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }
        
        // Calculate mean absolute difference (approximates high-frequency content)
        let mut diff_sum = 0.0f32;
        for i in 1..samples.len() {
            diff_sum += (samples[i] - samples[i - 1]).abs();
        }
        let mean_diff = diff_sum / (samples.len() - 1) as f32;
        
        // Calculate mean absolute amplitude
        let mean_abs: f32 = samples.iter().map(|s| s.abs()).sum::<f32>() / samples.len() as f32;
        
        // Avoid division by zero
        if mean_abs < 1e-10 {
            return 0.0;
        }
        
        // Approximate centroid frequency
        // Factor of 2 comes from Nyquist relationship
        self.sample_rate as f32 * mean_diff / (2.0 * mean_abs)
    }

    /// Check if features indicate a transient sound (keyboard click, etc.)
    /// Transients have both high ZCR AND high spectral centroid.
    fn is_transient(&self, zcr: f32, centroid: f32) -> bool {
        zcr > self.transient_zcr_threshold && centroid > self.transient_centroid_threshold
    }

    /// Check if features match voiced speech mode
    fn matches_voiced_mode(&self, db: f32, zcr: f32, centroid: f32) -> bool {
        db >= self.voiced_config.threshold_db
            && zcr >= self.voiced_config.zcr_range.0
            && zcr <= self.voiced_config.zcr_range.1
            && centroid >= self.voiced_config.centroid_range.0
            && centroid <= self.voiced_config.centroid_range.1
    }

    /// Check if features match whisper speech mode
    fn matches_whisper_mode(&self, db: f32, zcr: f32, centroid: f32) -> bool {
        db >= self.whisper_config.threshold_db
            && zcr >= self.whisper_config.zcr_range.0
            && zcr <= self.whisper_config.zcr_range.1
            && centroid >= self.whisper_config.centroid_range.0
            && centroid <= self.whisper_config.centroid_range.1
    }

    /// Convert sample count to milliseconds
    fn samples_to_ms(&self, samples: u64) -> u64 {
        samples * 1000 / self.sample_rate as u64
    }

    /// Reset all onset tracking state
    fn reset_onset_state(&mut self) {
        self.is_pending_voiced = false;
        self.is_pending_whisper = false;
        self.voiced_onset_count = 0;
        self.whisper_onset_count = 0;
    }
}

impl AudioProcessor for SpeechDetector {
    fn process(&mut self, samples: &[f32], app_handle: &AppHandle) {
        // Step 1: Calculate all features
        let rms = Self::calculate_rms(samples);
        let db = Self::amplitude_to_db(rms);
        let zcr = Self::calculate_zcr(samples);
        let centroid = self.estimate_spectral_centroid(samples);

        if !self.initialized {
            self.initialized = true;
            // Don't emit on first sample - wait for proper onset
            return;
        }

        // Step 2: Check for transient rejection (keyboard clicks, etc.)
        // Transients have both high ZCR AND high centroid
        if self.is_transient(zcr, centroid) {
            // Reset onset timers - transient breaks any pending speech detection
            self.reset_onset_state();
            // Don't affect confirmed speech within hold time
            if !self.is_speaking {
                return;
            }
        }

        // Step 3: Check feature matching for both modes
        let is_voiced = self.matches_voiced_mode(db, zcr, centroid);
        let is_whisper = self.matches_whisper_mode(db, zcr, centroid);
        let is_speech_candidate = is_voiced || is_whisper;

        if is_speech_candidate {
            // Sound matching speech features detected
            self.silence_sample_count = 0;

            if self.is_speaking {
                // Continue confirmed speech
                self.speech_sample_count += samples.len() as u64;
            } else {
                // Handle onset accumulation based on which mode matches
                let samples_len = samples.len() as u32;

                if is_voiced {
                    // Accumulate voiced onset
                    if !self.is_pending_voiced {
                        self.is_pending_voiced = true;
                        self.voiced_onset_count = samples_len;
                    } else {
                        self.voiced_onset_count += samples_len;
                    }

                    // Check if voiced onset threshold reached
                    if self.voiced_onset_count >= self.voiced_config.onset_samples {
                        self.is_speaking = true;
                        self.speech_sample_count = self.voiced_onset_count as u64;
                        self.reset_onset_state();
                        let _ = app_handle.emit("speech-started", SpeechEventPayload { duration_ms: None });
                        println!("[SpeechDetector] Speech started (voiced mode)");
                        return;
                    }
                }

                if is_whisper {
                    // Accumulate whisper onset (can run in parallel with voiced)
                    if !self.is_pending_whisper {
                        self.is_pending_whisper = true;
                        self.whisper_onset_count = samples_len;
                    } else {
                        self.whisper_onset_count += samples_len;
                    }

                    // Check if whisper onset threshold reached (and voiced didn't already trigger)
                    if !self.is_speaking && self.whisper_onset_count >= self.whisper_config.onset_samples {
                        self.is_speaking = true;
                        self.speech_sample_count = self.whisper_onset_count as u64;
                        self.reset_onset_state();
                        let _ = app_handle.emit("speech-started", SpeechEventPayload { duration_ms: None });
                        println!("[SpeechDetector] Speech started (whisper mode)");
                        return;
                    }
                }
            }
        } else {
            // No speech-like features detected
            if self.is_pending_voiced || self.is_pending_whisper {
                // Cancel pending speech - features didn't persist
                self.reset_onset_state();
            }
            
            if self.is_speaking {
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
