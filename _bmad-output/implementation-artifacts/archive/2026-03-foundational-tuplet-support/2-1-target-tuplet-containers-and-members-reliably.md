# Story 2.1: Target Tuplet Containers and Members Reliably

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want tuplet containers and tuplet notes to resolve to the correct target,
so that I can edit tuplets without ambiguity about what I selected.

## Scope Note

- This story is the interaction-ownership seam for Epic 2. It is not a tuplet-editing rewrite.
- Reuse the current sequencer hit-target, selection, and popover handoff patterns before adding any new selection state.
- Keep tuplets backend-owned and selection-driven. Do not introduce a parallel frontend-owned “active tuplet” or alternate flow-tracking path.
- MVP tuplets remain single-level only. Targeting needs to distinguish one tuplet container depth plus its member tones, while empty space inside the region remains non-selectable sequencer background rather than a separately selectable sequencer object.
- Follow the existing DOM-marker pattern for sequencer hit testing rather than inventing a second targeting system.

## Acceptance Criteria

1. Given a track containing a tuplet container and one or more tuplet notes, when the composer targets a tuplet note in the sequencer, then the product selects the correct note inside the tuplet region, and selection does not fall through to surrounding host-track material.
2. Given a visible tuplet region affordance in the sequencer, when the composer targets empty space inside that region rather than a note, then the sequencer does not create tuplet-container selection and instead treats the click like any other non-note background interaction.
3. Given a tuplet region contains both notes and empty slots, when the composer interacts within that region in the sequencer, then note hits resolve to the child note and empty space remains non-selectable rather than inventing a selectable sequencer rest or container target.
4. Given a tuplet region is shown in the sequencer, when no child tone inside it is selected, then the region remains visually distinct for orientation only and is not highlighted as a selected entry in its own right.
5. Given a tuplet note is selected, when the sequencer updates its visual state, then that note is highlighted in blue like any other selected note, and the surrounding tuplet region remains a secondary visual context cue rather than a separately selected object.
6. Given a tuplet note is selected, when selection metadata is shown in the bottom bar, left-panel note-value highlight, or equivalent selection-summary surface, then the product shows the selected tone's pitch, instrument, and stave/voice context using the same descriptive contract as host-track tone selection, and tuplet membership does not cause selection-driven UI state to fall back to "No selection" or omit the tone's literal stored duration.
7. Given an empty span inside a tuplet region is clicked in the sequencer, when selection is resolved, then the product does not create a selectable sequencer rest target or tuplet-container target.
8. Given tuplet region and note states are shown together in the sequencer, when the current target is communicated, then tone selection remains primary and the tuplet region stays a non-selectable context cue without sequencer-specific rest treatment.
9. Given tuplet selection changes on the active track, when the edit state updates, then the authoritative `ui.flow_id` remains correct, and no separate frontend-owned active-flow state is introduced for tuplets.
10. Given MVP tuplets are explicitly non-nestable, when selection behavior is implemented for tuplet content, then targeting only needs to distinguish one container depth plus its members and empty slots, and MVP does not require nested container-selection semantics.

## Tasks / Subtasks

- [x] Harden tuplet-aware hit targeting and selection dispatch in the existing sequencer seams (AC: 1, 2, 3, 10)
  - [x] Reuse and extend `getSequencerClickTarget()` in `src/services/sequencer-utils.ts` rather than introducing a second hit-test path.
  - [x] Preserve tone-first targeting precedence so child-tone clicks inside a tuplet do not fall through to host-track background interactions.
  - [x] Keep DOM targeting based on `data-tone-id` and `data-tuplet-id` ancestry in the rendered sequencer elements.
  - [x] Keep tuplet region metadata available for write-tool local timing without turning blank sequencer space into a selectable tuplet object.
- [x] Preserve and clarify the distinct visual states for container, member-tone, and empty-slot targeting (AC: 3, 4, 5, 7, 8)
  - [x] Extend the current tuplet region rendering in `src/windows/main/components/sequencer-track-tuplet.tsx` so selected-child-tone feedback stays distinct without adding a selectable-container state.
  - [x] Keep selected tuplet notes using the same selected-tone treatment already used by host tones in `src/windows/main/components/sequencer-track-tone.tsx`.
  - [x] Keep empty tuplet slots non-selectable in the sequencer and remove dedicated empty-slot hit geometry from the lane.
  - [x] Preserve non-color distinction through region boundary and local markers so tone selection remains legible without introducing selectable container or rest state.
- [x] Keep selection metadata and flow ownership aligned with the existing app contract (AC: 6, 9)
  - [x] Ensure tuplet-owned tone selection still resolves through the same tone-summary contract used by host-track tones in `src/windows/main/hooks/use-selected-tone-info.ts` and `src/windows/main/components/selected-tone-info-display.tsx`.
  - [x] Preserve `ui.flow_id` as the only authoritative active-flow slot for sequencer selection changes.
  - [x] Do not add frontend-only “active tuplet”, “focused tuplet lane”, or similar state that duplicates backend-owned selection truth.
- [x] Add regression coverage at the established test seams (AC: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
  - [x] Extend `src/services/sequencer-utils.test.ts` with tuplet-member-versus-container targeting and nested-DOM ancestry cases.
  - [x] Extend `src/windows/main/components/sequencer-track.test.tsx` to verify rendered container and child-tone markers, selection routing, and state distinctions.
  - [x] Extend `src/windows/main/components/sequencer-track-tone.test.tsx` only if selected-tone behavior changes for tuplet-owned tones.
  - [x] Extend `src/windows/main/hooks/use-selected-tone-info.test.ts` and `src/windows/main/components/selected-tone-info-display.test.tsx` if tuplet-owned tone selection needs summary regression coverage.
  - [x] Extend `src/services/popovers.test.ts` only if the create-and-select handoff changes.
- [x] Validate the touched seams with the standard repo checks
  - [x] Run the relevant frontend test files covering sequencer targeting and selection-summary behavior.
  - [x] Run `npm run lint` and `npm run check-types` after implementation.
  - [x] If any `src-tauri/` selection command or state-shaping code changes, run `cargo test --manifest-path src-tauri/Cargo.toml`.

## Dev Notes

### Story Foundation

- Epic 2 starts from the completed Epic 1 tuplet foundation: tuplets already persist as first-class container entries, child tones already live inside tuplet-owned `entries`, and creation already hands selection to the new tuplet region.
- This story is about making that foundation reliably targetable in the sequencer so later edit stories can mutate the correct object without ambiguity.
- The core product promise is precision: selecting a tuplet member must behave like selecting a real note, while the surrounding tuplet region stays visual-only in the sequencer and empty tuplet space remains absence of tones.

### Technical Requirements

- Reuse the existing hit-target contract in `src/services/sequencer-utils.ts`:
  - tone targets are resolved from `data-tone-id`
  - tuplet context is carried via `data-tuplet-id`
  - child-tone clicks inside a tuplet must continue to return a tone target with `tuplet_id`
  - non-tone clicks inside a tuplet region may still resolve tuplet context for write-tool geometry, but they must not produce sequencer selection
- Preserve the existing click-to-selection dispatch in `src/windows/main/components/sequencer-track.tsx`:
  - tones select through `actions.ui.sequencer.selectTone(...)`
  - blank tuplet-region clicks clear selection just like other non-note sequencer background clicks
- Preserve the existing selection types in `src/services/types.ts`:
  - `ToneSelection` remains authoritative for tones, optionally carrying `tuplet_id`
  - `TupletSelection` remains available for renderer-owned tuplet selection outside the sequencer lane
  - do not add nested-tuplet selection semantics in MVP
- Preserve current tick-space discipline:
  - host tones use host-track ticks
  - tuplet-owned tones use tuplet-local ticks inside the child `entries`
  - targeting and selection should not prematurely translate child tones into host-space identity
- Do not add sequencer-only empty-slot targets or container-selection domains; keep blank tuplet space as absence of tones.

### Architecture Compliance

- The architecture requires tuplets to remain first-class track entries that own child tone entries whose start positions lie inside the tuplet span.
- `tone_id` remains authoritative for tone identity. Tuplet-local tick or DOM ancestry is lookup context, not a replacement identity scheme.
- `ui.flow_id` remains the single authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport-derived consumers.
- The shared top bar stays limited to universal controls. Any tuplet-selection behavior belongs inside the existing sequencer workflow surface.
- MVP tuplets remain explicitly non-nestable. Do not widen the selection model to support deeper container hierarchies in this story.

### Library / Framework Requirements

- Frontend work stays in the current React 19 + TypeScript + Tauri invoke stack.
- Use the existing `actions.ts` invoke wrappers for selection commands rather than calling `invoke()` ad hoc from components.
- Keep styling consistent with current Tailwind-based dark-theme sequencer patterns and existing selected-entry blue highlight treatment.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary frontend seams:
  - `src/services/sequencer-utils.ts`
  - `src/services/actions.ts`
  - `src/services/types.ts`
  - `src/windows/main/components/sequencer-track.tsx`
  - `src/windows/main/components/sequencer-track-tuplet.tsx`
  - `src/windows/main/components/sequencer-track-tone.tsx`
  - `src/windows/main/hooks/use-selected-tone-info.ts`
  - `src/windows/main/components/selected-tone-info-display.tsx`
- Existing tests to extend before adding new test surfaces:
  - `src/services/sequencer-utils.test.ts`
  - `src/services/popovers.test.ts`
  - `src/windows/main/components/sequencer-track.test.tsx`
  - `src/windows/main/components/sequencer-track-tone.test.tsx`
  - `src/windows/main/hooks/use-selected-tone-info.test.ts`
  - `src/windows/main/components/selected-tone-info-display.test.tsx`
- Backend selection seams to touch only if necessary:
  - `src-tauri/src/ui/sequencer/` selection commands
  - related Tauri command registration/state reducers that already own sequencer selection

### Testing Requirements

- Verify a click on a tuplet child tone resolves to the tone, carries tuplet context, and does not fall through to the host track or the enclosing container.
- Verify a click on blank tuplet region space in the sequencer does not create container selection and does not implicitly select an inner tone.
- Verify tone-selected and non-selected tuplet-region states remain visually distinguishable in the sequencer without introducing selectable container or empty-slot rest state.
- Verify tuplet-owned tone selection continues to populate the existing bottom-bar summary instead of falling back to `No selection`.
- Verify `ui.flow_id` remains stable and authoritative through tuplet-related selection changes.
- Prefer extending the current React/unit seams over inventing fixture-heavy or end-to-end harnesses for this story.

### Established Code Patterns To Follow

- Reuse `getSequencerClickTarget()` instead of inventing a second sequencer hit-test path.
- Preserve the current DOM marker pattern: `data-tone-id` on tones and `data-tuplet-id` on tuplet regions and tuplet-owned tone ancestry.
- Reuse `SequencerTrackTuplet` and `SequencerTrackTone` as the rendering seams for targetability and selected-state visuals.
- Reuse `parseTupletPopover()` as the reference for create-to-select behavior; do not fork a separate tuplet activation flow.
- Reuse `useSelectedToneInfo()` and `SelectedToneInfoDisplay` for selection-summary behavior instead of creating a tuplet-only summary surface.
- Extend existing tests in place before adding new files. This repo already has coverage at the exact seams this story needs.

### Previous Story Intelligence

- Story 1.4 established the tuplet-aware sequencer note-entry path and introduced the local tick-space utilities used inside tuplet regions.
- Story 1.5 established ownership rules for tones that start inside or outside tuplets and preserved the “one stored tone, ownership by start position” rule.
- Story 1.6 and Story 1.7 hardened persistence, migration, and workflow seams without adding parallel frontend truth. Continue that seam-by-seam discipline here.
- Recent completed work already selected the newly created tuplet after popover confirmation. Story 2.1 should build on that selection handoff rather than replacing it.

### Git Intelligence Summary

- Recent relevant commits:
  - `d4617be1 feat: epic 1 retro`
  - `4965b07a feat(1.7): validate tuplet file workflow contract`
  - `c859a420 feat(1.6): tuplet file loading regression tests`
- The recent pattern is incremental seam hardening with focused tests and minimal scope. Follow that same approach for selection targeting: strengthen the existing targeting and selection seams rather than widening the feature surface.

### Documentation Impact

- If implementation changes the documented selection contract for tuplets, update the relevant developer docs in the same change.
- Likely documentation touchpoints if behavior changes materially:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md` only if project guardrails or required tuplet-selection practices change
- If no user-facing or architectural behavior changes beyond code-level hardening, document review can conclude that no docs update is needed, but that decision must be stated in the PR/work summary.

### Project Structure Notes

- The repo keeps frontend state as a display layer over Rust-owned state. Selection changes must continue to flow through the existing Tauri action path.
- The current sequencer already renders tuplets and tuplet-owned tones in separate passes inside `sequencer-track.tsx`; target resolution should align with that render structure.
- The selected-tone summary already derives instrument and staff/voice labels through `useSelectedToneInfo()`. Tuplet-owned tones should continue to use that same summary contract.
- There are already targeted unit tests for sequencer click targets, popover parsing, track rendering, and selected-tone info. Reuse them before adding any new harness.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 / Story 2.1 acceptance criteria
- Source: `_bmad-output/planning-artifacts/prd.md` — editing semantics, exact targeting expectations, and desktop workflow constraints
- Source: `_bmad-output/planning-artifacts/architecture.md` — tuplet container ownership, `tone_id` authority, and `ui.flow_id` rules
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — tuplet presentation, selection feedback, and empty-slot behavior
- Source: `docs/project-context.md` — active-flow, Tauri action, and frontend/backend ownership guardrails
- Source: `_bmad-output/implementation-artifacts/1-7-keep-versioning-and-offline-desktop-behavior-intact-for-tuplet-files.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Inspect and extend the existing sequencer hit-target and selection routing seams before adding any new UI behavior.
- Keep container, child-tone, and empty-slot targeting aligned with the current render tree and selection types.
- Add focused regression coverage at the current unit/component seams, then run the required validation commands.

### Debug Log References

- Verified Epic 2 Story 2.1 requirements in `_bmad-output/planning-artifacts/epics.md`.
- Verified architecture rules for tuplet ownership, `tone_id` authority, and `ui.flow_id` in `_bmad-output/planning-artifacts/architecture.md` and `docs/project-context.md`.
- Verified existing implementation seams for tuplet targeting and selection in `src/services/sequencer-utils.ts`, `src/windows/main/components/sequencer-track.tsx`, `src/windows/main/components/sequencer-track-tuplet.tsx`, `src/windows/main/components/sequencer-track-tone.tsx`, `src/services/popovers.ts`, `src/windows/main/hooks/use-selected-tone-info.ts`, and their existing tests.
- Verified recent implementation pattern from git history: focused seam-by-seam hardening with regression tests.
- Hardened the sequencer hit target contract so child tones keep tuplet context while blank tuplet region clicks behave like other non-note sequencer background interactions.
- Extended the Rust and TypeScript rest-selection payloads with optional tuplet context so empty-slot targeting can remain backend-owned without introducing a new selection type.
- Final abstraction decision for review: `tick` remains the sole positional source of truth, and `tuplet_id` determines the coordinate space for rest selection. When `tuplet_id` is present, `tick` is interpreted as tuplet-local tick; when absent, `tick` is interpreted as host-track tick. The earlier `tuplet_slot_index` experiment was removed as redundant presentation state so the contract stays aligned with the broader app pattern that identity is position plus context.
- Verified tuplet-owned tone summary resolution through the existing bottom-bar contract and updated the docs to clarify that sequencer tuplets are visual-only while renderer-owned selection still covers rests and tuplet containers.

### Completion Notes List

- Story prepared to direct the dev agent toward the current sequencer targeting and selection seams instead of introducing parallel state or ad hoc hit testing.
- The main implementation risk is losing tuplet context on tone selection or collapsing container, tone, and empty-slot states into one ambiguous UI state.
- The key existing patterns to preserve are DOM-marker hit testing, action-driven selection, tuplet-local child ownership, and the shared bottom-bar selection summary contract.
- Documentation review is explicitly part of the story so any material selection-contract change is documented in the same work.
- Implemented empty-slot interaction so the sequencer treats blank tuplet space as non-selectable background while preserving child-tone precedence for note hits and write-tool tuplet context.
- Requirement correction during implementation: sequencer rest and tuplet-container selection are intentionally out of scope. Rest and tuplet selection remain printed-score concerns where notation objects are rendered explicitly; the sequencer only selects visible tones in this story.
- Fixed `useSelectedToneInfo()` and `SelectedToneInfoDisplay` coverage so tuplet-owned tones resolve through the same summary contract as host-track tones instead of falling back to `No selection`.
- Extended the write-screen left panel to resolve tuplet-owned selected tones through that same shared tone-lookup contract, so literal stored durations still light the matching note-value button even though tuplet-aware duration mutation from that panel remains deferred.
- Validation passed with targeted frontend tests, `src/services/actions.test.ts`, `npm run lint`, `npm run check-types`, and `cargo test --manifest-path src-tauri/Cargo.toml`.
- Updated `docs/data-models.md` and `docs/development-guide.md` so the sequencer-versus-renderer ownership split for tuplets and rests is explicit.
- Code review closed with no blocking Story 2.1 defects. Follow-up concerns about general tuplet-child editing affordances and keyboard-driven editing were explicitly treated as later Epic 2 work rather than regressions in this selection-hardening story.

### File List

- `_bmad-output/implementation-artifacts/2-1-target-tuplet-containers-and-members-reliably.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/services/actions.test.ts`
- `src/services/actions.ts`
- `src/services/sequencer-utils.test.ts`
- `src/services/sequencer-utils.ts`
- `src/services/types.ts`
- `src/windows/main/components/selected-tone-info-display.test.tsx`
- `src/windows/main/components/sequencer-track.test.tsx`
- `src/windows/main/components/sequencer-track-tuplet.tsx`
- `src/windows/main/components/sequencer-track.tsx`
- `src/windows/main/hooks/use-selected-tone-info.test.ts`
- `src/windows/main/hooks/use-selected-tone-info.ts`
- `src-tauri/src/files.rs`
- `src-tauri/src/instructions/draw/draw_rest.rs`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/ui/mod.rs`
- `src-tauri/src/ui/sequencer.rs`

### Change Log

- 2026-03-16: Created Story 2.1 with repository-specific selection, targeting, testing, and documentation guardrails based on the existing tuplet implementation seams.
- 2026-03-16: Implemented tuplet container and member-tone targeting hardening, then tightened the sequencer contract so only tones are selectable there while tuplet regions remain visual-only context and empty space stays non-selectable, with updated docs and regression coverage.
- 2026-03-16: Extended Story 2.1's selected-tone metadata seam into the write-screen left panel so tuplet child tones highlight their literal stored note value there, while keeping tuplet duration mutation via that panel out of scope for later edit stories.
- 2026-03-16: Closed review and marked the story done after classifying broader tuplet-child editing and keyboard-edit concerns as deferred follow-up work for later Epic 2 stories.
