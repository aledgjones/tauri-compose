# Story 9.4: Continue Adding Real Host Abilities Until HALION Works In The Product Path

Status: in-progress

## Story

As a maintainer,
I want Compose to keep adding real host-owned abilities on the shared VST3 host seam until HALION works through the ordinary product path,
so that HALION progress comes from truthful backend-owned contract improvements rather than maintainer-only reruns or fake support behavior.

## Acceptance Criteria

1. Given prior HALION investigation relied on a VST-specific debugger-window probe workflow, when Story 9.4 begins, then Compose retires that debugger-window functionality, removes the matching backend probe path, and moves maintainer evidence onto backend-owned phase logging inside the ordinary direct-root editor-host flow and shared host core so the real product path becomes the only implementation target.
2. Given the current hardcoded blocked-bundle policy prevents ordinary product-path iteration on HALion Sonic, when Story 9.4 lands the smallest truthful host-core change set needed for a bounded ordinary attach, then Compose removes or narrows the HALion Sonic policy block as early as that proof allows so maintainers can iterate through real slot mounting and ordinary editor-host opens on the direct-root path, and Compose can mount the proven HALION bundle on an instrument slot without relying on maintainer-only fake-success behavior.
3. Given HALION still needs additional host behavior after early unblocking, when Story 9.4 adds more abilities, then every new host ability grows the shared backend-owned seam and is justified by official Steinberg public-contract evidence plus open-source prior art, using MuseScore as the primary comparator and JUCE, Ardour, and Carla as secondary comparators, rather than guessed private-GUID success paths, wrapper aliases, opaque stubs, or fake-success returns.
4. Given at least one currently working control bundle already proves the single-instrument baseline, when Story 9.4 lands, then that control bundle continues to validate the proven single-instrument path on the same shared seam, HALION-specific work does not regress ordinary bounded editor-host behavior for already-proven bundles, and route-choice, `PluginPart`, `MidiChannel`, shared-slot, broader multi-instrument UX, and HALion Sonic SE claims remain out of scope.

## Tasks / Subtasks

- [x] Task 1: Retire the VST-specific debugger workflow and remove the matching backend probe path before further HALION remediation work proceeds (AC: 1)
  - [x] Remove the HALION/VST maintainer load/init probe controls from `src/windows/debugger/index.tsx` and `src/windows/debugger/index.test.tsx`, and remove any matching frontend action/type plumbing in `src/services/actions.ts`, `src/services/actions.test.ts`, and `src/services/types.ts` that only exists to feed that debugger surface.
  - [x] Remove the dedicated backend probe commands and registrations from `src-tauri/src/playback/plugin_editor_poc.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/automation/registry.rs` so the repo no longer preserves a separate HALION probe lane.
  - [x] Stop treating the Debugger window as the preferred VST host workflow in `docs/development-guide.md`, and replace probe-oriented guidance with real-path-first logging guidance.
  - [x] Keep non-VST debugger capabilities unchanged. This task retires the VST-specific debugger workflow; it does not delete the general debugger window used by other subsystems.

- [ ] Task 2: Unblock HALion Sonic on the ordinary bounded path as soon as the minimum real host-core change set makes that safe enough for real iteration (AC: 2, 3)
  - [ ] Replace the current hardcoded ordinary HALion Sonic policy block in `src-tauri/src/playback/plugin_host_compatibility.rs` with narrower, evidence-driven gating once the shared seam reaches bounded ordinary-path attach proof.
  - [ ] Ensure `src-tauri/src/playback/plugin_editor_poc.rs` can open HALion Sonic through the ordinary direct-root editor-host path without using maintainer-only bypasses, and prove the same bundle can be mounted on an instrument slot through the real product path.
  - [ ] Keep HALion Sonic SE and the other currently blocked bundles unchanged unless this story produces equivalent bounded ordinary-path proof for them.

- [ ] Task 3: Add structured logging to the real host flow and build out additional host abilities iteratively from ordinary-path evidence and public prior art (AC: 1, 2, 3)
  - [x] Restore structured Rust-terminal logging on the ordinary path in `src-tauri/src/playback/plugin_editor_poc.rs` so policy-gate entry, shared host-core phase transitions during session build, native view attach phases, outer attach failures, and active-phase host callback clues all emit to stderr even if the app aborts before a UI error can render.
  - [ ] Start from `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` as the canonical lifecycle owner and keep new lifecycle or service work on that shared seam rather than reintroducing a probe-only owner.
  - [ ] Surface the existing host-core phase logging through the ordinary direct-root open path in `src-tauri/src/playback/plugin_editor_poc.rs` so module load, host context, component and controller initialization, connection-point setup, view preparation, and teardown are logged from the real path rather than a parallel probe path.
  - [ ] Use the post-Story-9.3 `controller_initialize` evidence and subsequent ordinary-path failures to identify the next missing public-contract behaviors, then implement the smallest truthful host additions needed to move HALION forward.
  - [ ] Root those additions in official Steinberg host-contract requirements and MuseScore's real host patterns for host context, component-handler wiring, state sync, and view attachment before falling back to broader comparator-host evidence from JUCE, Ardour, or Carla.
  - [ ] Prefer real host lifecycle and service objects over guessed wrapper aliases, opaque stubs, or fake-success branches.

- [ ] Task 4: Preserve control-bundle and bounded-scope regressions while HALION advances (AC: 4)
  - [ ] Keep one currently working control bundle such as `ProEQ.vst3` or `Reverb.vst3` in the ordinary-path validation matrix as a regression guard.
  - [ ] Preserve blocked posture for Groove Agent SE, Kontakt 7, MT-PowerDrumKit, and any other still-unproven bundle unless this story explicitly proves and updates them.
  - [ ] Keep `src-tauri/src/score/instruments/mod.rs` on direct-root-only route emission and preserve rejection of non-root `PluginPart` and `MidiChannel` routes.
  - [ ] Keep unsupported non-root preparation and runtime activation behavior unchanged in `src-tauri/src/playback/preparation.rs` and `src-tauri/src/playback/runtime_activation.rs`.
  - [ ] Keep machine-local inventory truth honest: discovery may continue to show bundles as available, but ordinary validation payloads must match the actual current bounded product path.

- [ ] Task 5: Update seam-local tests, maintainer evidence, and support artifacts around the real HALION path (AC: 1, 2, 3, 4)
  - [ ] Add or update seam-local Rust tests for the narrowed block-list behavior, ordinary HALION path gating, shared host-core usage, real-flow phase logging, and preserved direct-root-only route behavior.
  - [ ] Add or update JavaScript tests for retirement of the VST debugger surface and any remaining frontend invoke affordances that change with it.
  - [ ] Use ordinary slot mounting, ordinary editor open, and backend-owned real-flow logs as the primary validation loop after early unblock.
  - [ ] Update `docs/architecture.md`, `docs/development-guide.md`, `docs/playback-support-baseline.md`, and the relevant BMAD tracking artifacts when the ordinary product-path evidence changes.

## Dev Notes

### Story Intent

- Story 9.4 is no longer an evidence-only rerun story. It is the next real implementation slice after Story 9.3.
- The strong steering is deliberate and ordered:
  1. retire the VST-specific debugger-window probe workflow;
  2. remove the matching backend probe path rather than preserving it in reduced form;
  3. do the minimum real host work needed to unblock HALion Sonic on the ordinary direct-root path;
  4. keep growing the shared host seam only as real product-path testing plus real-flow logging proves the next public-contract gap.
- HALION progress should come from real slot mounting and ordinary editor-host opens as early as possible, not from debugger-driven probe iteration.

### This Story Owns

- Retiring the VST-specific debugger-window functionality and debugger-first workflow for HALION/VST host investigation.
- Removing the separate HALION probe-path commands instead of preserving them as a shadow workflow.
- Continuing the real host-owned remediation work on `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` and adjacent owner files.
- Narrowing or removing the ordinary HALion Sonic block as soon as bounded ordinary-path proof exists.
- Making HALion Sonic testable and iterable through real slot mounting and ordinary editor-host opens.
- Adding backend-owned phase logging to the real ordinary path so diagnostics follow the actual host lifecycle.
- Keeping one known-good control bundle in the validation matrix and preserving direct-root-only route scope.

### This Story Does Not Own

- Deleting the general debugger window used by renderer, performance, and other non-VST maintainer tooling.
- Broad multi-instrument, multi-part, or shared-slot product support.
- New `PluginPart` or `MidiChannel` route-choice emission.
- General removal of the blocked-bundle posture for all currently blocked plugins.
- Claiming HALion Sonic SE or any other blocked bundle works unless this story explicitly proves it.

### Product-Path Steering

- Interpret “remove the debugger window functionality” narrowly: retire the HALION/VST maintainer probe controls and the matching backend probe path. Do not delete the general debugger window used by other subsystems.
- Prefer early real-path unblocking for HALion Sonic once the smallest truthful host-core change set yields bounded ordinary direct-root attach proof. The goal is to shift the main validation loop onto real slot mounting and ordinary editor-host opens as soon as possible.
- Add logging to the real flow instead of preserving a separate probe lane. The ordinary direct-root attach path should emit the lifecycle evidence needed to debug the next missing host capability.
- After early unblock, add more public-contract behavior only when real ordinary-path testing and real-flow logging show the next missing host capability. Do not front-load speculative VST surface area just because the SDK exposes it.
- Primary prior-art order for this story: official Steinberg interface contract first; MuseScore's VST host architecture second; JUCE, Ardour, and Carla as secondary comparators for lifecycle ordering and teardown.
- “Early unblocking” does not mean: widen route choices, imply multi-instrument readiness, or remove policy gates for unrelated crash-prone bundles.

### Current Relevant Seams

- `src/windows/debugger/index.tsx`
  - Current VST-specific maintainer probe surface to retire without disturbing non-VST debugger panels.
- `src/services/actions.ts` and `src/services/types.ts`
  - Frontend invoke surface and shared types for the debugger-driven probe flow if that VST path is removed.
- `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs`
  - Canonical lifecycle owner for factory host context, component and controller initialization, component-handler timing, state sync, connection-point wiring, view preparation, and teardown.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Ordinary editor-host entry point, target resolution, product-path policy gate, real-flow logging owner, and validation emission.
- `src-tauri/src/lib.rs` and `src-tauri/src/automation/registry.rs`
  - Command and automation cleanup points when the backend probe path is removed.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Current hardcoded blocked-bundle policy, including HALion Sonic.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Machine-local inventory state and `editor_host_validation` payload ownership.
- `src-tauri/src/score/instruments/mod.rs`
  - Direct-root-only route-choice emission and route assignment validation for instrument slots.
- `src-tauri/src/playback/preparation.rs`
  - Unsupported advanced-route preparation seam that must remain unchanged.
- `src-tauri/src/playback/runtime_activation.rs`
  - Unsupported advanced-route runtime activation seam that must remain unchanged.

### Previous Story Intelligence

- Story 9.2 established `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` as the canonical lifecycle seam. Do not move lifecycle truth back into parallel probe-only branches.
- Story 9.3 removed probe-era fake-success behavior and proved that `ProEQ.vst3` still reaches `load_init_safe` while HALion Sonic still crashes during `controller_initialize` on both profiles.
- The next steering step is to stop centering HALION progress on any probe workflow at all and move the main iteration loop plus lifecycle logging onto the real ordinary product path as soon as bounded attach proof exists.
- The recent Epic 9 branch direction is explicit in git history: `feat(9.2): abandon probe direction`. Keep the code and story language aligned with that turn.

### Architecture Compliance

- Backend remains the sole owner of host lifecycle truth, blocked posture, ordinary validation, and maintainer evidence.
- React remains a projection surface only. HALION progress must come from backend-owned host and policy seams.
- Keep the acceptance claim limited to bounded single-instrument direct-root behavior even if HALION begins working through the ordinary path.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.5, TypeScript 6.0.2.
- Do not add new dependencies unless a narrow real host-owned seam need makes them unavoidable.

### Testing Requirements

- Add or update JavaScript tests for:
  - retirement of any VST-specific debugger-facing frontend affordances that still claim a debugger-first workflow.
- Add or update Rust tests for:
  - ordinary-path phase logging emitted from the shared host core and surfaced through the real flow;
  - narrowed HALion Sonic ordinary-path gating in `src-tauri/src/playback/plugin_host_compatibility.rs`;
  - preserved blocked posture for unrelated bundles;
  - ordinary HALION editor-host opens consuming the shared host core rather than a maintainer-only path;
  - preserved direct-root-only route-choice and non-root rejection in `src-tauri/src/score/instruments/mod.rs`;
  - preserved unsupported non-root preparation and runtime activation behavior.
- Recommended validation commands after implementation:
  - `npm run test:js -- src/windows/debugger/index.test.tsx src/services/actions.test.ts`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_host_compatibility --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml runtime_activation --lib -- --nocapture`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
  - `npm run check-types`
  - `npm run lint`
- Manual or maintainer validation should include:
  - one known-good control bundle on the ordinary path;
  - HALion Sonic through ordinary slot mounting and editor open as soon as the policy gate is narrowed;
  - inspection of backend-owned real-flow lifecycle logs when HALion still fails on the ordinary path;
  - confirmation that non-root route behavior and unrelated blocked bundles remain unchanged.

### Project Structure Notes

- Keep the general debugger window for renderer and performance diagnostics, but remove HALION/VST host iteration from `src/windows/debugger/index.tsx` once the real product path is ready to take over.
- Keep ordinary editor-host policy in `src-tauri/src/playback/plugin_host_compatibility.rs` and product-path orchestration in `src-tauri/src/playback/plugin_editor_poc.rs`.
- Keep route-choice and slot-mount logic in `src-tauri/src/score/instruments/mod.rs`; do not invent a second route-authoring surface for HALION.
- Keep maintainer evidence backend-owned in inventory, ordinary-path validation, and real-flow logging seams instead of introducing frontend-owned truth or a second diagnostics lane.

### References

- `_bmad-output/implementation-artifacts/tech-spec-halion-product-path-remediation.md`
  - Dedicated implementation steering for debugger retirement, early unblocking, and prior-art ordering.
- `_bmad-output/planning-artifacts/epics.md`
  - Story 9.4 title and acceptance criteria.
- `_bmad-output/planning-artifacts/sprint-change-proposal-2026-04-12.md`
  - Reframed 9.4 as the HALION product-path remediation step.
- `_bmad-output/implementation-artifacts/9-3-replace-probe-era-shims-with-a-real-host-owned-init-contract.md`
  - Post-Story-9.3 learnings and current failure boundary.
- `_bmad-output/implementation-artifacts/9-2-extract-a-stable-backend-owned-vst3-host-core-from-the-poc.md`
  - Host-core seam ownership and prior-art grounding.
- `_bmad-output/implementation-artifacts/tech-spec-halion-host-contract-hardening.md`
  - Real host-contract hardening guidance that 9.4 now extends into product-path remediation.
- `docs/project-context.md`
  - Backend-owned state and route-boundary guardrails.
- `docs/architecture.md`
  - Shared host-core seam and bounded native-editor handoff.
- `docs/development-guide.md`
  - Maintainer probe workflow and current controller-init profile guidance.
- `docs/playback-support-baseline.md`
  - Current HALion Sonic ordinary-path investigation posture and control-bundle evidence.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Current HALion Sonic ordinary block list.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - `ensure_editor_host_target_supported(...)` and ordinary editor-open path.
- `src-tauri/src/score/instruments/mod.rs`
  - Direct-root-only route-choice and slot-route validation.
- `/memories/repo/multi-instrument-vst-halion-spec.md`
  - Epic 9 planning guardrails and HALION-specific investigation history.
- `/memories/repo/plugin-editor-poc-live-vst3-host.md`
  - Shared seam status and post-Story-9.3 rerun findings.
- `/memories/repo/vst3-host-lifecycle-sanity-check.md`
  - Cross-host lifecycle guardrails relevant to the ordinary product path.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-12: Created the Story 9.4 implementation artifact after rebaselining Epic 9 so 9.4 now owns continued real host-owned remediation toward the ordinary HALION product path.
- 2026-04-12: Refined the story to retire the VST-specific debugger-window workflow and matching backend probe path, move diagnostics onto real-flow logging, unblock HALION on the real product path early, and ground further host work in official Steinberg guidance plus MuseScore-led prior art.
- 2026-04-12: Switched the active maintainer rerun loop from direct editor-open-first to realistic rack-slot mounting through MCP (`compose_action_score_playback_slot_create` plus `compose_action_score_playback_slot_assignPlugin`) and captured the same crash shape on the slot-owned session-build path.
- 2026-04-13: Confirmed the Steinberg wrapper-marker interfaces are public but wrapper-only contracts, so Compose keeps them unsupported; then ran a bounded composite init-host comparison that widened `PluginHostApplication` to publish truthful public `IComponentHandler` and `IComponentHandler2` during `controller.initialize()`, logged `IPluginFactory3::setHostContext(...)`, passed `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` with 60/60 tests, and reran ordinary `plugin.editorHost.open` against `HALion Sonic.vst3`. The real rerun still crashed during `controller_initialize`, logged `set_host_context_tresult=3`, never showed an init-host `IComponentHandler*` query, and produced crash report `solo-compose-2026-04-13-115029.ips` with `EXC_BAD_ACCESS` at `0x40` inside HALion Sonic during `IPluginBaseTrait::initialize`.
- 2026-04-13: Followed that comparison by restoring Steinberg-style `IPlugInterfaceSupport` semantics to the plug-side interface list and routing component/controller peer hookup through host-owned `ConnectionProxy` objects, then reread the fresh ordinary-path trace and crash report `solo-compose-2026-04-13-122239.ips`. The new rerun still died during `controller_initialize`, still logged only wrapper-marker or unknown interface probes plus `IHostApplication::getName`, still never showed any `IPlugInterfaceSupport` or init-host `IComponentHandler*` query, and repeated the same `EXC_BAD_ACCESS` at `0x40` inside HALion Sonic while Rust remained in `IPluginBaseTrait::initialize`.
- 2026-04-13: Recreated Steinberg `PlugProvider` and MuseScore's post-provider handler/state order more literally on the shared Rust host core by collapsing the lifecycle onto one provider-style setup path, deferring persisted component-state restoration until after provider initialization, and mirroring restored state into separate controllers through `setComponentState(...)`. Focused validation still passed with `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` at 55/55, but the next ordinary `plugin.editorHost.open` rerun against `HALion Sonic.vst3` still crashed during `controller_initialize`, and fresh crash report `solo-compose-2026-04-13-124708.ips` repeated the same `EXC_BAD_ACCESS` / `KERN_INVALID_ADDRESS at 0x40` boundary while Rust remained inside `IPluginBaseTrait::initialize` from `initialize_steinberg_plug_provider(...)`.
- 2026-04-13: Read follow-up crash report `solo-compose-2026-04-13-125859.ips` after the faithful provider-order translation and confirmed it still repeated the same `EXC_BAD_ACCESS` / `KERN_INVALID_ADDRESS at 0x40` boundary while Rust remained inside `IPluginBaseTrait::initialize` from `initialize_steinberg_plug_provider(...)`. That ruled out the possibility that the prior-art provider translation had already moved the crash window without the live trace showing it.
- 2026-04-13: Compared Compose's remaining `IHostApplication` success semantics against Steinberg `HostApplication`, then corrected the live callback lane so `IHostApplication::getName(...)` and successful `createInstance(...)` results for `IMessage` and `IAttributeList` now return Steinberg-style `kResultTrue`. Focused validation still passed with `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` at 55/55, but the next ordinary `plugin.editorHost.open` rerun still logged `IHostApplication::getName returned true`, repeated the second wrapper-marker query round, and produced crash report `solo-compose-2026-04-13-130949.ips` with the same `EXC_BAD_ACCESS` / `KERN_INVALID_ADDRESS at 0x40` boundary while Rust remained inside `IPluginBaseTrait::initialize` from `initialize_steinberg_plug_provider(...)`.
- 2026-04-13: Reproduced the realistic rack-slot mount again after a user-reported Sequence hang by driving `reset`, `score.playback.slot.create`, and `score.playback.slot.assignPlugin` through the automation API. The slot-assign request dropped with `curl: (52) Empty reply from server`, fresh crash report `solo-compose-2026-04-13-132518.ips` still showed the same main-thread `EXC_BAD_ACCESS` / `KERN_INVALID_ADDRESS at 0x40` boundary while Rust remained inside `IPluginBaseTrait::initialize` from `initialize_steinberg_plug_provider(...)`, and the follow-up JavaScript mitigation bounded crash-prone plugin-host `invoke()` calls so the rack/editor UI can surface a timeout instead of leaving local loading state pending when the backend stalls or dies.
- 2026-04-13: Restored the live shared seam from host-owned `ConnectionProxy` peers back to direct symmetric component/controller `IConnectionPoint` hookup so the next HALION rerun would sit on the simplest faithful ordinary-path baseline again. Focused validation still passed with `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` at 57/57, a fresh real slot-path control rerun kept `ProEQ.vst3` healthy, and HALion Sonic still crashed on the same provider path. Fresh crash report `solo-compose-2026-04-13-163533.ips` repeated the same main-thread `EXC_BAD_ACCESS` / `KERN_INVALID_ADDRESS at 0x40` boundary while Rust remained inside `IPluginBaseTrait::initialize` from `initialize_steinberg_plug_provider(...)`, so connection-proxy ownership is ruled out and the restored direct handshake is the current trusted baseline.
- 2026-04-13: Aligned unsupported host `FUnknown::queryInterface` responses with Steinberg `kNoInterface` instead of `kResultFalse`, kept focused validation green at 57/57, and confirmed `ProEQ.vst3` still opened cleanly on the ordinary bounded path. The first HALION rerun after that ABI cleanup reused an already-open native host window and produced crash report `solo-compose-2026-04-13-165657.ips`, but that report showed a Compose-side AppKit thread violation in `focus_native_host_window` on a `tokio-rt-worker`, not a moved HALION boundary.
- 2026-04-13: Fixed the existing-window fast path so native host-window focus and close now cross onto `run_on_main_thread`, then validated that fix by opening `ProEQ.vst3` twice through `plugin.editorHost.open` with the second call returning normally while automation health stayed up. A fresh HALION rerun from `existing_window=false` then returned to the trusted ordinary-path failure family: clean policy-gate pass, faithful provider-path session build, the same unsupported wrapper-marker query bursts plus successful `IHostApplication::getName`, and backend death before `controller.initialize(...)` returned. No fresh `.ips` had been emitted at capture time, so treat the clean terminal trace plus `solo-compose-2026-04-13-163533.ips` as the current unchanged-boundary evidence.
- 2026-04-13: Tackled the remaining composite-host delta directly on the shared ordinary seam. `PluginHostApplication` now owns and exposes the same truthful `IComponentHandler` and `IComponentHandler2` object during init-time `queryInterface`, and the shared host core now reuses that same handler object for the later `setComponentHandler(...)` call instead of allocating a second unrelated callback instance. Focused Rust validation still passed with `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` at 58/58.
- 2026-04-13: Tightened that composite-host translation further so the live ordinary seam no longer simulates composition by delegating handler-family `queryInterface` calls into a separate helper object. `PluginHostApplication` now exposes `IHostApplication`, `IComponentHandler`, and `IComponentHandler2` from one COM object with shared `queryInterface`, `addRef`, and `release`, while `IPlugInterfaceSupport` remains a separate truthful nested capability object. Focused Rust validation still passed with `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` at 58/58.
- 2026-04-13: Added diagnostics-only `PluginHostApplication` telemetry on the live shared seam so full `queryInterface` results now log requested IID, final `tresult`, returned object pointer, and host refcount transitions. Focused validation still passed with `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` at 57/57.
- 2026-04-13 20:34 BST: Proved the real rack-slot repro no longer depends on loading a `.soloc` file first. Starting from the blank in-memory shell (`file_path: null`, no active flow, layout `0` only), `reset` plus `score.playback.slot.create` returned slot `1`, and `score.playback.slot.assignPlugin` for `vst3:/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3` alone drove the same ordinary `origin=session_build` provider path before the HTTP request dropped with an empty reply. The tracked terminal trace now shows truthful unsupported wrapper-marker and unknown-interface probes, then a successful `IHostApplication` query with a non-null returned object pointer, `IHostApplication::getName`, a matching release from refcount `5` back to `4`, and then the second wrapper-marker round before backend death. That rerun still never reached `IHostApplication::createInstance`, `IPlugInterfaceSupport::isPlugInterfaceSupported`, or any init-host `IComponentHandler*` callback, and no newer `.ips` had been written at capture time, so pair the tracked rack-slot trace with `solo-compose-2026-04-13-200205.ips`, which remains materially unchanged from the trusted `solo-compose-2026-04-13-163533.ips` family.
- 2026-04-13 late rerun: Repeated the same blank-shell slot-path repro on the current in-progress worktree after the latest host changes by starting `npm run start:dev`, confirming `app.getActiveScore` still returned `file_path: null`, `active_flow_id: null`, and `layout_ids: [0]`, then driving `reset`, `score.playback.slot.create`, and `score.playback.slot.assignPlugin` for `vst3:/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3` through the automation API. The assign request again failed with `curl: (52) Empty reply from server`, but the terminal trace now stopped even earlier: it still reached `controller_initialize`, logged stable controller, host-context, host-application, and component-handler pointer labels, and then exited before any of the older wrapper-marker `queryInterface(...)` lines or `IHostApplication::getName` callback appeared. No fresher `solo-compose-*.ips` was written during capture, and port `4767` was gone immediately afterward, so treat this as confirmed controller-init failure on the real slot path with changed live callback visibility rather than as proof of a new native crash location until a fresh crash report or debugger capture lands.

### Completion Notes List

- Story file created with explicit steering to unblock HALion Sonic on the ordinary direct-root path as soon as bounded real-path proof exists.
- Dedicated implementation spec added to capture debugger-retirement-first ordering, early ordinary-path unblock, and Steinberg-plus-MuseScore prior-art steering.
- Story keeps the scope bounded to single-instrument direct-root behavior and preserves the current non-root route restrictions.
- Task 1 retired the VST-specific debugger probe UI, removed the frontend invoke and shared-type surface that fed it, and preserved the rest of the general debugger window.
- Task 1 removed the dedicated backend probe commands and automation exposure from the Tauri surface, trimmed the remaining internal probe-only runtime helpers from `plugin_editor_poc.rs`, and updated maintainer docs to point investigation back to the ordinary product path.
- Task 3 now emits structured Rust-terminal logs from the ordinary product path in `plugin_editor_poc.rs`, including policy-gate entry, shared host-core phase transitions during session build, native view attach phases, outer attach failures, and active-phase host callback clues so crashes still leave lifecycle evidence in stderr.
- Task 3 cleanup renamed the ordinary-path logging seam from leftover telemetry wording to a smaller phase-logger abstraction so the shared host core keeps one honest lifecycle logging interface without preserving probe-era terminology.
- Task 2 preparatory narrowing removed `HALion Sonic.vst3` from the hardcoded policy-gate block in `plugin_host_compatibility.rs`, left HALion Sonic SE and the other concrete problem bundles blocked, and aligned inventory/tests so HALion Sonic now reaches the ordinary editor-open path for real-flow stderr investigation.
- Task 3 comparison follow-up now keeps the shared full host application for `controller.initialize()` but allocates a distinct `PluginComponentHandler` callback object for `setComponentHandler(...)`, matching MuseScore's split handler shape while preserving the same bounded init-time host context on the ordinary path.
- Task 3 follow-up corrected the shared host-contract seam so both lifecycle profiles now reuse the same full host application during `controller.initialize()`; the ordinary path no longer routes through the leftover delegated init-host wrapper, which removes a mixed-identity init surface from real HALion investigation.
- Task 3 follow-up also widened ordinary-path callback logging across the full host application's already-supported public interfaces so future HALion reruns can reveal whether it touches run-loop, component-handler, unit, progress, or data-exchange seams after `getName` without changing the bounded contract surface.
- Task 3 follow-up now decodes `IComponentHandler::restartComponent` flags in ordinary-path stderr logs so HALion reruns show named Steinberg restart reasons such as `kNoteExpressionChanged` instead of raw integers when the controller requests host-side configuration refresh.
- Task 3 follow-up now logs full-host `PluginHostApplication::queryInterface` resolution on the ordinary path via the shared host object's own COM interface list, so future HALion reruns can reveal any additional supported or unsupported host-interface probes that happen after `restartComponent(...)` without restoring a separate init-wrapper host.
- Task 3 comparison follow-up restored a MuseScore-style `IComponentHandler::restartComponent(...)` `kResultOk` no-op on the ordinary path after the `kNotImplemented` rerun produced the same HALION crash boundary, so the next rerun can compare concrete prior art against the unchanged ordinary-path failure.
- Comparison result: the subsequent MuseScore-style `kResultOk` rerun also produced the same ordinary-path HALION crash boundary and the same null-address `EXC_BAD_ACCESS`, so the `restartComponent(kNoteExpressionChanged)` return value does not currently appear to be the lever that moves HALION forward.
- Task 3 host-surface hardening now narrows the shared `PluginHostApplication` to init-time services only, so `controller.initialize()` no longer sees handler-family interfaces through either `queryInterface` or `IPlugInterfaceSupport`, while the distinct `PluginComponentHandler` remains the sole owner of `IComponentHandler*`, unit, progress, and related callback interfaces before `setComponentHandler(...)`.
- Task 3 investigation support now exposes the ordinary `plugin_editor_poc_open` command through automation and generated MCP as `plugin.editorHost.open` and `compose_action_plugin_editorHost_open`, so maintainers can trigger the real bounded editor-open path remotely without restoring the retired probe workflow.
- New rack-path evidence: HALion Sonic can already be mounted through the existing rack-first MCP flow by creating a slot and assigning deterministic inventory id `vst3:/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3`; that more realistic rerun still aborts the app during shared session build at `controller_initialize` immediately after `IComponentHandler::restartComponent flags=64 (kNoteExpressionChanged) => ok`, which confirms the active failure boundary is in slot-owned initialization rather than native editor attach.
- Task 3 host-core restructuring now follows Steinberg `PlugProvider` and MuseScore order directly on the ordinary product path: no pre-init `setComponentHandler(...)`, no live pre-connection controller state mirror, connection-point wiring immediately after `controller.initialize(...)`, then post-connection `setComponentHandler(...)`, with runtime bus activation and processing setup still deferred until after controller initialization completes.
- New exact-order MCP evidence from the real product paths: `ProEQ.vst3` still mounts cleanly through `compose_action_score_playback_slot_assignPlugin`, while both the rack-slot mount and the direct `compose_action_plugin_editorHost_open` reruns for `HALion Sonic.vst3` now crash earlier during `controller_initialize` before any `IComponentHandler2` query or `restartComponent(kNoteExpressionChanged)` callback occurs. The last logged host-side work on the faithful order is truthful unsupported `PluginHostApplication::queryInterface` traffic for `IVst3ToVst2Wrapper`, `IVst3ToAAXWrapper`, `IVst3ToAUWrapper`, and three unknown interfaces, followed by a successful `IHostApplication::getName` return. Treat the older post-`restartComponent` boundary as hybrid-order-only evidence; the active next investigation target is earlier controller-init host-context behavior on the exact prior-art lifecycle.
- Task 3 prior-art translation follow-up narrowed `PluginComponentHandler` to the same truthful callback surface MuseScore exposes on that object, `IComponentHandler` plus `IComponentHandler2`, instead of advertising the broader optional handler-family interfaces through the same callback object.
- New ordinary-path rerun evidence after that narrowing: `ProEQ.vst3` still mounts through the real rack-slot MCP path, HALion Sonic still crashes at the same post-`restartComponent(kNoteExpressionChanged)` controller-init boundary, and the handler-object log now shows HALion querying only `IComponentHandler2` before the crash. Treat the wider optional handler-family surface as ruled out for the current blocker unless later evidence proves HALion needs a specific missing callback contract.
- Additional MCP coverage now shows the same boundary on the direct ordinary editor-open action: `compose_action_plugin_editorHost_open` targeting `HALion Sonic.vst3` also takes down the app during shared session build at the same post-`IComponentHandler::restartComponent flags=64 (kNoteExpressionChanged) => ok` `controller_initialize` edge, before native editor attachment completes. The active blocker is therefore shared across rack-slot assignment and ordinary editor-open flows, not specific to slot-owned routing.
- Instrumented follow-up on the real slot path now narrows the boundary inside `controller_initialize` itself: the shared host object, host-context pointer, controller pointer, and distinct component-handler pointer remain stable into the call, `IHostApplication::getName` writes `Solo Compose` and returns `ok`, and the app still dies before `controller.initialize(...)` returns after HALion calls `restartComponent(kNoteExpressionChanged)`. Treat the remaining gap as plugin work still executing after the restart callback inside controller init, not as a failed `getName` return or a later controller state-sync step.
- Native debugger follow-up now captures the crash on the real slot path under `lldb`: after the same ordinary host-callback sequence, HALion Sonic stops with `EXC_BAD_ACCESS (code=1, address=0x0)` at `HALion Sonic` `___lldb_unnamed_symbol_513edc + 824`, with the faulting instruction `ldr x8, [x0]`. That shifts the active evidence from “controller.initialize never returns” to “HALion continues past the host restart callback and then dereferences a null plugin-owned pointer inside controller init.”
- Composite init-host comparison follow-up temporarily widened the shared `PluginHostApplication` to publish truthful public `IComponentHandler` and `IComponentHandler2` during `controller.initialize()` and added factory host-context handoff logging, while keeping wrapper-marker interfaces unsupported and preserving the distinct post-connection `PluginComponentHandler` object. A fresh ordinary `plugin.editorHost.open` rerun still crashed at the same earlier controller-init boundary without HALion ever querying those newly exposed init-host handler interfaces, so treat composite init-host widening as ruled out for the current blocker.
- Fresh exact-order crash-report evidence from `solo-compose-2026-04-13-115029.ips` now shows `EXC_BAD_ACCESS` / `SIGSEGV` at `KERN_INVALID_ADDRESS 0x40` on the main thread inside HALion Sonic while Rust is still in `IPluginBaseTrait::initialize` from `build_plugin_host_core_session_with_logger(...)`. That keeps the active evidence on a plugin-owned dereference during controller init rather than a later host callback, state-sync, or native-editor attach failure.
- Steinberg-style host-surface follow-up now keeps `IPlugInterfaceSupport` aligned with Steinberg's plug-side interface list and uses host-owned `ConnectionProxy` peers for component/controller handshake on the shared seam, matching `HostApplication` and `PlugProvider` structure more closely without restoring wrapper aliases or fake host surfaces.
- New unchanged-boundary evidence after that follow-up: the next ordinary rerun still crashed during `controller_initialize`, still never queried `IPlugInterfaceSupport` or any init-host `IComponentHandler*` seam, and fresh crash report `solo-compose-2026-04-13-122239.ips` repeated the same `KERN_INVALID_ADDRESS 0x40` fault inside HALion Sonic. Treat the plug-interface-support correction and connection-proxy hookup as ruled out for the current blocker.
- Task 3 follow-up now translates the shared lifecycle owner more literally from Steinberg `PlugProvider` and MuseScore's provider flow: one provider-style setup path, no comparison-era lifecycle split, post-provider component-state restoration, and controller-side `setComponentState(...)` mirroring only after provider initialization completes.
- New stronger negative evidence after that translation: the ordinary HALION rerun still crashed during `controller_initialize`, still only reached wrapper/unknown interface probes plus `IHostApplication::getName`, and fresh crash report `solo-compose-2026-04-13-124708.ips` repeated the same `KERN_INVALID_ADDRESS 0x40` fault while Rust remained inside `IPluginBaseTrait::initialize` from `initialize_steinberg_plug_provider(...)`. Treat provider-order recreation and post-provider state placement as ruled out for the current blocker unless new evidence contradicts the current ordinary-path trace.
- Fresh follow-up evidence now also rules out the remaining Steinberg-style `IHostApplication` success-result mismatch as the current lever: `solo-compose-2026-04-13-125859.ips` first reconfirmed the same provider-path crash envelope, the host shim then switched `IHostApplication::getName(...)` and successful `createInstance(...)` results to `kResultTrue`, the next ordinary rerun logged `IHostApplication::getName returned true`, and `solo-compose-2026-04-13-130949.ips` still repeated the same `KERN_INVALID_ADDRESS 0x40` fault while Rust remained inside `initialize_steinberg_plug_provider(...)`.
- Task 3 UX containment follow-up now bounds crash-prone plugin-host frontend invokes for slot assign, slot unassign, and bounded editor open to 12 seconds, so Sequence can fail visibly instead of waiting forever when HALION stalls or drops the backend during host init. Treat that as user-facing containment only; the fresh slot-path rerun and `solo-compose-2026-04-13-132518.ips` keep the underlying controller-init crash boundary unchanged.
- Native debugger clarification after a later user-reported hang now rules out a new backend failure class there as well: the app had been left paused under `lldb`, and resuming that same real slot-path request immediately stopped again inside `HALion Sonic` at `___lldb_unnamed_symbol_1f10dfc + 24` with faulting instruction `ldr x19, [x0, #0x40]`. Fresh crash report `solo-compose-2026-04-13-135902.ips` confirms the same main-thread `KERN_INVALID_ADDRESS 0x40` boundary while Rust remains inside `initialize_steinberg_plug_provider(...)`, and its recorded register state shows `x0 = 0` at the fault. Treat `lldb` post-stop backtraces as unreliable on this target because even plain `bt` now faults the debugger after the stop; the current trustworthy native evidence is the repeated fault instruction plus the crash-report thread state.
- The shared host core has now been rolled back from host-owned `ConnectionProxy` peers to direct symmetric component/controller `IConnectionPoint` hookup so the live seam matches the simplest truthful provider-path baseline. Focused validation still passes at 57/57, `ProEQ.vst3` remains healthy on the real slot path, and fresh crash report `solo-compose-2026-04-13-163533.ips` still repeats the same main-thread `KERN_INVALID_ADDRESS 0x40` boundary while Rust remains inside `initialize_steinberg_plug_provider(...)`. Treat connection-proxy ownership as ruled out for the current blocker and keep the restored direct handshake as the trusted baseline for the next investigation step.
- The shared host shim now also returns Steinberg-style `kNoInterface` for unsupported host `queryInterface` requests and routes the existing-window focus/close fast path through `run_on_main_thread`, so ordinary reruns no longer contaminate HALION evidence with a Compose-side AppKit thread trap when a native host window is already open.
- The shared host seam now closes one more prior-art gap without widening the public plug-support contract: the init-time host context and the post-init `setComponentHandler(...)` handoff reuse one truthful `PluginComponentHandler` object, so controllers that query `IComponentHandler` or `IComponentHandler2` from the host context during initialize now see the same callback identity they receive later through the explicit handler assignment.
- Fresh validation after that fix keeps the control path honest: `ProEQ.vst3` still opens cleanly on the ordinary bounded path and now survives a second same-target `plugin.editorHost.open` call with `existing_window=true`, while a clean HALION rerun from `existing_window=false` still dies during `controller_initialize` after the same wrapper-marker probes and successful `IHostApplication::getName`. Treat `solo-compose-2026-04-13-165657.ips` as invalid HALION evidence and keep `solo-compose-2026-04-13-163533.ips` plus the fresh clean terminal trace as the current unchanged boundary.
- Fresh late rerun evidence on the current worktree still keeps HALION inside the real slot-owned `controller_initialize` failure family, but the emitted callback window is now shorter than the earlier 20:34 BST trace: the last recorded line is the stable controller, host-context, host-application, and component-handler pointer summary immediately before `controller.initialize(...)`, and the process exits before any wrapper-marker `queryInterface(...)` traffic or `IHostApplication::getName` callback is logged. Because no new `.ips` landed during capture, treat this as either an earlier controller-init crash window or a regression in callback visibility on the current seam, not yet as a proved native-fault move.
- A follow-up host-logging hardening pass then moved Steinberg IID labeling fully into the C++ host bridge so the ordinary slot-owned path no longer dereferences plugin-owned raw IID pointers from Rust callback telemetry. The next real `reset -> score.playback.slot.create -> score.playback.slot.assignPlugin(HALion Sonic)` rerun restored the full wrapper-marker, unknown-interface, successful `IHostApplication`, and `getName` callback sequence without moving the crash boundary, so treat unsafe Rust-side callback labeling as ruled out for the current blocker.
- The next exact-order host-surface comparison added a custom Steinberg-based `IPlugInterfaceSupport` object that logs `isPlugInterfaceSupported(...)` calls while preserving the same public host context shape. A fresh real slot-owned HALION rerun still died on the same second wrapper-marker round, never emitted any `IPlugInterfaceSupport::isPlugInterfaceSupported(...)` callback, and still produced no newer `.ips`, so treat plug-interface-support discovery as ruled out for the current blocker on the ordinary path.
- The latest current-worktree lifecycle comparison now prepares the component runtime before controller initialization on the ordinary slot-owned path: `component_initialize`, then `component_runtime_setup`, then `controller_create`, then `controller_initialize`. HALION still reproduced the same real rack-slot failure after the first wrapper-marker and unknown-interface probe round, successful `IHostApplication::getName`, and second wrapper-marker round, with no new `.ips` emitted. Treat pre-controller processor readiness as necessary cleanup for the current seam but not as a fix for the active HALION blocker.
- A 2026-04-14 Steinberg hostclass cleanup then narrowed the C++ edge again by dropping Compose's custom `IPlugInterfaceSupport` object and letting `ComposeHostApplication` return Steinberg's SDK-owned support object directly. Focused validation still passed with `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` at 58/58, but the fresh blank-shell slot-path rerun (`reset -> score.playback.slot.create -> score.playback.slot.assignPlugin(HALion Sonic)`) still died with `curl: (52) Empty reply from server`, and fresh crash report `solo-compose-2026-04-14-084026.ips` kept the same main-thread `KERN_INVALID_ADDRESS 0x40` boundary while Rust remained inside `initialize_steinberg_plug_provider(...)`.
- That same follow-up also strengthened the evidence around the three still-unsupported private probes without justifying guessed support: local Steinberg binaries on this machine now expose the private host-interface names `IHostApplicationObsolete`, `IHostApplicationA`, `IHostApplication2`, `IHostApplicationW`, and `IHostApplicationNameSetter`, plus HALION-local `IInternalVST3Wrapper` and `IClosableWrapper`, but the public vendored VST3 SDK still contains none of them and no external contract evidence was recovered for their GUIDs or method shapes. Per Story 9.4 guardrails, do not satisfy HALION by inventing private-interface aliases or fake-success stubs from those names alone; the current value is narrower diagnosis, not a newly supported contract.

### File List

- `_bmad-output/implementation-artifacts/9-4-continue-adding-real-host-abilities-until-halion-works-in-the-product-path.md`
- `_bmad-output/implementation-artifacts/tech-spec-halion-product-path-remediation.md`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `docs/playback-support-baseline.md`
- `docs/development-guide.md`
- `src/windows/debugger/index.tsx`
- `src/windows/debugger/index.test.tsx`
- `src/services/actions.ts`
- `src/services/actions.test.ts`
- `src/services/types.ts`
- `src-tauri/src/playback/plugin_host_compatibility.rs`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/automation/registry.rs`
- `docs/development-guide.md`
- `docs/playback-support-baseline.md`
- `src-tauri/src/automation/mcp.rs`
- `docs/architecture.md`
- `docs/data-models.md`
