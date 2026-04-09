# Story 8.3: Add Playing-Technique Properties Panels

Status: done

## Story

As a composer,
I want playing-technique markings to expose the shared properties workflow,
so that the new entry type integrates with the existing score-editing model.

## Acceptance Criteria

1. Given a playing-technique marking can be selected in the score, when it is the current selection, then the bottom-panel properties workflow shows a dedicated playing-technique properties surface, and that surface stays consistent with the existing directive-style properties architecture.
2. Given the first properties scope should stay bounded, when the initial properties panel is delivered, then it exposes the shared `Hidden` toggle only, and it does not widen into technique-specific editing controls before later scope explicitly requires them.
3. Given hidden markings should remain consistent with other score directives, when a playing technique is hidden through the properties workflow, then its render and hit-target behavior follow the existing hidden-entry rules, and the persisted score truth remains aligned with the shared directive editing model.

## Tasks / Subtasks

- [x] Task 1: Route selected playing-technique entries through the write-screen properties shell (AC: 1)
  - [x] Add a `useSelectedPlayingTechnique` hook under `src/windows/main/hooks/` that resolves the selected playing-technique entry from the selected flow, stave, and stave master track using the same stale-selection guard pattern as `useSelectedClef`.
  - [x] Update `src/windows/main/screens/write/components/properties-panel.tsx` to recognize `Selection::PlayingTechnique` / `selection.type === "PlayingTechnique"` and route it to a dedicated playing-technique properties section inside the existing bottom-panel `Properties` tab.
  - [x] Keep unresolved or stale playing-technique selections on the shared empty-state path instead of showing fallback controls or frontend-invented placeholder state.

- [x] Task 2: Add the bounded playing-technique properties section using the shared `Common` hidden control only (AC: 1, 2)
  - [x] Create `src/windows/main/screens/write/components/playing-technique-properties-section.tsx` parallel to the existing clef and key-signature sections.
  - [x] Reuse `CommonPropertiesSection` and wire its `onHiddenChange` callback through `actions.score.playingTechnique.update(flow_id, stave_id, playing_technique_id, hidden)`.
  - [x] Keep the surface intentionally narrow: no technique-kind picker, no delete button, no create affordance, no expression-map preview, and no playback-facing controls in this story.

- [x] Task 3: Preserve shared hidden-entry behavior instead of inventing a playing-technique-specific visibility model (AC: 2, 3)
  - [x] Reuse the existing persisted `hidden` field and backend update seam added in Story 8.1; do not add new persisted fields, migration work, or compatibility residue for this panel.
  - [x] Keep hidden playing-technique behavior aligned with the renderer and hit-test rules from Story 8.2: hidden entries do not draw and do not emit hit targets.
  - [x] Do not add frontend-only ghost rows, placeholder hit targets, or any alternate hidden-entry recovery workflow just for playing techniques.

- [x] Task 4: Add seam-local regression coverage and update maintained docs (AC: 1, 2, 3)
  - [x] Extend `src/windows/main/screens/write/components/properties-panel.test.tsx` to cover playing-technique routing and the presence of the shared `Hidden` switch.
  - [x] Add focused coverage for `useSelectedPlayingTechnique` covering successful resolution plus null cases for wrong selection type, missing flow or stave, missing master track, missing entry, wrong entry type, and tick mismatch.
  - [x] Add or extend component-level coverage so toggling the playing-technique hidden switch dispatches the existing `actions.score.playingTechnique.update(...)` call with the expected payload.
  - [x] Update the maintained docs that describe this architecture and workflow: `docs/architecture.md`, `docs/component-inventory.md`, and `docs/development-guide.md`.

## Dev Notes

### Story Intent

- Story 8.3 is the narrow properties-workflow follow-through for the playing-technique entry family introduced in Story 8.1 and made visible/selectable in Story 8.2.
- The core job is not to invent a new editing surface. It is to plug `PlayingTechnique` into the already-shipped write-screen properties shell and reuse the existing directive-style `Common` visibility control.
- The highest-risk failure mode is widening into technique-specific editing, playback controls, or a parallel panel architecture when the accepted scope is only one dedicated panel with the shared `Hidden` toggle.

### User Clarifications

- The “bottom-panel properties workflow” in this story is the existing write-screen `Properties` tab, not the Sequence rack panel, not the popover workflow, and not a new app-scoped window.
- The first shipped playing-technique properties surface exposes the shared `Hidden` toggle only.
- Hidden playing-technique entries must keep following the existing directive rules: they stop rendering and stop emitting hit targets once hidden.
- Story 8.3 should reuse the existing directive-style properties architecture already used for tempo, time signature, key signature, and clef.

### This Story Owns

- Selection-aware write-screen properties routing for `PlayingTechnique` entries.
- One dedicated playing-technique properties section that reuses the shared `Common` hidden toggle.
- Frontend regression coverage around selection resolution, properties-panel routing, and hidden-toggle dispatch.
- Documentation updates for the write-screen properties architecture and the playing-technique visibility workflow.

### This Story Does Not Own

- Editing `PlayingTechniqueKind` values such as changing `pizz` to `arco` in place.
- Popover creation, keyboard deletion, renderer hit targets, or score click selection; those belong to Stories 8.1 and 8.2.
- Any visible placeholder, list view, or dedicated recovery affordance for hidden playing-technique entries.
- Derived playing-technique context, expression-map condition matching, or playback behavior; those remain in Stories 8.4 through 8.6.

### Current Relevant Seams

- `src/windows/main/screens/write/components/properties-panel.tsx`
  - Selection-aware write-screen properties shell that already switches by directive selection type.
- `src/windows/main/screens/write/components/common-properties-section.tsx`
  - Shared directive visibility section with the `Hidden` toggle that 8.3 should reuse directly.
- `src/windows/main/screens/write/components/clef-properties-section.tsx`
  - Closest existing stave-master directive pattern for a hidden-toggle-only properties section.
- `src/windows/main/screens/write/components/key-signature-properties-section.tsx`
  - Another minimal `Common`-only properties surface; useful as the simplest frontend reference.
- `src/windows/main/screens/write/components/time-signature-properties-section.tsx`
  - Reference for a directive properties section that combines `Common` visibility controls with additional directive-specific controls; 8.3 should explicitly stop short of widening into that shape.
- `src/windows/main/hooks/use-selected-clef.ts`
  - Best current hook pattern for resolving a selected stave-master directive from backend-pushed score truth.
- `src/services/types.ts`
  - Frontend `Selection` union already includes `PlayingTechniqueSelection` with `flow_id`, `stave_id`, `playing_technique_id`, and `tick`.
- `src/services/actions.ts`
  - Existing `actions.score.playingTechnique.update(...)` wrapper for the narrow hidden-state mutation command.
- `src-tauri/src/score/entries/playing_technique.rs`
  - Backend owner of the persisted `hidden` field and the `score_entries_playing_technique_update` command.
- `src/windows/main/screens/write/components/properties-panel.test.tsx`
  - Current regression seam for selection-type routing in the write-screen properties shell.
- `_bmad-output/implementation-artifacts/8-1-add-durable-playing-technique-score-entries.md`
  - Previous-story foundation for the persisted `PlayingTechnique` data model and the narrow hidden update seam.
- `_bmad-output/implementation-artifacts/8-2-render-and-select-playing-technique-markings-above-the-stave.md`
  - Previous-story foundation for visible rendering, shared selection, hidden-entry draw suppression, and score hit targets.
- `docs/architecture.md`
  - Canonical architecture notes for flow-master versus stave-master directives and the write-screen properties shell.
- `docs/component-inventory.md`
  - Inventory of write-screen properties components and main-window selection hooks.
- `docs/development-guide.md`
  - Current maintained workflow guidance for bounded playing-technique authoring and hidden-entry behavior.

### Architecture Compliance

- Keep backend-owned score truth and backend-owned sequencer selection canonical. The properties panel must project backend state rather than introduce frontend-owned hidden-state truth.
- Reuse the existing write-screen properties shell and `CommonPropertiesSection` primitives; do not add a second properties architecture for playing techniques.
- Keep the mutation seam narrow: hidden-state changes must flow through `actions.score.playingTechnique.update(...)` and the existing backend command rather than a broad object replacement or local optimistic model.
- Preserve the current hidden-entry contract from prior directive work: once hidden, a playing-technique marking does not draw and does not emit hit targets.
- Do not add migration work or compatibility residue. Story 8.1 already introduced the persisted `hidden` field and the update seam.
- Do not widen this story into new authoring controls, recovery UI, playback widgets, or app-scoped editing windows.

### Implementation Guidance

- Prefer `useSelectedPlayingTechnique` as a direct mirror of `useSelectedClef`, because playing-technique entries are stave-master-owned and already carry `stave_id` in their selection payload.
- Create `PlayingTechniquePropertiesSection` as a thin wrapper around `CommonPropertiesSection`, parallel to `ClefPropertiesSection` and `KeySignaturePropertiesSection`.
- Use the existing `actions` wrapper rather than calling `invoke(...)` directly from the new component.
- Add a dedicated `data-testid` path for the new properties section only if it helps keep the panel-routing tests clear and consistent with the existing `TimeSignature`, `KeySignature`, and `Clef` cases.
- Prefer no Rust changes unless implementation exposes a genuine missing seam. The hidden update command, persisted field, and draw or hit-target suppression behavior already exist.
- If a hidden toggle causes the selected entry to stop resolving through normal backend state, the panel should fall back to the existing empty state rather than keeping a fabricated frontend copy alive.

### Risks And Regression Traps

- Widening the panel into kind-editing controls, delete controls, or expression-map context before later stories explicitly require them.
- Implementing the properties surface anywhere other than the existing write-screen bottom-panel `Properties` shell.
- Introducing frontend-owned hidden-state shadow data instead of reusing the existing backend update seam.
- Changing renderer or hit-test behavior for hidden playing-technique entries rather than relying on the shared hidden-entry contract already established in 8.2.
- Adding a playing-technique-specific hidden recovery affordance that diverges from the existing directive model.
- Forgetting to update the maintained docs that describe the properties architecture and playing-technique workflow.

### Testing Requirements

- Extend `src/windows/main/screens/write/components/properties-panel.test.tsx` so a selected playing-technique entry routes to the dedicated section and exposes the `Hidden` switch.
- Add focused tests for `useSelectedPlayingTechnique` using the same stale-selection and missing-state cases covered by the other selection hooks.
- Add or extend frontend interaction coverage so toggling the playing-technique `Hidden` switch dispatches `actions.score.playingTechnique.update(flow_id, stave_id, playing_technique_id, hidden)` with the expected values.
- Keep backend test additions bounded: only extend Rust tests if implementation uncovers a missing seam, because Story 8.1 already covers hidden persistence and Story 8.2 already covers hidden draw and hit-target suppression.
- Validate the implemented story with the normal repo guardrails used by neighboring write-screen work: `npm run lint`, `npm run check-types`, and targeted JS or Rust tests for any touched seams.

### Documentation Impact

- Update `docs/architecture.md` to add `PlayingTechniquePropertiesSection` to the write-screen properties architecture description.
- Update `docs/component-inventory.md` to list the new playing-technique properties component and the new selection hook.
- Update `docs/development-guide.md` to capture the shipped properties workflow boundary: the playing-technique properties panel exposes only the shared `Hidden` toggle and hidden entries remain non-rendered and non-hit-testable.
- `docs/data-models.md` should not need a schema update for this story unless implementation reveals that the existing hidden-field description needs clarification.
- No AI-agent instruction update is required unless implementation establishes a broader repo-wide rule beyond this feature-specific workflow.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- No new dependencies should be introduced for Story 8.3.

### File Structure Notes

- Keep the new properties section alongside the existing write-screen properties components under `src/windows/main/screens/write/components/`.
- Keep the new selection hook under `src/windows/main/hooks/` with the other write-screen selection resolvers.
- Keep playing-technique hidden mutation on the existing backend score-entry seam; do not create a second feature-specific command path.
- Keep story tracking under `_bmad-output/implementation-artifacts/` and update sprint status in the same pass.

### Git Intelligence Summary

- The latest relevant commit, `feat(8.2): playback technique rendering`, already wired renderer draw, hit testing, selection, popover authoring, keyboard deletion, and workflow docs for visible playing-technique entries.
- That commit shape strongly suggests Story 8.3 should remain mostly a write-screen frontend and docs story: consume the existing selection and hidden-update seams rather than reopening renderer, popover, or transport ownership.
- Story 8.1 already landed the persisted `hidden` field and the narrow update command, so 8.3 should not spend scope on new persistence or migration work.

### Latest Technical Information

- No external library research is needed for Story 8.3.
- The decisive current information is repo truth as of 2026-04-09: the write-screen properties shell already exists, minimal hidden-toggle-only directive sections already exist for clef and key signature, and playing-technique hidden updates already have a backend command plus frontend action wrapper.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 8 and Story 8.3 acceptance criteria.
- `_bmad-output/implementation-artifacts/8-1-add-durable-playing-technique-score-entries.md`
  - Persisted playing-technique data shape, hidden-field rationale, and narrow update seam.
- `_bmad-output/implementation-artifacts/8-2-render-and-select-playing-technique-markings-above-the-stave.md`
  - Visible score workflow, shared selection payload, hidden draw suppression, and playing-technique guardrails.
- `docs/architecture.md`
  - Write-screen properties shell and directive architecture rules.
- `docs/component-inventory.md`
  - Existing properties components and selection-hook inventory.
- `docs/data-models.md`
  - Persisted `PlayingTechnique` entry shape and hidden-field meaning.
- `docs/development-guide.md`
  - Playing-technique authoring guardrails and hidden-entry behavior.
- `docs/project-context.md`
  - Backend-owned state, write-screen architecture rules, and current stack baselines.
- `src/windows/main/screens/write/components/properties-panel.tsx`
  - Selection-aware properties shell.
- `src/windows/main/screens/write/components/common-properties-section.tsx`
  - Shared hidden-toggle section.
- `src/windows/main/screens/write/components/clef-properties-section.tsx`
  - Minimal stave-master directive properties reference.
- `src/windows/main/screens/write/components/key-signature-properties-section.tsx`
  - Minimal hidden-toggle-only directive properties reference.
- `src/windows/main/hooks/use-selected-clef.ts`
  - Selected stave-master directive resolution pattern.
- `src/services/types.ts`
  - `PlayingTechniqueSelection` in the frontend selection union.
- `src/services/actions.ts`
  - `actions.score.playingTechnique.update(...)` wrapper.
- `src-tauri/src/score/entries/playing_technique.rs`
  - Backend hidden-field mutation seam.
- `src/windows/main/screens/write/components/properties-panel.test.tsx`
  - Existing routing regression seam.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-09: Story prepared from Epic 8 planning, the completed 8.1 and 8.2 story artifacts, current repo docs, and the live write-screen properties-panel seams in `src/windows/main/screens/write/components/`.
- 2026-04-09: Story scope narrowed explicitly to the existing write-screen properties shell plus the shared `Hidden` toggle; technique-kind editing, renderer changes, and playback semantics remain deferred.
- 2026-04-09: Repo exploration confirmed that `PlayingTechniqueSelection` already exists in TypeScript, `actions.score.playingTechnique.update(...)` already wraps the backend hidden-update command, and the nearest frontend patterns are `ClefPropertiesSection`, `KeySignaturePropertiesSection`, and `useSelectedClef`.
- 2026-04-09: Implemented `useSelectedPlayingTechnique` as the stave-master selection resolver, routed `Selection::PlayingTechnique` through `PropertiesPanel`, and added the bounded `PlayingTechniquePropertiesSection` that reuses `CommonPropertiesSection` and `actions.score.playingTechnique.update(...)`.
- 2026-04-09: Added regression coverage for playing-technique selection resolution, properties-panel routing, and hidden-toggle dispatch, then validated with `npm run test:js`, `npm run check-types`, and `npm run lint`.

### Completion Notes List

- Story context created for Story 8.3.
- Story 8.3 moved to `ready-for-dev` in sprint tracking.
- The story explicitly reuses the existing write-screen properties shell and the shared `Common` hidden-toggle section.
- The story explicitly avoids widening into technique-kind editing, renderer changes, or playback behavior.
- The story calls out the exact docs that should move with implementation: `docs/architecture.md`, `docs/component-inventory.md`, and `docs/development-guide.md`.
- Added `useSelectedPlayingTechnique` to resolve selected stave-master playing-technique entries from backend-owned score truth with the same stale-selection guards used by other directive hooks.
- Routed `PlayingTechnique` selections through the write-screen `Properties` tab and added a dedicated `PlayingTechniquePropertiesSection` that exposes only the shared `Hidden` toggle.
- Confirmed hidden-toggle updates dispatch through `actions.score.playingTechnique.update(flow_id, stave_id, playing_technique_id, hidden)` without widening into technique-kind or playback controls.
- Updated maintained architecture and workflow docs for the new properties section and bounded hidden-only behavior.
- Validation completed: targeted red/green tests, full `npm run test:js`, `npm run check-types`, and `npm run lint` all passed.

### File List

- `_bmad-output/implementation-artifacts/8-3-add-playing-technique-properties-panels.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/component-inventory.md`
- `docs/development-guide.md`
- `src/windows/main/hooks/use-selected-playing-technique.test.ts`
- `src/windows/main/hooks/use-selected-playing-technique.ts`
- `src/windows/main/screens/write/components/playing-technique-properties-section.test.tsx`
- `src/windows/main/screens/write/components/playing-technique-properties-section.tsx`
- `src/windows/main/screens/write/components/properties-panel.test.tsx`
- `src/windows/main/screens/write/components/properties-panel.tsx`

### Change Log

- 2026-04-09: Created the Story 8.3 ready-for-dev artifact and advanced sprint tracking from backlog to ready-for-dev.
- 2026-04-09: Implemented the bounded playing-technique properties workflow, added regression coverage, updated maintained docs, and validated the JS guardrails before moving the story to review.
