# Story 3.2: Show Assignment Readiness in Instrument Rows and Panels

Status: review

## Story

As a composer,
I want to see playback readiness directly on score instruments,
so that I can tell at a glance which parts are playable and which need action.

## Acceptance Criteria

1. Given score instruments have resolved playback assignment state, when they are displayed in relevant score-context surfaces, then instrument rows show compact readiness or health state and the expanded instrument panel shows richer contextual playback information.
2. Given playback state can be Ready, Needs Attention, or Unavailable, when those states are shown in the UI, then they are legible without relying on color alone and the presentation remains consistent with the existing Compose design system.
3. Given an instrument is not ready for playback, when a user inspects that instrument's score-context UI, then they are given one primary contextual recovery path and the product does not force them into an unrelated app-scoped workflow by default.

## Tasks / Subtasks

- [x] Task 1: Add compact readiness presentation to Sequence instrument rows using backend-owned assignment truth only (AC: 1, 2)
  - [x] Update the Sequence instrument overview row to render a compact readiness indicator for each instrument using the existing backend-emitted assignment state.
  - [x] Keep the row compact and information-dense: use iconography and very short labels that remain legible without color alone.
  - [x] Preserve the current assignment control and other row actions without widening this story into a new assignment workflow.
- [x] Task 2: Turn the Sequence bottom-panel instrument surface into the richer score-context playback status panel for the selected instrument (AC: 1, 2, 3)
  - [x] Reuse the existing Sequence bottom-panel seam rather than creating a new window, new Setup-panel flow, or duplicate panel model.
  - [x] Show the selected instrument's current playback readiness, current effective plugin context when available, and plain-language status explanation.
  - [x] Keep the panel contextual to the selected instrument rather than broadening it into full inventory browsing or app-scoped management.
- [x] Task 3: Provide one primary contextual recovery path while preserving slice boundaries (AC: 3)
  - [x] Ensure non-ready states point users toward one score-context recovery action inside the Sequence workflow.
  - [x] Keep that path a lightweight handoff or focus action rather than full assignment-management expansion.
  - [x] Do not force the user into the dedicated Plugin Manager as the default answer for routine readiness issues.
- [x] Task 4: Keep readiness presentation aligned with backend ownership and Epic 3 sequencing (AC: 1, 2, 3)
  - [x] If the current emitted assignment contract is too technical for the promised user-facing labels, add or refine a backend-owned presentation seam instead of letting multiple React surfaces invent their own mapping logic.
  - [x] Do not absorb Story 3.3 manual assignment UX, Story 3.4 target selection, Story 3.5 shared-instance legibility, Story 3.8 activation, or Epic 5 recovery behavior.
  - [x] Keep any new copy, labels, and status grouping consistent across row and panel surfaces, with a bias toward minimal wording over explanatory prose in the row itself.
- [x] Task 5: Add focused seam-level validation and documentation updates for the new readiness UI (AC: 1, 2, 3)
  - [x] Add frontend tests for row and panel readiness rendering, accessibility-friendly status presentation, and the primary contextual handoff behavior.
  - [x] Add backend or contract-level tests only if the emitted readiness presentation seam changes.
  - [x] Update the relevant docs to reflect that Sequence owns score-context readiness display while Plugin Manager remains the app-scoped inventory surface.

## Dev Notes

### Story Intent

- Story 3.2 is the display slice for Epic 3's new backend-owned playback assignment state.
- Its job is to make playback readiness visible in score-context surfaces without turning this slice into manual assignment workflow, runtime activation, or reopen recovery work.
- The compact surface is the Sequence instrument row.
- The richer explanatory surface is the Sequence screen bottom panel for the currently selected instrument.

### User Decisions Captured

- The "expanded instrument panel" for this story is the existing Sequence bottom panel, not the Setup screen and not a new app-scoped window.
- The bottom panel should act as a score-context playback status panel for the currently selected instrument.
- The panel exists to explain playback readiness, current context, and next action, not to become a second Plugin Manager.
- The primary recovery path should stay inside the Sequence workflow as a lightweight contextual handoff.
- Story 3.3 still owns the fuller musician-facing manual assignment workflow.
- A direct "open selected plugin UI" button in the Sequence instrument row is not part of Story 3.2. Native plugin UI remains a later secondary handoff once the user-facing editor-host path is intentionally introduced beyond the current bounded maintainer seam.
- Status presentation should be compact and low-wordiness by default: short labels, strong placement, and icon support rather than verbose descriptive chips in the row.

### Bounded Slice Template

#### This Story Owns

- Showing compact readiness state on Sequence instrument rows.
- Showing richer contextual playback status for the selected instrument in the Sequence bottom panel.
- Providing one primary score-context recovery handoff for non-ready instruments.
- Keeping row and panel status presentation aligned to backend-owned truth and existing Compose visual language.

#### This Story Does Not Own

- A new dedicated window or a Setup-screen playback detail model.
- Full plugin inventory browsing, refresh controls, scan health management, or other Plugin Manager responsibilities.
- A redesigned or expanded manual assignment workflow. Story 3.3 owns that.
- A direct row-level native plugin-editor launch action.
- Multi-part target selection. Story 3.4 owns that.
- Shared-instance explanation beyond the bounded readiness context needed here. Story 3.5 owns the fuller shared-instance legibility slice.
- Plugin activation/loading for playback. Story 3.8 owns that.
- Reopen, persistence recovery, or degraded restore behavior. Epic 5 owns that.

#### Authoritative State Lives In

- Rust-owned resolved playback assignment state emitted through the existing general app state.
- The current score-context selection and Sequence bottom-panel state already used by the main window.
- The dedicated Plugin Manager window remains the canonical app-scoped inventory browser.

#### Future Stories Intentionally Deferred

- Richer musician-friendly assignment controls and picker workflow.
- User-facing native plugin-editor launch from score-context surfaces once that path is promoted beyond the current maintainer-only proof seam.
- Explicit target and channel selection.
- Shared-instance UI legibility beyond readiness context.
- Activation/runtime lifecycle and reopen recovery semantics.

#### Verification Required For Done

- Frontend tests prove rows and the selected-instrument panel render readiness consistently from backend-owned state.
- Tests prove non-ready states expose one primary contextual recovery path without routing by default into app-scoped inventory management.
- If the emitted state contract changes, the TypeScript mirror and contract-level coverage are updated together.
- Validation uses the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None. The Sequence bottom-panel decision and bounded recovery-path intent have been clarified.

### Scope Boundaries

- In scope:
  - Sequence instrument-row readiness display.
  - Sequence bottom-panel playback status display for the selected instrument.
  - One primary contextual handoff for non-ready states.
  - Minimal contract refinement only if required to keep readiness presentation backend-owned.
- Out of scope:
  - New inventory-management UI.
  - Broad assignment authoring workflow changes.
  - Runtime plugin-loading behavior.
  - Recovery persistence or app reopen behavior.

### Cross-Story Alignment

- Story 3.1 already established the backend-owned assignment-intent and resolved-state seam this story must consume.
- Story 3.2 should make that seam visible without inventing new playback truth in React.
- Story 3.3 can build the richer manual assignment UX on top of the same score-context row and bottom-panel surfaces clarified here.
- Story 3.4 and Story 3.5 can later enrich the panel's explanation with target and shared-instance detail without replacing the Story 3.2 presentation structure.
- Story 3.8 should be able to layer runtime activation on top of the same readiness surfaces rather than redefining them.

### Previous Story Intelligence

- Story 3.1 already emits `resolved_playback_assignment_by_instrument_id` for score-context consumers.
- Story 3.1 deliberately kept React from deriving assignment health locally; Story 3.2 must preserve that rule.
- Story 3.1 also preserved the existing row assignment control as a compatibility seam, which means Story 3.2 should reuse current score-context surfaces rather than inventing a parallel playback panel model.

### Architecture Compliance

- Rust remains the sole authoritative owner of assignment truth and any derived readiness presentation state that crosses the FE/BE boundary.
- React remains a presentation and interaction layer over that backend-owned state.
- Score-context playback relationships stay in contextual Sequence surfaces.
- Plugin Manager stays app-scoped and canonical for full inventory browsing.
- Readiness presentation must be accessible without relying on color alone.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing compact row surface for each score instrument.
  - Already reads the emitted resolved assignment state but only uses the effective plugin id today.
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - Existing Sequence bottom-panel seam that should become the richer score-context playback status panel for this story.
- `src/windows/main/components/plugins.tsx`
  - Current bottom-panel instruments/plugins surface contains legacy SCVI and VST-debugging residue.
  - This story should repurpose or constrain that seam toward selected-instrument playback status instead of app-scoped inventory browsing.
- `src/windows/main/hooks/use-bottom-panel-state.ts`
  - Existing selected-instrument context for the Sequence bottom panel.
  - Reuse this selection-driven seam rather than creating a separate selection model.
- `src/services/types.ts`
  - Current frontend contract mirrors `PlaybackAssignmentStatus` and `ResolvedPlaybackAssignment`.
  - If user-facing presentation needs a more precise backend-owned summary, update the contract here in lockstep with Rust.
- `src/services/state/context.tsx`
  - Existing general-state mirror already carries `resolved_playback_assignment_by_instrument_id`.
- `src-tauri/src/score/instruments/mod.rs`
  - Current backend assignment-resolution seam.
  - If the user-facing `Ready / Needs Attention / Unavailable` presentation cannot be expressed cleanly from the current status model, refine this seam here rather than in ad hoc React helpers.

### Implementation Guidance

- Use the backend-owned resolved assignment state as the only source of readiness truth.
- Keep row status compact: icon, short label, and consistent placement should do the work rather than color alone.
- Keep the panel explanatory but concise: tell the user what Compose thinks the current state is, why it is in that state, and what the next contextual action should be without drifting into verbose troubleshooting copy.
- Reuse the existing Sequence bottom-panel selection and navigation seams.
- Prefer a single contextual recovery handoff over multiple equal-weight troubleshooting options.
- Do not default users into the dedicated Plugin Manager for ordinary readiness problems.
- Do not add a direct row-level "open plugin UI" button in this story. The current native editor host remains a bounded maintainer seam, and the user-facing score-context launch path should be introduced later as a deliberate secondary handoff rather than smuggled into the readiness slice.
- If the current backend enum values (`Unassigned`, `Ready`, `NeedsAttention`) are too close to internal semantics rather than the UX-required presentation (`Ready`, `Needs Attention`, `Unavailable`), fix that at the backend-owned presentation seam instead of letting each component choose its own wording and grouping.
- Avoid broad redesign of the existing row assignment select or bottom-panel toolbars in this slice.

### Persistence And Migration Requirements

- No `.soloc` persistence change is expected for this story if it remains a display slice.
- If implementation unexpectedly changes any emitted contract only, do not add a persisted-data migration.
- If work drifts toward persisted state changes, stop and re-evaluate because that likely means the slice boundary has been crossed.

### File Structure Requirements

- Frontend presentation work should stay centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - `src/windows/main/components/plugins.tsx` or a closely related Sequence bottom-panel component seam if that surface is being repurposed
  - `src/windows/main/hooks/use-bottom-panel-state.ts`
- Contract changes, if needed, should stay centered in:
  - `src/services/types.ts`
  - `src/services/state/context.tsx`
  - `src-tauri/src/score/instruments/mod.rs`
- Do not spread this story into Plugin Manager window code unless a tiny contextual handoff hook is genuinely required.

### Testing Requirements

- Frontend tests should cover:
  - row rendering for all readiness states shown to users
  - panel rendering for the selected instrument across ready and non-ready cases
  - accessible status presentation that does not rely on color alone
  - one primary contextual recovery action for non-ready states
- Contract-level tests should cover any backend-owned presentation refinement introduced to support the user-facing state labels.
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings` only if Rust/backend contract changes are made

### Library / Framework Requirements

- Stay on the existing brownfield React/Tauri stack and repo conventions.
- Reuse the current icon, tokenized text, and design-system patterns already present in Sequence and Plugin Manager surfaces.
- Keep FE/BE contracts `snake_case` and enum alignment explicit if the backend contract changes.
- Do not add new state-management libraries or frontend-owned playback models.

### Documentation Impact

- Update `docs/architecture.md` or `docs/development-guide.md` if implementation changes how the Sequence bottom panel is described as a score-context playback surface.
- Keep the docs explicit that Plugin Manager remains the app-scoped inventory-management window.
- Update any component or developer guidance that still frames the Sequence bottom-panel plugin surface as primarily legacy SCVI/debugging scaffolding once this story gives it a score-context playback role.

### Git Intelligence Summary

- Recent work kept Epic 3 disciplined so far:
  - `ed1a09f8 feat(3.1): playback assignment setup`
  - `6f2d04fa chore: epic 2 retro`
  - `cfc3fe3e feat(2.8): abandon plugin image`
  - `ffaad55a feat(2.6): plugin ui`
  - `f2bb4a6a feat(2.5): bind plugin ui`
- Preserve that pattern here: deliver a clear readiness-display slice rather than compressing future assignment and runtime work into one story.

### Latest Technical Information

- No new external library or framework choice is required for this story.
- The current repo already has backend-owned resolved assignment state, selected-instrument Sequence context, and status-chip/icon patterns that should be reused rather than replaced.

### Project Structure Notes

- The current Sequence bottom-panel instruments/plugins surface is the right structural seam for this story, but its purpose needs to shift from legacy debugging residue toward selected-instrument playback context.
- The Setup screen is not the intended panel surface for Story 3.2.
- The dedicated Plugin Manager remains the canonical app-scoped inventory browser and should not be duplicated here.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.2 requirements
- `_bmad-output/planning-artifacts/prd.md` — FR10 and Epic 3 assignment requirements
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned playback state and score-context versus app-scoped boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — direct assignment surfaces, three-state playback model, and contextual recovery guidance
- `_bmad-output/implementation-artifacts/3-1-represent-playback-assignment-state-on-score-instruments.md` — previous story learnings and current backend seam
- `_bmad-output/implementation-artifacts/epic-2-retro-2026-03-22.md` — bounded-slice guardrails for Story 3.2
- `docs/project-context.md` — FE/BE ownership rules and Sequence/Plugin Manager boundary guidance
- `docs/architecture.md` — current assignment seam and app-state flow
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — compact row surface
- `src/windows/main/screens/sequence/components/bottom-panel.tsx` — Sequence bottom-panel seam
- `src/windows/main/components/plugins.tsx` — current bottom-panel plugin surface
- `src/windows/main/hooks/use-bottom-panel-state.ts` — selected-instrument panel context
- `src/services/types.ts` — frontend playback assignment contract
- `src-tauri/src/score/instruments/mod.rs` — backend assignment resolution seam

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `3.2`.
- Sprint status showed `3-2-show-assignment-readiness-in-instrument-rows-and-panels` in `backlog` before story creation.
- Artifact review confirmed Epic 3 must preserve bounded-slice discipline and keep assignment, activation, and recovery as separate slices.
- UX and architecture review confirmed the intended compact surface is the Sequence instrument row and the intended richer surface is a score-context panel rather than app-scoped inventory management.
- Human clarification confirmed the "expanded instrument panel" for this story is the Sequence screen bottom panel.
- Human clarification aligned with the recommended bounded recovery path: keep the panel score-contextual and leave the fuller manual assignment workflow to Story 3.3.
- Implemented shared readiness presentation mapping in Sequence-only presentation helpers without changing the backend contract.
- Replaced the Sequence Instruments bottom-panel inventory browse seam with a selected-instrument playback status panel and an in-score focus handoff to the existing row assignment control.
- Validated the completed slice with `runTests` (1145 passed), `npm run check-types`, and `npm run lint`.

### Completion Notes List

- Added compact readiness badges to Sequence instrument rows while preserving the existing assignment select and row actions.
- Added a selected-instrument playback status panel in the Sequence bottom panel with concise readiness copy, resolved plugin context, and one primary in-score recovery handoff.
- Kept readiness mapping centralized in a shared Sequence presentation seam and left the Rust/TypeScript playback assignment contract unchanged.
- Added focused frontend coverage for row readiness states, panel rendering, and the recovery focus handoff.
- Updated architecture and development guidance so Plugin Manager remains the app-scoped inventory surface while Sequence owns score-context readiness display.
- Validation complete: `runTests` passed, `npm run check-types` passed, and `npm run lint` passed.

### File List

- `_bmad-output/implementation-artifacts/3-2-show-assignment-readiness-in-instrument-rows-and-panels.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/development-guide.md`
- `src/windows/main/hooks/use-bottom-panel-state.ts`
- `src/windows/main/screens/sequence/components/bottom-panel.test.tsx`
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
- `src/windows/main/screens/sequence/components/playback-readiness.tsx`
- `src/windows/main/screens/sequence/components/playback-status-panel.test.tsx`
- `src/windows/main/screens/sequence/components/playback-status-panel.tsx`

## Change Log

- 2026-03-22: Story created with bounded Sequence-row and Sequence-bottom-panel readiness scope, explicit score-context recovery guidance, and sprint status advanced to ready-for-dev.
- 2026-03-22: Implemented Sequence row readiness badges, selected-instrument playback status panel, primary in-score recovery handoff, focused frontend tests, and documentation updates; story advanced to review.
