# Story 5.1: Add Targeted Regression Coverage for Each Tuplet Seam

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want targeted regression coverage across parser, rendering, playback, persistence, and migration seams,
so that tuplet support is protected where it can actually fail.

## Scope Note

- This story is the first Epic 5 hardening pass after the tuplet feature work in Epics 1 through 4. It strengthens the existing seam-owned regression corpus instead of adding new feature behavior.
- Coverage must stay close to the owning modules. Do not introduce a repo-wide tuplet test framework, generic coverage padding, or a second orchestration layer that hides which seam failed.
- The regression corpus should prove the shipped MVP triplet contract only: isolated tuplets, partially filled tuplets, boundary-crossing tones, projection-driven playback timing, and save/load round-trips.
- MVP tuplets remain explicitly non-nestable. Nested tuplet coverage belongs out of scope unless a later story intentionally broadens the product contract.

## Acceptance Criteria

1. **Given** the implemented tuplet feature set across earlier epics
   **When** regression coverage is added
   **Then** the suite includes explicit tests for parser, rendering, playback, persistence, and migration behavior relevant to supported tuplets
   **And** the coverage lives close to the owning modules rather than in a disconnected repo-wide framework

2. **Given** supported triplet cases and representative boundary cases
   **When** the regression corpus is run
   **Then** it exercises isolated triplets, partially filled tuplets, boundary splitting, projection-driven playback timing, and load/save round-trips
   **And** failures identify the relevant ownership seam clearly enough to debug

3. **Given** MVP tuplets are non-nestable
   **When** the regression corpus is defined
   **Then** nested tuplet coverage is excluded from MVP expectations
   **And** tests assume the single-level child-content invariant

## Tasks / Subtasks

- [ ] Audit the current tuplet regression footprint seam by seam before adding new tests (AC: 1, 2, 3)
  - [ ] Inventory the existing parser coverage around tuplet notation derivation, especially `src-tauri/src/instructions/parse/get_written_durations.rs` and nearby parser-local helpers.
  - [ ] Inventory rendering coverage for tuplet bracket, number, rest, tie, beam, and hit-target behavior across `src-tauri/src/instructions/draw/` and `src-tauri/src/renderer/`.
  - [ ] Inventory playback coverage in `src-tauri/src/playback/tuplet_projection.rs` and `src-tauri/src/playback/transport.rs`.
  - [ ] Inventory persistence and migration coverage in `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, and `src-tauri/src/migrations/`.
- [ ] Add or extend parser-local tuplets regressions close to the owning seam (AC: 1, 2, 3)
  - [ ] Reuse `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs` before adding any new helper surface.
  - [ ] Cover isolated tuplets, partially filled tuplets, and host/tuplet boundary-crossing material without introducing nested tuplets.
  - [ ] Keep parser assertions scenario-driven and musically named so failures identify the exact notation seam.
- [ ] Add or extend rendering regressions for supported tuplets behavior (AC: 1, 2)
  - [ ] Protect `draw_tuplet.rs` behavior for bracket and number presentation under supported triplet cases.
  - [ ] Extend nearby renderer tests only where needed to keep selection, hit targets, and drawn tuplet artifacts aligned with the shipped notation contract.
  - [ ] Avoid introducing screenshot-only or disconnected end-to-end rendering coverage when unit or parser-adjacent assertions can prove the seam more directly.
- [ ] Add or extend playback regressions for projection-driven tuplets timing (AC: 1, 2)
  - [ ] Reuse the existing exact-timing tests in `src-tauri/src/playback/tuplet_projection.rs` and the scheduled-note tests in `src-tauri/src/playback/transport.rs`.
  - [ ] Cover representative mixed host-plus-tuplet timing and boundary-crossing continuity without re-deriving tuplet semantics inside transport-local helpers.
- [ ] Add or extend persistence and migration regressions in the owning file workflow (AC: 1, 2, 3)
  - [ ] Keep save/load, deterministic round-trip, legacy non-tuplet load, and nested-child rejection coverage in `src-tauri/src/files.rs` and related persistence seams.
  - [ ] Confirm migration coverage continues to express the current stance clearly: legacy non-tuplet files remain loadable and no speculative tuplet migration is added unless compatibility actually requires it.
- [ ] Validate the resulting seam-owned regression corpus with standard repo checks
  - [ ] Run focused Rust tests for touched parser, draw, playback, and file-workflow seams.
  - [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml`.
  - [ ] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
  - [ ] Run `npm run lint` and `npm run check-types` if any frontend or docs-facing typed seams are touched.
  - [ ] Run `npm run build:fixtures` if any fixture JSON under `tests/fixtures/source/` changes.

## Dev Notes

### Story Foundation

- Epic 5 hardens the tuplet system after the product behavior was established across persistence, editing, notation, layout, and playback.
- The purpose of Story 5.1 is not to raise raw test counts. It is to ensure each tuplet seam has local regression proof that matches the actual architecture.
- The acceptance criteria map directly to the maintainer journey in the PRD: tuplets must remain exact and debuggable without hiding failures behind broad black-box coverage.
- This story should leave the codebase with a clear answer to: which module owns the failure if a shipped tuplet rule breaks?

### Technical Requirements

- Keep regressions near the seam that owns the behavior. Parser coverage belongs with parser modules, playback coverage belongs with playback modules, and persistence coverage belongs with the file path.
- Prefer direct unit tests and existing local harnesses over new top-level test frameworks.
- Cover the representative tuplets cases called out by the planning docs:
  - isolated triplets
  - partially filled tuplets with derived empty local space
  - host-owned tones that cross into tuplets
  - child-owned tones that extend back into host time
  - projection-driven playback timing
  - save/load round-trips and legacy non-tuplet continuity
- Keep MVP boundaries explicit: no nested tuplets, no post-MVP broader ratios, no implied support for unsupported directive content inside tuplets.

### Architecture Compliance

- Tuplets remain one shared semantic model across persistence, parsing, rendering, and playback.
- Regressions should prove that downstream seams consume the shared model rather than reconstructing ownership, boundary meaning, projection, or continuity locally.
- Preserve the timing invariant: tuplet container geometry is compressed on the host track, while crossing duration remains literal on the canonical expanded timeline.
- Migration discipline still applies. Tests may confirm the current no-new-migration stance, but they must not normalize ad hoc repair logic into the load path.

### Library / Framework Requirements

- Rust 2021 in the current Tauri stack remains the primary implementation surface.
- Reuse existing `#[cfg(test)]` modules and parser-local support utilities before introducing any new helper files.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary parser seams:
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
  - `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`
- Likely render seams:
  - `src-tauri/src/instructions/draw/draw_tuplet.rs`
  - `src-tauri/src/instructions/draw/draw_notehead.rs`
  - `src-tauri/src/instructions/draw/draw_rest.rs`
  - `src-tauri/src/renderer/hit_test.rs`
- Playback seams:
  - `src-tauri/src/playback/tuplet_projection.rs`
  - `src-tauri/src/playback/transport.rs`
- Persistence and migration seams:
  - `src-tauri/src/files.rs`
  - `src-tauri/src/soloc.rs`
  - `src-tauri/src/migrations/mod.rs`
  - `src-tauri/src/migrations/next.rs`
- Fixture and test-guidance touchpoints if discovery shows they are needed:
  - `tests/fixtures/source/rhythms/`
  - `tests/fixtures/README.md`

### Testing Requirements

- Parser tests should remain scenario-driven and local to parser ownership seams.
- Rendering tests should protect supported bracket, number, rest, tie, beam, and hit-target behavior without inventing a separate notation truth source.
- Playback tests should keep exact projection and active-flow behavior aligned with earlier Epic 4 guarantees.
- Persistence tests should preserve deterministic `.soloc` round-trips, legacy non-tuplet continuity, and rejection of unsupported nested tuplet child data.
- If fixture JSON changes, keep files grouped by concern and do not commit built `.soloc` outputs.

### Previous Story Intelligence

- Story 1.6 already established strong persistence coverage for deterministic tuplet round-trips, legacy non-tuplet continuity, and no speculative migration.
- Story 2.5 locked the explicit MVP delete and slice rules that should remain protected in parser and persistence behavior.
- Story 3.4 stabilized tuplets layout and spacing, and Story 4.4 locked playback to the active-flow boundary. Story 5.1 should protect those outcomes rather than redesign them.
- The most useful prior precedent for local regression style is Story 3.2's parser-local safety approach: seam-owned tests, musically named scenarios, and no repo-wide framework.

### Git Intelligence Summary

- Recent relevant commits:
  - `049288a2` `chore: epic 4 retro`
  - `57591fe3` `feat(4.4): keep MVP playback scoped to active flow`
  - `3c60c495` `feat(4.3): preserve host-only playback behavior`
  - `9e867590` `feat(4.2): finalize tuplet playback ordering`
- The recent delivery pattern is seam-by-seam hardening with explicit regression proof. Story 5.1 should continue that pattern rather than broadening into generalized test infrastructure.

### Documentation Impact

- Update documentation only if the story changes the documented testing strategy or clarifies where tuplet regressions must live.
- If fixture practices or test-placement expectations change materially, update `tests/fixtures/README.md`, `docs/development-guide.md`, or `docs/project-context.md` in the same change.
- Do not treat docs work as optional if implementation changes a durable developer expectation.

### Project Structure Notes

- The repo already expects parser-local test support to live under `src-tauri/src/instructions/parse/test_support/`.
- The repo already forbids a generic repo-wide notation framework when local parser support is sufficient.
- The project also expects clippy-clean Rust, migration review for persisted behavior, and exact shared-model semantics across tuplets seams.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 5 / Story 5.1 acceptance criteria
- Source: `_bmad-output/planning-artifacts/prd.md` — maintainer journey, regression-coverage expectations, and exact tuplets product contract
- Source: `_bmad-output/planning-artifacts/architecture.md` — shared tuplet model, boundary ownership, and non-nestable MVP structure
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — supported MVP authoring and editing boundaries that regressions must protect
- Source: `docs/project-context.md` — current rules for shared playback semantics, file workflow, and active-flow state
- Source: `docs/development-guide.md` — tuplet guardrails, timing invariant phrasing, and backend-owned seam rules
- Source: `docs/architecture.md` — tuplets notation contract and persistence/migration contract
- Source: `_bmad-output/implementation-artifacts/1-6-round-trip-tuplet-data-through-save-load-and-migration.md`
- Source: `_bmad-output/implementation-artifacts/3-2-add-regression-protection-for-multi-flow-notation-safety.md`
- Source: `_bmad-output/implementation-artifacts/4-4-keep-mvp-playback-scoped-to-the-active-flow.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Build a seam-by-seam regression matrix first so new tests close real gaps instead of duplicating existing coverage.
- Extend the owning Rust test modules and parser-local helpers before creating any new harnesses or fixtures.
- Validate that every new failure message points maintainers back to the owning seam with clear scenario names and stable assertions.
