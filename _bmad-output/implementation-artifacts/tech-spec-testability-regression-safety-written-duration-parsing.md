---
title: "Testability & Regression Safety Plan for Written Duration Parsing"
slug: "testability-regression-safety-written-duration-parsing"
created: "2026-03-06"
status: "ready-for-dev"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  - "Rust 2021"
  - "Tauri 2.10"
  - "rayon"
  - "rustc-hash (FxHashMap/FxHashSet)"
  - "cargo test"
  - "cargo llvm-cov"
files_to_modify:
  - "src-tauri/src/instructions/parse/get_written_durations.rs"
  - "src-tauri/src/instructions/parse/get_measures.rs"
  - "src-tauri/src/instructions/parse/get_tone_offsets.rs"
  - "src-tauri/src/instructions/parse/get_tone_offset_info.rs"
  - "src-tauri/src/instructions/parse/get_beams.rs"
  - "src-tauri/src/instructions/parse/get_shunts.rs"
  - "src-tauri/src/instructions/parse/get_accidentals.rs"
  - "src-tauri/src/instructions/parse/get_dots.rs"
  - "src-tauri/src/instructions/parse/get_articulations.rs"
  - "src-tauri/src/instructions/parse/get_stem_directions.rs"
  - "_bmad-output/implementation-artifacts/notation-edge-cases-written-durations.md"
code_patterns:
  - "clarity-first helper extraction from large recursive parsing methods"
  - "behavior-preserving refactors only"
  - "Notation and NotationTrack as parse-phase seam types reused across downstream modules"
  - "compact fixture builders for notation scenarios"
  - "direct #[cfg(test)] modules colocated with Rust source files"
  - "scenario-driven regression tests over coverage padding"
  - "Given/When/Then acceptance criteria"
test_patterns:
  - "targeted helper/rule tests"
  - "table-driven split scenario tests"
  - "end-to-end get_written_durations fixtures"
  - "explicit assertions on split points, durations, ties, and rests"
  - "small constructor helpers for Flow, Track, TimeSignature, Tone, Instrument, and Stave graphs"
  - "separate edge-case findings document for future follow-up"
---

# Tech-Spec: Testability & Regression Safety Plan for Written Duration Parsing

**Created:** 2026-03-06

## Overview

### Problem Statement

`get_written_durations` is a critical part of the score pipeline. It converts raw track data into written notation by applying beat-aware splitting rules, rhythm-pattern exceptions, tie propagation, and measure-driven boundaries. Although the file already contains many tests, the highest-risk logic is still hard to reason about and lacks enough meaningful regression coverage around whole musical outcomes. That makes future changes less safe than they should be for such a central notation engine.

### Solution

Refactor the written-duration parsing flow for clarity first and testability second, without changing behavior. Extract only the rule seams that reduce cognitive load, then add meaningful regression tests that cover helper rules, split scenarios, and complete `get_written_durations` outcomes from raw input. Capture any suspicious or ambiguous notation behaviors in a separate document for later follow-up rather than changing behavior in this work.

### Scope

**In Scope:**

- `get_written_durations` and the closely related parsing helpers needed to test it properly
- Clarity-first structural refactors that simplify reasoning about splitting rules and recursive flow
- Scenario-based regression tests for simple-meter splitting, dotted rhythms, offbeat patterns, rests across beat boundaries, and tie behavior
- End-to-end tests that assert full notation outcomes from raw track data
- A separate findings document listing suspicious edge cases or notation conventions discovered during investigation

**Out of Scope:**

- Intentional behavior changes to notation output
- Test-only abstractions that make production parsing code harder to read
- Coverage work driven purely by numeric targets
- Broad parsing refactors outside the written-duration area unless directly required to improve clarity or testability here

## Context for Development

### Codebase Patterns

- Parse-phase Rust code prioritizes performance and uses `FxHashMap` / `FxHashSet` rather than standard hash collections
- The parse pipeline already composes several modules around `Notation` and `NotationTrack`; `get_beams`, `get_shunts`, `get_articulations`, `get_dots`, and draw modules all consume this shape, so refactors must preserve those seams
- Existing tests in `get_written_durations.rs` lean heavily toward method-level and predicate-level coverage; the next layer should emphasize scenario outcomes and regression safety
- The render pipeline calls `get_measures` → `get_written_durations` → downstream parse passes in a fixed order, so meaningful end-to-end tests should mirror that production sequencing where practical
- Time-signature behavior is centralized in `TimeSignature` methods such as `ticks_per_bar()`, `get_tick_at_beat()`, `groupings_to_ticks()`, and `is_on_beat()`, so test fixtures should build on those primitives rather than duplicating tick math in assertions
- Test helpers across the Rust codebase are lightweight local factory functions inside `#[cfg(test)]` modules, not shared abstraction-heavy harnesses
- The preferred direction is simple production code with tests that mirror musical intent, not production code distorted around tests
- Current project guidance favors behavior-preserving refactors, `cargo test --lib`, and `cargo llvm-cov --lib` for validation

### Files to Reference

| File                                                                                       | Purpose                                                                                     |
| ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| `src-tauri/src/instructions/parse/get_written_durations.rs`                                | Core written-duration splitting logic, notation structures, and existing tests              |
| `src-tauri/src/instructions/parse/get_measures.rs`                                         | Measure and time-signature inputs that drive split boundaries                               |
| `src-tauri/src/instructions/parse/get_tone_offsets.rs`                                     | Produces vertical offsets that feed notation ordering and later parse passes                |
| `src-tauri/src/instructions/parse/get_tone_offset_info.rs`                                 | Post-processing applied to notation tracks inside `get_written_durations`                   |
| `src-tauri/src/instructions/parse/get_beams.rs`                                            | Related parse consumer of notation output and source of pattern expectations                |
| `src-tauri/src/instructions/parse/get_shunts.rs`                                           | Related parse consumer and useful reference for notation fixtures                           |
| `src-tauri/src/instructions/parse/get_accidentals.rs`                                      | Downstream parse consumer sensitive to notation grouping, rests, and tie/layout assumptions |
| `src-tauri/src/instructions/parse/get_dots.rs`                                             | Downstream parse consumer sensitive to dotted written durations                             |
| `src-tauri/src/instructions/parse/get_articulations.rs`                                    | Downstream parse consumer keyed off notation events and written grouping                    |
| `src-tauri/src/instructions/parse/get_stem_directions.rs`                                  | Downstream parse consumer that depends on final notation grouping and beam relationships    |
| `src-tauri/src/instructions/mod.rs`                                                        | Confirms production parse ordering and downstream dependencies on written durations         |
| `src-tauri/src/score/entries/time_signature.rs`                                            | Time-signature semantics that define beat math and bar/unit boundaries                      |
| `src-tauri/src/score/tracks.rs`                                                            | Minimal dual-index track structure used to build raw-entry fixtures                         |
| `src-tauri/src/score/entries/tone.rs`                                                      | Tone construction helpers and track entry behavior used by test fixtures                    |
| `src-tauri/src/score/flows.rs`                                                             | Minimal `Flow` / stave graph structure needed for end-to-end fixture setup                  |
| `src-tauri/src/score/staves.rs`                                                            | Stave container shape needed to wire instrument staves to tracks                            |
| `src-tauri/src/score/instruments/mod.rs`                                                   | Instrument structure consumed by `get_written_durations()` and tone-offset calculation      |
| `_bmad-output/implementation-artifacts/tech-spec-comprehensive-test-coverage-expansion.md` | Prior coverage spec that already identified this file as a critical parse hotspot           |

### Technical Decisions

- Preserve current behavior and treat any questionable notation rules as findings, not implementation changes in this spec
- Prefer extracting small pure helpers or clearly named rule evaluators only where they improve readability of `split_simple_unit()` and related flows; avoid introducing layers whose only purpose is test access
- Proceed cautiously with a test-first cadence: characterize current behavior before refactoring, then keep the suite green after each small structural change so refactors do not accidentally change notation logic
- Make characterization a hard gate: no structural refactor starts until baseline scenario outputs and unsupported-boundary behavior are captured in tests
- Use scenario fixtures and table-driven tests to express musical intent compactly
- Validate regressions with explicit expected split points, durations, rests, ties, and event starts; use full notation outcome assertions where that gives stronger confidence
- Include end-to-end tests for `get_written_durations()` so future refactors can be made with confidence across the full puzzle, not just isolated branches
- Keep fixture construction small and local: build only the minimum `Flow` / `Instrument` / `Stave` / `Track` graph needed to run the real parser
- Standardize one canonical assertion shape for scenario tests, such as a local `NotationExpectation` helper struct containing `tick`, `duration`, `is_rest`, `tone_ids`, and `tie_ids`
- Cover related helper behavior that materially affects outcomes (`get_measures`, tone-offset ordering/info, beam and shunt assumptions) when that strengthens regression confidence for written-duration changes
- Extend downstream confidence checks beyond beams and shunts to the notation consumers most likely to expose accidental logic drift: accidentals, dots, articulations, and stem directions
- Use focused tests and incremental refactor slices to guard against accidental logic drift; avoid broad fallback behavior or structural changes that obscure whether output semantics changed
- Preserve performance-sensitive structure unless there is a strong clarity reason to change it; in particular, avoid adding unnecessary whole-flow passes or removing existing parallel boundaries without explicit justification
- Create a dedicated follow-up findings document for suspicious edge cases discovered during investigation
- Initial suspicious areas to document separately: recursive split behavior in 2-beat units, pattern exceptions that can suppress otherwise expected splits, whole-file reliance on fixed eighth-note tick values in some rules, and several `todo!()` branches for non-simple time signatures or rare rest glyphs that define current functional boundaries

## Implementation Plan

### Tasks

- [ ] Task 1: Establish the behavior baseline and target assertions for written-duration parsing
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Audit the current test module and group existing coverage into three buckets: low-level method tests already present, missing rule-level tests, and missing whole-scenario regression tests. Add a durable characterization checkpoint consisting of a baseline scenario matrix plus baseline expected outputs for the current high-risk cases before structural refactors begin.
  - Notes: This is a planning-in-code task, not a behavior change. The aim is to stop adding ad hoc tests and instead build a deliberate regression net around the risky logic before structural changes begin. This checkpoint must exist before Task 2 starts.

- [ ] Task 2: Refactor `split_simple_unit()` and closely related rule paths for readability without changing behavior
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Extract the densest rule clusters in `split_simple_unit()` into small, clearly named helper functions that preserve current semantics, especially around: split-need detection, 3-beat pattern exceptions, off-grid split checks, non-writable duration splitting, rest-boundary splitting, and 4-beat middle-span exceptions.
  - Notes: Keep the helpers in the same file unless a move materially improves clarity. Do not introduce indirection whose only value is test access. Keep all tick math anchored in existing `TimeSignature` APIs where possible. Land this in small steps with tests added or updated before each refactor slice where practical. A refactor slice should cover only one rule cluster or one helper extraction family at a time.

- [ ] Task 3: Add compact fixture builders for meaningful notation scenarios
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Add small test-only constructors for the minimum graph needed to run real parsing: `Flow`, `Instrument`, `Stave`, `Track`, `Tracks`, `Measures`, `Tone`, and `TimeSignature`. Use these builders to express scenarios in musical terms rather than raw map assembly.
  - Notes: Keep builders local to the `#[cfg(test)]` module in `get_written_durations.rs` and treat that module as the owner of the primary parser-fixture vocabulary. Prefer a few focused helpers over a reusable mini-framework. The production code should stay simpler, not more abstract.

- [ ] Task 4: Strengthen direct rule and structure tests around `NotationTrack`
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Add targeted tests for rule helpers and structural behavior that are still under-protected, including: split recursion stop conditions, dotted-rest handling inside units, off-grid event detection, beat-boundary splits for non-writable durations, tie propagation after `split()`, and negative cases where pattern exceptions intentionally suppress splits.
  - Notes: These tests should assert exact event starts and durations, not just booleans, whenever a rule mutates notation structure.

- [ ] Task 5: Add table-driven scenario tests for the core musical patterns in simple meter
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Introduce scenario tables covering representative simple-meter outcomes for 2-, 3-, and 4-beat units. Include cases for dotted rhythms, offbeat groupings, qhq / qm. / m.q pattern preservation, rest splitting across beats, no-split cases, and unsupported non-simple time-signature boundary cases.
  - Notes: Each scenario should describe the raw input events and assert the final written notation sequence using one canonical local assertion format, such as `NotationExpectation { tick, duration, is_rest, tone_ids, tie_ids }`.

- [ ] Task 6: Add end-to-end `get_written_durations()` regression tests from raw score inputs
  - Files:
    - `src-tauri/src/instructions/parse/get_written_durations.rs`
    - `src-tauri/src/instructions/parse/get_measures.rs`
    - `src-tauri/src/instructions/parse/get_tone_offsets.rs`
    - `src-tauri/src/instructions/parse/get_tone_offset_info.rs`
  - Action: Build full parser tests that create realistic `Flow` / `Instrument` / `Track` graphs, run `get_measures()`, `get_tone_offsets()`, and `get_written_durations()`, and assert the resulting notation track contents. Add direct tests for any related helper function whose behavior is required to make the end-to-end expectations trustworthy.
  - Notes: Focus on scenarios that protect the complete puzzle, not isolated helpers for their own sake. If related helper files only need fixture support rather than production changes, keep modifications minimal. Include explicit characterization tests for unsupported `Compound`, `Complex`, and `Open` paths if current behavior is panic/unsupported, so that boundary remains intentional and visible.

- [ ] Task 7: Add downstream confidence checks for notation consumers where written-duration shape matters
  - Files:
    - `src-tauri/src/instructions/parse/get_beams.rs`
    - `src-tauri/src/instructions/parse/get_shunts.rs`
    - `src-tauri/src/instructions/parse/get_accidentals.rs`
    - `src-tauri/src/instructions/parse/get_dots.rs`
    - `src-tauri/src/instructions/parse/get_articulations.rs`
    - `src-tauri/src/instructions/parse/get_stem_directions.rs`
  - Action: Add or extend tests that prove downstream consumers behave correctly with notation structures produced by the new scenario fixtures, especially where written durations influence beam grouping, shunt assignment, accidental handling, dot calculation, articulation grouping, or stem-direction assumptions.
  - Notes: This is not a separate coverage initiative for these files. Only add tests that materially increase confidence that written-duration refactors have not broken adjacent parse behavior.

- [ ] Task 8: Capture suspicious notation behaviors and explicit current boundaries in a separate findings document
  - File: `_bmad-output/implementation-artifacts/notation-edge-cases-written-durations.md`
  - Action: Create a concise follow-up document listing edge cases, ambiguous conventions, and suspicious current behaviors found during implementation or test writing. Use a fixed schema with: `ID`, `Area`, `Scenario`, `Current Behavior`, `Why Suspicious`, `Preserved In This Spec`, and `Recommended Follow-up`.
  - Notes: Candidate findings already identified include recursive 2-beat handling, pattern exceptions that suppress expected splits, fixed eighth-note assumptions in some rule branches, and unsupported `Compound` / `Complex` / `Open` time signatures.

- [ ] Task 9: Validate confidence gains with focused test execution and coverage review
  - Files:
    - `src-tauri/src/instructions/parse/get_written_durations.rs`
    - `_bmad-output/implementation-artifacts/notation-edge-cases-written-durations.md`
    - `TESTING.md` (only if documentation needs a note about how to run or interpret the new parser-focused tests)
  - Action: Run `cargo test --lib` frequently during implementation, then run `cargo llvm-cov --lib` at three checkpoints: after baseline characterization lands, after scenario/end-to-end coverage lands, and at the end of the work. Document the before/after confidence story in terms of scenario coverage rather than just percentages. Update test documentation only if the new test layers introduce useful project-level guidance.
  - Notes: Do not ratchet thresholds solely for this work. If coverage improves materially, record the numbers in the findings or final implementation notes, but keep the focus on regression protection.

### Acceptance Criteria

- [ ] AC 1: Given the current written-duration behavior, when the clarity refactor is applied to `split_simple_unit()` and its supporting rule paths, then the output notation for existing covered scenarios remains unchanged.
- [ ] AC 1a: Given the parser before any refactor begins, when baseline characterization tests are added, then they capture known-good outputs for the current high-risk written-duration scenarios and become the parity gate for later refactors.
- [ ] AC 2: Given a simple-meter unit that does not require splitting, when the parser evaluates it, then no extra notation events are introduced and the original writable event boundaries are preserved.
- [ ] AC 3: Given a simple-meter unit containing non-writable durations or dotted rests in disallowed positions, when written durations are generated, then the resulting notation is split at the same beat boundaries required by current behavior.
- [ ] AC 4: Given recognized preservation patterns such as qhq, qm., m.q, and supported offbeat patterns, when written durations are generated, then the parser preserves those spans and does not apply the normal middle split for the unit.
- [ ] AC 5: Given a notation event that is split into multiple written durations, when the split occurs, then the resulting notation entries preserve tones and apply ties exactly as the current implementation does.
- [ ] AC 6: Given representative 2-beat, 3-beat, and 4-beat raw input scenarios, when table-driven scenario tests run, then each scenario asserts the ordered written notation result with explicit ticks, durations, rest state, and tie state.
- [ ] AC 7: Given realistic raw score inputs containing flow, stave, track, instrument, and tone data, when `get_measures()`, `get_tone_offsets()`, and `get_written_durations()` are run together, then the resulting notation tracks match expected end-to-end written outcomes.
- [ ] AC 7a: Given unsupported `Compound`, `Complex`, or `Open` time signatures, when written-duration parsing is invoked against them, then the current unsupported behavior is made explicit in tests and documentation rather than remaining implicit.
- [ ] AC 8: Given downstream parse consumers that rely on notation shape, when written-duration fixtures are fed into beam, shunt, accidental, dot, articulation, and stem-direction tests, then those consumers continue to behave consistently with the produced notation structure.
- [ ] AC 9: Given ambiguous or suspicious notation rules discovered during this work, when the implementation is complete, then they are documented in a separate findings document and not silently changed as part of this spec.
- [ ] AC 10: Given the completed work, when `cargo test --lib` and `cargo llvm-cov --lib` are run, then the parser-focused tests pass and provide materially stronger regression coverage for the written-duration pipeline.
- [ ] AC 10a: Given the completed refactor, when the parse pipeline is reviewed, then existing parallel boundaries and performance-sensitive traversal patterns are preserved unless a deliberate, documented exception was required for clarity.

## Additional Context

### Dependencies

- Existing score model fixtures for `Flow`, `Track`, `Tracks`, `Instrument`, `Measures`, and `Tone`
- Rust unit-test support in `#[cfg(test)]` modules
- `TimeSignature` helper methods for bar and beat math (`ticks_per_bar()`, `ticks_per_beat()`, `get_tick_at_beat()`, `groupings_to_ticks()`)
- Existing parse ordering in `generate_render_instructions()` so end-to-end tests mirror real invocation order
- Existing `rayon`-based parallel boundaries in parse-phase code, which should be preserved unless intentionally documented otherwise
- `cargo test --lib`
- `cargo llvm-cov --lib`

### Testing Strategy

Build confidence in layers:

1. **Rule layer** — test extracted helpers and `NotationTrack` structure changes directly where they represent real musical rules.
2. **Scenario layer** — use table-driven cases for representative simple-meter patterns and anti-patterns, asserting exact notation outcomes.
3. **End-to-end parser layer** — build minimal but real score graphs and run the production parsing flow through `get_measures()`, `get_tone_offsets()`, and `get_written_durations()`.
4. **Downstream sanity layer** — verify a small number of beam and shunt behaviors that depend on the produced notation shape.
5. **Boundary layer** — explicitly characterize unsupported time-signature behavior and any other intentional current limitations.

Execution style should be deliberately defensive:

- Prefer test-first or characterization-first changes for high-risk branches.
- Add the next failing test before refactoring a rule cluster whenever feasible.
- Keep each refactor slice small enough that a regression can be traced to one rule extraction or one fixture addition.
- Prefer clarity-preserving extractions over rewrites so accidental logic changes are easier to detect.
- If behavior is uncertain, lock it down with a test and document it before changing structure.

Use one canonical scenario assertion helper throughout the file so regression intent stays readable and comparable across tests.

Manual review should include scanning the scenario names and expected outputs to ensure they read like musical intent, not merely implementation detail. Favor meaningful, readable expectations over raw coverage padding.

### Notes

- Highest-risk area: `split_simple_unit()` currently mixes recursion, beat math, pattern exceptions, and split side effects in one place; readability gains here are the main maintainability win.
- Recommended working rhythm: baseline tests first, then one small refactor or rule extraction at a time, with `cargo test --lib` run frequently during the sequence to catch accidental notation changes immediately.
- Maximum safe refactor slice: one helper family or one rule cluster at a time, followed by green tests before moving on.
- The current functional boundary explicitly excludes implemented support for `Compound`, `Complex`, and `Open` time signatures in `get_written_durations()`; this work should preserve that boundary and document it.
- Avoid turning the parser into a dependency-injected system just to make tests easier. Local helpers and expressive fixtures are the preferred route.
- If a scenario reveals behavior that appears musically wrong but is currently relied on by the code, preserve it in implementation, cover it with a test if it is intentional, and record it in the separate findings document if it is suspicious.
- Primary fixture ownership should stay in `get_written_durations.rs`; related files should only receive thin support tests, not duplicate fixture frameworks.
- Future follow-up work can use the separate findings document to drive a behavior-change spec once the current parser behavior is fully characterized.
