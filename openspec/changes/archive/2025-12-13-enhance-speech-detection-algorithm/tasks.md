## 1. Core Feature Functions
- [x] 1.1 Add ZCR calculation function to `processor.rs`
- [x] 1.2 Add spectral centroid approximation function (first-difference method) to `processor.rs`

## 2. Dual-Mode Detection Structure
- [x] 2.1 Add configuration struct/fields for voiced mode parameters (threshold, ZCR range, centroid range, onset time)
- [x] 2.2 Add configuration struct/fields for whisper mode parameters (threshold, ZCR range, centroid range, onset time)
- [x] 2.3 Add transient rejection thresholds (ZCR > 0.40, centroid > 5500 Hz)
- [x] 2.4 Update `SpeechDetector::with_config` to accept mode-specific parameters

## 3. Detection Logic Implementation
- [x] 3.1 Implement transient rejection check (runs first, takes precedence)
- [x] 3.2 Implement voiced mode feature matching
- [x] 3.3 Implement whisper mode feature matching
- [x] 3.4 Add separate onset tracking for voiced vs whisper candidates
- [x] 3.5 Integrate dual-mode logic into `SpeechDetector::process`

## 4. State Machine Updates
- [x] 4.1 Update onset timer to support mode-specific durations (100ms voiced, 150ms whisper)
- [x] 4.2 Ensure transient detection resets onset timers
- [x] 4.3 Handle voiced-to-whisper and whisper-to-voiced transitions during speech

## 5. Default Parameter Updates
- [x] 5.1 Set voiced threshold to -40dB (from current -30dB)
- [x] 5.2 Set whisper threshold to -50dB
- [x] 5.3 Set voiced onset to 100ms, whisper onset to 150ms

## 6. Validation
- [x] 6.1 Test with normal voiced speech - verify events emit correctly
- [x] 6.2 Test with soft/whispered speech - verify detection works
- [x] 6.3 Test with keyboard clicks - verify rejection (no false positives)
- [x] 6.4 Test with mouse clicks - verify rejection
- [x] 6.5 Test with ambient noise/rumble - verify rejection
- [x] 6.6 Verify latency remains acceptable (no perceptible delay)
- [x] 6.7 Test transitions between whisper and voiced speech

## 7. Documentation
- [x] 7.1 Update code comments explaining dual-mode approach
- [x] 7.2 Document feature threshold rationale
- [x] 7.3 Add inline comments for ZCR and centroid calculations
