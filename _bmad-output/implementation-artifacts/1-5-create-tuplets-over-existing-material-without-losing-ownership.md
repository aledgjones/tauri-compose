# Story 1.5: Create Tuplets Over Existing Material Without Losing Ownership

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want creating a tuplet over existing notes to preserve valid ownership and written intent,
so that the new tuplet structure does not scramble already-authored material.

## Scope Note

- This story is a reconciliation story. It creates the tuplet container over existing material and moves or retains persisted tones according to ownership rules.
- This story does not solve full parser, engraving, or playback projection. Those responsibilities continue in Story 3.1, Story 3.3, Story 4.1, Story 4.2, and Story 5.1.
- The core rule is literal carry-over: when existing tones are reconciled into the tuplet, their stored tick positions relative to the consumed span and their stored durations are copied as-is rather than rewritten into new binary approximations or pre-normalized note values.
- Rests remain derived absence of tones. Creation may begin from a selected rest region, but reconciliation never persists explicit rest entries into tuplet child content.
- MVP tuplets remain explicitly non-nestable. This story applies to host-track material only.

## Acceptance Criteria

1. Given ordinary same-track material exists in the consumed host window, when tuplet creation is confirmed through the popover, then the product records the tuplet span as a bounded rhythmic container over that host window, and any tone whose start lies inside that window is represented in the tuplet child entries while crossing tones remain single persisted tones owned by their start position.
2. Given existing tones start inside the newly created tuplet span, when reconciliation moves them under the tuplet container, then their local start offsets and durations are copied into the tuplet child entries without duration rewriting or timing approximation, and written-value interpretation remains a later derivation concern rather than a creation-time conversion step.
3. Given an existing passage such as two eighth notes starting at ticks `0` and `16` is converted into a supported triplet container at tick `0`, when the tuplet is created, then the resulting child content preserves those two eighth-note events as literal copied tones inside the tuplet, and the remaining local slot is represented by derived empty space rather than a persisted rest entry.
4. Given notes cross into or out of a tuplet-governed span, when the updated score state is inspected after creation, then crossing tones remain one persisted tone owned by where they start, tones that start inside the tuplet live in child entries even if they extend out, and tone ownership and lookup remain compatible with existing flow, track, and selection expectations.
5. Given the target span already falls inside an existing tuplet-governed region or would require nested tuplet structure, when tuplet creation or reconciliation is attempted, then MVP does not create the new tuplet, and the existing structure remains unchanged.
6. Given reconciliation encounters existing material that truly cannot exist in the resulting authoritative model, when tuplet creation is committed, then the tuplet replaces the consumed host window using the defined reconciliation rules, and valid tones are not deleted merely because later parser, rendering, or playback stories have not yet implemented their derived boundary behavior.
7. Given tuplet creation is attempted from a selected rest region or a partially empty span, when the container is created, then existing tones inside the span are copied into the tuplet using their same relative starts and durations, and all remaining local time stays represented by absence of tones rather than newly persisted rest entries.
8. Given tuplet creation is rejected for an unsupported outcome in MVP, when the interaction completes, then the product follows existing popover rejection behavior without introducing tuplet-specific error messaging, and no partial ownership rewrite is left behind.

## Tasks / Subtasks

- [x] Reconcile existing host-track material into a new tuplet container using start-position ownership rules (AC: 1, 4, 5, 6, 7, 8)
  - [x] Update the existing tuplet creation seam so creating a tuplet over a consumed span inspects host-track tones whose starts fall inside or across that span.
  - [x] Move any tone whose start lies inside the consumed window into the new `Tuplet.entries` collection while leaving crossing host-owned tones on the track.
  - [x] Keep creation atomic so unsupported outcomes do not leave behind a partially rewritten track or partially populated tuplet container.
- [x] Preserve literal stored note values during reconciliation instead of rewriting durations at creation time (AC: 2, 3, 4, 7)
  - [x] Copy reconciled child tones with their same relative start offsets inside the consumed span and with their same stored durations.
  - [x] Do not pre-convert carried tones into equivalent heard-duration replacements, approximate binary timing, or newly respelled written values during container creation.
  - [x] Treat empty local tuplet time as derived absence of tones rather than explicit persisted rest entries.
- [x] Keep selection, flow ownership, and existing creation affordances coherent (AC: 1, 4, 7, 8)
  - [x] Preserve the current authoritative `ui.flow_id` behavior and current selection contract while tuplet creation rewrites ownership beneath the active track.
  - [x] Keep rest-originated tuplet creation compatible with the existing popover preconditions and selection model.
  - [x] Ensure lookup and downstream targeting can still distinguish host-owned crossing tones from child-owned in-tuplet tones after reconciliation.
- [x] Enforce MVP scope boundaries for unsupported creation cases (AC: 5, 6, 8)
  - [x] Reject nested or overlapping creation outcomes that would require tuplets inside tuplets or other unsupported structural repair.
  - [x] Follow the current popover no-op or local rejection pattern rather than introducing bespoke tuplet failure UX in this story.
  - [x] Only remove material when it genuinely cannot exist in the resulting authoritative model; do not treat not-yet-implemented derivation work as a reason to discard otherwise valid tones.
- [x] Add focused regression coverage for reconciliation over existing material (AC: 1, 2, 3, 4, 5, 6, 7, 8)
  - [x] Add Rust coverage around tuplet creation proving tones that start inside the consumed span move into child entries with unchanged stored durations and relative offsets.
  - [x] Add cases proving host-owned crossing tones remain on the track and are not storage-split solely because they cross the new tuplet boundary.
  - [x] Add cases proving partially filled tuplets preserve derived empty space, nested or overlapping creation is rejected, and failed creation leaves original ownership unchanged.
  - [x] Run the relevant checks for touched code paths: `npm run lint`, `npm run check-types`, `npm run test:js`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- [x] Update documentation for the reconciliation contract and deferred downstream responsibilities (AC: 2, 4, 6, 7)
  - [x] Update `docs/data-models.md` with the literal-carry reconciliation rule for tones moved into tuplets and the continued rule that rests are derived absence.
  - [x] Update any relevant developer-facing docs to state that Story 1.5 intentionally stops at ownership reconciliation and defers notation, rendering, playback projection, and broader editing consequences to later stories.

## Dev Notes

### Story Foundation

- Story 1.2 established tuplet creation through the popover as the authoritative container-creation seam.
- Story 1.3 established the sequencer's bounded tuplet-region treatment, and Story 1.4 established tuplet-aware note entry into existing empty tuplets.
- Story 1.5 now fills the foundational gap for creating a tuplet over already-authored host material rather than only over empty or newly entered space.
- Your clarified intent is the controlling example for this story: if a triplet is created at tick `0` over existing eighth notes at ticks `0` and `16`, the resulting tuplet contains those same two eighth-note events plus one derived empty slot. No clever duration conversion happens during creation.
- Later stories already own the consequences of this carry-over rule: Story 2.4 handles move and resize behavior across tuplet boundaries, Story 3.1 and Story 3.3 handle derivation and rendering, Story 4.1 and Story 4.2 handle playback projection and scheduling, and Story 5.1 hardens regression coverage across those seams.

### Technical Requirements

- Extend the current tuplet creation path rather than creating a second "reconcile existing material" API.
- The creation path must inspect the consumed host window and partition tones by where they start:
  - tones starting before the tuplet remain host-owned even if they extend into it
  - tones starting inside the consumed window move into `Tuplet.entries`
  - tones starting after the consumed window remain untouched
- When moving a tone into the tuplet, copy its local start offset relative to the tuplet start and copy its stored duration unchanged.
- Do not rewrite durations to preserve heard duration at creation time. Literal tick-to-written-value mapping remains the same inside and outside tuplets; later derivation projects those tones through the tuplet frame.
- Creation from a selected rest region is valid because rests are derived from absence of tones. There is no explicit rest payload to copy.
- Keep the operation atomic and backend-owned. The frontend may trigger creation and refresh selection state, but Rust must remain the source of truth for ownership reconciliation.
- A tone that is valid under the foundational ownership model must survive reconciliation even if later stories are still needed to render or play it perfectly.
- Only delete material if it genuinely cannot exist in the resulting authoritative model after applying the story's explicit rules. Deletion is a last-resort structural outcome, not a shortcut around deferred future work.
- Unsupported creation outcomes should follow existing rejection behavior with no special tuplet-specific messaging in MVP.

### Architecture Compliance

- The architecture's ownership rule remains non-negotiable: one tone is one stored event, and ownership is determined by where the tone starts.
- Story 1.5 intentionally leans on the architecture's literal mapping rule: a written eighth remains a written eighth on both sides of a tuplet boundary, and the tuplet changes projection through musical time rather than forcing creation-time duration conversion.
- Crossing a tuplet boundary still does not require a storage split. This story must not rewrite one tone into multiple persisted tones simply because a new tuplet edge now intersects it.
- Host-level directives remain outside MVP tuplet child content. Reconciliation should not attempt to move or recreate directives inside tuplets.
- MVP tuplets remain single-level. Any outcome requiring nested child tuplets is rejected rather than repaired.

### Library / Framework Requirements

- Frontend work should remain within the existing React + TypeScript + Tauri `invoke()` action flow.
- Backend work should follow the current Rust command pattern with `#[tauri::command(rename_all = "snake_case")]`, `AppContext` mutation behind `Mutex`, and affected-track emission through the existing renderer update path.
- Continue using `FxHashMap` in Rust-owned storage code.
- No new external library is expected for this story.

### File Structure Requirements

- Expected frontend touchpoints:
  - `src/services/actions.ts`
  - `src/services/types.ts`
  - any tuplet creation popover or creation-trigger surface already used by Story 1.2
  - any sequencer refresh or selection helper that reflects post-creation ownership changes
- Expected backend touchpoints:
  - `src-tauri/src/score/entries/tuplet.rs`
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/ui/sequencer.rs` if minimal selection or refresh state must reflect reconciled child ownership
  - `src-tauri/src/migrations/next.rs` only if implementation changes persisted shape in a way that requires pending migration work ahead of Story 1.6
- Keep the work focused. Do not pull full child-targeting, pitch editing, add/remove semantics, move/resize behavior, notation derivation, or playback projection forward from later stories.

### Testing Requirements

- Verify tones starting inside the consumed span move into `Tuplet.entries` with unchanged stored durations and correct local offsets.
- Verify tones starting before the tuplet and extending into it remain host-owned single persisted events.
- Verify a creation case equivalent to "ticks `0` and `16` become two literal child eighths plus one empty local slot" is covered explicitly.
- Verify empty local tuplet time remains derived absence of tones rather than explicit persisted rest entries.
- Verify nested or overlapping creation attempts are rejected without partial mutation.
- Verify rest-originated creation keeps the current creation affordance while still copying only actual tones.
- Run the relevant frontend and Rust checks after implementation.

### Previous Story Intelligence

- Story 1.2 established the tuplet container creation path and immediate selection handoff.
- Story 1.3 established the sequencer region geometry and visual boundedness needed to present the reconciled tuplet after creation.
- Story 1.3A simplified MVP child content to single-level `TupletEntries`, removing nested-child complexity from this reconciliation path.
- Story 1.4 established that child note insertion keeps the track-level mutation surface authoritative and treats rests as absence of tones. Story 1.5 should preserve those same invariants while backfilling pre-existing content into a newly created container.

### Git Intelligence Summary

- Recent relevant commits:
  - `37a3ee54 feat(1.4): add tuplet note entry in sequencer`
  - `1ede423a feat(1.3a): tuplet entries structures non-recursive`
  - `356b5203 feat: course correct`
  - `38c8bede fix: tweak tuple background colour`
  - `0a06d047 feat(1.3): description`
- Recent work has been progressive and seam-focused. Story 1.5 should follow that pattern by tightening the creation seam rather than broadening implementation into render or playback layers prematurely.

### Documentation Impact

- This story makes the reconciliation rule explicit: creating a tuplet over existing material carries valid tones into the tuplet literally, preserves crossing-tone ownership by start position, and leaves rests implicit.
- The docs should also state that this story prepares later parser, rendering, and playback stories rather than replacing them.

### Project Structure Notes

- The existing tuplet creation path already governs the host span and should remain the single place where creation over existing material is reconciled.
- The authoritative distinction after creation is still `track_id -> optional tuplet_id -> tone_id`; local offsets are lookup and storage coordinates, not tone identity.
- Selection summaries and detailed member targeting remain primarily Story 2.1 work. Story 1.5 only needs to preserve coherence after reconciliation, not deliver the full editing model.
- Migration analysis is required if the persisted container shape changes, but unreleased migration work must stay in `src-tauri/src/migrations/next.rs` and release scripts remain human-only.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 / Story 1.5; Epic 2 / Story 2.4; Epic 3 / Stories 3.1 and 3.3; Epic 4 / Stories 4.1 and 4.2; Epic 5 / Story 5.1
- Source: `_bmad-output/planning-artifacts/architecture.md` — agreed model invariants; ownership decision table; literal tick-to-written-value mapping rule
- Source: `_bmad-output/planning-artifacts/prd.md` — editing semantics; boundary-crossing tone rule; persistence and migration stakes
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — tuplet creation handoff; empty-slot treatment; existing rejection behavior
- Source: `docs/project-context.md` — frontend/backend mutation ownership; `ui.flow_id` guardrail; Rust and TypeScript implementation conventions
- Source: `_bmad-output/implementation-artifacts/1-2-create-tuplet-containers-through-a-tuplet-popover.md`
- Source: `_bmad-output/implementation-artifacts/1-3-reveal-and-select-the-created-tuplet-region-in-the-sequencer.md`
- Source: `_bmad-output/implementation-artifacts/1-3a-simplify-tuplet-child-content-for-non-nestable-mvp.md`
- Source: `_bmad-output/implementation-artifacts/1-4-enter-tuplet-notes-in-the-sequencer.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Extend `Track::create_tuplet(...)` in `src-tauri/src/score/entries/tuplet.rs` so the existing backend creation seam reconciles host-owned tones into child entries instead of creating an always-empty container.
- Preserve ownership and timing invariants by moving only tones whose starts fall inside the consumed host span, converting their ticks to tuplet-local offsets, and leaving crossing host-owned tones untouched.
- Prove the reconciliation contract with focused Rust unit coverage, then update the relevant tuplet documentation and run the required repo validations.

### Debug Log References

- Added focused Rust coverage in `src-tauri/src/score/entries/tuplet.rs` for literal carry-over, crossing host-tone preservation, and rejected overlapping creation state preservation.
- Validated with `npm run lint`, `npm run check-types`, `npm run test:js`, `cargo test --manifest-path src-tauri/Cargo.toml create_tuplet_`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Completion Notes List

- Updated `Track::create_tuplet(...)` to reconcile host-owned tones whose starts fall inside the consumed span into `Tuplet.entries`, rewriting only their local tick relative to the tuplet start and preserving stored durations unchanged.
- Kept crossing host-owned tones on the track, preserved current popover rejection behavior for overlapping tuplets, and left rest handling implicit as absence of tones.
- Added focused Rust regression coverage for literal carry-over and rejection preservation, then updated developer-facing documentation for the deferred downstream tuplet responsibilities.

### File List

- `_bmad-output/implementation-artifacts/1-5-create-tuplets-over-existing-material-without-losing-ownership.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src-tauri/src/score/entries/tuplet.rs`

## Change Log

- 2026-03-15: Reconciled existing host-owned tones into new tuplet containers at the backend creation seam, added focused Rust coverage for ownership preservation, and documented the literal carry-over contract plus deferred downstream tuplet work.
- 2026-03-15: Senior developer review approved the story with no story-scoped findings after confirming the flagged editing and targeting concerns are intentionally deferred to later Epic 2 stories.

## Senior Developer Review (AI)

### Reviewer

GPT-5.4

### Date

2026-03-15

### Outcome

Approve

### Notes

- Re-review scoped the implementation against Story 1.5's stated reconciliation-only boundary rather than later tuplet editing goals.
- No story-scoped findings remain. The implemented change in `Track::create_tuplet(...)` satisfies the ownership-reconciliation contract for host-owned tones that start inside the consumed span while preserving crossing-tone ownership and current rejection behavior for overlapping MVP tuplets.
- The previously raised concerns around member targeting, selection summaries, and richer child-tone editing belong to planned Epic 2 follow-up work, primarily Story 2.1, Story 2.4, and Story 2.5, and are explicitly outside Story 1.5's shipped scope.

### Retrospective Note

- Initial review feedback overreached the story boundary by treating planned Epic 2 editing and targeting follow-up work as Story 1.5 defects. In retrospective, tighten the review process so findings are first checked against the story's explicit scope note, stated deferrals, and adjacent epic ownership before classifying deferred work as missing or incomplete.
