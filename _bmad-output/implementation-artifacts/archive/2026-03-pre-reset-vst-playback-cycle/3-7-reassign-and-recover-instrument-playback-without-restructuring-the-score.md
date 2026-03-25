# Story 3.7: Repair And Reroute Instrument Playback Without Restructuring The Score

Status: deferred

Deferred reason: the current MVP will rely on the existing manual route-change path and explicit readiness messaging. A dedicated reroute or repair workflow refinement is useful later, but it is not required before Epic 4 playback hookup.

## Story

As a composer,
I want to change or repair an instrument's playback route directly,
so that a later UX refinement can reduce recovery friction without blocking first-audible playback work.

## Acceptance Criteria

1. Given a score instrument already has a route or attention-needed state, when the user selects a different usable slot or target from the contextual routing workflow, then the route is updated for that instrument and the score structure itself does not need to change.
2. Given an instrument is unresolved or needs attention, when the user follows the primary contextual recovery action, then they can reach the relevant rerouting path quickly and the workflow remains anchored in the score context.
3. Given rerouting occurs, when the new route is resolved, then readiness state updates accordingly and prior unresolved state does not linger as stale UI.

## Tasks / Subtasks

- [ ] Task 1: Extend the Sequence score-context recovery workflow so the selected instrument can be rerouted from the rack panel without leaving the score workflow (AC: 1, 2)
  - [ ] Reuse the existing selected-instrument state from the Sequence bottom-panel flow so `PlaybackRackPanel` can act on the currently selected instrument instead of remaining slot-only.
  - [ ] Add one bounded reroute surface in the rack panel for the selected instrument, including current route summary, a clear route action, and a direct way to route the selected instrument to a shown slot.
  - [ ] Keep the workflow anchored in Sequence. Do not introduce a new window, a plugin-manager-first workflow, or any DAW-style routing surface.
- [ ] Task 2: Build reroute and repair flows on top of the existing backend-owned route seams rather than inventing new playback truth (AC: 1, 3)
  - [ ] Reuse the existing explicit `score_instruments_assign_slot_route` and `score_instruments_unassign_route` actions for route mutation.
  - [ ] Preserve the rack-first invariant: instruments reroute only to existing slots; rerouting or clearing a route must not restructure players, instruments, or slot ownership.
  - [ ] Preserve the existing backend rule that routing to an existing empty slot is valid persisted intent that resolves as explicit `NeedsAttention`, not silent fallback readiness.
- [ ] Task 3: Close the current recovery gap for slots that need an explicit backend-verified target choice (AC: 1, 2, 3)
  - [ ] Keep the compact instrument-row picker as the fast path for simple direct reroutes, but hand off multi-target or otherwise ambiguous reroute cases into the selected-instrument rack workflow instead of failing with a dead-end error.
  - [ ] Use backend-owned `route_target_options_by_slot_id` to present only currently valid target choices for the chosen slot; React must not infer or synthesize targets.
  - [ ] If the backend cannot prove a valid richer target for a slot, expose only the verified direct route or keep the instrument in a recoverable state instead of guessing a part, channel, replacement slot, or shared-instance topology.
- [ ] Task 4: Ensure reroute state refresh is immediate and stale attention state cannot linger after success (AC: 3)
  - [ ] Drive post-reroute UI entirely from refreshed backend-owned `resolved_playback_route_by_instrument_id` state.
  - [ ] Clear stale route errors and stale unresolved presentation once a new route resolves successfully.
  - [ ] Preserve the compact `Ready`, `Needs Attention`, and `Unavailable` presentation contract without adding frontend-owned readiness derivation.
- [ ] Task 5: Add focused validation and documentation updates for the recovery workflow (AC: 1, 2, 3)
  - [ ] Add frontend tests for selected-instrument reroute controls in `PlaybackRackPanel`, including route-here, clear-route, empty-slot recovery, and multi-target handoff or choice presentation.
  - [ ] Add or update frontend tests for `InstrumentOverviewRow` so simple slot changes remain direct while ambiguous cases hand off into the rack workflow cleanly.
  - [ ] Add backend tests only if route validation or resolved-state behavior changes; otherwise rely on the existing route-validation seam and extend the UI-facing test coverage around state refresh.
  - [ ] Update documentation so Sequence remains the primary reroute and recovery surface, Plugin Manager stays app-scoped, and empty-slot reroutes remain explicit recoverable intent rather than hidden failure.

## Dev Notes

### Story Intent

- Story 3.7 is now an explicitly deferred UX refinement rather than part of the current Epic 3 completion path.
- Story 3.4 already introduced the explicit route command and the row-level fast path for direct slot changes, which is sufficient for the current MVP runway.
- This story can later make rerouting feel more like a score-context recovery action inside Sequence without blocking playback hookup.

### User Decisions Captured

- Sequence remains the score-context workflow surface for slot preparation, route authoring, and route repair.
- Plugin Manager remains app-scoped inventory browsing and is not the default answer for ordinary reroute or recovery work.
- Shared-slot and multi-target routing remain deferred, so Story 3.7 must not introduce hidden shared-slot reuse, guessed channel selection, or speculative multi-target behavior.
- Automatic routing remains deferred by user decision, so new instruments continue to rely on explicit manual routing and repair rather than automatic routing heuristics.
- Empty playback slots remain valid rack state, and a route to an existing empty slot is valid persisted intent that should resolve as `NeedsAttention` until the slot is prepared.

### Bounded Slice Template

#### This Story Owns

- Selected-instrument reroute and clear-route affordances inside the Sequence rack workflow.
- A primary contextual recovery path from non-ready instrument state into that reroute workflow.
- Explicit target selection in the score-context reroute flow when the chosen slot exposes more than one backend-verified target.
- Immediate UI refresh after reroute so stale unresolved state and stale error messaging do not linger.

#### This Story Does Not Own

- Slot creation or plugin assignment as a new product slice. Story 3.3 already owns slot preparation.
- Re-implementing the basic explicit route mutation seam from Story 3.4.
- Shared-slot reuse, part-sharing UX, or broader multi-target sophistication from Story 3.5.
- Automatic routing for new instruments from Story 3.6.
- Runtime plugin activation, audible playback, or transport behavior from Stories 3.8 and later epics.
- App-scoped inventory management, plugin troubleshooting workflows, or native plugin editor redesign.

#### Authoritative State Lives In

- Rust-owned persisted `Instrument.playback_route` intent.
- Rust-owned rack and slot state on `score.playback.rack`.
- Rust-owned `resolved_playback_route_by_instrument_id` and `route_target_options_by_slot_id` general-state payloads.
- Backend-owned selected route validation in `score.assign_instrument_route(...)` and related route-resolution helpers.

#### Future Stories Intentionally Deferred

- Shared-slot reuse and richer multi-target proof.
- Automatic routing on instrument creation.
- Runtime activation of slots and targets for playback.
- Reopen recovery and transport-safe live-update rules.

#### Verification Required For Done

- Tests prove a user can repair or change the selected instrument's route from the Sequence rack workflow without restructuring the score.
- Tests prove the recovery path stays in score context for unresolved or attention-needed instruments.
- Tests prove rerouting to an existing empty slot remains explicit recoverable state rather than hidden success or silent failure.
- Tests prove stale unresolved or stale error UI does not linger after successful reroute.

#### Open Questions Requiring Human Clarification

- None. The current planning artifacts, existing code seams, and repository notes are sufficient to define this slice without introducing new product decisions.

### Scope Boundaries

- In scope:
  - selected-instrument reroute controls in the Sequence rack panel
  - primary contextual recovery from non-ready state into that panel flow
  - explicit target choice for reroute when backend-owned choices require it
  - clear-route and reroute-to-empty-slot flows that preserve rack structure
  - immediate backend-driven readiness refresh after reroute
- Out of scope:
  - slot lifecycle redesign
  - shared-slot reuse behavior
  - auto-routing heuristics
  - plugin runtime activation or transport semantics
  - app-scoped inventory-management workflows

### Cross-Story Alignment

- Story 3.1 established the backend-owned rack, slot, route, target, and address model this story must continue to use.
- Story 3.2 established Sequence as the score-context rack and readiness surface.
- Story 3.3 established slot creation and plugin assignment as rack preparation work rather than route work.
- Story 3.4 established the explicit route command, backend-owned route choices, and the current compact row-level route picker.
- Story 3.7 should extend those seams into a better repair workflow, especially where the current row-level picker cannot choose a target safely.
- Stories 3.5 and 3.6 remain blocked guardrails, so this story must stay bounded to manual reroute and recovery only.

### Previous Story Intelligence

- Story 3.4 already added the validated full-route command and `route_target_options_by_slot_id`; do not invent a second route-authoring contract.
- The current `InstrumentOverviewRow` fast path only auto-resolves slots with zero or one backend-verified route choice; it throws a recoverable error when a slot requires an explicit target choice.
- The current `PlaybackRackPanel` remains slot-centric even though the Sequence bottom-panel state already tracks the selected instrument.
- Existing tests in `playback-rack-panel.test.tsx` explicitly assert that selected-instrument reroute controls are absent today, which makes them the natural seam to update for this story.
- Story 3.6 being blocked is a guardrail: recovery stays manual and explicit rather than becoming hidden convenience routing.

### Architecture Compliance

- Rust remains the sole authoritative owner of rack state, slot state, route state, route-choice emission, and route-health derivation.
- React remains the score-context presentation and interaction layer; it must not infer valid targets, route health, or replacement topology.
- Score-context playback relationships stay in contextual panels, while Plugin Manager and other app-scoped playback assets stay in dedicated windows.
- Playback-critical communication stays command-oriented and backend-owned; do not add noisy event-only recovery paths.
- If implementation changes persisted route semantics beyond the current route model, assess migration impact and update `src-tauri/src/migrations/next.rs`.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Current compact route picker and route error surface.
  - Already sets the selected instrument before mutating the route.
  - Currently throws a recoverable error when a slot needs an explicit target the compact picker cannot choose.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Current score-context rack surface for slot preparation and plugin assignment.
  - Natural place to add selected-instrument reroute and clear-route affordances.
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
  - Already contains assertions that selected-instrument reroute controls are absent today.
  - Should become the primary frontend regression seam for this story.
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - Existing row-level route tests should preserve the fast path and cover the improved recovery handoff.
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - Already passes the Sequence bottom-panel state into the rack panel tab flow.
- `src/windows/main/hooks/use-bottom-panel-state.ts`
  - Already resolves the selected instrument for Sequence and is the seam to reuse instead of creating parallel selected-instrument state.
- `src/services/actions.ts`
  - Existing `assignSlotRoute(...)` and `unassignRoute(...)` actions should remain the frontend command seam.
- `src/services/state/context.tsx`
  - Mirrors backend-owned general state including `resolved_playback_route_by_instrument_id` and `route_target_options_by_slot_id`.
- `src/services/types.ts`
  - Frontend mirror of route, route-choice, and readiness data.
- `src-tauri/src/score/instruments/mod.rs`
  - Canonical route validation and route-resolution seam.
  - Existing tests already prove routing to an existing empty slot is accepted and non-root targets for empty slots are rejected.
- `src-tauri/src/lib.rs`
  - Emits refreshed general state after route mutations.

### Legacy Or Superseded Seams To Avoid

- Do not reintroduce direct instrument-to-plugin assignment state.
- Do not fork a new frontend-owned reroute state model or target-derivation layer.
- Do not solve ordinary recovery by forcing users into Plugin Manager or another app-scoped window.
- Do not resurrect the legacy slot-only helper as the primary production route path.

### Implementation Guidance

- Treat the current instrument-row route select as the simple fast path, not as the only reroute UI.
- Use the selected-instrument Sequence rack panel as the contextual repair surface for ambiguous, attention-needed, or target-specific reroute work.
- Prefer shallow, explicit interactions: show the selected instrument, show the current route or health state, let the user clear or route to a specific slot, and only ask for a target when the backend says more than one choice is valid.
- Preserve low-indirection language and workflow. Users should feel they are changing an instrument's route, not managing host topology.
- Keep reroute success and failure legible through existing readiness presentation rather than adding a second status language.
- Preserve current behavior during transport unless the implementation already has a safe existing path; do not expand this story into new live-playback semantics.

### Persistence And Migration Requirements

- No migration should be required if implementation only reuses the current `InstrumentPlaybackRoute` shape and existing slot model.
- If a persisted route or recovery model changes beyond current fields, add or update the pending migration in `src-tauri/src/migrations/next.rs`.
- Keep machine-local plugin inventory and runtime-only recovery data out of `.soloc` persistence.

### File Structure Requirements

- Frontend work should stay centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - `src/windows/main/hooks/use-bottom-panel-state.ts`
  - `src/services/actions.ts`
  - `src/services/types.ts`
  - `src/services/state/context.tsx`
- Backend work, if required, should stay centered in:
  - `src-tauri/src/score/instruments/mod.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/automation/registry.rs` only if command-contract updates are unavoidable
  - `src-tauri/src/migrations/next.rs` only if persisted schema changes

### Testing Requirements

- Frontend tests should cover:
  - rendering selected-instrument reroute controls in `PlaybackRackPanel`
  - routing the selected instrument to a prepared slot from the rack panel
  - clearing the selected instrument route from the rack panel
  - routing the selected instrument to an existing empty slot and showing explicit `NeedsAttention`
  - handing off ambiguous target cases from the row fast path into the rack workflow without stale errors lingering
  - readiness presentation refreshing after successful reroute so old unresolved state disappears
- Backend tests should be extended only if route validation or resolved-state behavior changes. If that happens, cover:
  - explicit route assignment to empty slots
  - rejection of unsupported targets for empty slots
  - refreshed resolved route state after reroute mutations
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings` when Rust changes
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib` when Rust changes

### Library / Framework Requirements

- Stay on the existing React, TypeScript, Rust, Tauri, and Vite stack.
- Keep TypeScript and Rust enums aligned exactly for playback-route target and address modes.
- Reuse the existing general-state emission and `invoke(...)` command flow rather than adding a new orchestration layer.
- Prefer seam-local tests over a new playback-routing harness.

### Documentation Impact

- Update `docs/development-guide.md` so it describes Sequence as the primary manual reroute and recovery surface after slot preparation.
- Update `docs/architecture.md` if the selected-instrument rack workflow materially changes the documented Sequence reroute surface or the recovery flow.
- Update `docs/data-models.md` if empty-slot reroute semantics or route-choice presentation rules need clarification.
- Update `docs/project-context.md` only if new AI-agent guardrails are introduced for reroute or recovery work.

### Git Intelligence Summary

- Recent history shows the current Epic 3 direction and boundaries:
  - `08ec8b8c feat: blocked stories`
  - `a3ed9de4 feat: investigate HALion`
  - `dc6da699 feat(3.4): slot assignment`
  - `6d6e6ef8 chore: proposed rack ux improvements`
  - `0727803c feat(3.3): slot assignment`
- Treat the HALion investigation and blocked stories as guardrails, not invitations to pull shared-slot or auto-routing work into 3.7.

### Latest Technical Information

- `score.assign_instrument_route(...)` already persists the full route payload and validates target and address in Rust.
- `resolved_playback_route_by_instrument_id` and `route_target_options_by_slot_id` are already emitted on general state, so the reroute workflow can stay backend-driven.
- `assign_instrument_route_accepts_existing_empty_slot` and `assign_instrument_route_rejects_non_root_target_for_empty_slot` already exist in `src-tauri/src/score/instruments/mod.rs`, confirming the current empty-slot recovery model.
- `InstrumentOverviewRow` currently emits the recoverable error `Selected slot needs a specific route target that this picker cannot choose yet.`, which is the concrete UX gap this story should resolve.

### Project Structure Notes

- Preserve the sprint-status key `3-7-reassign-and-recover-instrument-playback-without-restructuring-the-score` for tracking continuity even though the epic story title is phrased as repair and reroute.

## Dev Agent Record

### Implementation Plan

- Revert the added selected-instrument reroute UI so the current shipped surface returns to slot cards plus the row-level slot picker.
- Keep Story 3.7 blocked until the multi-timbral work in Stories 3.5 and 3.6 is unblocked and the full reroute UX can be designed with the correct target model.

### Debug Log

- Removed the added selected-instrument reroute controls from `PlaybackRackPanel` and restored the existing slot-only rack workflow.
- Restored the original row-level ambiguous-target behavior and removed the new Sequence-panel handoff copy.
- Reverted the documentation updates tied to the removed UI and reclassified Story 3.7 as blocked behind the still-blocked multi-timbral stories.

### Completion Notes

- Removed the newly added reroute UI and returned the product surface to the existing three-state model: slot, slot plugin picker, and instrument route picker.
- Deferred the richer reroute and recovery UX until the post-MVP routing follow-ons are intentionally reopened.
- Validation completed after the revert with the standard frontend checks.

## File List

- \_bmad-output/implementation-artifacts/3-7-reassign-and-recover-instrument-playback-without-restructuring-the-score.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml

## Change Log

- 2026-03-24: Reverted the added selected-instrument reroute UI and moved Story 3.7 to blocked pending the blocked multi-timbral stories.
