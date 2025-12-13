# Design: Enhanced Speech Detection Algorithm

## Context
The speech detector must distinguish human speech from non-speech audio (keyboard clicks, mouse sounds, coughs, ambient noise) in real-time with minimal latency. The current implementation only uses RMS amplitude, which cannot differentiate between loud non-speech sounds and actual speech.

**Critical requirement**: The detector must reliably capture soft/whispered speech while still rejecting non-speech sounds. Whispered speech has different acoustic characteristics than voiced speech (lower amplitude, higher ZCR, breathier spectral profile).

## Goals
- Significantly reduce false positives from non-speech sounds
- Reliably detect soft and whispered speech (no false negatives for quiet speech)
- Maintain sub-millisecond processing latency per audio buffer
- Keep the implementation simple with no external dependencies (no ML models, no FFT libraries)
- Preserve existing onset/hold time debouncing behavior

## Non-Goals
- Perfect accuracy (some edge cases acceptable)
- Speaker identification or voice recognition
- Noise cancellation or audio enhancement

## Decisions

### Decision 1: Multi-Feature Detection Approach
Use a combination of three fast, complementary features:

1. **RMS Amplitude** (existing) - Basic energy detection
2. **Zero-Crossing Rate (ZCR)** - Distinguishes voiced speech from transients
3. **Spectral Centroid Approximation** - Identifies speech-band frequency content

**Rationale**: Each feature is O(n) complexity, requires no FFT, and captures different acoustic properties. Together they provide robust discrimination:
- Keyboard clicks: High amplitude but very high ZCR (>0.35) and high spectral centroid (>5kHz)
- Voiced speech: Moderate ZCR (0.02-0.20) and centroid in speech band (300-3500 Hz)
- Whispered speech: Higher ZCR (0.15-0.35) and higher centroid (500-5000 Hz) due to breathy/fricative content
- Low rumble: Very low ZCR (<0.01) and low centroid (<200 Hz)

### Decision 2: Dual-Mode Detection (Voiced + Whispered)

Rather than a single set of thresholds, use two detection modes that run in parallel:

**Voiced Speech Mode:**
| Feature | Range |
|---------|-------|
| RMS (dB) | > -40 dB |
| ZCR | 0.01 - 0.20 |
| Spectral Centroid | 250-4000 Hz |

**Whisper/Soft Speech Mode:**
| Feature | Range |
|---------|-------|
| RMS (dB) | > -50 dB (more sensitive) |
| ZCR | 0.10 - 0.40 (higher range for breathy sounds) |
| Spectral Centroid | 400-6000 Hz (shifted up for fricatives) |
| Temporal consistency | Must persist for 150ms+ (filters transients) |

A buffer is classified as speech if it matches EITHER mode. The whisper mode uses a longer onset requirement to prevent false triggers from brief noises that happen to fall in the whisper feature range.

**Rationale**: Whispered speech is acoustically closer to noise than voiced speech, so we need a separate detection path with compensating temporal constraints.

### Decision 3: Impulsive Transient Rejection

Keyboard clicks and similar impulsive sounds have a distinctive signature:
- Very high ZCR (>0.35)
- Very high spectral centroid (>5000 Hz)
- Short duration (<50ms typically)
- Sharp amplitude attack (high crest factor)

Add explicit transient rejection:
- If ZCR > 0.40 AND centroid > 5500 Hz → classify as transient, not speech
- This takes precedence over whisper mode detection

### Decision 4: Spectral Centroid Without FFT
Approximate spectral centroid using the first-difference method:
```
centroid_approx = sample_rate * mean(|diff(samples)|) / (2 * mean(|samples|))
```

This provides a frequency estimate correlated with true spectral centroid at ~10x lower computational cost than FFT.

**Rationale**: True FFT-based spectral analysis would add latency and complexity. The approximation is sufficient for speech/non-speech discrimination.

### Decision 5: Adaptive Amplitude Threshold

Use a lower base threshold (-50 dB) to catch whispered speech, but require feature validation to prevent noise triggering:
- Below -50 dB: Always silence (noise floor)
- -50 to -35 dB: Low amplitude zone - requires whisper mode feature match
- Above -35 dB: Normal zone - requires voiced or whisper mode match

This ensures soft speech is captured while the feature checks filter out low-level noise.

## Feature Threshold Summary

| Feature | Voiced Mode | Whisper Mode | Transient Reject |
|---------|-------------|--------------|------------------|
| RMS (dB) | > -40 | > -50 | any |
| ZCR | 0.01-0.20 | 0.10-0.40 | > 0.40 |
| Centroid | 250-4000 Hz | 400-6000 Hz | > 5500 Hz |
| Onset time | 100ms | 150ms | N/A |

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Whisper mode may catch some noise | Longer onset time (150ms) filters brief sounds |
| Very breathy speech near transient boundary | Transient reject requires BOTH high ZCR AND high centroid |
| Tuning may need adjustment per environment | Expose sensitivity parameter for future tuning |
| Sibilants (s, sh, f) in voiced speech | Handled by temporal continuity - surrounded by voiced frames |

## Algorithm Flow

```
For each audio buffer:
  1. Calculate RMS amplitude -> dB
  2. Calculate ZCR (zero crossings / sample count)
  3. Calculate spectral centroid approximation
  
  4. Check transient rejection:
     If ZCR > 0.40 AND centroid > 5500 Hz:
       -> Mark as transient noise, reset onset timers, continue
  
  5. Check voiced speech mode:
     If dB > -40 AND ZCR in [0.01, 0.20] AND centroid in [250, 4000]:
       -> Mark as voiced speech candidate
  
  6. Check whisper mode:
     If dB > -50 AND ZCR in [0.10, 0.40] AND centroid in [400, 6000]:
       -> Mark as whisper speech candidate
  
  7. Apply onset timer:
     - Voiced candidate: use 100ms onset
     - Whisper candidate (not voiced): use 150ms onset
     - Neither: reset onset timers
  
  8. Apply hold time for speech-ended detection (existing logic)
```

## Open Questions
- Should a unified "sensitivity" slider be exposed that adjusts multiple thresholds together? (Recommendation: Defer until real-world testing identifies need)
