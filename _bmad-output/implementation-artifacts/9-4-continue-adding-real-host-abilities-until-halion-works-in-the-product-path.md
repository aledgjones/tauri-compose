# Story 9.4: Continue Adding Real Host Abilities Until HALION Works In The Product Path

Status: ready-for-dev

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

- [ ] Task 1: Retire the VST-specific debugger workflow and remove the matching backend probe path before further HALION remediation work proceeds (AC: 1)
  - [ ] Remove the HALION/VST maintainer load/init probe controls from `src/windows/debugger/index.tsx` and `src/windows/debugger/index.test.tsx`, and remove any matching frontend action/type plumbing in `src/services/actions.ts`, `src/services/actions.test.ts`, and `src/services/types.ts` that only exists to feed that debugger surface.
  - [ ] Remove the dedicated backend probe commands and registrations from `src-tauri/src/playback/plugin_editor_poc.rs`, `src-tauri/src/lib.rs`, and `src-tauri/src/automation/registry.rs` so the repo no longer preserves a separate HALION probe lane.
  - [ ] Stop treating the Debugger window as the preferred VST host workflow in `docs/development-guide.md`, and replace probe-oriented guidance with real-path-first logging guidance.
  - [ ] Keep non-VST debugger capabilities unchanged. This task retires the VST-specific debugger workflow; it does not delete the general debugger window used by other subsystems.

- [ ] Task 2: Unblock HALion Sonic on the ordinary bounded path as soon as the minimum real host-core change set makes that safe enough for real iteration (AC: 2, 3)
  - [ ] Replace the current hardcoded ordinary HALion Sonic policy block in `src-tauri/src/playback/plugin_host_compatibility.rs` with narrower, evidence-driven gating once the shared seam reaches bounded ordinary-path attach proof.
  - [ ] Ensure `src-tauri/src/playback/plugin_editor_poc.rs` can open HALion Sonic through the ordinary direct-root editor-host path without using maintainer-only bypasses, and prove the same bundle can be mounted on an instrument slot through the real product path.
  - [ ] Keep HALion Sonic SE and the other currently blocked bundles unchanged unless this story produces equivalent bounded ordinary-path proof for them.

- [ ] Task 3: Add structured logging to the real host flow and build out additional host abilities iteratively from ordinary-path evidence and public prior art (AC: 1, 2, 3)
  - [ ] Start from `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` as the canonical lifecycle owner and keep new lifecycle or service work on that shared seam rather than reintroducing a probe-only owner.
  - [ ] Surface the existing host-core phase telemetry through the ordinary direct-root open path in `src-tauri/src/playback/plugin_editor_poc.rs` so module load, host context, component and controller initialization, connection-point setup, view preparation, and teardown are logged from the real path rather than a parallel probe path.
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
  - Current blocked HALion posture and control-bundle evidence.
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

### Completion Notes List

- Story file created with explicit steering to unblock HALion Sonic on the ordinary direct-root path as soon as bounded real-path proof exists.
- Dedicated implementation spec added to capture debugger-retirement-first ordering, early ordinary-path unblock, and Steinberg-plus-MuseScore prior-art steering.
- Story keeps the scope bounded to single-instrument direct-root behavior and preserves the current non-root route restrictions.

### File List

- `_bmad-output/implementation-artifacts/9-4-continue-adding-real-host-abilities-until-halion-works-in-the-product-path.md`
- `_bmad-output/implementation-artifacts/tech-spec-halion-product-path-remediation.md`
