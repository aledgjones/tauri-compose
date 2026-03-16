# Story 2.4: Reposition or Resize Tuplet Notes Within Supported Bounds

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want to drag tuplet notes to move, re-pitch, or resize them within the tuplet frame,
so that I can refine melodic and rhythmic shape without breaking exact tuplet semantics.

## Scope Note

- This story is about gesture-driven editing of existing tuplet-owned tones in the sequencer.
- Supported gesture families are: horizontal reposition, vertical re-pitch during drag, right-edge resize, and left-edge resize that changes start tick plus duration.
- Supported edits must preserve exact timing, one-stored-event-per-tone ownership, and single-level MVP tuplet structure.
- When a tone's start crosses the tuplet boundary, ownership must move with the start position in both directions: tuplet-owned tones become track-owned when they start outside the tuplet, and track-owned tones become tuplet-owned when they start inside the tuplet.
- Tuplet containers themselves are not resizable in this story.
- Snap duration remains literal across the boundary. If the active snap is an eighth note, outside the tuplet the gesture snaps to track eighths and inside the tuplet it snaps to tuplet eighths; the written duration choice stays the same while projection through time changes by context.
- Re-pitch is in scope when it happens through the existing drag gesture. Standalone pitch editing parity already exists from Story 2.2.
- Slot add/remove behavior remains Story 2.3 scope. Container delete and slice semantics remain Story 2.5 scope.
- All editing remains single-tone only. Multi-tone editing is not in the app yet and must not be introduced by this story.
- This story must not introduce frontend-owned tuplet edit state, nested tuplets, dissolve-to-host-time behavior, or a tuplet-only drag model separate from ordinary tone gestures.

## Acceptance Criteria

1. Given a tuplet note is edited for position or duration inside the tuplet, when the requested edit remains within the tuplet's supported local frame rules, then the change is applied using exact local timing and the result remains a valid tuplet state.
2. Given a tuplet note is dragged vertically using an existing pitch-drag gesture, when the requested edit stays within supported MVP bounds, then the product updates pitch using the same pitch-targeting rules used for ordinary track tones and tuplet membership remains an implementation detail rather than a separate drag behavior model.
3. Given a drag gesture changes pitch together with position or duration in a supported MVP case, when the edit is committed, then the product preserves the same user-facing drag semantics expected outside tuplets while applying the correct tuplet-local timing and ownership rules underneath and the gesture is not limited to timewise movement only.
4. Given a tone duration is dragged across a tuplet boundary in a supported MVP case, when the edit is committed, then the product preserves one continuous persisted tone and derives the boundary split at render or playback time and the user-facing gesture remains one continuous duration edit rather than a manual delete-and-recreate workflow.
5. Given a tone's start position is dragged across the tuplet boundary, when the edit is committed, then ownership moves to the context where the tone now starts, including tuplet-to-track and track-to-tuplet transfer as applicable, and the tone still remains exactly one stored event.
6. Given an edit would create unsupported overlap or otherwise violate MVP rules, when the composer attempts that edit, then the product rejects the unsupported outcome explicitly and it does not silently convert the passage into approximate non-tuplet timing.
7. Given the user attempts an unsupported tuplet edit in the sequencer, when the edit is rejected, then the product follows the existing product rejection behavior without introducing tuplet-specific error messaging and the tuplet region remains visually stable enough for the user to recover without losing orientation.
8. Given a tuplet sits next to non-tuplet material, when a supported move or resize occurs near the boundary, then ownership across the boundary remains explicit and hit targeting and selection remain stable.
9. Given a tone already exists before a tuplet is created and the composer later extends it across the tuplet area, when the resize completes, then the resulting host and tuplet segments preserve one coherent musical intent across the boundary and the tuplet's visible inner beats and boundary remain legible during and after the edit.
10. Given an edit would require nested tuplet structure to remain musically valid, when the composer attempts that edit, then MVP rejects or defers the edit rather than creating nested tuplets implicitly.
11. Given tuplet drag and resize behavior changes documented editing semantics or developer guardrails, when the story is completed, then the relevant docs and agent instructions are updated in the same change, or the completion notes explicitly record why no documentation update was required.
12. Given the active snap duration is a literal written value such as an eighth note, when the composer drags or resizes a tone across a tuplet boundary, then snapping continues to use that same literal written value in each context, using tuplet-local eighths inside the tuplet and track eighths outside it.
13. Given the sequencer contains tuplets, when the composer edits notes in this story, then only tone editing is supported and the tuplet container itself is not resized.
14. Given the app has no multi-tone editing support yet, when Story 2.4 is implemented, then all gesture handling remains single-tone only and no grouped move or resize behavior is introduced.

## Tasks / Subtasks

- [ ] Reuse the existing select-tool drag seams for tuplet-owned tones instead of inventing a tuplet-only gesture stack (AC: 1, 2, 3, 8, 10)
  - [ ] Keep all gesture handling single-tone only; do not introduce grouped selection or multi-tone drag logic.
  - [ ] Extend `src/windows/main/components/sequencer-track.tsx` so `handleSelectTool(...)` no longer exits early for `clickTarget.tuplet_id !== undefined` and tuplet-owned tones can enter the same drag pipeline as host-track tones.
  - [ ] Reuse `handlePositionDragging(...)`, `handleDurationAdjustment(...)`, and `handleLeftSideAdjustment(...)` for tuplet-owned tones, threading the known `tuplet_id` and owning `Tuplet` context into bounds and commit logic rather than branching to a separate tuplet drag implementation.
  - [ ] Reuse `getToneEntryFromClickTarget(...)` and `getTargetTupletFromClickTarget(...)` to resolve tuplet-owned tones and their container context from existing DOM click targets.
- [ ] Add tuplet-local bounds enforcement to shared movement and resize operations (AC: 1, 4, 5, 6, 8, 9, 10, 12, 13)
  - [ ] Reuse `src/windows/main/components/sequencer-tuplet-utils.ts` helpers such as `getTupletLocalTickFromClientX(...)` and `getTupletHostTicks(...)` to derive snapped tuplet-local positions and visible host-span limits during drag.
  - [ ] Update the shared frontend drag math so tuplet-owned tones clamp against the owning tuplet's local frame instead of only `flow.length`, while still allowing supported cross-boundary outcomes when the tone start moves into the other owning context.
  - [ ] Extend the Rust-owned mutation path in `src-tauri/src/score/entries/tone.rs` and any required shared track helpers in `src-tauri/src/score/tracks.rs` so move and resize commits reject invalid tuplet-local outcomes at the backend boundary instead of trusting the frontend to stay in bounds.
  - [ ] Preserve the rule that crossing the tuplet boundary does not force a storage split; resize and start-position edits may cross the boundary while still yielding one stored tone owned by where it starts.
  - [ ] Implement ownership transfer in both directions when the committed start position crosses the tuplet boundary: child-to-host when the start leaves the tuplet, host-to-child when the start enters it.
  - [ ] Keep snapping literal across the boundary so a single snap choice maps to track durations outside tuplets and the equivalent tuplet-local durations inside them.
- [ ] Preserve ordinary drag semantics while keeping tuplet membership backend-owned (AC: 2, 3, 4, 5, 6)
  - [ ] Keep pitch drag behavior on the existing `actions.score.tone.setPitch(...)` path with `tuplet_id` context, using the same pitch-targeting rules and optional audition behavior as ordinary tones.
  - [ ] Extend `actions.score.tone.shift(...)`, `actions.score.tone.setDuration(...)`, and `actions.score.tone.setPositionAndDuration(...)` or their backend command contracts only as much as necessary to carry tuplet-aware validation; do not add a parallel tuplet-only action family if the shared seams can be refined.
  - [ ] Preserve `tone_id` as the authoritative tone identity and `ui.flow_id` as the only authoritative active-flow slot; do not introduce parallel frontend-owned active-tuplet or active-flow drag state.
  - [ ] Keep tuplets themselves non-resizable; all resize work in this story applies only to tones.
- [ ] Keep sequencer interaction and visual recovery stable for accepted and rejected tuplet edits (AC: 6, 7, 8, 9)
  - [ ] Ensure rejected edits leave the tuplet region, slot markers, boundary lines, and tone selection stable enough that the user can continue working without losing orientation.
  - [ ] Preserve the existing bounded-region rendering in `src/windows/main/components/sequencer-track-tuplet.tsx`; any visual work here should support boundary legibility during drag, not redesign the tuplet presentation model.
  - [ ] Keep empty tuplet space as interaction context, not a selectable synthetic rest or separate edit target.
- [ ] Add focused regression coverage for tuplet move, re-pitch, and resize behavior (AC: 1, 2, 3, 4, 5, 6, 8, 9, 10)
  - [ ] Extend `src/windows/main/components/sequencer-track.test.tsx` to cover dragging a tuplet-owned tone horizontally within bounds, re-pitching it during drag, right-edge resize inside bounds, left-edge resize inside bounds, and rejection when a drag would violate supported MVP bounds.
  - [ ] Extend `src/services/sequencer-utils.test.ts` only if click-target or tuplet-context extraction changes.
  - [ ] Extend backend tests in `src-tauri/src/score/entries/tone.rs` and `src-tauri/src/score/tracks.rs` to cover tuplet-owned `shift`, `setDuration`, and `setPositionAndDuration` validation, by-tick consistency, cross-boundary ownership changes, and rejection of unsupported local-frame outcomes.
  - [ ] Add or update focused fixtures under `tests/fixtures/source/` only if score/notation rendering regressions need representative drag or resize scenarios; if fixture JSON changes, run `npm run build:fixtures`.
- [ ] Validate the touched seams with the standard repo checks
  - [ ] Run the relevant frontend tests covering sequencer tuplet drag and resize behavior.
  - [ ] Run the relevant Rust tests covering tuplet-aware tone movement and duration updates.
  - [ ] Run `npm run lint` and `npm run check-types` after implementation.
  - [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml` if Rust tone or track mutation seams change.
  - [ ] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` if Rust files change, because clippy-clean is a repo rule.
- [ ] Review documentation impact before closing the story (AC: 11)
  - [ ] Confirm whether the shipped drag and resize rules change documented tuplet editing semantics, rejection boundaries, or developer guardrails.
  - [ ] Update the relevant docs and agent instructions in the same change if the contract changed materially.
  - [ ] If no docs change is needed, record that explicit decision in the completion notes.

## Dev Notes

### Story Foundation

- Epic 2 is about editing tuplets without breaking musical intent. Story 2.4 is the gesture-editing story after targeting, pitch parity, and slot add/remove groundwork.
- Story 2.1 established reliable targeting of tuplet regions and child tones.
- Story 2.2 established that tuplet-owned pitch changes should flow through the same shared tone seams as ordinary tones.
- Story 2.3 established that note add/remove inside tuplets should reuse shared sequencer and Rust entry-management seams with explicit `tuplet_id` context.
- Story 2.4 should extend the same philosophy to drag editing: shared gesture semantics, backend-owned truth, and tuplet-local validation rather than a parallel tuplet editing mode.

### Technical Requirements

- Reuse the existing frontend drag handlers in `src/windows/main/components/sequencer-track.tsx`:
  - `handlePositionDragging(...)`
  - `handleDurationAdjustment(...)`
  - `handleLeftSideAdjustment(...)`
  - `handleSelectTool(...)`
- Reuse the existing tuplet helpers in `src/windows/main/components/sequencer-tuplet-utils.ts`:
  - `getTupletLocalTickFromClientX(...)`
  - `getTupletHostTicks(...)`
  - `getTupletRegion(...)`
  - `isTickInsideTupletRegion(...)`
- Reuse the existing frontend action wrappers in `src/services/actions.ts`:
  - `actions.score.tone.shift(track_id, entry_id, new_tick)`
  - `actions.score.tone.setDuration(track_id, tone_id, duration)`
  - `actions.score.tone.setPositionAndDuration(track_id, tone_id, tick, duration)`
  - `actions.score.tone.setPitch(track_id, tuplet_id, tone_id, pitch_int, accidental?)`
- Reuse and refine the existing backend tone seams in `src-tauri/src/score/entries/tone.rs`:
  - `Track::set_tone_duration(...)`
  - `Track::set_tone_position_and_duration(...)`
  - `score_entries_tone_duration_set(...)`
  - `score_entries_tone_set_position_and_duration(...)`
  - `score_entries_tone_shift(...)`
  - `resolve_tuplet_pitch_tick(...)`
- Reuse shared track indexing logic in `src-tauri/src/score/tracks.rs`, but add backend validation where tuplet-owned movement would otherwise escape or invalidate the local frame.
- Preserve exact timing and one-stored-event-per-tone ownership. A drag may cross a tuplet boundary, but the committed result must still be one stored tone whose ownership is determined by where it starts.
- Preserve literal duration semantics across the boundary. An eighth-note snap remains an eighth-note choice whether the edited segment is inside or outside the tuplet; only the projection into performed host time changes.
- Preserve MVP non-nestable rules. Any drag result that would require nested tuplets or nested repair must be rejected or deferred.

### Architecture Compliance

- Rust remains the source of truth for accepted tone edits. Frontend drag math may preview or clamp, but final validity must be enforced in backend-owned mutation code.
- Tuplets are local metric frames with child event streams. Validation must respect the tuplet's local frame rules rather than flattening everything to host-only heuristics.
- `tone_id` remains authoritative identity. `tuplet_id` is lookup and validation context, not a second identity model.
- Crossing a tuplet boundary does not itself force storage splitting. Rendering and playback remain responsible for derived boundary fragments.
- Host-track tones remain owned by where they start, even when a resize extends them through a tuplet span. Tuplet-owned tones remain child entries while their start stays inside the tuplet. When the start crosses the boundary, ownership must move to the new owning context without splitting the tone into multiple stored events.
- Snap duration is literal rather than context-specific in meaning. The same chosen written duration must project through either host or tuplet timing depending on where the segment is being edited.
- `ui.flow_id` remains the one authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport-facing consumers.

### Worked Example

- Example: a tone starts on the second tuplet eighth of an eighth-note triplet and is given a duration of four eighths.
- Storage: because the tone starts inside the tuplet, it is stored as one tuplet-owned child tone, not split into separate persisted host and tuplet tones.
- Duration meaning: the four eighths remain literal written duration intent. Two of those eighths fall inside the remaining tuplet span and the remaining two continue in ordinary track time after the tuplet boundary.
- Ownership: the tone stays tuplet-owned because its start remains inside the tuplet. If a later edit moved the start outside the tuplet, ownership would transfer to the host track while the tone still remained one stored event.
- Render/playback result: the boundary split is derived rather than persisted. The user should experience one continuous tone that begins at the second tuplet eighth, extends through the rest of the tuplet, and then continues for two ordinary eighths beyond the tuplet boundary.
- Sequencer expectation: no extra tuplet-specific visual model is required beyond keeping the tuplet boundary and inner beats legible while the crossing tone remains visibly continuous across that boundary.

### Library / Framework Requirements

- Frontend work stays in the current React 19 + TypeScript + Tauri invoke stack.
- Backend mutation authority remains Rust-owned state behind existing Tauri commands.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary frontend seams:
  - `src/windows/main/components/sequencer-track.tsx`
  - `src/windows/main/components/sequencer-track-tuplet.tsx`
  - `src/windows/main/components/sequencer-tuplet-utils.ts`
  - `src/services/actions.ts`
  - `src/services/sequencer-utils.ts` only if click-target plumbing changes
- Primary backend seams:
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/score/tracks.rs`
  - `src-tauri/src/score/entries/tuplet.rs` only if helper methods or tests need extension
- Existing tests to extend before adding new harnesses:
  - `src/windows/main/components/sequencer-track.test.tsx`
  - `src/services/sequencer-utils.test.ts`
  - backend tests in `src-tauri/src/score/entries/tone.rs`
  - backend tests in `src-tauri/src/score/tracks.rs`

### Testing Requirements

- Verify dragging a tuplet-owned tone horizontally within supported bounds updates its timing through exact tuplet-local rules.
- Verify vertical drag re-pitches a tuplet-owned tone using the same pitch-targeting rules and optional audition behavior as ordinary tones.
- Verify right-edge and left-edge resize of a tuplet-owned tone stay within supported local bounds unless the accepted outcome explicitly crosses the tuplet boundary under the one-stored-event rule.
- Verify crossing-boundary resize preserves one stored tone and leaves boundary splitting to derived render/playback behavior.
- Verify dragging a tone's start across the tuplet boundary moves ownership to the new start context in both directions while preserving one stored event.
- Verify literal snap duration remains stable across the boundary, using tuplet-local values inside the tuplet and track values outside it.
- Verify unsupported overlap, invalid local-frame outcomes, and nested-tuplet-requiring edits are rejected without approximate timing fallbacks.
- Verify the tuplet region, slot markers, hit targeting, and selection remain stable after rejected edits.
- Verify by-tick and by-id indexes remain consistent after accepted tuplet drag and resize operations.
- Verify tuplet containers themselves are not resizable through these gestures.
- Verify no multi-tone editing path is added or enabled.
- Undo/redo is not an acceptance requirement for this story because the app does not yet provide that infrastructure.

### Established Code Patterns To Follow

- Extend shared seams before adding new ones. Recent tuplet work in Stories 2.2 and 2.3 reused existing tone mutation and sequencer-targeting seams with explicit `tuplet_id` context.
- Treat tuplet membership as nested storage context, not a product-level alternate edit mode.
- Keep validation in Rust even if the frontend also clamps obvious invalid drags.
- Keep the visual tuplet region component focused on presentation and stability. Do not shift score truth or gesture state into the UI layer.
- Keep Story 2.4 tightly scoped to single-tone drag editing. Do not pull container delete, slice policy, tuplet-container resizing, or generalized multi-note editing into this story unless the epic source is revised.

### Previous Story Intelligence

- Story 2.1 solved reliable targeting for tuplet members and clarified that click resolution should return explicit `tuplet_id` context when a tone lives inside a tuplet.
- Story 2.2 solved tuplet-owned pitch parity by threading `tuplet_id` through shared tone pitch seams instead of adding a tuplet-only mutation API.
- Story 2.3 solved tuplet slot add/remove behavior by reusing the shared sequencer/action/Rust seams and preserving empty-slot rendering as visual context rather than persisted rest data.
- The same pattern should govern Story 2.4: do not invent a separate tuplet gesture model when the existing select-tool handlers and tone commands can be refined.

### Git Intelligence Summary

- Recent relevant commits:
  - `96cdae7d feat(2.3): basic tone editing`
  - `802f5e64 fix: no tuple highlights`
  - `6cd8bef8 feat(2.2): finish tuplet pitch editing parity`
  - `c8ea0449 feat(2.1): tuple tone selection info`
- The recent pattern is incremental seam extension in the sequencer and tone mutation layers, backed by focused regressions rather than new subsystem creation.
- Current code inspection shows the frontend already has generic drag handlers and the backend already has shared tone movement APIs, but tuplet-owned select-tool dragging is currently blocked early and backend move/resize validation is still host-track-centric.

### Documentation Impact

- This story may change documented tuplet editing semantics and should trigger a documentation impact review.
- Review these docs during implementation and update them if the shipped contract changes materially:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if any durable guardrail changes
  - `tests/fixtures/README.md` if new tuplets-related visual regression fixtures are introduced
- If implementation only fulfills already-documented behavior, record that no documentation update was required in the completion notes rather than skipping the review silently.

### Project Structure Notes

- The current `handleSelectTool(...)` implementation in `src/windows/main/components/sequencer-track.tsx` selects tuplet-owned tones but returns before entering drag logic. Story 2.4 should remove that limitation without regressing ordinary tone interactions.
- Current shared movement commands (`shift`, `setDuration`, `setPositionAndDuration`) do not carry explicit tuplet-aware validation. Backend refinement is the root-cause seam for safe tuplet drag support.
- `SequencerTrackTuplet` already renders a bounded region with slot markers using `data-tuplet-id`; that existing structure should remain the hit-target and orientation foundation during drag work.
- Main implementation risks:
  - allowing tuplet-owned tones to escape their local frame due to host-track-only clamping
  - corrupting ownership or indexes when a start drag crosses the tuplet boundary
  - introducing a second tuplet-only drag API instead of refining the shared seams
  - accidentally broadening scope into multi-note moves or Story 2.5 delete/slice rules

### Open Questions / Assumptions

- Clarified: ownership transfer is required in both directions when a tone start crosses the tuplet boundary.
- Clarified: snap duration is literal across the boundary. Inside the tuplet, snapping uses tuplet-local equivalents of the active snap value; outside it uses track timing with the same written duration meaning.
- Clarified: Story 2.4 is single-tone only. No grouped movement or resize behavior should be added.
- Clarified: tuplets themselves are not resizable.
- Assumption retained: pitch drag should behave identically to ordinary tones because Story 2.2 already established pitch-parity through shared seams.
- Assumption retained: unsupported edits should reuse the current local rejection behavior with no new tuplet-specific message chrome.
- Clarified: the intended visual result for tones whose duration extends in or out of a tuplet is the existing architecture rule of one stored tone plus derived boundary split in render/playback, with the tuplet boundary and inner beats remaining legible while the tone reads as continuous.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 / Story 2.4 acceptance criteria and Story 2.5 boundary
- Source: `_bmad-output/planning-artifacts/prd.md` — editing semantics, exact timing, one-stored-event rule, boundary crossing, and non-nestable MVP behavior
- Source: `_bmad-output/planning-artifacts/architecture.md` — local metric frame model, ownership-by-start rules, derived boundary splitting, and sequencer UX contract
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — sequencer-first tuplet editing, bounded-region presentation, slot-marker visibility, and rejection/recovery expectations
- Source: `docs/project-context.md` — Rust-owned truth, shared action seams, `ui.flow_id` guardrail, and score-render architecture
- Source: `_bmad-output/implementation-artifacts/2-3-add-and-remove-notes-within-tuplet-slots.md` — previous story seam-reuse pattern and scope discipline
- Source: recent code inspection of `src/windows/main/components/sequencer-track.tsx`, `src/windows/main/components/sequencer-track-tuplet.tsx`, `src/windows/main/components/sequencer-tuplet-utils.ts`, `src/services/actions.ts`, `src/services/sequencer-utils.ts`, `src-tauri/src/score/entries/tone.rs`, and `src-tauri/src/score/tracks.rs`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 2 Story 2.4 acceptance criteria and adjacent story boundaries in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed PRD guidance for exact timing, boundary-crossing edits, one-stored-event ownership, and non-nestable MVP rules in `_bmad-output/planning-artifacts/prd.md`.
- Reviewed architecture guidance for local metric frames, ownership-by-start, derived boundary splits, and sequencer obligations in `_bmad-output/planning-artifacts/architecture.md`.
- Reviewed UX guidance for sequencer-first tuplet editing, bounded-region presentation, slot markers, and rejection stability in `_bmad-output/planning-artifacts/ux-design-specification.md`.
- Reviewed Story 2.3 implementation guidance to preserve seam-reuse and scope discipline.
- Inspected current drag/edit seams in `src/windows/main/components/sequencer-track.tsx`, `src/windows/main/components/sequencer-track-tuplet.tsx`, `src/windows/main/components/sequencer-tuplet-utils.ts`, `src/services/actions.ts`, `src/services/sequencer-utils.ts`, `src-tauri/src/score/entries/tone.rs`, and `src-tauri/src/score/tracks.rs`.
- Reviewed recent git history to align the story with the current incremental tuplet implementation pattern.
- Captured and then resolved product decisions around ownership transfer, literal snap duration, non-resizable tuplets, single-tone-only editing, and the intended visual result for boundary-crossing durations.

### Completion Notes List

- Story 2.4 is intentionally scoped to single-tone drag editing inside the existing sequencer-first workflow.
- Story 2.5 retains explicit delete and slice policy work.
- The current codebase already contains most shared gesture and tone-mutation seams required for Story 2.4, but tuplet-owned drag entry is currently blocked in the frontend and bounds validation is still insufficient at the backend mutation layer.
- Documentation impact review is required before the story can be closed.

### File List

- `_bmad-output/implementation-artifacts/2-4-reposition-or-resize-tuplet-notes-within-supported-bounds.md`

### Change Log

- 2026-03-16: Created Story 2.4 implementation artifact with scope boundaries, shared-seam guidance, backend validation requirements, testing expectations, documentation impact tasks, and surfaced open questions.
