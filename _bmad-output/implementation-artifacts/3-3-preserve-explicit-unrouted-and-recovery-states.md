# Story 3.3: Preserve Explicit Unrouted And Recovery States

Status: ready-for-dev

## Story

As a composer,
I want unrouted and unresolved instruments to remain explicit and recoverable,
so that missing playback setup does not break my score or get hidden behind fallback behavior.

## Acceptance Criteria

1. Given an instrument has no suitable playback mapping or a previously valid route cannot currently resolve, when Compose evaluates playback state, then that state remains explicitly unrouted or non-ready instead of silently falling back and the score remains usable.
2. Given a non-ready instrument is shown in existing score-context surfaces, when I inspect its state, then the current score-context routing and rack seams remain the primary recovery path and the story does not invent a new repair UI flow.
3. Given playback readiness is backend-owned domain state, when unresolved or missing-route conditions are surfaced, then they preserve user intent and route truth instead of inventing frontend-only fallback state.
4. Given this story is intended as a validation-and-gap-check slice, when implementation begins, then the developer should prefer proving the current behavior is honest and recoverable or making the smallest necessary bug fix rather than adding UI complexity.

## Tasks / Subtasks

- [ ] Task 1: Validate that unrouted and unresolved playback states stay explicit in backend-owned truth (AC: 1, 3, 4)
  - [ ] Confirm instruments with no playable mapping remain explicitly unrouted or non-ready rather than silently appearing ready.
  - [ ] Confirm previously valid but currently unresolved routes preserve the intended route truth rather than collapsing into hidden fallback behavior.
  - [ ] Confirm score work remains usable even when playback is non-ready.
- [ ] Task 2: Keep recovery bounded to existing score-context seams (AC: 2, 4)
  - [ ] Treat the current score-context rack and routing seams as the primary recovery path.
  - [ ] Do not introduce a new repair button, wizard, panel, modal, or app-scoped flow for this story.
  - [ ] If a gap exists, prefer wiring or validating the existing score-context path rather than inventing a new one.
- [ ] Task 3: Preserve user intent and avoid frontend-only fallback logic (AC: 1, 3, 4)
  - [ ] Verify the surfaced non-ready state reflects backend-owned truth.
  - [ ] Do not add frontend-only logic that masks unresolved routes or pretends playback is ready.
  - [ ] Keep any necessary correction minimal and within existing state-consumption seams.
- [ ] Task 4: Keep explanatory behavior and UI changes strictly minimal (AC: 2, 4)
  - [ ] Do not broaden compact row or rack UI into a richer recovery workflow.
  - [ ] Do not add inline diagnostic prose, status boxes, or extra recovery controls to satisfy the story.
  - [ ] If implementation touches nearby non-ready messaging, keep it bounded and aligned with the already-approved minimal UI direction.
- [ ] Task 5: Add focused regression coverage for honest non-ready and recoverable behavior (AC: 1, 2, 3, 4)
  - [ ] Update or confirm tests for explicit unrouted and unresolved states.
  - [ ] Verify tests prove score-context seams remain usable when playback is non-ready.
  - [ ] Document explicitly that Story 3.3 is primarily a validation and bug-gap story, not a new recovery UX story.

## Dev Notes

### Story Intent

- Story 3.3 is about honesty and recoverability, not new interface design.
- "Recoverable" here means the non-ready state remains explicit, the score remains usable, and the user can continue through existing score-context seams.
- This story should check for gaps or bugs in how unrouted and unresolved states are preserved and surfaced.
- It should not be used to introduce new recovery UI flows.

### User Decisions Captured

- No new UI.
- Do not invent new UI flows for recovery.
- Treat this story mainly as validation of what is already there, checking for gaps or bugs.

### Bounded Slice Template

#### This Story Owns

- Validating that unrouted and unresolved states remain explicit.
- Validating that score-context recovery remains possible through existing seams.
- Minimal bug fixes needed to preserve honest non-ready state and recoverability.

#### This Story Does Not Own

- New recovery controls or repair flows.
- Plugin Manager escalation design.
- Richer inline diagnostics or expanded non-ready explanation.
- Reopen-session recovery or persistence repair behavior beyond current in-app state integrity.

#### Authoritative State Lives In

- Rust-owned route and readiness truth.
- Existing Sequence row and rack score-context surfaces that consume that truth.

#### Future Stories Intentionally Deferred

- Broader reopen and restore recovery.
- Advanced repair workflows.
- Richer route diagnostics and inventory-driven repair UX.

#### Verification Required For Done

- Tests prove explicit unrouted and unresolved conditions remain visible and honest.
- Tests prove the score stays usable and current score-context seams remain the recovery path.
- Validation confirms no new recovery UI flow was introduced without necessity.

#### Open Questions Requiring Human Clarification

- None. The preferred direction is explicit: validate the current behavior, fix gaps if needed, and do not add new UI.

### Scope Boundaries

- In scope:
  - Honest explicit unrouted or unresolved state.
  - Existing score-context recovery path validation.
  - Minimal bug fixes if gaps are found.
- Out of scope:
  - New repair UI.
  - New contextual actions.
  - New app-scoped recovery flow.
  - Session reopen and restoration redesign.

### Cross-Story Alignment

- Story 3.1 owns compact state visibility.
- Story 3.2 owns the existing bounded dropdown route interaction.
- Story 3.3 makes sure failure and non-ready states stay explicit and recoverable without lying to the user.
- Later recovery and persistence stories should own broader degraded reopen behavior and deeper repair flows.

### Prior Work Intelligence

- The current row and playback presentation already track explicit unrouted and needs-attention conditions.
- The current code also shows there is still inline route-error alert behavior nearby, which should not be allowed to grow into a larger recovery UX under this story.
- Existing planning consistently says missing or invalid routes must degrade to recoverable explicit states instead of silent fallback.
- Existing score-context seams should be preferred over new UI flows.

### Architecture Compliance

- Keep Rust as the source of truth for non-ready and recoverable playback states.
- Keep React as a thin consumer of that state.
- Preserve the score-context versus app-scoped boundary: routing, readiness, and recovery stay in score-context surfaces first.
- Do not add frontend-only fallback logic to make non-ready routes appear resolved.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing score-context row seam where unrouted state, route changes, and current route errors are surfaced.
- `src/windows/main/screens/sequence/components/playback-readiness.tsx`
  - Existing compact warning affordance seam for non-ready playback.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Existing mapping of backend truth into compact surfaced states.
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - Existing regression seam for unrouted and non-ready row behavior.

### Implementation Guidance

- Prefer validation over redesign.
- If everything already behaves honestly and recoverably in the supported path, the story may complete mostly through tests and small fixes.
- If a real gap exists, fix it in the existing score-context seam.
- Do not solve recoverability by adding a new UI path.
- Do not let this story become a diagnostic or troubleshooting workflow build-out.

### Persistence And Migration Requirements

- No persisted-data or migration work is expected for this story.
- If implementation starts requiring reopen-state, persistence, or migration changes, stop and re-evaluate because the scope has crossed into later recovery work.

### File Structure Requirements

- Keep implementation centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- Avoid spreading this story into new windows, new panels, persistence seams, or Plugin Manager workflows unless a proven minimal fix requires it.

### Testing Requirements

- Frontend tests should cover:
  - Explicit unrouted state staying visible.
  - Explicit unresolved or needs-attention state staying visible.
  - Non-ready playback not silently appearing ready.
  - Existing score-context seams remaining usable when playback is non-ready.
- Validate with the project-standard commands relevant to touched files:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`

### Library / Framework Requirements

- Reuse the existing row, readiness, and test seams.
- Do not add new recovery UI libraries, modal flows, or tooltip systems for this story.

### Documentation Impact

- Keep implementation-facing notes explicit that Story 3.3 is a validation and bug-gap story over existing score-context recovery behavior.
- No broad documentation update is expected unless a minimal fix changes current developer assumptions about explicit unrouted or unresolved handling.

### Latest Technical Information

- Planning and architecture both require explicit recoverable unrouted or attention-needed states instead of silent fallback.
- The current score-context surfaces already expose non-ready state and route-change seams, which supports keeping this story bounded.
- Later recovery work should own reopen or inventory-re-resolution behavior rather than backfilling it into this story.

### Project Structure Notes

- This story should stay small and verification-heavy.
- The likely acceptable outcome is confirming the current supported path plus fixing a narrow bug if one is found.
- The story should not produce a visible new recovery feature unless a true gap in the existing seam forces a minimal correction.

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 3.3 active acceptance criteria.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — recoverable playback states and one primary score-context recovery action.
- `_bmad-output/planning-artifacts/architecture.md` — explicit recoverable unrouted or attention-needed states and score-context ownership.
- `docs/project-context.md` — backend-owned truth and score-context UI boundaries.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — current score-context row seam.
- `src/windows/main/screens/sequence/components/playback-readiness.tsx` — compact non-ready affordance seam.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts` — compact state mapping seam.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

### Completion Notes List

### File List
