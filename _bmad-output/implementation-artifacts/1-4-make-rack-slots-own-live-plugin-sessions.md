# Story 1.4: Make Rack Slots Own Live Plugin Sessions

Status: done

## Story

As a maintainer,
I want each routed playback slot to own one real initialized VST instance at runtime,
so that playback and plugin state no longer depend on editor-window lifetime while project files remain portable.

## Acceptance Criteria

1. **Given** a slot is assigned to an available plugin
   **When** the backend prepares or restores that slot
   **Then** Rust creates or restores a slot-owned runtime session keyed by `slot_id`
   **And** that session is the authoritative live plugin ownership seam for the slot

2. **Given** a slot-owned session exists
   **When** playback or score-context workflows query readiness
   **Then** they reflect that real runtime ownership instead of editor-created approximation
   **And** they do not treat editor-open as the act that creates canonical runtime state

3. **Given** a slot is unassigned, removed, or the project closes
   **When** ownership ends
   **Then** the slot-owned session is torn down at that ownership boundary rather than at editor close
   **And** the teardown leaves persisted user intent consistent

4. **Given** a project is saved
   **When** playback state is persisted
   **Then** `.soloc` stores only the portable plugin intent and the minimum data needed to rehydrate the slot-owned session later
   **And** it does not persist live host-session objects or machine-local runtime cache state

## Tasks / Subtasks

- [x] Replace singleton editor-host session ownership with slot-owned runtime session ownership in the backend playback domain (AC: 1, 2, 3)
  - [x] Define the authoritative slot-owned runtime-session registry under Rust-owned playback state, keyed by `slot_id`, and remove or demote any competing singleton/global ownership path.
  - [x] Ensure the slot-owned registry owns real initialized plugin sessions, not just readiness metadata or bundle references.
  - [x] Preserve exact-instance identity so same-bundle plugins in different slots cannot alias or overwrite each other.
- [x] Move canonical session creation and restore boundaries off editor-open and onto slot lifecycle seams (AC: 1, 2)
  - [x] Initialize or restore the slot-owned session when a slot is prepared from persisted intent, same-session activation, or project-load recovery.
  - [x] Keep native editor windows as attach/detach views over an existing slot-owned session instead of the creator of the canonical runtime instance.
  - [x] Keep same-session activation and cross-session reopen concerns separate; do not solve reopen by persisting live runtime state.
- [x] Tear down slot-owned sessions only at ownership boundaries and keep persistence portable (AC: 3, 4)
  - [x] Teardown sessions when a slot is unassigned, removed, or the project shuts down, not when the editor window closes.
  - [x] Preserve user intent in persisted score state while removing runtime-only host objects from any save path.
  - [x] Assess migration impact and update `src-tauri/src/migrations/next.rs` only if persisted semantics or file shape actually change.
- [x] Extend validation and maintainer proof around the new slot-owned lifecycle (AC: 1, 2, 3, 4)
  - [x] Add focused Rust coverage for slot-owned creation, detach/reattach, teardown, and same-bundle multi-slot isolation.
  - [x] Extend persistence and readiness checks so `.soloc` round trips prove portable rehydration intent without serializing runtime sessions.
  - [x] Record documentation changes in architecture and testing guidance so downstream stories treat slot-owned sessions as the required foundation.

### Review Findings

- [x] [Review][Patch] Concurrent attach/sync race can create a duplicate slot-owned runtime session while an editor is attaching [src-tauri/src/playback/plugin_editor_poc.rs:3660]
- [x] [Review][Patch] Architecture and planning docs now claim `.soloc` persists restorable plugin configuration, but the staged implementation does not add any corresponding persistence seam [docs/architecture.md:61]
- [x] [Review][Patch] Same-session editor reopen stabilization and host-view clipping follow-up are now in place, with the focused plugin editor host suite passing after the final host-container subview sync patch [src-tauri/src/playback/plugin_editor_poc.rs:806]

## Dev Notes

### Story Intent

Story 1.4 is the first follow-up to the Story 1.3 trust baseline. The purpose is not to add audible playback or broaden plugin compatibility. The purpose is to move canonical live plugin ownership to the rack slot itself so later playback, editor-host, and reopen work all build on one backend-owned runtime model.

This story is the architectural handoff from `PlaybackActivationRuntime` as a same-session readiness bridge to slot-owned runtime sessions as the real live-plugin seam. If this story leaves canonical ownership on editor-open, bundle identity, or a singleton host session, Epic 4 remains blocked.

### Story Foundation

- Epic 1 is a trust-restoration sequence inserted before audible playback resumes.
- Story 1.3 closed the baseline honestly and identified the missing seam precisely: slot-owned live plugin sessions rebuilt from portable saved intent.
- This story must keep the support claim narrow: one single-instrument VST instance per slot, one authoritative runtime session per slot, no multi-target or multi-timbral expansion.
- Native editor windows remain secondary handoff surfaces. They must attach to or detach from an already-owned slot session rather than creating the session.

### Previous Story Intelligence

- Story 1.3 already repaired the bounded native editor seam around a detached live session and exact-instance targeting by slot identity plus bundle path.
- Story 1.3 also proved that resize bugs, reopen hangs, and stale same-bundle targeting are lifecycle defects, not isolated UI bugs.
- Existing same-session runtime activation uses backend-owned `PlaybackActivationRuntime` as readiness metadata only. It must not become the long-lived live plugin-session registry.
- The repo has already clarified the rack-first model: racks own slot lifecycle, slots own plugin assignment or unassignment, and instruments only own routes to existing slots.
- Previous work shows the main failure seams to guard against: stale detached ownership, teardown ordering races, resize callback reentry, bundle-equal instance confusion, and leaked runtime truth into persistence.

### Technical Requirements

- Canonical runtime ownership must move to a Rust-owned slot registry keyed by `slot_id`; do not leave a singleton `ACTIVE_PLUGIN_HOST_SESSION` or equivalent global path as competing truth.
- A slot-owned runtime session must represent a real initialized VST instance, not just an inventory selection or derived readiness flag.
- Playback readiness, score-context status, and editor attach flows must all consult the slot-owned session seam rather than treating editor-open as the creation point of canonical runtime state.
- Session teardown must happen at slot unassign, slot removal, or project shutdown boundaries. Editor close may transition the editor attachment state, but it must not destroy the canonical slot-owned session by itself.
- `.soloc` must continue to persist only portable plugin intent and the minimum rehydration metadata needed to rebuild the slot-owned session later. Do not persist live host objects, controller/component pointers, machine-local caches, or editor-host runtime state.
- Missing or invalid plugins on restore must degrade to explicit recoverable states such as `NeedsAttention` or `Unavailable`; do not silently remap or fall back to sampler-era behavior.
- Keep same-session activation and cross-session reopen separate. Same-session creation or restore belongs in this story; reopen initialization sequencing remains broader follow-up work, but persistence boundaries must already support it.
- Exact-instance identity remains mandatory. Same-bundle plugins in different slots must stay distinct runtime objects, and detached ownership from one slot must not silently block or alias another.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory truth, rack state, slot state, route state, playback-preparation state, transport state, and persisted playback-domain truth.
- The active playback product path must stay truly VST-owned. Do not reintroduce sampler-era or SCVI-era fallback ownership once the slot-owned seam exists.
- The playback runtime remains split between a Rust control plane and a realtime-safe playback plane. Do not make the audio path depend on UI thread state, editor lifetime, or lock-heavy host interactions.
- Native plugin editor hosting remains a bounded backend-owned seam, secondary to Compose-owned playback UX.
- Editor attach, focus, resize, detach, and teardown are editor-host responsibilities, but that seam must not create or own the canonical slot runtime state.
- Score-context surfaces own routing, readiness, and recovery relationships; app-scoped windows own inventory and deep editing. Do not push canonical runtime logic into React contexts or window-local code.

### Library / Framework Requirements

- Keep all playback-domain mutations in Rust and expose them through existing Tauri command and event patterns with `snake_case` payloads.
- Keep playback runtime logic under `src-tauri/src/playback/` and persisted playback-domain logic under backend score/files/migration seams.
- Keep frontend state limited to Rust-owned snapshots, derived display payloads, and transient presentation state.
- Treat VST3 plugins as untrusted third-party code. Host-safety, teardown ordering, and failure containment remain hard requirements.

### File Structure Requirements

- Likely owning seams include `src-tauri/src/playback/plugin_editor_poc.rs`, `src-tauri/src/playback/runtime_activation.rs`, `src-tauri/src/score/playback.rs`, `src-tauri/src/files.rs`, and `src-tauri/src/migrations/next.rs`.
- If a dedicated slot-owned runtime-session module or type is introduced, keep it under `src-tauri/src/playback/` and make it the only canonical session owner.
- Do not place new playback-domain truth in React contexts, `src/services/` helpers, or score-context components.
- Keep documentation updates in `docs/architecture.md`, `docs/project-context.md`, `TESTING.md`, and active story/planning artifacts if behavior or expectations change.

### Testing Requirements

- Extend owning Rust playback tests rather than creating a parallel playback-lifecycle framework.
- Add focused coverage for slot-owned session creation on slot preparation or restore, exact-instance isolation across same-bundle slots, detach/reattach behavior, and teardown on slot unassign or removal.
- Add persistence round-trip coverage proving `.soloc` stores only portable intent and that slot-owned runtime state is rebuilt from that intent rather than serialized directly.
- Extend readiness or trust-gate checks so they reflect real slot-owned runtime ownership instead of editor-created approximation.
- Validate touched areas with repo-standard commands at minimum: `npm run lint`, `npm run check-types`, and focused Rust tests. Run `cargo clippy` when Rust changes materially and existing warnings do not prevent it.

### Manual Verification Expectations

- Prepare one score with a routed instrument whose slot points at a supported single-instrument VST plugin.
- Confirm the backend reports readiness through the slot-owned session without requiring the native editor window to be open.
- Open the native editor from the score-context rack, close it, and reopen it in the same session to verify the editor attaches to the preserved slot-owned session rather than creating a second canonical instance.
- Where two slots reference the same plugin bundle, verify each slot still targets the intended exact runtime instance and that stale detached ownership from one slot does not hijack or block the other.
- Save and reopen the project to confirm persisted slot intent survives while runtime sessions are rehydrated from portable saved intent rather than serialized live state.

### Implementation Risks And Guardrails

- The biggest risk is introducing two ownership models at once, such as a slot-owned registry plus a leftover singleton editor-host session. Story 1.4 should delete or demote the obsolete path, not preserve it as parallel truth.
- A second risk is solving this only for editor-open and not for playback readiness. The slot-owned seam must serve playback and score-context status even when no editor window exists.
- A third risk is accidentally broadening scope into full reopen orchestration, multi-target routing, or audible playback. Keep the claim limited to slot-owned live-session ownership and portable persistence boundaries.
- A fourth risk is persisting live runtime objects or machine-local metadata into `.soloc`. That would violate portability and require migration complexity for the wrong reason.
- A fifth risk is leaving teardown keyed to editor close. The acceptance criteria require teardown at slot ownership boundaries instead.
- A sixth risk is failing to preserve exact-instance identity across same-bundle slots. Bundle-level matching is insufficient once slot-owned runtime state exists.
- A seventh risk is conflating same-session activation with cross-session reopen. The architecture explicitly separates them; keep the seams distinct.

### Relevant Code Seams

- `src-tauri/src/score/playback.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src-tauri/src/playback/runtime_activation.rs`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/plugin_playback_poc.rs`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src-tauri/src/playback/plugin_host_compatibility.rs`
- `src-tauri/src/files.rs`
- `src-tauri/src/lib.rs`
- `src/services/actions.ts`
- `src/services/types.ts`
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`

### Documentation Impact

Minimum expected documentation and planning review targets:

- `docs/architecture.md`
- `docs/project-context.md`
- `TESTING.md`
- `_bmad-output/planning-artifacts/architecture.md`
- `_bmad-output/planning-artifacts/epics.md` if story wording or sequencing guidance needs refinement
- any AI-agent instruction files whose playback rules change because slot-owned sessions are now the required ownership seam

### Git Intelligence Summary

- Recent work is consistent with foundation hardening rather than feature expansion. The most relevant commit is `dbc55d7e feat(1.3): plugin readiness and course correction of arch`.
- Story 1.3 touched the exact downstream seams this story should build on: `plugin_editor_poc.rs`, `runtime_activation.rs`, `files.rs`, state payloads, rack-panel UI consumers, and architecture/testing documentation.
- The implementation pattern to preserve is backend-owned cleanup and seam clarification, not introducing a second exploratory runtime path.

### Project Structure Notes

- The rack-first model is already established: racks own slot lifecycle, slots own plugin assignment, and instruments own only routes.
- `PlaybackRackSlot.target` remains optional, so empty slots are valid persisted state and must surface as explicit backend-owned non-ready conditions.
- `PlaybackActivationRuntime` is readiness and targeting metadata, not the long-lived live plugin-session registry.
- The bounded playback proof seam in `plugin_playback_poc_probe` is transport-adjacent only; it must not become a parallel live playback state model.
- Existing editor-host lifecycle guidance already favors preserving a detached session for same-session reopen, buffering plugin-driven resize during reattach, and tearing down stale detached ownership before same-bundle retargeting.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.4: Make Rack Slots Own Live Plugin Sessions]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming and Domain Language]
- [Source: _bmad-output/planning-artifacts/architecture.md#Failure Handling]
- [Source: _bmad-output/planning-artifacts/prd.md#Journey 4: Maintainer Validates The Playback Engine As Release-Credible Infrastructure]
- [Source: _bmad-output/planning-artifacts/prd.md#Functional Requirements]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Core User Experience]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Playback State And Recovery]
- [Source: docs/architecture.md]
- [Source: docs/project-context.md]
- [Source: /memories/repo/playback-rack-first-model.md]
- [Source: /memories/repo/plugin-editor-host-session-lifecycle.md]
- [Source: /memories/repo/same-session-playback-activation-story-3-8.md]
- [Source: /memories/repo/rack-playback-proof-seam.md]

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story context generated from epics, architecture, PRD, UX, Story 1.3 artifact, repo memories, and recent git history.
- Implemented a slot-owned live-session registry in `plugin_editor_poc.rs`, derived `PlaybackActivationRuntime` from live slot targets instead of inventory presence alone, and moved app-driven readiness refresh through that slot registry.
- Updated reset, inventory-refresh, and load-test seams so slot-owned runtime sessions rebuild from portable saved intent and teardown occurs at ownership boundaries rather than editor close.
- Follow-up fix: detached editor target switching now preserves slot-owned runtime sessions and only clears the editor reservation before opening another same-bundle slot target, preventing slot-target switching from tearing down canonical slot-owned sessions.
- Validation run: `npm run lint`, `npm run check-types`, `cd /Users/aledjones/compose/src-tauri && cargo check --lib`, `cargo test runtime_activation -- --nocapture`, `cargo test plugin_editor_poc -- --nocapture`, `cargo test files::tests:: -- --nocapture`.
- Follow-up validation run: `cd /Users/aledjones/compose/src-tauri && cargo test release_reserved_plugin_target_clears_detached_slot_target_for_switch -- --nocapture`, `cargo test detached_slot_target_can_switch_to_same_bundle_different_slot_after_release -- --nocapture`, `cargo test plugin_editor_poc -- --nocapture`, `cargo check --lib`.
- Maintainer manual proof follow-up: same-bundle plugins assigned to two slots now retain distinct in-session configurations per slot, and reopening/switching the bounded host window no longer collapses the two slots onto one visible instance.
- Manual verification step 1 PASS: prepared one routed score instrument against a supported single-instrument VST plugin on the VST-only rack path with no issues noted.
- Manual verification step 2 PASS: backend-owned readiness was already visible for the routed instrument and slot before opening any native editor window, with no issues noted.
- Manual verification step 3 FAIL: reopening the native editor window in the same session hung the app; the window also appeared with incorrect sizing on reopen, but the causal relationship between the hang and sizing issue remains unclear.
- Manual verification step 3 RETRY PASS: on a later retry during step-4 validation, reopening the native editor window in the same session succeeded; the earlier hang and incorrect sizing behavior appear intermittent rather than deterministic.
- Manual verification step 4 PASS: same-bundle slot switching succeeded across multiple open/close cycles; different slot targets reopened correctly without hijacking or blocking each other.
- Manual verification step 5 PASS: saving and reopening the project preserved slot and route intent, and runtime readiness was re-established from portable saved intent without requiring persisted live host-session state.
- Manual verification teardown expectation PASS: removing or unassigning the slot dropped readiness at the ownership boundary as expected.
- Intermittent reopen investigation follow-up: the bounded host close path was queuing native close preparation even when `CloseRequested` was already running on the main thread, which left a race where `Destroyed` could observe `close_prepared = false` and tear down the slot-owned session instead of detaching it for same-session reopen.
- Clipping follow-up 12: the host container now forces any plugin-owned NSView subviews to match the container bounds on attach and on later host-frame syncs, keeping the native child view aligned without reintroducing the reverted `view.onSize(...)` acknowledgment path.
- Final focused validation rerun PASS: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture` completed with `41 passed; 0 failed` after the host-container subview sync patch.
- Follow-up patch: `prepare_active_plugin_host_session_for_close` now performs close preparation inline when already on the main thread and only falls back to scheduled dispatch when it is not, so the detach path is armed before `Destroyed` cleanup runs.
- Follow-up validation run: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture`.
- Intermittent reopen investigation follow-up 2: latest manual evidence suggests incorrect resize is more likely a symptom than the cause; the stronger remaining hypothesis is that reopen occasionally stops making progress at a blocking wait or incomplete-session handoff during the main-thread attach/detach cycle.
- Follow-up instrumentation: added wait-start/wait-finish timing logs around the plugin host's main-thread handoff waits (`attach_native_host_surface`, deferred host resize dispatch, detached-session shutdown, slot-runtime creation, and slot-runtime teardown) so the next repro shows which wait, if any, stops completing.
- Follow-up validation run 2: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture`.
- Intermittent reopen investigation follow-up 3: the captured BBC reopen logs show the reopen attach path does complete, but once attached the host can enter a non-resizable resize feedback loop: plugin-driven `resizeView` queues a deferred host resize, the resulting `WindowResized` event schedules a non-resizable attach-layout sync, and that deferred `view.onSize` echo triggers another plugin-driven resize.
- Follow-up patch 3: non-resizable attach-layout follow-up sync is now only armed during `Reserved` or `Detached` negotiation, not once the session is already `Attached`, so plugin-driven host resize still works but the attached-session echo loop no longer replays `view.onSize` back into the plugin after every host resize.
- Follow-up validation run 3: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture`.
- Intermittent reopen investigation follow-up 4: the next BBC trace shows the reopen path and deferred host resize both complete without a stuck `wait-start`; if a hang remains, it is now more likely after `setContentSize(...)` returns, during the next main-thread event-loop turn rather than in the explicit attach/detach handoff waits.
- Follow-up instrumentation 4: after each successful deferred host resize, the host now schedules one extra `post-resize main-thread tick` marker so the next repro can confirm whether the main thread continues draining callbacks after the resize completion log.
- Follow-up validation run 4: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture`.
- Intermittent reopen investigation follow-up 5: the next trace produced the `post-resize main-thread tick`, so the main thread is still alive immediately after deferred resize completion; the remaining question is whether the native host window actually settles to the requested size shortly afterward or stops progressing between the immediate tick and the subsequent resize/event delivery.
- Follow-up instrumentation 5: after each successful deferred host resize, the host now schedules a delayed post-resize sample that logs the requested size and the actual `contentView` size about 50ms later.
- Follow-up validation run 5: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture`.
- Intermittent reopen investigation follow-up 6: the next BBC trace showed neither the immediate nor delayed post-resize follow-up after `deferred host resize applied`, which points to the buffered plugin-driven resize being dispatched too early from inside the reattach transaction rather than the main thread being globally deadlocked.
- Follow-up patch 6: buffered plugin-driven resize dispatch now happens only after `attach_native_host_surface` has fully returned its successful outcome, instead of being kicked off from inside the main-thread attach closure.
- Follow-up validation run 6: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture`.
- Intermittent reopen investigation follow-up 7: the newest BBC trace still stops immediately after `deferred host resize applied`, so the next uncertainty is whether execution reaches the post-resize probe scheduling at all or whether `run_on_main_thread(...)` is now failing silently for those follow-up markers.
- Follow-up patch 7: post-resize probe scheduling now logs explicit scheduling failures, the delayed sample uses a plain thread instead of blocking an async-runtime worker, and `resizeView` now reads `plugin_can_resize` from the live slot-owned session path instead of only the legacy singleton active-session store.
- Follow-up validation run 7: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture`.
- Intermittent reopen investigation follow-up 8: the next BBC trace proved the post-resize probes are being scheduled, but the main thread still does not execute them after the immediate buffered reopen resize starts, which points back at the buffered reattach resize transaction itself rather than the probe callbacks.
- Follow-up patch 8: buffered plugin-driven resize after same-session reattach is now delayed by 50ms before dispatch so the reopened native window can finish its immediate attach/focus transaction before `setContentSize(...)` runs.
- Follow-up validation run 8: `cd /Users/aledjones/compose/src-tauri && cargo test plugin_editor_poc -- --nocapture`.
- Intermittent reopen investigation follow-up 9: the 50ms delayed buffered reattach resize stabilizes BBC same-session reopen, which suggests the reopened native host window needs one short post-attach settling interval before the first buffered plugin-driven `setContentSize(...)` call.
- Follow-up patch 9: the host window is now created hidden, `attach_native_host_surface` flushes any buffered reattach resize synchronously before it returns, and `plugin_editor_poc_open` only shows and focuses the window after that readiness boundary passes so the stabilization delay is no longer user-visible.
- Follow-up validation run 9: backend file diagnostics for `src-tauri/src/playback/plugin_editor_poc.rs` are clean; focused Rust rebuild remains environment-blocked until disk space is available.
- Intermittent reopen investigation follow-up 10: later plugin-driven size growth exposed a separate surface issue: the NSWindow content size changed, but the embedded native host container could still remain at the older bounds until a later resize sync, leaving the plugin UI clipped at its pre-growth extent.
- Follow-up patch 10: plugin-driven deferred host resize now immediately syncs the embedded host-container frame to the current `contentView` bounds in the same main-thread `setContentSize(...)` transaction instead of waiting for a later `WindowResized` pass.
- Follow-up validation run 10: backend file diagnostics for `src-tauri/src/playback/plugin_editor_poc.rs` are clean; focused Rust rebuild remains environment-blocked until disk space is available.
- Intermittent reopen investigation follow-up 11: clipping still persisted after the container-frame sync, which suggests the attached plugin view itself was not being told to relayout at the new bounds during plugin-driven growth and was remaining at its older frame.
- Follow-up patch 11: after each plugin-driven deferred host resize, the host now calls `view.onSize(...)` with the current host bounds to notify the attached plugin view of the new frame, and `resizeView` now ignores duplicate same-size requests when the host window already matches the requested bounds so that acknowledgment does not requeue the same resize indefinitely.
- Follow-up validation run 11: backend file diagnostics for `src-tauri/src/playback/plugin_editor_poc.rs` are clean; focused Rust rebuild remains environment-blocked until disk space is available.
- Follow-up revert 11: the explicit `view.onSize(...)` acknowledgment and duplicate same-size guard reintroduced the same-session hang, so that patch has been backed out; the host-container frame sync from follow-up 10 remains in place while the clipping seam is investigated through a lower-risk native-child-view path instead.
- `cargo clippy --lib -- -D warnings` remains blocked by pre-existing dead-code warnings in untouched files `src-tauri/src/playback/audio.rs` and `src-tauri/src/playback/mixer.rs`.

### Completion Notes List

- Added a Rust-owned slot runtime-session synchronizer that prepares one live initialized VST session per prepared slot, keyed by `slot_id`, and tears those sessions down on unassign, slot removal, reset, inventory invalidation, or project replacement.
- Changed editor-host open/close behavior so slot-target editor windows attach to and detach from an existing slot-owned session instead of creating or destroying the canonical runtime instance.
- Corrected detached same-session editor retargeting so switching the single host window between same-bundle slot targets clears only the editor reservation and does not tear down the previously detached slot-owned runtime session.
- Manual verification now confirms same-bundle slot targets can hold distinct in-session plugin configurations across close/reopen of the bounded host window.
- Rebased `PlaybackActivationRuntime` on live slot targets, updated file-load test support to use the live-target model, and added readiness tests for live-session gating plus same-bundle multi-slot isolation.
- Updated architecture and testing guidance so downstream work treats slot-owned sessions as the required playback foundation.

### File List

- \_bmad-output/implementation-artifacts/1-4-make-rack-slots-own-live-plugin-sessions.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- TESTING.md
- docs/architecture.md
- docs/project-context.md
- src-tauri/src/files.rs
- src-tauri/src/lib.rs
- src-tauri/src/playback/plugin_editor_poc.rs
- src-tauri/src/playback/plugin_inventory.rs
- src-tauri/src/playback/runtime_activation.rs

## Change Log

- 2026-03-25: Implemented slot-owned live plugin sessions, rebased readiness on live slot targets, updated persistence/readiness tests, and documented the slot-owned lifecycle foundation.
- 2026-03-25: Fixed detached same-bundle slot switching so editor retargeting preserves slot-owned runtime sessions instead of tearing them down during the target change.
