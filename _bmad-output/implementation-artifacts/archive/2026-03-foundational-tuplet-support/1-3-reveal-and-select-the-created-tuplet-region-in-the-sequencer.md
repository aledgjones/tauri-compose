# Story 1.3: Render and Select the Created Tuplet Region in the Sequencer

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want a created triplet to render as a clear sequencer region with visible timing cues,
so that I can recognize and target the tuplet structure before entering notes.

## Scope Note

- This story was narrowed to the durable sequencer presentation and targeting contract for tuplets.
- Viewport auto-reveal after creation is not part of the shipped scope for Story 1.3 and remains deferred.

## Acceptance Criteria

1. Given the active track contains a tuplet container, when the sequencer renders that track, then the tuplet span appears as one visibly bounded region on the host lane, and the region remains visible even when `tuplet.entries.by_id` contains no child tones.
2. Given a tuplet region is shown in the sequencer, when the user reads the governed span, then the lane remains visually distinct from ordinary host-track space and a compact `3:2` annotation appears in the sticky tick bar aligned to the consumed host ticks.
3. Given the tuplet annotation is shown in the sequencer, when the user works within the tuplet region, then the `3:2` annotation remains centered over the governed span in the sticky top ruler and does not use a solid background that obscures tones underneath it.
4. Given the tuplet span is visible in the sequencer, when the user inspects its internal timing structure, then the vertical slot indicators inside the region appear slightly compressed to show tuplet warping, and that compressed treatment remains confined to the tuplet span without destabilizing the broader host grid.
5. Given the tuplet region is visible in the sequencer, when the user targets it directly, then the click resolves to the backend-owned tuplet container selection contract instead of clearing selection or falling through as ordinary lane space.
6. Given the tuplet container has no entered notes yet, when it is first shown in the sequencer, then the grouped tuplet region remains visible and targetable as empty content, and the empty local slots remain legible as available tuplet positions.

## Tasks / Subtasks

- [x] Render tuplet containers as first-class sequencer regions on the active track (AC: 1, 2, 4, 6)
  - [x] Extend the sequencer track rendering path in `src/windows/main/components/sequencer-track.tsx` so it no longer filters to tones only; tuplets must render as bounded regions even when `entries.by_id` contains no child tones.
  - [x] Add a dedicated sequencer tuplet presentation component, likely `src/windows/main/components/sequencer-track-tuplet.tsx`, to keep region drawing and label behavior separate from tone rendering and aligned with the repo's one-component-per-file pattern.
  - [x] Derive host-span width from existing ratio metadata (`note_count`, `in_space_of`, `base_note_value`) rather than adding duplicate persisted span fields; reuse the existing frontend `noteDurationToTicks(...)` helper or add a small tuplet-specific helper near the sequencer UI.
- [x] Add tuplet-region targeting and selection behavior in the sequencer lane (AC: 5, 6)
  - [x] Extend `src/services/sequencer-utils.ts` so sequencer hit testing can resolve tuplet containers in addition to tones, using a distinct DOM target such as `data-tuplet-id`.
  - [x] Update `handleSelectTool(...)` in `src/windows/main/components/sequencer-track.tsx` so clicking a tuplet region selects the container through the existing backend-owned selection contract instead of falling through to clear selection.
  - [x] Preserve the current rule that `ui.flow_id` remains authoritative by reusing the existing `Selection::Tuplet` payload and `apply_selection(...)` path in `src-tauri/src/ui/sequencer.rs`.
- [x] Show the compact sticky `3:2` label and bounded visual treatment without obscuring notes (AC: 2, 3)
  - [x] Draw a compact tuplet annotation in the sticky tick bar, aligned to the governed region span rather than as detached floating text.
  - [x] Keep the label visually readable while the sequencer scrolls by using the existing sticky top-bar conventions in the sequencer instead of introducing a separate overlay window.
  - [x] Ensure the label and region styling stay transparent or lightly stroked/tinted so later child tones remain visible underneath the annotation.
- [x] Expose compressed internal slot structure for empty tuplets without changing the host grid contract (AC: 4, 6)
  - [x] Render local slot boundaries inside the tuplet region using slightly compressed spacing relative to the surrounding host tick grid, but confine that treatment to the tuplet span only.
  - [x] Keep the outer host tick bar stable; tuplets should visually communicate warped local slots inside the region, not redefine global sequencer timing.
  - [x] Make empty slots targetable and visually quieter than tones so the region remains actionable before Story 1.4 adds note entry inside it.
- [x] Cover the new region behavior with focused frontend tests close to the owning code (AC: 1, 2, 3, 4, 5, 6)
  - [x] Extend `src/windows/main/components/sequencer-track.test.tsx` for tuplet region rendering, tuplet click targeting, empty-region persistence, and compressed local marker visibility.
  - [x] Extend `src/windows/main/components/sequencer.test.tsx` for sticky tuplet label behavior at the sequencer-container level.
  - [x] Extend `src/services/sequencer-utils.test.ts` for tuplet click-target parsing.
- [x] Update documentation only where this story establishes durable sequencer behavior expectations (AC: 2, 3, 4, 6)
  - [x] Update the most relevant developer docs if the sequencer's tuplet-region contract and tick-bar annotation behavior become durable patterns future tuplet work must follow.

## Dev Notes

### Story Foundation

- Story 1.2 already creates a real backend-owned `Entry::Tuplet` and immediately selects it through `actions.ui.sequencer.selectTuplet(...)`; Story 1.3 should treat that selection handoff as the starting point, not re-solve creation.
- The product requirement here is not generic sequencer polish. It is the durable presentation seam that makes backend-owned tuplets legible and targetable in the sequencer before any child notes exist.
- Story 1.3 stops at region rendering, annotation, targeting, and empty-slot visibility. Writing notes inside that region belongs to Story 1.4.

### Technical Requirements

- Keep Rust as the source of truth for score state. The frontend may derive view-only geometry for tuplet regions, but it must not create a frontend-owned tuplet timing model.
- Reuse the existing tuplets DTO shape in `src/services/types.ts`: `note_count`, `in_space_of`, `base_note_value`, `groupings`, and nested `entries`.
- Derive tuplet consumed width from ratio metadata rather than adding new duplicated host/local span fields to persisted state or FE state.
- The existing frontend helper `noteDurationToTicks(...)` in `src/services/utils.ts` already maps `NoteDuration` enum values to tick counts. Use that helper or a thin wrapper instead of inlining new duration tables in the sequencer.
- Preserve the current `Selection::Tuplet` contract in both TypeScript and Rust. Story 1.3 needs to use it for sequencer targeting and reveal, not add a second container-selection payload.
- Preserve the current `Selection::Tuplet` contract in both TypeScript and Rust. Story 1.3 needs to use it for sequencer targeting and selection, not add a second container-selection payload.
- The sequencer must keep host-grid semantics stable. Compressed tuplet markers are a local visual affordance inside the region, not a global remapping of tick positions.

### Architecture Compliance

- The architecture explicitly requires tuplets to appear as one visibly bounded sequencer region with a compact `3:2` annotation aligned to the governed span, sticky top behavior, compressed internal markers, and empty-slot persistence. Treat those as architecture obligations, not optional UI interpretation.
- The architecture also requires the active flow to remain authoritative through `ui.flow_id`; Story 1.3 must preserve that by continuing to flow selection changes through `apply_selection(...)` in `src-tauri/src/ui/sequencer.rs`.
- Do not introduce a second sequencer overlay model detached from existing track rendering. The current sequencer component tree is the correct place to render tuplet regions.
- Keep the implementation compatible with the brownfield track-entry model where tuplets and tones share `track.entries.by_id`; the sequencer should render both entry classes from the same authoritative track data.

### Library / Framework Requirements

- Frontend implementation should stay in the current React 19 + TypeScript + Tailwind stack and follow existing component patterns.
- External web research was not available in this workflow context. Use the repo-pinned stack and project docs as the technical baseline.

### File Structure Requirements

- Current implementation seams:
  - `src/windows/main/components/sequencer.tsx` owns the sequencer viewport and passes stored scroll coordinates into `DragScroll`.
  - `src/windows/main/components/sequencer-track.tsx` currently renders only tones and owns write/select/slice/erase pointer behavior.
  - `src/services/sequencer-utils.ts` currently recognizes only tone targets via `data-tone-id`.
- Expected touchpoints for this story:
  - `src/windows/main/components/sequencer.tsx`
  - `src/windows/main/components/sequencer-track.tsx`
  - `src/windows/main/components/sequencer-track-tuplet.tsx` if a dedicated tuplet region component is added
  - `src/windows/main/components/tick-bar.tsx`
  - `src/services/sequencer-utils.ts`
  - `src/windows/main/components/sequencer.test.tsx`
  - `src/windows/main/components/sequencer-track.test.tsx`
  - `src/services/sequencer-utils.test.ts`
- Keep the work focused on sequencer presentation and targeting. Do not pull parser, renderer, playback, viewport auto-reveal, or child-note-entry logic forward into this story.

### Testing Requirements

- Validate that empty tuplets render as visible bounded regions even when `tuplet.entries.by_id` is empty.
- Validate that tuplet regions expose a clickable container target distinct from tone targets and that selection does not collapse to null on tuplet clicks.
- Validate the compact `3:2` annotation and sticky behavior at the component level as far as the current test setup allows.
- Validate that compressed local marker rendering stays confined to the tuplet span and does not mutate the outer host tick grid rendering.
- Run the relevant repo checks after implementation:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`

### Previous Story Intelligence

- Story 1.2 already added `Selection::Tuplet` on both the frontend and backend, plus the creation flow that selects the new tuplet container immediately after popover confirmation. Story 1.3 should reuse that path.
- Story 1.2 intentionally stopped short of visible sequencer-region rendering. Its notes explicitly call out that the current sequencer track UI only renders tones and that visible tuplet-region rendering belongs to this story.
- Story 1.2 also established the rule that tuplets are created from the current selection tick with no frontend-owned draft state. Story 1.3 should preserve that product boundary.

### Git Intelligence Summary

- Recent relevant commits:
  - `52cd2a1c feat(1.2): description`
  - `a503dd49 feat(1.2): add tuplet popover creation flow`
  - `10dc0067 feat(1.1): init tuplets`
- The current repo pattern for new score-edit behavior is consistent: TypeScript actions and UI state trigger Rust-owned mutations or selections, and the frontend renders from emitted state without parallel ownership.
- The recent tuplet work already touched `src/ui/popover.tsx`, `src/services/popovers.ts`, `src/services/actions.ts`, `src/services/types.ts`, `src-tauri/src/ui/sequencer.rs`, and `src-tauri/src/score/entries/tuplet.rs`; Story 1.3 should build directly on those seams.

### Documentation Impact

- If this story makes the sequencer-region contract durable for future tuplet work, update the relevant developer-facing docs in the same change.
- At minimum, keep any docs that describe sequencer targeting or tuplet-region annotation expectations aligned with the shipped behavior.

### Project Structure Notes

- `src/windows/main/components/sequencer-track.tsx` currently filters `track.entries.by_id` to `entry.type === "Tone"`, so empty tuplets are invisible even though the tuplets already exist in track data.
- `src/services/sequencer-utils.ts` currently only parses `data-tone-id`, so tuplet containers are not yet targetable through sequencer clicks.
- `src/windows/main/components/renderer.tsx` already maps score-surface tuplet hits to `actions.ui.sequencer.selectTuplet(...)`, so score-side selection plumbing is already in place.
- `src/windows/main/hooks/use-selected-tone-info.ts` intentionally returns `null` for `Tuplet` selections today. That is acceptable unless Story 1.3 deliberately expands the selection-summary UI.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 / Story 1.3 / Requirements Inventory
- Source: `_bmad-output/planning-artifacts/architecture.md` — Sequencer UX Contract; Focus, Feedback, And Accessibility Contract; Recommended Persisted Data Model
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — Tuplet UX Requirements (Foundational Triplet Support)
- Source: `docs/project-context.md` — Technology Stack & Versions; Critical Implementation Rules; Framework-Specific Rules
- Source: `_bmad-output/implementation-artifacts/1-2-create-tuplet-containers-through-a-tuplet-popover.md` — previous story learnings and current handoff boundary
- Source: `src/windows/main/components/sequencer.tsx` — sequencer viewport ownership and `DragScroll` usage
- Source: `src/windows/main/components/sequencer-track.tsx` — current track rendering and pointer interaction behavior
- Source: `src/components/drag-scroll.tsx` — current scroll-state sync behavior
- Source: `src/services/sequencer-utils.ts` — current sequencer click-target parsing
- Source: `src/services/types.ts` — current `Tuplet` DTO and `Selection` union
- Source: `src/services/utils.ts` — `noteDurationToTicks(...)` helper
- Source: `src-tauri/src/ui/sequencer.rs` — `Selection::Tuplet`, `apply_selection(...)`, and sequencer selection commands
- Source: `src/windows/main/components/renderer.tsx` — score hit-test to tuplet selection handoff
- Source: `src-tauri/src/score/entries/tuplet.rs` — ratio-derived tuplet width semantics (`consumed_ticks()` / `local_length_ticks()`)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Extend the sequencer track rendering path so tuplets render as first-class bounded regions, including empty tuplets with no child tones.
- Add tuplet click targeting and reuse the existing tuplet selection contract so sequencer interaction matches the current backend-owned flow model.
- Add a sticky tick-bar annotation aligned to the governed tuplet span without obscuring lane content.
- Add focused tests for region rendering, targeting, sticky annotation, and compressed local markers.

### Debug Log References

- `git --no-pager log --oneline -5`
- `npm run lint`
- `npm run check-types`
- `npm run test:js`

### Completion Notes List

- Story context generated from the active epic, architecture, UX specification, project context, prior story output, and current code seams.
- Validation task file `_bmad/core/tasks/validate-workflow.xml` was not present in the workspace, so checklist validation was performed manually against the available create-story rubric and source artifacts.
- Sequencer tracks now render backend-owned tuplets as first-class bounded regions, using ratio-derived host width, a faint background tint, and compressed internal slot markers that keep empty tuplets legible before note entry exists.
- The `3:2` annotation now lives in the sticky tick bar as a bottom-anchored ruler annotation aligned to the governed span, keeping the track body focused on slot structure rather than text overlay.
- Story 1.3's shipped scope is the durable sequencer rendering and targeting contract for tuplets; viewport auto-reveal remains deferred.
- Added focused frontend coverage for tuplet hit-target parsing, tuplet region rendering, bar-aware local marker styling, tick-bar annotation rendering, and the host-grid suppression rules inside tuplet spans.
- Updated developer docs to record the new sequencer tuplet-region contract and the dedicated `SequencerTrackTuplet` component.
- Validation passed with `npm run lint`, `npm run check-types`, `npm run test:js`, and the full JS suite via the workspace test runner.

### File List

- \_bmad-output/implementation-artifacts/1-3-reveal-and-select-the-created-tuplet-region-in-the-sequencer.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- docs/component-inventory.md
- docs/data-models.md
- src/services/sequencer-utils.test.ts
- src/services/sequencer-utils.ts
- src/windows/main/components/sequencer-track.test.tsx
- src/windows/main/components/sequencer-track-tuplet.tsx
- src/windows/main/components/sequencer-track.tsx
- src/windows/main/components/sequencer-tuplet-utils.ts
- src/windows/main/components/sequencer.test.tsx
- src/windows/main/components/sequencer.tsx
- src/windows/main/components/tick-bar.tsx

## Change Log

- 2026-03-14: Rendered tuplets as sequencer regions with compressed slot markers, moved the ratio annotation into the tick bar, narrowed Story 1.3 to the shipped sequencer rendering and targeting contract, and aligned the developer docs with the shipped behavior.
- 2026-03-14: Senior developer review aligned the story ACs, task list, and file list with the implemented scope and approved the story as done.

## Senior Developer Review (AI)

### Reviewer

GPT-5.4

### Date

2026-03-14

### Outcome

Approve

### Notes

- The original story text still described viewport auto-reveal work that was not part of the shipped implementation.
- The implemented behavior consistently covers bounded tuplet-region rendering, sticky tick-bar annotation, compressed internal slot markers, empty-region persistence, and tuplet click targeting.
- The story has been updated to make those shipped behaviors authoritative and to remove stale `DragScroll` and reveal claims from the task list, testing notes, and file list.
