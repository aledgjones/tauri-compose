# Story 1.6: Initialize And Restore Slot-Owned Sessions On Assignment And Reopen

Status: done

## Story

As a composer,
I want assigned slots to initialize their plugin instances during assignment, preparation, or project reopen from portable saved intent, including restorable plugin configuration,
so that the plugin is already the intended running instrument before I ever open its editor.

## Acceptance Criteria

1. **Given** I assign a plugin to a slot
   **When** the backend accepts the assignment
   **Then** the slot enters an explicit initialization or recovery path that can produce a real running slot-owned session without requiring editor-open
   **And** the resulting state is surfaced through backend-owned readiness semantics

2. **Given** a project is reopened and the plugin is available
   **When** slot and route intent are restored
   **Then** the slot-owned session is restored or reinitialized before editor-open and without rewriting project intent
   **And** the restored session reflects the portable saved intent recorded in `.soloc`, including the previously selected restorable instrument or patch state needed for the intended running instrument

3. **Given** the plugin cannot be initialized or restored
   **When** the backend surfaces state
   **Then** Compose reports an explicit recoverable non-ready condition rather than silently pretending the slot is initialized
   **And** the score remains usable

4. **Given** `.soloc` is loaded
   **When** slot and plugin configuration are rehydrated
   **Then** the backend uses the portable saved intent to rebuild the slot-owned runtime session, including slot-to-plugin relationships and any restorable plugin configuration required to recover the intended instrument selection
   **And** the file is not required to contain live runtime state

## Tasks / Subtasks

- [x] Extend the Rust-owned slot lifecycle so assignment and project restore rebuild runtime sessions before editor-open (AC: 1, 2, 4)
  - [x] Audit the current slot lifecycle triggers in `src-tauri/src/score/instruments/mod.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/files.rs` to confirm exactly when `sync_slot_runtime_sessions(...)` already runs and where assignment-time or reopen-time gaps still exist.
  - [x] Ensure plugin assignment, project replacement/load, and any other owning restoration seam trigger slot-session rebuild through the existing slot-owned runtime registry in `src-tauri/src/playback/plugin_editor_poc.rs` rather than through editor-open.
  - [x] Keep same-session editor attach/detach secondary to this work. Do not move canonical ownership back onto editor-open, and do not introduce a second singleton or bundle-level runtime seam.
- [x] Define and persist only the portable rehydration intent required to restore the intended running instrument (AC: 2, 4)
  - [x] Audit the current `.soloc` playback payload in `src-tauri/src/files.rs`, `src-tauri/src/score/playback.rs`, and related score types to determine whether the existing slot and route intent already carries enough restorable plugin configuration.
  - [x] If persisted semantics must change, add the minimum portable fields required for restorable plugin configuration, keep machine-local discovery and live runtime state out of `.soloc`, and update `src-tauri/src/migrations/next.rs` in the same story.
  - [x] Preserve `next_id` discipline and any slot-id migration behavior. Do not derive new slot ids from serialized numeric collisions or bundle identity.
- [x] Surface explicit backend-owned recovery and readiness semantics for initialized versus non-ready slots (AC: 1, 3)
  - [x] Keep `PlaybackActivationRuntime` as readiness metadata derived from live slot sessions, not as the canonical runtime-session registry.
  - [x] Ensure failed initialization or reopen restoration degrades to explicit backend-owned recoverable state such as `NeedsAttention` or `Unavailable` instead of fake readiness, silent fallback, or route loss.
  - [x] Preserve score usability when recovery fails: route intent stays legible and recoverable even when the current machine cannot produce a live runtime session.
- [x] Add seam-local regression coverage for assignment, reopen, persistence, and failure handling (AC: 1, 2, 3, 4)
  - [x] Extend Rust tests in `src-tauri/src/playback/plugin_editor_poc.rs`, `src-tauri/src/playback/runtime_activation.rs`, and `src-tauri/src/files.rs` for assignment-time initialization, reopen-time restoration, missing-plugin degradation, and same-bundle multi-slot isolation.
  - [x] Add or refine targeted tests around persistence round-trips so `.soloc` proves portable rehydration intent without serializing live runtime objects.
  - [x] Add frontend tests only if emitted readiness payloads or existing rack-surface assumptions materially change.
- [x] Update documentation and story artifacts for the restored slot-owned-session contract (AC: 2, 4)
  - [x] Update docs and planning artifacts if persistence shape, reopen behavior, or maintainer proof expectations change.
  - [x] Keep Epic 4 gating language aligned: Story 1.6 is part of the slot-owned runtime-session prerequisite, not proof of audible playback, multi-instrument behavior, or broader plugin compatibility.

### Review Findings

- [x] [Review][Patch] Clear persisted `plugin_state` when runtime capture reports no transferable state [src-tauri/src/playback/plugin_editor_poc.rs:4226]
- [x] [Review][Patch] Surface save-time slot-state export failures instead of silently writing stale restore intent [src-tauri/src/files.rs:2364]

## Dev Notes

### Story Intent

Story 1.6 closes the remaining Epic 1 ownership gap after Stories 1.4 and 1.5. The repo now has slot-owned live runtime sessions and transient editor attachment, but this story must ensure those live sessions are created from slot lifecycle seams that occur before editor-open and survive `.soloc` reopen from portable saved intent.

This is still foundation work, not feature expansion:

- make assignment-time and reopen-time slot-session restoration trustworthy
- keep `.soloc` portable and backend-owned
- preserve explicit recoverable non-ready states
- do not broaden scope into audible playback, richer route UX, or generalized plugin-state authoring

### Story Foundation

- Story 1.3 established the narrow single-instrument-VST-instance trust baseline and blocked Epic 4 until the slot-owned architecture is proven durable across save and reopen.
- Story 1.4 moved canonical live VST ownership into a slot-keyed runtime-session registry in `src-tauri/src/playback/plugin_editor_poc.rs` and clarified that `.soloc` stores portable intent, not live runtime state.
- Story 1.5 proved native editor windows are transient attach/detach views over those slot-owned sessions rather than the creator of the canonical instance.
- Story 1.6 is the assignment-time and reopen-time restoration seam that makes those earlier foundations usable without requiring editor-open.

### Previous Story Intelligence

- Story 1.4 already created the slot-owned registry and routed lifecycle refresh through app-owned seams such as assignment, unassignment, slot removal, inventory refresh, reset, and project load.
- Story 1.4 also documented a critical limitation: docs started to mention restorable plugin configuration, but the persisted shape might still be under-specified. Story 1.6 must either prove the current payload is enough or add the minimum portable fields needed.
- Story 1.5 narrowed the editor contract further: editor close detaches view ownership but must not destroy the slot-owned runtime session while the slot still exists.
- Repo memory confirms `PlaybackActivationRuntime` is readiness metadata only, slot sessions are keyed by `slot_id`, same-bundle isolation depends on slot identity plus bundle path, and live host objects must stay out of `.soloc`.

### Technical Requirements

- Canonical live plugin ownership remains the slot-owned runtime-session registry under `src-tauri/src/playback/plugin_editor_poc.rs`.
- Assignment or restore must rebuild slot-owned sessions before editor-open. Editor-open must stay a bounded attach flow over an existing session.
- `.soloc` must continue to store portable rehydration intent only. Do not persist host objects, editor-window state, machine-local discovery cache, or other live runtime artifacts.
- If the intended instrument or patch selection requires persisted plugin configuration, store only the minimum portable form needed to restore that selection on a compatible machine.
- Reopen must resolve persisted slot intent against current machine-local inventory. Missing or unavailable plugins must degrade to explicit recoverable non-ready states instead of silent remap or hidden failure.
- Preserve exact-instance isolation across same-bundle slots. Rehydration or recovery must still target slot identity, not bundle identity alone.
- Treat VST3 plugins as untrusted code. Initialization, teardown, and recovery paths must remain defensive and must not rely on frontend timing or editor-host lifetime.
- Assess migration impact if persisted semantics change. Any unreleased migration belongs in `src-tauri/src/migrations/next.rs`.

### Architecture Compliance

- Rust remains the sole owner of playback inventory truth, slot/session truth, route truth, readiness truth, and persistence semantics.
- The rack-first model remains authoritative: racks own slot lifecycle, slots own plugin assignment or unassignment, and instruments own routes to already-existing slots.
- `PlaybackActivationRuntime` stays a derived readiness snapshot over live slot sessions rather than a second runtime registry.
- Score-context surfaces may present readiness and recovery, but they must not become a second playback topology or restoration engine.
- Same-session activation behavior and cross-session reopen behavior are related but distinct. This story may connect their owning seams, but it must not conflate them into a frontend-only shortcut.

### Library / Framework Requirements

- Keep backend mutations in existing Rust/Tauri seams with snake_case payloads.
- Prefer extending existing slot/session helpers in `plugin_editor_poc.rs`, `runtime_activation.rs`, `files.rs`, and score playback types instead of inventing parallel restoration flows.
- Keep frontend impact minimal and bounded to existing state consumers such as `resolved_playback_route_by_instrument_id` and existing rack or plugin surfaces.
- Do not add new routine UI whose primary purpose is to prove restoration literally. Existing score-context and maintainer-facing surfaces should remain sufficient.

### File Structure Requirements

- Primary owning backend seam: `src-tauri/src/playback/plugin_editor_poc.rs`
- Readiness and route-status seam: `src-tauri/src/playback/runtime_activation.rs`
- Persistence and load/save seam: `src-tauri/src/files.rs`
- Score playback data model seams: `src-tauri/src/score/playback.rs`, `src-tauri/src/score/instruments/mod.rs`
- App-level state refresh seams: `src-tauri/src/lib.rs`
- Migration seam if persisted semantics change: `src-tauri/src/migrations/next.rs`
- Likely frontend verification seams if payload shape changes: `src/services/actions.ts`, `src/services/types.ts`, `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`

### Testing Requirements

- Prefer seam-local Rust coverage over a new cross-cutting playback harness.
- Minimum expected Rust coverage:
  - assignment-time slot initialization creates a real slot-owned runtime session
  - reopen-time project restore rebuilds sessions from portable saved intent
  - missing or unavailable plugins degrade to explicit recoverable non-ready state
  - same-bundle multi-slot restoration keeps exact slot isolation
  - teardown on unassign, removal, reset, or project replacement leaves persistence portable and runtime truth clean
- Minimum expected validation commands when changes are in scope:
  - `cd src-tauri && cargo test plugin_editor_poc -- --nocapture`
  - `cd src-tauri && cargo test runtime_activation -- --nocapture`
  - `cd src-tauri && cargo test files::tests:: -- --nocapture`
  - `npm run lint`
  - `npm run check-types`
- If persisted semantics change, add or update round-trip coverage proving `.soloc` contains only portable intent and still restores correctly.

### Manual Verification Expectations

- Prepare one routed score instrument against a supported single-instrument VST plugin.
- Assign the plugin to a slot and confirm backend-owned readiness becomes visible without opening the native editor.
- Save the score, reopen the `.soloc` file, and confirm the slot-owned runtime session is restored or reinitialized from portable saved intent before editor-open.
- Confirm the restored plugin reflects the intended restorable instrument or patch state needed for the supported single-instrument workflow.
- Remove or hide the plugin from current machine-local availability, reopen the score, and confirm Compose surfaces a recoverable non-ready condition while leaving the score usable.
- Where two slots target the same bundle, verify each slot still restores as its own slot-owned instance rather than collapsing onto bundle-level identity.

### Implementation Risks And Guardrails

- Biggest risk: the persisted portable-intent payload may still be too thin to restore the intended instrument or patch selection. Prove sufficiency or extend it minimally.
- Do not reintroduce editor-open, bundle identity, or frontend state as canonical runtime ownership.
- Do not serialize live runtime objects to solve reopen.
- Do not silently remap a missing plugin to another available plugin unless an explicit product rule exists.
- Do not broaden this story into audible transport behavior, expression-map workflows, multi-target routing, or generalized plugin compatibility work.
- Be careful with main-thread initialization and teardown ordering. Slot-session creation already crosses main-thread host seams; do not introduce deadlocks or race-prone double creation.

### Relevant Code Seams

- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/runtime_activation.rs`
- `src-tauri/src/files.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/score/playback.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src-tauri/src/migrations/next.rs`
- `src/services/actions.ts`
- `src/services/types.ts`
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`

### Git Intelligence Summary

- Recent upstream sequence is coherent and narrow: `29a5f3cd feat(1.4): plugin stability` established the slot-owned runtime foundation and `9880a641 feat(1.5): transient plugin editors` hardened editor attach/detach on top of it.
- The recent sprint correction explicitly added Story 1.6 rather than reopening Story 1.3, which means this work should close the discovered initialization and reopen gap without disturbing the verified trust baseline.
- Recent implementation history favors backend seam hardening, focused tests, and documentation updates rather than new UX surfaces.

### Project Structure Notes

- Docs already state that non-persisted slot-owned live plugin sessions are keyed by `slot_id`, rebuilt from portable saved intent plus machine-local inventory truth, and torn down at ownership boundaries instead of editor-window close.
- `resolved_playback_route_by_instrument_id` is already the backend-owned route-status surface for the frontend. Preserve and extend that truth rather than inventing frontend recovery logic.
- The current testing guide already includes save/reopen proof in the single-instrument reset baseline. Story 1.6 should strengthen that proof with explicit restored-session expectations rather than changing the trust gate itself.

### Documentation Impact

Review and update these if implementation changes behavior, persistence shape, or maintainer proof expectations:

- `docs/architecture.md`
- `docs/project-context.md`
- `TESTING.md`
- `_bmad-output/planning-artifacts/architecture.md`
- `_bmad-output/planning-artifacts/epics.md`
- relevant AI-agent instruction files if persistence or playback rules change

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.6: Initialize And Restore Slot-Owned Sessions On Assignment And Reopen]
- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-03-25.md#Proposal E4: Add Story 1.6 to initialize and restore slot-owned sessions before UI-open]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming and Domain Language]
- [Source: _bmad-output/planning-artifacts/architecture.md#Failure Handling]
- [Source: _bmad-output/planning-artifacts/prd.md#Journey 4: Maintainer Validates The Playback Engine As Release-Credible Infrastructure]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#3.6 Native Plugin UI Escalation]
- [Source: docs/architecture.md]
- [Source: docs/project-context.md]
- [Source: TESTING.md]
- [Source: /memories/repo/slot-owned-live-runtime-sessions-story-1-4.md]
- [Source: /memories/repo/plugin-editor-host-session-lifecycle.md]

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story context generated from the current Epic 1 plan, sprint change proposal, architecture, PRD, UX specification, project context, prior Story 1.4 and 1.5 artifacts, repository memory, and recent git history.
- Confirmed the existing slot-owned runtime-session registry and `sync_slot_runtime_sessions(...)` seam already exist in `src-tauri/src/playback/plugin_editor_poc.rs`; Story 1.6 is about completing assignment/reopen restoration and persistence boundaries, not inventing a new ownership model.
- Confirmed current docs and testing guidance already describe slot-owned sessions as rebuilt from portable saved intent and already include save/reopen in the manual trust proof, so any implementation drift here must update both code and docs together.
- Confirmed sprint tracking still had Story 1.6 in `backlog` before this context artifact was created.
- Audited the owning runtime seams in `src-tauri/src/score/instruments/mod.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/files.rs`: assignment and load already refresh `PlaybackActivationRuntime` through `sync_slot_runtime_sessions(...)`, but the persisted score DTO did not yet carry any portable plugin-state restore blob.
- Rooted the restore approach in the prior host-comparator research: keep runtime/editor ownership instance-addressed, rebuild sessions from concrete slot identity plus saved intent before UI attach, and treat editor-open as attach-only over a prepared runtime session rather than the creator of canonical host state.
- Assessed migration impact for the new optional slot-target plugin-state payload and kept `src-tauri/src/migrations/next.rs` unchanged because older files remain loadable without transformation when the new field defaults to `None`.
- Added save-time capture of transferable VST3 component state from live slot sessions, restore-time application of that state during slot-session build, and sync invalidation when the saved slot state changes for the same `slot_id` and bundle path.

### Completion Notes List

- Story 1.6 context created with explicit focus on assignment-time initialization, reopen-time restoration, portable saved intent, and recoverable non-ready states.
- Prior-story learnings from Stories 1.4 and 1.5 were carried forward so the dev agent does not regress slot ownership or editor-host boundaries.
- Documentation impact and migration-assessment obligations are included directly in the story because persistence behavior is part of the acceptance criteria.
- Added `PlaybackSlotTarget.plugin_state` as the minimal portable restore payload, captured from live slot-owned VST sessions at save time and restored during runtime-session creation before editor-open.
- Kept `PlaybackActivationRuntime` as derived readiness metadata only; missing bundles or failed state restore continue to degrade cleanly to backend-owned `NeedsAttention` without dropping route intent.
- Added seam-local tests in `src-tauri/src/playback/plugin_editor_poc.rs`, `src-tauri/src/playback/runtime_activation.rs`, and `src-tauri/src/files.rs` for saved-state identity, desired-target propagation, and portable round-trip degradation behavior.
- Validation completed: `cargo test plugin_editor_poc -- --nocapture`, `cargo test runtime_activation -- --nocapture`, `cargo test files::tests:: -- --nocapture`, `npm run lint`, `npm run check-types`, `npm run test:rust`, and `npm run test:js`.

### File List

- \_bmad-output/implementation-artifacts/1-6-initialize-and-restore-slot-owned-sessions-on-assignment-and-reopen.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- \_bmad-output/planning-artifacts/architecture.md
- TESTING.md
- docs/architecture.md
- docs/project-context.md
- src-tauri/src/files.rs
- src-tauri/src/playback/plugin_editor_poc.rs
- src-tauri/src/playback/runtime_activation.rs
- src-tauri/src/score/instruments/mod.rs
- src-tauri/src/score/playback.rs
- src/services/types.ts

## Change Log

- 2026-03-25: Created Story 1.6 implementation context and moved sprint tracking to ready-for-dev.
- 2026-03-25: Implemented portable slot-target plugin-state capture/restore, updated readiness/persistence documentation, and validated the story with targeted plus full regression suites.
