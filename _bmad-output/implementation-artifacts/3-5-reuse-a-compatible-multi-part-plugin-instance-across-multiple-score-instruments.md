# Story 3.5: Reuse A Compatible Rack Slot Across Multiple Score Instruments

Status: blocked

Blocked reason: unresolved verified multi-target plugin path for shared-slot routing. HALion remains unproven after investigation, and BBCSO is sufficient to continue validating the single-instrument VST playback principle without forcing Story 3.5 now.

## Story

As a composer,
I want compatible score instruments to share one rack slot through explicit route targets,
so that HALion-style playback setups work as a first-class MVP workflow instead of a later extension.

## Acceptance Criteria

1. Given one score instrument is already using a compatible rack slot, when the user routes another score instrument to that same slot through a different supported target, then Compose reuses the existing compatible slot rather than silently creating a duplicate instance and the resulting shared-slot relationship is explicit in backend-owned route state.
2. Given more than one score instrument can share the same compatible rack slot, when those routes are shown in score-context surfaces, then the UI makes the shared-slot linkage legible and users can still understand each instrument's own readiness and route target.
3. Given the compatible rack slot later becomes unavailable or invalid, when Compose refreshes route state, then each affected score instrument moves into an explicit recoverable state and the product preserves the shared-slot relationship as user intent rather than guessing a replacement topology.

## Tasks / Subtasks

- [ ] Task 1: Extend the backend route-capability seam so one prepared slot can expose more than one verified route target without frontend inference (AC: 1, 3)
  - [ ] Build on the existing rack-first model where slots already exist and instruments only own routes to those slots.
  - [ ] Replace the current direct-output-only `playback_route_choices_for_slot` emission with backend-owned multi-target choices only when a prepared plugin slot has real, bounded capability proof.
  - [ ] Keep `PluginPart` and `MidiChannel` choices grounded in plugin-derived capability facts or an already-proven bounded backend seam; do not invent target lists in React.
  - [ ] Preserve the current explicit failure behavior when the backend cannot prove safe shared-slot targets for a slot.
- [ ] Task 2: Make slot reuse explicit in route mutation and resolution behavior rather than creating duplicate slot topology (AC: 1, 3)
  - [ ] Ensure assigning a second instrument to the same slot through a distinct verified target preserves the existing `slot_id` and stores the new instrument's own target and address intent.
  - [ ] Do not treat reuse as a direct-root duplicate route for multiple instruments unless the backend explicitly proves that is a valid supported shared-target case.
  - [ ] Keep degraded resolution per instrument so a missing plugin, invalid slot, or invalidated target moves each route to explicit recoverable state without silently clearing or replacing intent.
  - [ ] Preserve the rule that removing or unrouting one instrument must not implicitly delete the shared slot or another instrument's route.
- [ ] Task 3: Surface shared-slot linkage in Sequence without turning the workflow into host-topology management (AC: 2)
  - [ ] Extend the existing Sequence score-context surfaces so users can see when multiple instruments intentionally share the same slot.
  - [ ] Keep the primary mental model instrument-first: each row still shows that instrument's own readiness and target while the shared slot remains legible as a bounded secondary detail.
  - [ ] Reuse the existing instrument overview and playback readiness presentation seams instead of creating a new app-scoped shared-routing window.
  - [ ] Keep Plugin Manager app-scoped and keep native plugin UI as a secondary handoff rather than the default shared-slot workflow.
- [ ] Task 4: Guard the story against the current HALion validation gap (AC: 1, 2, 3)
  - [ ] Treat the HALion investigation artifact as a prerequisite evidence source for any real multi-target emission or manual validation because HALion is currently the only available multi-timbral plugin in this repo context.
  - [ ] If the backend still cannot prove HALion-backed multi-target capability safely, stop short of shipping guessed `PluginPart` or `MidiChannel` choices and escalate via course-correction rather than faking Story 3.5 completion.
  - [ ] Keep any new diagnostics, capability facts, or slot-sharing metadata backend-owned and explicit about what is proven versus still unproven.
- [ ] Task 5: Add focused backend, frontend, and documentation validation for shared-slot reuse (AC: 1, 2, 3)
  - [ ] Add Rust tests for verified multi-target choice emission, second-instrument route assignment into an existing slot, and degraded shared-slot recovery behavior.
  - [ ] Add frontend tests for shared-slot legibility in Sequence, per-instrument readiness summaries, and non-duplication of slot creation during second-instrument routing.
  - [ ] Update docs to describe shared-slot reuse as a bounded score-first route behavior and to record any remaining HALion evidence limits clearly.

## Dev Notes

### Story Intent

- Story 3.5 is the first slice that makes one-slot multi-instrument routing real rather than merely modeled in enums and persistence types.
- It follows Story 3.4, which created explicit route authoring, but 3.4 still emits only direct plugin output choices for prepared slots.
- This story owns deliberate shared-slot reuse only for compatible multi-target plugins. It does not own plugin discovery, slot preparation, editor-host proof work, activation for playback, or transport behavior.
- The user clarified an important execution constraint: HALion has not been made to work yet and is currently the only available multi-timbral instrument for validating this path.

### User Decisions Captured

- Shared-slot reuse must stay rack-first: slots own plugin assignment, instruments own explicit routes into existing slots.
- Compose must not guess multi-target capability, shared-target legality, or replacement topology.
- Sequence remains the score-context workflow surface for shared-slot routing visibility.
- HALion readiness remains a real prerequisite confidence gate, not a detail to hand-wave away inside Story 3.5.

### Bounded Slice Template

#### This Story Owns

- Backend-owned emission of verified multi-target route choices for a prepared slot when the slot's plugin genuinely supports them.
- Assigning a second or later score instrument to an already-existing compatible slot through a distinct verified target and address.
- Sequence score-context presentation that makes shared-slot linkage legible while preserving instrument-specific route readiness.
- Shared-slot degraded-state handling that preserves route intent and avoids silent fallback topology.

#### This Story Does Not Own

- Plugin discovery, inventory scanning, or plugin-manager browsing.
- Slot creation or slot plugin assignment. Story 3.3 owns that.
- Initial explicit route authoring contract creation. Story 3.4 owns that foundation.
- Making HALion editor-host usable; that belongs to the HALion investigation artifact and any follow-up implementation from it.
- Route activation for playback. Story 3.8 owns that.
- Reopen recovery, transport stability, or audible playback proof.

#### Authoritative State Lives In

- Rust-owned persisted `Instrument.playback_route` intent.
- Rust-owned `score.playback.rack` slot truth and plugin assignment truth.
- Rust-owned `route_target_options_by_slot_id` capability contract.
- Rust-owned `resolved_playback_route_by_instrument_id` presentation truth.
- Backend-owned plugin inventory and any backend-owned diagnostics or capability facts used to justify richer targets.

#### Future Stories Intentionally Deferred

- Automatic routing for newly created instruments in Story 3.6.
- Repair and reroute workflows in Story 3.7.
- Runtime activation and playback participation in Story 3.8.
- Session persistence and reopen recovery in Story 3.9 and Epic 5.

#### Verification Required For Done

- Backend tests prove a second instrument can reuse an existing slot only through a verified supported target path.
- Tests prove the implementation does not silently create a duplicate slot when reusing a compatible slot.
- Frontend tests prove shared-slot linkage is legible from existing Sequence surfaces without introducing a new topology-management workflow.
- Validation proves that when a shared slot becomes invalid, each affected instrument moves to explicit recoverable state without silent fallback.
- Any claimed multi-target behavior is backed by backend capability proof rather than enums alone.

#### Open Questions Requiring Human Clarification

- None. The user explicitly clarified that HALion is not yet working and is the only current multi-timbral validation target, so the story should carry that constraint as a guardrail rather than asking the developer to guess around it.

### Scope Boundaries

- In scope:
  - backend-owned multi-target choice emission for a prepared compatible slot
  - explicit reuse of one slot across multiple instruments through distinct supported targets
  - shared-slot legibility in Sequence score-context surfaces
  - degraded shared-slot route recovery without silent replacement
  - focused tests and docs for bounded shared-slot semantics
- Out of scope:
  - inventing target lists in React
  - pretending direct plugin output alone satisfies multi-part shared-slot support
  - HALion editor-host lifecycle repair work
  - slot preparation, transport, reopen recovery, or plugin-manager scope expansion

### Cross-Story Alignment

- Story 3.1 established the rack-first playback ownership model that this story must preserve.
- Story 3.2 made readiness and rack summaries visible in Sequence, providing the presentation seams this story should extend.
- Story 3.3 prepared slots and assigned plugins but explicitly stopped short of route creation.
- Story 3.4 added explicit route authoring and a backend-owned route-choice contract, but the current emitted choices still only cover direct plugin output.
- Story 3.5 is therefore the first story that must turn the existing `PluginPart` or `MidiChannel` model into verified reusable slot behavior.
- Story 3.8 must later activate whatever verified shared-slot target intent this story persists; 3.5 must not collapse into playback activation.

### Previous Story Intelligence

- The Story 3.4 artifact already corrected Epic 3 toward explicit rack-first route authoring and established `route_target_options_by_slot_id` as the authoritative backend contract.
- Current frontend route selection already lives on `InstrumentOverviewRow`, while `PlaybackRackPanel` remains focused on slot preparation and bounded editor handoff.
- The backend `assign_instrument_route` seam already accepts a full `InstrumentPlaybackRoute`, so 3.5 should extend verified route-choice emission and resolution rather than inventing a parallel mutation path.
- The codebase already defines `PluginPart` and `MidiChannel` enums, but current `playback_route_choices_for_slot` still returns only `Plugin output` for non-empty slots.
- The HALion investigation quick spec exists specifically to avoid distorting Story 3.5 around an unproven plugin path.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory, slot truth, route truth, capability facts, and recoverable state derivation.
- React remains the presentation and command layer. It must not infer shared-slot capability or synthesize plugin parts or MIDI channels on its own.
- `.soloc` persists portable rack and route intent only; machine-local plugin inventory or discovery details remain outside project files.
- Missing or invalid shared-slot routes must degrade to explicit recoverable state rather than silent fallback or duplicate-instance creation.
- If persisted route or slot-sharing semantics change beyond the current model, migration impact must be assessed and handled in `src-tauri/src/migrations/next.rs`.

### Current Relevant Seams

- `src-tauri/src/score/instruments/mod.rs`
  - Current ownership seam for route choice emission, route assignment, and resolved playback state.
  - Today it emits only direct plugin output choices for non-empty slots.
- `src-tauri/src/score/playback.rs`
  - Canonical data model for `InstrumentPlaybackRoute`, `PlaybackRouteChoice`, `PlaybackRouteTargetMode`, and `PlaybackRouteAddressMode`.
- `src-tauri/src/lib.rs`
  - Emits `route_target_options_by_slot_id` and `resolved_playback_route_by_instrument_id` on general state updates.
- `src/services/types.ts`
  - Frontend mirror of playback route and route-choice contracts.
- `src/services/actions.ts`
  - Existing explicit route command seam used by Sequence surfaces.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Current score-context route-selection seam that should remain the primary reuse entry path.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Existing presentation helper for route target, address, and readiness summaries.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Current slot-preparation and bounded editor-handoff seam that may need shared-slot legibility support but should not become a new routing topology manager.
- `_bmad-output/implementation-artifacts/tech-spec-investigate-halion-failure-before-story-3-5.md`
  - Current prerequisite evidence artifact describing why HALion readiness still gates trustworthy 3.5 implementation.

### Legacy Or Superseded Seams To Avoid

- Any reintroduction of direct instrument-to-plugin assignment instead of route-to-slot intent.
- Any frontend-owned multi-part capability matrix.
- Any claim that shared-slot reuse is complete while HALion-backed multi-target proof is still unavailable.
- Expanding Plugin Manager or native plugin UI into the default workflow for ordinary score-context routing.

### Implementation Guidance

- Start by extending backend-owned route-choice emission so prepared slots can report more than one verified choice when the plugin and bounded host seam can genuinely support it.
- Reuse the existing explicit route mutation seam rather than creating a second “shared slot” command path.
- Make slot reuse explicit by preserving one `slot_id` across multiple instruments with distinct target and address payloads.
- Keep the UX score-first: users should understand “these instruments share slot 2, but each route has its own target” rather than being asked to manage host topology.
- If HALion capability proof is still blocked, keep the story implementation honest. Do not ship guessed part or channel options just because the enums exist.

### Persistence And Migration Requirements

- No migration should be required if implementation only starts emitting and consuming richer already-modeled route choices while preserving the existing `InstrumentPlaybackRoute` shape.
- If new persisted fields are introduced to preserve shared-slot intent or capability facts beyond the current route model, add or update the pending migration in `src-tauri/src/migrations/next.rs`.
- Keep machine-local plugin-derived capability facts outside `.soloc` unless the architecture explicitly decides they are portable project intent.

### File Structure Requirements

- Backend or contract work should stay centered in:
  - `src-tauri/src/score/instruments/mod.rs`
  - `src-tauri/src/score/playback.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/automation/registry.rs` if route command contracts change
  - `src-tauri/src/migrations/next.rs` only if persisted schema changes
- Frontend work should stay centered in:
  - `src/services/types.ts`
  - `src/services/actions.ts`
  - `src/services/actions.test.ts`
  - `src/services/state/context.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
- Documentation updates should stay centered in:
  - `docs/architecture.md`
  - `docs/data-models.md`
  - `docs/development-guide.md`
  - `docs/project-context.md` if agent rules or required guardrails change

### Testing Requirements

- Backend tests should cover:
  - emitting verified multi-target choices for a compatible prepared slot
  - assigning two instruments to one slot through distinct valid targets
  - rejecting unsupported or guessed multi-target assignments
  - preserving explicit recoverable state when shared-slot plugins or targets become invalid
- Frontend tests should cover:
  - showing shared-slot linkage from existing Sequence surfaces
  - preserving per-instrument readiness and target summaries for shared slots
  - refusing workflows that would imply duplicate slot creation during reuse
  - keeping Plugin Manager and native plugin UI out of the common shared-slot route flow
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib`

### Library / Framework Requirements

- Stay on the existing React, TypeScript, Rust, and Tauri stack.
- Keep all FE and BE enums aligned exactly for route target and address modes.
- Reuse the existing backend-emitted general state shape rather than adding frontend-owned playback state.
- Prefer seam-local tests and bounded diagnostics over a new repo-wide multi-part harness.

### Documentation Impact

- Update `docs/architecture.md` if shared-slot reuse changes the authoritative route-choice contract or shared-slot state model.
- Update `docs/data-models.md` if the meaning of shared-slot route target or address semantics is refined.
- Update `docs/development-guide.md` to describe how shared-slot reuse behaves in Sequence and to state any remaining HALion validation limit clearly.
- Update `docs/project-context.md` if the required AI-agent guardrails around shared-slot proof or HALion dependency change.

### Git Intelligence Summary

- Recent branch history shows the current Epic 3 path and the new prerequisite investigation:
  - `a3ed9de4 feat: investigate HALion`
  - `dc6da699 feat(3.4): slot assignment`
  - `6d6e6ef8 chore: proposed rack ux improvements`
  - `0727803c feat(3.3): slot assignment`
  - `0abd1a14 feat(3.2): assignment readieness`
- Treat `a3ed9de4` as critical context: the repo now explicitly records that HALion readiness is not yet proven and that Story 3.5 must not fake that proof.

### Latest Technical Information

- `InstrumentPlaybackRoute` already persists `slot_id`, `target`, and `address`, so the data model can represent shared-slot targets without inventing a new route type.
- `PlaybackRouteTargetMode` and `PlaybackRouteAddressMode` already include `PluginPart` and `MidiChannel`, but current production route-choice emission still exposes only direct plugin output.
- `route_target_options_by_slot_id` is already part of the backend-emitted general state, so shared-slot capability work should extend that contract instead of creating a second capability channel.
- The HALion investigation artifact states that verified multi-target evidence must come from plugin-derived facts observed from a real HALion instance or bounded host attempt; static enum support is not proof.

### Project Structure Notes

- Preserve the legacy sprint key and artifact path for sprint-tracking continuity.
- Keep shared-slot routing on the existing Sequence score-context seams rather than inventing a new app-scoped workflow.
- Treat the current HALion investigation artifact as a prerequisite confidence guardrail for implementation, not optional background reading.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 3 Story 3.5 acceptance criteria and downstream boundaries
- `_bmad-output/planning-artifacts/prd.md` — FR12 and bounded one-slot multi-instrument routing expectations
- `_bmad-output/planning-artifacts/architecture.md` — backend-owned rack, slot, route, and recoverable-state rules
- `_bmad-output/planning-artifacts/ux-design-specification.md` — score-first shared-slot legibility and bounded target-choice UX
- `_bmad-output/implementation-artifacts/3-4-select-a-part-or-channel-when-assigning-a-compatible-multi-part-plugin.md` — explicit route-authoring foundation and route-choice contract
- `_bmad-output/implementation-artifacts/tech-spec-investigate-halion-failure-before-story-3-5.md` — prerequisite HALion readiness investigation and proof constraints
- `docs/project-context.md` — backend ownership, Sequence scope, and plugin-inventory rules
- `docs/development-guide.md` — maintainer guidance and HALion validation posture
- `docs/architecture.md` — current backend-emitted route state and playback workflow guidance
- `docs/data-models.md` — persisted playback route and rack semantics
- `src-tauri/src/score/instruments/mod.rs` — route choice emission, mutation, and resolution seam
- `src-tauri/src/score/playback.rs` — canonical route and slot data model
- `src-tauri/src/lib.rs` — general-state emission seam
- `src/services/types.ts` — frontend mirror of playback route contracts
- `src/services/actions.ts` — explicit route invoke seam
- `src/services/state/context.tsx` — frontend state mirror
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — score-context route-selection seam
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts` — readiness and route-summary seam
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx` — slot-preparation and bounded editor-handoff seam

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `3.5`.
- Sprint tracking preserved the legacy `3-5-reuse-a-compatible-multi-part-plugin-instance-across-multiple-score-instruments` key and artifact naming.
- Artifact review confirmed that planning documents already require one-slot multi-instrument routing as bounded MVP behavior.
- Code exploration confirmed that the current route-choice contract and route-assignment seam exist, but production choice emission still stops at direct plugin output.
- Branch history and the HALion investigation artifact confirmed that HALion is not yet a proven working multi-target plugin path and must remain a guardrail for Story 3.5 rather than an implicit assumption.

### Completion Notes List

- Created the Story 3.5 context artifact as the bounded shared-slot reuse slice after explicit route authoring.
- Captured the current HALion constraint as a first-class implementation guardrail because it is the only available multi-timbral validation target and is not yet working.
- Directed the dev agent toward extending backend-owned route-choice proof and shared-slot legibility instead of inventing frontend capability logic.
- Preserved the rack-first invariant and explicitly deferred activation, recovery, and transport work to later stories.
- Story status changed to `blocked` after sprint review because the multi-target plugin prerequisite remains unresolved and the team can continue proving core VST playback with BBCSO's single-instrument path.

### File List

- \_bmad-output/implementation-artifacts/3-5-reuse-a-compatible-multi-part-plugin-instance-across-multiple-score-instruments.md
