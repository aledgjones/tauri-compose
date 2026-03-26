# Story 3.2: Assign And Change Single-Target Playback Routes

Status: ready-for-dev

## Story

As a composer,
I want to assign or change a single-target playback route for an instrument,
so that I can connect a musical part to a usable rack slot without restructuring the score.

## Acceptance Criteria

1. Given an instrument has no playback route or an existing route, when I use the existing score-context routing dropdown in the instrument row, then I can assign, change, or clear that instrument's single-target playback route through the rack-first model.
2. Given the current MVP is still single-target-first, when I use that routing dropdown, then the workflow remains bounded to the existing slot-selection interaction and does not introduce extra target pickers, address editors, panels, or setup flows.
3. Given only some routes are currently usable, when the supported routing dropdown presents choices, then it stays aligned to backend-owned contextual validity and does not broaden the UI into richer choice handling that belongs to later stories.
4. Given this story is intended as a sanity-check slice over an already-present control, when implementation begins, then the developer should prefer proving the current dropdown is sufficient or making the smallest necessary correction rather than inventing new routing UI.

## Tasks / Subtasks

- [ ] Task 1: Treat the existing instrument-row dropdown as the supported Story 3.2 routing control (AC: 1, 2, 4)
  - [ ] Reuse the current Sequence instrument-row route dropdown as the only supported interaction surface for assigning, changing, or clearing a route.
  - [ ] Keep the interaction framed as instrument-to-rack-slot routing rather than legacy assignment language.
  - [ ] Do not add a second routing surface, wizard, popover, target picker, or panel flow for this story.
- [ ] Task 2: Keep the route-authoring slice bounded to the current single-target MVP (AC: 1, 2, 3)
  - [ ] Preserve the current single-target-first workflow.
  - [ ] Do not expose address editing, multi-target choice, multi-instrument VST refinement, or channel-management UI in this story.
  - [ ] If backend-owned richer target data exists, do not widen the row interaction to expose it here.
- [ ] Task 3: Sanity-check backend validity boundaries without adding UI complexity (AC: 3, 4)
  - [ ] Verify the dropdown remains aligned with backend-owned route validity for the supported MVP path.
  - [ ] If a small correction is needed so invalid or unsupported routes are not treated as ordinary ready choices, keep that correction minimal and within the existing control.
  - [ ] Do not solve unsupported richer target-selection cases by adding new inline controls in this story.
- [ ] Task 4: Keep failure and explanatory behavior bounded (AC: 2, 4)
  - [ ] Do not expand the row into a diagnostic or setup surface while touching route assignment behavior.
  - [ ] Avoid adding new inline explanatory boxes, address detail, or workflow copy in the name of route authoring.
  - [ ] If implementation touches nearby error handling, keep any cleanup strictly minimal and avoid broad UX redesign.
- [ ] Task 5: Validate the existing dropdown path with focused tests and notes (AC: 1, 2, 3, 4)
  - [ ] Update or confirm tests for assign, change, and clear behavior through the current dropdown.
  - [ ] Add or refine tests only where needed to prove the bounded MVP interaction and backend-valid choice behavior.
  - [ ] Document explicitly that Story 3.2 keeps the current dropdown model and does not introduce a more complex routing UI.

## Dev Notes

### Story Intent

- Story 3.2 should be treated as a minimal route-authoring slice over an interaction that is already largely present.
- The intended control is the existing instrument-row dropdown.
- The user does not want extra complexity here.
- This story should validate or lightly correct the current slot-selection path, not design a richer routing system.

### User Decisions Captured

- This is essentially the same interaction that already exists.
- The dropdown used to assign a slot to an instrument is the intended control.
- Avoid adding complexity at this stage.
- Prefer a sanity check and minimal correction over new UI.

### Bounded Slice Template

#### This Story Owns

- Assigning, changing, and clearing a route through the existing instrument-row dropdown.
- Keeping the workflow single-target-first and rack-first.
- Minimal validation that the current control behaves correctly for the supported MVP path.

#### This Story Does Not Own

- New route-target selection UI.
- Address editing or address display.
- Multi-target or multi-instrument VST workflows.
- Recovery UX beyond what already exists in bounded form.
- Plugin Manager, rack panel redesign, or new score-context route surfaces.

#### Authoritative State Lives In

- Rust-owned route validity and route target choice state.
- Existing Sequence row presentation and action seams in React.

#### Future Stories Intentionally Deferred

- Richer target disambiguation workflows.
- Advanced route repair or recovery flows.
- Multi-target, channel-addressed, or multi-timbral authoring UI.

#### Verification Required For Done

- Tests prove the current dropdown can assign, change, and clear a supported route.
- Tests prove the story remains bounded to the existing control.
- Validation confirms no new route-authoring UI was introduced without a clear need.

#### Open Questions Requiring Human Clarification

- None. The preferred direction is explicit: keep the dropdown, keep the flow simple, and avoid new complexity.

### Scope Boundaries

- In scope:
  - Existing instrument-row dropdown route assignment.
  - Minimal route change and clear behavior.
  - Focused sanity-check around backend-valid choices.
- Out of scope:
  - New control types.
  - Advanced target-selection UX.
  - Address-level editing or display.
  - Broader playback setup redesign.

### Cross-Story Alignment

- Story 3.1 owns compact playback-state visibility on the row.
- Story 3.2 owns the bounded route assignment interaction through the existing dropdown.
- Story 3.3 and later stories own deeper unrecoverable-state handling and richer repair flows.
- Future advanced routing stories should reopen richer targeting only when the product is ready for that complexity.

### Prior Work Intelligence

- The current row already includes a dropdown backed by `assignSlotRoute` and `unassignRoute` actions.
- The current implementation already reads backend-owned `route_target_options_by_slot_id`.
- The current implementation also contains an explicit unsupported richer-target guard, which is a sign to keep this story bounded rather than adding a new chooser.
- Existing row tests already cover much of the current interaction seam and should be reused before creating new UI.

### Architecture Compliance

- Keep Rust as the source of truth for route validity and route options.
- Keep React as the thin interaction layer over the existing row control.
- Preserve the single-target-first MVP boundary.
- Do not introduce a parallel frontend-owned route-authoring model.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing dropdown, route mutation path, and current bounded interaction seam.
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - Existing tests for assign, change, clear, and unsupported richer-target behavior.
- `src/services/actions.ts`
  - Existing `assignSlotRoute` and `unassignRoute` action seams.
- `src/services/types.ts`
  - Existing `route_target_options_by_slot_id` contract.

### Implementation Guidance

- Prefer no UX change if the current dropdown already satisfies the bounded story after Story 3.1 alignment.
- If a correction is required, keep it inside the existing dropdown seam.
- Do not add a richer target chooser to solve unsupported edge cases in this story.
- Do not turn this story into a route diagnostics or recovery redesign.
- Treat any unsupported richer-target path as evidence that the scope should stay bounded, not that the row needs to grow.

### Persistence And Migration Requirements

- No persisted-data or migration work is expected for this story.
- If implementation starts requiring schema changes or deeper route-model work, stop and re-evaluate because the slice boundary has been crossed.

### File Structure Requirements

- Keep implementation centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `src/services/actions.ts`
  - `src/services/types.ts`
- Avoid spreading this story into new windows, panels, or backend model seams unless a proven minimal correction requires it.

### Testing Requirements

- Frontend tests should cover:
  - Assigning a route from the existing dropdown.
  - Changing a route from one slot to another.
  - Clearing a route through the unrouted option.
  - Bounded behavior when a slot would require richer target disambiguation than this story supports.
- Validate with the project-standard commands relevant to touched files:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`

### Library / Framework Requirements

- Reuse the existing Select control, action seams, and state contract.
- Do not add new routing UI libraries, tooltip systems, or modal flows for this story.

### Documentation Impact

- Keep implementation-facing notes explicit that Story 3.2 uses the existing dropdown and does not authorize richer routing UI.
- No broad documentation update is expected unless a minimal correction changes developer assumptions about the supported MVP route-selection path.

### Latest Technical Information

- The current implementation already provides the intended dropdown interaction.
- Backend-owned route target choices are already emitted to the frontend.
- The code already guards against unsupported richer-target selection, which supports keeping this story minimal.

### Project Structure Notes

- This story should stay small.
- The most likely acceptable outcome is validation plus a minor correction, not a new UI build.
- If the current implementation is already sufficient for the supported MVP path, the story may complete with test and note updates rather than visible UI change.

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 3.2 active acceptance criteria.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — single-target-first score-context routing expectations.
- `docs/project-context.md` — backend-owned route truth and compact score-context UI boundaries.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — existing dropdown seam.
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` — existing test seam.
- `/memories/repo/sequence-route-authoring-story-3-4.md` — later verified note that backend emits bounded route choices and unsupported richer targets should not drive more UI here.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

### Completion Notes List

### File List
