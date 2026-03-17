# Story 3.1: Parse Tuplet-Spanned Content Into Backend-Owned Notation

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want tuplet content to derive notation from the backend model,
so that the score reads from real rhythmic structure instead of frontend reconstruction.

## Scope Note

- This story is parser-first. It owns backend notation derivation for supported single-level tuplets and the automated proof that the derived notation is correct before any bracket or number drawing work begins.
- Story 3.1 stops at backend-owned notation output and parser-facing contracts. It must not absorb score-surface bracket drawing, tuplet-number placement, collision tuning, or other renderer chrome from Story 3.2.
- The story may extend shared notation data structures and parser outputs so later score-renderer work can consume tuplet context, but it must stay within the current parser and renderer pipeline shape rather than introducing a parallel notation system.
- Empty tuplet spans remain derived notation results. Do not persist rest entries, add frontend-owned reconstruction state, or build a renderer-only fallback representation.
- MVP tuplets remain structurally non-nestable. Parser work only needs to support one tuplet level and should reject or ignore nested-tuples-as-child-content scenarios outside that contract.
- Existing non-tuplet written-duration behavior is protected product behavior. This story must add targeted tuplet logic without casually rewriting simple-meter engraving decisions that current tests intentionally characterize.

## Acceptance Criteria

1. Given a score containing supported tuplets, when notation derivation runs, then the parser evaluates authoritative track content through the tuplet's local frame and the resulting notation is produced from backend-owned rhythmic data in a form the existing parser and renderer pipeline can consume without a frontend reconstruction layer.
2. Given empty spans exist inside a tuplet-governed local frame, when notation is derived, then rests are inferred from absence of projected sounding content in the governed local tuplet slots, including host-owned crossing tones and tuplet-child tones, and no frontend reconstruction layer is required to invent missing rhythmic structure.
3. Given supported tuplet derivation scenarios are exercised in automated tests, when the parser-local notation suite runs, then the expected derived notation output is asserted directly for grouped tuplet content, inferred rests, and boundary-crossing fragments and Story 3.1 can be validated without relying on score-surface rendering.
4. Given tuplets are introduced into the parse path, when existing non-tuplet passages are derived, then current non-tuplet notation behavior remains unchanged and the implementation stays within the current parser and renderer pipeline shape.
5. Given MVP tuplets are structurally non-nestable, when parser support is implemented, then the notation path only needs to cover single-level tuplets and nested tuplet parser behavior is not part of MVP scope.

## Tasks / Subtasks

- [ ] Extend backend notation derivation from authoritative tuplets instead of frontend reconstruction (AC: 1, 2, 4, 5)
  - [ ] Rework `src-tauri/src/instructions/parse/get_written_durations.rs` so the parser can enter a tuplet-local frame, derive local notation, and flatten the result back into the current pipeline order.
  - [ ] Keep the derivation authoritative to persisted host-track `Tuplet` entries and child `entries`; do not add a frontend-owned tuplet notation cache or renderer-side reconstruction layer.
  - [ ] Preserve the current event-stream mental model: rests remain derived from absence of tones, tuplets are bounded local metric containers, and ownership stays determined by where stored tones start.
- [ ] Introduce only the shared parser-side notation context needed for later renderer consumption (AC: 1, 4, 5)
  - [ ] Extend shared notation structs or parser contracts only as far as needed to carry tuplet grouping context through the existing parse pipeline.
  - [ ] Update parser consumers under `src-tauri/src/instructions/parse/` if the shared notation contract changes, while keeping Story 3.1 scoped away from renderer chrome decisions.
  - [ ] Avoid introducing a parallel "triplet parser" or isolated notation subsystem detached from the current parse flow.
- [ ] Infer empty-span notation inside tuplet local frames, including crossing-tone cases (AC: 2, 4)
  - [ ] Ensure absence of sounding content in tuplet-local slots derives rests without persisting new rest entries.
  - [ ] Cover host-owned tones that cross into a tuplet and tuplet-owned tones that interact with host context so boundary-crossing notation fragments remain parser-owned results.
  - [ ] Preserve tie identity and tone-id authority across derived fragments where the same stored tone reads through multiple contexts.
- [ ] Add parser-local regression coverage that proves Story 3.1 independently of score rendering (AC: 3, 4, 5)
  - [ ] Extend `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs` rather than creating a repo-wide notation test framework.
  - [ ] Add focused tests in `src-tauri/src/instructions/parse/get_written_durations.rs` for a fully occupied tuplet, a partially occupied tuplet with inferred rest space, and a host/tuplet boundary-crossing scenario.
  - [ ] Add a regression proving representative non-tuplet passages still normalize to the same notation output after tuplet support is introduced.
  - [ ] Keep tests parser-local and structural; visual bracket/number verification belongs to Story 3.2.
- [ ] Validate the touched parser seams with the standard Rust checks (AC: 3, 4, 5)
  - [ ] Run focused Rust tests for written-duration parsing and any directly impacted parse modules.
  - [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml` after parser changes.
  - [ ] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean is a repo rule when Rust files change.
- [ ] Review documentation impact before closing the story
  - [ ] Confirm whether the shipped parser-owned tuplet notation contract changes developer-facing documentation or guardrails.
  - [ ] Update the relevant docs and agent instructions in the same change if the durable contract changes materially.
  - [ ] If no docs change is needed, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 3 is about reading tuplets as real notation. Story 3.1 owns the backend notation derivation seam; Stories 3.2 and 3.3 own visible tuplet marking and broader engraving coherence after parser output is trustworthy.
- The planning risk this story must resolve is validation: parser output must be directly assertable in automated tests so 3.1 does not depend on later rendering stories for proof.
- Earlier tuplet work intentionally stopped at persisted timing, ownership, and sequencer interaction. `docs/data-models.md` states parser derivation, engraving, and playback projection were deferred to the later tuplet stories that own those seams.

### Technical Requirements

- Treat tuplets as local notation frames parsed from authoritative track content, then flatten the result back into host ordering with tuplet context and projected timing.
- Preserve the current parser worldview where tracks remain event streams and rests are derived from absence of tones.
- Keep written durations musically meaningful across host and tuplet contexts. Boundary splitting is a parser concern; persisted storage remains one tone per stored event unless an edit intentionally creates another tone.
- Support only single-level tuplets for MVP. Nested child tuplets are out of scope and should not drive recursive parser generalization here.
- Do not change persistence or save/load semantics in this story. 3.1 is a parse-layer change and should not require a migration unless implementation unexpectedly changes serialized state, which would need explicit follow-up.

### Architecture Compliance

- The architecture explicitly recommends local parse, then flatten: parse the host track as an event stream, parse `Tuplet` child `entries` inside a local frame, produce normal local notations, then flatten back into host ordering with tuplet context and projected timing.
- This story must preserve the current parser and renderer pipeline shape instead of introducing a special-case triplet parser or a frontend-derived notation layer.
- Parser-local shared test support belongs under `src-tauri/src/instructions/parse/test_support/`; the repo rule is to keep shared written-duration parser test harness code there and leave musical scenario tables in `get_written_durations.rs` unless a focused follow-up intentionally splits support by concern.
- Existing non-tuplet notation behavior is protected. Repository memory notes that `get_written_durations.rs` already has characterization tests locking in some simple-meter engraving deviations; Story 3.1 must avoid accidental broad rewrites while adding tuplet support.

### Library / Framework Requirements

- Backend work stays in Rust 2021 within the existing Tauri application architecture.
- Continue using `FxHashMap`-based score structures and existing parse-module patterns.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary implementation seam:
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
- Primary shared parser test-support seam:
  - `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`
  - `src-tauri/src/instructions/parse/test_support/mod.rs` only if the test-support module surface changes
- Likely parser consumers to review if notation contracts change:
  - `src-tauri/src/instructions/parse/get_beams.rs`
  - `src-tauri/src/instructions/parse/get_shunts.rs`
  - `src-tauri/src/instructions/parse/get_stem_directions.rs`
  - `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
  - `src-tauri/src/instructions/parse/get_articulations.rs`
  - `src-tauri/src/instructions/parse/get_accidentals.rs`
  - `src-tauri/src/instructions/parse/get_dots.rs`
  - `src-tauri/src/instructions/parse/get_stem_lengths.rs`
- Explicitly out of scope for 3.1:
  - score-surface bracket and number drawing under `src-tauri/src/instructions/draw/`
  - frontend score overlay or sequencer reconstruction logic

### Testing Requirements

- Add parser-local structural assertions that inspect `NotationTrack` output directly rather than inferring correctness from score rendering.
- Cover at least these scenarios:
  - a supported tuplet with all local slots occupied by derived note content
  - a supported tuplet with empty local span that must derive a rest
  - a boundary-crossing tone that yields multiple derived notation fragments across host and tuplet contexts
  - a representative non-tuplet regression proving unchanged output
- Keep validation close to the parser seam. Do not create a repo-wide notation test framework.
- Story 3.1 does not need visual regression fixtures because visible bracket/number treatment is not owned here.

### Established Code Patterns To Follow

- Extend existing seams before introducing new ones. Recent tuplet stories succeeded by threading new context through the current structures rather than building parallel flows.
- Keep backend state authoritative and derived notation backend-owned.
- Treat tuplets as bounded local metric containers, not as overlay-only annotations.
- Use close, structural regression coverage in the same module that owns the behavior.

### Previous Story Intelligence

- Story 2.5 reinforced the current project pattern: reuse and refine shared seams, keep empty tuplet space derived, and add focused regressions close to the owning module.
- Story 2.4 established the canonical ownership and projection model for host versus tuplet timing. Story 3.1 should reuse that reasoning for notation derivation instead of inventing a renderer-specific interpretation.
- The project docs already frame visible rest glyphs and notation-object selection as printed-score renderer concerns, which is why 3.1 should stop at parser output and leave visual chrome to later stories.

### Git Intelligence Summary

- Recent relevant commits:
  - `33a49140` `chore: epic 2 retro`
  - `21b576eb` `chore(2.5): mark story done`
  - `77c7fa2e` `feat(2.5): tuplet aware slice and delete`
  - `6407f59f` `fix: snap expectations.`
  - `ba93d2a7` `Complete Story 2.4 tuplet drag and snap work`
- The current implementation pattern is incremental: tighten one authoritative seam, add targeted regressions, and defer adjacent concerns to the next story rather than blending them together.

### Documentation Impact

- Review these docs during implementation and update them if the shipped parser-owned notation contract changes materially:
  - `docs/architecture.md`
  - `docs/data-models.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` only if the work establishes a durable new repo rule
- If implementation only fulfills already-documented behavior, record that no documentation update was required in the completion notes rather than skipping the review silently.

### Project Structure Notes

- Current parser code centers written-duration derivation in `get_written_durations.rs`, with many downstream parse modules consuming `Notation` or `NotationTrack`. Any shared-contract change should be deliberate and compile through the current parse pipeline.
- `src-tauri/src/instructions/parse/test_support/mod.rs` currently exposes only `get_written_durations_test_harness`; extend that local support rather than creating a second parser test harness family.
- There is no current tuplet-aware draw implementation under `src-tauri/src/instructions/draw/`; that absence is expected and should not be "fixed" in Story 3.1.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 / Story 3.1 and 3.2 boundaries
- Source: `_bmad-output/planning-artifacts/prd.md` — FR13, FR33, NFR5, NFR7, and regression-safety requirements for backend-owned notation
- Source: `_bmad-output/planning-artifacts/architecture.md` — child track shape, derivation rules, local parse-then-flatten parser architecture, and single-level tuplet scope
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — tuplet UX requirement that empty tuplet creation reads as rest-bearing score content and boundary-crossing representation stays coherent
- Source: `docs/data-models.md` — explicit deferral of parser derivation and engraving to later tuplet stories
- Source: `docs/project-context.md` — parser-local test-support guardrail, Rust-owned state, and current stack conventions
- Source: `_bmad-output/implementation-artifacts/2-5-apply-explicit-mvp-delete-and-slice-rules-for-tuplet-members-and-containers.md` — prior seam-reuse, close-regression, and documentation-impact pattern
- Source: `/memories/repo/written-durations-audit.md` — caution that existing written-duration characterization tests intentionally lock in current simple-meter behavior

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 3 story boundaries and acceptance criteria in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed PRD notation, regression-safety, and pipeline-shape requirements in `_bmad-output/planning-artifacts/prd.md`.
- Reviewed architecture guidance for child track shape, local parse-then-flatten flow, derivation rules, and MVP non-nestable scope in `_bmad-output/planning-artifacts/architecture.md`.
- Reviewed tuplet UX notes in `_bmad-output/planning-artifacts/ux-design-specification.md` to keep score-visible behavior scoped correctly between parser and renderer stories.
- Reviewed `docs/data-models.md` and `docs/project-context.md` for deferral boundaries, parser-test-support rules, and project conventions.
- Inspected current parser seams in `src-tauri/src/instructions/parse/get_written_durations.rs` and `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`.
- Reviewed repository memory about existing written-duration characterization risks and recent git history through Story 2.5.

### Completion Notes List

- Story 3.1 created with explicit parser-first validation criteria so it can be verified before score rendering stories begin.
- Epic 3.1 and 3.2 boundaries tightened to keep parser derivation separate from visible bracket/number rendering.

### File List

- `_bmad-output/planning-artifacts/epics.md`
- `_bmad-output/implementation-artifacts/3-1-parse-tuplet-spanned-content-into-backend-owned-notation.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Change Log

- 2026-03-17: Revised Epic 3.1 acceptance criteria to require parser-local validation of derived tuplet notation before renderer work.
- 2026-03-17: Revised Epic 3.2 acceptance boundary to consume derived parser output rather than recomputing tuplet structure in a parallel renderer path.
- 2026-03-17: Created Story 3.1 implementation artifact and moved Epic 3 / Story 3.1 to in-progress / ready-for-dev.
