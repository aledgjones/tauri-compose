# Story 9.2: Extract A Stable Backend-Owned VST3 Host Core From The POC

Status: ready-for-dev

## Story

As a maintainer,
I want the lifecycle-critical VST3 host logic extracted into a stable backend-owned seam,
so that Compose can preserve the working single-instrument path while making HALion-class initialization failures fixable through one truthful host core.

## Acceptance Criteria

1. Given lifecycle-critical host logic currently lives mostly inside the bounded native host proof-of-concept, when Story 9.2 completes, then Compose exposes a clearer backend-owned VST3 host-core seam for factory host context, component and controller setup, handler timing, connection-point wiring, view ownership, and deterministic teardown, and maintainer-only diagnostic paths consume that seam instead of remaining the sole owner of lifecycle truth.
2. Given the single-instrument baseline is already proven and current blocked bundles remain blocked in ordinary flows, when the host core is extracted, then the existing supported single-instrument editor-host and playback-adjacent behavior stays stable, and blocked-bundle policy, direct-root route emission, and unsupported non-root activation remain unchanged in ordinary product behavior.
3. Given the extraction is grounded in Steinberg, JUCE, Ardour, and Carla prior art plus the completed HALion research, when the story is documented and reviewed, then the implementation explicitly cites those host patterns as the rationale for the new seam, and obsolete probe-only branches or dead-end controller-init variants are retired or clearly isolated if they no longer contribute to the hardened path.

## Tasks / Subtasks

- [ ] Task 1: Define a stable backend-owned VST3 host-core boundary inside the existing playback owner seam (AC: 1, 3)
  - [ ] Identify the lifecycle-critical responsibilities already mixed into `src-tauri/src/playback/plugin_editor_poc.rs`, including factory host context setup, component creation and initialization, controller creation and initialization, component-handler timing, connection-point wiring, view ownership, and teardown ordering.
  - [ ] Introduce a named host-core seam under `src-tauri/src/playback/` that becomes the canonical owner of those responsibilities, whether the first cut stays in `plugin_editor_poc.rs` behind clearer internal structure or moves selected logic into adjacent owner files.
  - [ ] Keep maintainer probes and editor-open flows as consumers of the host core rather than parallel owners of lifecycle truth.
  - [ ] Preserve backend ownership throughout; React and debugger surfaces must continue consuming backend truth only.

- [ ] Task 2: Preserve the proven single-instrument path and current blocked posture while extracting the host core (AC: 2)
  - [ ] Keep the current ordinary blocked-bundle posture in `src-tauri/src/playback/plugin_host_compatibility.rs` unchanged for HALion Sonic, HALion Sonic SE, Groove Agent SE, Kontakt 7, and MT-PowerDrumKit unless a later story explicitly reclassifies them.
  - [ ] Keep current direct-root-only route emission in `src-tauri/src/score/instruments/mod.rs` unchanged.
  - [ ] Keep unsupported non-root preparation and runtime activation behavior in `src-tauri/src/playback/preparation.rs` and `src-tauri/src/playback/runtime_activation.rs` unchanged.
  - [ ] Preserve the current supported single-instrument control path as the regression baseline rather than treating host-core extraction as a license to widen scope.

- [ ] Task 3: Retire or isolate dead-end probe-only lifecycle branches after the new seam exists (AC: 1, 3)
  - [ ] Audit maintainer-only controller-init profile permutations and keep only the narrow evidence paths still needed after host-core extraction.
  - [ ] Remove or isolate lifecycle variants that were already proven dead ends by Story 9.1 and the completed VST3 host research.
  - [ ] Ensure any remaining maintainer-only paths are clearly marked as investigation tools and do not own ordinary product behavior.
  - [ ] Keep `maintainer_load_init_probe` evidence distinct enough from canonical ordinary validation to support future scanner-style work.

- [ ] Task 4: Add seam-local regression coverage and documentation for the hardened host-core seam (AC: 1, 2, 3)
  - [ ] Add focused Rust tests that prove the extracted host core preserves lifecycle ordering, teardown expectations, and first-failure reporting semantics without widening route behavior.
  - [ ] Keep tests seam-local under `src-tauri/src/playback/`; do not introduce a repo-wide multi-instrument harness in this story.
  - [ ] Update `docs/development-guide.md` if maintainer-facing host workflow or diagnostics ownership changes materially.
  - [ ] Update `docs/architecture.md` and `docs/data-models.md` if the story introduces a durable named host-core seam or changes durable backend-owned validation payload boundaries.
  - [ ] Keep support-facing docs honest: current support posture remains unchanged unless later stories produce stronger evidence.

## Dev Notes

### Story Intent

- Story 9.2 is the first implementation slice after the completed Epic 9 research and Story 9.1 failure classification.
- The goal is not multi-timbral product support yet. The goal is to replace the current proof-of-concept ownership model with a stable backend-owned host core that future fixes can target.
- This story should make the repo structurally ready for richer VST work while preserving the proven single-instrument path.

### Why This Story Exists Now

- Story 9.1 already established the first-failure classifications for HALion-family bundles.
- The completed research artifact concluded that Compose does not primarily need more verdict-writing or ad hoc probe variants. It needs a stable, truthful, backend-owned VST3 host boundary with validation tooling beside it.
- The strongest current prior art from Steinberg, JUCE, Ardour, and Carla converges on the same model: layered host services, ordered lifecycle control, optional peer wiring, and deterministic reverse teardown.

### Prior-Art Grounding

- Steinberg remains the primary contract source:
  - VST3 host correctness still centers on truthful `IHostApplication` behavior, `IPluginFactory3::setHostContext(...)`, ordered component and controller setup, `IComponentHandler`, `IConnectionPoint`, `IMessage`, `IAttributeList`, `IPlugView`, and `IPlugFrame`.
  - The SDK still ships host helper classes, host examples, test host, validator, AudioHost, EditorHost, and VST3 Inspector.
- JUCE remains a strong host-structure comparator:
  - composite host context;
  - early factory host context setup;
  - ordered component holder lifecycle;
  - explicit message-thread-sensitive init paths.
- Ardour remains the strongest operational comparator:
  - dedicated `HostApplication` plus `VST3PI` separation;
  - explicit connection proxies;
  - scanner process with crash containment and timeout handling;
  - reverse teardown ordering.
- Carla remains the strongest low-level ownership comparator:
  - explicit host application, component handler, plugin frame, and run loop structs;
  - centralized lifecycle ownership in `PluginPointers`;
  - discovery path that mirrors runtime init ordering.

### This Story Owns

- Defining and extracting the stable backend-owned VST3 host core from the current proof-of-concept seam.
- Preserving current single-instrument support behavior and blocked ordinary flows while that extraction happens.
- Converting maintainer-only diagnostics into consumers of the new seam rather than owners of lifecycle truth.
- Retiring or isolating probe-only dead ends that no longer serve the hardened path.

### This Story Does Not Own

- Proving advanced multi-timbral capability through `setIoMode`, `IUnitInfo`, `getUnitByBus`, or `getRoutingInfo`.
- Any route-choice emission changes, `PluginPart` or `MidiChannel` enablement, or shared-slot behavior.
- Any user-facing claim that HALion or broader multi-instrument playback is already supported.
- The scanner-style compatibility lane itself; that belongs to Story 9.3.
- The first hardened HALion rerun verdict; that belongs to Story 9.4.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Current owner of most lifecycle-critical VST3 host behavior, including `build_plugin_host_session(...)`, host-context setup, first-failure capture, connection-point helpers, view creation, and attach flow.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Backend-owned inventory and validation payload storage, including `editor_host_validation` and `maintainer_load_init_probe`.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Ordinary blocked-bundle posture for the bounded seam.
- `src-tauri/src/score/instruments/mod.rs`
  - Direct-root-only route-choice emission that must remain unchanged in this story.
- `src-tauri/src/playback/preparation.rs`
  - Unsupported advanced-route preparation seam that must remain unchanged in this story.
- `src-tauri/src/playback/runtime_activation.rs`
  - Unsupported advanced-route activation seam that must remain unchanged in this story.

### Extraction Guidance

- Prefer the smallest real structural cut that creates a truthful named host-core seam.
- Do not let file motion become the goal. If the first stable seam lives as clearer internal structure plus a small adjacent owner module, that is acceptable.
- The extracted seam should own at least these ordered responsibilities:
  - factory acquisition and optional `setHostContext(...)`;
  - component creation and initialization;
  - controller creation and initialization;
  - component-handler timing;
  - component-to-controller state sync and connection-point handshake;
  - view creation, `setFrame`, and attach preparation;
  - deterministic reverse teardown.
- Keep the seam backend-owned and reusable across:
  - ordinary bounded editor-host flows;
  - maintainer load/init probes;
  - future scanner-style compatibility work.

### Risks And Regression Traps

- Turning host-core extraction into a broad architectural rewrite that delays actual progress.
- Breaking the existing single-instrument supported path while trying to generalize the seam.
- Leaving maintainer probe logic as the de facto lifecycle owner after the new seam exists.
- Allowing extraction work to widen direct-root route behavior or unsupported route activation paths.
- Preserving dead-end probe variants that make the hardened path harder to reason about.

### Testing Requirements

- Add or extend Rust tests for:
  - preserved lifecycle ordering through the new host-core boundary;
  - deterministic teardown and cleanup behavior;
  - ordinary blocked posture remaining unchanged while the new seam exists;
  - maintainer probe paths consuming the host core instead of duplicating lifecycle ownership;
  - no route-choice or non-root activation behavior changing as a side effect of the extraction.
- Recommended validation commands after implementation:
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_host_compatibility --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml runtime_activation --lib -- --nocapture`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
  - `npm run check-types`
  - `npm run lint`
- Manual validation should include:
  - at least one currently working single-instrument control bundle;
  - at least one HALion-class failing reference bundle through the maintainer path;
  - confirmation that ordinary blocked user-facing posture remains unchanged.

### Documentation Impact

- Update `docs/architecture.md` if the story introduces a durable named host-core seam or changes owner boundaries materially.
- Update `docs/data-models.md` if the backend-owned validation payload or maintainer-probe storage boundaries change materially.
- Update `docs/development-guide.md` if maintainer workflow or probe ownership changes materially.
- Do not broaden support-facing docs in this story; support posture remains unchanged until later evidence justifies it.

### Architecture Compliance

- Backend remains the sole owner of host lifecycle truth, blocked posture, and investigation results.
- React remains a projection surface only.
- Current single-instrument trust posture remains intact and must not be used as proof of multi-timbral readiness.
- `.soloc` persistence rules remain unchanged. Investigation and validation state stay machine-local and backend-owned.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- Do not add new dependencies unless the extraction reveals a narrow unavoidable need.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Updated Epic 9 and Story 9.2 acceptance criteria.
- `_bmad-output/implementation-artifacts/9-1-classify-reference-multi-instrument-plugin-load-and-initialization-failures.md`
  - Completed failure-classification baseline and current maintainer probe constraints.
- `_bmad-output/implementation-artifacts/tech-spec-investigate-multi-instrument-vst-support-with-halion.md`
  - Research basis and updated implementation-direction guidance.
- `_bmad-output/planning-artifacts/research/technical-vst3-host-init-compatibility-best-practices-research-2026-04-11.md`
  - Prior-art synthesis and strategic recommendation to build a stable backend-owned host core.
- `docs/project-context.md`
  - Backend ownership and bounded playback-domain guardrails.
- `docs/development-guide.md`
  - Maintainer-facing host workflow guidance.
- `docs/playback-support-baseline.md`
  - Current blocked posture and reference classification language.
- `/memories/repo/multi-instrument-vst-halion-spec.md`
  - Existing Epic 9 planning memory and guardrails.
