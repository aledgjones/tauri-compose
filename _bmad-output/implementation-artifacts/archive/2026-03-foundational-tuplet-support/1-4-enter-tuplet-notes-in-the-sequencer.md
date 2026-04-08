# Story 1.4: Enter Tuplet Notes in the Sequencer

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want to add notes for a created triplet directly in the sequencer,
so that tuplets fit the same primary note-entry workflow I already use.

## Scope Note

- This story covers tuplet note insertion only.
- Full tuplet-member targeting, add/remove refinement, and move/resize semantics remain progressive follow-up work in Epic 2.
- The public mutation surface must remain track-level. Tuplet-aware helpers may exist internally, but callers should extend the existing tone-creation seam rather than switching to a parallel tuplet-note API.

## Acceptance Criteria

1. Given a tuplet container has been created on the active track, when the composer enters notes in the sequencer within that tuplet span, then those notes are written into the tuplet child event stream through the active tuplet multiplier rather than plain host-time entry rules, and their stored timing remains exact while the written value stays musically intelligible inside the tuplet frame.
2. Given a sequencer view showing a track with a tuplet container, when the composer targets note entry inside the tuplet span, then the sequencer exposes a tuplet-aware entry region that maps input to valid child positions, and the user does not need to calculate approximate binary timing values manually.
3. Given a composer prepares a note inside the tuplet span, when the entry is committed, then the system checks the active tuplet multiplier in the same way other contextual directives affect interpretation, and the resulting note is stored with exact backend timing rather than approximate binary rounding.
4. Given tuplet notes are entered through the sequencer, when the resulting score state is inspected, then flow, track, and tone ownership remain unambiguous, and the current authoritative `ui.flow_id` behavior remains intact.
5. Given entered tuplet notes coexist with empty local space in the same region, when the sequencer refreshes after note entry, then entered notes remain legible as members of the tuplet container, and any remaining empty local slots stay targetable for later entry.
6. Given the composer is already working inside an MVP tuplet span, when an action would require creating another tuplet inside that governed local frame, then the product rejects or defers that action, and MVP does not create nested tuplets.

## Tasks / Subtasks

- [x] Extend the existing track-level tone creation seam with optional tuplet context instead of adding a parallel tuplet-note API (AC: 1, 2, 3, 4)
  - [x] Extend `actions.score.tone.create(...)` in `src/services/actions.ts` and `score_entries_tone_create(...)` in `src-tauri/src/score/entries/tone.rs` with optional `tuplet_id` context rather than introducing `score.tuplet.tone.create(...)` or another adjacent mutation API.
  - [x] Extend `Track::create_tone(...)` so the `Track` remains the authoritative mutation entry point and routes insertion to host `entries` or `Tuplet.entries` based on optional tuplet context; convenience helpers on `Tuplet` or `TupletEntries` are allowed only as internal helpers under that track-owned mutation path.
  - [x] Keep the addressing model explicit as `track_id -> optional tuplet_id -> tone_id`; do not make child tone mutation depend on translated host ticks or a separate frontend-owned identity scheme.
  - [x] Preserve existing chord ergonomics by allowing multiple tones at the same local tuplet tick just as multiple host tones can already share one tick.
- [x] Add tuplet-aware sequencer write-path logic that derives local insertion positions directly from the tuplet region (AC: 1, 2, 3, 5)
  - [x] Update the sequencer write flow in `src/windows/main/components/sequencer-track.tsx` and any tuplet helper modules so pointer entry inside a tuplet region resolves directly to a tuplet-local tick from the region's local slot geometry rather than computing a host tick and translating it afterward.
  - [x] Reuse the current snap and written-duration preparation rules for inserted tones, but apply them in the tuplet's local timeline; ticks and durations remain relative to their immediate parent context.
  - [x] Keep this story insert-only: do not broaden the drag-to-resize or drag-to-reposition paths into full tuplet-aware editing semantics here.
- [x] Preserve backend-owned tuplet invariants during insertion (AC: 1, 3, 4, 6)
  - [x] Keep MVP tuplet child content single-level and tone-only by building on the `TupletEntries` constraint from Story 1.3A rather than reintroducing nested-capable child insertion.
  - [x] Do not convert tuplet-local ticks back into track ticks or vice versa during insertion. The tuplet has its own local timeline, and child entries must stay parent-relative.
  - [x] Keep rests represented by absence of tones inside the tuplet child frame; note insertion must not create explicit persisted rest entries to fill unused local space.
- [x] Keep sequencer selection and state handoff coherent without pulling full member-targeting work forward (AC: 4, 5)
  - [x] Preserve `ui.flow_id` as the authoritative active-flow slot through the existing selection/update paths.
  - [x] If the insertion flow selects the newly created child tone, carry the minimal tuplet context needed for that write-path handoff only; do not broaden this story into the general member-targeting work planned for Story 2.1.
  - [x] Keep empty local slots visibly targetable after insertion so later note-entry passes can continue filling the tuplet region without rediscovering structure.
- [x] Add focused regression coverage close to the owning modules (AC: 1, 2, 3, 4, 5, 6)
  - [x] Add frontend tests around the sequencer write path proving tuplet-region clicks create tones through the existing `actions.score.tone.create(...)` seam with `track_id`, local tuplet tick, local duration, and optional `tuplet_id` context.
  - [x] Add Rust tests around `Track::create_tone(...)` proving host insertion still works unchanged, tuplet insertion writes into `Tuplet.entries`, multiple tones can share one local tuplet tick, and invalid or missing tuplet context does not create incorrect state.
  - [x] Add or update integration-level tests for the sequencer refresh so newly inserted tuplet notes remain visible within the container while empty local slots remain targetable.
  - [x] Run the relevant checks for touched code paths: `npm run lint`, `npm run check-types`, `npm run test:js`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- [x] Update documentation for the new insertion contract and sequencer behavior (AC: 1, 2, 4, 5)
  - [x] Update `docs/data-models.md` to document that tuplet child tone insertion extends the existing track-level mutation seam with optional tuplet context and keeps child ticks relative to the tuplet parent.
  - [x] Update any relevant developer docs if the sequencer write-path contract or tone-mutation expectations become a durable project rule for future tuplet stories.

## Dev Notes

### Story Foundation

- Story 1.2 already creates backend-owned tuplet containers through the popover and hands selection back through the existing sequencer/flow state.
- Story 1.3 already renders tuplets as bounded sequencer regions with compressed internal slot cues, so Story 1.4 should build on that region geometry rather than inventing a second tuplet entry surface.
- Story 1.3A already constrained MVP tuplet child content to single-level, tone-only `TupletEntries`. Story 1.4 should rely on that invariant directly instead of adding repeated nested guards everywhere.
- This story is intentionally progressive and iterative. It adds insertion into existing tuplets only. Broader member targeting, removal, pitch editing, repositioning, resizing, and cross-boundary manipulation belong to later Epic 2 stories.

### Technical Requirements

- Extend the current tone creation path instead of adding a new adjacent API. The public mutation surface should remain `Track`-owned end to end:
  - frontend: `actions.score.tone.create(...)`
  - backend command: `score_entries_tone_create(...)`
  - score mutation: `Track::create_tone(...)`
- Optional tuplet context should be passed through that existing tone-creation path so the caller stays at track scope while still being able to target a child tuplet stream.
- Ticks are always relative to their immediate parent:
  - host-track tones use track-relative ticks
  - tuplet child tones use tuplet-local ticks
  - durations follow the same parent-relative rule
- Do not convert tuplet-local ticks into host-track ticks or host-track ticks into tuplet-local ticks as part of insertion logic. Fractional or round-tripped tick translation is explicitly out of bounds for this story.
- The sequencer should derive tuplet-local positions directly from the tuplet region's local slot geometry and current snap/written-duration intent rather than creating a host tick first and translating afterward.
- Multiple tones at one local tuplet tick are valid and should follow the existing chord ergonomics rather than introducing one-note-per-slot restrictions.
- Keep Rust as the source of truth. The frontend may derive click geometry and temporary UI state, but it must not invent a parallel tuplet timing or ownership model.
- Keep rests implicit through absence of tones inside `Tuplet.entries`; insertion should not introduce explicit persisted rest entries.
- Keep `ui.flow_id` authoritative. This story must not introduce parallel frontend-owned active-flow or active-tuplet state.
- Unsupported outcomes should follow existing rejection/no-op behavior patterns rather than introducing bespoke tuplet-only error messaging for MVP.

### Architecture Compliance

- The architecture's core ownership rule still applies: a tone is one stored event owned by where it starts. Story 1.4 only covers tones that start inside an existing tuplet and therefore belong in `Tuplet.entries`.
- The architecture explicitly describes tuplets as host-track containers with local metric frames. Story 1.4 should therefore extend host-track mutation with tuplet-aware routing rather than treating tuplets as a second public timeline.
- The PRD and UX spec both require sequencer-native note entry inside tuplets as part of the MVP path, but they also defer richer manipulation semantics to follow-on work. Keep this story limited to insertion.
- MVP tuplets remain non-nestable. Do not add tuple-path abstractions or nested child insertion behavior here.

### Library / Framework Requirements

- Frontend work should stay within the existing React + TypeScript + Tauri `invoke()` action flow.
- Backend work should follow the current Rust command pattern with `#[tauri::command(rename_all = "snake_case")]`, `AppContext` mutation behind `Mutex`, and affected-track emission through the existing renderer update path.
- Continue using `FxHashMap` in Rust-owned storage code.
- No new external library is expected for this story.

### File Structure Requirements

- Expected frontend touchpoints:
  - `src/services/actions.ts`
  - `src/services/types.ts`
  - `src/windows/main/components/sequencer-track.tsx`
  - `src/windows/main/components/sequencer-track-tuplet.tsx`
  - `src/windows/main/components/sequencer-track.test.tsx`
  - any existing tuplet-region utility used to derive local slot geometry
- Expected backend touchpoints:
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/score/entries/tuplet.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/automation/registry.rs` if automation command signatures must stay aligned
  - `src-tauri/src/ui/sequencer.rs` only if minimal write-path selection context must carry tuplet ownership
- Keep the work focused. Do not pull Story 1.5 reconciliation-over-existing-material work, Story 2.1 generalized member targeting, Story 2.3 add/remove behavior, or Story 2.4 move/resize semantics forward unnecessarily.

### Testing Requirements

- Verify host-track note entry remains unchanged when no `tuplet_id` is provided.
- Verify tuplet note entry inserts into the correct `Tuplet.entries` collection with local ticks and local durations preserved exactly as provided.
- Verify multiple child tones can exist at the same local tuplet tick.
- Verify invalid tuplet context is rejected safely and does not create broken host or child state.
- Verify the sequencer continues to show inserted tuplet notes as members of the bounded tuplet region while leaving remaining empty local slots targetable.
- Run the relevant frontend and Rust checks after implementation.

### Previous Story Intelligence

- Story 1.2 established the tuplet creation and selection handshake through `actions.score.tuplet.create(...)`, `score_entries_tuplet_create(...)`, and `Selection::Tuplet`.
- Story 1.3 established the bounded sequencer region and compressed local markers, which should now be reused as the visual/geometry basis for note insertion.
- Story 1.3A replaced generic nested child entries with `TupletEntries` so MVP child insertion is already constrained to tones only.
- Current code already centralizes tone mutation through `actions.score.tone.create(...)`, `score_entries_tone_create(...)`, and `Track::create_tone(...)`; Story 1.4 should extend those seams instead of duplicating them.

### Git Intelligence Summary

- Recent relevant commits:
  - `1ede423a feat(1.3a): tuplet entries structures non-recursive`
  - `356b5203 feat: course correct`
  - `38c8bede fix: tweak tuple background colour`
  - `0a06d047 feat(1.3): description`
  - `f544812e fix: tuple popover trigger`
- Recent work shows the tuplet direction is being refined progressively rather than delivered as one broad edit pass. Keep Story 1.4 narrowly additive and compatible with that incremental approach.

### Documentation Impact

- This story establishes a durable mutation rule for future tuplet work: extend the existing track-level tone creation seam with optional tuplet context rather than creating adjacent APIs.
- Update docs in the same implementation if that rule or the parent-relative tick contract becomes newly explicit in the codebase.

### Project Structure Notes

- `handleWriteTool(...)` in `src/windows/main/components/sequencer-track.tsx` currently computes host-track ticks and immediately calls `actions.score.tone.create(...)`. That is the primary seam to extend for tuplet insertion.
- `actions.score.tone.create(...)` and `score_entries_tone_create(...)` currently accept only track-level tick/duration inputs with no tuplet context.
- `Selection::Tuplet` already exists, but general child-member targeting is intentionally later work. Avoid broadening Story 1.4 into a full tuplet-member targeting project.
- `TupletEntries` currently stores tones only and already supports multiple ids per local tick through `by_tick` + `by_id`, which aligns with chord entry expectations.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 / Story 1.4; Epic 2 / Stories 2.1 to 2.4 scope boundaries
- Source: `_bmad-output/planning-artifacts/prd.md` — MVP scope; Editing semantics; Data model and persistence stakes
- Source: `_bmad-output/planning-artifacts/architecture.md` — Agreed model invariants; child track shape; creation and reconciliation rules
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — Tuplet creation handoff; sequencer tuplet presentation; note and empty-slot states
- Source: `docs/project-context.md` — critical implementation rules; frontend/backend action architecture; `ui.flow_id` guardrail
- Source: `_bmad-output/implementation-artifacts/1-2-create-tuplet-containers-through-a-tuplet-popover.md`
- Source: `_bmad-output/implementation-artifacts/1-3-reveal-and-select-the-created-tuplet-region-in-the-sequencer.md`
- Source: `_bmad-output/implementation-artifacts/1-3a-simplify-tuplet-child-content-for-non-nestable-mvp.md`
- Source: `src/services/actions.ts` — existing score action surface
- Source: `src/services/types.ts` — shared DTOs for `Tuplet`, `Track`, and selections
- Source: `src-tauri/src/score/entries/tone.rs` — current track-level tone creation and mutation seam
- Source: `src-tauri/src/score/entries/tuplet.rs` — tuplet child storage and track-owned tuplet creation helpers
- Source: `src-tauri/src/ui/sequencer.rs` — selection/state ownership and `ui.flow_id` handling
- Source: `src/windows/main/components/sequencer-track.tsx` — current write-path implementation

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- `npm run lint`
- `npm run check-types`
- `npm run test:js`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Completion Notes List

- Extended the existing track-owned tone creation seam with optional `tuplet_id` context across TypeScript actions, Rust commands, automation dispatch, and `Track::create_tone(...)` routing.
- Added tuplet-local sequencer write mapping and rendered child tones inside bounded tuplet regions while preserving empty local slot targeting and host-only drag/resize semantics.
- Fixed tuple-aware write routing so clicks on existing child tones still create through the same tuplet-owned track seam rather than falling back to host-track insertion.
- Added minimal write-path tone selection context via optional `tuplet_id` so child tones can be highlighted without broadening member-targeting scope.
- Hardened backend tuplet insertion by resolving pitch context from projected host time for directive lookup and rejecting overlapping MVP tuplet creation.
- Added focused regression coverage for action dispatch, tuplet-region note entry, child-tone click routing, tuplet child visibility, host insertion preservation, shared local-tick chords, overlapping-tuplet rejection, projected pitch-context lookup, and invalid tuplet context rejection.
- Updated `docs/data-models.md` and `docs/development-guide.md` to record the durable insertion contract and future tuplet guardrails.
- Validation passed: `npm run lint`, `npm run check-types`, `npm run test:js`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### File List

- `_bmad-output/implementation-artifacts/1-4-enter-tuplet-notes-in-the-sequencer.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src/services/actions.test.ts`
- `src/services/actions.ts`
- `src/services/sequencer-utils.test.ts`
- `src/services/sequencer-utils.ts`
- `src/services/types.ts`
- `src/windows/main/components/sequencer-track.test.tsx`
- `src/windows/main/components/sequencer-track-tone.tsx`
- `src/windows/main/components/sequencer-track.tsx`
- `src/windows/main/components/sequencer-tuplet-utils.ts`
- `src-tauri/src/automation/registry.rs`
- `src-tauri/src/instructions/draw/draw_accidentals.rs`
- `src-tauri/src/instructions/draw/draw_notehead.rs`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/renderer/hit_test.rs`
- `src-tauri/src/score/entries/tone.rs`
- `src-tauri/src/score/entries/tuplet.rs`
- `src-tauri/src/ui/sequencer.rs`

## Senior Developer Review (AI)

### Reviewer

- Aled via GPT-5.4 on 2026-03-15

### Outcome

- Approved after fixes.

### Findings Resolved

- Fixed tuplet write routing so pointer entry over an existing tuplet child tone still resolves the owning `tuplet_id` and creates through the existing track-level tone seam.
- Fixed backend tuplet-child pitch resolution to derive directive lookup context from projected host time instead of raw local tuplet ticks.
- Fixed MVP tuplet creation invariants so overlapping/nested-at-host-span tuplets are rejected by the backend creation path.
- Fixed story bookkeeping so the File List includes the changed sequencer click-target test surface discovered during review.

### Scope Clarification

- The review originally flagged missing write-tool drag-resize behavior for tuplet creation. That item is intentionally deferred rather than fixed here. Story 1.4 remains insert-only, and broader tuplet reposition/resize semantics belong to Story 2.4.
- Tuplet-note selection now resolves correctly, but the bottom-bar selection summary still needs to follow `selection.tuplet_id` when describing selected child tones. That UI follow-up is explicitly owned by Story 2.1: Target Tuplet Containers and Members Reliably.

### Validation

- `npm run lint`
- `npm run check-types`
- `npm run test:js`
- `cargo test --manifest-path src-tauri/Cargo.toml create_tone_`
- `cargo test --manifest-path src-tauri/Cargo.toml create_tuplet_`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Change Log

- 2026-03-15: Implemented track-owned tuplet note insertion, tuplet-local sequencer write mapping, child-tone rendering, focused regression coverage, and documentation updates for the insertion contract.
- 2026-03-15: Senior review fixes: child-tone write routing stays tuplet-aware, overlapping MVP tuplets are rejected, and tuplet pitch context now resolves from projected host time.
