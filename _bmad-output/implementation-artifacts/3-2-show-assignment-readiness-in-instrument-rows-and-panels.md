# Story 3.2: Show Current Rack And Route Summaries In Sequence

Status: done

## Story

As a composer,
I want to see the current rack and each instrument's route summary directly in Sequence,
so that I can tell at a glance which parts are playable and which need action.

## Acceptance Criteria

1. Given score instruments have resolved playback route state, when they are displayed in relevant score-context surfaces, then the Sequence bottom panel shows the current rack and its slot instances, including slots with no plugin currently assigned, and instrument rows or expansions show compact route summaries and health state.
2. Given playback state can be Ready, Needs Attention, or Unavailable, when those states are shown in the UI, then they are legible without relying on color alone and the presentation remains consistent with the existing Compose design system.
3. Given an instrument is not ready for playback, when a user inspects that instrument's score-context UI, then they are given one primary contextual recovery path and the product does not force them into an unrelated app-scoped workflow by default.

## Tasks / Subtasks

- [x] Task 1: Replace the superseded selected-instrument readiness panel with a current-rack surface in the Sequence bottom panel (AC: 1, 2)
  - [x] Reuse the existing Sequence bottom-panel seam and show the current score rack plus its hosted slot instances.
  - [x] Label rack slots in the MVP format `{slot index}. {Plugin Name}` so duplicate plugin instances remain distinguishable.
  - [x] Make unassigned slots legible as valid rack objects rather than silently hiding them.
  - [x] Keep the bottom panel score-contextual to the current score rather than widening it into Plugin Manager responsibilities.
- [x] Task 2: Show compact route summaries and health state on Sequence instrument surfaces (AC: 1, 2)
  - [x] Update the instrument row to summarize the route target, slot context, and current health without turning the row into a status-heavy badge strip.
  - [x] Use the existing instrument expansion or closely related score-context seam for richer route detail when needed, including route-owned expression-map linkage summary if that data already exists.
  - [x] Keep the row compact, scannable, and accessible through text, iconography, and placement rather than color alone.
- [x] Task 3: Keep recovery contextual and bounded to the score workflow (AC: 3)
  - [x] Provide one primary recovery handoff from non-ready route states into the relevant score-context rack or route surface.
  - [x] Avoid defaulting users into the dedicated Plugin Manager for routine route repair.
  - [x] Do not absorb Story 3.3 slot creation, Story 3.4 route authoring, Story 3.5 shared-slot explanation, Story 3.8 activation, or Epic 5 reopen recovery behavior.
- [x] Task 4: Preserve backend ownership of rack and route truth (AC: 1, 2, 3)
  - [x] Consume the Rust-owned rack, slot, target, address, route, and health state defined by Story 3.1 rather than letting React infer topology locally.
  - [x] If the UI needs a lighter presentation contract, derive it in Rust and mirror it in TypeScript rather than creating ad hoc frontend mapping helpers.
  - [x] Keep Sequence the presentation layer over backend-owned score-context playback state.
- [x] Task 5: Add focused validation and docs for the corrected Sequence rack workflow (AC: 1, 2, 3)
  - [x] Update frontend tests for bottom-panel rack display, instrument-row route summaries, and contextual recovery behavior.
  - [x] Add backend or contract-level tests only if Story 3.2 requires a refined emitted route-summary payload.
  - [x] Update docs so Sequence is described as the current-rack and route-summary surface while Plugin Manager remains the app-scoped inventory browser.

## Dev Notes

### Story Intent

- Story 3.2 is no longer the assignment-readiness display slice.
- Its corrected job is to make the current score rack and each instrument's route summary visible in Sequence after the rack-and-routing pivot.
- The Sequence bottom panel should show the current rack and hosted slot instances.
- Empty slots are valid part of the current rack and must remain visible so later slot-preparation and route-authoring stories build on the same mental model.
- Instrument rows and expansions should summarize route state compactly without reintroducing the rejected assignment-first UX.

### User Decisions Captured

- The stakeholder rejected the prior Story 3.2 implementation because it centered the wrong product object: direct instrument-to-plugin assignment.
- The corrected model is rack-first and backend-led in Rust.
- The Sequence bottom panel is the primary surface for the current rack.
- Instrument surfaces are primarily route-focused and can show target, slot, health, and linked route context.
- Plugin Manager remains the app-scoped inventory browser, not the current score rack.
- The old readiness-heavy selected-instrument panel should be superseded rather than extended.

### Bounded Slice Template

#### This Story Owns

- Showing the current rack and hosted slot instances in the Sequence bottom panel.
- Showing compact route summaries and route health in Sequence instrument surfaces.
- Providing one primary contextual recovery path for non-ready route states.
- Keeping rack and route presentation aligned to Rust-owned truth and existing Compose visual language.

#### This Story Does Not Own

- Creating or loading new rack slots from inventory. Story 3.3 owns that.
- Creating or changing routes to targets. Story 3.4 owns that.
- Shared-slot legibility beyond the compact summary needed to understand current route context. Story 3.5 owns the fuller slice.
- Automatic routing for newly created instruments. Story 3.6 owns that.
- Repair and reroute workflow depth beyond one primary contextual handoff. Story 3.7 owns that.
- Activating rack slots and routed targets for playback. Story 3.8 owns that.
- Session reopen restoration and degraded recovery behavior. Epic 5 owns that.
- Full plugin inventory browsing, refresh control, or maintainer troubleshooting flows. Plugin Manager owns those.

#### Authoritative State Lives In

- Rust-owned rack, slot, target, address, route, and health state emitted through the general app state.
- The existing Sequence score-context selection and bottom-panel state.
- Machine-local plugin inventory remains a separate backend-owned seam outside project data.

#### Future Stories Intentionally Deferred

- Musician-facing slot creation and plugin picking workflow.
- Explicit target and channel selection UI.
- Shared-slot reuse explanation across multiple score instruments.
- Runtime activation, transport readiness, and reopen repair semantics.
- Full route-owned expression-map editing workflow.

#### Verification Required For Done

- Frontend tests prove the Sequence bottom panel renders the current rack and slot labels from backend-owned state.
- Frontend tests prove instrument surfaces render compact route summaries and health consistently.
- Tests prove non-ready states expose one primary score-context recovery path without routing by default into Plugin Manager.
- If the emitted state contract changes, Rust and TypeScript coverage are updated together.
- Validation uses the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None. The rack-first model, the bottom-panel role, and the score-context versus app-scoped boundary have been clarified through the sprint course correction.

### Scope Boundaries

- In scope:
  - Sequence bottom-panel current-rack display.
  - Sequence instrument-row or expansion route summaries.
  - Compact route health presentation using the three user-facing states.
  - One primary contextual recovery handoff.
  - Minimal backend presentation refinement only if needed to keep route summaries backend-owned.
- Out of scope:
  - Plugin inventory management.
  - Slot creation or route-authoring workflows.
  - Activation and playback runtime behavior.
  - Reopen recovery and persistence repair flows.

### Cross-Story Alignment

- Story 3.1 defines the backend-owned rack, slot, target, address, and route model that this story must display.
- Story 3.2 should make that topology visible in Sequence without inventing a second frontend model.
- Story 3.3 should build slot creation on top of the same bottom-panel rack surface clarified here.
- Story 3.4 should layer route editing onto the same instrument surfaces rather than replacing this story's summaries.
- Story 3.5 can later enrich the UI with stronger shared-slot legibility once the basic route-summary structure is in place.
- Story 3.8 should be able to layer activation and runtime state on top of the same rack and route surfaces rather than redefining them.

### Previous Story Intelligence

- The rewritten Story 3.1 artifact establishes the corrected Epic 3 foundation: backend-owned rack and route state, not direct assignment truth.
- The superseded Story 3.2 implementation proved that a readiness-first selected-instrument panel is the wrong UX object for the corrected product direction.
- The sprint change proposal explicitly says to replace the old Story 3.2 implementation rather than build future work on top of it.
- Preserve the Epic 2 and Epic 3 guardrail that React must not derive playback topology independently from Rust.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory, rack state, slot state, route state, and route health.
- React remains the presentation and interaction layer over that backend-owned state.
- Score-context playback relationships stay in Sequence surfaces.
- Plugin Manager remains app-scoped and canonical for full inventory browsing.
- User-facing health states must be accessible without relying on color alone.
- The story must preserve the route-first mental model rather than drifting back into assignment-first wording or UI structure.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing compact row surface for each score instrument.
  - Must shift from assignment-flavored summary toward slot and route summary.
- `src/windows/main/screens/sequence/components/instrument-overview.tsx`
  - Coordinates the instrument-row list and selection context.
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - Existing Sequence bottom-panel seam.
  - Should become the current-rack surface for this story's corrected model.
- `src/windows/main/screens/sequence/components/playback-status-panel.tsx`
  - Existing selected-instrument status panel from the superseded Story 3.2 implementation.
  - Treat this as a replacement target, not as the final product model to preserve.
- `src/windows/main/screens/sequence/components/playback-readiness.tsx`
  - Existing presentation helper for compact health states.
  - May be reusable only if it cleanly adapts to route-based summaries.
- `src/windows/main/hooks/use-bottom-panel-state.ts`
  - Existing Sequence bottom-panel selection seam.
  - Reuse rather than creating a separate rack-selection model.
- `src/services/types.ts`
  - Frontend contract mirror for backend-owned playback state.
  - Update here in lockstep with Rust if Story 3.2 needs a lighter route-summary payload.
- `src/services/state/context.tsx`
  - Existing general-state mirror used by Sequence surfaces.
- `src-tauri/src/score/instruments/mod.rs`
  - Current backend seam for instrument playback state.
  - After Story 3.1, this should be the route-focused source rather than an assignment-focused helper.
- `src-tauri/src/lib.rs`
  - Emits Rust-owned general state to the frontend.
- `src-tauri/src/ui/screens/sequence.rs`
  - Existing Sequence UI state seam for bottom-panel behavior.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Machine-local plugin inventory truth.
  - Relevant for route resolution, but not the UI model to expose directly in Sequence.

### Legacy Or Superseded Seams To Avoid

- `src-tauri/src/playback/sampler.rs`
  - Legacy sampler seam. Do not extend it for rack or route summaries.
- `src-tauri/src/playback/scvi.rs`
  - Retired-path legacy seam. Not part of the VST rack model.
- `src/windows/main/components/plugins.tsx`
  - Legacy contextual browse and debugging surface.
  - Do not let it remain the implicit model for the Sequence bottom panel if it conflicts with the corrected current-rack role.
- The old Story 3.2 artifact content itself should be treated as superseded.

### Implementation Guidance

- Show the current rack in the bottom panel, not a selected-instrument status essay.
- Keep the row summary compact: slot label, route target summary, and one health signal should do the work.
- Preserve the three user-facing states `Ready`, `Needs Attention`, and `Unavailable`, but derive them from backend-owned route state.
- Prefer lightweight contextual recovery handoff over dense troubleshooting copy.
- Do not default the user into Plugin Manager for ordinary routing problems.
- Keep route-owned expression-map context to a summary or linkage hint only if the data already exists; do not invent the later editing workflow here.
- Do not reintroduce the rejected assignment-first terminology in UI copy, docs, or dev notes.
- If the route summary needs backend help to stay concise, add a Rust-owned summary seam instead of frontend recomputation.

### Persistence And Migration Requirements

- No new `.soloc` migration should be required if Story 3.2 remains a display slice over the Story 3.1 model.
- If this story only refines emitted UI-facing route summaries, do not add a persisted-data migration.
- If work unexpectedly changes persisted rack or route intent, stop and re-evaluate because that means the slice boundary has been crossed back into Story 3.1 or later recovery work.

### File Structure Requirements

- Frontend presentation work should stay centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview.tsx`
  - `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - `src/windows/main/screens/sequence/components/playback-status-panel.tsx` or its replacement seam
  - `src/windows/main/hooks/use-bottom-panel-state.ts`
- Contract changes, if needed, should stay centered in:
  - `src/services/types.ts`
  - `src/services/state/context.tsx`
  - `src-tauri/src/score/instruments/mod.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/ui/screens/sequence.rs`
- Do not spread this story into Plugin Manager window code unless a tiny contextual handoff is genuinely required.

### Testing Requirements

- Frontend tests should cover:
  - bottom-panel rendering of the current rack and hosted slots
  - compact route summary rendering on instrument rows or expansions
  - accessible health-state presentation without relying on color alone
  - the one primary contextual recovery path for non-ready route states
- Contract-level tests should cover any backend-owned route-summary or health-presentation refinement introduced for the UI.
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings` only if Rust/backend contract changes are made

### Library / Framework Requirements

- Stay on the existing brownfield React/Tauri stack and repo conventions.
- Reuse the current Sequence design-system patterns and iconography rather than introducing a new playback-specific UI language.
- Keep FE/BE contracts `snake_case` and enum/value alignment explicit if the backend contract changes.
- Do not add new state-management libraries or frontend-owned playback models.

### Documentation Impact

- Update `docs/architecture.md` so the Sequence bottom panel is described as the current-rack surface rather than the superseded readiness panel.
- Update `docs/development-guide.md` where it still describes the bottom panel as selected-instrument readiness and recovery only.
- Update `docs/component-inventory.md` if component responsibilities change materially during the replacement.
- Keep docs explicit that Plugin Manager remains the app-scoped inventory browser and that Sequence owns score-context rack and route summaries.

### Git Intelligence Summary

- Recent work shows the old Story 3.2 implementation landed on the branch but is now superseded by the course correction:
  - `5df3c9e2 feat(3.2): old pas at epic 3 stories`
  - `ed1a09f8 feat(3.1): playback assignment setup`
  - `6f2d04fa chore: epic 2 retro`
  - `cfc3fe3e feat(2.8): abandon plugin image`
  - `ffaad55a feat(2.6): plugin ui`
- Treat that first commit as evidence of the rejected slice, not as the desired scope to preserve.

### Latest Technical Information

- The sprint change proposal explicitly rewrites Story 3.2 around current-rack display and route summaries in Sequence.
- No new external library or framework choice is required for this story.
- The current repo already has the Sequence row, bottom-panel, and backend-emitted playback state seams needed for this corrected display slice.

### Project Structure Notes

- Preserve the existing artifact path and sprint key for this replacement so sprint tracking continuity is maintained while the story content is corrected.
- The Sequence bottom panel should move away from the superseded selected-instrument readiness role and toward current-rack display.
- The dedicated Plugin Manager remains the canonical app-scoped inventory browser and should not be duplicated here.

### References

- `_bmad-output/planning-artifacts/epics.md` — corrected Epic 3 Story 3.2 requirements
- `_bmad-output/planning-artifacts/prd.md` — Epic 3 route-first functional requirements
- `_bmad-output/planning-artifacts/architecture.md` — Rust-owned playback state, score-context boundaries, and backend ownership rules
- `_bmad-output/planning-artifacts/ux-design-specification.md` — Sequence rack surface, compact route state, and panel-versus-window scope guidance
- `_bmad-output/planning-artifacts/archive/2026-03-epic-3-planning-history/sprint-change-proposal-2026-03-22-rack-routing-pivot.md` — archived rationale and required Story 3.2 replacement
- `_bmad-output/implementation-artifacts/3-1-represent-playback-assignment-state-on-score-instruments.md` — corrected Story 3.1 foundation notes
- `_bmad-output/implementation-artifacts/3-3-manually-assign-a-plugin-to-a-score-instrument.md` — downstream story that must continue from the corrected rack model
- `docs/project-context.md` — FE/BE ownership rules and legacy seam guardrails
- `docs/architecture.md` — current architecture language that will need correction for this story
- `docs/development-guide.md` — current Sequence bottom-panel guidance that must be realigned
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — compact route-summary surface
- `src/windows/main/screens/sequence/components/instrument-overview.tsx` — instrument list orchestration seam
- `src/windows/main/screens/sequence/components/bottom-panel.tsx` — current-rack display seam
- `src/windows/main/screens/sequence/components/playback-status-panel.tsx` — superseded panel seam to replace or repurpose
- `src/windows/main/hooks/use-bottom-panel-state.ts` — score-context bottom-panel state seam
- `src/services/types.ts` — frontend playback contract mirror
- `src/services/state/context.tsx` — general app-state mirror
- `src-tauri/src/score/instruments/mod.rs` — backend route-state seam
- `src-tauri/src/lib.rs` — Rust-to-frontend state emission seam
- `src-tauri/src/ui/screens/sequence.rs` — Sequence UI state seam
- `src-tauri/src/playback/plugin_inventory.rs` — machine-local inventory truth

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `3.2` for replacement after course correction.
- Sprint tracking showed the old `3-2-show-assignment-readiness-in-instrument-rows-and-panels` artifact in `review`, which conflicted with the corrected Epic 3 story definition.
- Artifact review confirmed the epics file, UX spec, architecture, and sprint change proposal now define Story 3.2 as a rack-and-route summary slice in Sequence.
- The old Story 3.2 implementation was treated as superseded and not as a base to extend.
- The artifact path and sprint key were preserved to avoid unrelated tracking churn while replacing the story content.
- Replaced the selected-instrument playback-status panel with a new rack-first bottom-panel surface that renders current score rack slots, including unassigned slots, and highlights the focused instrument route when applicable.
- Reworked Sequence instrument-row summaries to show slot context, route target text, accessible health state, and one bounded `Open rack` recovery handoff for non-ready routes.
- Added focused frontend tests for rack-slot rendering, row summaries, and contextual recovery behavior; no Rust contract change was required.
- Validation passed with `npm run test:js`, `npm run lint`, and `npm run check-types`.

### Completion Notes List

- Replaced the old assignment-readiness Story 3.2 context with the corrected current-rack and route-summary scope.
- Reset the story status to `ready-for-dev` so implementation can restart from the corrected model.
- Preserved explicit scope boundaries so slot creation, route authoring, shared-slot UX, activation, and reopen recovery remain in later stories.
- Added current code-seam guidance, legacy seams to avoid, and doc impact for the corrected Sequence workflow.
- Implemented `PlaybackRackPanel` as the Sequence current-rack surface and removed the superseded selected-instrument status panel.
- Updated row-level route summaries to use backend-owned rack and resolved-route state, keeping `Ready`, `Needs Attention`, and `Unavailable` legible through text and iconography rather than color alone.
- Kept recovery bounded to Sequence by routing non-ready instrument surfaces into the current rack panel instead of Plugin Manager.
- Updated architecture, development-guide, component-inventory, and project-context docs to describe the rack-first Sequence workflow.

### File List

- `_bmad-output/implementation-artifacts/3-2-show-assignment-readiness-in-instrument-rows-and-panels.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/component-inventory.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `src/windows/main/screens/sequence/components/bottom-panel.test.tsx`
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
- `src/windows/main/screens/sequence/components/playback-status-panel.test.tsx` (deleted)
- `src/windows/main/screens/sequence/components/playback-status-panel.tsx` (deleted)

## Change Log

- 2026-03-23: Replaced the Sequence selected-instrument readiness panel with a current-rack panel, updated instrument-row route summaries and bounded recovery, added focused frontend coverage, and refreshed the Sequence rack workflow docs.

- 2026-03-23: Story replaced after the rack-and-routing course correction, superseding the assignment-readiness version and resetting the slice to ready-for-dev.

- 2026-03-22: Story created with bounded Sequence-row and Sequence-bottom-panel readiness scope, explicit score-context recovery guidance, and sprint status advanced to ready-for-dev.
- 2026-03-22: Implemented Sequence row readiness badges, selected-instrument playback status panel, primary in-score recovery handoff, focused frontend tests, and documentation updates; story advanced to review.
