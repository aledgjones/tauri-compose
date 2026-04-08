# Story 2.3: Add and Remove Notes Within Tuplet Slots

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want to add notes into empty tuplet space and remove notes from occupied tuplet space,
so that I can shape real triplet patterns inside one rhythmic container.

## Scope Note

- This story is about slot-level note entry and member removal inside an existing tuplet. It is not a general tuplet-editing story.
- Supported add behavior is entering a note into an available tuplet-local slot through the existing sequencer-first note-entry path.
- Supported remove behavior is removing a tuplet-owned note so the vacated span becomes empty tuplet space represented by absence of tones, not a persisted rest entry.
- Repositioning, resizing, drag-to-re-pitch, or any other gesture that changes a tone's start tick or duration remains Story 2.4 scope.
- Container-level delete and slice semantics remain Story 2.5 scope, except that removing a single tuplet member from an occupied slot is in scope here.
- Empty tuplet space is visual interaction context, not a persisted entry and not a selectable synthetic rest.
- MVP tuplets remain single-level only. This story must not introduce nested tuplets, nested repair, or a frontend-owned parallel tuplet editing state.

## Acceptance Criteria

1. Given a valid tuplet container with empty local space, when the composer enters a note into an available tuplet position, then the note is stored through tuplet-aware exact timing on the authoritative track model and the tuplet remains structurally valid.
2. Given a tuplet-owned note is removed by the composer, when the removal is applied, then the sounding content is removed from that local span and the remaining empty span is represented by absence of tones rather than an explicit persisted rest entry.
3. Given a tuplet-owned note has been removed from the sequencer, when the region refreshes, then the vacated local slot remains visibly targetable inside the tuplet region and the tuplet does not visually collapse into ordinary host-track space.
4. Given a partially filled tuplet, when the score state is re-evaluated after note removal, then the container remains intact and the empty region is available for later note entry.
5. Given an add or remove action would require nested tuplet insertion or repair, when the action is evaluated, then MVP rejects or defers that outcome and slot operations remain limited to single-level tuplet content.
6. Given tuplet slot add or remove behavior changes documented editing semantics or developer guardrails, when the story is completed, then the relevant docs and agent instructions are updated in the same change, or the completion notes explicitly record why no documentation update was required.

## Tasks / Subtasks

- [x] Reuse the existing shared note-entry and note-removal seams for tuplet slot operations (AC: 1, 2, 4, 5)
  - [x] Extend `src/windows/main/components/sequencer-track.tsx` so pen-tool note entry can target empty tuplet-local slots through the existing `actions.score.tone.create(...)` path with the correct `tuplet_id` context.
  - [x] Keep tuplet-local note creation on the existing shared Rust command path in `src-tauri/src/score/entries/tone.rs` via `score_entries_tone_create`, `Track::create_tone(...)`, and `resolve_tuplet_pitch_tick(...)` rather than adding a tuplet-only creation command.
  - [x] Extend erase-tool note removal in `src/windows/main/components/sequencer-track.tsx` to remove a tuplet-owned tone through `actions.score.tone.remove(...)` with the known `tuplet_id` context.
  - [x] Reuse `Track::remove_tone(tone_id, tuplet_id)` and `TupletEntries::remove_entry(...)` in `src-tauri/src/score/entries/tone.rs` and `src-tauri/src/score/entries/tuplet.rs` instead of inventing a second delete path.
- [x] Preserve exact timing, ownership, and single-level tuplet invariants during slot operations (AC: 1, 2, 4, 5)
  - [x] Ensure any note created in a tuplet slot uses tuplet-local exact timing projection instead of approximate host-grid rounding.
  - [x] Preserve the rule that one tone is one stored event, owned by where it starts; do not split or migrate surrounding tones as part of this story.
  - [x] Reject or defer outcomes that would require nested tuplets or nested repair behavior.
  - [x] Preserve `ui.flow_id` as the only authoritative active-flow slot; do not introduce frontend-owned active-tuplet flow state.
- [x] Keep empty-slot visuals and interaction feedback stable after add/remove operations (AC: 2, 3, 4)
  - [x] Preserve the existing tuplet-region rendering in `src/windows/main/components/sequencer-track-tuplet.tsx`, including compressed slot markers, centered annotation, and visible empty-slot structure after child-note deletion.
  - [x] Ensure sequencer targeting continues to treat empty tuplet space as interaction context rather than a selected synthetic note or rest.
  - [x] Keep child-note selection behavior aligned with ordinary tone selection when a newly entered tuplet note becomes the active target.
- [x] Add focused regressions for tuplet slot entry and member removal (AC: 1, 2, 3, 4, 5)
  - [x] Extend `src/windows/main/components/sequencer-track.test.tsx` to cover adding a note into an empty tuplet slot, removing a tuplet-owned note, and preserving visible slot markers after removal.
  - [x] Extend `src/services/sequencer-utils.test.ts` only if click-target resolution changes for empty tuplet slots or member notes.
  - [x] Extend backend tests in `src-tauri/src/score/entries/tone.rs` and `src-tauri/src/score/entries/tuplet.rs` to cover tuplet-aware create/remove flows, exact timing projection, and stable by-tick/by-id indexing after member removal.
  - [x] Add or extend action-layer tests in `src/services/actions.test.ts` only if the shared FE-to-BE create/remove contract changes.
- [x] Validate the touched seams with the standard repo checks
  - [x] Run the relevant frontend tests covering sequencer tuplet note entry and removal.
  - [x] Run the relevant Rust tests covering tuplet-aware tone creation and deletion.
  - [x] Run `npm run lint` and `npm run check-types` after implementation.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` if Rust entry-management seams change.
- [x] Review documentation impact before closing the story (AC: 6)
  - [x] Confirm whether the shipped behavior changes any documented tuplet editing rules, empty-slot interaction guidance, or developer guardrails.
  - [x] Update the relevant docs and agent instructions in the same change if the contract changed materially.
  - [x] If no docs change is needed, record that explicit decision in the implementation completion notes.

## Dev Notes

### Story Foundation

- Epic 2 is about editing tuplets without breaking musical intent. Story 2.3 is the first member-lifecycle story after selection and pitch-parity groundwork.
- Story 2.1 established reliable targeting of tuplet containers, child notes, and empty tuplet regions with single-level MVP semantics.
- Story 2.2 established that once a tuplet-owned tone is targeted, mutation should flow through shared tone seams rather than a separate tuplet editing model.
- Story 2.3 should build directly on those seams: entering and removing child notes should feel like ordinary sequencer work with tuplet membership handled as nested storage context.

### Technical Requirements

- Reuse the existing frontend action wrappers in `src/services/actions.ts`:
  - `actions.score.tone.create(track_id, tick, duration, pitch_int, accidental, velocity, articulations, tuplet_id?)`
  - `actions.score.tone.remove(track_id, tuplet_id | null, tone_id)`
- Reuse the existing backend tone commands in `src-tauri/src/score/entries/tone.rs`:
  - `score_entries_tone_create(...)`
  - `score_entries_tone_duration_remove(...)`
  - `Track::create_tone(...)`
  - `Track::remove_tone(...)`
  - `resolve_tuplet_pitch_tick(...)`
- Reuse the existing child-entry indexing in `src-tauri/src/score/entries/tuplet.rs`:
  - `TupletEntries::insert_entry(...)`
  - `TupletEntries::remove_entry(...)`
- Preserve the exact timing model. Tuplet note entry must project through tuplet-local timing; it must not approximate to ordinary host-grid ticks.
- Preserve the current rule that rests inside tuplets are derived from absence of tones. Removing a note must leave empty space, not create a persisted rest entry.
- Do not introduce a tuplet-specific create/remove API, temporary frontend-owned tuplet edit state, or container-level delete behavior in this story.

### Architecture Compliance

- Each tone remains exactly one stored event and is owned by where it starts.
- Tuplets behave like local metric frames with child event streams; rests inside governed local slots remain derived from absence of tones.
- `tone_id` remains the authoritative tone identity. `tuplet_id` is lookup context for nested child storage, not a second identity model.
- Crossing the tuplet boundary does not itself force a storage split. Story 2.3 must not restructure surrounding tones just to satisfy slot operations.
- MVP tuplets are intentionally single-level only. Any path that would require nested tuplets must be rejected or deferred rather than represented internally.
- `ui.flow_id` remains the one authoritative active-flow slot across score and Sequence surfaces.

### Library / Framework Requirements

- Frontend work stays in the current React 19 + TypeScript + Tauri invoke stack.
- Backend mutation authority remains Rust-owned state behind existing Tauri commands.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary frontend seams:
  - `src/windows/main/components/sequencer-track.tsx`
  - `src/windows/main/components/sequencer-track-tuplet.tsx`
  - `src/services/actions.ts`
  - `src/services/sequencer-utils.ts`
- Primary backend seams:
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/score/entries/tuplet.rs`
  - `src-tauri/src/score/tracks.rs` only if shared entry-index maintenance needs adjustment
- Existing tests to extend before adding new harnesses:
  - `src/windows/main/components/sequencer-track.test.tsx`
  - `src/services/sequencer-utils.test.ts`
  - `src/services/actions.test.ts`
  - backend tests in `src-tauri/src/score/entries/tone.rs`
  - backend tests in `src-tauri/src/score/entries/tuplet.rs`

### Testing Requirements

- Verify entering a note into an empty tuplet slot stores it through the shared track/tone action seam with the correct `tuplet_id` context.
- Verify removing a tuplet-owned note removes only that sounding content and leaves empty local space represented by absence of tones.
- Verify vacated tuplet slots remain visibly targetable and the enclosing tuplet region does not collapse after note removal.
- Verify a partially filled tuplet remains intact and available for later note entry after member removal.
- Verify exact timing projection and by-tick/by-id indexing remain stable for tuplet child entries after create/remove operations.
- Verify no nested tuplet structure is created or preserved by slot operations.
- Undo/redo is not an acceptance requirement for this story because the app does not yet provide that infrastructure.

### Established Code Patterns To Follow

- Extend shared seams before adding new ones. The repo pattern for recent tuplet work is narrow seam reuse with focused regressions.
- Treat tuplet membership as nested storage context, not a separate product-level editing mode.
- Keep empty-slot rendering in the existing tuplet region component; do not replace it with synthetic rest entries or a second interaction layer.
- Extend existing tests in place rather than introducing a tuplet-only add/remove harness.
- Keep Story 2.3 tightly scoped to slot entry/removal. Do not absorb drag, resize, reposition, container delete, or slice semantics.

### Previous Story Intelligence

- Story 2.1 solved reliable targeting for tuplet members and clarified that MVP targeting only needs one container depth plus members and empty regions.
- Story 2.2 solved tuplet-owned pitch parity by threading known `tuplet_id` context through shared tone seams rather than adding a tuplet-only mutation path.
- The same lesson applies here: add/remove behavior should reuse existing track/tone seams with explicit tuplet context, not reconstruct ownership through ad hoc scanning or new APIs.
- Story 2.2 also kept the implementation intentionally narrow. Preserve that discipline here by not pulling Story 2.4 drag behavior or Story 2.5 container lifecycle rules forward.

### Git Intelligence Summary

- Recent relevant commits:
  - `802f5e64 fix: no tuple highlights`
  - `6cd8bef8 feat(2.2): finish tuplet pitch editing parity`
  - `c8ea0449 feat(2.1): tuple tone selection info`
- The current implementation pattern is to tighten the existing sequencer and tone seams, then add focused regression coverage. Story 2.3 should follow the same pattern.

### Documentation Impact

- This story changes implementation behavior in a way that may affect documented editing semantics and developer guardrails.
- Review these docs during implementation and update them if the shipped contract changes materially:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if any durable guardrail changes
- If implementation is pure seam reuse that only fulfills already-documented behavior, record that no documentation update was required in the completion notes rather than skipping the review silently.

### Project Structure Notes

- The sequencer already renders tuplet regions and slot markers separately from child tones. That is an asset: empty slots should remain visible without adding fake persisted entries.
- The action layer already accepts optional `tuplet_id` for tone create/remove flows. Prefer plumbing existing context through callers instead of widening the API surface.
- The main implementation risks are:
  - accidentally adding notes on the host track instead of the tuplet child stream
  - collapsing empty slots after note deletion
  - letting Story 2.3 drift into drag/resize or container-delete semantics owned by later stories

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 / Story 2.3, Story 2.4, and Story 2.5 scope boundaries
- Source: `_bmad-output/planning-artifacts/prd.md` — editing semantics, exact timing, derived rests, and non-nestable MVP rules
- Source: `_bmad-output/planning-artifacts/architecture.md` — local metric frame model, derived-rest rule, `tone_id` authority, sequencer empty-slot contract, and deletion rule boundaries
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — tuplet sequencer presentation, empty-slot visibility, and selection/feedback expectations
- Source: `docs/project-context.md` — shared action path, Rust-owned state, and `ui.flow_id` guardrail
- Source: `_bmad-output/implementation-artifacts/2-2-edit-pitch-inside-a-tuplet-without-changing-timing.md` — prior story seam-reuse pattern and scope discipline

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 2 Story 2.3 acceptance criteria and adjacent story boundaries in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed PRD guidance for exact timing, derived rests, deletion semantics, and non-nestable tuplets in `_bmad-output/planning-artifacts/prd.md`.
- Reviewed architecture guidance for local metric frames, `tone_id` authority, empty-slot rendering obligations, and container delete boundaries in `_bmad-output/planning-artifacts/architecture.md`.
- Reviewed UX guidance for sequencer tuplet presentation, empty-slot visibility, and selection behavior in `_bmad-output/planning-artifacts/ux-design-specification.md`.
- Inspected the prior implementation artifact for Story 2.2 and recent git history to keep the story aligned with current seam-reuse patterns.
- Explored the current codebase seams for tuplet note create/remove flows in `sequencer-track.tsx`, `sequencer-track-tuplet.tsx`, `actions.ts`, `sequencer-utils.ts`, `tone.rs`, and `tuplet.rs`.
- Confirmed the existing sequencer write/erase flows already route tuplet slot add/remove through the shared action seams and Rust entry-management path without introducing a tuplet-only API.
- Added focused regression coverage in `src/windows/main/components/sequencer-track.test.tsx`, `src-tauri/src/score/entries/tone.rs`, and `src-tauri/src/score/entries/tuplet.rs` for tuplet child removal and stable empty-slot/index behavior.
- Ran `vitest` on `src/windows/main/components/sequencer-track.test.tsx`, targeted `cargo test` cases for the new Rust regressions, `npm run lint`, `npm run check-types`, and the full `cargo test --manifest-path src-tauri/Cargo.toml` suite.

### Completion Notes List

- Story 2.3 is intentionally narrowed to slot-level note entry and member removal only.
- Story 2.4 retains drag, reposition, resize, and gesture-driven timing edits.
- Story 2.5 retains container delete and slice semantics.
- Empty tuplet space should remain represented by absence of tones plus visible slot markers, not by persisted rest entries or synthetic selectable objects.
- Documentation impact review is required before this story is closed.
- Verified that Story 2.3 behavior was already implemented on the shared sequencer/action/Rust seams; this change closes the story by adding the missing focused regressions and validation evidence.
- Added frontend regressions for erasing a tuplet-owned tone through `actions.score.tone.remove(...)` with `tuplet_id` context and for preserving visible tuplet slot markers after the child note is removed.
- Added backend regressions proving `Track::remove_tone(...)` preserves the enclosing tuplet plus surviving child indexes and `TupletEntries::remove_entry(...)` removes only the targeted member from shared local-tick indexes.
- Documentation review outcome: no docs or agent-instruction update was required because the shipped contract already matched existing tuplet editing guidance; this pass only hardened regression coverage around already-supported behavior.

### File List

- `_bmad-output/implementation-artifacts/2-3-add-and-remove-notes-within-tuplet-slots.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/windows/main/components/sequencer-track.test.tsx`
- `src-tauri/src/score/entries/tone.rs`
- `src-tauri/src/score/entries/tuplet.rs`

### Change Log

- 2026-03-16: Created Story 2.3 implementation artifact with code-seam guidance, scope boundaries, testing expectations, and docs-impact requirements.
- 2026-03-16: Validated existing tuplet slot add/remove behavior on the shared seams, added focused frontend/Rust regressions, ran required repo checks, and moved the story to review.
- 2026-03-16: User accepted Story 2.3 and the story status was closed to done.
