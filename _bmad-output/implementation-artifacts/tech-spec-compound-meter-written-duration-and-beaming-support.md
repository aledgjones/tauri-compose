---
title: 'Compound Meter Written-Duration And Beaming Support'
slug: 'compound-meter-written-duration-and-beaming-support'
created: '2026-03-09'
status: 'Implementation Complete'
stepsCompleted: [1, 2, 3, 4, 5, 6]
tech_stack:
	- 'Rust 2021 in the Tauri score-render pipeline'
	- 'rayon parallel iterators for per-track analysis'
	- 'rustc-hash FxHashMap/FxHashSet collection patterns'
	- 'inline Rust #[test] modules with parser and renderer helper fixtures'
files_to_modify:
	- 'src-tauri/src/instructions/parse/get_written_durations.rs'
	- 'src-tauri/src/instructions/parse/get_beams.rs'
	- 'src-tauri/src/instructions/draw/draw_rest.rs'
	- 'docs/development-guide.md'
code_patterns:
	- 'Measure-local parse pipeline stages feed downstream analysis and drawing in fixed order from get_written_durations into beams, dots, accidentals, spacing, stems, and draw consumers'
	- 'Time-signature boundary logic is expressed in ticks via TimeSignature helpers such as default groupings and groupings_to_ticks rather than staff-level rendering code'
	- 'Notation rewriting is encoded as NotationTrack split helpers that preserve tones, ties, rests, and chord membership while recursively splitting at musical boundaries'
	- 'Per-track work is parallelized and then reassembled into FxHashMap outputs keyed by track id'
test_patterns:
	- 'get_written_durations.rs uses named scenario tables and expectation helpers to assert exact notation fragments, ties, rests, and chord preservation'
	- 'Unsupported meter behavior is currently characterized with #[should_panic] tests in get_written_durations.rs'
	- 'get_beams.rs uses focused inline unit tests for grouping, beam counts, and time-signature-specific beam behavior including 6/8'
	- 'Draw modules such as draw_rest.rs and draw_ties.rs use narrow renderer unit tests for glyph selection and geometry side effects'
---

# Tech-Spec: Compound Meter Written-Duration And Beaming Support

**Created:** 2026-03-09

## Overview

### Problem Statement

The parse pipeline recognizes compound time signatures, but written-duration generation still panics for compound meters instead of producing musically correct notation. That leaves compound-meter scores unsupported and also blocks dependent systems such as beaming, spacing, ties, and downstream notation consumers from handling compound rhythmic structures correctly.

### Solution

Define a Gould-driven rule set for compound-meter rhythmic spelling using the repo's existing default grouping model, implement compound written-duration splitting in the parse phase, and extend dependent notation systems so compound rhythms produce correct written durations, ties, rests, beams, and downstream layout behavior.

### Scope

**In Scope:**

- Compound-meter written-duration splitting for eighth-based compound meters where the numerator is a multiple of three greater than three
- Mandatory v1 regression coverage for `6/8`, `9/8`, `12/8`, plus one higher-count proof case such as `15/8`
- Default grouping behavior only, using the existing time-signature grouping model
- Beaming behavior and any downstream parse or draw consumers that depend on written-duration output
- Notes, rests, ties, chords, barline crossings, and ambiguous rhythmic cases that should prefer the visible big beat
- Validation and normalization of the user's idiomatic examples against Elaine Gould's rules before they become regression cases
- Regression tests, fixture strategy if needed, and documentation updates for notation behavior and developer expectations

**Out of Scope:**

- Complex meter support
- Open meter support
- User-authored custom beat groupings beyond the current default-grouping model
- Unrelated rendering changes that do not materially depend on compound written-duration output

## Context for Development

### Codebase Patterns

- Written durations are generated in the Rust parse pipeline and feed multiple downstream parse and draw stages.
- `get_written_durations()` currently handles simple meters and panics for compound, complex, and open time signatures.
- `TimeSignature::kind()` and default groupings already classify compound signatures and expose grouping boundaries that downstream systems use.
- `get_written_durations()` performs measure-local splitting first at event boundaries, then applies meter-specific rewrite helpers on `NotationTrack`; the existing simple-meter helpers are the primary extension pattern for this feature.
- Beaming already depends on `NotationTrack` output and grouping boundaries, but current logic is still biased toward simple-meter and eighth-pattern assumptions even though it already carries a basic `6/8` regression.
- The repo uses high-level scenario tests in `get_written_durations.rs` to characterize notation rules and preserve musically motivated exceptions, with helper fixtures and exact fragment expectations instead of snapshot files.
- Downstream parse and draw consumers largely trust `Notation` semantics rather than recomputing rhythm structure, so incorrect compound splitting will cascade into ties, dots, spacing, stems, accidentals, and rest drawing.
- Collection and pipeline style is consistent across the parser: parallel per-track passes, deterministic `FxHashMap` assembly, and minimal cross-stage mutation.
- Gould is the source of truth for notation decisions; where cases are ambiguous in compound notation, the implementation should prefer showing the big beat.
- The implementation must cover not just split logic but notation consequences for ties, rests, chords, beams, and any other consumer that feeds in or out of notation splitting.
- Resolution support must be validated end-to-end because the current renderer still has unresolved rest-glyph gaps for some smaller durations.

### Compound Rule Baseline

These rule statements are the implementation baseline for this spec. They are a distilled project interpretation of the Gould-backed compound-meter behavior discussed during discovery and are intended to make the spec executable without requiring the developer to reconstruct the rules from conversation context.

- In supported compound meters, the visible big beat is a dotted value equal to three written eighths.
- If a note or chord exactly occupies one big beat and does not hide a more important arrival, prefer one dotted value rather than rewriting it into smaller fragments.
- If a note, chord, or rest crosses a big-beat boundary and its unsplit spelling would hide that boundary, split at the crossed big-beat boundary and preserve continuity with ties for sounding material.
- Rests must not conceal big-beat boundaries or barlines; split rests at every crossed big-beat boundary and measure boundary.
- When multiple spellings are technically writable, prefer the spelling that keeps the big beat visually obvious.
- Beam groups must reinforce the same big-beat boundaries shown by the written durations; in straight compound eighth-note motion that means groups of three eighths per big beat.
- Chord tone membership and tie sets must remain identical before and after any rewrite.
- The implementation may preserve an unsplit span only when the resulting notation still makes the compound beat structure clearer than the alternative split spelling.

### Supported Meter Set

- V1 support is explicitly targeted at eighth-based compound signatures: `6/8`, `9/8`, `12/8`, and the same default-grouping rule for larger `x/8` compound signatures where `x` is a multiple of three greater than three.
- The minimum proof corpus must include `6/8`, `9/8`, `12/8`, and one higher-count example such as `15/8`.
- Non-eighth compound classifications that may currently be labeled `Compound` by code are not part of this delivery unless implementation confirms they obey the same rule set without introducing new notation decisions; otherwise they should remain explicitly unsupported and documented.

### Files to Reference

| File                                                                             | Purpose                                                                                          |
| -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `src-tauri/src/instructions/parse/get_written_durations.rs`                      | Current written-duration parser, simple-meter rules, panic boundary, and regression-test harness |
| `src-tauri/src/instructions/parse/get_beams.rs`                                  | Current beaming logic that consumes `NotationTrack` and grouping boundaries                      |
| `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`                     | Horizontal spacing consumer that sizes notation output after splitting and beam assignment       |
| `src-tauri/src/instructions/parse/get_dots.rs`                                   | Dot placement consumer that trusts dotted `Notation` output from duration splitting              |
| `src-tauri/src/instructions/mod.rs`                                              | Parse pipeline order showing downstream dependencies of written durations                        |
| `src-tauri/src/score/entries/time_signature.rs`                                  | Time-signature kind detection, default groupings, and beat/group boundary helpers                |
| `src-tauri/src/instructions/draw/draw_ties.rs`                                   | Example downstream draw consumer of tie information from notation splitting                      |
| `src-tauri/src/instructions/draw/draw_rest.rs`                                   | Rest rendering path that may need support for newly reachable compound outputs                   |
| `docs/development-guide.md`                                                      | Candidate developer-facing doc target for notation-rule and test-practice updates                |
| `docs/project-context.md`                                                        | Repo rules and pipeline conventions                                                              |
| `_bmad-output/implementation-artifacts/notation-edge-cases-written-durations.md` | Existing findings and notation-rule context for written-duration behavior                        |

### Technical Decisions

- Treat Elaine Gould as the notation authority for compound-meter rhythmic spelling.
- Use the rule baseline in this spec as the project-ready operational summary of that authority; if a test example conflicts with the baseline after validation, update the baseline and the example together before implementation proceeds.
- Use existing default time-signature groupings for v1 rather than introducing authored custom groupings.
- Interpret compound meters according to general eighth-based compound grouping logic; when a case is ambiguous, preserve the visible big beat.
- Normalize and validate the user's idiomatic examples before they are accepted as normative regression scenarios.
- Treat written-duration splitting and beaming as one connected notation feature for this work because the correctness boundary crosses both modules.
- Evaluate compound support against the explicitly supported eighth-based set in this spec, with `6/8`, `9/8`, `12/8`, and one higher-count proof case required before the work is considered complete.
- Prefer extending existing tick-based grouping helpers and `NotationTrack` rewrite patterns over introducing meter-specific special cases scattered across draw code.
- Keep the parse pipeline order intact; compound support should be expressed by improving stage behavior, not by reordering downstream consumers.
- Treat newly reachable sub-thirty-second rests as part of the implementation boundary, because `draw_rest.rs` still lacks glyph and hit-box handling for `SixtyFourth` and `HudredTwentyEighth` outputs.
- Migration assessment: this feature is expected to change only derived notation and render-time analysis, not persisted score structure or `.soloc` schema. If implementation requires persisted model changes, serialized grouping changes, or altered load/save semantics, work must stop and add the required pending migration before release prep.

## Implementation Plan

### Tasks

- [x] Task 1: Define the compound-meter splitting rule model on top of the existing `NotationTrack` rewrite flow.
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Add compound-specific unit splitting helpers that operate on default compound groupings, derive dotted-big-beat boundaries, and preserve existing event-first splitting behavior.
  - Notes: Reuse `TimeSignature::groupings_to_ticks()` and existing split recursion patterns instead of introducing renderer-side rhythm logic. The helper API must expose enough structure to distinguish “exactly one big beat”, “crosses one big-beat boundary”, and “crosses a barline”.

- [x] Task 2: Implement compound written-duration rewriting in `get_written_durations()` and remove the current panic path for supported compound meters.
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Replace `TimeSignatureType::Compound => todo!()` with compound dispatch that rewrites each supported measure into notation fragments matching the rule baseline in this spec, including notes, rests, ties, and barline crossings.
  - Notes: Preserve the current unsupported behavior for `Complex` and `Open`; ambiguity should favor the visible big beat. Unsupported non-eighth compound classifications should remain explicit and documented rather than silently inheriting unvalidated behavior.

- [x] Task 3: Extend the regression corpus to codify normalized Gould-aligned compound scenarios.
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Convert the current compound panic tests into positive scenario tests and add the minimum regression corpus defined in this spec.
  - Notes: Normalization means converting any shorthand or ambiguous user example into explicit tick/duration inputs plus one canonical expected notation output. If a user shorthand conflicts with the rule baseline, the canonical test must follow the baseline and note that the original shorthand was adjusted.

- [x] Task 4: Align beaming behavior with the new compound notation output.
  - File: `src-tauri/src/instructions/parse/get_beams.rs`
  - Action: Update compound grouping and break logic so beam formation follows default compound beat groups and does not visually contradict the new written-duration spelling.
  - Notes: Preserve existing simple-meter behavior and add explicit beam-break tests for straight `6/8`, `9/8`, and `12/8` eighth-note runs, plus at least one syncopated case that breaks at a big-beat boundary because the duration spelling does.

- [x] Task 5: Close downstream rendering gaps exposed by newly reachable compound outputs.
  - File: `src-tauri/src/instructions/draw/draw_rest.rs`
  - Action: Add rest rendering support and hit-target sizing for any smaller writable rest durations that become reachable through compound splitting.
  - Notes: This task is mandatory if any canonical compound regression produces a rest whose base duration is `SixtyFourth` or `HudredTwentyEighth`. If the full canonical corpus proves that no supported compound case emits those rest values, add a regression asserting that fact and remove draw changes from the final implementation diff.

- [x] Task 6: Document the notation rules and regression expectations for future work.
  - File: `docs/development-guide.md`
  - Action: Add developer guidance describing compound-meter written-duration behavior, Gould as the notation authority, the default-grouping assumption, the supported meter set, and the expected parser test style for rhythm-rule regressions.
  - Notes: A rule is “durable” for `.github/copilot-instructions.md` purposes if future notation features would be required to follow it even outside this feature, such as a new mandatory documentation or regression standard.

### Minimum Regression Corpus

- `6/8`: six straight eighth notes beam and spell as `3 + 3`.
- `6/8`: dotted-quarter plus dotted-quarter remains two big beats.
- `6/8`: a note starting before beat two and ending after it is rewritten to show the crossed big-beat boundary with a tie.
- `6/8`: a rest spanning the beat-two boundary is split into two rest fragments.
- `6/8`: a chord spanning the beat-two boundary preserves both tones on both tied fragments.
- `9/8`: nine straight eighth notes beam and spell as `3 + 3 + 3`.
- `9/8`: syncopation crossing one internal big-beat boundary is split to reveal that boundary.
- `9/8`: a rest-led syncopation keeps the beat grouping visible.
- `12/8`: twelve straight eighth notes beam and spell as `3 + 3 + 3 + 3`.
- `12/8`: a barline-crossing note is split with ties at the barline and any required internal big-beat boundaries.
- `12/8`: a barline-crossing rest remains split at the barline and any crossed big-beat boundaries.
- Higher-count proof case: at least one `15/8` or equivalent `x/8` compound case confirms the same default-grouping algorithm scales beyond `12/8`.

### Acceptance Criteria

- [x] AC 1: Given a score in a time signature classified as `TimeSignatureType::Compound`, when `get_written_durations()` runs, then it returns notation output without panicking and preserves the original musical durations through writable fragments and ties.
- [x] AC 2: Given a supported compound meter and an event whose duration is exactly one big beat, when written durations are generated, then the event is represented by one dotted value rather than by smaller tied fragments unless a barline forces a split.
- [x] AC 3: Given a supported compound meter and an event that starts before one big-beat boundary and ends after it, when the unsplit spelling would hide that crossed boundary, then the output is split at that boundary and preserves continuity with ties for sounding material.
- [x] AC 4: Given compound-meter rests that cross group or measure boundaries, when written durations are generated, then the rests are split at the required boundaries and remain drawable by downstream consumers.
- [x] AC 5: Given compound-meter notes or chords that cross group or measure boundaries, when written durations are generated, then ties and chord tone sets are preserved exactly across all resulting fragments.
- [x] AC 6: Given straight eighth-note runs in `6/8`, `9/8`, and `12/8`, when `get_beams()` runs, then beams break only at big-beat boundaries, producing `3 + 3`, `3 + 3 + 3`, and `3 + 3 + 3 + 3` groupings respectively.
- [x] AC 7: Given existing simple-meter regression scenarios, when the new compound support ships, then the current simple-meter notation and beaming tests continue to pass unchanged.
- [x] AC 8: Given the minimum regression corpus in this spec and any additional normalized user examples, when the parser test suite runs, then each case passes as an explicit scenario test in `get_written_durations.rs` with canonical expectations derived from the rule baseline.
- [x] AC 9: Given compound rhythms that produce smaller writable rests, when rendering executes, then rest glyph selection and hit targets remain valid for the emitted durations.
- [x] AC 10: Given the completed feature, when a developer reads the project guidance, then the compound-meter notation rules, default-grouping assumption, and regression-test expectations are documented in the developer docs.
- [x] AC 11: Given the supported compound-meter set in this spec, when implementation completes, then `6/8`, `9/8`, `12/8`, and one higher-count `x/8` proof case are covered by tests and any unsupported non-eighth compound classifications remain explicitly documented.
- [x] AC 12: Given the project migration rules, when this feature is implemented without persisted-data or file-format changes, then the spec and implementation both explicitly record that no migration was required; if persisted behavior changes, then a pending migration is added before release work.

### Documentation Impact

- Update `docs/development-guide.md` with the compound-meter notation rules, the Gould authority decision, and the expectation that rhythmic regressions be captured as explicit parser scenarios rather than ad hoc fixes.
- Review `.github/copilot-instructions.md` during implementation; update it if this work establishes a project-wide rule that future notation features must also follow, such as mandatory scenario-based rhythm regressions or required notation-rule documentation sections in specs.
- No architecture document update is required unless implementation reveals a broader parser-pipeline contract change beyond compound-meter support.

## Additional Context

### Dependencies

- Compound written-duration rules depend on `TimeSignature` grouping helpers, current `NotationTrack` split/tie behavior, and downstream beaming/layout/rendering consumers.
- Final scenario coverage depends on validating normalized examples against Gould.
- No new external libraries are expected; the implementation should stay within existing Rust/Tauri parser infrastructure.
- The feature depends on the current parse pipeline ordering remaining stable from written durations into beams, dots, accidentals, spacing, stems, and draw consumers.
- The supported meter family for this spec depends on eighth-based compound grouping; non-eighth compound classifications are intentionally excluded unless explicitly validated.

### Testing Strategy

- Add parser scenario tests for every item in the minimum regression corpus in this spec before treating the feature as complete.
- Add parser scenario tests in `get_written_durations.rs` for normalized `6/8`, `9/8`, `12/8`, and one higher-count `x/8` case covering straight grouping, syncopation, rests, ties, chords, and barline crossings.
- Replace current compound `#[should_panic]` expectations with positive assertions for supported compound signatures while keeping `Complex` and `Open` panic characterization until those meters are implemented.
- Extend `get_beams.rs` tests so compound beam groupings are checked against the new written-duration behavior, including explicit break expectations for straight `6/8`, `9/8`, and `12/8` runs and at least one syncopated split case.
- Add or update narrow draw tests in `draw_rest.rs` only if smaller rest durations become reachable through the new split rules.
- Run the relevant Rust test targets for parser, beaming, and draw modules after implementation; manual score inspection should confirm that written durations, ties, beams, dots, and spacing remain visually coherent in representative compound examples.
- Manual review checklist: confirm big-beat visibility, beam breaks, rest splitting, tie continuity, chord preservation, and unchanged simple-meter output.

### Notes

- The user's supplied examples are starting material, not yet canonical expected output.
- Normalization process: convert shorthand into explicit tone/rest events, validate the case against the rule baseline above, encode the canonical expected notation output, and record any material correction from the shorthand if the shorthand conflicted with the baseline.
- Some extreme-resolution outputs may expose existing downstream gaps, such as unsupported rest glyphs, which should be treated as part of the feature boundary if compound rules can produce them.
- The highest-risk failure mode is implementing local split rules that pass parser tests but create contradictory beam groups or unreadable compound notation when rendered; parser and beaming work should therefore be validated together.
- Another risk is overfitting to `6/8` and accidentally missing general compound behavior for larger dotted-beat group counts, so scenario coverage must include at least `9/8` and `12/8`.
- Custom compound groupings remain out of scope for this version; if they become a requirement later, the work should extend the same tick-boundary architecture rather than replacing it.
- No migration is expected because this feature changes derived notation, not persisted score schema. If implementation disproves that assumption, the migration requirement in project instructions takes priority over this spec.
