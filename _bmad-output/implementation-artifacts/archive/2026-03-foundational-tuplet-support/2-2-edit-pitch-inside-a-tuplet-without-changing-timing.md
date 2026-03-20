# Story 2.2: Edit Pitch Inside a Tuplet Without Changing Timing

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want to edit the pitch of notes inside a tuplet using the same tone-manipulation behaviors available for ordinary track tones,
so that tuplet membership does not create a separate editing model or disturb rhythmic intent.

## Scope Note

- This story is about pitch-manipulation parity for a selected tuplet-owned tone. Tuplet membership is an implementation detail, not a product-level behavioral fork.
- Supported edits in this story are the existing track-level pitch actions that do not change tick or duration: direct pitch set, diatonic step shift, and octave shift.
- Explicit accidental editing is out of scope for this story. Pitch changes should continue to infer accidental spelling from key signature context the same way ordinary track tones do unless a caller already passes an explicit accidental through an existing shared contract.
- Duration, resize, reposition, drag-to-re-pitch, slice, and ownership-transfer behavior remain later-story scope. Story 2.4 continues to own drag gestures and other mutations that change timing, ownership context, or pitch through pointer movement.
- Selection source is not the product boundary for this story. If a tone is already selected and a supported shared pitch action is invoked, the tuplet-owned tone should mutate through the same command path as an ordinary track tone.
- MVP tuplets remain single-level only. This story must not introduce new nesting concepts, tuplet-specific pitch APIs, or parallel frontend-owned active-tuplet state.

## Acceptance Criteria

1. Given a tuplet-owned tone is selected, when the composer applies a supported pitch edit that ordinary track tones already support, then the selected tone's pitch updates successfully and no other tone attributes change.
2. Given a tuplet-owned tone is selected, when its pitch changes, then its `tone_id`, tick, duration, tuplet ownership, and selection remain unchanged.
3. Given a key signature is active at the selected tone's musical context, when the composer changes the pitch of a tuplet-owned tone without requesting an explicit accidental override, then the stored pitch spelling is resolved using the same key-signature-aware rules used for ordinary track tones.
4. Given adjacent non-tuplet material surrounds the tuplet, when a pitch edit is applied to a tuplet-owned tone, then surrounding host-track tones and container ownership remain unchanged and no boundary-side effects are introduced.
5. Given a tone starts inside a tuplet and extends beyond the tuplet boundary, when the composer edits only its pitch, then the tone remains one persisted event owned by its original start position and is not split, moved, or re-owned.
6. Given a tone starts outside a tuplet and extends through a tuplet boundary, when the composer edits only its pitch through the existing shared tone-manipulation path, then the tone remains one persisted host-owned event and pitch mutation does not introduce boundary-driven splitting or ownership changes.
7. Given a tuplet-owned tone is selected, when a supported pitch edit completes, then selection-driven UI surfaces update the edited tone exactly as they do for ordinary track tones, including selected-tone summaries and any pitch-dependent feedback already driven from shared tone selection.
8. Given a supported pitch edit is invoked for a selected tuplet-owned tone, when the command is dispatched, then the implementation reuses the existing shared tone-mutation actions and backend tone commands rather than introducing a tuplet-specific pitch API or parallel state path.
9. Given a supported pitch edit is applied to a selected tuplet-owned tone on the active flow, when application state refreshes, then `ui.flow_id` remains the authoritative active-flow slot and no extra frontend-owned tuplet flow state is introduced.

## Tasks / Subtasks

- [x] Reuse the existing shared tone pitch-mutation command path for tuplet-owned tones (AC: 1, 2, 8)
  - [x] Inspect and extend the existing backend tone pitch commands in `src-tauri/src/score/entries/tone.rs` so `set_tone_pitch`, `shift_tone_pitch_by_steps`, and `shift_tone_pitch_by_octaves` can resolve a selected tuplet-owned tone through the same tone identity contract used for ordinary track tones.
  - [x] Preserve `tone_id` as the mutation authority and treat tuplet location as lookup context rather than introducing a second identity model.
  - [x] Reuse the existing frontend action wrappers in `src/services/actions.ts` rather than adding tuplet-specific invoke calls.
- [x] Keep pitch edits isolated from timing and ownership state (AC: 1, 2, 4, 5, 6, 9)
  - [x] Ensure pitch-only mutation does not alter tick, duration, local-versus-host ownership, or `ui.flow_id`.
  - [x] Verify boundary-crossing tones remain single persisted tones owned by their start position after pitch edits.
  - [x] Do not pull position or duration mutation behavior forward from Story 2.4.
- [x] Preserve track-level pitch semantics and key-signature behavior for tuplet-owned tones (AC: 3, 7)
  - [x] Keep key-signature-aware pitch inference aligned with the existing ordinary-track `set_tone_pitch` behavior.
  - [x] Ensure selected-tone summary surfaces continue to reflect the edited tuplet-owned tone through the shared selected-tone contract.
  - [x] Support the same currently shipped pitch-edit triggers for a selected tone, except for explicit accidental-only editing which remains separate scope.
- [x] Add focused regression coverage for tuplet-owned pitch mutation parity (AC: 1, 2, 3, 4, 5, 6, 7, 8, 9)
  - [x] Extend backend tests in `src-tauri/src/score/entries/tone.rs` to cover pitch mutation for tuplet-owned tones, including key-signature inference and boundary-crossing ownership stability.
  - [x] Extend `src/services/actions.test.ts` only if the shared pitch-action contract changes.
  - [x] Extend `src/windows/main/hooks/use-keyboard-shortcuts.test.ts` for selected tuplet-owned tones if keyboard pitch shortcuts are part of the current shared track-tone editing path.
  - [x] Extend `src/windows/main/hooks/use-selected-tone-info.test.ts` and `src/windows/main/components/selected-tone-info-display.test.tsx` only if pitch-edit refresh behavior needs explicit regression coverage.
  - [x] Extend `src/windows/main/components/sequencer-track.test.tsx` only if an existing pitch-editing surface for selected tones needs tuplet-specific parity coverage without introducing position changes.
- [x] Validate the touched seams with the standard repo checks
  - [x] Run the relevant frontend and Rust tests covering shared tone pitch mutation and tuplet-owned tone selection metadata.
  - [x] Run `npm run lint` and `npm run check-types` after implementation.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` if any Rust tone-mutation or selection-resolution seams change.

## Dev Notes

### Story Foundation

- Story 2.1 established reliable selection of tuplet-owned tones and confirmed that tuplet child notes must behave like ordinary selected tones in shared UI surfaces.
- Story 2.2 builds directly on that seam: once a tuplet-owned tone is selected, pitch mutation should behave like track-level tone mutation rather than a special tuplet workflow.
- The product rule for this story is simple and strict: changing pitch must not imply a timing edit, ownership move, or tuplet-specific editing mode.

### Technical Requirements

- Reuse the existing backend tone pitch operations in `src-tauri/src/score/entries/tone.rs`:
  - `set_tone_pitch`
  - `shift_tone_pitch_by_steps`
  - `shift_tone_pitch_by_octaves`
- Reuse the existing frontend action wrappers in `src/services/actions.ts`:
  - `actions.score.tone.setPitch(...)`
  - `actions.score.tone.shiftPitchByStep(...)`
  - `actions.score.tone.shiftPitchByOctave(...)`
- Keep pitch-edit semantics aligned with current track-level tone behavior:
  - `tone_id` remains the mutation target, with optional `tuplet_id` carried as direct nested lookup context when selection already knows the owning tuplet
  - key signature context resolves pitch spelling when no explicit accidental override is requested
  - pitch mutation alone must not change tick, duration, or ownership context
- Explicit accidental-only editing is separate scope. Do not expand this story into accidental editing just because the app already has related controls.
- Do not treat selection source as the main abstraction. Whether a tone was selected from the sequencer or another existing workflow surface is irrelevant once the shared tone action path is invoked.
- Do not include mixed gestures that mutate pitch through pointer movement in this story, whether they also change tick or duration or only re-pitch vertically. Sequencer drag behavior remains Story 2.4 scope.

### Architecture Compliance

- The architecture requires one persisted tone event per tone, owned by the context where that tone starts.
- A tone that starts inside a tuplet remains tuplet-owned even if it crosses the boundary; a tone that starts outside remains host-owned even if it extends through the tuplet span.
- Crossing the tuplet boundary never by itself forces a storage split. Pitch mutation must preserve that rule.
- `tone_id` remains the selected tone identity, while optional `tuplet_id` provides the exact nested storage context needed to address that tone directly without backend owner scans.
- `ui.flow_id` remains the single authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport-facing consumers.
- MVP tuplets are single-level only. Do not add nested-tuplet handling or defensive branching for unsupported nesting models.

### Library / Framework Requirements

- Frontend work stays in the current React 19 + TypeScript + Tauri invoke stack.
- All FE mutations continue to flow through the `actions` object in `src/services/actions.ts`.
- Backend mutation authority remains Rust-owned state behind the existing Tauri command path.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/score/entries/tone.rs`
  - related Tauri command registration in `src-tauri/src/lib.rs` only if the shared command path changes
- Primary frontend seams:
  - `src/services/actions.ts`
  - `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - `src/windows/main/hooks/use-selected-tone-info.ts`
  - `src/windows/main/components/selected-tone-info-display.tsx`
  - `src/windows/main/components/sequencer-track.tsx` only if a current pitch-edit trigger for selected tones must be made tuplet-safe without changing position
- Existing tests to extend before adding new test surfaces:
  - `src/services/actions.test.ts`
  - `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
  - `src/windows/main/hooks/use-selected-tone-info.test.ts`
  - `src/windows/main/components/selected-tone-info-display.test.tsx`
  - backend tests in `src-tauri/src/score/entries/tone.rs`

### Testing Requirements

- Verify that tuplet-owned tones can use the same supported pitch-mutation actions as ordinary track tones.
- Verify that pitch-only edits preserve `tone_id`, tick, duration, ownership, and selection.
- Verify key-signature-aware accidental inference for tuplet-owned pitch edits where no explicit accidental override is supplied.
- Verify both boundary cases remain stable after pitch edits:
  - tone starts inside tuplet and extends out
  - tone starts outside tuplet and extends in
- Verify selected-tone summary surfaces continue to reflect the edited tuplet-owned tone without falling back to missing-selection behavior.
- Verify no tuplet-specific command path or frontend-owned active-tuplet editing state is introduced.
- Undo/redo is not an acceptance requirement for this story because the app does not yet provide that infrastructure.

### Established Code Patterns To Follow

- Reuse the existing shared tone action wrappers and Rust tone mutation commands before considering any new command surface.
- Preserve the repo rule that the frontend is a display layer over Rust-owned state.
- Extend existing tests in place rather than introducing a parallel tuplet-only pitch-edit harness.
- Keep tuplet-specific handling limited to the lookup and ownership seams actually required by nested child storage.
- Treat tuplet membership as implementation detail in the mutation path, not as a reason to create a separate user-facing edit model.

### Previous Story Intelligence

- Story 2.1 closed the targeting ambiguity problem and explicitly aligned tuplet-owned tone selection with the shared selected-tone summary contract.
- Story 2.1 also clarified that tuplets in the sequencer are visual-only context and that tone selection remains the primary interaction model.
- That means Story 2.2 should extend the existing selected-tone mutation path, not introduce a tuplet-focused edit mode or separate UI state.
- Earlier Epic 1 work already established the ownership rule that one stored tone remains one stored event and that start position determines host-versus-tuplet ownership.

### Git Intelligence Summary

- Recent relevant commits:
  - `c8ea0449 feat(2.1): tuple tone selection info`
  - `d4617be1 feat: epic 1 retro`
  - `4965b07a feat(1.7): validate tuplet file workflow contract`
- The recent pattern is narrow seam hardening with focused regression tests. Keep Story 2.2 equally tight: pitch parity only, no premature move/resize work.

### Documentation Impact

- If implementation changes the documented tone-mutation contract for tuplet-owned tones, update the relevant developer docs in the same change.
- Likely documentation touchpoints only if behavior or guardrails change materially:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
- If the implementation is pure seam reuse with no behavioral contract change beyond enabling parity, document review may conclude that no docs update is needed, but that decision should be recorded in the implementation summary.

### Project Structure Notes

- The repo already has shared action wrappers and keyboard-shortcut dispatch for pitch shifting on selected tones.
- The selected-tone summary contract is already shared between host-track and tuplet-owned tones after Story 2.1.
- The main implementation risk is accidentally treating tuplet-owned tones as second-class mutation targets and introducing a tuplet-only command path or identity model.
- The second major risk is accidental scope creep into duration or position editing because some current UI gestures combine vertical and horizontal motion. This story should not absorb that work.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 / Story 2.2 and Story 2.4 scope boundary
- Source: `_bmad-output/planning-artifacts/prd.md` — editing semantics, one-tone persistence, and boundary-crossing requirements
- Source: `_bmad-output/planning-artifacts/architecture.md` — ownership by start position, `tone_id` authority, and boundary rules
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — selection and editing feedback expectations
- Source: `docs/project-context.md` — Rust-owned state, shared `actions.ts` mutation path, and `ui.flow_id` guardrails
- Source: `_bmad-output/implementation-artifacts/2-1-target-tuplet-containers-and-members-reliably.md` — prior story learnings and shared selected-tone contract

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Verified Story 2.2 placeholder requirements and Story 2.4 boundary in `_bmad-output/planning-artifacts/epics.md`.
- Verified PRD and architecture rules for one-event persistence, ownership by start position, and boundary-crossing stability in `_bmad-output/planning-artifacts/prd.md` and `_bmad-output/planning-artifacts/architecture.md`.
- Verified existing shared pitch-mutation seams in `src-tauri/src/score/entries/tone.rs`, `src/services/actions.ts`, and `src/windows/main/hooks/use-keyboard-shortcuts.ts`.
- Verified recent story pattern from git history: tight seam reuse with focused regression coverage.
- Added red-phase regressions for tuplet-owned pitch mutation in `src-tauri/src/score/entries/tone.rs` and keyboard pitch shortcut parity in `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`.
- Validated the implementation with `use-keyboard-shortcuts.test.ts`, `use-selected-tone-info.test.ts`, `npm run lint`, `npm run check-types`, and `cargo test --manifest-path src-tauri/Cargo.toml`.
- Revisited the implementation to remove backend owner scanning and instead thread optional `tuplet_id` context through the shared tone pitch action and command path.
- Collapsed the keyboard shortcut hook back to a single nested-aware tone selection path and extended the same optional `tuplet_id` contract to shared remove, accidental, and articulation actions so nested tones do not require a separate branch.

### Implementation Plan

- Thread optional `tuplet_id` context through `src/services/actions.ts`, the Tauri tone pitch commands, and the known tone-pitch callers so the backend does not have to search for ownership.
- Keep key-signature lookup projected to host musical time for tuplet-owned tones so pitch spelling stays aligned with ordinary track-tone rules.
- Collapse duplicated frontend tone-selection branches once the shared tone action contract can carry optional `tuplet_id` for all applicable selected-tone shortcuts.

### Completion Notes List

- Story 2.2 is intentionally narrowed to pitch-parity behavior only.
- Duration, reposition, resize, and drag-to-re-pitch behavior stay in Story 2.4 to avoid scope overlap.
- Explicit accidental editing remains separate from this story even though related controls exist elsewhere in the app.
- The core rule for implementation is that tuplet membership is a nested-storage concern, not a distinct tone-editing product model.
- Replaced the backend owner-scan helper with explicit optional `tuplet_id` context on `set_tone_pitch`, `shift_tone_pitch_by_steps`, and `shift_tone_pitch_by_octaves`.
- Synced the automation registry action metadata with the shipped optional `tuplet_id` contract so automation callers and docs match the executable handlers.
- Updated the shared tone action wrappers, automation registry, keyboard shortcuts, and sequencer pitch caller to pass known tuplet context instead of reconstructing it later.
- Removed the separate keyboard shortcut selection branch by unifying selected-tone lookup on `getSelectedTone(...)` plus optional `tuplet_id`.
- Extended nested parity coverage to the shared selected-tone shortcuts for delete, accidental, and articulation so the same tone-selection contract applies consistently beyond pitch-shift commands.
- Accepted contract clarification: `tone_id` stays the selected tone identity, and optional `tuplet_id` is required direct lookup context for nested tones rather than a backend-discovered fallback.
- Added Rust regressions for tuplet-owned pitch edits, accidental updates, removal, articulation propagation, host-owned boundary-crossing stability, and key-signature inference; added hook regressions for selected tuplet-owned keyboard editing parity.
- Documentation review result: no docs update required because the shipped contract stayed as shared seam reuse rather than a new behavior surface.

### File List

- `_bmad-output/implementation-artifacts/2-2-edit-pitch-inside-a-tuplet-without-changing-timing.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/score/entries/tone.rs`
- `src-tauri/src/score/entries/tuplet.rs`
- `src-tauri/src/automation/registry.rs`
- `src/services/actions.ts`
- `src/services/actions.test.ts`
- `src/windows/main/components/sequencer-track.tsx`
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
- `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`

### Change Log

- 2026-03-16: Enabled shared pitch mutation parity for tuplet-owned tones, added focused backend and hook regressions, and validated with frontend lint/typecheck plus full Rust tests.
- 2026-03-16: Reworked the implementation to use one nested-aware selected-tone path with optional `tuplet_id`, extending the shared remove, accidental, and articulation shortcuts so tuplet-owned tones no longer require special frontend branches.
