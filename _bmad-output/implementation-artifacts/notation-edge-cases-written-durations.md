# Notation Edge Cases — Written Durations

## Agent Handoff Summary

This document now carries not only the edge cases, but also the validated musical context behind them. The key lesson from this review is that the written-duration parser is convention-driven, not reducible to one universal splitting rule. Future agents should assume that many of the "exceptions" are actually part of correctness.

### High-Level Conclusions

- The parser intentionally applies specific musical exception patterns before the more general simple-meter splitting rules.
- That design is correct for both `3/4` and `4/4`; many cases that look inconsistent at first are actually long-standing notation conventions.
- High-level scenario tests are essential here because they protect musical convention, not just mechanical tick math.
- For single-track notation, rhythmic events can be treated as one flattened rhythmic stream once a preservation pattern no longer matches.
- Multi-voice behavior is a separate musical concept and should not be inferred from current single-track written-duration behavior.

### Validated Status of Findings

- `F-001`: Known TODO, not a bug.
- `F-002`: Intentional `4/4` preservation-rule surface, not a bug.
- `F-003`: Intentional tie-propagation rule, not a bug.
- `F-004`: Intentional `3/4` eighth-grid offbeat rule, not a bug.
- `F-005`: Intentional `3/4` rule — a half note starting on beat 2 should remain unsplit.
- `F-006`: Fixed in current branch — in `3/4`, `quarter + dotted-quarter + eighth note` remains unsplit, while `quarter + dotted-quarter + eighth rest` now splits at beat 3.
- `F-007`: Intentional design approach — nuanced exception matching, not a bug.
- `F-008`: Intentional single-track fallback behavior after an offbeat-pattern match fails, not a bug.

### Important Musical Rules Confirmed During Review

- In `3/4`, a half note that begins exactly on beat 2 is correct when left unsplit.
- In `3/4`, dotted-quarter handling is subtle and context-sensitive; a final note and a final rest do not imply the same engraving choice.
- In `3/4` and `4/4`, exception-pattern matching exists because there is no single clean "always split here" rule that satisfies notation convention.
- In near-miss offbeat cases on a single track, losing the offbeat-pattern match means the parser should fall back to the flattened rhythmic stream rather than try to preserve the near-match visually.

### Guidance for Future Agents

- Do not assume that a surprising unsplit value is a bug; check whether it is a protected convention first.
- When changing `split_three_beat_simple_unit()` or the `4/4` preservation helpers, review the high-level scenario tests before proposing simplifications.
- `F-006` was the actionable notation bug from this review and is now fixed in the current branch.
- If future work introduces multi-voice stave logic, revisit single-track assumptions separately rather than altering current behavior implicitly.

## F-001

- **ID:** F-001
- **Area:** Unsupported time-signature branches
- **Scenario:** `Compound`, `Complex`, and `Open` time signatures reach `get_written_durations()`.
- **Current Behavior:** The parser panics via `todo!()` in the current unsupported branches.
- **Why Suspicious:** This is a deliberate current boundary, not an accidental regression, but the failure mode stays abrupt and easy to miss without explicit characterization.
- **Preserved In This Spec:** Yes. Regression tests now lock the panic boundary in place.
- **Status:** Known TODO, not currently treated as a bug.
- **Recommended Follow-up:** Keep as-is until compound/open meter support work is scheduled, then replace `todo!()` with explicit unsupported-path errors.

## F-002

- **ID:** F-002
- **Area:** Four-beat preservation patterns
- **Scenario:** `qhq`, `qm.`, `m.q`, and full-span offbeat patterns bypass the normal middle split in simple four-beat units.
- **Current Behavior:** The parser preserves those spans exactly and exits early from the split path.
- **Why Suspicious:** These are intentional musically motivated exceptions embedded directly in recursive split logic, so accidental edits can be hard to spot.
- **Preserved In This Spec:** Yes. Scenario and end-to-end regression tests cover representative preserved spans.
- **Status:** Known design rule, not currently treated as a bug.
- **Recommended Follow-up:** Keep these rules stable during future refactors and consider documenting each preserved pattern with notation examples near the production helper path.

## F-003

- **ID:** F-003
- **Area:** Tie propagation during structural splits
- **Scenario:** A notation event is split into two written events.
- **Current Behavior:** The left fragment receives ties to its own tone ids, while the right fragment keeps the original tie set.
- **Why Suspicious:** This is an intentional rule in the current model, but it is subtle and easy to break during refactors.
- **Preserved In This Spec:** Yes. Direct regression coverage now asserts the exact left/right tie distribution.
- **Status:** Known design rule, not currently treated as a bug.
- **Recommended Follow-up:** Preserve this rule during future refactors and extract the tie-write rule into a named helper if split construction changes again.

## F-004

- **ID:** F-004
- **Area:** Fixed eighth-grid assumptions
- **Scenario:** Three-beat split rules inspect off-grid starts using eighth-note spacing regardless of the original beat size.
- **Current Behavior:** Off-grid detection uses the fixed eighth-note tick value to force beat-boundary splits.
- **Why Suspicious:** The logic is deliberate for current `3/4` offbeat handling and should not be generalized blindly, because smaller values such as sixteenths should not necessarily follow the same preservation rule.
- **Preserved In This Spec:** Yes. The behavior remains unchanged.
- **Status:** Intentional rule, not currently treated as a bug.
- **Recommended Follow-up:** Preserve this as a specific `3/4` offbeat readability rule unless future notation work deliberately expands or redefines the supported syncopation patterns.

## F-005

- **ID:** F-005
- **Area:** 3/4 beat-two spanning notes
- **Scenario:** In `3/4`, a half note starts on beat 2 and runs to the bar end.
- **Current Behavior:** The parser currently leaves the beat-two span as a single half note rather than rewriting it as quarter-tied-quarter.
- **Why Suspicious:** This can look surprising at first glance, but it is the intended current rule for a half note that begins exactly on beat 2 in `3/4`.
- **Preserved In This Spec:** Yes. High-level regression tests now characterize the current output explicitly.
- **Status:** Intentional rule, not currently treated as a bug.
- **Recommended Follow-up:** Keep explicit regression coverage for this case so future `3/4` refactors do not accidentally split it.

## F-006

- **ID:** F-006
- **Area:** 3/4 syncopated beat-two patterns
- **Scenario:** In `3/4`, a bar contains quarter note + dotted-quarter + final eighth slot. The subtle rule is:
  - quarter + dotted-quarter + eighth note should stay unsplit at beat 3
  - quarter + dotted-quarter + eighth rest should split at beat 3
- **Current Behavior:** The note-ending version is intentionally preserved unsplit, and the rest-ending counterpart is now explicitly split at beat 3 so the final beat stays visible.
- **Why Suspicious:** These two shapes look very similar in raw duration terms, but they should not be engraved the same way. The presence of a final rest changes the readability requirement in `3/4`.
- **Preserved In This Spec:** Yes. The new scenario suite locks down the current rendering so future changes are visible immediately.
- **Status:** Fixed in current branch.
- **Recommended Follow-up:** Preserve explicit regression coverage for both sides of the rule so future `3/4` refactors keep the note-ending case unsplit and the rest-ending case split at beat 3.

## F-007

- **ID:** F-007
- **Area:** 3/4 vs 6/8 visual ambiguity
- **Scenario:** `3/4` shapes with dotted-quarter centers or paired dotted-quarter gestures can resemble compound-meter grouping even though they must read as simple triple time.
- **Current Behavior:** The parser handles these with specific exception-pattern matches first, then falls back to the general simple-meter rules.
- **Why Suspicious:** The behavior can look inconsistent if viewed as though one universal split rule should apply, but the current design intentionally relies on nuanced pattern-specific exceptions because no single hard-and-fast rule fits all cases.
- **Preserved In This Spec:** Yes. The scenario tests now document both the preserved and rewritten cases.
- **Status:** Intentional design approach, not currently treated as a bug.
- **Recommended Follow-up:** Preserve and extend the explicit pattern coverage so future refactors keep applying the exception patterns before the more general splitting rules.

## F-008

- **ID:** F-008
- **Area:** 4/4 near-miss offbeat intrusions
- **Scenario:** A bar almost matches a preserved offbeat syncopation pattern, but an extra on-beat event intrudes.
- **Current Behavior:** Once the extra on-beat event breaks the offbeat-pattern match, the parser falls back to the general single-track rules and effectively flattens the note events into one rhythmic stream, producing the tied/fragmented written result.
- **Why Suspicious:** The output can look odd if read as though the original offbeat span should still be preserved, but that expectation is no longer valid once the added on-beat event changes the rhythmic pattern.
- **Preserved In This Spec:** Yes. The regression suite now records the exact current output.
- **Status:** Intentional single-track behavior, not currently treated as a bug.
- **Recommended Follow-up:** Keep this characterization as protection for single-track rhythmic flattening. If multi-voice stave support changes notation expectations later, handle that as a separate musical concept rather than changing this single-track rule implicitly.
