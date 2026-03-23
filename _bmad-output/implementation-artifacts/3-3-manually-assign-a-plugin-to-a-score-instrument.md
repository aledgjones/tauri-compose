# Story 3.3: Manually Assign a Plugin to a Score Instrument

Status: ready-for-dev

## Story

As a composer,
I want to assign an available single-part plugin to a score instrument from the score workflow,
so that I can prepare that instrument for playback quickly without leaving my composition context.

## Acceptance Criteria

1. Given a score instrument is selected and a usable plugin inventory exists, when the user chooses a plugin from the contextual assignment control, then that plugin is assigned to the selected score instrument and the assignment is stored as backend-owned playback intent.
2. Given the score-context picker is used for assignment, when the user opens it, then it presents a filtered, musician-friendly view of currently usable plugins and it does not require the user to manage the full plugin inventory window just to make an assignment.
3. Given a manual assignment is completed successfully, when the instrument's playback state is refreshed, then its readiness state reflects the new assignment and the instrument becomes eligible for later playback stories.

## Tasks / Subtasks

- [ ] Task 1: Reuse the existing Sequence score-context assignment surfaces instead of inventing a parallel workflow (AC: 1, 2)
  - [ ] Keep the Sequence instrument row as the compact quick-assignment surface.
  - [ ] Reuse the Sequence bottom panel as the richer score-context surface for the selected instrument so row and panel stay one assignment model rather than two competing flows.
  - [ ] Do not introduce a new dedicated window, Setup-screen assignment model, or app-scoped picker for this story.
- [ ] Task 2: Switch the assignment picker from legacy sampler-era plugin data to the backend-owned usable VST inventory seam (AC: 1, 2)
  - [ ] Replace sampler-era plugin-list consumption in the score-context assignment control with the current VST inventory source of truth for usable plugins.
  - [ ] Filter the picker to assignment-ready single-part options that Compose can currently use.
  - [ ] Keep availability filtering backend-aligned so the picker does not invent a second frontend-only inventory truth.
- [ ] Task 3: Deliver a compact, musician-friendly manual assignment interaction (AC: 1, 2)
  - [ ] Preserve the efficient score-first feel of the current row control while making the available choices clearer and more musician-friendly.
  - [ ] Keep labels and UI text concise rather than technical or verbose.
  - [ ] Avoid broad metadata browsing, scan-management controls, or debug-oriented detail inside the picker itself.
- [ ] Task 4: Persist manual single-part assignment through the existing backend-owned intent model and refresh readiness state (AC: 1, 3)
  - [ ] Route all assignment changes through the existing backend command and persisted assignment-intent seam.
  - [ ] Ensure successful assignment refreshes the emitted readiness state used by Story 3.2 row and panel presentation.
  - [ ] Preserve current single-part compatibility paths without widening this story into runtime activation, playback verification, or reopen recovery work.
- [ ] Task 5: Keep Story 3.3 bounded to manual single-part assignment only (AC: 1, 2, 3)
  - [ ] Do not expose part/channel targeting UI; Story 3.4 owns that.
  - [ ] Do not introduce shared-instance reuse logic; Story 3.5 owns that.
  - [ ] Do not add user-facing native plugin-editor launch controls in the Sequence row or picker; that remains later secondary handoff work.
  - [ ] Do not reopen Plugin Manager responsibilities such as refresh, full inventory browsing, or unavailable-plugin troubleshooting.
- [ ] Task 6: Add focused seam-level validation and documentation updates for score-context assignment (AC: 1, 2, 3)
  - [ ] Add frontend tests covering row and panel assignment interactions, usable-plugin filtering, and readiness refresh after assignment.
  - [ ] Add contract or backend tests only where needed to confirm the assignment seam still stores direct single-part playback intent and refreshes emitted state correctly.
  - [ ] Update relevant docs so they describe Sequence score-context assignment as distinct from Plugin Manager inventory management.

## Dev Notes

### Story Intent

- Story 3.3 is the manual assignment slice that follows Story 3.2's readiness-display slice.
- Its job is to let the user choose a currently usable single-part plugin from within the score workflow and store that choice as backend-owned playback intent.
- The compact quick-assignment surface remains the Sequence instrument row.
- The richer contextual assignment surface remains the Sequence bottom panel for the selected instrument.

### User Decisions Captured

- Story 3.3 is a direct continuation of Story 3.2: the same Sequence row and Sequence bottom-panel surfaces should now support score-context manual assignment.
- The interaction should stay compact and low-wordiness, with clarity through minimalism rather than heavy explanatory picker UI.
- The bottom panel should support the score-context assignment workflow without becoming a second Plugin Manager.
- A direct row-level or picker-level "open selected plugin UI" button is still future work, not part of Story 3.3.

### Bounded Slice Template

#### This Story Owns

- Manual single-part plugin assignment from Sequence score-context surfaces.
- Switching score-context plugin choice over to the usable VST inventory seam.
- Refreshing readiness state after a successful assignment.
- Keeping the assignment interaction musician-friendly and compact.

#### This Story Does Not Own

- New app-scoped inventory-management workflows or Plugin Manager expansion.
- Multi-part target or channel selection. Story 3.4 owns that.
- Shared-instance reuse rules or UI. Story 3.5 owns that.
- Automatic assignment for new instruments. Story 3.6 owns that.
- Reassignment/recovery workflows beyond completing a direct single-part assignment. Story 3.7 owns that.
- Runtime plugin activation/loading for playback. Story 3.8 owns that.
- User-facing native plugin-editor launch controls.
- Reopen or persisted recovery behavior. Epic 5 owns that.

#### Authoritative State Lives In

- Rust-owned persisted assignment intent on the score instrument.
- Rust-owned emitted readiness state consumed by Story 3.2 surfaces.
- Backend-owned machine-local VST inventory availability, not frontend-invented plugin lists.

#### Future Stories Intentionally Deferred

- Explicit target selection for compatible multi-part plugins.
- Shared-instance reuse and legibility.
- Native plugin-editor launch from score-context surfaces.
- Runtime activation/lifecycle and reopen recovery behavior.

#### Verification Required For Done

- Frontend tests prove a user can assign a currently usable plugin from Sequence score-context surfaces.
- Tests prove the picker shows currently usable plugins without reopening Plugin Manager responsibilities.
- Tests prove readiness updates after assignment using the backend-owned state seam.
- Validation uses the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None identified. Story 3.2 already clarified the Sequence row and bottom-panel surfaces, and Story 3.3 now continues that same score-context model.

### Scope Boundaries

- In scope:
  - score-context manual assignment for single-part plugins
  - row and panel assignment affordances within Sequence
  - usable-plugin filtering from the current VST inventory seam
  - readiness refresh after assignment
- Out of scope:
  - inventory management and refresh workflows
  - unavailable-plugin recovery UX beyond bounded readiness refresh
  - plugin editor launch
  - multi-part targeting and shared-instance behavior
  - runtime activation and playback behavior

### Cross-Story Alignment

- Story 3.1 created the backend-owned assignment-intent and resolved-state seam this story must mutate.
- Story 3.2 clarified that rows show compact readiness while the Sequence bottom panel is the richer score-context playback surface.
- Story 3.3 should build directly on those surfaces instead of introducing new assignment surfaces.
- Story 3.4 can extend the same assignment workflow with explicit target selection for compatible multi-part plugins.
- Story 3.5 can later reuse the same surfaces to explain shared-instance reuse without replacing the 3.3 single-part path.
- Story 3.8 can later consume the assigned plugin identity for runtime activation without redefining how assignment is authored.

### Previous Story Intelligence

- Story 3.1 already stores direct plugin assignment as backend-owned playback intent and emits resolved readiness state.
- Story 3.2 explicitly reserved manual assignment UX for this story while keeping readiness display compact and score-contextual.
- The current Sequence row already has a compact assignment control, but it still consumes sampler-era plugin data and needs to be brought onto the current VST inventory seam.

### Architecture Compliance

- Rust remains the sole authoritative owner of assignment truth.
- The frontend presents and triggers assignment changes but does not own plugin availability truth or assignment semantics.
- Plugin Manager remains the canonical app-scoped inventory-management window.
- Score-context assignment must feel like choosing a usable sound for a musical part, not configuring a DAW-style routing topology.
- The interaction should remain compact and not regress the low-wordiness direction chosen for Story 3.2.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing compact assignment control already calls the assignment action.
  - Currently consumes sampler-era plugin data and needs to move to the usable VST inventory seam.
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - Existing Sequence bottom-panel seam that should participate in the richer contextual assignment flow.
- `src/windows/main/components/plugins.tsx`
  - Current bottom-panel plugin surface contains inventory-browse and maintainer-oriented affordances.
  - This story should either refine or constrain that seam toward score-context assignment rather than broad inventory browsing.
- `src/windows/main/hooks/use-bottom-panel-state.ts`
  - Existing selected-instrument context for the Sequence bottom panel.
- `src/services/actions.ts`
  - Current `actions.score.instrument.assignPlugin()` frontend action already routes through the backend command seam.
- `src/services/types.ts`
  - Current frontend contract for assignment intent, resolved state, and plugin inventory shapes.
- `src/services/state/*`
  - Current app-state and plugin-inventory mirrors should remain the single frontend read path.
- `src-tauri/src/score/instruments/mod.rs`
  - Current backend command and direct-assignment intent seam.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Current backend-owned usable VST inventory seam that should back score-context manual assignment.

### Implementation Guidance

- Reuse the existing row assignment control where practical instead of inventing a brand-new picker just because the data source is changing.
- If the richer panel surface also offers assignment, it should be the same score-context model as the row, not a second implementation with different filtering or semantics.
- Consume the usable VST inventory seam, not the legacy sampler-era plugin list, for assignment choices.
- Filter the presented options to plugins Compose can currently use for this single-part workflow.
- Keep picker copy and labels concise; the user should feel like they are choosing a playable sound, not debugging the inventory system.
- Route every assignment change through the backend command seam and rely on emitted state refresh afterward.
- Do not add multi-part options, channel selectors, shared-instance controls, unavailable-plugin troubleshooting, or native plugin-editor launch in this story.
- Do not expand this story into activation or audible verification; later stories own runtime playback behavior.

### Persistence And Migration Requirements

- No persisted-data migration is expected if this story only reuses the existing direct-assignment intent model introduced in Story 3.1.
- If implementation tries to change the persisted assignment schema, stop and re-evaluate because that likely means Story 3.4 or Story 3.5 scope is leaking in.

### File Structure Requirements

- Frontend work should stay centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - `src/windows/main/components/plugins.tsx` or a closely related Sequence bottom-panel component seam
  - `src/windows/main/hooks/use-bottom-panel-state.ts`
  - `src/services/actions.ts`
  - `src/services/types.ts`
  - `src/services/state/*`
- Backend or contract refinements, if needed, should stay centered in:
  - `src-tauri/src/score/instruments/mod.rs`
  - `src-tauri/src/playback/plugin_inventory.rs`
- Do not spread this story into Plugin Manager window code unless a tiny shared picker or data-exposure refinement is genuinely necessary.

### Testing Requirements

- Frontend tests should cover:
  - assigning a plugin from the Sequence row
  - assigning a plugin from the richer selected-instrument panel if that affordance is added there
  - filtering to currently usable plugins only
  - readiness refresh after assignment
  - preserving compact, concise assignment UI rather than verbose inventory-browser behavior
- Backend or contract tests should cover:
  - direct single-part assignment continues to persist as backend-owned intent
  - emitted readiness state refresh remains aligned after assignment
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings` only if Rust/backend work is required

### Library / Framework Requirements

- Stay on the existing brownfield React/Tauri stack and repo conventions.
- Reuse existing Sequence controls and design-system patterns rather than introducing a new picker framework.
- Keep FE/BE contracts `snake_case` and enum alignment explicit if contract updates are required.
- Do not add a new frontend-owned state or plugin-management layer.

### Documentation Impact

- Update `docs/architecture.md` or `docs/development-guide.md` if the Sequence bottom-panel plugin surface is redefined from legacy browse/debug behavior into score-context manual assignment.
- Keep documentation explicit that Plugin Manager remains the canonical app-scoped inventory-management surface.
- Update any docs that still imply the Sequence row consumes sampler-era plugin lists once this story moves assignment to the VST inventory seam.

### Git Intelligence Summary

- Recent work kept Epic 3 incremental and seam-owned:
  - `ed1a09f8 feat(3.1): playback assignment setup`
  - `6f2d04fa chore: epic 2 retro`
  - `cfc3fe3e feat(2.8): abandon plugin image`
  - `ffaad55a feat(2.6): plugin ui`
  - `f2bb4a6a feat(2.5): bind plugin ui`
- Preserve that pattern here: upgrade the existing assignment seam to the real VST inventory path rather than replacing it with a larger new workflow.

### Latest Technical Information

- The current row assignment control already routes through `actions.score.instrument.assignPlugin()`, which is the right mutation seam to keep using.
- The key architectural shift for this story is not a new command. It is moving score-context assignment choices onto the backend-owned usable VST inventory seam.
- Native plugin editor launch remains a secondary later handoff, not part of this manual assignment slice.

### Project Structure Notes

- Story 3.3 should feel like a direct continuation of Story 3.2 rather than a separate redesign.
- The Sequence row remains the compact quick-assignment surface.
- The Sequence bottom panel remains the richer contextual surface.
- Plugin Manager remains the app-scoped inventory-management window.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.3 requirements
- `_bmad-output/planning-artifacts/prd.md` — FR7 and FR11 requirements
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned playback state and score-context versus app-scoped boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — direct assignment surfaces and contextual plugin-selection expectations
- `_bmad-output/implementation-artifacts/3-1-represent-playback-assignment-state-on-score-instruments.md` — assignment-intent and resolved-state foundation
- `_bmad-output/implementation-artifacts/3-2-show-assignment-readiness-in-instrument-rows-and-panels.md` — readiness-display surface decisions that Story 3.3 continues
- `_bmad-output/implementation-artifacts/epic-2-retro-2026-03-22.md` — bounded-slice guardrails for Epic 3
- `docs/project-context.md` — FE/BE ownership rules and Sequence/Plugin Manager scope guidance
- `docs/architecture.md` — current playback state and Sequence-panel boundary guidance
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — existing row assignment control
- `src/windows/main/screens/sequence/components/bottom-panel.tsx` — Sequence bottom-panel seam
- `src/windows/main/components/plugins.tsx` — current plugin surface to refine or constrain
- `src/windows/main/hooks/use-bottom-panel-state.ts` — selected-instrument context
- `src/services/actions.ts` — assignment action seam
- `src/services/types.ts` — frontend playback and inventory contract
- `src-tauri/src/score/instruments/mod.rs` — backend assignment command and intent model
- `src-tauri/src/playback/plugin_inventory.rs` — backend-owned usable plugin inventory seam

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `3.3`.
- Sprint status showed `3-3-manually-assign-a-plugin-to-a-score-instrument` in `backlog` before story creation.
- Artifact review confirmed Story 3.3 should be a direct continuation of Story 3.2's Sequence row and bottom-panel surfaces.
- Codebase analysis confirmed the current Sequence row already has a compact assignment control, but it still reads sampler-era plugin data and must move to the backend-owned VST inventory seam.
- UX and architecture review confirmed the picker must stay score-contextual, compact, musician-friendly, and distinct from Plugin Manager responsibilities.
- User direction confirmed a preference for compact, low-wordiness UI and continuity between Story 3.2 and Story 3.3.

### Completion Notes List

- Story context created for a bounded score-context manual single-part assignment slice.
- Explicitly anchored Story 3.3 to the same Sequence row and Sequence bottom-panel surfaces defined in Story 3.2.
- Captured the required migration from sampler-era plugin choices to the backend-owned usable VST inventory seam.
- Deferred multi-part targeting, shared-instance reuse, plugin-editor launch, and runtime activation to later stories.

### File List

- `_bmad-output/implementation-artifacts/3-3-manually-assign-a-plugin-to-a-score-instrument.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-03-22: Story created as the score-context manual assignment continuation of Story 3.2, with explicit VST-inventory sourcing, compact UI guidance, and sprint status advanced to ready-for-dev.
