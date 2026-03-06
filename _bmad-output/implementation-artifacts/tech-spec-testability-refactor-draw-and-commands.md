---
title: "Testability Refactor — Draw Layer & Tauri Command Extraction"
slug: "testability-refactor-draw-and-commands"
created: "2026-03-06"
status: "ready-for-dev"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  [
    "rust 2021",
    "tauri 2.10",
    "skia-safe 0.93 (Metal)",
    "cargo-llvm-cov",
    "FxHashMap (rustc-hash)",
  ]
files_to_modify:
  - "renderer/mod.rs"
  - "instructions/draw/*.rs (20 files)"
  - "score/entries/tone.rs"
  - "score/flows.rs"
  - "score/players.rs"
  - "score/instruments/mod.rs"
code_patterns:
  - "impl methods on Track/Score/Flow/Player for extracted logic"
  - "headless RendererContext (backend: None, Converter(1.0, 1.0))"
  - "thin tauri command → delegates to impl method"
  - "FxHashMap::default() for all HashMap usage"
test_patterns:
  - "pattern-match on DrawInstruction variants (no PartialEq/Debug)"
  - "factory helpers: make_tone, make_note, make_renderer"
  - "float assertions: (result - expected).abs() < 1e-5"
  - "use super::*; in #[cfg(test)] mod tests"
---

# Tech-Spec: Testability Refactor — Draw Layer & Tauri Command Extraction

**Created:** 2026-03-06

## Overview

### Problem Statement

Rust test coverage is capped at ~53% regions / ~46% lines because (a) 20 draw functions require `RendererContext` which is currently untestable in unit tests (0% coverage on 19 of 20 draw files), and (b) ~32 tauri commands across 4 files embed business logic that can't be unit tested without a full Tauri runtime. Both areas need structural changes to unlock further coverage growth.

### Solution

A phased refactoring initiative with mixed small phases — each phase extracts a small amount of business logic from tauri commands into testable `impl` methods AND adds headless `RendererContext` tests for draw functions. Each phase is independently reviewable, committable, and moves coverage forward incrementally with per-phase coverage targets.

### Scope

**In Scope:**

- Headless `RendererContext` test infrastructure (construct with `backend: None`, inspect `instructions` vec)
- Draw function tests for all 20 draw files in `instructions/draw/`
- Extract business logic from tauri commands into `impl` methods in: `tone.rs` (12 cmds), `flows.rs` (6 cmds), `players.rs` (5 cmds), `instruments/mod.rs` (9 cmds)
- Unit tests for all extracted methods
- Per-phase coverage targets (ratcheting thresholds upward each phase)
- Pure helper extraction from draw files (similar to existing `get_accidental_slot_offset`)
- Derive `Debug` + `PartialEq` on `DrawInstruction` and `HitTarget` for test assertions

**Out of Scope:**

- Trait abstraction / renderer mocking (not needed — headless RendererContext suffices)
- Testing `engraves.rs` / `meta.rs` commands (already trivially thin setters — 31 commands)
- Frontend test coverage changes
- Any functional/behavioral changes — strict behavior preservation only
- `lib.rs` / `main.rs` / Metal backend / playback modules
- Commands that are already pure delegation (`tone_shift` → `Track::shift_entry`, `tone_duration_remove` → `Track::remove_entry`)

## Context for Development

### Codebase Patterns

- Well-tested files (tracks.rs 98%, duration.rs 100%) keep logic in `impl` blocks and have zero `#[tauri::command]` functions
- Poorly tested files (tone.rs 36%, flows.rs 28%, players.rs 26%, instruments/mod.rs 34%) embed business logic directly in `#[tauri::command]` function bodies
- Draw functions already record `DrawInstruction` variants into a `pub instructions: Vec<DrawInstruction>` field — `RendererContext::new()` creates one with `backend: None` and `Converter::new(1.0, 1.0)` (1:1 space→px mapping for easy test assertions)
- Test helpers follow pattern: private factory functions (`make_tone`, `make_note`, `four_four`) per test module
- `FxHashMap::default()` used everywhere (never `HashMap::new()`)
- `DrawInstruction` currently derives only `Clone` — needs `Debug` + `PartialEq` for test assertions (or use pattern matching)

### Files to Reference

| File                                | Purpose                                                                         | Current Coverage (Regions) |
| ----------------------------------- | ------------------------------------------------------------------------------- | -------------------------- |
| `renderer/mod.rs`                   | `RendererContext`, `DrawInstruction`, `HitTarget` definitions                   | 0% (246 regions)           |
| `renderer/converter.rs`             | `Converter` struct, space↔px conversion                                         | Has tests                  |
| `score/entries/tone.rs`             | `Tone` struct, 12 tauri commands, 3 `impl Track` methods                        | 36% (731 regions)          |
| `score/flows.rs`                    | `Flow`/`Flows` structs, 6 tauri commands, 4 `impl AppContext` methods           | 28% (386 regions)          |
| `score/players.rs`                  | `Player`/`Players` structs, 5 tauri commands                                    | 26% (334 regions)          |
| `score/instruments/mod.rs`          | `Instrument`/`Instruments` structs, 9 tauri commands, `Score::update_counts()`  | 34% (519 regions)          |
| `score/tracks.rs`                   | `Track` struct, `insert_entry`/`shift_entry`/`remove_entry` — reference pattern | 98%                        |
| `instructions/draw/*.rs` (20 files) | All draw functions — 0% except `draw_accidentals.rs` (64%)                      | 0% (19 files)              |

### Technical Decisions

- **Headless over mocking**: RendererContext already stores instructions internally with all fields `pub` — no trait abstraction needed. `backend: None` is the default; draw methods just push to the vec without touching the backend.
- **Mixed phases**: Each phase combines draw + command work for steady progress across both fronts
- **Derive Debug+PartialEq**: Adding derives to `DrawInstruction` enables cleaner test assertions vs. pattern matching. Safe change — derives don't affect runtime behavior.
- **Behavior preservation**: Tauri commands remain identical in signature and behavior; they just delegate to new `impl` methods
- **Pitch resolution dedup**: `tone_create` and `tone_pitch_set` both duplicate the accidental-from-key-signature resolution pattern — extract once as `Track::resolve_pitch()`
- **Sampler sync dedup**: 4 commands across `players.rs` and `instruments/mod.rs` duplicate the sampler sync pattern — extract as helper

### Investigation Findings — Command Classification

#### tone.rs (12 commands)

| Command                          | Logic             | Target         | Notes                                                |
| -------------------------------- | ----------------- | -------------- | ---------------------------------------------------- |
| `tone_create`                    | Thick (25 lines)  | `Track`        | Pitch resolution + articulation inheritance + insert |
| `tone_duration_set`              | Thin (2 lines)    | `Track`        | Single field set                                     |
| `tone_slice`                     | Medium (15 lines) | `Track`        | Split tone at tick, create remainder                 |
| `tone_pitch_set`                 | Medium (12 lines) | `Track`        | Pitch resolution (duplicates create logic)           |
| `tone_duration_remove`           | Delegation        | —              | Already calls `Track::remove_entry`                  |
| `tone_set_position_and_duration` | Medium (8 lines)  | `Track`        | Manual index management — code smell                 |
| `tone_shift`                     | Delegation        | —              | Already calls `Track::shift_entry`                   |
| `tone_unify_articulations`       | Medium (8 lines)  | `Track`        | Reconcile articulations at tick                      |
| `tone_pitch_shift_by_steps`      | Medium (10 lines) | `Track`        | Needs key sig from master track (cross-entity)       |
| `tone_pitch_shift_by_octaves`    | Thin (3 lines)    | `Track`        | Single delegation                                    |
| `tone_accidental_set`            | Thin (4 lines)    | `Track`/`Tone` | Base pitch + accidental offset                       |
| `tone_articulation_set`          | Thin (5 lines)    | `Track`        | Toggle + propagate to tick                           |

#### flows.rs (6 commands)

| Command                 | Logic      | Target       | Notes                                                    |
| ----------------------- | ---------- | ------------ | -------------------------------------------------------- |
| `flows_create`          | Thick      | `AppContext` | Creates flow + master track + wires players/layouts      |
| `flows_rename`          | Thin       | `Flow`       | Single field set                                         |
| `flows_move`            | Thin       | `Flows`      | Vec reorder                                              |
| `flows_assign_player`   | Delegation | —            | Already delegates to `AppContext::assign_player_to_flow` |
| `flows_unassign_player` | Delegation | —            | Already delegates                                        |
| `flows_remove`          | Thick      | `AppContext` | Multi-step teardown (players, tracks, layouts, UI)       |

#### players.rs (5 commands)

| Command                       | Logic  | Target       | Notes                                        |
| ----------------------------- | ------ | ------------ | -------------------------------------------- |
| `players_create`              | Thick  | `AppContext` | Create + wire flows/layouts                  |
| `players_move`                | Thin   | `Players`    | Vec reorder + `update_counts()`              |
| `players_assign_instrument`   | Medium | `AppContext` | Conditional flow wiring                      |
| `players_unassign_instrument` | Thick  | `AppContext` | Flow unwiring + sampler sync (3 state locks) |
| `players_remove`              | Thick  | `AppContext` | Complex teardown + sampler sync              |

#### instruments/mod.rs (9 commands)

| Command                     | Logic  | Target                     | Notes                                           |
| --------------------------- | ------ | -------------------------- | ----------------------------------------------- |
| `instruments_create`        | Medium | `AppContext`/`Instruments` | Definition lookup + ID gen                      |
| `instruments_move`          | Thick  | `AppContext`               | Cross-player logic, conditional flow assignment |
| `instruments_remove`        | Medium | `Instruments`              | Simple removal + sampler sync                   |
| `instruments_assign_plugin` | Medium | `Instrument`               | Field set + sampler sync                        |
| `instruments_set_volume`    | Thin   | `Instrument`               | Clamp + set + mixer param                       |
| `instruments_set_pan`       | Thin   | `Instrument`               | Same pattern as volume                          |
| `instruments_set_mute`      | Thin   | `Instrument`               | Same pattern                                    |
| `instruments_set_solo`      | Thin   | `Instrument`               | Same pattern                                    |

### Draw File Complexity Tiers

| Tier                                                   | Files                                                                                                      | Strategy                                                  |
| ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| **Trivial** (1 instruction, no branching)              | `draw_stem`, `draw_clef`, `draw_flag`, `draw_braces`, `draw_names`, `draw_staves`, `draw_systemic_barline` | Direct headless test; ~5 min each                         |
| **Low** (few instructions, simple branching)           | `draw_dot`, `draw_accidentals` (partial), `draw_sub_brackets`, `draw_key_signature`, `draw_time_signature` | Headless test + extract pure helpers                      |
| **Moderate** (multiple instructions, shunts/selection) | `draw_notehead`, `draw_rest`, `draw_articulations`                                                         | Headless test + multiple branches to cover                |
| **High** (complex iteration, nested maps)              | `draw_barline`, `draw_beam`, `draw_ledger_lines`, `draw_brackets`, `draw_ties`                             | Need test fixture builders; extract pure geometry helpers |

### Baseline Coverage (pre-refactoring)

| Metric    | Current | Threshold |
| --------- | ------- | --------- |
| Regions   | 52.87%  | 50%       |
| Functions | 67.69%  | 65%       |
| Lines     | 45.56%  | 40%       |
| Tests     | 644     | —         |

## Implementation Plan

### Phase 1: Draw Test Infrastructure + Trivial Draw Tests

**Goal**: Establish headless RendererContext testing pattern and cover the simplest draw files. This phase builds the foundation that all subsequent draw tests depend on.

**Coverage target**: Regions 54%, Functions 70%, Lines 47%

- [x] Task 1.1: Add `Debug` + `PartialEq` derives to `DrawInstruction` and `HitTarget`
  - File: `src-tauri/src/renderer/mod.rs`
  - Action: Change `#[derive(Clone)]` to `#[derive(Clone, Debug, PartialEq)]` on `DrawInstruction` enum and `HitTarget` struct. Also add `Debug` + `PartialEq` to any referenced types that lack them (e.g. `FontFamily`, `Justify`, `Align` — check if `Selection` already has them, if not use pattern matching for HitTarget tests instead).
  - Notes: Pure derive additions — zero runtime behavior change. If `Selection` or `Color` don't impl `PartialEq`, leave `HitTarget` as `Clone` only and use pattern matching in tests. Test by running `cargo test --lib`.

- [x] Task 1.2: Create shared draw test helper: `make_renderer()`
  - File: `src-tauri/src/renderer/mod.rs` (add `#[cfg(test)]` module)
  - Action: Add a `pub(crate)` test helper function that constructs a bare `RendererContext` with `backend: None`, `Converter::new(1.0, 1.0)`, `px_per_mm: 1.0`, zero-sized dimensions, empty vecs, default bounds, `pixel_density: 1.0`. This avoids `Mutex` wrapping (which `RendererContext::new()` does).
  - Notes: Since test modules can't easily share helpers across files, define this as `pub(crate) fn make_test_renderer() -> RendererContext` inside `#[cfg(test)] mod tests` in `renderer/mod.rs`, or as a `#[cfg(test)] pub mod test_helpers` at the renderer module level so draw test files can import it.

- [x] Task 1.3: Add headless tests for trivial draw files (7 files)
  - `draw_stem.rs` (96% regions), `draw_clef.rs` (96%), `draw_flag.rs` (97%), `draw_braces.rs` (98%), `draw_names.rs` (99%), `draw_staves.rs` (99%), `draw_systemic_barline.rs` (98%)
  - Files: `draw_stem.rs`, `draw_clef.rs`, `draw_flag.rs`, `draw_braces.rs`, `draw_names.rs`, `draw_staves.rs`, `draw_systemic_barline.rs`
  - Action: For each file, add a `#[cfg(test)] mod tests` block that:
    1. Constructs a `RendererContext` using the helper from Task 1.2
    2. Constructs minimal input fixtures (Point, StemDef, etc.)
    3. Calls the draw function
    4. Asserts on `renderer.instructions.len()` and pattern-matches the instruction variant to verify correct type, color, and approximate coordinates
  - Notes: These files each produce a single `DrawInstruction` with no branching — they are the simplest possible draw tests. Each test should take ~5–10 lines. Start with `draw_stem.rs` as the template pattern, then replicate for the others.

- [x] Task 1.4: Add basic tests for `RendererContext` drawing methods
  - File: `src-tauri/src/renderer/mod.rs`
  - Action: Test `line()`, `text()`, `circle()`, `shape()`, `curve()`, `rectangle()`, `dashed_line()`, and `clear()`. Construct a `RendererContext`, call each method, verify the corresponding `DrawInstruction` is pushed to `self.instructions`. Test `clear()` empties both `instructions` and `hit_targets`.
  - Notes: These are the most foundational tests — they verify the infrastructure that all draw tests rely on.

---

### Phase 2: Low-Complexity Draw Tests + tone.rs Thin Command Extraction

**Goal**: Cover draw files with simple branching and extract the simplest tone commands into `impl Track` methods.

**Coverage target**: Regions 56%, Functions 72%, Lines 49%

- [x] Task 2.1: Add tests for low-complexity draw files (5 files)
  - Files: `draw_dot.rs`, `draw_accidentals.rs` (add `draw_accidental` function test), `draw_sub_brackets.rs`, `draw_key_signature.rs`, `draw_time_signature.rs`
  - Action: For each file:
    1. Construct headless `RendererContext`
    2. Build required data fixtures (e.g. `DotOffsets = FxHashSet<i8>`, `AccidentalNotation`, `Tone`, `KeySignature`, `TimeSignature`)
    3. Call the draw function with different input combinations to cover branches
    4. Assert on instruction count and variant fields
  - Notes: `draw_dot` has 3 branches (post_shunt / up_stem_flag / default) — test all three. `draw_accidentals` already has pure helper tests; add a test for the `draw_accidental` function itself. Optionally extract `dot_x_offset()` as a pure helper from `draw_dot.rs`.

- [x] Task 2.2: Extract `Track::set_tone_duration()` from `score_entries_tone_duration_set`
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action:
    1. Add method to `impl Track`: `pub fn set_tone_duration(&mut self, tone_id: UniqueId, duration: Ticks)` — looks up entry by ID, if `Entry::Tone`, sets `tone.duration = duration`.
    2. Update `score_entries_tone_duration_set` command to call `track.set_tone_duration(tone_id, duration)`.
    3. Add unit test for the new method.
  - Notes: Simplest possible extraction — 2 lines of logic. Good first example of the pattern.

- [x] Task 2.3: Extract `Track::set_tone_pitch()` with shared pitch resolution
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action:
    1. Add method `pub fn resolve_pitch(&self, tick: Tick, pitch_int: u8, accidental: Option<Accidental>) -> Pitch` — encapsulates the repeated pitch resolution logic (if `Some(accidental)` → `Pitch::new`, else → lookup key sig → `Pitch::from_int`).
    2. Add method `pub fn set_tone_pitch(&mut self, tone_id: UniqueId, pitch_int: u8, accidental: Option<Accidental>)` — calls `self.resolve_pitch()` + sets `tone.pitch`.
    3. Update `score_entries_tone_pitch_set` command to delegate.
    4. Add unit tests for `resolve_pitch` (with/without explicit accidental, with/without key signature on track) and `set_tone_pitch`.
  - Notes: This deduplicates pitch resolution — `tone_create` will also use `resolve_pitch` in Phase 3.

- [x] Task 2.4: Extract `Track::shift_tone_pitch_by_octaves()` and `Track::set_tone_accidental()`
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action:
    1. `pub fn shift_tone_pitch_by_octaves(&mut self, tone_id: UniqueId, octaves: i8) -> Option<u8>` — delegates to `tone.pitch.shift_by_octaves()`.
    2. `pub fn set_tone_accidental(&mut self, tone_id: UniqueId, accidental: Accidental) -> Option<u8>` — computes base pitch + accidental offset, creates new `Pitch`.
    3. Update both commands to delegate.
    4. Add unit tests for both methods.
  - Notes: Both are thin extractions (3–4 lines each). Low risk.

---

### Phase 3: Moderate Draw Tests + tone.rs Medium Command Extraction

**Goal**: Cover draw files with shunts/selection branching. Extract the medium-complexity tone commands.

**Coverage target**: Regions 58%, Functions 74%, Lines 52%

- [x] Task 3.1: Add tests for moderate draw files (3 files)
  - Files: `draw_notehead.rs`, `draw_rest.rs`, `draw_articulations.rs`
  - Action:
    1. **draw_notehead**: Test with no shunt, pre-shunt, post-shunt. Test with selection matching the tone (blue color) vs. no selection (black). Verify both `DrawInstruction::Text` and `HitTarget` output.
    2. **draw_rest**: Test each rest glyph variant. Verify text instruction with correct glyph and position.
    3. **draw_articulations**: Test with different articulation types and stem directions. Verify correct glyph placement relative to notehead.
  - Notes: `draw_notehead` has 13 params but most are scalars — the main fixture work is constructing `Shunts` with a matching `(Tick, UniqueId)` key. Create a `make_shunts()` helper.

- [x] Task 3.2: Extract `Track::create_tone()` from `score_entries_tone_create`
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action:
    1. Add `pub fn create_tone(&mut self, tone_id: UniqueId, tick: Tick, duration: Ticks, pitch_int: u8, accidental: Option<Accidental>, velocity: u8, articulations: Articulations) -> UniqueId` — calls `self.resolve_pitch()` (from Task 2.3), resolves articulation inheritance via `first_articulations_at_tick`, creates and inserts tone, propagates articulations.
    2. Update command to: generate ID, call `track.create_tone(...)`, emit.
    3. Add tests: create tone with explicit accidental, create tone inferring from key sig, create tone inheriting articulations from existing tones, create tone with its own articulations overwriting others.
  - Notes: This is the largest single extraction (~25 lines). Reuses `resolve_pitch` from Task 2.3.

- [x] Task 3.3: Extract `Track::slice_tone()` from `score_entries_tone_slice`
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action:
    1. Add `pub fn slice_tone(&mut self, tone_id: UniqueId, new_tone_id: UniqueId, slice_tick: Tick) -> Option<UniqueId>` — clones tone, validates slice point is within tone duration, shortens original, creates remainder, inserts it.
    2. Update command to: generate new ID, call `track.slice_tone(...)`, emit.
    3. Add tests: slice in middle (happy path), slice at start (returns None), slice at end (returns None), slice outside range (returns None), verify both halves have correct tick/duration/pitch.
  - Notes: Guard clauses (`slice_tick <= start || slice_tick >= end`) are the key edge cases to test.

- [x] Task 3.4: Extract `Track::set_tone_position_and_duration()` and `Track::unify_articulations_at_entry()`
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action:
    1. `pub fn set_tone_position_and_duration(&mut self, tone_id: UniqueId, tick: Tick, duration: Ticks)` — removes from old `by_tick` index, updates fields, adds to new index. Should use `shift_entry` + duration set internally rather than manually managing indices.
    2. `pub fn unify_articulations_at_entry(&mut self, entry_id: UniqueId)` — reconciles articulations: if entry has articulations, apply to all at tick; else inherit from neighbours.
    3. Update both commands to delegate.
    4. Tests for position change (verify `by_tick` index updated correctly), unify with existing articulations, unify with empty articulations.
  - Notes: `set_tone_position_and_duration` currently reaches into `track.entries.by_tick` directly — the extracted method should clean this up to use existing `Track` index management methods where possible.

- [x] Task 3.5: Extract `Track::shift_tone_pitch_by_steps()`
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action:
    1. Add `pub fn shift_tone_pitch_by_steps(&mut self, tone_id: UniqueId, steps: i8, key_signature: &KeySignature) -> Option<u8>` — the method takes an already-resolved key signature (the command is responsible for looking up the master track to get it).
    2. Update command: look up flow → master track → key sig, then call `track.shift_tone_pitch_by_steps(tone_id, steps, &key_sig)`.
    3. Tests with different step values and key signatures.
  - Notes: The key signature resolution stays in the command since it requires cross-entity lookup (flow → master track). The method only does the pitch mutation.

- [x] Task 3.6: Extract `Track::toggle_articulation_at_tone()`
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action:
    1. Add `pub fn toggle_articulation_at_tone(&mut self, tone_id: UniqueId, articulation: Articulation)` — looks up tone, clones its articulations, toggles the given articulation, applies to all tones at tick.
    2. Update command to delegate.
    3. Tests: toggle on (adds articulation), toggle off (removes), propagation to other tones at same tick.
  - Notes: Uses existing `apply_articulations_to_tick()` internally.

---

### Phase 4: Complex Draw Tests — Barlines, Beams, Ledger Lines

**Goal**: Cover the high-complexity draw files that need fixture builders and have heavy branching.

**Coverage target**: Regions 60%, Functions 76%, Lines 54%

- [x] Task 4.1: Create shared test fixture builders for draw tests
  - File: `src-tauri/src/instructions/draw/mod.rs` (add `#[cfg(test)] pub mod test_helpers`)
  - Action: Create reusable builder functions:
    1. `make_vertical_spacing(stave_ids: &[(UniqueId, f32, f32)]) -> VerticalSpacing` — builds spacing with given stave positions
    2. `make_vertical_spans(span_pairs: &[(UniqueId, UniqueId, f32, f32, f32, f32)]) -> VerticalSpans` — builds spans matching spacing IDs
    3. `make_beam(start: Tick, stop: Tick, ticks: &[(Tick, u8)]) -> Beam` — builds a Beam with tick→beam_count mapping
    4. `make_stem_lengths(entries: &[(Tick, StemDef)]) -> StemLengths` — builds tick→StemDef mapping
  - Notes: These builders eliminate the boilerplate needed to test complex draw files. Each builder takes the minimum info and constructs consistent data structures.

- [x] Task 4.2: Add tests for `draw_barline.rs`
  - File: `src-tauri/src/instructions/draw/draw_barline.rs`
  - Action: Test all 8 `BarlineDrawType` variants (Single, Double, Triple, Dashed, Final, StartRepeat, EndRepeat, EndStartRepeat). For each, verify:
    1. Correct number of line/dashed_line instructions
    2. Repeat dot circles for repeat variants (EndRepeat, StartRepeat, EndStartRepeat)
    3. Correct top/bottom coordinates from vertical spans
  - Notes: Use fixture builder from Task 4.1. Need at least 1 barline span and 1 stave in vertical spacing. Optionally extract the `top`/`bottom` calculation as a pure helper.

- [x] Task 4.3: Add tests for `draw_beam.rs` + extract pure helpers
  - File: `src-tauri/src/instructions/draw/draw_beam.rs`
  - Action:
    1. Extract `fn beam_angle(start: &StemDef, stop: &StemDef) -> f32` as a pure helper (currently inline in `draw_beam`).
    2. Extract `fn stem_length_tweak(direction: &Direction) -> f32` as a pure helper.
    3. Test extracted helpers directly (angle calculation with various stem positions, zero-division guard, direction-based tweak).
    4. Test `draw_beam` with headless renderer: simple 2-note beam (1 outer shape), beam with inner segments (multiple shapes), fractional beam, up vs. down stem direction.
  - Notes: The Beam struct fixture needs `ticks`, `start`, `stop`, `beam_segments`, and `fractional_beams` populated consistently. Use builder from Task 4.1.

- [x] Task 4.4: Add tests for `draw_ledger_lines.rs`
  - File: `src-tauri/src/instructions/draw/draw_ledger_lines.rs`
  - Action: Test ledger lines above staff, below staff, and combination. Verify correct number of line instructions and y-positions.
  - Notes: Has 4 functions (185 regions). Needs `LedgerLines` + `VerticalSpacing` fixtures.

- [x] Task 4.5: Add tests for `draw_brackets.rs` and `draw_ties.rs`
  - File: `src-tauri/src/instructions/draw/draw_brackets.rs`, `draw_ties.rs`
  - Action:
    1. **draw_brackets**: Test bracket rendering with vertical span. Verify line instructions.
    2. **draw_ties**: Test tie curve rendering between two note positions. This is the most complex draw file (261 regions, 9 functions). Focus on the main `draw_tie` function first, then `draw_tie_segment`. Test up-curve and down-curve variants.
  - Notes: `draw_ties` has the most internal functions (9). May benefit from extracting tie control point calculation as a pure helper.

---

### Phase 5: flows.rs + instruments/mod.rs Command Extraction

**Goal**: Extract business logic from flow and instrument tauri commands into testable methods.

**Coverage target**: Regions 62%, Functions 78%, Lines 56%

- [ ] Task 5.1: Extract `Flows::rename()` and `Flows::reorder()`
  - File: `src-tauri/src/score/flows.rs`
  - Action:
    1. Add `pub fn rename(&mut self, flow_id: UniqueId, title: String)` to `impl Flows` — looks up flow, sets title.
    2. Add `pub fn reorder(&mut self, from_index: usize, to_index: usize)` to `impl Flows` — removes from `order[from]`, inserts at `to`.
    3. Update `score_flows_rename` and `score_flows_move` commands to delegate.
    4. Add tests for both methods.
  - Notes: Trivial extractions. Low risk. These methods already fit naturally on `Flows`.

- [ ] Task 5.2: Extract `Score::update_counts()` tests
  - File: `src-tauri/src/score/instruments/mod.rs`
  - Action: `Score::update_counts()` already lives on `impl Score` and is testable — just needs tests. Test:
    1. Two violins from same player → numbered "I", "II"
    2. Two violins from different players → still numbered
    3. Single instrument → no count assigned
    4. Mixed instrument types → independent counting
  - Notes: This method already exists in testable form — it just lacks tests. Requires constructing a `Score` with `Players`, `Instruments`, and player `order`. May need a `make_score()` test helper.

- [ ] Task 5.3: Extract instrument mixer methods
  - File: `src-tauri/src/score/instruments/mod.rs`
  - Action:
    1. Add to `impl Instrument`:
       - `pub fn set_volume(&mut self, volume: u8) -> u8` — clamps to 0–127, sets, returns clamped value
       - `pub fn set_pan(&mut self, pan: u8) -> u8` — same pattern
       - `pub fn set_mute(&mut self, mute: bool)` — sets mute
       - `pub fn set_solo(&mut self, solo: bool)` — sets solo
    2. Update the 4 mixer commands to: call the impl method, then update `transport_shared.mix_params` separately.
    3. Add tests for each method (especially volume/pan clamping).
  - Notes: The `transport_shared` update stays in the command (it needs `Arc<TransportShared>`). We're extracting just the data mutation.

- [ ] Task 5.4: Add tests for `impl AppContext` flow methods
  - File: `src-tauri/src/score/flows.rs`
  - Action: The 4 `impl AppContext` methods (`assign_player_to_flow`, `assign_instrument_to_flow`, `unassign_player_from_flow`, `unassign_instrument_from_flow`) already exist on `AppContext`. If we can construct an `AppContext` in tests (no Tauri runtime needed — it's just data), test these directly. Otherwise, extract the score-mutation logic out of `AppContext` and onto `Score`.
  - Notes: **Investigate first**: Check if `AppContext` can be constructed in tests. It contains `score: Score`, `ui: UIState`, `id_counter`, etc. If it's all plain data, a `make_app_context()` test helper would unlock testing for all `impl AppContext` methods across flows, players, and instruments. If it can't be constructed (e.g. requires Tauri types), then extract the logic onto `Score` instead.

---

### Phase 6: players.rs Command Extraction + Sampler Sync Dedup

**Goal**: Extract player command logic and deduplicate the sampler sync pattern.

**Coverage target**: Regions 64%, Functions 80%, Lines 58%

- [ ] Task 6.1: Extract `Players::reorder()`
  - File: `src-tauri/src/score/players.rs`
  - Action:
    1. Add `pub fn reorder(&mut self, from_index: usize, to_index: usize)` to `impl Players`.
    2. Update `score_players_move` to call `players.reorder(...)` then `score.update_counts()`.
    3. Add test.
  - Notes: Identical pattern to `Flows::reorder()` from Task 5.1.

- [ ] Task 6.2: Deduplicate sampler sync pattern
  - File: New helper function or method (location TBD based on Task 5.4 investigation)
  - Action: The following pattern appears in 4 commands:
    ```rust
    let assigned_plugin_ids: Vec<_> = state.score.instruments.by_id.values()
        .filter_map(|i| i.assigned_plugin_id.clone())
        .collect();
    let mut sampler = sampler_state.lock().unwrap();
    let audio = audio_state.lock().unwrap();
    let sample_rate = audio.sample_rate();
    sampler.sync_with_instruments(&assigned_plugin_ids, sample_rate);
    sampler.emit(&app);
    ```
    Extract as a standalone function: `fn sync_sampler(score: &Score, sampler_state: &SamplerContextState, audio_state: &AudioContextState, app: &AppHandle)`.
  - Notes: This doesn't need to be on a struct — it's a coordination helper. The function itself isn't unit-testable (needs sampler/audio state), but the extraction reduces duplication and makes the commands cleaner. The main benefit is code clarity, not direct testability.

- [ ] Task 6.3: Extract remaining player command logic (if AppContext is constructable)
  - File: `src-tauri/src/score/players.rs`
  - Action: Based on findings from Task 5.4:
    - **If AppContext constructable**: Add tests for `score_players_create` by testing the orchestration logic on `AppContext` directly.
    - **If not**: Extract the player-creation logic that doesn't touch sampler/audio into a `Score` method: `pub fn create_player(&mut self, player_id: UniqueId, player_type: PlayerType)` + `pub fn wire_player_to_flows_and_layouts(&mut self, player_id: UniqueId)`.
  - Notes: This task's approach depends on Phase 5 investigation. Mark as "conditional" — adjust scope based on Task 5.4 findings.

---

### Phase 7: Coverage Threshold Ratchet + Remaining Gaps

**Goal**: Bump coverage thresholds, sweep for any remaining low-hanging fruit, document final state.

**Coverage target**: Regions 65%+, Functions 80%+, Lines 58%+

- [ ] Task 7.1: Bump coverage thresholds
  - File: `src-tauri/Cargo.toml` or coverage config, `TESTING.md`
  - Action: Update coverage thresholds to match achieved levels (ratchet up). Update `TESTING.md` with new values.
  - Notes: Run `cargo llvm-cov --lib` to get final numbers before setting thresholds.

- [ ] Task 7.2: Audit remaining uncovered regions
  - Action: Run `cargo llvm-cov --lib` and review any files still below 50% regions. Identify if there are quick wins (untested pure functions) that were missed.
  - Notes: This is a sweep task — may generate follow-up work items.

- [ ] Task 7.3: Add `constrain_pan()` and `update_bounds()` tests to `renderer/mod.rs`
  - File: `src-tauri/src/renderer/mod.rs`
  - Action: These are pure computation methods on `RendererContext` that don't need a backend. Test `constrain_pan` with various viewport/page/zoom combos. Test `update_bounds` with `backend: None` (should just set fields without error).
  - Notes: Quick wins for renderer coverage.

## Acceptance Criteria

### Draw Layer Testing

- [x] AC 1: Given a `RendererContext` constructed with `backend: None` and `Converter::new(1.0, 1.0)`, when any draw function from `instructions/draw/` is called with valid inputs, then `renderer.instructions` contains the expected `DrawInstruction` variant(s) with correct field values.

- [x] AC 2: Given `DrawInstruction` and `HitTarget` with `Debug` + `PartialEq` derives (or pattern-matched assertions), when a draw test asserts on instruction contents, then the assertion compiles and provides useful error messages on failure.

- [x] AC 3: Given `draw_barline` called with each of the 8 `BarlineDrawType` variants, when the function completes, then `renderer.instructions` contains the correct number and type of line/dashed_line/circle instructions per variant.

- [x] AC 4: Given `draw_beam` called with a simple 2-note beam, when the function completes, then `renderer.instructions` contains at least 1 `DrawInstruction::Shape` representing the outer beam.

- [x] AC 5: Given `draw_notehead` called with `selection: Some(Selection::Tone { tone_id, .. })` matching the tone, when rendered, then the text instruction uses color `rgb(14, 165, 233)` (blue).

### Command Extraction — tone.rs

- [x] AC 6: Given a `Track` with a key signature entry, when `Track::resolve_pitch(tick, pitch_int, None)` is called, then the returned `Pitch` has an accidental inferred from the key signature at or before that tick.

- [x] AC 7: Given a `Track` with a tone, when `Track::slice_tone(tone_id, new_id, slice_tick)` is called with `slice_tick` within the tone's duration, then the original tone's duration is shortened, a new tone is inserted at `slice_tick` with the remaining duration, and `Some(new_id)` is returned.

- [x] AC 8: Given a `Track` with a tone, when `Track::slice_tone()` is called with `slice_tick` at or before the tone's start tick (or at/after its end), then `None` is returned and no modifications are made.

- [x] AC 9: Given a `Track` with multiple tones at the same tick, when `Track::create_tone()` is called with non-empty articulations, then all existing tones at that tick have their articulations updated to match.

- [x] AC 10: Given the `score_entries_tone_create` command before and after refactoring, when called with identical parameters, then the resulting `Score` state is identical (behavior preservation).

### Command Extraction — flows/players/instruments

- [ ] AC 11: Given a `Score` with instruments across players, when `Score::update_counts()` is called, then instruments of the same type across the player order receive sequential count labels.

- [ ] AC 12: Given an `Instrument`, when `instrument.set_volume(200)` is called, then `instrument.volume` is clamped to `127`.

- [ ] AC 13: Given the coverage thresholds after all phases complete, when `cargo llvm-cov --lib --fail-under-regions=X` is run, then the test suite passes with the ratcheted thresholds.

## Additional Context

### Dependencies

- No new external crates required
- All work uses existing: `skia-safe` (for `Color` type in tests), `rustc-hash` (`FxHashMap`/`FxHashSet`)
- Phase 5 Task 5.4 may reveal whether `AppContext` is constructable in tests — this determines the approach for testing flow/player orchestration logic

### Testing Strategy

- **Unit tests only** — no integration tests needed. All extracted methods operate on plain data structures.
- **Headless rendering** — `RendererContext` with `backend: None`, `Converter::new(1.0, 1.0)` (1:1 space→px)
- **Pattern**: Each test follows the Arrange → Act → Assert pattern. Factory helpers construct minimal fixtures.
- **Coverage verification**: Run `cargo llvm-cov --lib` after each phase to verify coverage improvements. Bump thresholds at Phase 7.
- **Behavior verification**: After each command extraction, run `cargo test --lib` to verify 0 regressions. The tauri commands remain identical in signature — only the internal implementation changes.

### Risk Mitigation

- **Highest risk**: `Track::create_tone()` extraction (Phase 3, Task 3.2) — the largest single extraction at ~25 lines of logic involving pitch resolution, articulation inheritance, and insertion. Mitigated by: extracting `resolve_pitch` first (Phase 2), having comprehensive tests for the new method, and the command's behavior being verified by existing frontend functionality as a safety net.
- **Medium risk**: `set_tone_position_and_duration` manual index management — currently reaches into `track.entries.by_tick` directly. The extraction should clean this up to use existing Track methods, but must preserve the same net effect. Mitigated by: testing both the Track method and verifying `by_tick` index consistency.
- **Low risk**: Draw tests (all phases) — these are additive-only changes (new test modules). Zero risk of behavioral regression since no production code is modified.
- **Low risk**: `Debug`/`PartialEq` derives — purely additive trait implementations with no runtime effect.

### Notes

- The **sampler sync dedup** (Task 6.2) improves code quality but doesn't directly increase coverage — the sampler sync code path is inherently untestable without a sampler fixture.
- **AppContext constructability** (Task 5.4) is the key unknown — if AppContext is plain data (no Tauri runtime types in its struct), it unlocks direct testing of ~10 `impl AppContext` methods. If not, logic must be extracted to `Score` methods instead. Resolve this early in Phase 5.
- Some `impl Track` methods in tone.rs already exist (`get_tones_at_tick`, `first_articulations_at_tick`, `apply_articulations_to_tick`) — new methods should follow the same pattern on the same `impl Track` block.
- **Future work** (out of scope): Refactoring the "thick" AppContext commands (`flows_create`, `flows_remove`, `players_create`, `players_remove`) into a more transaction-like pattern would further improve testability, but is a larger architectural change best tackled separately.
