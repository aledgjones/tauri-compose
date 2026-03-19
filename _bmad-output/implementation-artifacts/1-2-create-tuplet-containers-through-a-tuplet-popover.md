# Story 1.2: Create Tuplet Containers Through a Tuplet Popover

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want to create a triplet from a focused popover in the existing workflow,
so that I can declare the tuplet structure before entering notes.

## Acceptance Criteria

1. Given a supported note-entry context on an instrument track, when a composer opens the tuplet popover and confirms a supported triplet shape, then the product creates a tuplet container for the targeted host-track span, and the container is inserted as backend-owned score data rather than a temporary UI annotation.
2. Given the composer invokes tuplet creation from the primary note-entry workflow, when the MVP popover opens, then the hotkey trigger is `;`, and the popover opens at the current selection tick.
3. Given the MVP tuplet popover is shown, when the composer configures a supported triplet, then the main input uses an explicit learned syntax such as `3e/2` to encode both the triplet ratio and the written base note in one text field.
4. Given the composer enters an invalid popover value, when the popover is submitted, then the invalid value is ignored using the same pattern as other current popovers, and the product does not introduce a special tuplet-only invalid-input flow for MVP.
5. Given the popover is opened from keyboard or score-attached interaction, when it becomes active, then focus lands on the ratio input first, and the popover closes cleanly on accepted input, blur, or Escape.
6. Given the composer invokes tuplet creation from the primary note-entry workflow, when the current selection is not a tone or rest with direct instrument-track ownership data, then tuplet confirmation does not create a container, and the product does not guess a fallback target from unrelated UI context.

## Tasks / Subtasks

- [x] Add a tuplet popover entry point that matches the existing popover architecture (AC: 2, 3, 5)
  - [x] Extend `PopoverType` in `src/services/popovers.ts` and `src/ui/popover.tsx` with a tuplet option rather than creating a separate overlay mechanism.
  - [x] Add the `;` keyboard trigger in `src/ui/popover.tsx` using the same document-level selection-coordinate lookup used by the existing directive popovers.
  - [x] Keep popover lifecycle behavior consistent with the current implementation: autofocus on the primary text input, close on escape or blur, and avoid opening a second popover while one is already active.
- [x] Implement tuplet popover parsing and input behavior for MVP triplet creation (AC: 2, 3, 4)
  - [x] Add a `parseTupletPopover` helper in `src/services/popovers.ts` that accepts only supported MVP triplet input and maps it to a backend create action.
  - [x] Model the UI around one explicit tuplet text input such as `3e/2`, so the ratio and written base note are entered together without a secondary picker control.
  - [x] Reuse the current invalid-input pattern from other popovers: reject bad terms in the parser, catch them in `src/ui/popover.tsx`, log locally, and leave the wider app state unchanged.
- [x] Add the frontend-to-backend tuplet creation handshake with no frontend-owned rhythmic state (AC: 1, 3)
  - [x] Introduce a `score.tuplet.create(...)` action in `src/services/actions.ts` and corresponding tests in `src/services/actions.test.ts`.
  - [x] Extend the Rust Tauri command surface in `src-tauri/src/lib.rs` and the owning score mutation seam to create an `Entry::Tuplet` on the targeted track using the persisted ratio metadata added in Story 1.1.
  - [x] Resolve the target flow, instrument, track, tick, and host span from existing sequencer state and selection rather than inventing a parallel frontend tuplet draft model.
- [x] Add selection-state plumbing needed for popover handoff without pulling Story 1.3 work forward unnecessarily (AC: 1, 5)
  - [x] Extend the sequencer selection types in `src/services/types.ts` and `src-tauri/src/ui/sequencer.rs` to support selecting a tuplet container directly.
  - [x] Add `actions.ui.sequencer.selectTuplet(...)` and the matching Rust command so the creation flow can hand selection back to a concrete score/sequencer target after creation.
  - [x] Keep `ui.flow_id` as the authoritative active-flow slot by following the existing `apply_selection` pattern in `src-tauri/src/ui/sequencer.rs`.
- [x] Cover the new behavior with focused frontend and Rust tests close to the owning code (AC: 1, 2, 3, 4, 5, 6)
  - [x] Add parser tests in `src/services/popovers.test.ts` for valid triplet input, invalid input, and explicit suffix behavior.
  - [x] Add popover interaction tests in `src/ui/popover.test.tsx` for the `;` shortcut, autofocus, submit behavior, tone/rest-only tuplet confirmation, hotkey suppression inside text inputs, and close behavior.
  - [x] Add action/selection command tests in `src/services/actions.test.ts` and `src-tauri/src/ui/sequencer.rs` covering tuplet selection and flow handoff.
  - [x] Add Rust creation tests near the tuplet mutation seam proving container creation inserts `Entry::Tuplet` into the correct track with the expected ratio metadata and unchanged non-tuplet behavior.
- [x] Update docs and agent guidance touched by the new creation workflow (AC: 1, 5)
  - [x] Update `docs/data-models.md` or other relevant docs if container creation semantics, ownership rules, or selection payloads become more explicit during implementation.
  - [x] Update AI-agent instructions only if this story establishes a durable new project rule beyond the already documented tuplet model and `ui.flow_id` ownership rule.

## Dev Notes

### Story Foundation

- Epic 1 remains the backend-owned exact-timing foundation. Story 1.2 is the first user-facing seam for that work, but it still must treat Rust as authoritative score state.
- The scope here is container creation through a popover and the creation handshake back into sequencer context. Immediate region reveal, sticky `3:2` sequencer presentation, and visually distinct empty-slot rendering belong primarily to Story 1.3 and Story 1.4.
- This story must create real `Entry::Tuplet` data, not a temporary frontend draft annotation to be reconciled later.

### Technical Requirements

- Keep all authoritative tuplet data in Rust. The frontend may hold ephemeral popover UI state only; it must not own a second tuplet timing or container model.
- Reuse the persisted ratio-driven `Tuplet` shape from Story 1.1: `note_count`, `in_space_of`, `base_note_value`, `groupings`, and child `entries`. Do not reintroduce duplicate persisted host/local span fields in the UI layer.
- Resolve the creation target from the existing active flow, selected tick, sequencer instrument, and sequencer track. If the current context is insufficient, reject creation locally rather than guessing.
- Use the current selection tick as the tuplet start position, but only allow tuplet confirmation when the active selection is a tone or rest that already includes instrument, stave, and track ownership data.
- Preserve the current invalid-input pattern used by other popovers: parser rejects unknown input, `Popover` catches and logs the error, and the app does not enter a special tuplet-only validation flow.
- Keep `ui.flow_id` authoritative. Tuplet creation and selection may update it through the existing selection path, but this story must not add separate frontend-owned active-flow state.

### Architecture Compliance

- Story 1.1 already established `Entry::Tuplet(Box<Tuplet>)` plus the recursive `Tuplet.entries: Entries` model. Story 1.2 must build on that shape rather than adding an alternate creation-only representation.
- Use the existing Rust-owned selection contract as the model: `apply_selection` in `src-tauri/src/ui/sequencer.rs` sets `ui.flow_id` and sequencer selection together. Tuplet selection should follow that same pattern.
- The architecture and PRD both require creation from the existing workflow rather than a disconnected rhythm-authoring mode. Use the current popover stack and sequencer context instead of introducing a standalone tuplet window or tool mode.
- This story should stop at container creation and handoff. Do not pull note-entry-inside-tuplet logic into this implementation; that belongs to Story 1.4.

### Library / Framework Requirements

- Frontend code should stay within the existing React + TypeScript popover flow in `src/ui/popover.tsx` and the action layer in `src/services/actions.ts`.
- Backend changes should follow the existing Tauri command pattern: `#[tauri::command(rename_all = "snake_case")]`, mutate `AppContext`, and emit general state updates.
- Selection DTOs must stay in sync between Rust and TypeScript, using the existing tagged-union / tagged-enum patterns.
- External web research was not available in this workflow context. Use the repo-pinned stack and the existing project docs as the authoritative technical baseline.

### File Structure Requirements

- Expected frontend touchpoints:
  - `src/ui/popover.tsx`
  - `src/services/popovers.ts`
  - `src/services/popovers.test.ts`
  - `src/services/actions.ts`
  - `src/services/actions.test.ts`
  - `src/services/types.ts`
  - `src/ui/popover.test.tsx`
- Expected Rust/backend touchpoints:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/ui/sequencer.rs`
  - the owning tuplet creation/mutation seam under `src-tauri/src/score/`
  - `src-tauri/src/score/entries/tuplet.rs` if helper constructors or ratio-derived utilities are needed
- Keep changes surgical. Story 1.2 should not force broad renderer, parser, or playback updates beyond whatever is minimally required to keep tuplets selectable and state updates coherent.

### Testing Requirements

- Add frontend coverage for the new popover parser, keyboard trigger, and tone/rest-only tuplet confirmation behavior.
- Add React interaction coverage for focus entry, submit, invalid-input no-op behavior, text-input hotkey suppression, and close behavior.
- Add Rust coverage for tuplet creation and tuplet selection commands, including `ui.flow_id` updates and unchanged behavior for existing selection variants.
- Validate with the existing repo checks that are relevant to touched code:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 1.1 already added the persisted tuplet container entry, TypeScript DTO mirror, and docs updates. Reuse that work instead of re-specifying container shape in the frontend.
- The persisted tuplet representation was simplified to ratio metadata plus nested `Entries`; container creation in this story should continue to derive spans from ratio metadata instead of storing duplicate timing fields.
- Story 1.1 also updated project guidance to reflect that `Entry` now includes `Tuplet`. Do not reintroduce stale six-variant assumptions in new code or docs.

### Git Intelligence Summary

- Recent relevant commit: `10dc0067 feat(1.1): init tuplets`.
- Current implementation patterns show that new score operations are expected to flow through `src/services/actions.ts`, Tauri commands in `src-tauri/src/lib.rs`, and Rust-owned state mutation with emitted general-state updates.
- The repo currently centralizes keyboard edit shortcuts in `src/windows/main/hooks/use-keyboard-shortcuts.ts`, while directive popover triggers live in `src/ui/popover.tsx`. Keep that split coherent when adding the `;` shortcut.

### Documentation Impact

- If this story makes tuplet selection, popover triggers, or container-creation semantics user-visible enough to affect developer expectations, update the relevant docs in the same implementation.
- At minimum, keep `docs/data-models.md` and any selection-structure documentation accurate if a new tuplet selection payload is introduced.

### Project Structure Notes

- The current popover system is score-selection driven. It obtains coordinates from `actions.ui.renderer.getSelectionCoordinates()` and derives flow/tick from the existing sequencer selection. The tuplet popover should fit that model.
- The current `Selection` unions in both TypeScript and Rust do not yet include tuplets. Story 1.2 likely needs that addition to support post-create selection handoff cleanly.
- The current sequencer track UI only renders tones. That is acceptable for this story as long as container creation and selection plumbing land cleanly; visible tuplet-region rendering is handled by follow-on stories.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 / Story 1.2 / Requirements Inventory
- Source: `_bmad-output/planning-artifacts/prd.md` — Authoring and Editing Behavior; Data Model and Persistence Stakes; Functional Requirements FR1, FR3, FR6, FR11
- Source: `_bmad-output/planning-artifacts/architecture.md` — Recommended Persisted Data Model; Sequencer UX Contract; Focus, Feedback, And Accessibility Contract; Creation And Reconciliation Rules
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — Tuplet UX Requirements (Foundational Triplet Support)
- Source: `docs/project-context.md` — Technology Stack & Versions; Critical Implementation Rules; Framework-Specific Rules
- Source: `_bmad-output/implementation-artifacts/1-1-add-persisted-tuplet-container-entries.md` — previous story learnings and persisted tuplet model
- Source: `src/ui/popover.tsx` — existing popover lifecycle, keyboard triggers, and selection-derived flow/tick lookup
- Source: `src/services/popovers.ts` — existing parser pattern for score popovers
- Source: `src/services/actions.ts` — frontend invoke layer and sequencer selection actions
- Source: `src/services/types.ts` — current `Tuplet` DTO and `Selection` unions
- Source: `src-tauri/src/ui/sequencer.rs` — Rust selection enum, `apply_selection`, and sequencer command patterns
- Source: `src/windows/main/components/sequencer-track.tsx` — current sequencer tone-entry flow and track-context assumptions
- Source: `src/windows/main/components/renderer.tsx` — score hit-test to selection handoff

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Add a tuplet popover path to the existing score popover system, triggered by `;`, with one explicit tuplet text input using learned syntax such as `3e/2`.
- Wire the parser to a new `score.tuplet.create(...)` action and matching Rust command so confirmation creates a real `Entry::Tuplet` on the targeted track.
- Extend sequencer selection state for tuplets so creation can hand selection state back to a concrete backend-owned target without adding parallel UI state.
- Add focused frontend and Rust tests for the shortcut, parser, selection plumbing, and container creation path.

### Debug Log References

- `npm run lint`
- `npm run check-types`
- `npm run test:js`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `cargo test --manifest-path src-tauri/Cargo.toml`

### Completion Notes List

- Added a tuplet branch to the existing popover flow with a `;` keyboard trigger, an empty text input for explicit learned syntax such as `3e/2`, and the same invalid-input logging/no-op behavior used by the other popovers.
- Added `score.tuplet.create(...)` plus a Rust `score_entries_tuplet_create` command that inserts backend-owned `Entry::Tuplet` containers on the selected host track without introducing frontend rhythmic draft state.
- Extended sequencer selection with a `Tuplet` payload and matching frontend/Rust selection commands so tuplet creation hands selection state back through `apply_selection` and keeps `ui.flow_id` authoritative.
- Added focused JS and Rust coverage for the parser, popover shortcut and submit flow, tone/rest-only tuplet confirmation behavior, text-input hotkey suppression, action payloads, selection serialization, and tuplet track insertion semantics.
- Updated `docs/data-models.md` to document MVP tuplet creation semantics and the new UI-only tuplet selection payload.

## Senior Developer Review (AI)

### Reviewer

GPT-5.4 on 2026-03-14

### Outcome

Approved after follow-up fixes

### Notes

- Follow-up fixes now suppress popover hotkeys inside text inputs and require tone/rest selection data for tuplet confirmation instead of falling back to unrelated sequencer context.
- The product specification and story record now describe the shipped single-field tuplet syntax instead of the discarded ratio-plus-picker concept.
- The low-level Rust creation seam remains aligned with the current creation-command pattern: frontend flows own product-specific input validation while Rust validates target existence and mutates score state.

### File List

- \_bmad-output/implementation-artifacts/1-2-create-tuplet-containers-through-a-tuplet-popover.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- docs/data-models.md
- src/services/actions.test.ts
- src/services/actions.ts
- src/services/popovers.test.ts
- src/services/popovers.ts
- src/services/types.ts
- src/ui/popover.test.tsx
- src/ui/popover.tsx
- src-tauri/src/lib.rs
- src-tauri/src/score/entries/tuplet.rs
- src-tauri/src/ui/sequencer.rs
- src/windows/main/components/renderer.tsx
- src/windows/main/hooks/use-keyboard-shortcuts.ts
- src/windows/main/hooks/use-selected-tone-info.ts

## Change Log

- 2026-03-14: Added backend-owned tuplet popover creation, tuplet selection handoff, focused frontend and Rust coverage, and data-model documentation updates.
- 2026-03-14: Code review pass corrected story/docs claims to match current behavior, added a backend-validation-boundary test, and moved the story back to in-progress.
- 2026-03-14: Fixed tuplet popover gating so it only opens from note/rest selections, suppressed popover hotkeys inside text inputs, expanded frontend coverage for those paths, and aligned the active specs with the shipped single-field UX.
