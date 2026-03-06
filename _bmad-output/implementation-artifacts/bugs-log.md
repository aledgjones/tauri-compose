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
