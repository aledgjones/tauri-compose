# Story 3.1: Parse Tuplet-Spanned Content Into Backend-Owned Notation

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want tuplet content to derive notation from the backend model,
so that the score reads from real rhythmic structure instead of frontend reconstruction.

## Scope Note

- This story is parser-first. It owns backend notation derivation for supported single-level tuplets and the automated proof that the derived notation is correct before any bracket or number drawing work begins.
- Story 3.1 stops at backend-owned notation output and parser-facing contracts. It must not absorb score-surface bracket drawing, tuplet-number placement, collision tuning, or other renderer chrome from Story 3.2.
- The story may extend shared notation data structures and parser outputs so later score-renderer work can consume tuplet context, but it must stay within the current parser and renderer pipeline shape rather than introducing a parallel notation system.
- The preferred parser contract is a light one: each tuplet remains an isolated local `NotationTrack` in its own right, wrapped only by the minimal host-span anchoring metadata needed to place that local unit back into the main flow without forcing a host-only flattening model.
- Story 3.1 should emit enough stable local notation structure that later renderer and spacing stories can add local spacing work without redesigning the parser contract, but 3.1 itself does not own spacing calculation yet.
- The parser contract must preserve three distinct concepts for tuplet-derived material: semantic address in the local owning context, written duration used for notation decisions, and projected host-space occupancy used by later spacing and draw consumers. Story 3.1 must not collapse those three meanings into one host-tick-plus-duration pair.
- Empty tuplet spans remain derived notation results. Do not persist rest entries, add frontend-owned reconstruction state, or build a renderer-only fallback representation.
- A completely empty triplet is a special readability case: it should read as one base-note rest per local triplet slot. For an eighth-note triplet, an empty container therefore reads as three eighth rests in the local frame rather than as host-projected binary rest values.
- Outside that special empty-triplet case, tuplet-local rest and note spelling should follow the existing written-duration split logic already used for the equivalent local metric frame.
- MVP tuplets remain structurally non-nestable. Parser work only needs to support one tuplet level and should reject or ignore nested-tuples-as-child-content scenarios outside that contract.
- Existing non-tuplet written-duration behavior is protected product behavior. This story must add targeted tuplet logic without casually rewriting simple-meter engraving decisions that current tests intentionally characterize.

## Acceptance Criteria

1. Given a score containing supported tuplets, when notation derivation runs, then the parser evaluates authoritative track content through the tuplet's local frame and produces backend-owned notation from that local rhythmic structure without any frontend reconstruction layer.
2. Given notation is derived inside a tuplet-governed local frame, when the parser emits tuplet-aware output, then the contract preserves local semantic address, written duration, and projected host-space occupancy as distinct concepts so downstream renderer and spacing consumers do not have to recover them from one overloaded host-tick-plus-duration pair.
3. Given a tuplet is parsed as an isolated rhythmic unit, when the parser emits tuplet-aware output, then it keeps the tuplet as an anchored local `NotationTrack` while also carrying the host-span anchoring metadata needed to place that unit back into the main flow.
4. Given empty spans exist inside a tuplet-governed local frame, when notation is derived, then rests are inferred from absence of projected sounding content in the governed local tuplet slots, including host-owned crossing tones and tuplet-child tones, and a completely empty triplet normalizes to one base-note rest per local slot while other local rest spelling follows the existing local split-unit behavior.
5. Given supported tuplet derivation scenarios are exercised in automated tests, when the parser-local notation suite runs, then the expected derived notation output is asserted directly for grouped tuplet content, empty-triplet rest spelling, partially-filled local-frame spelling, and boundary-crossing fragments and Story 3.1 can be validated without relying on score-surface rendering.
6. Given tuplets are introduced into the parse path, when existing non-tuplet passages are derived, then current non-tuplet notation behavior remains unchanged and the implementation stays within the current parser and renderer pipeline shape.
7. Given MVP tuplets are structurally non-nestable, when parser support is implemented, then the notation path only needs to cover single-level tuplets and nested tuplet parser behavior is not part of MVP scope.

## Tasks / Subtasks

- [x] Amend the shared parser-side notation contract so tuplet-derived output preserves semantic address, written duration, and projected host occupancy separately (AC: 1, 2, 3, 6, 7)
  - [x] Rework `src-tauri/src/instructions/parse/get_written_durations.rs` so the parser enters a tuplet-local frame, derives local notation there, and emits an anchored local `NotationTrack` plus host-span anchoring metadata.
  - [x] Ensure that anchored-track shape is stable enough for later stories to add spacing and width comparison without redesigning the parser contract.
  - [x] Do not treat flattening as permission to erase tuplet-local semantics into host-only tick arithmetic; downstream spacing, tie, and draw consumers should not need to infer tuplet behavior from `tick + duration` alone.
  - [x] Reuse the Story 2.4 address-lattice reasoning: tuplets replace the covered host span semantically, and projection back to host space is a separate concern from local notation identity.
- [x] Normalize writable notation inside the tuplet-local frame before renderer stories consume it (AC: 1, 4, 5, 7)
  - [x] Treat the tuplet as a local metric frame for written-duration splitting and reuse the existing split-unit code against that local frame rather than inventing new tuplet-only splitting rules.
  - [x] For a completely empty triplet, emit one base-note rest per local slot as a special readability rule.
  - [x] Outside that special empty-triplet case, let partially filled tuplets follow the existing local split-unit behavior for the equivalent local metric frame.
  - [x] Preserve tie identity and tone-id authority across derived fragments where the same stored tone reads through multiple local and host contexts.
- [x] Add parser-local proof that the amended contract holds for tuplet and non-tuplet material (AC: 2, 3, 5, 6, 7)
  - [x] Extend `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs` rather than creating a repo-wide notation test framework.
  - [x] Add focused tests covering empty tuplets, partially filled tuplets, boundary-crossing tones, the anchored local-`NotationTrack` contract, and the distinction between local semantic position, written duration, and host occupancy.
  - [x] Add regression cases for the currently characterized fragile non-tuplet written-duration paths rather than only straight happy-path material.
- [x] Extend backend notation derivation from authoritative tuplets instead of frontend reconstruction (AC: 1, 2, 5, 6)
  - [x] Rework `src-tauri/src/instructions/parse/get_written_durations.rs` so the parser can enter a tuplet-local frame, derive local notation there, and project the result into the later renderer-facing contract without losing local semantics.
  - [x] Keep the derivation authoritative to persisted host-track `Tuplet` entries and child `entries`; do not add a frontend-owned tuplet notation cache or renderer-side reconstruction layer.
  - [x] Preserve the current event-stream mental model: rests remain derived from absence of tones, tuplets are bounded local metric containers, and ownership stays determined by where stored tones start.
- [x] Introduce only the shared parser-side notation context needed for later renderer consumption (AC: 1, 2, 5, 6)
  - [x] Extend shared notation structs or parser contracts only as far as needed to carry tuplet grouping context, local semantic position, and host projection metadata through the existing parse pipeline.
  - [x] Update parser consumers under `src-tauri/src/instructions/parse/` if the shared notation contract changes, while keeping Story 3.1 scoped away from renderer chrome decisions.
  - [x] Avoid introducing a parallel "triplet parser" or isolated notation subsystem detached from the current parse flow.
- [x] Infer empty-span notation inside tuplet local frames, including crossing-tone cases (AC: 3, 4)
  - [x] Ensure absence of sounding content in tuplet-local slots derives rests without persisting new rest entries.
  - [x] Cover host-owned tones that cross into a tuplet and tuplet-owned tones that interact with host context so boundary-crossing notation fragments remain parser-owned results.
  - [x] Preserve tie identity and tone-id authority across derived fragments where the same stored tone reads through multiple contexts.
- [x] Validate the touched parser seams with the standard Rust checks (AC: 4, 5, 6)
  - [x] Run focused Rust tests for written-duration parsing and any directly impacted parse modules.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` after parser changes.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean is a repo rule when Rust files change.
- [x] Review documentation impact before closing the story
  - [x] Confirm whether the shipped parser-owned tuplet notation contract changes developer-facing documentation or guardrails.
  - [x] Update the relevant docs and agent instructions in the same change if the durable contract changes materially.
  - [x] If no docs change is needed, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 3 is about reading tuplets as real notation. Story 3.1 owns the backend notation derivation seam; Stories 3.2 and 3.3 own visible tuplet marking and broader engraving coherence after parser output is trustworthy.
- The planning risk this story must resolve is validation: parser output must be directly assertable in automated tests so 3.1 does not depend on later rendering stories for proof.
- Earlier tuplet work intentionally stopped at persisted timing, ownership, and sequencer interaction. `docs/data-models.md` states parser derivation, engraving, and playback projection were deferred to the later tuplet stories that own those seams.

### Carry-Forward Invariants

- For Epic 3, refer to the inherited Story 2.4 and Story 2.5 boundary/projection rules collectively as the `tuplet carry-forward invariants`.
- Story 3.1 inherits the established tuplet invariants from Story 2.4 and Story 2.5 rather than redefining them from scratch. Treat those stories as normative context for host-versus-tuplet ownership, boundary crossing, continuous tone identity, and local-versus-host projection.
- In particular, notation should carry forward the same core principle already proven in sequencer work: tuplets are isolated local units whose durations are distributed through projection, while crossing tones remain one stored tone whose derived fragments are split only at the boundary.
- Future tuplet stories in Epic 3 should refer back to the tuplet carry-forward invariants and this Story 3.1 contract instead of restating the full boundary-crossing model each time.

### Technical Requirements

- Treat tuplets as local notation frames parsed from authoritative track content. The simplest host-side behavior should remain as close as possible to the current track logic: the covered host span behaves like an implicit placeholder while the tuplet itself is derived as an isolated unit.
- The preferred contract is light: a tuplet owns a local `NotationTrack` plus the minimal host-span metadata needed to place that local unit back into the main flow.
- Flattening back into host ordering therefore means anchoring the isolated local `NotationTrack` in the main flow, not erasing that unit into host-only tick arithmetic.
- The local tuplet frame defines the writable-splitting grammar for the governed span. In MVP an eighth-note triplet therefore behaves like an embedded three-eighth-note unit for local note/rest spelling, even though it consumes two eighths of host time.
- A completely empty triplet is a special readability case and must derive one base-note rest per local slot. Outside that special case, local note/rest spelling should reuse the existing split-unit behavior for the equivalent local metric frame.
- Boundary-crossing tones must derive piecewise notation fragments against the local frame while preserving one stored-tone authority and enough projection metadata for later spacing, tie, and playback consumers.
- Preserve the current parser worldview where tracks remain event streams and rests are derived from absence of tones.
- Keep written durations musically meaningful across host and tuplet contexts. Boundary splitting is a parser concern; persisted storage remains one tone per stored event unless an edit intentionally creates another tone.
- Support only single-level tuplets for MVP. Nested child tuplets are out of scope and should not drive recursive parser generalization here.
- Do not change persistence or save/load semantics in this story. 3.1 is a parse-layer change and should not require a migration unless implementation unexpectedly changes serialized state, which would need explicit follow-up.

### Notation Contract

Story 3.1 emits tuplet-derived notation as isolated local units that can later be rendered back into the main flow. The goal of this contract is to preserve local tuplet semantics without forcing later stories to recover meaning from host-only tick arithmetic.

A supported tuplet should therefore be represented as a thin anchored region with two responsibilities:

- It owns the local `NotationTrack` for the tuplet frame.
- It carries the host-span anchoring metadata needed to place that local unit back into the main score flow.

Conceptually, the contract should preserve these meanings separately:

- `semantic address`
  The logical position of a derived notation in its owning context. For host material this is a host address. For tuplet-owned material this is the local tick inside the anchored tuplet `NotationTrack`.

- `written duration`
  The duration used for notation spelling, grouping, dots, rests, and other written-duration decisions inside the local frame.

- `host occupancy`
  The start/end span that the anchored local `NotationTrack` occupies in the surrounding host flow, plus the projection rule used to map local ticks back into that host span.

A conceptual shape for the parser output is:

```rust
type SemanticAddress = {
  owner_id: Option<UniqueId>,
  tick: Tick,
};

type HostSpan = {
  start_tick: Tick,
  end_tick: Tick,
};

type AnchoredNotationTrack = {
  owner_id: UniqueId,
  host_span: HostSpan,
  notation: NotationTrack,
};
```

This shape is illustrative, not prescriptive. Exact Rust type names may differ, but the contract must preserve the same distinctions. The important point is that existing `NotationTrack` semantics and helpers remain the local unit of notation, while the anchor carries only the extra host projection context. `owner_id` is intentionally generic here: for Story 3.1 it will be the tuplet container id, but the contract is written so the same anchored-track pattern can be reused later for other bounded notation regions.

### Derivation Rules

- The host track continues through its existing processing with the governed tuplet span treated as an implicit placeholder rather than as ordinary host notation content.
- The tuplet is derived as its own isolated local `NotationTrack`.
- Tuplet-local written splitting should reuse the existing split-unit behavior against the equivalent local frame rather than inventing new tuplet-only splitting rules.
- A completely empty triplet is a special readability case and should emit one base-note rest per local slot.
- Outside that special empty-triplet case, partially filled tuplets should follow the existing local split-unit behavior for the equivalent local metric frame.
- Boundary-crossing tones remain one stored tone. Derivation may emit multiple local notations or host-side companions at the tuplet boundary, but continuity still follows stored tone identity.
- Adjacent tuplets remain separate isolated units even if their host spans touch exactly.

### Contract Boundaries

- Story 3.1 owns the isolated anchored local `NotationTrack` and the host-span anchoring metadata.
- Story 3.1 does not yet own spacing calculation.
- Later renderer and spacing stories may run spacing passes against the stable local `NotationTrack` exposed by this contract.
- The Story 3.1 contract must therefore be stable enough that later spacing work can be added without redesigning parser output.

### Architecture Compliance

- Story 2.4 already established the stronger canonical model: tuplets replace their covered host span semantically and projection is resolved through a local address space plus a host mapping, not through host-tick exception math. Story 3.1 must carry that same distinction into the parser contract.
- The architecture explicitly recommends local parse, then flatten: parse the host track as an event stream, parse `Tuplet` child `entries` inside a local frame, produce normal local notations, then anchor that isolated local unit back into host ordering with tuplet context and projected timing.
- This story must preserve the current parser and renderer pipeline shape instead of introducing a special-case triplet parser or a frontend-derived notation layer.
- Spacing remains an explicit later-story concern. Story 3.1 should preserve the contract needed for later local spacing work, but it should not absorb spacing calculation or width resolution prematurely.
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
  - a completely empty triplet whose governed local frame derives one base-note rest in every local slot
  - a partially filled tuplet whose local note/rest spelling follows the existing split-unit behavior for the equivalent local metric frame
  - a boundary-crossing tone that yields multiple derived notation fragments across host and tuplet contexts
  - a contract-level case proving the anchored local `NotationTrack` keeps local semantic position, written duration, and host occupancy distinguishable in the parser output
  - representative fragile non-tuplet regressions proving unchanged output
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
- Story 2.4 also established that duration rendering across a tuplet boundary is address-span projection, not one-width-times-duration math. Story 3.1 therefore needs a parser contract that keeps local written value and host occupancy distinct enough for later stories to preserve that rule.
- Story 2.4 and Story 2.5 already contain the detailed refinement around tones that cross tuplet boundaries. Story 3.1 should reference and reuse that context rather than re-deriving a second boundary model in parser terms.
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
- Reviewed architecture guidance for child track shape, local parse-then-project flow, derivation rules, and MVP non-nestable scope in `_bmad-output/planning-artifacts/architecture.md`.
- Reviewed tuplet UX notes in `_bmad-output/planning-artifacts/ux-design-specification.md` to keep score-visible behavior scoped correctly between parser and renderer stories.
- Reviewed `docs/data-models.md`, `docs/project-context.md`, and repository memory around Story 2.4/2.5 to align the notation contract with the existing sequencer address/projection model.
- Reset stale completion state after the earlier implementation was reverted so the story reflects the current ready-for-dev contract rather than historical code that is no longer present.
- Implemented anchored local tuplet notation derivation plus host placeholder stripping in `src-tauri/src/instructions/parse/get_written_durations.rs`.
- Extended that parser contract so tuplet-owned tones that cross out of the tuplet emit host-track continuation notation instead of stopping at the local anchored track boundary.
- Extended `src-tauri/src/instructions/parse/get_tone_offsets.rs` so tuplet child tones receive parser offset info under the containing host clef.
- Extended parser-local harness coverage in `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs` and added Story 3.1 contract tests in `src-tauri/src/instructions/parse/get_written_durations.rs`.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml --lib`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- Re-ran focused validation with `cargo test --manifest-path src-tauri/Cargo.toml get_written_durations_ --lib` and `cargo test --manifest-path src-tauri/Cargo.toml score::entries::tone::tests --lib` after fixing the missing tuplet-owned-to-host projection path.
- Fixed the post-review regressions by restoring explicit barline-driven written-duration splits, projecting carried tones into adjacent tuplets on the continuous timeline, and honoring persisted tuplet grouping metadata in local derivation.
- Re-ran focused validation with `cargo test --manifest-path src-tauri/Cargo.toml get_written_durations_splits_tones_at_explicit_barlines --lib`, `cargo test --manifest-path src-tauri/Cargo.toml get_written_durations_projects_crossing_tone_into_adjacent_tuplet_tracks --lib`, `cargo test --manifest-path src-tauri/Cargo.toml get_written_durations_honors_tuplet_groupings_when_deriving_local_notation --lib`, `cargo test --manifest-path src-tauri/Cargo.toml get_written_durations_ --lib`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Completion Notes List

- Added an anchored local-tuples parser contract: host notation now strips governed tuplet placeholders while each tuplet exposes an isolated local `NotationTrack` plus host-span anchoring metadata.
- Derived local tuplet notation from authoritative child entries plus host-owned crossing tones, and normalized completely empty triplets to one base-note rest per local slot.
- Added the missing symmetric boundary projection so tuplet-owned tones that extend past the tuplet end now render a host-track continuation fragment as part of the same parser-owned result.
- Added parser-local proof for full triplet occupancy, empty-triplet rests, host boundary crossings, tuplet-owned boundary crossings back into host space, and retained the existing non-tuplet written-duration regression corpus.
- Updated developer-facing docs and agent instructions to make the tuplet timing invariant explicit: host geometry stays compressed to the denominator span while crossing duration remains literal on the expanded timeline.
- Closed the review defects by restoring explicit barline splits, preserving carried tones across adjacent tuplets, and deriving local tuplet notation with persisted grouping boundaries.

### File List

- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/3-1-parse-tuplet-spanned-content-into-backend-owned-notation.md`
- `.github/copilot-instructions.md`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src-tauri/src/instructions/parse/get_shunts.rs`
- `src-tauri/src/instructions/parse/get_stem_directions.rs`
- `src-tauri/src/instructions/parse/get_tone_offsets.rs`
- `src-tauri/src/instructions/parse/get_written_durations.rs`
- `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`

### Change Log

- 2026-03-17: Revised Epic 3.1 acceptance criteria to require parser-local validation of derived tuplet notation before renderer work.
- 2026-03-17: Revised Epic 3.2 acceptance boundary to consume derived parser output rather than recomputing tuplet structure in a parallel renderer path.
- 2026-03-17: Created Story 3.1 implementation artifact and moved Epic 3 / Story 3.1 to in-progress / ready-for-dev.
- 2026-03-17: Clarified that Story 3.1 must preserve semantic address, written duration, and host occupancy as distinct concepts, derive base-note-value rests inside the local tuplet frame, and carry Story 2.4's projection rules into the notation contract.
- 2026-03-17: Reset stale completion records after the earlier code implementation was reverted and aligned the story with the refined reimplementation plan.
- 2026-03-17: Implemented anchored local tuplet notation derivation, empty-triplet local-slot rest normalization, tuplet child tone offset support, and parser-local contract coverage for Story 3.1.
- 2026-03-17: Fixed the missing tuplet-owned-to-host boundary projection so tones that start inside tuplets and extend out now emit host-track continuation notation, and documented the shared tuplet timing invariant in the developer guide and agent instructions.
- 2026-03-17: Fixed review findings by restoring explicit barline split boundaries, preserving adjacent-tuplet carried tone projection, and honoring persisted tuplet groupings in local derivation; reran focused and full Rust validation.
