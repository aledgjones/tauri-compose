# Story 3.3: Load Or Create A Rack Slot From The Available Plugin Inventory

Status: done

## Story

As a composer,
I want to load or create a rack slot from an available plugin in the score workflow,
so that I can prepare a reusable playback destination without leaving my composition context.

## Acceptance Criteria

1. Given a usable plugin inventory exists, when the user adds a slot from the contextual rack workflow, then Compose creates an empty rack slot as backend-owned playback intent, and when the user chooses or updates that slot's plugin from the slot-local picker, then Compose stores the chosen plugin on that existing slot.
2. Given the score-context picker is used for slot preparation, when the user opens it, then it presents a filtered, musician-friendly view of currently usable plugins and it does not require the user to manage the full plugin inventory window just to prepare a slot.
3. Given a rack slot is created successfully, when Sequence state is refreshed, then the current rack reflects the new slot and exposes a routeable slot destination for later routing stories without yet connecting any instrument to it.

## Tasks / Subtasks

- [x] Task 1: Reuse the corrected Sequence rack surfaces for slot preparation instead of inventing a parallel workflow (AC: 1, 2, 3)
  - [x] Keep the Sequence bottom panel as the primary current-rack surface where slot preparation occurs.
  - [x] Let row-level or compact score-context affordances hand off into that same rack surface rather than creating a second slot-preparation model.
  - [x] Do not introduce a new dedicated window, app-scoped setup flow, or Plugin Manager-dependent workflow for ordinary slot preparation.
- [x] Task 2: Source slot preparation from the backend-owned usable VST inventory seam (AC: 1, 2)
  - [x] Consume the current usable plugin inventory truth from the existing Rust-owned inventory seam rather than sampler-era plugin lists or frontend-only filtering.
  - [x] Present only plugins Compose can currently use for this score-context slot-preparation workflow.
  - [x] Keep the picker musician-friendly, concise, and contextual instead of exposing maintainer-oriented inventory details.
- [x] Task 3: Create backend-owned rack-slot intent without widening into routing or activation (AC: 1, 3)
  - [x] Persist the prepared slot into the current score rack using the backend-owned rack and slot model established by Story 3.1.
  - [x] Bind the chosen plugin identity to that newly created slot as part of the same backend-owned slot intent.
  - [x] Refresh emitted Sequence state so the current rack shows the new slot immediately after successful preparation.
  - [x] Keep this slice about prepared slot intent only; do not activate plugin runtime state or require first-audible-playback behavior.
- [x] Task 4: Keep Story 3.3 bounded away from route selection and shared-slot semantics (AC: 1, 2, 3)
  - [x] Do not capture instrument-to-slot target routing here; Story 3.4 owns explicit route authoring.
  - [x] Do not silently collapse this workflow into shared-slot reuse across instruments; Story 3.5 owns explicit shared-slot reuse behavior.
  - [x] Do not add automatic routing, repair or reroute flows, native plugin-editor launch, or runtime activation behavior.
- [x] Task 5: Add focused seam-level validation and docs for score-context slot preparation (AC: 1, 2, 3)
  - [x] Add frontend tests covering slot preparation from the Sequence rack workflow, usable-plugin filtering, and rack refresh after success.
  - [x] Add backend or contract tests only where needed to confirm prepared slot intent is stored in backend-owned rack state and emitted coherently to Sequence.
  - [x] Update docs so they describe Sequence as the score-context rack surface and Plugin Manager as the app-scoped inventory-management surface.

## Dev Notes

### Story Intent

- Story 3.3 is the slot-preparation slice that follows Story 3.2's current-rack display slice.
- Its job is to let the user choose a currently usable plugin from within the score workflow, create a rack slot, and assign that plugin to the slot in backend-owned score state.
- This story prepares the destination side of the model only. It does not create an instrument-to-slot route.
- This story must not author an instrument route, guess a target, or activate a plugin instance for playback.
- The current rack remains the score-context object; plugin inventory remains an app-scoped backend-owned resource.

### User Decisions Captured

- The corrected Epic 3 model is rack-first: users prepare rack slots before they author explicit instrument routes.
- Slot preparation should stay musician-friendly, compact, and score-contextual rather than reopening Plugin Manager responsibilities.
- The Sequence bottom panel is the primary current-rack surface for this story's workflow.
- Native plugin editor launch remains deliberate future handoff work, not part of slot preparation.
- Shared-slot reuse must stay explicit later work rather than being silently inferred here.

### Bounded Slice Template

#### This Story Owns

- Preparing a rack slot from the current usable plugin inventory inside Sequence score context.
- Assigning the chosen plugin to that newly prepared slot as part of the same bounded workflow.
- Exposing a routeable slot destination that later stories can connect instruments to.
- Refreshing the current rack surface after successful slot preparation.
- Keeping the plugin picker filtered, musician-friendly, and backed by Rust-owned inventory truth.
- Storing prepared slot intent in backend-owned rack state for later routing stories.

#### This Story Does Not Own

- Explicit route or target selection from a score instrument to a slot. Story 3.4 owns that.
- Shared-slot reuse across multiple instruments. Story 3.5 owns that.
- Automatic routing for new instruments. Story 3.6 owns that.
- Repair and reroute behavior beyond bounded slot preparation. Story 3.7 owns that.
- Runtime slot activation or playable target loading. Story 3.8 owns that.
- Same-session runtime lifecycle stabilization beyond bounded slot preparation, plus project reopen recovery. Story 3.8 and Epic 5 own that.
- Full plugin inventory browsing, rescan, or troubleshooting workflows. Plugin Manager owns those.

#### Authoritative State Lives In

- Rust-owned persisted rack and slot intent defined by the corrected Story 3.1 model.
- Rust-owned emitted general state consumed by Sequence rack surfaces.
- Backend-owned machine-local usable VST inventory under `src-tauri/src/playback/plugin_inventory.rs`.

#### Future Stories Intentionally Deferred

- Explicit target-selection workflow for compatible multi-target plugins.
- Shared-slot reuse legibility and multi-instrument linkage.
- Automatic routing decisions and reroute workflows.
- Runtime activation, session lifecycle, and reopen recovery behavior.
- Native plugin-editor launch from score-context playback surfaces.

#### Verification Required For Done

- Frontend tests prove a user can prepare a rack slot from Sequence score-context surfaces.
- Tests prove the picker shows currently usable plugins without reopening Plugin Manager responsibilities.
- Tests prove the current rack refreshes after successful slot preparation using backend-owned state.
- Validation uses the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None. User clarification confirmed this story is the create-slot-and-assign-plugin workflow, not an implicit shared-slot reuse decision.

### Scope Boundaries

- In scope:
  - score-context rack-slot preparation from usable plugin inventory
  - Sequence current-rack surface updates after successful slot preparation
  - filtered musician-friendly plugin picking for slot preparation
  - backend-owned persisted slot intent and emitted rack refresh
- Out of scope:
  - instrument-to-slot target routing
  - shared-slot reuse behavior
  - plugin runtime activation or first audible playback
  - inventory management, rescan, or unavailable-plugin troubleshooting workflows

### Cross-Story Alignment

- Story 3.1 defines the backend-owned rack and slot model that this story must mutate.
- Story 3.2 makes the current rack visible in Sequence and gives this story its score-context surface.
- Story 3.3 prepares the reusable rack destination that later route stories will target.
- Story 3.4 should author explicit instrument-to-slot target routes on top of the slots prepared here.
- Story 3.5 should make deliberate shared-slot reuse explicit rather than letting this story smuggle it in through slot preparation.
- Story 3.6 can later perform the convenience path on instrument creation by creating a slot, mounting a sensible default plugin into it, and creating a safe route only when Compose can do that without ambiguity.
- Story 3.8 should activate the prepared slots and routed targets later without redefining slot authorship.

### Previous Story Intelligence

- The rewritten Story 3.1 artifact establishes rack, slot, target, address, and route state as the corrected backend-owned foundation.
- The rewritten Story 3.2 artifact makes the Sequence bottom panel the current-rack surface and reserves slot preparation for this story.
- The superseded direct-assignment Story 3.3 artifact should not be treated as the base implementation model; it centered the wrong product object.
- Epic 2 established the machine-local plugin inventory and the Plugin Manager window as the canonical app-scoped inventory surface.

### Architecture Compliance

- Rust remains the sole authoritative owner of inventory truth, rack state, slot state, route state, and emitted playback-related UI state.
- The frontend presents score-context rack workflows and invokes backend mutations; it must not invent a second rack or inventory truth.
- Sequence remains the score-context rack surface; Plugin Manager remains the app-scoped inventory browser.
- Slot preparation must store score-owned intent only and must not persist machine-local inventory state into `.soloc`.
- This story must not silently guess routes, targets, shared-slot linkage, or runtime activation outcomes.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - Current Sequence bottom-panel seam that Story 3.2 repurposes into the current-rack surface.
  - Primary score-context surface for slot preparation in this story.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing compact instrument-row seam.
  - May provide a bounded handoff into the current-rack surface, but should not become a second rack-authoring implementation.
- `src/windows/main/screens/sequence/components/instrument-overview.tsx`
  - Coordinates the row list and selected-instrument context that can hand off into the bottom panel.
- `src/windows/main/screens/sequence/components/playback-status-panel.tsx`
  - Superseded selected-instrument status seam from the old Story 3.2 implementation.
  - Do not let this remain the conceptual center of slot preparation if it conflicts with the corrected current-rack role.
- `src/windows/main/components/plugins.tsx`
  - Legacy contextual browse/debug seam that may need refinement or replacement so it behaves like score-context slot preparation instead of a second Plugin Manager.
- `src/windows/main/hooks/use-bottom-panel-state.ts`
  - Existing bottom-panel state seam that should continue to mediate the current score-context panel.
- `src/services/actions.ts`
  - Current frontend mutation seam.
  - Extend the explicit playback-slot actions rather than reintroducing any direct instrument-to-plugin shortcut.
- `src/services/types.ts`
  - Frontend contract mirror for backend-owned rack, slot, route, and plugin inventory state.
- `src/services/state/context.tsx`
  - General app-state mirror used by Sequence surfaces.
- `src-tauri/src/score/instruments/mod.rs`
  - Current backend mutation seam closest to playback-domain intent.
  - Keep slot lifecycle and route mutation separate so this story does not reintroduce instrument-owned slot setup.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Backend-owned usable plugin inventory seam that must back the slot-preparation picker.
- `src-tauri/src/lib.rs`
  - Emits backend-owned general state and command wiring to the frontend.
- `src-tauri/src/ui/screens/sequence.rs`
  - Sequence UI state seam that may need bounded support for current-rack slot preparation.

### Legacy Or Superseded Seams To Avoid

- Reintroducing any direct instrument-to-plugin mutation seam.
  - Slot preparation must stay rack-first so instruments continue to route to already-existing slots.
- The old Story 3.3 direct-assignment artifact itself should be treated as superseded.
- Plugin Manager window code should not become the implementation home for ordinary score-context slot preparation.

### Implementation Guidance

- Build slot preparation on the corrected current-rack surface from Story 3.2 rather than resurrecting assignment-first UX.
- Consume the backend-owned usable plugin inventory seam, not legacy sampler-era plugin lists or frontend-derived availability.
- Keep the picker concise and musician-friendly; users should feel like they are preparing a sound destination, not auditing inventory metadata.
- Persist prepared slot intent in the score-owned rack model, including the chosen plugin assigned to the created slot, and refresh emitted rack state after success.
- Treat the result as a routeable destination candidate in the rack, not as implicit proof that any instrument is already connected.
- Do not capture route targets, instrument linkage, or shared-slot reuse semantics in this story.
- Do not activate plugins for playback here; runtime activation belongs to Story 3.8.
- If duplicate-plugin behavior becomes contentious, prefer explicit product handling over hidden shared-slot inference.

### Persistence And Migration Requirements

- No new `.soloc` migration should be required if this story only uses the rack and slot model established in Story 3.1.
- If implementation tries to change the persisted rack or slot schema again, stop and re-evaluate because that likely means Story 3.1 scope is leaking or the foundation is still unstable.
- Keep machine-local inventory details outside persisted project data.

### File Structure Requirements

- Frontend work should stay centered in:
  - `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview.tsx`
  - `src/windows/main/components/plugins.tsx` or a closely related corrected Sequence bottom-panel seam
  - `src/windows/main/hooks/use-bottom-panel-state.ts`
  - `src/services/actions.ts`
  - `src/services/types.ts`
  - `src/services/state/context.tsx`
- Backend or contract refinements, if needed, should stay centered in:
  - `src-tauri/src/score/instruments/mod.rs`
  - `src-tauri/src/playback/plugin_inventory.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/ui/screens/sequence.rs`
- Do not spread this story into Plugin Manager window code unless a tiny shared inventory exposure refinement is genuinely necessary.

### Testing Requirements

- Frontend tests should cover:
  - preparing a rack slot from the Sequence current-rack workflow
  - filtering to currently usable plugins only
  - refreshing the current rack after successful slot preparation
  - preserving compact, concise score-context UI rather than verbose inventory-browser behavior
- Backend or contract tests should cover:
  - prepared slot intent persists in backend-owned rack state
  - emitted Sequence rack state refresh remains coherent after slot preparation
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings` only if Rust or backend contract work is required

### Library / Framework Requirements

- Stay on the existing brownfield React/Tauri stack and repo conventions.
- Reuse existing Sequence design-system patterns and bottom-panel grammar rather than introducing a new slot-picker framework.
- Keep FE/BE contracts `snake_case` and enum alignment explicit if contract updates are required.
- Do not add new frontend-owned playback state or inventory-management layers.

### Documentation Impact

- Update `docs/architecture.md` and `docs/development-guide.md` if they still imply score-context playback setup is direct instrument assignment rather than rack-slot preparation followed by routing.
- Update `docs/component-inventory.md` if the bottom-panel plugin surface changes materially while being realigned to the corrected current-rack role.
- Keep docs explicit that Plugin Manager remains the canonical app-scoped inventory-management surface.

### Git Intelligence Summary

- Recent work shows the branch is mid-correction from the superseded assignment-first model:
  - `5df3c9e2 feat(3.2): old pas at epic 3 stories`
  - `ed1a09f8 feat(3.1): playback assignment setup`
  - `6f2d04fa chore: epic 2 retro`
  - `cfc3fe3e feat(2.8): abandon plugin image`
  - `ffaad55a feat(2.6): plugin ui`
- Treat the first two commits as evidence of the superseded direction, not as the product model to preserve unchanged.

### Latest Technical Information

- The planning artifacts now define Story 3.3 as score-context rack-slot preparation, not direct instrument assignment.
- The current codebase now exposes explicit slot and route seams for Story 3.1 groundwork, and the earlier direct-assignment compatibility paths have been removed.
- No new external library or framework choice is required for this story.

### Project Structure Notes

- Preserve the existing artifact path and sprint key so sprint tracking continuity remains stable while the story content is corrected.
- Story 3.3 should feel like the next step after Story 3.2 current-rack visibility, not a return to the rejected assignment-first UX.
- The bottom panel should remain the current-rack surface; Plugin Manager remains app-scoped.

### References

- `_bmad-output/planning-artifacts/epics.md` — corrected Epic 3 Story 3.3 requirements and Story 3.4 / 3.5 boundaries
- `_bmad-output/planning-artifacts/prd.md` — score-first rack and routing goals, FR7, FR11, and MVP slot-preparation direction
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned playback state, recoverable degraded state, and ownership boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — score-first routing model, panels-versus-windows scope, and filtered plugin picker expectations
- `_bmad-output/planning-artifacts/archive/2026-03-epic-3-planning-history/sprint-change-proposal-2026-03-22-rack-routing-pivot.md` — archived rationale for rewriting Story 3.3 as a slot-preparation slice
- `_bmad-output/implementation-artifacts/3-1-represent-playback-assignment-state-on-score-instruments.md` — corrected rack-and-slot foundation notes
- `_bmad-output/implementation-artifacts/3-2-show-assignment-readiness-in-instrument-rows-and-panels.md` — corrected current-rack display slice that this story builds on
- `_bmad-output/implementation-artifacts/epic-2-retro-2026-03-22.md` — bounded-slice guardrails for Epic 3 preparation
- `docs/project-context.md` — FE/BE ownership rules, legacy seam guardrails, and Sequence-versus-Plugin-Manager boundaries
- `src/windows/main/screens/sequence/components/bottom-panel.tsx` — current-rack surface seam
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — compact score-context handoff seam
- `src/windows/main/screens/sequence/components/instrument-overview.tsx` — instrument-list orchestration seam
- `src/windows/main/screens/sequence/components/playback-status-panel.tsx` — superseded selected-instrument seam to avoid recentring
- `src/windows/main/components/plugins.tsx` — legacy contextual plugin surface to refine or constrain
- `src/windows/main/hooks/use-bottom-panel-state.ts` — bottom-panel state seam
- `src/services/actions.ts` — frontend mutation seam with direct-assignment residue to avoid extending blindly
- `src/services/types.ts` — frontend playback and inventory contract mirror
- `src/services/state/context.tsx` — general app-state mirror
- `src-tauri/src/score/instruments/mod.rs` — backend playback-domain mutation seam
- `src-tauri/src/playback/plugin_inventory.rs` — backend-owned usable plugin inventory seam
- `src-tauri/src/lib.rs` — Rust-to-frontend command and state wiring
- `src-tauri/src/ui/screens/sequence.rs` — Sequence UI state seam

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `3.3` for replacement after the rack-and-routing course correction.
- Sprint tracking still preserved the legacy story key and artifact path, so the rewrite keeps those stable while replacing the story content.
- Artifact review confirmed the approved Epic 3 sequence is now rack foundation, rack display, slot preparation, route authoring, shared-slot reuse, and only later activation.
- Source review confirmed the corrected workflow is now built on explicit slot and route seams, and direct-assignment compatibility paths should not be reintroduced.
- UX and architecture review confirmed slot preparation must stay score-contextual, compact, musician-friendly, and distinct from Plugin Manager responsibilities.
- Reworked the implementation after AC1 correction to use separate slot creation and slot plugin assignment invokes, matching the existing player and instrument mutation patterns.
- Rebuilt `PlaybackRackPanel` into mixer-style horizontal grey cards with an add-slot icon in the top bar and slot-local available-plugin pickers for empty or already-assigned slots.
- The combined `score_playback_slots_prepare` seam was removed because it encoded the wrong workflow shape for the corrected story.

### Completion Notes List

- Replaced the superseded direct-assignment Story 3.3 context with the corrected rack-slot preparation scope.
- Preserved explicit story boundaries so route authoring, shared-slot reuse, activation, and lifecycle behavior remain in later stories.
- Added implementation guardrails around legacy direct-assignment seams so the dev agent does not accidentally rebuild the rejected model.
- Incorporated user clarification that this story creates a slot and assigns the chosen plugin to it, with no remaining clarification question at story-prep time.
- Tightened the story wording so slot preparation is clearly separated from later instrument routing and later automatic default setup on instrument creation.
- Aligned the implementation to the corrected empty-slot-first workflow by creating slots separately and assigning or updating plugins from the slot-local picker.
- Replaced the previous stacked prepare UI with mixer-inspired horizontal rack cards, an add-slot top-bar button, and an end spacer for sparse slot counts.
- Removed the unnecessary combined prepare command and restored the explicit create-slot then assign-plugin mutation pattern.
- Updated focused frontend and backend tests around empty-slot creation, slot-local plugin assignment, and the absence of the combined prepare seam.
- Updated architecture, development-guide, and component-inventory docs to describe add-slot creation and slot-local available-plugin pickers inside Sequence.

### File List

- `_bmad-output/implementation-artifacts/3-3-manually-assign-a-plugin-to-a-score-instrument.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/component-inventory.md`
- `docs/development-guide.md`
- `src/services/actions.test.ts`
- `src/services/actions.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`

## Change Log

- 2026-03-22: Story created as the superseded score-context manual assignment continuation of the old Story 3.2 model.
- 2026-03-23: Story rewritten after course correction to the approved rack-slot preparation scope while preserving the existing artifact path and sprint key.
- 2026-03-23: Clarified that Story 3.3 creates a slot and assigns the chosen plugin to it, removing the obsolete open question.
- 2026-03-23: Clarified that Story 3.3 prepares a routeable slot destination only, while Story 3.4 owns routing and Story 3.6 owns automatic default setup on instrument creation.
- 2026-03-23: Implemented score-context slot preparation in the Sequence rack panel, added the backend slot-preparation command, added focused frontend/backend validation, and updated docs.
- 2026-03-23: Corrected the implementation to create empty slots from a top-bar add control, assign plugins from slot-local pickers, remove the combined prepare command, and restyle the rack surface into horizontal mixer-like cards.
