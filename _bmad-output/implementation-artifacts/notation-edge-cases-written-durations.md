# Notation Edge Cases — Written Durations

## F-001

- **ID:** F-001
- **Area:** Unsupported time-signature branches
- **Scenario:** `Compound`, `Complex`, and `Open` time signatures reach `get_written_durations()`.
- **Current Behavior:** The parser panics via `todo!()` in the current unsupported branches.
- **Why Suspicious:** The boundary is intentional today, but the failure mode stays abrupt and easy to miss without explicit characterization.
- **Preserved In This Spec:** Yes. Regression tests now lock the panic boundary in place.
- **Recommended Follow-up:** Replace `todo!()` with explicit unsupported-path errors once notation support work is planned.

## F-002

- **ID:** F-002
- **Area:** Four-beat preservation patterns
- **Scenario:** `qhq`, `qm.`, `m.q`, and full-span offbeat patterns bypass the normal middle split in simple four-beat units.
- **Current Behavior:** The parser preserves those spans exactly and exits early from the split path.
- **Why Suspicious:** These are musically motivated exceptions embedded directly in recursive split logic, so accidental edits can be hard to spot.
- **Preserved In This Spec:** Yes. Scenario and end-to-end regression tests cover representative preserved spans.
- **Recommended Follow-up:** Consider documenting each preserved pattern with notation examples near the production helper path.

## F-003

- **ID:** F-003
- **Area:** Tie propagation during structural splits
- **Scenario:** A notation event is split into two written events.
- **Current Behavior:** The left fragment receives ties to its own tone ids, while the right fragment keeps the original tie set.
- **Why Suspicious:** The behavior is correct for the current model, but it is subtle and easy to break during refactors.
- **Preserved In This Spec:** Yes. Direct regression coverage now asserts the exact left/right tie distribution.
- **Recommended Follow-up:** Extract the tie-write rule into a named helper if later refactors touch split construction again.

## F-004

- **ID:** F-004
- **Area:** Fixed eighth-grid assumptions
- **Scenario:** Three-beat split rules inspect off-grid starts using eighth-note spacing regardless of the original beat size.
- **Current Behavior:** Off-grid detection uses the fixed eighth-note tick value to force beat-boundary splits.
- **Why Suspicious:** The logic is deliberate for current simple-meter handling, but it couples readability to an implicit grid assumption.
- **Preserved In This Spec:** Yes. The behavior remains unchanged.
- **Recommended Follow-up:** Revisit whether the grid should be derived from time-signature semantics if broader meter support is added.

## F-005

- **ID:** F-005
- **Area:** 3/4 beat-two spanning notes
- **Scenario:** In `3/4`, a half note starts on beat 2 and runs to the bar end.
- **Current Behavior:** The parser currently leaves the beat-two span as a single half note rather than rewriting it as quarter-tied-quarter.
- **Why Suspicious:** The result is legal in tick terms, but it weakens the visible beat-3 division in a meter where readability is especially sensitive.
- **Preserved In This Spec:** Yes. High-level regression tests now characterize the current output explicitly.
- **Recommended Follow-up:** Revisit whether late-bar spans in `3/4` should prefer beat-explicit ties over longer unsplit values.

## F-006

- **ID:** F-006
- **Area:** 3/4 syncopated beat-two patterns
- **Scenario:** In `3/4`, patterns such as quarter + dotted-quarter + eighth, or an eighth-led quarter spanning beat 2, were expected to split at the beat.
- **Current Behavior:** The parser often preserves the middle dotted or quarter span unsplit across beat 2 instead of forcing a beat-visible rewrite.
- **Why Suspicious:** These are exactly the rhythms most likely to start looking like compound-meter groupings or otherwise obscure the beat map in simple triple time.
- **Preserved In This Spec:** Yes. The new scenario suite locks down the current rendering so future changes are visible immediately.
- **Recommended Follow-up:** Audit `3/4`-specific readability rules separately from generic simple-meter recursion, especially for beat-two entries and dotted spans.

## F-007

- **ID:** F-007
- **Area:** 3/4 vs 6/8 visual ambiguity
- **Scenario:** `3/4` shapes with dotted-quarter centers or paired dotted-quarter gestures can resemble compound-meter grouping even though they must read as simple triple time.
- **Current Behavior:** Some patterns are rewritten to avoid the `6/8` look, but others remain preserved as larger unsplit spans.
- **Why Suspicious:** The mixed behavior suggests the parser has important special cases, but not yet a complete, easy-to-reason-about readability policy for triple meter.
- **Preserved In This Spec:** Yes. The scenario tests now document both the preserved and rewritten cases.
- **Recommended Follow-up:** Define an explicit policy for `3/4` readability patterns and align the helper rules to that policy instead of relying on branch-by-branch exceptions.

## F-008

- **ID:** F-008
- **Area:** 4/4 near-miss offbeat intrusions
- **Scenario:** A bar almost matches a preserved offbeat syncopation pattern, but an extra on-beat event intrudes.
- **Current Behavior:** The parser can resolve the bar into overlapping short fragments, including merged tick entries, rather than into a cleaner beat-explicit rewrite.
- **Why Suspicious:** The output is still characterized now, but it is harder to reason about musically and may hide whether the notation shape is genuinely intentional or an artifact of split ordering.
- **Preserved In This Spec:** Yes. The regression suite now records the exact current output.
- **Recommended Follow-up:** Inspect whether near-miss offbeat cases should normalize to simpler beat-aligned notation once the intended preservation boundaries are formalized.
