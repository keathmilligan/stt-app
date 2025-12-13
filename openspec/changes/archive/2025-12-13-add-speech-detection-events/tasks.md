## 1. Backend Implementation

- [x] 1.1 Modify `AudioProcessor` trait to accept `AppHandle` in `process()` method
- [x] 1.2 Update `SilenceDetector` to accept new signature (ignore AppHandle)
- [x] 1.3 Create `SpeechDetector` struct with configurable threshold and hold time
- [x] 1.4 Implement hold time tracking using sample count (derive from sample rate)
- [x] 1.5 Emit `speech-started` event on silence-to-speech transition
- [x] 1.6 Emit `speech-ended` event after hold time elapses during silence
- [x] 1.7 Update `audio.rs` to pass `AppHandle` to processor in callback

## 2. Frontend Integration

- [x] 2.1 Define TypeScript interface for speech event payloads
- [x] 2.2 Add event listeners for `speech-started` and `speech-ended`
- [x] 2.3 Add visual indicator or logging to confirm events are received

## 3. Processor Selection

- [x] 3.1 Replace `SilenceDetector` with `SpeechDetector` as default processor
- [x] 3.2 Pass sample rate to `SpeechDetector` for accurate hold time calculation

## 4. Verification

- [x] 4.1 Test speech-started event fires when speaking begins
- [x] 4.2 Test speech-ended event fires after silence exceeds hold time
- [x] 4.3 Test brief pauses don't trigger speech-ended (debouncing works)
- [x] 4.4 Test events don't fire when processing toggle is disabled
