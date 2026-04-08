# Story 3.4: Keep Route And Plugin Session Behavior Stable In-App

Status: done

## Story

As a composer,
I want routed instruments and their plugin sessions to remain stable while I work,
so that changing routes or reopening relevant playback surfaces does not make the workflow feel fragile.

## Acceptance Criteria

1. Given an instrument has a valid routed playback target, when I continue using the score workflow in the same app session, then the route remains associated with the intended rack and slot state and the existing plugin-session behavior remains usable for that routed instrument.
2. Given I change a route for an existing instrument, when the new route is applied through the supported current workflow, then the old and new route state transitions remain coherent and the score does not need restructuring.
3. Given route and plugin session state are backend-owned, when score-context routing workflows are used repeatedly in the same session, then the frontend continues reflecting backend truth and does not introduce parallel route-state ownership.
4. Given this story is intended as a stability and validation slice, when implementation begins, then the developer should prefer regression checks and the smallest safe correction over new workflow design or surface expansion.

## Tasks / Subtasks

- [x] Task 1: Validate same-session route and slot association stability through the existing workflow (AC: 1, 2, 4)
  - [x] Confirm a routed instrument stays associated with the intended rack and slot while the user continues working in the same app session.
  - [x] Confirm the supported current route-change workflow does not leave stale route associations behind.
  - [x] Confirm score work does not require restructuring after a route change.
- [x] Task 2: Validate plugin-session behavior remains usable without redesigning the workflow (AC: 1, 4)
  - [x] Confirm the existing in-app route and related playback-surface interactions do not destabilize the slot-owned plugin-session foundation.
  - [x] Treat this as a stability check over the existing session model rather than a new plugin-session feature story.
  - [x] If a gap is found, keep any fix tightly bounded to the existing seam.
- [x] Task 3: Keep frontend behavior aligned to backend-owned truth (AC: 2, 3, 4)
  - [x] Verify repeated use of current score-context routing flows continues to reflect backend-owned state.
  - [x] Do not introduce frontend-owned fallback state or duplicate route ownership while fixing any issue.
  - [x] Reuse existing state contracts and routing/session seams.
- [x] Task 4: Avoid destabilizing working flows while touching this area (AC: 1, 2, 3, 4)
  - [x] Do not add new route/session UI, new repair workflow, or new app-scoped control surface for this story.
  - [x] Do not broaden this story into transport, playback-preparation, or reopen-recovery redesign.
  - [x] If error handling or state cleanup needs adjustment, keep it minimal and regression-focused.
- [x] Task 5: Add focused regression coverage for stability in the supported path (AC: 1, 2, 3, 4)
  - [x] Update or confirm tests that route changes remain coherent through the existing workflow.
  - [x] Add or refine tests only where needed to prove no same-session stability regression.
  - [x] Document explicitly that Story 3.4 is a stability gate and validation story, not a new feature-surface story.

### Review Findings

- [x] [Review][Patch] Preserve backend-assigned plugin display when the assigned plugin disappears from inventory entirely [src/windows/main/screens/sequence/components/playback-rack-panel.tsx:201]
- [x] [Review][Patch] Do not move Story 3.4 to review until route-transition and same-session stability proof matches the tasks marked complete [_bmad-output/implementation-artifacts/3-4-keep-route-and-plugin-session-behavior-stable-in-app.md:20] — closed by user direction when marking the story done.

## Dev Notes

### Story Intent

- Story 3.4 should be treated as a same-session stability and regression story.
- The emphasis is preserving what already works and checking for coherence gaps, not adding new behavior surfaces.
- This story exists because the area took time to stabilize and should not be casually reopened through feature creep.
- The most likely acceptable outcome is validation plus a minimal correction if a real instability is found.

### User Decisions Captured

- Treat 3.4 the same way as 3.2 and 3.3: sanity check and validation story.
- Avoid letting this story come in and break a now-stable flow.
- No new UI flow.
- No new complexity unless a proven bug requires the smallest safe fix.

### Bounded Slice Template

#### This Story Owns

- Validating route-to-slot association stability in the current app session.
- Validating coherent route-change transitions in the current supported workflow.
- Validating that frontend state continues to mirror backend-owned route/session truth.
- Minimal bug fixes required to preserve stability.

#### This Story Does Not Own

- New routing workflow surfaces.
- New plugin-session management UI.
- Transport redesign, playback-preparation redesign, or reopen recovery redesign.
- Multi-target or advanced route-authoring expansion.

#### Authoritative State Lives In

- Rust-owned route, slot, and plugin-session runtime truth.
- Existing score-context state and interaction seams in Sequence.

#### Future Stories Intentionally Deferred

- Broader audible-playback and transport trust work.
- Reopen-session recovery semantics.
- Advanced routing/session orchestration beyond the current supported path.

#### Verification Required For Done

- Tests prove the current supported same-session route path remains coherent.
- Tests prove repeated score-context routing interactions do not drift away from backend truth.
- Validation confirms no new route/session UI or parallel state model was introduced.

#### Open Questions Requiring Human Clarification

- None. The direction is explicit: validate stability, preserve working behavior, and keep fixes minimal.

### Scope Boundaries

- In scope:
  - Same-session route stability.
  - Same-session plugin-session usability validation.
  - Coherent route-change transitions.
  - Minimal regression fixes.
- Out of scope:
  - New UI flows.
  - Route/session redesign.
  - Reopen recovery.
  - Audible-playback expansion.

### Cross-Story Alignment

- Story 3.1 owns compact state visibility.
- Story 3.2 owns the bounded route assignment interaction.
- Story 3.3 owns explicit non-ready and recoverable-state honesty.
- Story 3.4 makes sure those current in-app route and session behaviors remain stable when used repeatedly in the same session.
- Later stories should own larger playback/runtime or reopen stability concerns.

### Prior Work Intelligence

- Story 1.4 through Story 1.6 already established slot-owned runtime-session ownership and transient editor attachment; this story must not reopen that foundation casually.
- Existing route flows already read backend-owned `route_target_options_by_slot_id` and use explicit route commands rather than frontend inference.
- Existing row and rack seams, plus their tests, already cover much of the current supported path and should be reused before any code reshaping.
- The strongest risk here is not missing UI. It is destabilizing already-working session behavior through unnecessary changes.

### Architecture Compliance

- Keep Rust as the authoritative owner of route, slot, and plugin-session state.
- Keep React as the consumer of emitted backend truth.
- Do not introduce parallel route-state ownership in the UI.
- Preserve the slot-owned runtime-session foundation rather than replacing it with route- or instrument-owned session logic.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Current route-change seam and same-session interaction entrypoint.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Current score-context rack surface used in repeated route workflows.
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - Existing route interaction regression seam.
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
  - Existing rack-surface regression seam.
- `src/services/actions.ts`
  - Explicit route command seam.
- `src/services/types.ts`
  - Existing backend-owned route choice contract.

### Implementation Guidance

- Prefer validation over redesign.
- If the supported same-session path is already stable, complete the story through tests and notes rather than visible UI or architecture changes.
- If a bug is found, fix it in the existing route/session seam with the narrowest safe change.
- Do not let this story expand into a “better workflow” project.
- Do not use this story to introduce richer route/session abstractions in React.

### Persistence And Migration Requirements

- No persisted-data or migration work is expected for this story.
- If implementation starts requiring persisted session, reopen, or migration changes, stop and re-evaluate because the scope has crossed into later recovery work.

### File Structure Requirements

- Keep implementation centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
  - `src/services/actions.ts`
  - `src/services/types.ts`
- Avoid spreading this story into new windows, new workflow surfaces, or backend redesign unless a minimal stability fix requires it.

### Testing Requirements

- Frontend tests should cover:
  - Repeated use of the current route workflow without losing coherence.
  - Changing a route and preserving valid post-change state.
  - Existing rack and row surfaces continuing to reflect backend truth through repeated same-session interactions.
- Validate with the project-standard commands relevant to touched files:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`

### Library / Framework Requirements

- Reuse the existing state contract, route actions, and current Sequence seams.
- Do not add new state-management layers, session coordinators, or UI libraries for this story.

### Documentation Impact

- Keep implementation-facing notes explicit that Story 3.4 is a stability and validation story over the current supported path.
- No broad documentation update is expected unless a minimal correction changes developer assumptions about the supported in-app route/session behavior.

### Latest Technical Information

- The current route-authoring path is already backend-validated and bounded to current supported choices.
- The slot-owned runtime-session foundation is already an approved baseline and should be protected, not redefined.
- This story should act as a regression gate before later playback work expands scope.

### Project Structure Notes

- This story should remain small and defensive.
- The preferred outcome is “prove it stays stable” rather than “add something new.”
- Any visible new surface or broad refactor is a warning sign that scope has drifted.

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 3.4 active acceptance criteria and planning note.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — trust and stability expectations for ongoing workflow use.
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned route/session truth and trust-gating architecture concerns.
- `docs/project-context.md` — slot-owned session foundation and score-context ownership rules.
- `/memories/repo/sequence-route-authoring-story-3-4.md` — verified note on bounded backend-owned route choices and score-context route flows.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` targeted regression: `keeps showing the backend-assigned plugin when that plugin is no longer selectable`
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` targeted regression: `keeps showing a missing backend-assigned plugin when the inventory entry disappears entirely`
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` targeted regression: `keeps the empty -- state when the slot has no backend-assigned plugin`
- `src/ui/select.test.tsx` targeted regression: `renders option adornments inside the open list`
- `npm run check-types`
- `npm run lint`
- `npm run test:js`

### Completion Notes List

- Validated the existing score-context route workflow against the current row and rack regressions to keep Story 3.4 bounded as a same-session stability gate rather than a new workflow slice.
- Fixed one frontend/backend truth gap in the rack panel: slots now continue showing their backend-assigned plugin even when that plugin is no longer selectable from the usable inventory list.
- Extended that fix to the fully missing-inventory case so backend-assigned plugin truth still appears even when the inventory entry disappears entirely in-session.
- Added a warning adornment for unavailable backend-assigned plugin options while keeping the genuine `--` empty-slot state unchanged.
- Kept the correction seam-local in `PlaybackRackPanel` by preserving the assigned plugin as a disabled display option instead of adding route or session state outside the backend-owned model.
- Added a focused regression proving blocked-or-otherwise-unselectable assigned plugins do not collapse to `--`, preserving same-session route and plugin-session coherence in-app.

### File List

- `_bmad-output/implementation-artifacts/3-4-keep-route-and-plugin-session-behavior-stable-in-app.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/ui/select.test.tsx`
- `src/ui/select.tsx`
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`

## Change Log

- 2026-03-26: Preserved backend-owned slot plugin truth in the Sequence rack panel when an assigned plugin is no longer selectable, and added focused stability regression coverage for Story 3.4.
