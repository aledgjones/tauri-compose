# Story 3.1: Show Compact Rack-First Route State On Instruments

Status: ready-for-dev

## Story

As a composer,
I want each score instrument to show its current playback route and whether playback will work,
so that I can tell at a glance if the instrument is fine or needs action.

## Acceptance Criteria

1. Given a score contains instruments with different playback situations, when I view score-context instrument surfaces, then each instrument shows a compact route summary in the existing instrument-row routing surface and the presentation stays rack-first rather than assignment-first.
2. Given an instrument is in the normal playable state, when the row is displayed, then no special Ready badge, icon, or extra inline decoration is required beyond the ordinary route control presentation.
3. Given an instrument is not ready for playback, when the row is displayed, then the control shows one compact inline warning affordance that makes the state distinguishable from Ready without relying on color alone.
4. Given an instrument needs user action for playback to continue, when the user inspects the compact warning affordance, then a brief native hover title is sufficient and the UI does not add inline explanatory message boxes, address text fields, or other proof-oriented detail.
5. Given the current MVP is still single-target-first, when compact route state is shown, then it avoids surfacing address-level text or multi-instrument VST refinement detail that belongs to later routing work.

## Tasks / Subtasks

- [ ] Task 1: Keep Story 3.1 on the existing compact row seam instead of adding a new playback UI surface (AC: 1, 2, 3, 4)
  - [ ] Reuse the existing instrument-row route control in Sequence rather than introducing a separate badge strip, panel, or message area.
  - [ ] Keep the presentation framed as instrument-to-playback-route, using the current rack-first terminology.
  - [ ] Do not introduce new route-detail sections, status blocks, or explanatory inline chrome purely to prove acceptance criteria.
- [ ] Task 2: Present non-ready playback as one compact inline warning state while leaving Ready visually quiet (AC: 2, 3, 4)
  - [ ] Leave the ordinary playable state free of special readiness markings.
  - [ ] Show a small warning icon inline in the route select for any non-ready state that requires user action.
  - [ ] Ensure the warning state remains legible without relying on color alone.
- [ ] Task 3: Keep compact state text and hover behavior intentionally brief (AC: 3, 4, 5)
  - [ ] Use a concise native title on hover for the non-ready warning affordance.
  - [ ] Keep the hover copy short and action-oriented rather than diagnostic or technical.
  - [ ] Do not add address summaries, target-address text boxes, or inline explanatory error messages as part of this story.
- [ ] Task 4: Preserve backend-owned route truth while keeping the compact mapping bounded (AC: 1, 3, 5)
  - [ ] Consume backend-owned route state and existing row presentation seams rather than introducing new frontend-owned playback truth.
  - [ ] Keep compact display logic aligned with the approved three-state model: Ready, Needs Attention, Unavailable.
  - [ ] Treat Needs Attention and Unavailable as the same compact UI outcome for this story: playback needs user action.
- [ ] Task 5: Add focused validation and lightweight documentation alignment for the compact row-state slice (AC: 1, 2, 3, 4, 5)
  - [ ] Update frontend tests around instrument-row playback state so Ready remains visually quiet and non-ready states show the bounded warning affordance.
  - [ ] Verify the story does not add address-detail UI, inline message boxes, or extra proof controls.
  - [ ] Update implementation-facing notes only if needed so later stories do not assume Story 3.1 introduced richer inline route diagnostics.

## Dev Notes

### Story Intent

- Story 3.1 is a compact visibility slice, not a route-authoring or recovery-workflow slice.
- The user only needs to know one thing at the row level: will this instrument play or not.
- Ready is business-as-usual and should not draw attention to itself.
- Needs Attention and Unavailable remain distinct backend states, but for this story they can share one compact UI treatment because both mean playback needs user action.
- Do not use this story to start surfacing address-level routing detail, multi-instrument VST nuance, or explanatory status prose.

### User Decisions Captured

- Avoid adding address info as inline text at this stage.
- Keep the approved three-state model: Ready, Needs Attention, Unavailable.
- Ready should not have a special inline marker.
- Needs Attention and Unavailable can collapse to the same compact warning treatment in the row UI.
- A small warning icon inline in the select is enough for non-ready states.
- Native hover title is enough; do not over-explain error states inline.
- Avoid fiddly inline message boxes and similar over-explanatory patterns.

### Bounded Slice Template

#### This Story Owns

- Compact route-state visibility on the existing Sequence instrument-row routing seam.
- A visually quiet Ready state.
- A single compact inline warning treatment for non-ready playback states.
- Brief native-hover clarification for the warning affordance.

#### This Story Does Not Own

- Route assignment or route-changing workflow depth. Story 3.2 owns that active interaction slice.
- Slot creation, plugin-picking, or Plugin Manager behavior.
- Detailed recovery UX, route repair UX, or richer contextual actions. Story 3.3 and later stories own that.
- Address-level text presentation, multi-target explanation, or multi-instrument VST refinement.
- Inline message-box explanations for compact row states.

#### Authoritative State Lives In

- Rust-owned route and resolved playback state emitted through existing app-state seams.
- Existing Sequence instrument-row presentation seams in React.

#### Future Stories Intentionally Deferred

- Richer route diagnostics and recovery guidance.
- Address-level or target-level detail for more advanced routing scenarios.
- Multi-instrument or multi-timbral VST explanation surfaces.
- Expanded route repair and rerouting workflow design.

#### Verification Required For Done

- Frontend tests prove the existing row seam shows compact route state without adding extra UI surfaces.
- Frontend tests prove Ready has no special warning marker.
- Frontend tests prove non-ready states surface one compact warning affordance and do not rely on color alone.
- Validation confirms the slice does not add inline message boxes, address-detail text, or proof-oriented controls.

#### Open Questions Requiring Human Clarification

- None. The compact-state direction is clear: quiet Ready state, bounded non-ready warning, and no extra inline explanation.

### Scope Boundaries

- In scope:
  - Compact route summary on the existing row surface.
  - Quiet Ready presentation.
  - One bounded non-ready inline warning affordance.
  - Brief native title hover.
- Out of scope:
  - New route detail UI.
  - New inline status prose or message boxes.
  - Address-level display.
  - Richer recovery workflows or additional controls.

### Cross-Story Alignment

- Story 3.1 should only make compact row state legible.
- Story 3.2 should continue to own assigning or changing the single-target playback route.
- Story 3.3 and later stories should own deeper non-ready recovery semantics and repair flows.
- Future multi-instrument VST or address-detail work should refine the model later rather than leaking into this ticket now.

### Prior Work Intelligence

- The backend-owned rack-first model already exists and should remain the source of truth.
- The current Sequence row already contains the route control, so Story 3.1 should refine that seam rather than invent another one.
- Existing presentation helpers already compute richer slot, target, and address summaries; this story should resist surfacing that extra detail prematurely.
- The current warning presentation is too narrow because it only appears for one state and carries more detail than the user wants at this level.

### Architecture Compliance

- Keep Rust as the sole authoritative owner of route truth and resolved playback state.
- Keep React as the thin presentation layer over that state.
- Preserve the score-context versus app-scoped boundary; this story stays in the Sequence row seam.
- Do not introduce parallel frontend-owned state or UI logic that reinterprets routing complexity beyond the compact display need.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing instrument-row route control and the primary implementation seam for this story.
- `src/windows/main/screens/sequence/components/playback-readiness.tsx`
  - Existing inline readiness affordance seam.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Existing compact-state presentation helper that should stay bounded and avoid leaking future detail into the row.
- `src/services/types.ts`
  - Relevant only if the frontend contract needs a lighter presentation label, though current evidence suggests the story can likely stay frontend-only.

### Implementation Guidance

- Prefer the smallest change that makes non-ready playback legible in the existing route control.
- Keep Ready visually ordinary.
- Use one compact warning icon for states that require action.
- Use native title hover for a brief explanation only.
- Do not add inline message boxes, banners, helper paragraphs, address summaries, or extra controls.
- Do not turn the row into a diagnostic surface.
- If richer backend distinctions remain useful internally, keep them internal; the compact row only needs to signal whether playback needs action.

### Persistence And Migration Requirements

- No persisted-data or migration work is expected for this story.
- If implementation starts requiring persisted schema changes, stop and re-evaluate because the slice boundary has been crossed.

### File Structure Requirements

- Keep implementation centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
- Avoid spreading this story into Plugin Manager, bottom-panel rack workflow, backend routing model, or persistence seams unless a small contract adjustment is proven necessary.

### Testing Requirements

- Frontend tests should cover:
  - Ready rows having no special warning marker.
  - Needs Attention rows showing the bounded warning treatment.
  - Unavailable rows showing the same bounded warning treatment.
  - Compact state remaining distinguishable without relying on color alone.
  - No inline explanatory message boxes or address-detail text appearing from this story.
- Validate with the project-standard commands relevant to touched files:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`

### Library / Framework Requirements

- Reuse the existing React, TypeScript, Tauri, and Sequence component seams.
- Do not add a new UI framework, state-management path, or tooltip system for this story.
- Native title hover is sufficient for the bounded hover requirement.

### Documentation Impact

- No broad user-facing documentation update is expected for this bounded UI refinement.
- If implementation-facing notes or later-story assumptions currently imply richer inline route diagnostics, update those notes so Story 3.1 remains correctly scoped.

### Latest Technical Information

- The current implementation already computes more detail than the desired compact UI should show.
- The existing row seam and route select are the correct place to refine compact route state.
- Future routing sophistication should stay deferred until later stories reopen that scope intentionally.

### Project Structure Notes

- This story should remain a small Sequence-row refinement.
- Avoid broadening the ticket into backend model work, Plugin Manager work, or multi-surface UI changes.
- If the current route-update failure alert appears while touching nearby code, treat it as separate from this compact-state story unless the touched implementation makes a bounded cleanup unavoidable.

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 3.1 intent and bounded acceptance criteria for compact rack-first route state.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — compact row ownership, quiet score-first routing, and minimal non-ready explanation.
- `docs/project-context.md` — frontend/backend ownership, score-context UI boundaries, and compact score-first UX rules.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — existing row seam.
- `src/windows/main/screens/sequence/components/playback-readiness.tsx` — existing compact warning affordance seam.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts` — existing row-state presentation helper.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

### Completion Notes List

### File List
