# Story 3.6: Attempt Automatic Routing For New Score Instruments

Status: blocked

Blocked Reason: User decided not to pursue the bounded auto-routing heuristic because the behavior is not useful enough to justify the added complexity. New instruments should continue to rely on explicit manual routing.

## Story

As a composer,
I want Compose to attempt a sensible playback route when I create a new score instrument,
so that I can get to playable setup faster without manual work every time.

## Acceptance Criteria

1. Given a user creates a new score instrument, when Compose has sufficient local inventory or mapping information to make a routing attempt, then it attempts to prepare a suitable playback route automatically and the result is reflected in the instrument's route state.
2. Given some plugins may require an explicit target decision to be unambiguous, when automatic routing cannot choose that target safely, then Compose leaves the instrument in an explicit recoverable state instead of guessing hidden target or shared-slot routing and the user can complete the route manually.
3. Given no suitable automatic route is available, when the new instrument is created, then the instrument remains explicitly unrouted and the user can see that further action is needed.
4. Given automatic routing is a convenience rather than hidden magic, when it succeeds or fails, then the resulting state is visible in the score workflow and the user can still manually change the route afterward.

## Tasks / Subtasks

- [ ] Task 1: Add a backend-owned auto-routing attempt at instrument creation time that preserves the rack-first model (AC: 1, 2, 3, 4)
  - [ ] Extend the current instrument-creation seam so new instruments can trigger one bounded backend auto-routing attempt before general state is emitted.
  - [ ] Keep the invariant from Story 3.1 and Story 3.4: if auto-routing succeeds, Compose prepares or reuses a valid slot destination first, then stores route intent to that existing slot.
  - [ ] Centralize the auto-routing decision in Rust so the frontend continues to display state rather than deciding candidates or mutating playback topology itself.
  - [ ] Ensure any successful auto-routing path updates all backend-owned state needed for route summaries and later playback work.
- [ ] Task 2: Define a bounded, workable candidate-selection strategy for the current MVP instead of inventing a durable mapping system (AC: 1, 2, 3)
  - [ ] Prefer a simple backend-owned candidate-selection heuristic grounded in currently available local inventory and existing instrument definition data.
  - [ ] Treat currently workable single-target VST paths such as BBCSO-style plugins as the primary proof path for this story.
  - [ ] Do not introduce expression-map-level logic, reusable mapping libraries, or broader playback asset systems that belong to later epics.
  - [ ] If multiple candidate plugins or targets are equally plausible and the backend cannot prove one safe choice, treat the case as ambiguous and leave the instrument unresolved rather than guessing.
- [ ] Task 3: Keep blocked shared-slot and multi-target behavior out of automatic routing while Story 3.5 remains blocked (AC: 2, 3, 4)
  - [ ] Do not auto-select `PluginPart`, `MidiChannel`, or shared-slot reuse behavior when the plugin path requires a human choice or a still-unproven multi-target capability.
  - [ ] Treat Story 3.5's blocked status as an implementation guardrail: no hidden shared-slot routing, no guessed channel assignment, no implicit multi-part reuse.
  - [ ] Leave ambiguous or unsupported multi-target candidates explicitly unrouted or attention-needed so the existing manual route flow can resolve them later.
  - [ ] Make sure successful auto-routing remains compatible with later manual rerouting and repair workflows.
- [ ] Task 4: Surface auto-routing outcomes clearly in existing Sequence score-context presentation (AC: 1, 3, 4)
  - [ ] Reuse the current backend-owned `resolved_playback_route_by_instrument_id` and readiness presentation surfaces so users can immediately see whether the new instrument is ready or still needs action.
  - [ ] Keep Plugin Manager app-scoped; do not turn automatic routing failure into a forced inventory-management workflow.
  - [ ] Ensure the score workflow shows a clear recoverable state when auto-routing fails or is skipped.
  - [ ] Preserve the user's ability to manually change or complete the route after the auto-routing attempt.
- [ ] Task 5: Add focused tests and documentation updates for the auto-routing attempt path (AC: 1, 2, 3, 4)
  - [ ] Add Rust tests for instrument creation with a clear single-target candidate, with no candidate, and with ambiguous or blocked multi-target cases.
  - [ ] Add frontend tests proving the new instrument's readiness or unrouted state is visible after creation and that manual routing remains available afterward.
  - [ ] Update documentation so the auto-routing behavior is described as a best-effort convenience with safe fallback, not as a guarantee.

## Dev Notes

### Story Intent

- Story 3.6 is the bounded auto-routing slice that follows explicit manual route authoring.
- Its purpose is not to make routing magical. Its purpose is to make a sensible best-effort attempt when a new score instrument is created and to leave clear recoverable state when that attempt cannot be made safely.
- This story is workable even while Story 3.5 is blocked because its acceptance criteria explicitly require Compose not to guess hidden target or shared-slot routing when choice would be ambiguous.
- The current product goal can be validated through single-target VST paths such as BBCSO-style plugins without forcing multi-target shared-slot proof into this story.

### User Decisions Captured

- Story 3.5 remains blocked and must not be pulled into this story implicitly.
- BBCSO-style single-instrument VST support is sufficient to prove the current VST playback principle for Story 3.6.
- When the backend cannot choose a route safely, the correct behavior is explicit unrouted or recoverable state, not a clever guess.
- Sequence remains the score-context surface where the result should be visible and manually correctable.

### Bounded Slice Template

#### This Story Owns

- One backend-owned automatic routing attempt when a new score instrument is created.
- Creating any needed slot or route state only when the backend can make a safe unambiguous choice.
- Explicit fallback to unrouted or recoverable state when no suitable or safe route can be chosen.
- Score-context visibility of the outcome and preservation of manual route control afterward.

#### This Story Does Not Own

- Shared-slot reuse or multi-target auto-selection. Story 3.5 owns that and is currently blocked.
- Repair and reroute workflows beyond the automatic attempt plus existing manual path. Story 3.7 owns the dedicated repair flow.
- Activation of plugin runtime for playback. Story 3.8 owns that.
- Expression-map logic, durable mapping assets, or later playback semantics.
- Plugin Manager workflow expansion or native plugin UI troubleshooting.

#### Authoritative State Lives In

- Rust-owned persisted `Instrument.playback_route` intent.
- Rust-owned rack and slot state on `score.playback.rack`.
- Rust-owned emitted `resolved_playback_route_by_instrument_id` presentation state.
- Backend-owned machine-local plugin inventory and any backend-owned candidate-selection logic used for the auto-routing attempt.

#### Future Stories Intentionally Deferred

- Shared-slot reuse and multi-target routing in Story 3.5.
- Dedicated reroute and repair flow in Story 3.7.
- Activation and first audible playback in Story 3.8.
- Session persistence and reopen recovery in Story 3.9 and Epic 5.

#### Verification Required For Done

- Backend tests prove a newly created instrument auto-routes only when a safe candidate exists.
- Tests prove ambiguous or unavailable candidates leave explicit unrouted or recoverable state rather than guessed routing.
- Frontend tests prove the resulting readiness state is visible and manual route changes remain possible afterward.
- Validation uses the project-standard commands relevant to touched files.

#### Open Questions Requiring Human Clarification

- None. The user explicitly clarified that BBCSO-style single-target VST support is enough to continue proving the current playback direction while shared-slot multi-target work remains blocked.

### Scope Boundaries

- In scope:
  - backend-owned auto-routing attempt on instrument creation
  - single-target candidate selection using current local inventory and bounded heuristics
  - safe fallback to unrouted or attention-needed state
  - score-context visibility of success or failure
  - focused tests and docs for best-effort auto-routing
- Out of scope:
  - shared-slot reuse or guessed multi-target assignment
  - expression-map or durable mapping asset creation
  - forced Plugin Manager workflows for ordinary auto-routing failure
  - runtime playback activation or transport behavior

### Cross-Story Alignment

- Story 3.1 established the rack-first ownership model: slots are prepared destinations and instruments own routes to existing slots.
- Story 3.2 made readiness and route state legible in Sequence and provides the surfaces this story should reuse.
- Story 3.3 established slot preparation and plugin assignment as backend-owned rack work.
- Story 3.4 established explicit route authoring and the backend-owned route-choice contract the user can fall back to when auto-routing cannot choose safely.
- Story 3.5 is blocked, which means Story 3.6 must explicitly avoid hidden shared-slot or multi-target routing behavior.
- Story 3.7 will later own deliberate repair and reroute flows after this story's best-effort attempt.

### Previous Story Intelligence

- The Story 3.4 artifact established that React must not infer valid targets and that manual route authoring uses explicit backend-owned route choices.
- The blocked Story 3.5 artifact now provides an important guardrail: no guessed shared-slot reuse, no hidden channel assignment, and no fake multi-target capability proof.
- The current backend `score_instruments_create` seam still creates instruments with `playback_route: None` and emits general state immediately, so 3.6 needs to extend that creation path or extract a helper around it.
- Existing backend route-choice emission is still bounded to direct plugin output for prepared non-empty slots, which makes single-target auto-routing the current practical path.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory, slot state, route state, route validation, and route-health derivation.
- React remains the score-context presentation layer; it must not decide candidates or create auto-routing topology itself.
- Plugin Manager stays app-scoped; automatic routing outcomes should be visible in Sequence without pushing users into inventory management.
- Missing or unresolved candidates must degrade to explicit recoverable unrouted state rather than silent fallback or guessed topology.
- If persisted route semantics change beyond the already-modeled fields, migration impact must be assessed and handled in `src-tauri/src/migrations/next.rs`.

### Current Relevant Seams

- `src-tauri/src/score/instruments/mod.rs`
  - `score_instruments_create` currently creates the instrument with `playback_route: None` and emits updated general state.
  - `create_playback_slot`, `assign_plugin_to_slot`, and `assign_instrument_route` are the existing backend seams that can support a rack-first automatic attempt.
- `src-tauri/src/lib.rs`
  - Emits `resolved_playback_route_by_instrument_id` and `route_target_options_by_slot_id` on general state updates.
- `src/services/actions.ts`
  - Frontend create-instrument action currently calls `score_instruments_create`; do not move auto-routing decision-making into the frontend.
- `src/services/actions.test.ts`
  - Existing instrument-create invoke test should evolve if the contract changes.
- `src/services/types.ts`
  - Frontend mirrors of route state and readiness state.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Current compact score-context route summary seam.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Existing helper that should present the auto-routing result from backend truth.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Existing score-context rack surface the user can use afterward if the automatic attempt leaves recoverable state.

### Legacy Or Superseded Seams To Avoid

- Any reintroduction of direct instrument-to-plugin assignment state.
- Any frontend-owned auto-routing candidate inference.
- Any hidden shared-slot or multi-target routing while Story 3.5 is blocked.
- Expanding sampler-era seams as the VST auto-routing model.

### Implementation Guidance

- Start from the current backend instrument-creation seam and keep the auto-routing attempt inside Rust.
- Preserve the rack-first order of operations: if a route is going to exist, the slot destination must exist first.
- Prefer a bounded, obviously safe candidate strategy grounded in current local inventory and existing instrument-definition information.
- Treat single-target candidate success as the primary path for this story.
- If no clear single-target candidate exists, do not guess. Leave the new instrument explicitly unrouted and let the existing manual route-authoring flow handle it.
- Do not couple this story to multi-target capability proof, plugin activation, or transport behavior.

### Persistence And Migration Requirements

- No migration should be required if implementation only creates already-modeled slot and route intent during instrument creation.
- If any persisted schema changes are introduced beyond the current slot and route model, add or update the pending migration in `src-tauri/src/migrations/next.rs`.
- Keep machine-local plugin inventory or candidate-selection cache details out of `.soloc` unless the architecture explicitly decides they are portable project intent.

### File Structure Requirements

- Backend or contract work should stay centered in:
  - `src-tauri/src/score/instruments/mod.rs`
  - `src-tauri/src/score/playback.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/automation/registry.rs` if create-command or route-command automation contracts change
  - `src-tauri/src/migrations/next.rs` only if persisted schema changes
- Frontend work should stay centered in:
  - `src/services/actions.ts`
  - `src/services/actions.test.ts`
  - `src/services/types.ts`
  - `src/services/state/context.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
- Documentation updates should stay centered in:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/data-models.md`
  - `docs/project-context.md` if the agent guardrails change

### Testing Requirements

- Backend tests should cover:
  - creating a new instrument when one clear single-target candidate exists and auto-routing succeeds
  - creating a new instrument when no candidate exists and the result remains explicitly unrouted
  - creating a new instrument when only ambiguous multi-target or shared-slot candidates exist and the backend refuses to guess
  - preserving valid slot and route truth when auto-routing succeeds
- Frontend tests should cover:
  - readiness summary updates after instrument creation
  - explicit unrouted or attention-needed state after failed or skipped auto-routing
  - continued manual routing availability after the automatic attempt
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib`

### Library / Framework Requirements

- Stay on the existing brownfield React, TypeScript, Rust, and Tauri stack.
- Reuse the existing backend state emission flow rather than introducing a frontend-owned auto-routing model.
- Keep FE and BE enums aligned exactly for route target and route address modes.
- Prefer seam-local tests over a new repo-wide auto-routing harness.

### Documentation Impact

- Update `docs/architecture.md` if the instrument-creation seam now includes bounded automatic routing behavior.
- Update `docs/development-guide.md` so it describes auto-routing as a best-effort attempt with explicit unrouted fallback.
- Update `docs/data-models.md` if route-intent or slot-preparation behavior during creation needs clarification.
- Update `docs/project-context.md` if the required AI-agent rule changes around blocked shared-slot behavior or bounded auto-routing.

### Git Intelligence Summary

- Recent branch history shows the current Epic 3 baseline and the blocked shared-slot prerequisite:
  - `a3ed9de4 feat: investigate HALion`
  - `dc6da699 feat(3.4): slot assignment`
  - `6d6e6ef8 chore: proposed rack ux improvements`
  - `0727803c feat(3.3): slot assignment`
  - `0abd1a14 feat(3.2): assignment readieness`
- Treat the HALion investigation commit and the blocked 3.5 story as guardrails, not prerequisites for this story's single-target proof path.

### Latest Technical Information

- `score_instruments_create` currently inserts new instruments with `playback_route: None` and emits updated general state immediately.
- The backend already has the necessary primitive seams for a rack-first automatic attempt: slot creation, slot plugin assignment, and explicit route assignment.
- `route_target_options_by_slot_id` currently exposes only direct plugin output for prepared slots, which aligns with making single-target auto-routing the current safe MVP path.
- The frontend create-instrument action already uses the backend command directly, so auto-routing can be introduced without inventing a new frontend orchestration layer.

### Project Structure Notes

- Preserve the legacy sprint key and artifact path for story-tracking continuity even though the epic text says "routing" while the sprint key uses "assignment".
- Keep the auto-routing outcome visible in existing Sequence score-context surfaces rather than inventing a new onboarding workflow.
- Treat blocked Story 3.5 as an active scope boundary for this artifact.

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 3.6 acceptance criteria and downstream boundaries
- `_bmad-output/planning-artifacts/prd.md` — FR8, FR9, and unrouted fallback expectations
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned routing, recoverable state, and score-workflow boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — readiness legibility and score-context scope model
- `_bmad-output/implementation-artifacts/3-4-select-a-part-or-channel-when-assigning-a-compatible-multi-part-plugin.md` — explicit manual route-authoring foundation
- `_bmad-output/implementation-artifacts/3-5-reuse-a-compatible-multi-part-plugin-instance-across-multiple-score-instruments.md` — blocked shared-slot guardrails
- `docs/project-context.md` — backend ownership, app-scoped Plugin Manager, and workflow rules
- `src-tauri/src/score/instruments/mod.rs` — instrument creation, slot preparation, route mutation, and route resolution seams
- `src-tauri/src/lib.rs` — general playback state emission seam
- `src/services/actions.ts` — frontend create-instrument and route action seams
- `src/services/actions.test.ts` — existing invoke-contract coverage for create and route actions
- `src/services/types.ts` — frontend mirrors of route state
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — current route-summary seam
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts` — readiness presentation seam
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx` — current rack surface used for manual follow-up

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `3.6`.
- Sprint tracking preserved the legacy `3-6-attempt-automatic-assignment-for-new-score-instruments` key and artifact naming.
- Artifact review confirmed that `3.6` is explicitly compatible with leaving instruments unrouted when safe auto-routing is not possible.
- Adjacent-story analysis confirmed that blocked Story 3.5 must remain a guardrail: no hidden shared-slot or multi-target auto-selection.
- Code exploration confirmed that the backend already owns instrument creation, slot preparation, and explicit route assignment, while the create command currently leaves `playback_route` empty.
- Implemented and validated a bounded auto-routing prototype, then reverted it after product review because the heuristic did not provide enough user value.
- Story status changed to `blocked` rather than `review` so the artifact reflects the decision not to pursue this direction.

### Completion Notes List

- Created the Story 3.6 context artifact as the bounded auto-routing slice after manual route authoring.
- Captured the user decision that BBCSO-style single-target VST paths are enough to keep proving the VST playback direction while 3.5 stays blocked.
- Directed the dev agent toward extending backend-owned instrument creation and route preparation rather than inventing a frontend orchestration flow.
- Added explicit guardrails against hidden shared-slot or multi-target guesses while Story 3.5 remains blocked.
- The bounded auto-routing prototype was intentionally backed out. Compose continues to rely on explicit manual routing for newly created instruments.

### File List

- \_bmad-output/implementation-artifacts/3-6-attempt-automatic-assignment-for-new-score-instruments.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- docs/architecture.md
- docs/data-models.md
- docs/development-guide.md
- src-tauri/src/automation/registry.rs
- src-tauri/src/playback/plugin_inventory.rs
- src-tauri/src/score/instruments/mod.rs
- src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx
- src/windows/main/screens/sequence/components/playback-readiness-presentation.ts

### Change Log

- 2026-03-24: Reverted the bounded backend auto-routing prototype after deciding the heuristic was not useful enough to keep.
