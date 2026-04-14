# Story 9.1: Classify Reference Multi-Instrument Plugin Load And Initialization Failures

Status: ready-for-dev

## Story

As a maintainer,
I want Compose to record the first failing phase when a reference multi-instrument plugin is loaded,
so that we can distinguish blocked host-contract work from plugin-side failure before claiming advanced support.

## Acceptance Criteria

1. Given a maintainer runs a reference multi-instrument plugin through the bounded diagnostic path, when the probe executes, then Compose records phase-specific results for module load, factory resolution, component creation, controller creation and initialization, connection-point setup, and view creation or attach, and the first failing phase is preserved instead of being collapsed into a generic blocked verdict.
2. Given a reference plugin fails or crashes before a bounded non-crashing load or initialization state is reached, when normal product support posture or route choices are computed, then the plugin remains blocked in ordinary user-facing flows, and Compose does not claim multi-instrument capability for that plugin.
3. Given the diagnostic result changes the repo's understanding of support posture, when the investigation output is recorded, then the support baseline and developer-facing docs are updated in the same change, and the blocked reason remains explicit about whether the first failure appears host-contract-related or plugin-side.

## Tasks / Subtasks

- [ ] Task 1: Extend the existing backend validation contract to preserve the first failing load/init phase (AC: 1)
  - [ ] Reuse `PluginEditorHostValidationPhase`, `PluginEditorHostValidation`, and `PluginEditorHostAttemptError` in `src-tauri/src/playback/plugin_inventory.rs` and `src-tauri/src/playback/plugin_editor_poc.rs` where possible instead of creating a second unrelated diagnostics model.
  - [ ] Ensure the bounded diagnostic result can distinguish at least these ordered phases: module load, factory resolution, component creation, controller creation, controller initialization, connection-point setup, and view creation or attach.
  - [ ] Preserve the first failing phase and its raw maintainer-facing message even when later steps are skipped.
  - [ ] If current phase names or payload fields are insufficient, evolve the shared backend contract once and mirror it through `src/services/types.ts` rather than layering aliases in React.

- [ ] Task 2: Add a maintainer-only diagnostic probe path that bypasses ordinary blocked-target gating without broadening product support (AC: 1, 2)
  - [ ] Keep ordinary user-facing flows and route-choice computation governed by the existing blocked posture in `src-tauri/src/playback/plugin_host_compatibility.rs`, `src-tauri/src/playback/plugin_inventory.rs`, `src-tauri/src/score/instruments/mod.rs`, `src-tauri/src/playback/preparation.rs`, and `src-tauri/src/playback/runtime_activation.rs`.
  - [ ] Add or extend an explicit maintainer-only command path in the playback owner seam, preferably alongside `src-tauri/src/playback/plugin_editor_poc.rs`, so a blocked reference plugin such as HALion can still be probed for first-failure classification.
  - [ ] Reuse the real host setup order in `build_plugin_host_session(...)`; do not create a fake fixture-only lifecycle or frontend-driven probe.
  - [ ] Keep the probe investigation-only: no route-choice emission, no `PluginPart` or `MidiChannel` enablement, and no shared-slot behavior.

- [ ] Task 3: Keep blocked posture explicit and distinguish host-contract failure from plugin-side failure (AC: 2, 3)
  - [ ] Make the recorded result clear about whether Compose failed before or during host-contract setup versus the plugin failing after the host satisfied the relevant bounded phase.
  - [ ] Keep current product behavior honest: blocked bundles remain blocked in ordinary inventory and editor-host flows until the investigation produces stronger evidence and later stories explicitly expand scope.
  - [ ] Do not collapse the maintainer result back into the existing generic unsupported message if the probe found a more precise first failing phase.
  - [ ] Preserve the current direct-root-only route behavior; Story 9.1 must not change `playback_route_choices_for_slot(...)`, advanced preparation, or runtime activation semantics.

- [ ] Task 4: Add seam-local regression coverage and maintainer documentation for the classifier (AC: 1, 2, 3)
  - [ ] Add focused Rust tests for the phase classifier, serialization of the recorded result, and the distinction between ordinary blocked posture and explicit maintainer-only probe results.
  - [ ] Keep tests in the owner seams under `src-tauri/src/playback/` rather than introducing a repo-wide multi-instrument harness.
  - [ ] Update `docs/playback-support-baseline.md` with the latest concrete blocked reason and first-failure classification language for the reference plugin.
  - [ ] Update `docs/development-guide.md` with the maintainer probe workflow, stop conditions, and interpretation rules for host-contract versus plugin-side failure.
  - [ ] Update `docs/data-models.md` or `docs/architecture.md` only if Story 9.1 introduces a durable new backend payload or command contract that later Epic 9 stories depend on.
  - [ ] Update `.github/copilot-instructions.md` or `docs/project-context.md` only if the implementation establishes a durable repo rule such as "multi-instrument route choices must remain blocked until plugin-derived proof exists."

## Dev Notes

### Story Intent

- Story 9.1 is the first implementation slice of Epic 9's host-hardening sequence.
- The job here is not to make HALion or any other multi-instrument plugin "supported." The job is to record how far a real reference plugin gets through bounded host setup before the first failure.
- The output of this story should give later Epic 9 stories a trustworthy starting point: are we still blocked at load/init, and if so, at which phase and for what likely reason?

### Historical Context To Carry Forward

- The old Story 3.5 shared-slot path was deferred specifically because HALion-backed multi-target proof did not exist and the repo could not safely guess part or channel support.
- The 2026-03 HALion investigation spec correctly separated editor-host or lifecycle investigation from shared-slot routing implementation. Epic 9 preserves that separation rather than reviving the old story wholesale.
- The current 2026-04 Epic 9 research already promoted Steinberg and MuseScore prior art into the planning basis. Build on it; do not redo the investigation from scratch or replace it with guessed host behavior.

### Prior-Art Guardrails

- Steinberg's multi-timbral guidance is the primary contract source. In particular:
  - `IComponent::setIoMode(...)` ordering matters for simple mode, but Story 9.1 should stop at load/init classification and must not claim capability proof just because an editor opens.
  - Multi-timbral structure is plugin-reported through interfaces such as `IUnitInfo`, `IUnitHandler`, `IUnitData`, `IProgramListData`, `getUnitByBus`, `selectUnit`, `getSelectedUnit`, and `getRoutingInfo`; Story 9.1 should only prepare the ground for later stories to inspect those facts.
  - Steinberg workflow diagrams and hosting examples remain the comparison baseline for expected host call ordering, especially around component or controller init and view attach.
- MuseScore and other comparator hosts are secondary lifecycle references:
  - instance identity is explicit and backend-owned;
  - view creation is tied to a live instance and main-threaded;
  - editor windows and resize lifecycle stay instance-bound rather than bundle-family-bound.
- Prior art should be used to explain or validate a failing phase, not to justify broadening Compose's support claim.

### This Story Owns

- A bounded maintainer-only diagnostic path for recording the first failing load/init phase of a reference multi-instrument plugin.
- A backend-owned result contract that later Epic 9 stories can consume without inventing frontend truth.
- Clear separation between ordinary blocked product posture and explicit maintainer investigation output.
- Documentation updates that record the new investigation workflow and current blocked reason precisely.

### This Story Does Not Own

- Capability probing for `simple_mode_only`, `advanced_mode_proven`, or `not_proven`; that belongs to Story 9.2.
- Verdict writing or scope recommendations; those belong to Stories 9.3 and 9.4.
- Any route-choice emission changes, `PluginPart` or `MidiChannel` enablement, or shared-slot behavior.
- Any frontend UX whose main purpose is to prove acceptance criteria literally.
- Any claim that HALion or broader multi-instrument playback is already supported.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_inventory.rs`
  - Owns `PluginEditorHostValidationPhase`, `PluginEditorHostValidation`, machine-local inventory truth, and stored validation payloads.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Already owns `PluginEditorHostAttemptError`, first-failure phase capture, and bounded editor-host lifecycle setup.
  - `build_plugin_host_session(...)` is the existing real host-order seam; prefer extending or reusing it over creating a new parallel probe implementation.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Owns the ordinary blocked-target posture for the bounded editor-host seam.
- `src-tauri/src/score/instruments/mod.rs`
  - Must remain direct-root-only for route-choice emission in Story 9.1.
- `src-tauri/src/playback/preparation.rs`
  - Must continue rejecting advanced route shapes in Story 9.1.
- `src-tauri/src/playback/runtime_activation.rs`
  - Must continue treating non-root route shapes as unsupported in Story 9.1.
- `src/services/types.ts`
  - Frontend or shared type mirror if the backend diagnostic contract changes.
- `docs/playback-support-baseline.md`
  - Canonical support posture and blocked-bundle explanation.
- `docs/development-guide.md`
  - Maintainer workflow and diagnostic runbook home.

### Implementation Guidance

- Prefer a sibling diagnostic result or a narrow evolution of the existing `editor_host_validation` payload rather than a second independent inventory or diagnostic taxonomy.
- Treat the current blocked list as product posture, not as proof that maintainer diagnostics are unnecessary. Story 9.1 should introduce an explicit maintainer probe path instead of weakening the ordinary gate.
- Keep any persisted diagnostic state machine-local and backend-owned. Do not write investigation results into `.soloc`.
- Preserve the exact first failing phase. If a plugin fails at controller initialization, the result must not be rewritten as generic unsupported plugin target.
- If a plugin crashes before a clean Rust error can be returned, record the last confirmed phase reached and the bounded crash context as precisely as the seam allows.
- Use Steinberg hosting examples or validator and workflow diagrams as comparison tools when lifecycle order looks suspicious.
- Use MuseScore-style instance-owned lifecycle thinking when deciding where a diagnostic result should attach; it should attach to a concrete bundle path or live instance context, not to vague vendor-family assumptions.

### Risks And Regression Traps

- Accidentally broadening ordinary product flows just to make the maintainer probe runnable.
- Creating a second diagnostics model that drifts from `plugin_inventory.rs` and `plugin_editor_poc.rs`.
- Treating successful editor-host attach as proof of advanced multi-instrument capability.
- Letting Story 9.1 leak into Story 9.2 by adding capability probing or `setIoMode` experimentation here.
- Reclassifying route-choice or runtime-activation behavior based on enums rather than plugin-derived proof.
- Losing the current blocked-bundle rationale in docs while the product still behaves as blocked.

### Testing Requirements

- Add or extend Rust tests for:
  - phase-specific result capture and serialization;
  - first-failure preservation when later phases are skipped;
  - ordinary blocked posture remaining unchanged while an explicit maintainer probe path exists;
  - no advanced route choices being emitted as a side effect of the classifier work.
- Recommended validation commands after implementation:
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_host_compatibility --lib -- --nocapture`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
  - `npm run check-types`
  - `npm run lint`
- Maintainer manual validation should include:
  - running the explicit reference-plugin probe on `HALion Sonic.vst3` or the chosen reference target;
  - recording the last successful phase and first failing phase;
  - confirming ordinary product inventory or editor-host posture remains blocked after the probe unless a later story intentionally changes it.

### Documentation Impact

- Update `docs/playback-support-baseline.md` to record the latest first-failure classification and to keep the blocked posture explicit.
- Update `docs/development-guide.md` with the maintainer-only probe workflow, interpretation guidance, and stop condition that Story 9.1 does not imply capability proof.
- Update `docs/data-models.md` if a durable diagnostic payload is added or existing machine-local validation payloads change shape.
- Update `docs/architecture.md` only if Story 9.1 introduces a durable named backend investigation seam that later stories depend on.
- Update AI instructions only if the story establishes a durable repo-wide rule about proof-gated multi-instrument investigation behavior.

### Architecture Compliance

- Backend remains the sole owner of investigation truth, blocked posture, and any stored diagnostic result.
- React remains a consumer of backend truth only; it must not infer failure phases or capability status.
- The product continues to treat multi-instrument route choices as unsupported until later plugin-derived proof exists.
- `.soloc` persistence rules remain unchanged. Investigation state is machine-local and maintainer-facing, not project intent.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- Do not add new dependencies for Story 9.1 unless a narrow serialization or test need is unavoidable.

### File Structure Notes

- Keep the owner implementation under `src-tauri/src/playback/`.
- Keep shared contract adjustments localized to `src/services/types.ts` only if the backend result shape crosses the Tauri boundary.
- Keep maintainer-facing narrative docs under `docs/`; do not create a second long-lived investigation tracker if the existing baseline and development guide can carry the truth.
- Keep story tracking under `_bmad-output/implementation-artifacts/` and update sprint status in the same pass.

### Git Intelligence Summary

- The current recent shipped work is still Epic 8 playback proof (`feat(8.6): close out 8.6 and 8 retro`, `feat(8.6): bowed playback tests`, `feat(8.5): expression maps`), so Story 9.1 should not assume any hidden multi-instrument groundwork landed after Epic 8.
- The current planning work already promoted the HALion investigation into Epic 9. Reuse the new research artifact and do not backslide into the deferred Story 3.5 assumption that enums alone justify route expansion.

### Latest Technical Information

- Current repo truth already exposes many of the right host-side interfaces in `plugin_editor_poc.rs`, but capability evidence remains only `NotProven` and no `setIoMode` call exists anywhere in `src-tauri/src`.
- `PluginEditorHostValidationPhase` already captures many of the right lifecycle phases, including `ModuleLoad`, `FactoryCreate`, `ComponentCreate`, `ControllerCreate`, `ControllerInitialize`, `ConnectionPointHandshake`, `ViewCreate`, and `ViewAttach`. Story 9.1 should reuse this structure where possible instead of starting over.
- Steinberg's current VST 3 developer portal still states that simple mode must call `IComponent::setIoMode(...)` before any other call and that complex multi-timbral structure depends on plugin-reported units and routing interfaces. That reinforces the Story 9.1 boundary: classify load/init first, then let Story 9.2 probe capability.
- Steinberg's SDK still ships hosting examples and a validator or test host, and MuseScore still keys live editor actions to concrete instance identity and main-thread view creation. Those are the right comparator anchors when diagnosing host-contract gaps.

### Open Questions Requiring Human Clarification

- None. The story should proceed with an explicit maintainer-only probe path while keeping ordinary product behavior blocked, which satisfies the current acceptance criteria without asking the developer to guess at capability scope.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 9 and Story 9.1 acceptance criteria.
- `_bmad-output/implementation-artifacts/tech-spec-investigate-multi-instrument-vst-support-with-halion.md`
  - Prior-art-grounded Epic 9 research basis and stop conditions.
- `_bmad-output/implementation-artifacts/archive/2026-03-pre-reset-vst-playback-cycle/tech-spec-investigate-halion-failure-before-story-3-5.md`
  - Earlier HALion investigation framing and lifecycle contract risks.
- `_bmad-output/implementation-artifacts/archive/2026-03-pre-reset-vst-playback-cycle/3-5-reuse-a-compatible-multi-part-plugin-instance-across-multiple-score-instruments.md`
  - Deferred shared-slot story and guardrail against guessed multi-target proof.
- `docs/project-context.md`
  - Backend ownership and bounded playback-domain rules.
- `docs/playback-support-baseline.md`
  - Current blocked-bundle posture and maintainer evidence language.
- `docs/development-guide.md`
  - Maintainer-facing host and inventory workflow guidance.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Shared validation payloads and machine-local inventory truth.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Real host lifecycle seam and first-failure capture.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Ordinary blocked-target policy seam.
- `src-tauri/src/score/instruments/mod.rs`
  - Direct-root-only route-choice seam that must remain unchanged in Story 9.1.
- `src-tauri/src/playback/preparation.rs`
  - Unsupported advanced-route preparation seam.
- `src-tauri/src/playback/runtime_activation.rs`
  - Unsupported advanced-route activation seam.
- `/memories/repo/multi-instrument-vst-halion-spec.md`
  - Epic 9 planning memory and proof-first guardrails.
- `/memories/repo/plugin-host-open-source-comparators.md`
  - Comparator host lifecycle and instance-ownership patterns.
- Steinberg VST 3 Developer Portal, Complex Plug-in Structures / Multi-timbral Instruments
  - simple mode, `setIoMode(...)`, `IUnitInfo`, `getUnitByBus`, `selectUnit`, `getSelectedUnit`, and `IComponent::getRoutingInfo`.
- Steinberg VST 3 Developer Portal, Minimum Host Requirements For VST 3 Support
  - host interface expectations including `IPlugInterfaceSupport`.
- Steinberg VST 3 Developer Portal, Workflow Diagrams
  - expected host call sequencing for processor, controller, and view lifecycle.
- Steinberg SDK `public.sdk/samples/vst-hosting`
  - hosting examples and validator comparator.
- MuseScore `IVstInstancesRegister`, `VstPluginInstance`, `VstViewDialog`, and instrument playback-channel model
  - instance-owned lifecycle, main-thread view creation, and score or playback separation.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-10: Story prepared from Epic 9 planning, the promoted HALion research artifact, the archived Story 3.5 or HALion investigation context, current playback owner seams, repo memory, and current Steinberg or MuseScore prior art.
- 2026-04-10: Story context explicitly anchors the implementation to existing validation types in `plugin_inventory.rs` and `plugin_editor_poc.rs` so the dev agent does not invent a parallel diagnostics model.
- 2026-04-10: Story context preserves the bounded product posture: ordinary flows stay blocked, route-choice emission stays direct-root-only, and capability probing is deferred to Story 9.2.
- 2026-04-10: Prior-art grounding refreshed from Steinberg's current developer portal pages for complex plug-in structures, host requirements, and workflow diagrams, plus current MuseScore VST instance lifecycle references.

### Completion Notes List

- Story context created for Story 9.1.
- Story 9.1 is ready for dev and preserves blocked product posture while later Epic 9 stories harden the host boundary.
- The story explicitly roots implementation guidance in Steinberg and MuseScore prior art to avoid guessed host or routing behavior.
- Sprint tracking should move Epic 9 to `in-progress` and Story 9.1 to `ready-for-dev` in the same pass.

### File List

- `_bmad-output/implementation-artifacts/9-1-classify-reference-multi-instrument-plugin-load-and-initialization-failures.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
