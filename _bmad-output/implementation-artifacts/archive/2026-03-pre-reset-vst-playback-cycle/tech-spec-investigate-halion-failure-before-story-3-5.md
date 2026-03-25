---
title: "Investigate HALion Failure Before Story 3.5"
slug: "investigate-halion-failure-before-story-3-5"
created: "2026-03-23"
status: "ready-for-dev"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
	- "Rust 2021 + Tauri 2.10.1 macOS native host seam"
	- "VST3 SDK 3.8 host/component/controller lifecycle"
	- "React 19.2.4 + TypeScript 5.9.3 frontend gating and state mirrors"
	- "Vitest + Testing Library frontend tests"
	- "Rust #[test] seam-local host and routing tests"
files_to_modify:
	- "src-tauri/src/playback/plugin_inventory.rs"
	- "src-tauri/src/playback/plugin_host_compatibility.rs"
	- "src-tauri/src/playback/plugin_editor_poc.rs"
	- "src/windows/main/components/plugins.tsx"
	- "src/windows/plugin-manager/index.test.tsx"
	- "src/windows/main/components/plugins.test.tsx"
	- "docs/development-guide.md"
	- "docs/project-context.md"
	- "src/services/types.ts"
code_patterns:
	- "Plugin inventory snapshots can mark entries unsupported before any editor-host attempt runs"
	- "Backend-owned plugin host lifecycle with a thin Tauri window wrapper"
	- "Explicit unsupported-target gate in compatibility helper before host attachment"
	- "VST3 host shim exposes selective interfaces through PluginHostApplication"
	- "Frontend plugin surfaces derive availability and editor-open posture from backend inventory validation results"
test_patterns:
	- "Rust host seam tests live next to plugin_editor_poc.rs and assert lifecycle/interface behavior"
	- "Rust compatibility tests live next to plugin_host_compatibility.rs"
	- "Rust plugin inventory tests cover unsupported-editor-host classification and persisted validation state"
	- "Frontend window tests use Vitest plus Testing Library with renderWithProviders"
	- "Frontend plugin inventory contracts mirror Rust payloads and use snake_case fields"
---

# Tech-Spec: Investigate HALion Failure Before Story 3.5

**Created:** 2026-03-23

## Overview

### Problem Statement

HALion Sonic is the only available multi-timbral plugin for validating Epic 3's multi-part and shared-slot path, but the current codebase records HALion Sonic as unsupported in the bounded native editor-host seam and does not yet prove real HALion-backed multi-target routing behavior. Without a bounded investigation and fix, Story 3.5 risks being implemented against assumptions rather than a trusted plugin path.

### Solution

Investigate the HALion Sonic-specific failure path, identify the exact seam or seams blocking usable behavior, and define bounded fixes that make HALion usable enough for the current Epic 3 work to be trusted. This spec will explicitly stop short of implementing shared-slot reuse behavior itself.

### Scope

**In Scope:**

- Diagnose the HALion Sonic-specific failure path in the current macOS VST3 product seams.
- Determine whether the blocker is isolated to the native editor-host path, broader plugin lifecycle handling, routing metadata exposure, or a combination.
- Define bounded fixes required to make HALion usable in the current work, including editor-host usability because that is required for trust in the current path.
- Define verification criteria for "HALion is usable enough to proceed with Epic 3" without claiming full audible playback proof.
- Record explicit guardrails that keep shared-slot reuse product behavior in Story 3.5.

**Out of Scope:**

- Implementing shared-slot reuse behavior itself.
- Multi-instrument route-linkage UI or score-context shared-slot presentation.
- Full audible playback proof or Epic 4 transport/audio behavior.
- Broad plugin compatibility expansion beyond HALion Sonic unless directly required by diagnosis.
- Sprint-plan changes unless the investigation proves they are necessary.

## Context for Development

### Codebase Patterns

- Rust is the authoritative owner of plugin inventory, host/runtime state, routing state, and persisted playback-domain state.
- React and Tauri frontend surfaces should remain thin presentation and command layers over backend-owned seams.
- The current bounded native editor-host path already treats HALion Sonic as unsupported based on recorded failure evidence.
- Story 3.4 intentionally stopped short of shared-slot reuse; this investigation must not silently consume Story 3.5's product behavior.
- Epic 3 depends on a rack-first model: slots own plugin assignment, instruments own explicit routes to existing slots.
- HALion Sonic is currently blocked by an explicit unsupported-list gate in `plugin_host_compatibility.rs`, and plugin inventory classification in `plugin_inventory.rs` also marks it as an unsupported editor-host target before any real host attempt reaches the maintainer workflow.
- The macOS host path in `plugin_editor_poc.rs` loads the bundle, resolves `GetPluginFactory`, sets host context via `IPluginFactory3`, creates and initializes the component, creates and initializes the edit controller, optionally syncs component state, connects component/controller `IConnectionPoint`s, and finally creates and attaches an `IPlugView`.
- The host shim intentionally exposes only a subset of VST3 host interfaces. Current tests prove `IUpdateHandler`, `IConnectionPoint`, `IMidiLearn`, `INoteExpressionController`, and `ChannelContext::IInfoListener` are reported unsupported, which is a plausible contributor to Steinberg-family controller-init failures.
- The route-authoring seam in `score/instruments/mod.rs` already models `PluginPart` and `MidiChannel`, but `playback_route_choices_for_slot` still emits only a single direct-plugin choice for non-empty slots, so real HALion-backed multi-target proof does not yet exist.
- Frontend plugin surfaces currently derive status and native-editor availability from backend `validation_failure` state, so any move away from name-based rejection has to update both backend inventory classification and the shared `plugins.tsx` presentation contract.

### Files to Reference

| File                                                                                                                  | Purpose                                                                                                       |
| --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `docs/project-context.md`                                                                                             | Authoritative AI-agent rules for backend ownership, playback seams, and current workflow boundaries           |
| `docs/development-guide.md`                                                                                           | Documents current unsupported HALion editor-host status and maintainer validation expectations                |
| `_bmad-output/planning-artifacts/epics.md`                                                                            | Source of Story 3.5 intent and the HALion-style MVP requirement                                               |
| `_bmad-output/implementation-artifacts/3-4-select-a-part-or-channel-when-assigning-a-compatible-multi-part-plugin.md` | Current pre-3.5 boundary and route-authoring seam context                                                     |
| `src-tauri/src/playback/plugin_inventory.rs`                                                                          | Inventory discovery and cached validation state that currently tags HALion Sonic as `unsupported_editor_host` |
| `src-tauri/src/playback/plugin_host_compatibility.rs`                                                                 | Current unsupported-list logic and HALion rejection messaging                                                 |
| `src-tauri/src/playback/plugin_editor_poc.rs`                                                                         | Existing native editor-host implementation and HALion-related tests                                           |
| `src/windows/main/components/plugins.tsx`                                                                             | Shared production UI that maps backend validation state into plugin availability and native-editor actions    |
| `src/services/types.ts`                                                                                               | Frontend mirror of backend diagnostics and plugin inventory payload fields                                    |
| `src/windows/plugin-manager/index.test.tsx`                                                                           | Frontend gating tests that currently treat HALion as unsupported in the editor-host workflow                  |
| `src/windows/main/components/plugins.test.tsx`                                                                        | Shared plugin-surface tests that mirror the unsupported-target posture                                        |
| `package.json`                                                                                                        | Confirms Vitest, Testing Library, and standard validation commands                                            |

### Technical Decisions

- HALion Sonic will be treated as the specific investigation target, not a generic proxy for future compatible plugins.
- Editor-host usability is mandatory for this investigation's success because the current Epic 3 path is not considered trustworthy otherwise.
- The investigation must establish a structured diagnostics-first result before it adds new host-interface support or changes any compatibility policy.
- The investigation may expose backend-owned diagnostics or plugin-derived capability facts, but it must not change route-authoring behavior, route-choice emission, or cross-instrument assignment workflows that belong to Story 3.5.
- Any task that starts implementing "route a second instrument into the same slot" or "show shared-slot linkage across instruments" belongs to Story 3.5 and is out of scope here.
- External vendor documentation is part of the evidence base for this spec. The VST3 docs confirm that `IPluginFactory` owns class enumeration and instance creation, `IPlugView` defines host obligations for attach, resize, focus, and frame callbacks, and `IUpdateHandler` is a host-implemented dependency-notification interface. Those contracts matter because the current host shim explicitly declines some of them.
- Because HALion Sonic is hard-blocked before host attachment today, the investigation must distinguish "unsupported by policy" from "fails after a real host attempt" and only keep the unsupported gate if the deeper failure remains justified by current evidence.
- Verified multi-target evidence in this spec means plugin-derived facts observed from a real HALion Sonic instance or bounded host attempt. Static enum support, hardcoded capability flags, or frontend-only assumptions do not count as proof.
- If deeper investigation proves HALion Sonic requires additional host interfaces or lifecycle ordering, those fixes belong in the bounded host seam only after the concrete failing lifecycle phase and missing contract are recorded in the diagnostics path.

## Implementation Plan

### Tasks

- [x] Task 1: Replace the unconditional HALion Sonic editor-host blacklist with evidence-driven target admission
  - File: `src-tauri/src/playback/plugin_host_compatibility.rs`
  - File: `src-tauri/src/playback/plugin_inventory.rs`
  - Action: Remove or narrow the current HALion Sonic bundle-name rejection and the derived inventory `unsupported_editor_host` classification so `HALion Sonic.vst3` can reach the bounded host seam for real validation instead of failing by policy before host setup begins.
  - Notes: Keep the compatibility helper, but make it represent evidence from actual bounded-host failures rather than a permanent vendor-name denylist. Preserve existing non-HALion exclusions only if the repo still has concrete failure evidence that is not being renegotiated by this story, and add regression coverage so known non-HALion exclusions are not accidentally re-enabled.
- [x] Task 2: Make the macOS VST3 host seam record a concrete HALion Sonic failure phase before attempting any bounded contract expansion
  - File: `src-tauri/src/playback/plugin_editor_poc.rs`
  - Action: Audit and tighten `build_plugin_host_session` so the host records each lifecycle phase for HALion Sonic, persists or returns a structured failure/result payload for maintainer use, and identifies the exact phase and missing contract before any new host-interface or callback support is added.
  - Notes: Investigation indicates the highest-risk areas are `IPluginFactory3::setHostContext`, pre/post-`setComponentHandler`, component-controller `IConnectionPoint` handshake, and currently unsupported host interfaces such as `IUpdateHandler` or related Steinberg-owned dependency/update hooks. Any added host support must be justified by the recorded failing phase, remain bounded to this seam, and not expand into full transport or automation work.
- [x] Task 3: Convert HALion frontend/editor-host posture from “blocked by name” to “backed by actual validation result”
  - File: `src/windows/main/components/plugins.tsx`
  - File: `src/windows/plugin-manager/index.test.tsx`
  - File: `src/windows/main/components/plugins.test.tsx`
  - Action: Update the shared inventory/editor-host UI behavior and tests so HALion Sonic is no longer treated as categorically unsupported by bundle family alone, and instead reflects the real structured result emitted by the backend validation path.
  - Notes: This task must not broaden the user-facing claim to “HALion fully supported.” The UI should only state what the bounded seam has actually proven and must remain driven by backend truth.
- [x] Task 4: Add a diagnostics-only proof seam for HALion readiness that stops short of Story 3.5 shared-slot behavior
  - File: `src-tauri/src/playback/plugin_editor_poc.rs`
  - File: `src-tauri/src/playback/plugin_inventory.rs`
  - File: `src/services/types.ts`
  - Action: If HALion Sonic can be initialized successfully, expose enough backend-owned diagnostic truth to determine whether the plugin is editor-host usable and whether the current product can observe plugin-derived capability evidence without changing route-choice emission or implementing multi-instrument shared-slot routing.
  - Notes: This must not create the actual Story 3.5 workflow. Acceptable outputs are bounded diagnostics, plugin-derived capability facts, or an explicit “still not provable” result grounded in real host behavior. Static enum support or frontend-only inference is not acceptable evidence.
- [x] Task 5: Update maintainer guidance and investigation runbook for HALion-specific validation
  - File: `docs/development-guide.md`
  - File: `docs/project-context.md`
  - Action: Replace the stale “HALion is on the unsupported list” guidance with the new bounded validation posture, including how to reproduce the HALion Sonic host attempt, what structured logs or results to inspect, and what outcome counts as sufficient Epic 3 confidence.
  - Notes: Documentation must clearly state that this work is a prerequisite confidence gate for Epic 3 and not proof of Epic 4 audible playback, and it must leave behind a repo-persisted evidence trail describing the exact lifecycle phase reached and the basis for any compatibility-policy change.

### Acceptance Criteria

- [ ] AC 1: Given `HALion Sonic.vst3` is installed locally, when plugin inventory is refreshed or the bounded native editor-host workflow is invoked, then the backend does not classify or reject it purely because of a hardcoded unsupported bundle-name rule.
- [ ] AC 2: Given HALion Sonic still fails during bounded host setup, when the host attempt runs, then the failure is emitted and documented as a structured result tied to a concrete lifecycle phase such as module load, component creation, controller creation, controller initialization, view creation, or view attachment rather than to a preemptive policy gate.
- [ ] AC 3: Given HALion Sonic fails during bounded host setup, when the investigation reaches a root-cause conclusion, then the repo records the concrete failing phase and missing host contract before any new host-interface or callback support is added.
- [ ] AC 4: Given HALion Sonic requires additional Steinberg host-contract support during controller initialization, when the bounded host seam is updated, then only the minimal bounded callbacks or interfaces justified by the recorded failing phase are supplied and HALion no longer fails for that specific missing-contract reason.
- [ ] AC 5: Given HALion Sonic becomes editor-host usable in the bounded seam, when a maintainer runs the documented validation flow, then the host window opens, remains attached, and closes cleanly without leaving stale lifecycle reservation state.
- [ ] AC 6: Given HALion editor-host usability is established, when the backend reports readiness evidence for later Epic 3 work, then it emits either plugin-derived capability facts from the real HALion Sonic instance or an explicit bounded result that the current seam still cannot prove that capability; static FE/BE enum support does not count as proof.
- [ ] AC 7: Given this quick spec is a prerequisite confidence gate, when implementation is complete, then Story 3.5 product behavior remains unimplemented and no task silently adds shared-slot reuse UX, second-instrument routing, or cross-instrument linkage presentation.

## Additional Context

### Dependencies

- Access to the existing HALion installation used in local validation.
- Existing macOS VST3 discovery, host, and routing seams already in this repository.
- Steinberg VST3 SDK interface contracts for `IPluginFactory`, `IPlugView`, and `IUpdateHandler`, plus HALion Sonic product documentation to interpret likely controller/editor expectations.
- Local maintainer ability to run the bounded host flow on macOS and inspect Rust logs from the plugin host POC.

### Testing Strategy

- Add or update Rust seam tests around `plugin_host_compatibility.rs` and `plugin_inventory.rs` so HALion Sonic is no longer rejected or classified unsupported solely by bundle-name policy.
- Extend Rust tests in `plugin_editor_poc.rs` for the revised host-interface support surface, lifecycle-state handling, structured diagnostics payloads, and HALion Sonic-specific host-attempt expectations.
- Update frontend Vitest coverage in `src/windows/plugin-manager/index.test.tsx` and `src/windows/main/components/plugins.test.tsx` so HALion posture follows actual validation status rather than the old unsupported-name assumption.
- Manual validation on macOS must include at least one HALion Sonic host attempt, successful or failing, with logs captured at the exact lifecycle step where the result occurs.
- If backend diagnostics or capability facts are added, cover their Rust serialization and frontend type alignment in `src/services/types.ts` and the consuming plugin inventory surfaces.
- Standard validation commands should include `npm run test:js`, `npm run check-types`, `cd src-tauri && cargo test --lib`, and `cd src-tauri && cargo clippy --lib -- -D warnings` for touched seams.

### Notes

- This spec exists to avoid distorting Story 3.5 around an unproven plugin path.
- It is acceptable for the outcome to recommend a sprint adjustment later, but that is not the default assumption for this artifact.
- Current highest-probability anchor points are: the unconditional unsupported-target gate, the VST3 host shim's interface support surface, and the controller initialization handshake in `build_plugin_host_session`.
- Steinberg documentation matters here because the current host shim explicitly reports `IUpdateHandler`, `IConnectionPoint`, `IMidiLearn`, `INoteExpressionController`, and `ChannelContext::IInfoListener` as unsupported at the host-support layer, while HALion product docs indicate a richer Steinberg ecosystem feature set than the current bounded seam was originally designed to satisfy.
- Success for this spec is not “HALion plays audio.” Success is “HALion reaches a trustworthy bounded-host state, and the repo can now say exactly whether Epic 3 confidence is justified or still blocked.”
- If implementation discovers that HALion editor-host usability necessarily requires broad host abstractions, transport work, or user-facing shared-slot behavior, stop and escalate rather than letting this spec absorb Story 3.5 or Epic 4.
