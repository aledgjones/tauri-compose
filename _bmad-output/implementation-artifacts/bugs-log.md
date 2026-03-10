# Bugs Log

## B-001 Hidden time signatures add horizontal spacing

- **Date:** 2026-03-06
- **Area:** notation spacing / visual debugger fixtures
- **Reported via:** `barline-repeat-gallery` hidden time-signature case
- **Symptom:** A hidden time signature still reserved horizontal space, so repeat-boundary fixture layouts showed an unwanted gap before the following notation.
- **Root cause:** `TimeSignature::metrics()` returned the standard numeric bounding box for all draw types, including `TimeSignatureDrawType::Hidden`.
- **Fix:** Return zero width and zero right padding for hidden time signatures, and add a regression test covering hidden metrics.
- **Status:** Fixed

## B-002 Key signatures reserve spacing even when no stave can render them

- **Date:** 2026-03-06
- **Area:** notation spacing / clef-key-signature interaction
- **Reported via:** `signature-transition-gallery` stacked-repeat key-signature case
- **Symptom:** A key signature could disappear on percussion-only or non-renderable clef contexts while the layout still reserved horizontal space for it.
- **Root cause:** Horizontal spacing always used the flow-master key-signature metrics and did not check whether any stave had a current clef that could actually render the key signature.
- **Fix:** Only reserve key-signature width when at least one stave has a renderable clef context at that tick; add regression coverage for percussion-clef-only contexts.
- **Status:** Fixed

## B-003 Hidden clef was modeling absence as a draw type

- **Date:** 2026-03-06
- **Area:** clef model / debugger fixtures
- **Reported via:** repeat-boundary visual fixture review
- **Symptom:** The score model included a `Hidden` clef draw type even though “no clef change” should be represented by the absence of a clef entry, not by a hidden clef symbol.
- **Root cause:** `Hidden` had been acting as an internal sentinel to suppress rendering while still satisfying APIs that expected a `Clef` value.
- **Fix:** Remove `Hidden` from `ClefDrawType`, update tests and fixtures to omit clef changes instead, and keep percussion as the only non-key-signature-rendering clef variant.
- **Status:** Fixed

## B-004 Cancelling key signatures were not reserving width

- **Date:** 2026-03-06
- **Area:** notation spacing / key-signature cancellation
- **Reported via:** per-stave key-signature spacing review
- **Symptom:** A `0`-offset key signature following a non-zero signature did not reserve horizontal space for cancelling natural symbols.
- **Root cause:** The per-stave spacing pass checked renderability against the current `0`-offset signature, whose `pattern()` is `None`, instead of against the previous signature that supplies the cancellation glyph count.
- **Fix:** When the current key signature has `offset == 0`, evaluate renderability and width using the previous key signature at that tick, then reserve the cancelling-natural width accordingly.
- **Status:** Fixed

## B-005 Compound-beat leading rests can be written too small

- **Date:** 2026-03-10
- **Area:** written-duration parser / compound-meter rest spelling
- **Reported via:** ongoing 3/4 versus 6/8 parity review
- **Symptom:** In compound times, rests inside a compound beat can be fragmented into multiple eighth rests when they should be written as a larger leading rest value.
- **Expected rule:** Work in beat-local spellings for compound beats. `[8N, 8R, 8R]` is already correct. `[4R, 8N]` is correct and must not be rewritten as `[8R, 8R, 8N]`.
- **Scope note:** This rule applies to compound times only. `3/8` is excluded and should continue to behave like a halved version of `3/4`, not like the compound-beat grouping used by `6/8`, `9/8`, `12/8`, and `15/8`.
- **Required regression test:** Add a compound-meter scenario where a dotted-quarter beat contains `32` ticks of leading silence followed by an eighth note, and assert that the beat is written as `[4R, 8N]` rather than `[8R, 8R, 8N]`.
- **Suggested fixture shape:** `tone_specs: vec![(1, 0, 48, 60), (2, 80, 16, 64)]` under `make_time_signature_for_test(0, 6, NoteDuration::Eighth)`.
- **Status:** Open
