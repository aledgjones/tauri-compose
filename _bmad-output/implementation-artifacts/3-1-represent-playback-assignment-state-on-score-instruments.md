# Story 3.1: Define Backend-Owned Rack, Slot, Target, Address, And Route State

Status: ready-for-dev

## Story

As a composer,
I want Compose to carry explicit backend-owned rack and route state,
so that score instruments can participate in a consistent playback topology without frontend-owned guesswork.

## Acceptance Criteria

1. Given a score contains instruments that may or may not have playback prepared, when Compose resolves playback topology for that score, then the backend owns explicit rack, slot, target, address, and route state and that state is available to score-context UI surfaces without frontend-owned truth.
2. Given some instruments may not yet have a valid playback mapping, when their topology is resolved, then they are represented explicitly as unrouted or attention-needed and Compose does not silently substitute hidden fallback behavior.
3. Given this is the first score-routing story, when it is complete, then the model supports both single-target plugin routing and explicit multi-target or channel-addressed routing and it can represent the MVP case where multiple score instruments intentionally share one rack slot through distinct routes.

## Tasks / Subtasks

- [ ] Task 1: Replace the assignment-first score model with backend-owned rack, slot, target, address, and route types that match the corrected Epic 3 language (AC: 1, 2, 3)
  - [ ] Define durable Rust score-domain types for the current score rack, hosted slots, stable product-facing targets, engine-specific address payloads, and instrument routes.
  - [ ] Keep the authoritative routing truth backend-owned and serializable with the score while separating portable rack or route intent from machine-local plugin inventory state.
  - [ ] Preserve the bounded MVP semantics only: single-target routing, explicit supported multi-target or channel-addressed routing, and shared-slot intent where compatible plugins expose it.
- [ ] Task 2: Expose backend-owned resolved route state through existing FE or BE state flows without creating parallel frontend topology truth (AC: 1, 2)
  - [ ] Add Rust-owned resolved rack or route state that score-context surfaces can consume in later Epic 3 stories.
  - [ ] Mirror the new backend contract in TypeScript with the repo's existing `snake_case` and numeric-enum conventions.
  - [ ] Ensure score-context UI surfaces receive rack and route state through existing general-state seams rather than a new provider or manager-local fetch path.
- [ ] Task 3: Migrate current direct plugin consumers onto backend-owned route helpers while preserving bounded compatibility where it is still needed (AC: 1, 2, 3)
  - [ ] Update current consumers that read direct plugin identity from instrument state so they use backend-owned effective-route helpers rather than legacy field layouts.
  - [ ] Keep current single-target compatibility working for existing row controls, keyboard audition, transport preparation, and automation seams where that is still required during the transition.
  - [ ] Avoid silently treating route resolution as generic DAW routing; only expose the product-approved rack or target semantics.
- [ ] Task 4: Add persistence, migration, and seam-level validation for the rack and routing foundation (AC: 1, 2, 3)
  - [ ] Update `.soloc` save or load serialization to persist the new portable rack and route intent.
  - [ ] Add the required pending migration in `src-tauri/src/migrations/next.rs` if persisted score data changes.
  - [ ] Add focused Rust and TypeScript tests for serialization, migration, compatibility helpers, and state exposure to score-context surfaces.
- [ ] Task 5: Update implementation-facing docs for the rack-and-routing pivot and this story's slice boundaries (AC: 1, 2, 3)
  - [ ] Update data-model and architecture docs so they describe rack, slot, target, address, and route ownership instead of direct instrument-to-plugin assignment.
  - [ ] Keep the docs explicit that this story owns the representational foundation, while rack display, slot creation, route authoring, activation, and recovery remain owned by later stories.

## Dev Notes

### Story Intent

- Story 3.1 is the foundation slice for the corrected Epic 3 rack-and-routing model.
- Its job is to define the durable backend-owned representation for the current score rack and each instrument's explicit route into that rack.
- This story must not try to finish the Sequence rack UI, slot-creation workflow, route-authoring UX, activation path, or reopen recovery behavior.
- The prior direct-assignment framing is now a superseded stepping stone. This story exists to give Epic 3 one stable playback topology model that later stories can build on cleanly.

### User Decisions Captured

- Preserve the Epic 2 retrospective guardrail: this story creates one Rust-owned routing truth and does not create a parallel frontend topology model.
- Preserve the score-first UX promise: score instruments route into a current score rack rather than owning app-scoped host topology directly.
- Keep this slice representational. Do not absorb Story 3.2 rack or route presentation, Story 3.3 slot creation, Story 3.4 route authoring, Story 3.5 shared-slot UX legibility, Story 3.8 activation, or Epic 5 reopen or recovery behavior.
- Keep the product-facing model stable: `target` is the user-meaningful destination, while `address` is the engine-specific routing payload. For MVP, the only supported address variant is VST-rooted.

### Bounded Slice Template

#### This Story Owns

- Defining the durable Rust-owned rack, slot, target, address, and route representation for score playback.
- Distinguishing portable rack or route intent from machine-local plugin inventory truth.
- Exposing resolved route state to score-context UI consumers through existing app state.
- Migrating persisted score data if the instrument or playback payload shape changes.
- Preserving bounded compatibility for current single-target consumers while the rest of Epic 3 catches up.

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
- Focused tests proving current single-target compatibility consumers still work through the new seam where temporary compatibility is required.
- Frontend tests proving score-context state consumers can read the new backend contract without inventing local derived truth.
- Validation with the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None identified at story-prep time. The updated epics, PRD, architecture, UX, and sprint-change proposal are aligned enough to proceed with a bounded foundation slice.

### Scope Boundaries

- In scope:
  - Rust-owned rack, slot, target, address, and route representation for score playback.
  - Rust-owned resolved route state exposed to score-context surfaces.
  - Migration and persistence updates required by the new model.
  - Compatibility adjustments for current transport, audition, automation, and transitional score-context consumers.
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
- Preserve current single-target behavior through bounded adapters or helper methods while later Epic 3 stories catch up.
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
- `_bmad-output/planning-artifacts/sprint-change-proposal-2026-03-22-rack-routing-pivot.md` — rationale for reopening and rewriting Story 3.1 around rack and route state
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

- Story selected explicitly as `3.1`.
- The planning artifacts now define Epic 3 Story 3.1 as `Define Backend-Owned Rack, Slot, Target, Address, And Route State`.
- The existing 3.1 implementation artifact was still framed around direct assignment and marked `done`, which no longer matched the approved rack-and-routing plan.
- Sprint tracking still showed `3-1-represent-playback-assignment-state-on-score-instruments: done` even though the sprint change proposal explicitly recommended reopening Story 3.1 under the corrected model.
- Artifact review confirmed the corrected story must be backend-first, Rust-owned, and bounded away from the later Sequence UI, slot-creation, route-authoring, activation, and recovery slices.
- Project-context review confirmed Rust remains the single source of truth and `.soloc` must keep machine-local inventory outside project persistence.

### Completion Notes List

- Story context rewritten from the superseded assignment-first foundation to the approved rack-and-routing foundation.
- Scope boundaries preserved so Story 3.1 owns only the backend topology model and its emitted state seams.
- Migration and documentation impact called out explicitly because the persisted playback-domain shape is expected to change again under the corrected model.
- Sprint tracking reset so Epic 3 can re-implement the true foundation instead of treating the superseded slice as complete.

### File List

- `_bmad-output/implementation-artifacts/3-1-represent-playback-assignment-state-on-score-instruments.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-03-23: Story rewritten to the approved rack-and-routing foundation scope, replacing the superseded assignment-first framing and resetting the slice to ready-for-dev.
