# Story 2.5: Apply Explicit MVP Delete and Slice Rules for Tuplet Members and Containers

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want delete and slice behavior for tuplets to follow clear rules,
so that correction workflows are predictable.

## Scope Note

- This story defines explicit MVP semantics for deleting tuplet members, deleting tuplet containers, and slicing tones that are inside or cross tuplet boundaries.
- Removing a single tuplet-owned note remains the same slot-level behavior established in Story 2.3: only that member is removed, the tuplet remains, and the vacated span becomes empty tuplet space represented by absence of tones.
- Deleting a tuplet container is in scope here and must remove the container together with all child notes authored inside it; MVP does not dissolve child content back into host-track tones.
- This story may add the shared action and backend mutation seam for tuplet-container deletion, but it must not promise a user-triggerable UI hookup yet. Score-renderer-driven container deletion belongs to upcoming work and current sequencer or keyboard deletion paths should not be retrofitted as a stopgap.
- Slicing is in scope only for tones. Tuplet containers themselves are not sliced, resized, or structurally repaired into multiple tuplets in this story.
- Boundary-crossing tones remain governed by the one-stored-event-per-tone rule for each resulting tone after a slice. The product may create two persisted tones because a slice intentionally creates two musical tones, but it must not create extra persisted fragments solely because a tuplet boundary was crossed.
- Empty tuplet space remains derived interaction context, not a persisted rest and not a selectable synthetic object.
- MVP tuplets remain single-level only. Delete and slice behavior must reject or defer outcomes that would require nested tuplet creation, nested repair, or overlapping host-and-child ownership.
- This story must not introduce frontend-owned tuplet editing state, dissolve-to-host-time reconstruction, or a parallel tuplet-only delete/slice workflow detached from existing tone and selection seams.

## Acceptance Criteria

1. Given the composer deletes a tuplet member, when the action is applied, then only that note is removed and the tuplet container remains in place with the vacated span becoming empty local tuplet space.
2. Given the composer deletes the tuplet container, when the action is confirmed, then the tuplet and every child tone authored inside that tuplet span are removed together as one container-level delete intent and MVP does not attempt dissolve-back-to-host-time behavior.
3. Given the composer slices a tone that is fully inside a tuplet or crosses a tuplet boundary, when the slice action is applied in a supported MVP case, then the result remains governed by the one-stored-event-per-tone ownership rule for the resulting tones after the edit and any tuplet-boundary fragment needed for notation or playback remains a derived result rather than an extra persisted split introduced solely by the boundary.
4. Given a slice splits a boundary-crossing tone into two resulting tones whose starts land on opposite sides of the tuplet boundary, when the slice is committed, then each resulting tone is stored in the owning context determined by its own start tick: a leading segment that still starts inside the tuplet remains tuplet-owned while an out-of-tuplet remainder becomes host-track owned, and the same rule applies in reverse for a host-owned leading segment with a tuplet-owned remainder.
5. Given a delete action would otherwise require unsupported repair behavior, when the action is processed, then the product applies the defined MVP rule above rather than inventing a new structural interpretation and no overlapping host and child ownership remains afterward.
6. Given a delete or slice path would require nested tuplet repair behavior, when the action is evaluated, then MVP does not create or preserve nested tuplet structure as part of the repair and the single-level tuplet invariant remains intact.

## Tasks / Subtasks

- [x] Reuse and extend the existing shared delete/slice seams rather than introducing a parallel tuplet-editing API family (AC: 1, 2, 3, 4, 5, 6)
  - [x] Extend `src/services/actions.ts` so tone slicing can carry explicit `tuplet_id` context when the selected tone is tuplet-owned or when the current interaction already knows the owning tuplet.
  - [x] Add a shared frontend action for tuplet container removal in `src/services/actions.ts` instead of routing container deletion through ad hoc UI-only state changes.
  - [x] Keep member deletion on the existing `actions.score.tone.remove(...)` path with `tuplet_id` context rather than inventing a separate member-delete command.
  - [x] Update the corresponding Rust command surface in `src-tauri/src/score/entries/tone.rs` and `src-tauri/src/score/entries/tuplet.rs` so delete and slice operations remain backend-owned mutations behind Tauri commands.
- [x] Make tone slicing tuplet-aware while preserving ownership-by-start and derived boundary splitting (AC: 3, 4, 5, 6)
  - [x] Extend `Track::slice_tone(...)` and `score_entries_tone_slice(...)` in `src-tauri/src/score/entries/tone.rs` so slicing can operate on host-owned tones, tuplet-owned tones, and boundary-crossing tones without assuming all storage is host-track only.
  - [x] Ensure each resulting tone after a supported slice is stored exactly once in the owning context determined by its own start tick.
  - [x] Preserve the rule that crossing a tuplet boundary does not itself create extra persisted fragments; any additional split needed at the tuplet edge remains a notation/render/playback derivation concern.
  - [x] Reject or defer slice outcomes that would require nested tuplets, overlapping host-and-child storage for the same resulting tone, or any other unsupported repair interpretation.
- [x] Add explicit tuplet-container deletion with MVP removal semantics only (AC: 2, 4, 5)
  - [x] Introduce a backend-owned tuplet removal seam that deletes the selected tuplet entry and all of its child tones together from the authoritative track model.
  - [x] Preserve the current MVP rule that container deletion removes the tuplet outright; do not attempt to re-home child tones onto the host track and do not synthesize rest entries.
  - [x] Reuse existing track and tuplet index maintenance so `by_tick` and `by_id` remain consistent after container removal.
  - [x] Update any public automation/action registry metadata that enumerates supported score mutations if the command surface changes.
- [x] Add the container-delete action seam now while explicitly deferring user-triggerable UI hookup to score-renderer follow-up work (AC: 2)
  - [x] Expose the shared action and backend command needed for future container deletion without wiring it into current sequencer, keyboard-shortcut, or background-click deletion paths.
  - [x] Preserve the existing behavior for tone, rest, and directive deletion cases in current UI surfaces; this story should not retrofit tuplet container deletion onto a stopgap interaction.
  - [x] Keep empty tuplet space non-selectable and non-deletable. Future UI hookup must remain tied to explicit tuplet-container targeting rather than background clicks in the tuplet region.
- [x] Preserve Story 2.4 address-lattice and ownership-transfer guardrails while adding delete/slice behavior (AC: 1, 3, 4, 5, 6)
  - [x] Build on the canonical sequencer address lattice and current shared ownership-context patterns rather than reintroducing host-only assumptions into delete/slice code.
  - [x] Ensure any slice-triggered follow-up selection or geometry refresh continues to derive from the same canonical address model used for rendering, hit-testing, and snapping.
  - [x] Keep `ui.flow_id` as the only authoritative active-flow slot; do not add frontend-owned active-tuplet flow state during delete/slice handling.
- [x] Add focused regressions for explicit MVP delete and slice rules (AC: 1, 2, 3, 4, 5, 6)
  - [x] Extend `src/services/actions.test.ts` to cover tone slice with `tuplet_id` context and tuplet-container removal action wiring.
  - [x] Add or extend action/command-level coverage to prove the tuplet-container delete seam exists for future score-renderer work without asserting a current keyboard or sequencer hookup.
  - [x] Extend backend tests in `src-tauri/src/score/entries/tone.rs` for slicing tuplet-owned tones, slicing host-owned tones that cross into tuplets, and slice rejection for unsupported nested-repair cases.
  - [x] Extend backend tests in `src-tauri/src/score/entries/tuplet.rs` and any shared track-index tests so removing a container clears child ownership cleanly without collapsing unrelated track state.
  - [x] Extend sequencer-facing tests only where selection or post-mutation refresh behavior changes; do not create a brand-new tuplet delete/slice test harness if the existing suites can cover the seam.
- [x] Validate the touched seams with the standard repo checks
  - [x] Run the relevant frontend tests covering action wiring and keyboard-shortcut deletion behavior.
  - [x] Run the relevant Rust tests covering tone slicing and tuplet/container removal behavior.
  - [x] Run `npm run lint` and `npm run check-types` after implementation.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` if Rust mutation seams change.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` if Rust files change, because clippy-clean is a repo rule.
- [x] Review documentation impact before closing the story
  - [x] Confirm whether the shipped delete/slice semantics change documented tuplet editing rules, developer guardrails, or workflow expectations.
  - [x] Update the relevant docs and agent instructions in the same change if the contract changes materially.
  - [x] If no docs change is needed, record that explicit decision in the completion notes.

## Dev Notes

### Story Foundation

- Epic 2 is about editing tuplets without breaking musical intent. Story 2.5 is the explicit rule-setting story for delete and slice behavior after targeting, pitch parity, slot add/remove, and drag/resize groundwork.
- Story 2.3 already established the rule for member removal: removing a tuplet-owned note leaves empty local tuplet space and keeps the container intact.
- Story 2.4 established the canonical sequencer address lattice and shared ownership-context seams for host-to-tuplet and tuplet-to-host movement. Story 2.5 must build on that work rather than bypass it.
- The current codebase still has a host-only assumption in slice handling and no shared tuplet-container delete action. This story exists to make those semantics explicit and backend-owned.

### Technical Requirements

- Reuse and refine the existing frontend action wrappers in `src/services/actions.ts`:
  - `actions.score.tone.remove(track_id, tuplet_id | null, tone_id)`
  - `actions.score.tone.slice(...)` must become tuplet-aware instead of assuming host-only ownership.
  - `actions.score.tuplet.create(...)` already exists; add a corresponding shared remove action for container deletion.
  - Do not wire the new container-delete action into current keyboard or sequencer deletion flows yet; the user-triggerable score-renderer surface for container deletion is upcoming work.
- Reuse and refine the existing backend tone seams in `src-tauri/src/score/entries/tone.rs`:
  - `Track::remove_tone(...)`
  - `Track::slice_tone(...)`
  - `score_entries_tone_duration_remove(...)`
  - `score_entries_tone_slice(...)`
  - `resolve_tuplet_pitch_tick(...)` where exact pitch-resolution context remains relevant for resulting tones.
- Reuse child-entry indexing and container helpers in `src-tauri/src/score/entries/tuplet.rs`:
  - `TupletEntries::remove_entry(...)`
  - existing tuplet lookup and index-maintenance helpers
- Preserve exact timing and ownership semantics:
  - one tone is one stored event until a user intentionally slices it into two tones
  - after slicing, each resulting tone is still exactly one stored event owned by where that resulting tone starts
  - if a tuplet-owned tone is sliced at a point that leaves the remainder starting outside the tuplet, the first result stays tuplet-owned and the remainder becomes host-track owned
  - if a host-owned tone is sliced at a point that leaves the remainder starting inside the tuplet, the first result stays host-track owned and the remainder becomes tuplet-owned
  - crossing a tuplet boundary must not create extra persisted fragments beyond those two intentional slice results
- Preserve the current MVP rule that deleting a tuplet container removes the container and its children together. Do not dissolve or reconstruct host-track tones from child content.
- Preserve empty local tuplet space as absence of tones. Do not create persisted rest entries as part of delete or slice repair.
- Preserve the product's existing rejection behavior for unsupported edits. Do not add a tuplet-specific exception mode or silent fallback to approximate non-tuplet timing.

### Architecture Compliance

- Rust remains the source of truth for accepted delete and slice mutations. Frontend code should dispatch shared actions and reflect refreshed state, not decide final ownership or repair outcomes on its own.
- Tuplets remain bounded local metric containers with child event streams. Child membership is storage context, not a second product-level tone identity model.
- Ownership stays start-based:
  - if a resulting tone starts inside the tuplet, it belongs in the tuplet child `entries`
  - if a resulting tone starts outside the tuplet, it belongs on the host track
- Crossing a tuplet boundary never by itself forces persisted storage splitting. Derived score/render/playback fragments remain downstream concerns.
- MVP tuplets stay single-level only. Any delete or slice path that would require nested tuplets or nested repair must be rejected or deferred.
- The canonical frontend projection model from Story 2.4 remains authoritative. Delete and slice follow-up behavior must not reintroduce host-tick-only geometry assumptions.
- `ui.flow_id` remains the one authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport-facing consumers.

### Library / Framework Requirements

- Frontend work stays in the current React 19 + TypeScript + Tauri invoke stack.
- Backend mutation authority remains Rust-owned state behind existing Tauri commands.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary frontend seams:
  - `src/services/actions.ts`
  - `src/services/actions.test.ts`
  - `src/windows/main/components/sequencer-track.tsx` only if erase/select workflows need follow-up integration
  - `src/windows/main/components/sequencer-address-lattice.ts` only if post-slice refresh helpers need extension
- Primary backend seams:
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/score/entries/tuplet.rs`
  - `src-tauri/src/automation/registry.rs` if the command surface changes
  - `src-tauri/src/ui/sequencer.rs` only if selection clearing or refresh behavior needs command support
- Existing tests to extend before adding new harnesses:
  - `src/services/actions.test.ts`
  - backend tests in `src-tauri/src/score/entries/tone.rs`
  - backend tests in `src-tauri/src/score/entries/tuplet.rs`

### Testing Requirements

- Verify deleting a tuplet-owned member still removes only that note and leaves the tuplet container plus visible empty local space intact.
- Verify deleting a selected tuplet container removes the container and all child tones together without dissolving them into host-track tones.
- Verify slicing a tuplet-owned tone keeps both resulting tones valid under ownership-by-start rules and does not create extra persisted boundary fragments.
- Verify slicing a host-owned tone that crosses into a tuplet can yield a host-owned head and a tuplet-owned tail when the slice point makes that ownership split correct.
- Verify slicing a tuplet-owned tone that extends out of the tuplet can yield a tuplet-owned head and a host-owned tail when the new tail starts outside the tuplet.
- Verify the shared tuplet-container delete action exists for future score-renderer use without requiring current UI hookup tests.
- Verify unsupported slice or delete cases reject cleanly without creating nested tuplets, overlapping host-and-child ownership, or approximate timing fallbacks.
- Verify track and tuplet `by_tick`/`by_id` indexes remain stable after member removal, container removal, and supported slice outcomes.
- Undo/redo is not an acceptance requirement for this story because the app does not yet provide that infrastructure.

### Established Code Patterns To Follow

- Extend shared seams before adding new ones. Recent tuplet work has succeeded by threading explicit context through existing action and Rust mutation seams.
- Treat tuplet membership as nested storage context, not as a separate editing mode.
- Keep backend commands and automation metadata aligned. If a new tuplet-remove command is added, update the registry rather than leaving the public mutation surface undocumented.
- Keep empty tuplet space derived and non-selectable. Do not replace it with synthetic persisted content.
- Keep Story 2.5 tightly scoped to explicit MVP delete and tone-slice rules. Do not absorb container slicing, advanced repair, nested tuplets, or dissolve-to-host reconstruction.

### Previous Story Intelligence

- Story 2.3 proved that member deletion should reuse the existing shared remove seam with `tuplet_id` context and preserve empty local space as absence of tones.
- Story 2.4 replaced the discarded cursor-translation path with a canonical sequencer address lattice and sparse snap overlay; render, hit-testing, and snapping now share one address model.
- Story 2.4 also established shared ownership-context seams for host↔tuplet movement. Story 2.5 should use the same ownership-by-start reasoning for slice results instead of introducing a special-case repair model.
- The address lattice and tuplet projection helpers already exist. Delete/slice behavior should refresh through those same structures rather than building a second geometry interpretation for post-edit state.

### Git Intelligence Summary

- Recent relevant commits:
  - `6407f59f fix: snap expectations.`
  - `ba93d2a7 Complete Story 2.4 tuplet drag and snap work`
  - `d1bb39f2 chore: update story to follow arch guidlines`
  - `1fd644a4 chore(2.4): story`
  - `96cdae7d feat(2.3): basic tone editing`
- The recent pattern is consistent: tighten the existing sequencer and Rust seams, then add focused regression coverage. Story 2.5 should follow the same path.

### Documentation Impact

- This story may change documented tuplet editing semantics and should trigger a documentation impact review.
- Review these docs during implementation and update them if the shipped contract changes materially:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if any durable guardrail changes
- If implementation only fulfills already-documented behavior, record that no documentation update was required in the completion notes rather than skipping the review silently.

### Project Structure Notes

- Current user-triggerable tuplet-container deletion belongs to upcoming score-renderer work, so this story should stop at the shared action/backend seam and must not retrofit tuplet deletion onto current keyboard-shortcut or sequencer stopgaps.
- `actions.score.tone.slice(...)` and `score_entries_tone_slice(...)` currently assume host-track storage. That is the primary technical gap for this story.
- `Track::remove_tone(...)` and `TupletEntries::remove_entry(...)` already prove the backend can remove tuplet-owned child tones cleanly. Container delete should reuse existing index-maintenance patterns instead of rebuilding ownership state manually.
- The main implementation risks are:
  - leaving slice host-only and thereby storing tuplet-owned results in the wrong context
  - accidentally dissolving tuplet child content back onto the host track during container deletion
  - creating extra persisted fragments at the tuplet boundary instead of keeping boundary splitting derived
  - letting Story 2.5 drift into unsupported container-slice or nested-repair behavior

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 / Story 2.5 acceptance criteria and adjacent story boundaries
- Source: `_bmad-output/planning-artifacts/prd.md` — explicit tuplet editing rules, slice semantics, boundary-crossing ownership, and non-nestable MVP scope
- Source: `_bmad-output/planning-artifacts/architecture.md` — ownership-by-start, derived boundary splits, local metric frames, and child storage rules
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — empty-slot visibility, sequencer selection behavior, and rejection stability expectations
- Source: `docs/project-context.md` — shared action path, Rust-owned state, and `ui.flow_id` guardrail
- Source: `_bmad-output/implementation-artifacts/2-3-add-and-remove-notes-within-tuplet-slots.md` — prior member-removal seam reuse and scope discipline
- Source: `_bmad-output/implementation-artifacts/2-4-reposition-or-resize-tuplet-notes-within-supported-bounds.md` — address-lattice direction and shared ownership-context patterns

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 2 Story 2.5 acceptance criteria and adjacent story boundaries in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed PRD guidance for explicit tuplet edit semantics, slice behavior, ownership-by-start, and non-nestable MVP rules in `_bmad-output/planning-artifacts/prd.md`.
- Reviewed architecture guidance for local metric frames, one-stored-event ownership, derived boundary splits, and child-entry storage in `_bmad-output/planning-artifacts/architecture.md`.
- Reviewed UX guidance for sequencer tuplet presentation, empty-slot visibility, selection behavior, and rejection stability in `_bmad-output/planning-artifacts/ux-design-specification.md`.
- Reviewed Story 2.3 implementation guidance to preserve member-removal semantics and seam-reuse discipline.
- Reviewed Story 2.4 implementation guidance and repository memory to preserve the canonical address-lattice and ownership-context design.
- Inspected current delete/slice seams in `src/services/actions.ts`, `src/windows/main/hooks/use-keyboard-shortcuts.ts`, `src-tauri/src/score/entries/tone.rs`, `src-tauri/src/score/entries/tuplet.rs`, `src-tauri/src/ui/sequencer.rs`, and `src-tauri/src/automation/registry.rs`.
- Reviewed recent git history to align the story with the current incremental tuplet implementation pattern.

### Completion Notes List

- Added shared `actions.score.tuplet.remove(...)` / `score_entries_tuplet_remove(...)` seams for backend-owned container deletion without retrofitting current sequencer or keyboard deletion paths.
- Reworked tone slicing to use current and target ownership context, with the sequencer slice tool snapping through the canonical address lattice instead of host-only ticks.
- `Track::slice_tone(...)` now resolves host and tuplet addresses against one continuous ownership lattice, preserving exactly two persisted tones after a supported slice and rejecting invalid target-context splits.
- Extended documentation in `docs/development-guide.md` and `docs/data-models.md`; reviewed `.github/copilot-instructions.md` and no agent-instruction update was required because the new rules are story-scoped behavior, not a repo-wide workflow change.
- Validation completed: targeted Vitest suites for action wiring and keyboard shortcuts, `npm run test:js`, `npm run lint`, `npm run check-types`, focused Rust tone/tuplet tests, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### File List

- `_bmad-output/implementation-artifacts/2-5-apply-explicit-mvp-delete-and-slice-rules-for-tuplet-members-and-containers.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src/services/actions.ts`
- `src/services/actions.test.ts`
- `src/windows/main/components/sequencer-track.tsx`
- `src-tauri/src/automation/registry.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/score/entries/tone.rs`
- `src-tauri/src/score/entries/tuplet.rs`

### Change Log

- 2026-03-17: Created Story 2.5 implementation artifact with explicit MVP delete/slice semantics, shared-seam guidance, architecture guardrails, testing expectations, and documentation-impact tasks.
- 2026-03-17: Implemented tuplet-aware tone slicing, added backend-owned tuplet container deletion seams, extended regression coverage, and documented the shipped delete/slice ownership rules.
