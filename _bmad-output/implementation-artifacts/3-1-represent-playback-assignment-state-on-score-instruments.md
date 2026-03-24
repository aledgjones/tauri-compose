# Story 3.1: Define Backend-Owned Rack, Slot, Target, Address, And Route State

Status: done

## Story

As a composer,
I want Compose to carry explicit backend-owned rack and route state,
so that score instruments can participate in a consistent playback topology without frontend-owned guesswork.

## Acceptance Criteria

1. Given a score contains instruments that may or may not have playback prepared, when Compose resolves playback topology for that score, then the backend owns explicit rack, slot, target, address, and route state and that state is available to score-context UI surfaces without frontend-owned truth.
2. Given some instruments may not yet have a valid playback mapping, when their topology is resolved, then they are represented explicitly as unrouted or attention-needed and Compose does not silently substitute hidden fallback behavior, including the case where an instrument routes to an existing slot that does not currently have a plugin assigned.
3. Given this is the first score-routing story, when it is complete, then the model supports both single-target plugin routing and explicit multi-target or channel-addressed routing and it can represent the MVP case where multiple score instruments intentionally share one rack slot through distinct routes.
4. Given Story 3.1 owns the foundation slice, when it is complete, then backend-owned mutation seams exist for slot creation or removal, plugin assignment or unassignment on slots, and instrument route assignment or unassignment without making instruments the owner of slot lifecycle.

## Tasks / Subtasks

- [x] Task 1: Replace the assignment-first score model with backend-owned rack, slot, target, address, and route types that match the corrected Epic 3 language (AC: 1, 2, 3)
  - [x] Define durable Rust score-domain types for the current score rack, hosted slots, stable product-facing targets, engine-specific address payloads, and instrument routes.
  - [x] Keep the authoritative routing truth backend-owned and serializable with the score while separating portable rack or route intent from machine-local plugin inventory state.
  - [x] Allow a rack slot to exist before a plugin is assigned so rack preparation and instrument routing remain separate concerns.
  - [x] Preserve the bounded MVP semantics only: single-target routing, explicit supported multi-target or channel-addressed routing, and shared-slot intent where compatible plugins expose it.
- [x] Task 2: Expose backend-owned resolved route state through existing FE or BE state flows without creating parallel frontend topology truth (AC: 1, 2)
  - [x] Add Rust-owned resolved rack or route state that score-context surfaces can consume in later Epic 3 stories.
  - [x] Mirror the new backend contract in TypeScript with the repo's existing `snake_case` and numeric-enum conventions.
  - [x] Ensure score-context UI surfaces receive rack and route state through existing general-state seams rather than a new provider or manager-local fetch path.
- [x] Task 3: Add minimal backend-owned slot and route mutation seams without preserving legacy direct-assignment integrity beyond compile/run stability (AC: 1, 2, 3, 4)
  - [x] Add explicit backend commands for slot creation or removal, slot plugin assignment or unassignment, and instrument route assignment or unassignment.
  - [x] Replace instrument-facing direct plugin mutation with route-first seams; any temporary residue exists only where needed to keep the app compiling and running during the transition.
  - [x] Keep slot lifecycle independent from instrument deletion or route removal.
- [x] Task 4: Migrate current direct plugin consumers onto backend-owned route helpers without treating legacy behavior as a compatibility requirement (AC: 1, 2, 3)
  - [x] Update current consumers that read direct plugin identity from instrument state so they use backend-owned effective-route helpers rather than legacy field layouts.
  - [x] Keep current row controls, keyboard audition, transport preparation, and automation seams compiling and runnable while they move onto backend-owned route helpers.
  - [x] Avoid silently treating route resolution as generic DAW routing; only expose the product-approved rack or target semantics.
- [x] Task 5: Add persistence, migration, and seam-level validation for the rack and routing foundation (AC: 1, 2, 3, 4)
  - [x] Update `.soloc` save or load serialization to persist the new portable rack and route intent.
  - [x] Add the required pending migration in `src-tauri/src/migrations/next.rs` if persisted score data changes.
  - [x] Add focused Rust and TypeScript tests for serialization, migration, compatibility helpers, and state exposure to score-context surfaces.
- [x] Task 6: Update implementation-facing docs for the rack-and-routing pivot and this story's slice boundaries (AC: 1, 2, 3, 4)
  - [x] Update data-model and architecture docs so they describe rack, slot, target, address, and route ownership instead of direct instrument-to-plugin assignment.
  - [x] Keep the docs explicit that this story owns the representational foundation, while rack display, slot creation, route authoring, activation, and recovery remain owned by later stories.

## Dev Notes

### Story Intent

- Story 3.1 is the foundation slice for the corrected Epic 3 rack-and-routing model.
- Its job is to define the durable backend-owned representation for the current score rack and each instrument's explicit route into that rack.
- The rack owns slot lifecycle, slots can exist before a plugin is assigned, and instruments only store routes to already-existing slots.
- This story must not try to finish the Sequence rack UI, slot-creation workflow, route-authoring UX, activation path, or reopen recovery behavior.
- The prior direct-assignment framing is now a superseded stepping stone. This story exists to give Epic 3 one stable playback topology model that later stories can build on cleanly.

### User Decisions Captured

- Preserve the Epic 2 retrospective guardrail: this story creates one Rust-owned routing truth and does not create a parallel frontend topology model.
- Preserve the score-first UX promise: score instruments route into a current score rack rather than owning app-scoped host topology directly.
- Keep this slice representational. Do not absorb Story 3.2 rack or route presentation, Story 3.3 slot creation, Story 3.4 route authoring, Story 3.5 shared-slot UX legibility, Story 3.8 activation, or Epic 5 reopen or recovery behavior.
- Include the minimal backend mutation seams needed so later stories can mutate the clarified model without re-opening Story 3.1.
- Keep the product-facing model stable: `target` is the user-meaningful destination, while `address` is the engine-specific routing payload. For MVP, the only supported address variant is VST-rooted.

### Bounded Slice Template

#### This Story Owns

- Defining the durable Rust-owned rack, slot, target, address, and route representation for score playback.
- Defining the backend-owned slot lifecycle and minimal mutation seams for creating or removing slots, assigning or unassigning plugins on slots, and assigning or unassigning instrument routes.
- Distinguishing portable rack or route intent from machine-local plugin inventory truth.
- Exposing resolved route state to score-context UI consumers through existing app state.
- Migrating persisted score data if the instrument or playback payload shape changes.
- Keeping the app compiling and runnable while current consumers move onto the new backend-owned route model.

#### This Story Does Not Own

- The Sequence bottom-panel rack surface or row-level route summaries. Story 3.2 owns that display slice.
- The musician-facing slot-creation workflow from available plugins. Story 3.3 owns that UX.
- The explicit route-creation or target-selection workflow. Story 3.4 owns that.
- Shared-slot UX legibility across instruments. Story 3.5 owns that.
- Automatic routing for newly created instruments. Story 3.6 owns that.
- Repair or rerouting UX beyond representational state. Story 3.7 owns that.
- Runtime activation or loading of routed slots and targets for playback. Story 3.8 owns that.
- Session reopen restoration and degraded reopen recovery. Epic 5 owns that behavior.

#### Authoritative State Lives In

- Rust score-domain types under `src-tauri/src/score/` for persisted rack or route intent.
- Rust-owned derived state emitted through the existing general app state for score-context UI consumption.
- Machine-local plugin availability remains in the existing plugin-inventory seam under `src-tauri/src/playback/plugin_inventory.rs` and must not be copied into `.soloc`.

#### Future Stories Intentionally Deferred

- User-facing rack display, route summaries, and compact readiness presentation in Sequence.
- Slot creation, route editing, target selection, and shared-slot reuse UX.
- Playback activation, runtime lifecycle, and reopen recovery semantics.
- Route-owned expression-map editing and secondary native-plugin-editor handoffs.

#### Verification Required For Done

- Rust tests proving the new rack and route model serializes, deserializes, and migrates correctly.
- Focused tests proving current consumers still compile and run through the new seam while the legacy direct-assignment path is being retired.
- Frontend tests proving score-context state consumers can read the new backend contract without inventing local derived truth.
- Validation with the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None identified at story-prep time. The updated epics, PRD, architecture, UX, and sprint-change proposal are aligned enough to proceed with a bounded foundation slice.

### Scope Boundaries

- In scope:
  - Rust-owned rack, slot, target, address, and route representation for score playback.
  - Backend-owned slot lifecycle primitives that later stories can call without reintroducing instrument-owned slot creation semantics.
  - Rust-owned resolved route state exposed to score-context surfaces.
  - Migration and persistence updates required by the new model.
  - Compile/run-stable migration adjustments for current transport, audition, automation, and transitional score-context consumers.
- Out of scope:
  - New score-surface presentation polish beyond plumbing the new state.
  - New app-scoped windows, Plugin Manager expansion, or plugin-editor affordances.
  - Full route-authoring UX, slot-picking UI, or shared-slot orchestration UX.
  - Broad transport rearchitecture or realtime-path redesign.

### Cross-Story Alignment

- Story 3.2 depends on this story to expose one backend-owned rack or route state that rows and the Sequence bottom panel can present without React-side reinvention.
- Story 3.3 depends on this story to create or load slots into a durable score-context rack rather than persisting direct instrument-to-plugin assignment.
- Story 3.4 and Story 3.5 depend on this story to provide a representational home for explicit target choice and shared-slot linkage rather than bolting those semantics onto assignment-intent leftovers later.
- Story 3.8 should be able to consume backend-owned routed slot or target inputs without needing to own the persisted playback-topology schema itself.
- Epic 5 recovery stories should be able to degrade and restore rack and route intent from the same durable model rather than reverse-engineering legacy assignment fields.

### Prior Work Intelligence

- Epic 2 established machine-local plugin inventory truth in `src-tauri/src/playback/plugin_inventory.rs` and a dedicated app-scoped browse surface without turning inventory into score-owned persisted data.
- Epic 2 also reinforced that backend-owned truth must remain singular and that routing, activation, and recovery must be delivered as separate slices rather than prematurely bundled.
- The sprint change proposal for the rack-and-routing pivot explicitly states Story 3.1 should no longer remain done as written; it must become the backend foundation story for rack, slot, target, address, and route state.
- The previous Story 3.1 implementation artifact captured useful backend-owned model-seam thinking, but it is still framed as direct assignment. Treat that earlier slice as input to refine, not as the accepted end state for the corrected Epic 3 plan.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory, rack state, slot state, route state, playback preparation state, and transport state.
- `.soloc` stores portable project intent and rack or route references, not machine-local plugin discovery or cache state.
- The frontend remains a thin presentation layer over Rust-owned route and health state.
- Score-context playback relationships belong in contextual surfaces; this story must not reopen app-scoped inventory-management concerns.
- Missing or invalid mappings must degrade to explicit recoverable unrouted states rather than silent fallback.
- Playback-critical consumers should keep using backend-owned, precomputed, lock-safe data rather than frontend-derived inference.

### Current Relevant Seams

- `src-tauri/src/score/instruments/mod.rs`
  - Current instrument playback intent is still the nearest persisted seam and will likely need refactoring or extraction to support route-first ownership cleanly.
  - Any legacy direct-plugin helpers should become temporary compatibility shims, not the long-term model.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Remains the machine-local inventory truth and should inform route resolution without becoming persisted score data.
- `src-tauri/src/playback/transport.rs`
  - Playback preparation and scheduling must stop depending on direct instrument-plugin identity and move toward backend-owned route resolution.
- `src-tauri/src/playback/sampler.rs`
  - Still a transitional legacy seam; only keep bounded compatibility as needed, and do not use it as the model template for routing.
- `src-tauri/src/automation/registry.rs`
  - Automation-facing playback payloads may need careful evolution if route state replaces direct plugin identity in the backend contract.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Tone audition currently depends on direct assignment-era seams and may need backend-owned effective-route compatibility helpers.
- `src/services/types.ts`
  - Frontend mirrors must move from assignment-first shapes toward rack or route-first backend contracts.
- `src-tauri/src/migrations/next.rs`
  - If persisted score data changes, the pending migration must land here as part of this story.

### Implementation Guidance

- Prefer introducing an explicit rack-and-routing model rather than stretching assignment-intent terminology to cover new semantics.
- Keep the model split clear:
  - portable score-owned rack or route intent belongs in persisted score data
  - resolved route health or readiness belongs in backend-owned emitted state
  - machine-local plugin inventory and availability remain outside project files
- Keep the app compiling and runnable while current consumers move onto the new route helpers; do not preserve legacy direct-assignment behavior beyond that stability bar.
- Avoid a big-bang frontend rewrite. Replace the backend model seam first, then let later stories reshape Sequence surfaces around it.
- Do not make React responsible for inferring unrouted, missing-target, or attention-needed states from scattered fields.
- If a compact summary is needed for current consumers, derive it in Rust and emit it as part of state rather than recomputing it independently in multiple UI surfaces.
- Keep explicit future support visible in the model for:
  - direct single-target routing
  - targeted multi-target or channel-addressed routing
  - intentional multi-instrument shared-slot linkage
- Do not implement arbitrary bus routing, generic host topology, or non-VST address semantics beyond the approved MVP boundary.

### Persistence And Migration Requirements

- This story is expected to change persisted score data beyond the current assignment-era shape.
- If the persisted schema changes, add the unreleased migration step in `src-tauri/src/migrations/next.rs`.
- Update any load or save tests that assume direct plugin identity on the instrument is the full persisted playback model.
- Keep `.soloc` portable by persisting rack and route references plus user intent, not local inventory cache data or resolved machine-local availability.

### File Structure Requirements

- Backend or domain changes should stay centered in:
  - `src-tauri/src/score/`
  - `src-tauri/src/files.rs`
  - `src-tauri/src/migrations/next.rs`
  - `src-tauri/src/playback/`
  - `src-tauri/src/automation/registry.rs`
- Backend consumer updates will likely touch:
  - `src-tauri/src/playback/transport.rs`
  - `src-tauri/src/playback/sampler.rs`
  - `src-tauri/src/lib.rs` only if new state payloads or commands must be wired through existing seams
- Frontend contract updates will likely touch:
  - `src/services/types.ts`
  - `src/services/state/context.tsx`
  - read-only score-context consumers that currently expect assignment-era payloads
- Do not spread this story into Plugin Manager window code, plugin-editor host code, or unrelated transport UX.

### Testing Requirements

- Rust tests should cover:
  - default empty or unrouted state for the new rack and route model
  - serialization and deserialization of persisted rack or route intent
  - migration from existing persisted assignment-era data if schema changes
  - compatibility helpers used by transport preparation, audition-related consumers, and any temporary single-target bridge logic
- Frontend tests should cover:
  - updated type consumption in score-context components that read the backend routing contract
  - no React-side reinvention of rack or route truth
  - continued bounded compatibility for current read-only consumers if transitional emitted fields remain
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings`

### Library / Framework Requirements

- Stay on the existing brownfield stack and current repo patterns.
- Keep FE/BE contracts `snake_case` and keep enum or value alignment explicit between Rust and TypeScript.
- Do not add a new state-management library or frontend-only routing model.
- Reuse existing Tauri command and state-emission seams instead of introducing parallel pathways.

### Documentation Impact

- Update `docs/data-models.md` to describe the persisted rack, slot, target, address, and route representation and the split between portable intent and resolved state.
- Update `docs/architecture.md` where it still references assignment-first ownership so the implementation guide matches the corrected Epic 3 model.
- Update any developer guidance that still implies playback state is only a direct instrument-to-plugin relationship.
- Keep docs explicit that this story changes the model seam, not the later rack UI, route-authoring UX, activation, or recovery slices.

### Latest Technical Information

- The planning artifacts now consistently describe Epic 3 as a rack-and-routing epic rather than a direct-assignment epic.
- The current codebase still contains assignment-era terminology and compatibility seams, so the developer should expect a careful model transition rather than a one-file rename.
- The project remains macOS-only, Rust-owned for domain truth, and offline-first; no external service or new framework choice is needed for this story.

### Project Structure Notes

- The Sequence bottom panel is now the intended rack-facing score-context surface for Story 3.2 and later, not the place to define backend truth in this story.
- The current flat direct-plugin assumptions likely appear across test fixtures, helper constructors, emitted state payloads, and automation seams. Budget for broad but mechanical updates in backend and contract tests.
- Legacy sampler-era seams are transitional compatibility surfaces, not the target architecture for Epic 3. If a compatibility bridge is needed, keep it clearly temporary and backend-owned.

### References

- `_bmad-output/planning-artifacts/epics.md` — corrected Epic 3 story set and Story 3.1 acceptance criteria
- `_bmad-output/planning-artifacts/prd.md` — Epic 3 functional requirements and offline or local routing rules
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned state, persisted intent, and FE or BE ownership boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — score-first routing, rack context, and direct-routing workflow expectations
- `_bmad-output/planning-artifacts/archive/2026-03-epic-3-planning-history/sprint-change-proposal-2026-03-22-rack-routing-pivot.md` — archived rationale for reopening and rewriting Story 3.1 around rack and route state
- `_bmad-output/implementation-artifacts/epic-2-retro-2026-03-22.md` — bounded-slice guardrails and Epic 3 prep requirements
- `docs/project-context.md` — FE or BE ownership rules, migration workflow, and legacy sampler guardrails
- `src-tauri/src/score/instruments/mod.rs` — current nearest persisted playback-intent seam
- `src-tauri/src/playback/plugin_inventory.rs` — machine-local plugin inventory truth
- `src-tauri/src/playback/transport.rs` — current playback-preparation seam that must move off direct instrument-plugin assumptions
- `src-tauri/src/playback/sampler.rs` — transitional compatibility seam only
- `src/windows/main/hooks/use-keyboard-shortcuts.ts` — current audition dependency on assignment-era data
- `src/services/types.ts` — frontend mirror of playback-domain fields
- `src-tauri/src/automation/registry.rs` — automation-facing playback payload shape

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Implemented a new persisted score playback model in `src-tauri/src/score/playback.rs` and wired `Score.playback` plus `Instrument.playback_route` through Rust-owned state and migrations.
- Replaced emitted assignment-era general state with `resolved_playback_route_by_instrument_id` and updated bounded compatibility consumers in transport, audition, automation, and Sequence score-context surfaces.
- Added pending migration coverage for assignment-intent payloads, including `next_id`-aware slot allocation and durable rack construction during load.
- Fixed a transport test-fixture regression by restoring direct route assignment in multi-track scheduling fixtures.
- Updated implementation-facing docs in `docs/architecture.md` and `docs/data-models.md` to describe rack, slot, target, address, and route ownership.
- Validated the story with `npm run check-types`, `npm run test:js`, `npm run lint`, `cargo clippy --lib -- -D warnings`, and `cargo test --lib`.

### Completion Notes List

- Added backend-owned rack, slot, target, address, and route types to persisted score state and kept the portable rack model separate from machine-local plugin inventory truth.
- Exposed backend-owned resolved route state through the existing general app-state seam and updated TypeScript mirrors to the new route-first contract.
- Migrated transport preparation, keyboard audition, automation payloads, and Sequence read-only playback readiness consumers to backend-owned effective-route helpers.
- Added or updated Rust and TypeScript tests for migration, state exposure, compatibility helpers, and routing-aware playback scheduling.
- Updated architecture and data-model documentation to reflect the rack-and-routing foundation while preserving later-story boundaries for route authoring, activation, and recovery.

### File List

- `_bmad-output/implementation-artifacts/3-1-represent-playback-assignment-state-on-score-instruments.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/data-models.md`
- `src-tauri/src/score/playback.rs`
- `src-tauri/src/score/mod.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/automation/registry.rs`
- `src-tauri/src/migrations/next.rs`
- `src-tauri/src/score/flows.rs`
- `src-tauri/src/bin/performance.rs`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/instructions/draw/draw_names.rs`
- `src-tauri/src/instructions/draw/draw_staves.rs`
- `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
- `src-tauri/src/instructions/parse/get_vertical_spacing.rs`
- `src-tauri/src/instructions/parse/get_vertical_spans.rs`
- `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`
- `src/services/types.ts`
- `src/services/state/context.tsx`
- `src/windows/main/hooks/use-bottom-panel-state.ts`
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
- `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
- `src/windows/main/components/sequencer.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
- `src/windows/main/screens/sequence/components/playback-status-panel.tsx`
- `src/windows/main/screens/sequence/components/playback-status-panel.test.tsx`

## Change Log

- 2026-03-23: Implemented the backend-owned rack-and-routing foundation, updated route-first docs, passed the required validation suite, and moved the story to review.
