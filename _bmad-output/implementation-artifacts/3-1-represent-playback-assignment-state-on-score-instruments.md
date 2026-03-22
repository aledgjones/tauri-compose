# Story 3.1: Represent Playback Assignment State on Score Instruments

Status: done

## Story

As a composer,
I want score instruments to carry explicit playback assignment state,
so that Compose can tell me whether each instrument is ready, needs attention, or is unassigned.

## Acceptance Criteria

1. Given a score contains instruments that may or may not have playback assigned, when Compose resolves playback state for those instruments, then each instrument has explicit assignment state owned by the backend and that state is available to score-context UI surfaces.
2. Given some instruments may not yet have a valid playback mapping, when their state is resolved, then they are represented explicitly as unassigned or attention-needed and Compose does not silently substitute hidden fallback behavior.
3. Given this is the first score-assignment story, when it is complete, then the assignment model supports both direct single-part plugin assignments and explicit multi-part or channel-addressed assignments and it can represent the MVP case where multiple score instruments intentionally share one plugin instance.

## Tasks / Subtasks

- [x] Task 1: Replace the current single-field plugin assignment with a Rust-owned playback-assignment model that can represent current and future Epic 3 workflows without introducing frontend-owned truth (AC: 1, 2, 3)
  - [x] Introduce a durable backend assignment-intent type under the Rust score domain that can represent at least: unassigned, direct single-part assignment, explicit targeted multi-part or channel-addressed assignment, and the shared-instance intent needed for later Epic 3 stories.
  - [x] Keep the authoritative assignment truth backend-owned and serializable with the score while separating portable assignment intent from machine-local inventory state.
  - [x] Add compatibility helpers so existing code that only needs an effective plugin id can keep functioning during this slice without requiring Story 3.8 activation work to land early.
- [x] Task 2: Expose resolved assignment state through existing backend-to-frontend state contracts without shipping the later UI readiness design in this story (AC: 1, 2)
  - [x] Add a Rust-owned resolved-assignment or assignment-health view model that score-context surfaces can read in later stories.
  - [x] Mirror the new backend contract in TypeScript types using the repo's existing `snake_case` and numeric-enum conventions.
  - [x] Ensure score-context UI surfaces receive the new state through existing general-state flows rather than a new parallel provider or manager-local fetch path.
- [x] Task 3: Preserve current behavior for legacy consumers while moving them onto the new model seam (AC: 1, 2, 3)
  - [x] Update current assignment-related consumers that read `assigned_plugin_id` directly so they use the new helper or resolved-assignment seam instead of depending on the old field layout.
  - [x] Preserve current single-part assignment behavior for the existing instrument-row picker and keyboard audition path without widening this story into richer UI or transport changes.
  - [x] Keep transport, sampler sync, automation output, and selection-driven audition behavior functionally coherent after the model change.
- [x] Task 4: Add persistence, migration, and seam-level validation for the new assignment model (AC: 1, 2, 3)
  - [x] Update `.soloc` save/load serialization to persist the new portable assignment intent.
  - [x] Add the required pending migration in `src-tauri/src/migrations/next.rs` if persisted score data changes.
  - [x] Add focused Rust and TypeScript tests for serialization, migration, compatibility helpers, and state exposure to score-context surfaces.
- [x] Task 5: Update developer documentation for the new assignment-state model and slice boundaries (AC: 1, 2, 3)
  - [x] Update the data-model and architecture docs to describe the new assignment-intent versus resolved-state split.
  - [x] Keep the docs explicit that assignment representation is owned here, while readiness presentation, manual assignment UX, shared-instance UI, activation, and recovery remain owned by later stories.

## Dev Notes

### Story Intent

- Story 3.1 is the foundation slice for Epic 3.
- Its job is to define the durable backend-owned representation for playback assignment state on score instruments.
- This story must not try to finish the user-facing readiness UI, manual assignment flow, activation path, reopen recovery behavior, or full shared-instance runtime behavior.
- The current `assigned_plugin_id` field is too narrow for Epic 3's required single-part, explicit target, and shared-instance workflows. This story is where that limitation gets fixed at the model seam.

### User Decisions Captured

- Preserve the Epic 2 retrospective guardrail: this story creates one Rust-owned assignment truth and does not create a parallel frontend assignment model.
- Preserve the score-first UX promise: assignment remains a property of score instruments rather than becoming app-scoped host-topology configuration.
- Keep this slice representational. Do not absorb Story 3.2 readiness presentation, Story 3.3 manual assignment UX, Story 3.4 target-selection UX, Story 3.5 shared-instance UX legibility, Story 3.8 activation, or Epic 5 reopen/recovery behavior.

### Bounded Slice Template

#### This Story Owns

- Defining the durable Rust-owned playback-assignment representation on score instruments.
- Distinguishing portable assignment intent from machine-local plugin inventory truth.
- Exposing resolved assignment state to score-context UI consumers through existing app state.
- Migrating persisted score data if the instrument payload shape changes.
- Preserving current single-part assignment behavior through compatibility helpers while the broader Epic 3 workflow is still incomplete.

#### This Story Does Not Own

- The compact readiness badges and richer panel presentation language for end users. Story 3.2 owns that display slice.
- The musician-facing manual assignment picker workflow. Story 3.3 owns that UX.
- The explicit part or channel selection step for compatible multi-part plugins. Story 3.4 owns that.
- Shared-instance assignment UX and legibility across multiple instruments. Story 3.5 owns that.
- Automatic assignment for newly created instruments. Story 3.6 owns that.
- Reassignment and recovery UX beyond representational state. Story 3.7 owns that.
- Runtime activation/loading of assigned plugins for playback. Story 3.8 owns that.
- Session reopen restoration and degraded reopen recovery. Epic 5 owns that behavior.

#### Authoritative State Lives In

- Rust score-domain types under `src-tauri/src/score/` for persisted assignment intent.
- Rust-owned derived state emitted through the existing general app state for score-context UI consumption.
- Machine-local plugin availability remains in the existing plugin-inventory seam under `src-tauri/src/playback/plugin_inventory.rs` and must not be copied into `.soloc`.

#### Future Stories Intentionally Deferred

- User-facing compact state labels (`Ready`, `Needs Attention`, `Unavailable`) as polished score-context UI.
- Manual assignment controls, target selection flows, and shared-instance reuse UX.
- Playback activation, runtime lifecycle, and reopen recovery semantics.

#### Verification Required For Done

- Rust tests proving the new assignment model serializes, deserializes, and migrates correctly.
- Focused tests proving existing single-part assignment consumers still work through the new seam.
- Frontend tests proving score-context state consumers can read the new assignment contract without inventing local derived truth.
- Validation with the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None identified at story-prep time. The epic, architecture, UX, and retrospective guidance are aligned enough to proceed with a bounded representation slice.

### Scope Boundaries

- In scope:
  - Rust-owned assignment-intent representation on instruments.
  - Rust-owned resolved assignment state exposed to score-context surfaces.
  - Migration and persistence updates required by the new model.
  - Compatibility adjustments for existing transport, sampler sync, automation, and audition consumers.
- Out of scope:
  - New score-surface presentation polish beyond plumbing the new state.
  - New app-scoped windows, manager workflows, or plugin-editor affordances.
  - Full multi-part target selection UX or actual shared-instance orchestration.
  - Broad transport rearchitecture or realtime-path redesign.

### Cross-Story Alignment

- Story 3.2 depends on this story to expose one backend-owned resolved state that rows and panels can present without re-deriving health in React.
- Story 3.3 depends on this story to store manual single-part assignment as durable playback intent.
- Story 3.4 and Story 3.5 depend on this story to provide a representational home for explicit target choice and shared-instance linkage rather than bolting those semantics onto `assigned_plugin_id` later.
- Story 3.8 should be able to consume an effective plugin-identity helper or derived activation input without needing to own the persisted assignment schema itself.
- Epic 5 recovery stories should be able to degrade and restore assignment intent from the same durable model rather than reverse-engineering legacy string fields.

### Previous Story Intelligence

- Epic 2 established machine-local plugin inventory truth in `src-tauri/src/playback/plugin_inventory.rs` and a dedicated app-scoped browse surface without turning inventory into score-owned persisted data.
- Epic 2 also reinforced that backend-owned truth must remain singular and that assignment, activation, and recovery must be delivered as separate slices rather than prematurely bundled.
- The retro for Epic 2 explicitly called out Story 3.1 as the place to define backend-owned assignment truth while refusing hidden fallback behavior.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory, assignment state, playback preparation state, and transport state.
- `.soloc` stores portable project intent and assignment references, not machine-local plugin discovery/cache state.
- The frontend remains a thin presentation layer over Rust-owned assignment and health state.
- Score-context playback relationships belong in contextual surfaces; this story must not reopen app-scoped inventory-management concerns.
- Missing or invalid mappings must degrade to explicit recoverable states rather than silent fallback.
- Playback-critical consumers should keep using backend-owned, precomputed, lock-safe data rather than frontend-derived inference.

### Current Relevant Seams

- `src-tauri/src/score/instruments/mod.rs`
  - Current `Instrument` struct still persists `assigned_plugin_id: Option<String>`.
  - `Score::assigned_plugin_ids()` and `score_instruments_assign_plugin()` currently assume a single string plugin reference.
- `src-tauri/src/playback/sampler.rs`
  - `sync_sampler_with_score()` and `SamplerContext::sync_with_instruments()` currently consume a flat list of assigned plugin ids.
  - This story should preserve single-part compatibility while preparing for richer assignment semantics later.
- `src-tauri/src/playback/transport.rs`
  - `collect_scheduled_notes()` reads `instrument.assigned_plugin_id` directly today.
  - Do not widen this story into full shared-instance playback, but do stop transport from depending on the old field layout directly.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Tone audition currently depends on `instrument.assigned_plugin_id`.
  - Preserve current audition behavior via a compatibility seam instead of blocking on later activation stories.
- `src/services/types.ts`
  - Frontend `Instrument` mirrors the old `assigned_plugin_id` field today.
  - Update the TypeScript mirror and any read-only consumers to the new backend contract.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing row picker shows current single-part assignment.
  - Keep it working, but do not treat this story as the final readiness or assignment UX.
- `src-tauri/src/automation/registry.rs`
  - Current automation output exposes `assigned_plugin_id`.
  - Update the automation-facing payload carefully if the backend type changes so MCP and automation consumers stay aligned.
- `src-tauri/src/migrations/next.rs`
  - If persisted score data changes, the pending migration must land here as part of this story.

### Implementation Guidance

- Prefer introducing a new explicit assignment model rather than stuffing more semantics into `assigned_plugin_id`.
- Keep the model split clear:
  - portable score-owned assignment intent belongs in persisted score data
  - resolved assignment health/readiness belongs in backend-owned emitted state
  - machine-local plugin inventory and availability remain outside project files
- Preserve current single-part flows through adapters or helper methods while the rest of Epic 3 catches up.
- Avoid a big-bang replacement that forces every later story to undo temporary shortcuts.
- Do not make React responsible for inferring whether an assignment is unassigned, attention-needed, or unavailable from scattered fields.
- If you need a compact summary for current consumers, derive it in Rust and emit it as part of state rather than recomputing it independently in multiple UI surfaces.
- Keep explicit future support visible in the model for:
  - direct single-part assignment
  - targeted multi-part or channel-addressed assignment
  - intentional multi-instrument shared-instance linkage
- Do not implement richer routing or arbitrary host-topology concepts beyond those bounded MVP semantics.

### Persistence And Migration Requirements

- This story likely changes persisted score data beyond the current `assigned_plugin_id` string field.
- If the persisted schema changes, add the unreleased migration step in `src-tauri/src/migrations/next.rs`.
- Update any load/save tests that assume `Instrument { assigned_plugin_id }` is the entire persisted assignment model.
- Keep `.soloc` portable by persisting assignment references and user intent, not local inventory cache data or resolved machine-local availability.

### File Structure Requirements

- Backend/domain changes should stay centered in:
  - `src-tauri/src/score/instruments/`
  - `src-tauri/src/score/mod.rs`
  - `src-tauri/src/files.rs`
  - `src-tauri/src/migrations/next.rs`
  - `src-tauri/src/automation/registry.rs`
- Backend consumer updates will likely touch:
  - `src-tauri/src/playback/sampler.rs`
  - `src-tauri/src/playback/transport.rs`
- Frontend contract updates will likely touch:
  - `src/services/types.ts`
  - `src/services/actions.ts` only if action signatures genuinely need to evolve
  - read-only score-context consumers such as `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/hooks/use-keyboard-shortcuts.ts`
- Do not spread this story into Plugin Manager window code, plugin-editor host code, or unrelated transport UX.

### Testing Requirements

- Rust tests should cover:
  - default unassigned state for the new assignment model
  - serialization and deserialization of persisted assignment intent
  - migration from existing persisted `assigned_plugin_id` data if schema changes
  - compatibility helpers used by sampler sync, transport scheduling, and audition-related consumers
- Frontend tests should cover:
  - updated `Instrument` type consumption in score-context components
  - preservation of current assignment-picker rendering for the single-part case
  - no React-side reinvention of assignment truth
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings`

### Library / Framework Requirements

- Stay on the existing brownfield stack and current repo patterns.
- Keep FE/BE contracts `snake_case` and keep enum/value alignment explicit between Rust and TypeScript.
- Do not add a new state-management library or frontend-only playback model.
- Reuse existing Tauri command and state-emission seams instead of introducing parallel pathways.

### Documentation Impact

- Update `docs/data-models.md` to describe the new persisted playback-assignment representation on `Instrument` and the split between portable assignment intent and resolved state.
- Update `docs/architecture.md` where it currently references persisted `assigned_plugin_id` so the architecture matches the richer Epic 3 model.
- Update any developer guidance that still implies assignment state is only a nullable plugin-id string.
- Keep docs explicit that this story changes the model seam, not the later UX, activation, or recovery slices.

### Git Intelligence Summary

- Recent commits show Epic 2 closed with disciplined bounded slices rather than broad end-to-end playback work:
  - `485b5e6d feat: finish manual plugin inventory rescan workflow`
  - `f2bb4a6a feat(2.5): bind plugin ui`
  - `ffaad55a feat(2.6): plugin ui`
  - `cfc3fe3e feat(2.8): abandon plugin image`
  - `6f2d04fa chore: epic 2 retro`
- Keep the same pattern here: fix the assignment-model seam cleanly before layering UI and runtime behavior on top.

### Latest Technical Information

- The current codebase still routes assignment through `score_instruments_assign_plugin(instrument_id, plugin_id)` and mirrors that field into TypeScript.
- The current architecture and UX docs already expect a richer score-first assignment model with explicit part/channel choice and visible shared-instance linkage, so Story 3.1 should move the data model in that direction now rather than doubling down on a legacy string field.
- The project remains macOS-only, Rust-owned for domain truth, and offline-first; no external service or new framework choice is needed for this story.

### Project Structure Notes

- The current Sequence instrument row is a temporary consumer of assignment truth, not the final definition of assignment UX.
- The current flat `assigned_plugin_id` field appears across many test fixtures and helper constructors. Budget for broad but mechanical updates in tests and builder code.
- The existing sampler-era seams are legacy compatibility surfaces, not the target architecture for Epic 3. If a compatibility bridge is needed, keep it clearly transitional and backend-owned.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.1 through 3.8 requirements
- `_bmad-output/planning-artifacts/prd.md` — Epic 3 functional requirements and offline/local assignment rules
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned state, persisted intent, and FE/BE ownership boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — score-first assignment, readiness, and direct-assignment workflow expectations
- `_bmad-output/implementation-artifacts/epic-2-retro-2026-03-22.md` — bounded-slice guardrails and Epic 3 prep requirements
- `docs/project-context.md` — FE/BE ownership rules, migration workflow, and legacy sampler guardrails
- `docs/data-models.md` — current persisted `Instrument` shape
- `src-tauri/src/score/instruments/mod.rs` — current instrument model and assignment command
- `src-tauri/src/playback/sampler.rs` — current sampler sync dependency on assigned plugin ids
- `src-tauri/src/playback/transport.rs` — current transport dependency on direct plugin-id lookup
- `src/windows/main/hooks/use-keyboard-shortcuts.ts` — current audition dependency on instrument assignment
- `src/services/types.ts` — frontend mirror of instrument assignment fields
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — current score-context consumer of assignment state
- `src-tauri/src/automation/registry.rs` — automation-facing instrument payload shape

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Baseline commit before review-fix follow-up: `6f2d04fa1886a583bece70fbd5d8922957de8742`.
- Story selected explicitly as `3.1`.
- Sprint status showed `epic-3` in `backlog` and `3-1-represent-playback-assignment-state-on-score-instruments` in `backlog` before story creation.
- Artifact review confirmed Epic 2's retrospective requires the bounded-slice template for every Epic 3 story and explicitly names Story 3.1 as the backend-owned representation slice.
- Codebase analysis confirmed the current `assigned_plugin_id` string field is consumed directly by score instrumentation, transport scheduling, sampler sync, automation output, and keyboard audition.
- Architecture and UX analysis confirmed the target model must support direct single-part assignment, explicit multi-part/channel targets, and visible shared-instance linkage without drifting into DAW-style routing.
- Implemented `PlaybackAssignmentIntent`, `PlaybackAssignmentTarget`, and `ResolvedPlaybackAssignment` in `src-tauri/src/score/instruments/mod.rs`, then routed direct string consumers through backend-owned effective-plugin helpers.
- Emitted `resolved_playback_assignment_by_instrument_id` from `soloc://state/general` so score-context React surfaces consume backend-owned assignment status rather than re-deriving health locally.
- Added pending migration support in `src-tauri/src/migrations/next.rs` to rewrite legacy persisted `assigned_plugin_id` payloads into `assignment_intent`.
- Validation completed with `npm run lint`, `npm run check-types`, `npm run test:js`, and `cd src-tauri && cargo clippy --lib -- -D warnings`.
- Review follow-up: tightened compatibility helpers so targeted/shared intents no longer silently fall through legacy direct-assignment runtime paths.
- Review follow-up: resolved playback assignment status now consults machine-local inventory availability before marking a direct assignment ready.

### Completion Notes List

- Story context created as a bounded backend-owned representation slice.
- Guardrails added to prevent this story from absorbing readiness UI, manual assignment UX, shared-instance UX, activation, or recovery behavior.
- Migration and documentation impact called out explicitly because the persisted assignment model is expected to change.
- Replaced persisted instrument plugin assignment strings with a numeric-enum-backed assignment intent model that supports unassigned, direct, targeted, and shared-instance semantics.
- Preserved current single-part row picker, transport scheduling, sampler sync, automation payloads, sequencer track plugin resolution, and keyboard audition through backend-owned effective-plugin compatibility helpers.
- Added backend-emitted resolved assignment state to the general app state contract and updated TypeScript mirrors and tests to consume that seam.
- Added migration coverage plus score-domain tests for default unassigned state, effective plugin compatibility, targeted-assignment attention state, and resolved assignment map generation.
- Updated architecture and data-model docs to describe the persisted intent versus emitted resolved-state split and the bounded scope of this slice.
- Fixed review findings by making readiness inventory-aware and by gating legacy runtime compatibility to direct available assignments only.

### File List

- `_bmad-output/implementation-artifacts/3-1-represent-playback-assignment-state-on-score-instruments.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/data-models.md`
- `src-tauri/src/automation/registry.rs`
- `src-tauri/src/bin/performance.rs`
- `src-tauri/src/instructions/draw/draw_names.rs`
- `src-tauri/src/instructions/draw/draw_staves.rs`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
- `src-tauri/src/instructions/parse/get_vertical_spacing.rs`
- `src-tauri/src/instructions/parse/get_vertical_spans.rs`
- `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/migrations/next.rs`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src-tauri/src/playback/sampler.rs`
- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/score/flows.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src/services/state/context.tsx`
- `src/services/types.ts`
- `src/windows/main/components/mixer-channel.test.tsx`
- `src/windows/main/components/mixer.test.tsx`
- `src/windows/main/components/sequencer.tsx`
- `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview.test.tsx`
- `src/windows/main/screens/sequence/components/miniature-piano-roll.test.tsx`
- `src/windows/main/screens/setup/components/player-panel-instrument-list-item.test.tsx`
- `src/windows/main/screens/setup/components/player-panel-item.test.tsx`
- `src/windows/main/screens/setup/components/player-panel.test.tsx`

## Change Log

- 2026-03-22: Replaced persisted `assigned_plugin_id` with backend-owned `assignment_intent`, emitted resolved playback assignment state to score-context UI, migrated legacy payloads, updated compatibility consumers, refreshed tests, and documented the new assignment seam.
