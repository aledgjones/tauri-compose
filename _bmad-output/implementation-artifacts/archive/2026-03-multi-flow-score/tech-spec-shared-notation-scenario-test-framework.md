---
title: 'Shared Notation Scenario Test Framework'
slug: 'shared-notation-scenario-test-framework'
created: '2026-03-10'
status: 'completed'
stepsCompleted: [1, 2, 3, 4, 5]
tech_stack:
	- 'Rust 2021 test modules inside src-tauri parser code'
	- 'Tauri score parser fixtures built from Flow, Track, Stave, Instrument, and TimeSignature'
	- 'rustc-hash FxHashMap and FxHashSet for lightweight parser test data construction'
files_to_modify:
	- 'src-tauri/src/instructions/parse/mod.rs'
	- 'src-tauri/src/instructions/parse/get_written_durations.rs'
	- 'src-tauri/src/instructions/parse/test_support/mod.rs'
	- 'src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs'
	- 'docs/development-guide.md'
	- '.github/copilot-instructions.md'
code_patterns:
	- 'Keep shared test helpers parser-local and test-only rather than introducing a repo-wide framework layer'
	- 'Preserve low-level direct unit tests in get_written_durations.rs; only behavior-level scenario support moves'
	- 'Prefer narrow helper extraction: expectation normalization, parser fixture construction, and scenario execution helpers'
	- 'Migrate scenario clusters incrementally, starting with simple-meter written-duration groups'
test_patterns:
	- 'Scenario-table regressions with canonical NotationExpectation fragments'
	- 'Focused parser tests in get_written_durations.rs remain the primary validation surface'
	- 'Run targeted Rust tests for written-duration scenario groups, then the broader Rust suite as final validation'
---

# Tech-Spec: Shared Notation Scenario Test Framework

**Created:** 2026-03-10

## Overview

### Problem Statement

`get_written_durations.rs` has an effective scenario-driven regression style for notation behavior, but the harness that supports it is still embedded directly inside that file's test module. That makes future bug-regression additions slower and less consistent than they should be, especially when adding new simple-meter coverage around specific rhythmic patterns.

### Solution

Extract a small shared test-only notation scenario framework for parser behavior tests, then reuse it first in `get_written_durations.rs` for simple-meter written-duration scenarios. Keep the framework narrow, parser-local, and behavior-focused so future notation regressions are easier to add without over-engineering the test layer. The shared module owns the harness only; the actual musical scenario tables remain in `get_written_durations.rs` so the behavior examples stay next to the parser logic they document.

### Scope

**In Scope:**

- Extracting notation scenario support from `get_written_durations.rs` into a parser-local shared test-only module
- Reusing that support in `get_written_durations.rs` only
- Standardizing shared `NotationExpectation`, normalization, parser fixture, and scenario execution helpers for notation behavior tests
- Migrating simple-meter written-duration scenario groups cautiously, one concern cluster at a time
- Updating developer guidance for future notation regression tests

**Out of Scope:**

- Beam test migration
- Generic test frameworks for unrelated parser modules
- Moving the actual scenario tables out of `get_written_durations.rs`
- Replacing low-level direct unit tests for helpers such as glyphs, split mechanics, and duration predicates
- Production behavior changes to written-duration parsing
- Broad repo-wide test abstraction

## Context for Development

### Codebase Patterns

- `get_written_durations.rs` already contains a proven scenario-table pattern for notation behavior regressions.
- Low-level helper tests in the same file are clearer as direct unit tests and should remain local.
- The preferred implementation style for this work is cautious and incremental rather than sweeping refactor.
- Shared test support should stay parser-local so future support modules can follow the same principle without becoming a generic framework layer.
- There is no existing `test_support` module pattern under `src-tauri/src/instructions/parse/`, so this spec introduces the smallest viable local structure.
- The current reusable harness is concentrated in one helper block: `NotationExpectation`, normalization helpers, parser fixture builders, and `assert_written_duration_scenario*` helpers all live together in `get_written_durations.rs`.
- Repeated local `Scenario` structs appear across multiple simple-meter and compound-meter clusters, which confirms the scenario style is established but still duplicated at the call-site level.
- Those repeated scenario tables are intentional behavior documentation and should remain in the consumer test file; the duplication this spec targets is harness duplication, not the musical examples themselves.

### Files to Reference

| File                                                        | Purpose                                                                                         |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `src-tauri/src/instructions/parse/get_written_durations.rs` | Current home of notation scenario helpers, parser fixtures, and written-duration behavior tests |
| `src-tauri/src/instructions/parse/mod.rs`                   | Parse module entry point where test-only support can be exposed locally                         |
| `docs/development-guide.md`                                 | Existing guidance for rhythm-rule regression expectations                                       |

### Files to Create

| File                                                                                  | Purpose                                                                                                                        |
| ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `src-tauri/src/instructions/parse/test_support/mod.rs`                                | New parser-local test-support module boundary gated to tests                                                                   |
| `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs` | New shared test-only home for notation expectations, normalization, fixture builders, and written-duration scenario assertions |

### Technical Decisions

- Use a parser-local `test_support` structure so future behavior-level helpers can be added as isolated modules following the same principle.
- Limit the first implementation strictly to `get_written_durations.rs`.
- Keep the migration step-by-step: extract helpers first, then migrate scenario groups cautiously.
- Preserve the distinction between behavior-level scenario tests and low-level direct unit tests.
- Keep the initial shared surface narrow: a notation expectation type, expectation constructor, normalization/assertion helpers, parser fixture builders, and scenario execution helpers with and without explicit flow length.
- Do not attempt to centralize every local `Scenario` struct in the first pass; the first value is removing harness duplication, not inventing a generic data DSL.
- Start migration with the simple-meter scenario groups already clustered in `get_written_durations.rs`, especially the `4/4` and `3/4` behavior tables, before deciding whether the same extraction should absorb the compound corpus.
- Treat the scenario tables themselves as local executable specification: they should stay in `get_written_durations.rs` even after the harness moves into shared parser-local test support.
- Wire the shared support through `#[cfg(test)] pub(crate) mod test_support;` in `src-tauri/src/instructions/parse/mod.rs` so the module is visible to sibling parser unit-test modules but absent from production builds.
- Keep the helper API and filename written-duration-oriented: `get_written_durations_test_harness.rs` should expose only the harness pieces already used by `get_written_durations.rs`, not a generic cross-parser scenario DSL.
- Limit first-pass consumers to `get_written_durations.rs`; if another parser test wants to reuse the support later, that should be a separate follow-up decision rather than implicit scope creep in this work.

## Implementation Plan

### Tasks

- [x] Add a parser-local `test_support` module under `src-tauri/src/instructions/parse/` and expose it in a test-only way from `parse/mod.rs`.
- [x] Create `src-tauri/src/instructions/parse/test_support/mod.rs` and `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`, then move the current notation scenario harness out of `get_written_durations.rs` without changing production code paths.
- [x] In `src-tauri/src/instructions/parse/mod.rs`, add `#[cfg(test)] pub(crate) mod test_support;` so sibling parser test modules can import the shared helpers without exposing any production runtime API.
- [x] Update `get_written_durations.rs` tests to import the shared helpers while keeping both the scenario tables and the low-level non-scenario tests local.
- [x] Migrate the existing `4/4` and `3/4` simple-meter scenario clusters to the shared helper surface with no additional regrouping beyond what is required to adopt the new harness.
- [x] Leave compound-meter and other specialized regressions in place unless the extracted harness can be used there with zero extra abstraction work and no scenario-table restructuring.
- [x] Update `docs/development-guide.md` so future notation regressions use the shared parser-local support instead of recreating helper blocks.
- [x] Update `.github/copilot-instructions.md` so future agents are explicitly guided to keep the shared harness in parser-local test support while leaving the musical scenario tables in `get_written_durations.rs`.

### Acceptance Criteria

1. Given the current written-duration scenario coverage, when the shared notation test-support module is introduced, then `get_written_durations.rs` no longer defines local copies of `NotationExpectation`, notation normalization helpers, parser fixture builders, or scenario assertion helpers.
2. Given the new parser-local shared module, when tests use it, then it provides the shared helpers without exposing any production runtime API.
3. Given the simple-meter written-duration tests, when the refactor is complete, then at least one `4/4` scenario cluster and one `3/4` scenario cluster in `get_written_durations.rs` use the shared helper module and retain the same behavioral expectations without moving the scenario tables out of that file.
4. Given low-level direct tests in `get_written_durations.rs`, when the framework is introduced, then those tests remain local and readable rather than being forced through the scenario framework.
5. Given the module wiring in `src-tauri/src/instructions/parse/mod.rs`, when production code is compiled, then the shared test-support module is absent from non-test builds and available only to parser unit-test code.
6. Given the completed spec and documentation, when a developer reads the guidance, then the distinction between shared harness utilities and local behavior-case tables is explicit in both `docs/development-guide.md` and `.github/copilot-instructions.md`.
7. Given the completed extraction, when focused validation runs with `cargo test get_written_durations` from `src-tauri/`, then the migrated written-duration scenario tests remain green.
8. Given the completed extraction, when the full Rust suite runs via `npm run test:rust`, then no written-duration behavior regressions are introduced.

### Documentation Impact

- Update `docs/development-guide.md` so future notation regression work points contributors at the shared parser-local support module.
- Clarify that the shared framework is for harness utilities only, while behavior-case tables stay in the consumer test file and low-level helper tests remain local.
- Update `.github/copilot-instructions.md` so future agents are told to keep shared written-duration harness code in parser-local test support while leaving scenario tables in `get_written_durations.rs`.

## Additional Context

### Dependencies

- Existing parser test fixture construction in `get_written_durations.rs` depends on score-domain types such as `Flow`, `Track`, `Tracks`, `Instrument`, `Stave`, `Staves`, `TimeSignature`, and `Entry` variants.
- The shared support module will rely on the same parser functions already used by the current test harness: `get_measures`, `get_tone_offsets`, and `get_written_durations` call orchestration in the consumer tests.
- No new third-party dependencies are needed; this is a local Rust test refactor.

### Testing Strategy

- Run focused Rust tests for `get_written_durations` from `src-tauri/` first, using `cargo test get_written_durations`, to catch extraction mistakes quickly.
- Re-run the broader written-duration regression corpus after the helper move, including compound-meter scenarios that still consume the shared helper path.
- Finish with `npm run test:rust` to confirm the refactor did not disturb adjacent parser behavior.

### Notes

- This work is intended to make future notation regressions easier to add without over-engineering the test layer.
- The first consumer remains `get_written_durations.rs` only.
- The current concentration of helper code around the `NotationExpectation` and parser-fixture block makes this a low-risk extraction if the migration stays narrow.
- This is a harness extraction spec, not a test reorganization project. Any regrouping beyond the minimum needed to adopt the shared helpers should be treated as out of scope for the first pass.

## Review Notes

- Adversarial review completed
- Findings: 10 total, 8 fixed, 2 skipped
- Resolution approach: walk-through
