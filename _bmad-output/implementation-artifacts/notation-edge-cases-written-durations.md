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
