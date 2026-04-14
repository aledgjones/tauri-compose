title: "HALION Product-Path Remediation"
type: "implementation-spec"
created: "2026-04-12"
status: "in-progress"
baseline_commit: "8db3c1f0"
context:

- "../../docs/project-context.md"
- "../../docs/architecture.md"
- "../../docs/development-guide.md"
- "../planning-artifacts/epics.md"
- "9-4-continue-adding-real-host-abilities-until-halion-works-in-the-product-path.md"
- "tech-spec-halion-host-contract-hardening.md"

---

## Intent

Story 9.3 hardened the real backend-owned host contract and removed fake-success residue, but HALION remains blocked from the ordinary product path and still fails at `controller_initialize`. Story 9.4 should not continue as a debugger-led investigation loop or preserve a separate backend probe path. The next implementation slice is to move both debugging and implementation onto the real product path.

The ordered goal for this spec is:

1. retire the VST-specific debugger-window workflow that became the default investigation surface;
2. remove the matching backend probe path instead of preserving it in reduced form;
3. land the smallest truthful host-core changes needed to unblock HALION on the ordinary direct-root path early enough for real slot-mount testing;
4. continue adding real VST3 host behavior only as the unblocked ordinary path and its real-flow logs prove the next public-contract gap.

## Interpretation Of "Remove The Debugger Window Functionality"

This spec interprets that instruction narrowly and deliberately:

- Remove the HALION/VST-specific maintainer load/init probe controls, debugger-first workflow, and debugger-first documentation.
- Do not delete the general debugger window used by renderer, performance, and other non-VST maintainer tooling.
- Move lifecycle evidence onto backend-owned phase logging inside the ordinary direct-root host flow and shared host core instead of preserving a separate probe harness.

## Boundaries And Constraints

**Always:**

- Keep `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` as the canonical lifecycle owner.
- Use ordinary slot mounting and ordinary editor open on the direct-root path as the primary validation loop as soon as early unblock is possible.
- Use backend-owned logging on that ordinary path for phase evidence instead of a second probe lane.
- Root new host work in official Steinberg public contract requirements first, then MuseScore's real host implementation patterns, then broader comparator-host evidence from JUCE, Ardour, and Carla.
- Keep direct-root-only route emission, blocked posture for unrelated bundles, and bounded single-instrument scope intact.
- Update docs and BMAD artifacts in the same change when the stable workflow or support posture changes.

**Ask First:**

- If a proposed fix would require a new user-facing support claim beyond bounded single-instrument direct-root behavior.
- If a proposed fix appears to require vendor-private interface support without public Steinberg evidence or a strong open-source comparator basis.
- If removing the VST debugger probe surface would affect non-VST debugger features or other maintainer workflows.

**Never:**

- Do not add guessed private-GUID aliases, wrapper shims, opaque stubs, or fake-success return paths.
- Do not reintroduce a probe-only lifecycle owner, keep a reduced backend probe path, or keep the debugger window as the main iteration surface for HALION.
- Do not widen route-choice behavior, `PluginPart`, `MidiChannel`, shared-slot workflows, or multi-instrument claims in this slice.

## Ordered Execution Plan

### 1. Retire The VST-Specific Debugger Surface

- Remove HALION/VST maintainer probe controls from `src/windows/debugger/index.tsx` and matching tests.
- Remove or narrow frontend action plumbing in `src/services/actions.ts`, `src/services/actions.test.ts`, and `src/services/types.ts` when it only exists to support the debugger probe UI.
- Remove the dedicated backend probe commands and their registrations from `src-tauri/src/playback/plugin_editor_poc.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/automation/registry.rs`.
- Rewrite `docs/development-guide.md` so it no longer describes the Debugger window or any dedicated probe path as the preferred VST host workflow.

### 2. Unblock HALION Early On The Ordinary Path

- Make the smallest real host-core changes needed for bounded ordinary direct-root attach and editor open.
- Narrow or remove the HALION Sonic block in `src-tauri/src/playback/plugin_host_compatibility.rs` as soon as that bounded proof exists.
- Prove the same bundle can mount on an instrument slot and open through the real product path.

### 3. Grow The Real Host Contract Iteratively

- Surface the existing `PluginHostBuildTelemetry` phase data through the ordinary direct-root path so module load, host context, component/controller initialization, connection-point setup, view preparation, and teardown are logged from the real flow.
- Use the next real ordinary-path failure boundary to choose the next host addition.
- Add only the public-contract behavior needed to move past that boundary.
- Test each increment on the ordinary slot-mounted path first, using real-flow logs from that same path to classify the next missing host behavior.
- Treat the 2026-04-13 Steinberg `IPlugInterfaceSupport` correction plus `ConnectionProxy` port as already tried and ruled out for the current blocker: the subsequent ordinary rerun kept the same early `controller_initialize` crash boundary, never queried `IPlugInterfaceSupport`, and repeated the same `KERN_INVALID_ADDRESS 0x40` crash inside HALion Sonic.
- Treat the 2026-04-13 direct Rust recreation of Steinberg `PlugProvider` and MuseScore's post-provider handler/state order as already tried and ruled out too: the shared host core now uses one provider-style setup path, restores persisted component state only after provider initialization, mirrors it into separate controllers through `setComponentState(...)`, and still crashes at the same `controller_initialize` boundary on the next ordinary HALION rerun. Fresh crash report `solo-compose-2026-04-13-124708.ips` repeated the same `KERN_INVALID_ADDRESS 0x40` fault while Rust remained inside `IPluginBaseTrait::initialize` from `initialize_steinberg_plug_provider(...)`.
- Treat the 2026-04-13 Steinberg-style `IHostApplication` success-result correction as already tried and ruled out as well: after `solo-compose-2026-04-13-125859.ips` reconfirmed the same provider-path crash envelope, Compose aligned `IHostApplication::getName(...)` and successful `createInstance(...)` returns with Steinberg `kResultTrue`, focused tests still passed at 55/55, the next ordinary rerun logged `IHostApplication::getName returned true`, and fresh crash report `solo-compose-2026-04-13-130949.ips` still repeated the same `KERN_INVALID_ADDRESS 0x40` fault while Rust remained inside `initialize_steinberg_plug_provider(...)`.
- Treat the later rollback from host-owned `ConnectionProxy` peers back to direct symmetric component/controller `IConnectionPoint` hookup as already tried and now the trusted live baseline too: focused tests still passed at 57/57, a fresh real slot-path control rerun kept `ProEQ.vst3` healthy, and fresh crash report `solo-compose-2026-04-13-163533.ips` still repeated the same `KERN_INVALID_ADDRESS 0x40` fault while Rust remained inside `initialize_steinberg_plug_provider(...)`. That rules out connection-proxy ownership as the active lever and keeps the restored direct handshake as the comparison point for the next change.
- Treat the 2026-04-13 unsupported-host `queryInterface` parity cleanup as already tried host-contract cleanup too: Compose now returns Steinberg `kNoInterface` instead of `kResultFalse` for unsupported host `FUnknown::queryInterface` requests, focused tests still passed at 57/57, and `ProEQ.vst3` remained healthy on the ordinary bounded path.
- Treat crash report `solo-compose-2026-04-13-165657.ips` as invalid HALION evidence. That rerun reused an already-open native host window and crashed Compose itself on a `tokio-rt-worker` inside `focus_native_host_window` because `makeKeyAndOrderFront:` ran off the main thread. The ordinary product path now routes existing-window focus and error-path close through `run_on_main_thread`, and the same-window fast path was revalidated by opening `ProEQ.vst3` twice while automation health stayed up.
- A fresh ordinary HALION rerun after that fix again started from `existing_window=false`, advanced through the same faithful provider-path session build, repeated the same unsupported wrapper-marker query bursts plus successful `IHostApplication::getName`, and then killed the backend before `controller.initialize(...)` returned. No fresh `.ips` had been emitted yet at capture time, so the real-flow terminal trace plus `solo-compose-2026-04-13-163533.ips` remain the current unchanged-boundary evidence.
- Treat the next late 2026-04-13 blank-shell slot-path rerun on the current worktree as changed but still incomplete evidence: `reset`, `score.playback.slot.create`, and `score.playback.slot.assignPlugin` for `HALion Sonic.vst3` again dropped with `curl: (52) Empty reply from server`, and the live trace still reached `controller_initialize`, but the last emitted line was only the stable controller, host-context, host-application, and component-handler pointer summary immediately before `controller.initialize(...)`. That rerun no longer showed the earlier wrapper-marker `queryInterface(...)` or `IHostApplication::getName` callbacks before exit, and no new `.ips` landed during capture. The next investigation therefore has to distinguish between a genuinely earlier controller-init crash window and a regression in host-callback visibility on the current seam before drawing a stronger native-boundary conclusion.
- The next credible investigation target is therefore still not lifecycle order, connection-proxy ownership, pre-vs-post-provider state placement, unsupported-host `queryInterface` result parity, or the existing-window fast path. It is the remaining controller-init interoperability on that already-faithful provider path, with immediate emphasis on distinguishing a genuinely earlier pre-`getName` crash window from a regression in host-callback visibility.

### 4. Preserve The Bounded Product Claim

- Keep ProEQ or Reverb as a standing control-bundle regression guard.
- Keep HALion Sonic SE and other blocked bundles blocked unless this same story proves them separately.
- Keep direct-root-only route behavior and unsupported advanced-route activation unchanged.

## Prior-Art Anchors

### Official Steinberg Contract

- `Steinberg::Vst::IHostApplication` is the mandatory host callback interface passed into `IPluginBase::initialize(...)`, with `getName(...)` and `createInstance(...)` forming the basic host callback surface.
- `Steinberg::IPluginFactory3::setHostContext(...)` is the official factory-side host-context handoff.
- `Steinberg::Vst::IEditController` requires real `setComponentHandler(...)`, state exchange through `setComponentState(...)`, and real editor creation through `createView(...)`.
- `Steinberg::Vst::IComponentHandler` defines real edit and restart callbacks through `beginEdit(...)`, `performEdit(...)`, `endEdit(...)`, and `restartComponent(...)`.

These interfaces define the minimum truthful public contract. Story 9.4 should extend Compose by implementing the real host behaviors these interfaces imply, not by inventing fake compatibility branches around them.

### MuseScore As Primary Open-Source Comparator

MuseScore is the primary comparator for this story because it shows a real notation-oriented host architecture rather than a pure plug-in sandbox:

- It builds on Steinberg SDK host classes in `public.sdk/source/vst/hosting/*` rather than inventing a separate fake host layer.
- `src/framework/vst/internal/vstpluginprovider.cpp` wraps `Steinberg::Vst::PlugProvider`, runs real initialization, and then wires the controller to a real component handler.
- `src/framework/vst/internal/vstcomponenthandler.cpp` and `.h` implement a real `IComponentHandler` plus advanced handler behavior instead of returning placeholder success.
- `src/framework/vst/internal/vstplugininstance.cpp` owns controller/component state buffers, exposes real component/controller/view accessors, and creates the editor view from the real controller.
- `src/framework/vst/qml/Muse/Vst/vstview.cpp` and `src/framework/vst/widgets/vstviewdialog_qwidget.cpp` attach the real plug-in view to a platform window through `IPlugFrame` and `attached(...)`.

Compose should follow those patterns at the level of ownership and lifecycle ordering, not copy code.

### Secondary Comparators

- JUCE for ordered initialization and explicit host-context propagation.
- Ardour for explicit lifecycle ownership and teardown discipline.
- Carla for safer validation/discovery posture and bounded compatibility inspection.

Use them when MuseScore does not cover the exact lifecycle seam, but keep Steinberg plus MuseScore as the primary steering pair.

## Code Map

- `../../src/windows/debugger/index.tsx` - current VST-specific debugger controls to retire.
- `../../src/windows/debugger/index.test.tsx` - frontend regression seam for debugger-surface removal.
- `../../src/services/actions.ts` - frontend invoke surface for the current debugger probe workflow.
- `../../src/services/actions.test.ts` - regression seam for action changes.
- `../../src/services/types.ts` - shared probe-profile and probe-log types if the debugger path is narrowed.
- `../../src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` - canonical lifecycle owner for the real host seam.
- `../../src-tauri/src/playback/plugin_editor_poc.rs` - ordinary editor-host open path and the owner that should surface real-flow phase logging.
- `../../src-tauri/src/playback/plugin_host_compatibility.rs` - current ordinary blocked-bundle gate.
- `../../src-tauri/src/playback/plugin_inventory.rs` - backend-owned validation payloads and support posture truth.
- `../../src-tauri/src/score/instruments/mod.rs` - direct-root-only slot routing and assignment validation.
- `../../src-tauri/src/lib.rs` and `../../src-tauri/src/automation/registry.rs` - command and automation cleanup points when the backend probe path is removed.

## Tasks And Acceptance

**Execution:**

- [ ] Retire the VST-specific debugger workflow from `../../src/windows/debugger/index.tsx` and related frontend plumbing so HALION/VST host work stops centering the Debugger window.
- [ ] Remove the dedicated backend probe commands and registrations from `../../src-tauri/src/playback/plugin_editor_poc.rs`, `../../src-tauri/src/lib.rs`, and `../../src-tauri/src/automation/registry.rs`.
- [ ] Update `../../docs/development-guide.md` to remove debugger-first and probe-first VST guidance and describe the real-path-first logging workflow.
- [ ] Make the smallest truthful host-core changes on `../../src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` and adjacent owner files needed to reach bounded ordinary direct-root attach for HALION.
- [ ] Surface host-core phase logging through the ordinary direct-root path in `../../src-tauri/src/playback/plugin_editor_poc.rs` so the real path emits the lifecycle evidence needed to classify the next blocker.
- [ ] Narrow or remove the HALION Sonic block in `../../src-tauri/src/playback/plugin_host_compatibility.rs` as soon as ordinary-path proof exists, then validate real slot mounting and ordinary editor open.
- [ ] Continue adding host behavior only when the real ordinary path reveals the next missing public-contract capability, keeping every addition rooted in Steinberg plus MuseScore evidence.
- [ ] Preserve ProEQ or Reverb as a control-bundle regression guard, preserve blocked posture for unrelated bundles, and keep direct-root-only route scope unchanged.
- [ ] Update docs and BMAD artifacts in the same change when the primary workflow or support posture shifts.

**Acceptance Criteria:**

1. The VST-specific debugger-window probe workflow and matching backend probe commands are retired, and repo guidance no longer presents either as the preferred HALION/VST host workflow.
2. HALION Sonic is unblocked on the ordinary direct-root path as soon as the smallest truthful host-core change set yields bounded ordinary attach proof, and the same bundle can mount on an instrument slot through the real product path.
3. Ordinary-path phase logging supplies the lifecycle evidence needed to classify the next missing host capability without a second probe lane.
4. Additional host work after early unblock is justified by official Steinberg contract evidence and MuseScore-led open-source prior art, without private-GUID aliases, opaque stubs, wrapper shims, or fake-success returns.
5. A known-good control bundle remains stable, unrelated blocked bundles stay blocked, and direct-root-only route scope remains intact.

## Verification

**Commands:**

- `npm run test:js -- src/windows/debugger/index.test.tsx src/services/actions.test.ts`
- `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml plugin_host_compatibility --lib -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml runtime_activation --lib -- --nocapture`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- `npm run check-types`
- `npm run lint`

**Maintainer runbook:**

1. Start Compose with `npm run start:dev`.
2. Validate one known-good control bundle on the ordinary path.
3. After the HALION block is narrowed, validate HALION Sonic through real slot mounting and ordinary editor open on the direct-root path.
4. Inspect backend-owned real-flow logs from the ordinary path if HALION still fails, and use those logs to choose the next host capability to implement.

## Documentation Impact

- Update `../../docs/development-guide.md` to remove debugger-first and probe-first VST host guidance and document the real-path-first logging workflow.
- Update `../../docs/architecture.md` if retiring the VST debugger workflow or promoting the ordinary path changes the durable host-workflow contract.
- Update `../../docs/playback-support-baseline.md` when HALION's blocked reason changes or the ordinary path becomes usable.
- Keep `../planning-artifacts/epics.md`, `9-4-continue-adding-real-host-abilities-until-halion-works-in-the-product-path.md`, and `sprint-status.yaml` aligned with the ordered implementation posture.
- Update `../../.github/copilot-instructions.md` if the repo's durable VST-host implementation rules change.
