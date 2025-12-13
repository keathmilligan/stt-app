## Context

SuperFlow's vision describes a multi-phase audio processing pipeline where cadence analysis enables responsive, natural conversation. Phase 1 requires detecting natural speech pauses vs. intentional breaks to know *when* to respond. The current `SilenceDetector` processor logs state transitions but doesn't communicate them to the frontend.

This change adds event emission to enable the frontend to react to speech boundaries, laying groundwork for the adaptive timeout and acknowledgment feedback described in the vision.

## Goals / Non-Goals

**Goals:**
- Emit `speech-started` and `speech-ended` events to the frontend when voice activity changes
- Provide configurable detection parameters (threshold, hold time) for tuning
- Maintain low-latency, non-blocking behavior in the audio callback
- Keep the processor architecture extensible for future cadence analysis

**Non-Goals:**
- Implement full cadence analysis (Phase 1 vision) - that's future work
- Add adaptive timeout logic - this change only provides the events
- Persist or learn user speech patterns - future enhancement
- Distinguish "thinking pauses" from "end-of-thought pauses" - requires more sophisticated analysis

## Decisions

### Decision: Event-based communication via Tauri AppHandle

The processor needs to communicate state changes to the frontend. Options considered:

1. **Polling from frontend** - Frontend periodically queries backend state
   - Pro: Simple to implement
   - Con: Adds latency, increases IPC overhead, doesn't feel responsive

2. **Callback-based processor** - Processor accepts a closure to invoke on events
   - Pro: Decoupled from Tauri
   - Con: Complicates ownership (closure must be `Send + Sync`), harder to test

3. **Channel-based (mpsc)** - Processor sends events to a channel, separate thread emits
   - Pro: Decouples detection from emission
   - Con: Adds complexity, extra thread, potential for event lag

4. **Direct AppHandle emission** - Processor holds `AppHandle` clone and emits directly
   - Pro: Simplest, uses existing Tauri pattern, low latency
   - Con: Couples processor to Tauri

**Decision**: Use direct `AppHandle` emission. The coupling to Tauri is acceptable since this is a Tauri app, and it matches the existing `audio-samples` event pattern. Keeps latency minimal.

### Decision: Modify AudioProcessor trait to support event emission

The current `AudioProcessor::process(&mut self, samples: &[f32])` signature doesn't provide access to `AppHandle`. Options:

1. **Store AppHandle in processor** - Pass at construction, clone into struct
   - Pro: Each processor owns its handle
   - Con: Requires passing handle through `set_processing_enabled`

2. **Pass AppHandle to process()** - Change signature to `process(&mut self, samples: &[f32], app: &AppHandle)`
   - Pro: Explicit dependency, flexible
   - Con: All processors must accept it even if unused

3. **Event callback pattern** - Pass a generic event emitter closure
   - Pro: Testable, decoupled
   - Con: Complex lifetimes, harder to reason about

**Decision**: Pass `AppHandle` to `process()`. This is explicit about dependencies and matches Rust conventions. Processors that don't need it simply ignore the parameter. Minor breaking change to the trait, but internal API only.

### Decision: Hold time for debouncing

Speech detection needs debouncing to avoid rapid-fire events from brief pauses. A "hold time" delays the `speech-ended` event until silence persists for a minimum duration.

**Decision**: Use configurable hold time, default 300ms. This provides:
- Enough tolerance for natural inter-word pauses
- Quick enough response for end-of-utterance detection
- Adjustable for future cadence analysis needs

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Event emission in audio callback adds latency | `AppHandle::emit` is non-blocking, uses internal channel |
| Hold time delays speech-ended event | Acceptable trade-off for stability; 300ms is fast enough for human perception |
| Trait change breaks existing processors | Internal API only; update `SilenceDetector` in same change |

## Open Questions

1. Should we emit event payloads with additional data (e.g., confidence level, duration)?
   - Initial implementation: Keep payloads minimal. Add data as cadence analysis needs emerge.

2. Should `SilenceDetector` be deprecated in favor of new `SpeechDetector`?
   - Keep both for now. `SilenceDetector` is useful for debugging. Can consolidate later.
