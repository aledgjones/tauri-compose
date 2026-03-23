# Story 3.4: Create Or Change A Route From A Score Instrument To A Slot Target

Status: done

## Story

As a composer,
I want to create or change an explicit route from a score instrument to a slot target,
so that the playback destination for my instrument stays direct and unambiguous.

## Acceptance Criteria

1. Given a selected rack slot exposes one or more supported targets, when the user routes an instrument from the score workflow, then Compose captures the chosen target as part of that instrument's playback route and the selected target is visible in the instrument's score-context playback state.
2. Given a requested target is unavailable, invalid, or not safely routable, when the user attempts to complete the route, then Compose prevents ambiguous routing and surfaces a clear recoverable outcome and it does not silently guess an alternative target or instance.
3. Given this MVP support must stay score-first rather than drifting into DAW routing design, when the target-selection workflow is implemented, then it exposes only the explicit plugin target choices needed for the supported workflow and it does not expand into arbitrary bus routing or generic host-topology configuration.

## Tasks / Subtasks

- [x] Task 1: Add bounded route-authoring UI to existing Sequence score-context surfaces (AC: 1, 2, 3)
  - [x] Reuse the current Sequence route surfaces so the user can choose or change a route from the instrument context and the current-rack panel instead of creating a new app-scoped workflow.
  - [x] Keep the common path direct: choose a prepared slot, then choose a target only when that slot exposes more than one backend-verified target.
  - [x] Preserve one explicit unrouted or clear-route action for recoverable repair without deleting slots or reopening Plugin Manager responsibilities.
- [x] Task 2: Extend the backend-owned route mutation seam to persist explicit target and address intent (AC: 1, 2)
  - [x] Build on the existing rack-first ownership model where instruments route only to already-existing slots.
  - [x] Replace or extend the current slot-only route command so route creation stores the selected `slot_id`, `PlaybackRouteTarget`, and `PlaybackRouteAddress` instead of hard-coding only `InstrumentPlaybackRoute::direct(slot_id)` for every case.
  - [x] Keep all validation in Rust so invalid slot ids, invalid targets, unsupported target modes, and stale slot state fail with explicit recoverable errors rather than frontend guesses.
- [x] Task 3: Expose supported target choices from Rust rather than inferring them in React (AC: 1, 2, 3)
  - [x] Add or refine a backend-owned contract that tells the UI which route targets are currently valid for a prepared slot.
  - [x] Limit target options to bounded MVP semantics grounded in actual VST-facing concepts already modeled by Compose: `Plugin`, `PluginPart`, and `MidiChannel`.
  - [x] If the backend cannot prove a richer target safely for a slot, expose only the verified direct plugin target and do not invent speculative part or channel choices in the UI.
- [x] Task 4: Keep route summaries and route-health presentation aligned with backend truth after authoring (AC: 1, 2)
  - [x] Refresh the existing `resolved_playback_route_by_instrument_id` presentation after successful route changes so instrument rows and the rack panel show the chosen target and address consistently.
  - [x] Preserve explicit `Unrouted`, `Ready`, and `NeedsAttention` behavior when a slot is missing, empty, unavailable, or no longer safely routable.
  - [x] Do not silently create duplicate slots, shared-slot reuse behavior, or activation side effects while authoring a route.
- [x] Task 5: Add focused backend, frontend, and contract validation plus doc updates (AC: 1, 2, 3)
  - [x] Add backend tests for route creation or change with direct-plugin, plugin-part, and invalid-target cases where the seam supports them.
  - [x] Add frontend tests for the score-context route picker flow, recoverable failure messaging, and summary refresh after success.
  - [x] Update docs so they describe Sequence as the score-context route-authoring surface and keep Plugin Manager app-scoped.

## Dev Notes

### Story Intent

- Story 3.4 is the explicit route-authoring slice that follows Story 3.3 slot preparation.
- Its job is to connect a score instrument to an already-existing rack slot and capture the chosen target and address as backend-owned route intent.
- This story must keep the product mental model route-first: the user routes an instrument into a prepared slot target, not directly into a plugin object.
- This story owns bounded explicit target choice, including part or channel selection only when the backend can expose those targets as verified supported choices.
- This story must not collapse into shared-slot reuse logic, plugin activation, reopen recovery, or app-scoped inventory management.

### User Decisions Captured

- The corrected Epic 3 model is rack-first: slots are prepared first, then instruments route into them.
- Sequence remains the score-context workflow surface for rack and route work.
- Plugin Manager remains the app-scoped inventory browser and is not the default answer for ordinary route authoring or repair.
- Target selection should stay bounded and musician-facing rather than exposing generic host-topology configuration.
- Ambiguous target choice must remain explicit; Compose must not silently guess a part, channel, or replacement slot.

### Bounded Slice Template

#### This Story Owns

- Creating or changing the explicit route from an instrument to a prepared rack slot.
- Capturing the route's selected target and address as backend-owned score intent.
- Exposing one bounded target-selection workflow in Sequence when a slot has more than one backend-verified target.
- Clearing an instrument route without deleting the underlying slot.
- Refreshing route summaries and recoverable health state after route authoring.

#### This Story Does Not Own

- Creating new slots or assigning plugins to slots. Story 3.3 owns that.
- Reusing one compatible slot across multiple instruments as an explicit product slice. Story 3.5 owns that.
- Automatic routing for newly created instruments. Story 3.6 owns that.
- Repair and reroute workflows beyond the bounded author-or-clear route flow. Story 3.7 owns that.
- Activating plugins or targets for playback. Story 3.8 owns that.
- Session-lifecycle stabilization, reopen recovery, or degraded reopen handling. Story 3.9 and Epic 5 own those.
- Native plugin-editor launch, Plugin Manager browsing, or maintainer troubleshooting workflows.

#### Authoritative State Lives In

- Rust-owned persisted `Instrument.playback_route` intent.
- Rust-owned rack and slot state on `score.playback.rack`.
- Rust-owned emitted `resolved_playback_route_by_instrument_id` presentation state.
- Backend-owned machine-local plugin inventory and any backend-owned slot-target capability seam needed to support explicit target choice.

#### Future Stories Intentionally Deferred

- Deliberate shared-slot reuse UX and legibility.
- Automatic routing and repair flows.
- Runtime target activation and first-audible-playback behavior.
- Session persistence and reopen recovery behavior.
- Route-owned expression-map editing and native plugin-editor launch from route context.

#### Verification Required For Done

- Frontend tests prove a user can create, change, and clear a route from the score-context Sequence workflow.
- Backend tests prove route persistence captures the expected slot, target, and address semantics and rejects invalid routing requests.
- Tests prove route-summary presentation refreshes from backend-owned state after success.
- Validation uses the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None. The planning artifacts already establish that Story 3.4 owns explicit route authoring after slot preparation and before shared-slot reuse.

### Scope Boundaries

- In scope:
  - route creation or change to an existing slot
  - bounded target choice for supported slot targets
  - route clearing without deleting slots
  - backend validation and emitted state refresh for route summaries
  - focused Sequence score-context UI for route authoring
- Out of scope:
  - slot creation or slot plugin assignment
  - shared-slot reuse as a dedicated product behavior
  - automatic routing and repair workflows
  - runtime activation and transport playback behavior
  - app-scoped inventory, editor-host, or troubleshooting workflows

### Cross-Story Alignment

- Story 3.1 established the backend-owned rack, slot, target, address, and route model this story must use rather than replace.
- Story 3.2 made the current rack and route summaries visible in Sequence and provides the presentation surfaces this story should extend.
- Story 3.3 prepares routeable slots with assigned plugins but explicitly stops short of creating the instrument route.
- Story 3.4 must add explicit route authoring on top of those prepared slots without smuggling in shared-slot semantics or plugin activation.
- Story 3.5 should later make multi-instrument reuse of one compatible slot explicit and legible after this story makes route targets authorable.
- Story 3.8 should later activate routed slots and targets for playback using the route intent captured here rather than redefining route ownership.

### Previous Story Intelligence

- The corrected Story 3.2 artifact established Sequence as the current-rack and route-summary surface and kept recovery score-contextual.
- The corrected Story 3.3 artifact established that slot preparation creates the destination first and explicitly deferred route authoring to this story.
- The route-first pivot proposal is authoritative for Epic 3 sequencing and scope boundaries.
- Existing frontend actions and backend commands already provide slot-only route assignment or unassignment seams, but the UI does not yet expose route authoring and the current command path hard-codes direct root routing.

### Architecture Compliance

- Rust remains the sole authoritative owner of rack state, slot state, route state, route validation, and route-health derivation.
- React remains the score-context presentation and input layer; it must not infer valid part or channel targets on its own.
- Plugin Manager stays app-scoped; Sequence stays score-contextual.
- Route authoring must preserve the rack-first invariant: instruments own only routes to existing slots.
- If persisted route semantics change beyond the already-modeled fields, migration impact must be assessed and handled in `src-tauri/src/migrations/next.rs`.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Current compact route-summary surface with the existing `Open rack` recovery handoff.
  - Likely entry point for initiating route creation or change from instrument context.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Current score-context rack surface that already handles slot creation and slot plugin assignment.
  - Likely surface for focused slot selection and bounded target selection.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Existing summary and status presentation helper.
  - Must continue to consume backend-owned route truth rather than frontend-derived target logic.
- `src/windows/main/hooks/use-bottom-panel-state.ts`
  - Current bottom-panel state seam that ties the selected instrument to the Sequence rack panel.
- `src/services/actions.ts`
  - Existing frontend action seam already exposes `assignSlotRoute` and `unassignRoute`.
  - Route-authoring work should extend this seam rather than bypassing it.
- `src/services/actions.test.ts`
  - Existing invoke-contract tests for route actions.
  - Update if command shape or route-authoring actions expand.
- `src/services/types.ts`
  - Frontend mirrors for `InstrumentPlaybackRoute`, `ResolvedPlaybackRoute`, route target enums, and route address enums.
- `src/services/state/context.tsx`
  - Existing mirror of backend-owned general app state used by Sequence surfaces.
- `src-tauri/src/score/playback.rs`
  - Canonical score-owned playback types including `PlaybackRouteTargetMode`, `PlaybackRouteAddressMode`, and `InstrumentPlaybackRoute::direct`.
- `src-tauri/src/score/instruments/mod.rs`
  - Current backend seam for assigning or clearing routes and resolving route health.
  - The present `score_instruments_assign_slot_route` path is the key seam to extend or replace because it only persists the direct root route today.
- `src-tauri/src/lib.rs`
  - Registers Tauri commands and emits backend-owned general playback state.
- `src-tauri/src/automation/registry.rs`
  - Mirrors route commands for automation and may need lockstep updates if route-authoring command contracts change.
- `src-tauri/src/migrations/next.rs`
  - Only relevant if route persistence schema changes beyond the already-modeled route fields.

### Legacy Or Superseded Seams To Avoid

- Any reintroduction of direct instrument-to-plugin assignment state.
  - Route authoring must continue through slot and route intent only.
- Any frontend-only inference of valid multi-part or MIDI-channel targets.
  - If the backend cannot prove a target, the UI must not invent it.
- Any attempt to solve route authoring by reopening Plugin Manager as the default workflow.
- Extending sampler-era seams as the playback authoring model.

### Implementation Guidance

- Start from the current Sequence rack panel and instrument row seams rather than inventing a new route window.
- Treat the existing slot-only route helper as compatibility residue for the single direct-target case, not as the final route-authoring contract.
- Prefer one explicit backend-owned route-authoring command or tightly related command set that captures the full route shape in one validated mutation.
- Keep the user-facing workflow compact: slot choice first, target choice only when needed, one clear failure state if the route cannot be completed safely.
- If a slot exposes exactly one backend-verified direct target, a one-step route assignment is acceptable; if more than one target is valid, the user must choose explicitly.
- Do not create duplicate slots or reuse another instrument's slot implicitly as part of route authoring.
- Do not activate the plugin runtime, infer playback readiness locally, or couple this story to first-audible-playback work.

### Persistence And Migration Requirements

- No migration should be required if implementation only starts using the already-modeled `target` and `address` fields on `InstrumentPlaybackRoute`.
- If implementation changes the persisted route shape or slot-target persistence format, add or update the pending migration in `src-tauri/src/migrations/next.rs` before considering the story complete.
- Keep machine-local plugin inventory and target-discovery cache details out of `.soloc` data.

### File Structure Requirements

- Frontend work should stay centered in:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - `src/windows/main/hooks/use-bottom-panel-state.ts`
  - `src/services/actions.ts`
  - `src/services/actions.test.ts`
  - `src/services/types.ts`
  - `src/services/state/context.tsx`
- Backend or contract work should stay centered in:
  - `src-tauri/src/score/playback.rs`
  - `src-tauri/src/score/instruments/mod.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/automation/registry.rs`
  - `src-tauri/src/migrations/next.rs` only if persisted route schema changes
- Do not spread this story into Plugin Manager window code unless a tiny shared picker primitive becomes genuinely necessary.

### Testing Requirements

- Frontend tests should cover:
  - creating a route from an instrument into a prepared slot
  - changing a route from one slot or target to another
  - clearing a route back to explicit unrouted state
  - presenting bounded target choices only when backend-owned options exist
  - showing clear recoverable errors for invalid or stale route requests
- Backend tests should cover:
  - successful route assignment for direct plugin target
  - successful route persistence for plugin-part or MIDI-channel target when supported by the seam
  - invalid slot ids and invalid target requests failing safely
  - resolved route state remaining `NeedsAttention` for missing slot, empty slot, or unavailable target cases
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings` if Rust or command-contract work is touched

### Library / Framework Requirements

- Stay on the existing brownfield React, TypeScript, Rust, and Tauri stack.
- Reuse existing Sequence UI primitives and state flow rather than introducing a new state-management layer.
- Keep FE and BE enums aligned exactly for route target and route address modes.
- Do not add new frontend-owned playback models or route-health derivation logic.

### Documentation Impact

- Update `docs/architecture.md` if the route-authoring seam, target-enumeration contract, or Sequence route workflow changes materially.
- Update `docs/development-guide.md` so it describes the current Sequence route-authoring flow alongside slot preparation and keeps Plugin Manager app-scoped.
- Update `docs/data-models.md` if route target or address semantics, allowed modes, or persistence expectations are refined during implementation.
- Keep any updated docs explicit about the rack-first invariant and the rule that backend-owned target choices must stay grounded in verified VST-facing semantics.

### Git Intelligence Summary

- Recent branch history shows the rack-routing pivot is now the active implementation direction:
  - `6d6e6ef8 chore: proposed rack ux improvements`
  - `0727803c feat(3.3): slot assignment`
  - `0abd1a14 feat(3.2): assignment readieness`
  - `d52854c0 feat(3.1): massive cleanup`
  - `de30023d chore: twirl epic 3 stories`
- Treat the `3.2` and `3.3` commits as evidence of the in-progress correction rather than proof that the remaining route-authoring seam is already complete.

### Latest Technical Information

- `InstrumentPlaybackRoute` already persists `slot_id`, `target`, and `address`, and the shared data model already defines `Plugin`, `PluginPart`, and `MidiChannel` target or address modes.
- The current `score_instruments_assign_slot_route` command only accepts `instrument_id` and `slot_id` and persists `InstrumentPlaybackRoute::direct(slot_id)`, which is insufficient for the full explicit target-authoring workflow planned here.
- Existing tests in `src-tauri/src/score/instruments/mod.rs` already include a `PluginPart` route fixture that resolves to `NeedsAttention` when no supporting slot exists, which is evidence that explicit non-root targets are part of the intended model.
- The frontend currently displays route summaries and rack cards but does not expose any route-authoring control beyond opening the rack surface.

### Project Structure Notes

- Preserve the legacy sprint key and artifact path for story-tracking continuity, even though the corrected story content is route-authoring rather than the older narrower title.
- Keep route authoring on the same Sequence score-context surfaces that Stories 3.2 and 3.3 already established.
- Plugin Manager remains app-scoped and should not become the fallback implementation home for this story.

### References

- `_bmad-output/planning-artifacts/epics.md` — corrected Epic 3 Story 3.4 acceptance criteria and downstream boundaries
- `_bmad-output/planning-artifacts/prd.md` — rack-first routing goals and MVP route-first scope
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned playback state and Sequence-versus-Plugin-Manager boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — direct routing mental model and bounded target-choice expectations
- `_bmad-output/planning-artifacts/sprint-change-proposal-2026-03-22-rack-routing-pivot.md` — authoritative rack-routing pivot and Epic 3 sequencing
- `_bmad-output/implementation-artifacts/3-2-show-assignment-readiness-in-instrument-rows-and-panels.md` — corrected Sequence rack and route-summary slice
- `_bmad-output/implementation-artifacts/3-3-manually-assign-a-plugin-to-a-score-instrument.md` — corrected slot-preparation slice that explicitly defers route authoring to Story 3.4
- `docs/project-context.md` — backend ownership, Sequence current-rack surface, and Plugin Manager scope rules
- `docs/architecture.md` — backend-emitted resolved route state and current Sequence rack workflow
- `docs/development-guide.md` — current slot-preparation and Plugin Manager guidance
- `docs/data-models.md` — persisted route, target, and address types
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — compact route-summary and recovery seam
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx` — score-context rack panel seam
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts` — backend-owned route summary presentation seam
- `src/windows/main/hooks/use-bottom-panel-state.ts` — Sequence bottom-panel state seam
- `src/services/actions.ts` — route-action invoke seam
- `src/services/actions.test.ts` — route-action contract tests
- `src/services/types.ts` — frontend playback route types and enums
- `src/services/state/context.tsx` — frontend mirror of backend general state
- `src-tauri/src/score/playback.rs` — canonical playback route types and defaults
- `src-tauri/src/score/instruments/mod.rs` — route mutation and resolution seam
- `src-tauri/src/lib.rs` — command registration and emitted route state
- `src-tauri/src/automation/registry.rs` — automation mirror for route commands
- `src-tauri/src/migrations/next.rs` — pending migration seam if persisted route shape changes

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `3.4`.
- Sprint tracking still preserves the legacy 3.4 story key and artifact naming, so this artifact keeps that continuity while replacing the story content with the corrected route-authoring scope.
- Artifact review confirmed the planning artifacts and previous Epic 3 stories now define the sequence as rack foundation, rack display, slot preparation, route authoring, shared-slot reuse, then later activation and recovery slices.
- Code exploration confirmed that the backend already owns route truth and exposes slot-only assign or unassign seams, but the UI still lacks route authoring and the current command path hard-codes only direct root routing.
- The story therefore treats explicit target capture and any required backend-owned target-enumeration seam as primary implementation scope rather than optional UI polish.
- Implemented backend-owned `PlaybackRouteChoice` emission on `soloc://state/general` and switched the explicit route command to accept a full `InstrumentPlaybackRoute` payload.
- Simplified the Sequence routing UX so instrument overview rows own slot selection and target visibility while rack cards stay focused on slot preparation.
- Validation completed with `npm run lint`, `npm run check-types`, `npm run test:js`, `cargo test --lib -q`, and `cargo clippy --lib -- -D warnings`.

### Completion Notes List

- Created the corrected Story 3.4 context as the explicit route-authoring slice after slot preparation.
- Preserved the legacy sprint key and artifact path so sprint tracking continuity remains stable.
- Added guardrails against frontend target inference, Plugin Manager scope creep, and accidental shared-slot or activation leakage.
- Made the existing slot-only route command gap explicit so the dev agent does not assume Story 3.4 is already mostly implemented.
- Added backend-owned slot route-choice emission, explicit route validation, and explicit route persistence for direct plugin targets while rejecting unsupported richer targets.
- Added direct slot selection and unrouting on instrument overview rows with always-visible target summaries, and removed the extra rack-card route controls.
- Added focused backend and frontend tests for direct route assignment, unsupported-target rejection, recoverable UI failures, and clear-route behavior.
- Updated architecture, development-guide, and data-models docs to describe Sequence as the score-context route-authoring surface and `route_target_options_by_slot_id` as the authoritative target-choice contract.

### File List

- docs/architecture.md
- docs/data-models.md
- docs/development-guide.md
- src/services/actions.test.ts
- src/services/actions.ts
- src/services/state/context.tsx
- src/services/types.ts
- src-tauri/src/automation/registry.rs
- src-tauri/src/lib.rs
- src-tauri/src/score/instruments/mod.rs
- src-tauri/src/score/playback.rs
- src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx
- src/windows/main/screens/sequence/components/playback-rack-panel.tsx
- src/windows/main/screens/sequence/components/playback-readiness-presentation.ts
- src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx
- src/windows/main/screens/sequence/components/instrument-overview-row.tsx

### Change Log

- 2026-03-23: Implemented bounded score-context route authoring in Sequence with backend-owned route choices, explicit route validation, focused UI tests, backend tests, and supporting documentation updates.
- 2026-03-23: Simplified the Sequence route UX so overview rows own slot selection and target visibility while rack cards return to plugin-slot preparation only.
