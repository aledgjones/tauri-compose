# Story 9.2: Extract A Stable Backend-Owned VST3 Host Core From The POC

Status: review

## Story

As a maintainer,
I want the lifecycle-critical VST3 host logic extracted into a stable backend-owned seam,
so that Compose can preserve the working single-instrument path while making HALion-class initialization failures fixable through one truthful host core.

## Acceptance Criteria

1. Given lifecycle-critical host logic currently lives mostly inside the bounded native host proof-of-concept, when Story 9.2 completes, then Compose exposes a clearer backend-owned VST3 host-core seam for factory host context, component and controller setup, handler timing, connection-point wiring, view ownership, and deterministic teardown, and maintainer-only diagnostic paths consume that seam instead of remaining the sole owner of lifecycle truth.
2. Given the single-instrument baseline is already proven and current blocked bundles remain blocked in ordinary flows, when the host core is extracted, then the existing supported single-instrument editor-host and playback-adjacent behavior stays stable, and blocked-bundle policy, direct-root route emission, and unsupported non-root activation remain unchanged in ordinary product behavior.
3. Given the extraction is grounded in Steinberg, JUCE, Ardour, and Carla prior art plus the completed HALion research, when the story is documented and reviewed, then the implementation explicitly cites those host patterns as the rationale for the new seam, and obsolete probe-only branches or dead-end controller-init variants are retired or clearly isolated if they no longer contribute to the hardened path.

## Tasks / Subtasks

- [x] Task 1: Define a stable backend-owned VST3 host-core boundary inside the existing playback owner seam (AC: 1, 3)
  - [x] Identify the lifecycle-critical responsibilities already mixed into `src-tauri/src/playback/plugin_editor_poc.rs`, including factory host context setup, component creation and initialization, controller creation and initialization, component-handler timing, connection-point wiring, view ownership, and teardown ordering.
  - [x] Introduce a named host-core seam under `src-tauri/src/playback/` that becomes the canonical owner of those responsibilities, whether the first cut stays in `plugin_editor_poc.rs` behind clearer internal structure or moves selected logic into adjacent owner files.
  - [x] Keep maintainer probes and editor-open flows as consumers of the host core rather than parallel owners of lifecycle truth.
  - [x] Preserve backend ownership throughout; React and debugger surfaces must continue consuming backend truth only.

- [x] Task 2: Preserve the proven single-instrument path and current blocked posture while extracting the host core (AC: 2)
  - [x] Keep the current ordinary blocked-bundle posture in `src-tauri/src/playback/plugin_host_compatibility.rs` unchanged for HALion Sonic, HALion Sonic SE, Groove Agent SE, Kontakt 7, and MT-PowerDrumKit unless a later story explicitly reclassifies them.
  - [x] Keep current direct-root-only route emission in `src-tauri/src/score/instruments/mod.rs` unchanged.
  - [x] Keep unsupported non-root preparation and runtime activation behavior in `src-tauri/src/playback/preparation.rs` and `src-tauri/src/playback/runtime_activation.rs` unchanged.
  - [x] Preserve the current supported single-instrument control path as the regression baseline rather than treating host-core extraction as a license to widen scope.

- [x] Task 3: Retire or isolate dead-end probe-only lifecycle branches after the new seam exists (AC: 1, 3)
  - [x] Audit maintainer-only controller-init profile permutations and keep only the narrow evidence paths still needed after host-core extraction.
  - [x] Remove or isolate lifecycle variants that were already proven dead ends by Story 9.1 and the completed VST3 host research.
  - [x] Ensure any remaining maintainer-only paths are clearly marked as investigation tools and do not own ordinary product behavior.
  - [x] Keep `maintainer_load_init_probe` evidence distinct enough from canonical ordinary validation to support future scanner-style work.

- [x] Task 4: Add seam-local regression coverage and documentation for the hardened host-core seam (AC: 1, 2, 3)
  - [x] Add focused Rust tests that prove the extracted host core preserves lifecycle ordering, teardown expectations, and first-failure reporting semantics without widening route behavior.
  - [x] Keep tests seam-local under `src-tauri/src/playback/`; do not introduce a repo-wide multi-instrument harness in this story.
  - [x] Update `docs/development-guide.md` if maintainer-facing host workflow or diagnostics ownership changes materially.
  - [x] Update `docs/architecture.md` and `docs/data-models.md` if the story introduces a durable named host-core seam or changes durable backend-owned validation payload boundaries.
  - [x] Keep support-facing docs honest: current support posture remains unchanged unless later stories produce stronger evidence.

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
- The real HALION host-contract hardening and cleanup-in-place follow-up; that belongs to Story 9.3.
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

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-11: Loaded the BMAD story workflow, sprint status, project context, Epic 9 story context, and the existing `plugin_editor_poc.rs` / maintainer-probe seams before implementation.
- 2026-04-11: Extracted the lifecycle-critical VST3 host ordering into `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs`, keeping ordinary editor-host opens and maintainer load/init probes on one backend-owned lifecycle seam.
- 2026-04-11: Retired the inline probe-profile timing helpers from `plugin_editor_poc.rs`, isolated the remaining host-only controller-init investigation profile inside the shared host core, and added seam-local lifecycle-profile and teardown-order tests.
- 2026-04-11: Updated `docs/architecture.md`, `docs/data-models.md`, and `docs/development-guide.md` to document the new host-core ownership and prior-art rationale while keeping support posture unchanged.
- 2026-04-11: Validation passed with `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture`, `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib -- --nocapture`, `cargo test --manifest-path src-tauri/Cargo.toml plugin_host_compatibility --lib -- --nocapture`, `cargo test --manifest-path src-tauri/Cargo.toml runtime_activation --lib -- --nocapture`, `cargo test --manifest-path src-tauri/Cargo.toml --lib`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `npm run check-types`, and `npm run lint`.
- 2026-04-12: Retrieved the matching macOS crash report and persisted maintainer probe log for HALion Sonic, confirmed the `EXC_BAD_ACCESS (KERN_INVALID_ADDRESS at 0x40)` still lands inside `controller_initialize`, then used local disassembly to show the faulting HALion helper immediately dereferences `[x0 + 0x40]`, which is more consistent with a null internal wrapper object than a bare `release()` on a null COM result.
- 2026-04-12: Re-ran HALion with the wrapper-marker alias and non-null unsupported fallback in place, confirmed the crash remained at the same `controller_initialize` boundary after the plugin had seen the three HALion-only unknown GUIDs plus `IHostApplication::getName`, and narrowed the next bounded maintainer experiment to success-path aliasing for those three GUID probes instead of only returning `kNoInterface`.
- 2026-04-12: Compared the newest HALion crash report (`solo-compose-2026-04-12-110638.ips`) against the earlier `0x40` crash and confirmed the boundary moved into Compose's own `IHostApplication::getName` shim. HALion now reaches `getName` after the GUID aliases, but in that run it passed an invalid `String128` out-buffer pointer (`0x60`), so the host shim was first hardened to stop dereferencing obviously bogus buffers.
- 2026-04-12: The follow-up HALion rerun (`solo-compose-2026-04-12-112306.ips`) moved past the host-side `getName` write crash and exposed the next contract gap: macOS `kInvalidArgument` is the integer `2`, and HALion reused that result as a UTF-16 pointer inside its own string helper (`0x12ab094: ldrh w9, [x8], #0x2`). The host shim now keeps the invalid-buffer guard but returns a logged no-op `kResultOk` for obviously bogus `getName` buffers instead of feeding HALion the toxic error code.
- 2026-04-12: The next HALion rerun (`solo-compose-2026-04-12-125309.ips`) moved past that UTF-16 helper and exposed another raw-pointer contract gap on the host side: HALion reached `IHostApplication::createInstance`, but Compose crashed first while trying to label a sentinel-like `TUID*` value (`0xFFFFFFFF`) for probe logging. The maintainer-only host shim now copies raw callback IIDs through a bounded pointer heuristic before dereferencing them, logs unreadable values as `invalid_pointer(...)`, reuses the existing non-null unsupported fallback for invalid `queryInterface` IIDs, and lets `createInstance` fall back to the valid class or interface IID when HALion supplies one good ID plus one garbage sentinel.
- 2026-04-12: The follow-up HALion rerun advanced one step further and showed that the host still assumed the `createInstance` output-object pointer itself was writable. HALion provided `obj = 0x1`, which caused a Rust panic on the eager `*obj = null_mut()` clear. The maintainer-only host shim now validates object-output pointers before writing, and the same guard also covers the non-null unsupported `queryInterface` fallback path.
- 2026-04-12: The next HALion rerun (`solo-compose-2026-04-12-133048.ips`) proved that bounded failure was still unsafe for `createInstance` invalid-output-pointer paths. With the misaligned write removed, HALion crashed again inside the same helper that had previously reused `kInvalidArgument == 2` as a pointer, but this time the fault address was `0x3` after `IHostApplication::createInstance` returned `kNotImplemented` for `obj = 0x1`. The maintainer-only host shim now keeps the invalid-output-pointer write guard but returns a logged no-op `kResultOk` for null or invalid `createInstance` output pointers instead of feeding HALion the toxic `3` result code.
- 2026-04-12: The latest HALion rerun (`solo-compose-2026-04-12-134745.ips`) moved past the toxic `0x2` and `0x3` result-code crashes but still failed during `controller_initialize`. The new crash was a main-thread `SIGBUS` with PC alignment inside Compose's own `__TEXT` segment at bytes that decode to the rodata string `src/playback/plugin_editor_poc.rs`, which is consistent with HALion invoking a post-`FUnknown` virtual slot on one of the three proprietary init-time GUID interfaces rather than treating them as true empty markers. The maintainer-only host shim now serves `EEC89683-80DB4336-9177273E-138A763B`, `97E507DA-E5594EEA-A5AC4042-E458F2B4`, and `CC0B7716-362B4EE5-89EE5F41-5458463F` from dedicated opaque stub interfaces with padded no-op slots instead of aliasing them to the host application's `FUnknown` surface.

### Completion Notes List

- Added the backend-owned VST3 host core at `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` and made `plugin_editor_poc.rs` delegate session build, view ownership, and deterministic teardown through that seam.
- Preserved the current ordinary blocked-bundle posture, direct-root-only route behavior, and non-root activation limits while keeping ordinary editor-host flows and maintainer probes on the same lifecycle truth.
- Isolated the remaining host-only controller-init investigation profile inside the shared host core and removed the old top-level probe-profile timing helpers from `plugin_editor_poc.rs`.
- Added seam-local Rust coverage for lifecycle-profile mapping and reverse teardown ordering in `src-tauri/src/playback/plugin_editor_poc.rs`.
- Updated `docs/architecture.md`, `docs/data-models.md`, and `docs/development-guide.md` to document the host-core seam, its Steinberg/JUCE/Ardour/Carla rationale, and unchanged support posture.
- Validation completed successfully with the targeted Rust owner-seam tests, the full Rust library test suite, `cargo clippy`, `npm run check-types`, and `npm run lint`.
- Retrieved the HALion Sonic crash evidence from macOS `DiagnosticReports` and the persisted maintainer probe log, then changed the maintainer-only host-only controller-init profile so the empty Steinberg wrapper-marker IIDs alias back to the host application's `FUnknown` surface.
- Confirmed that a balanced non-null fallback `FUnknown` on unsupported init-time probes did not move the HALion crash boundary, then tightened the investigation profile again so the three observed HALion-only GUID probes `EEC89683-80DB4336-9177273E-138A763B`, `97E507DA-E5594EEA-A5AC4042-E458F2B4`, and `CC0B7716-362B4EE5-89EE5F41-5458463F` first advanced through an investigation-only alias path before the later `solo-compose-2026-04-12-134745.ips` rerun showed that at least one of them is not behaving like an empty marker.
- Replaced that earlier empty-marker alias for the three HALion-only GUIDs with dedicated opaque stub interfaces whose padded post-`FUnknown` slots log and return no-op `kResultOk`, because the latest PC-alignment crash into Compose rodata is consistent with HALion invoking a later virtual method on one of those proprietary interfaces.
- Hardened both host-application `getName` implementations so obviously bogus `String128` out-buffers such as HALion's observed `0x60` pointer are logged and treated as a no-op `kResultOk`. That keeps the host shim from dereferencing the bogus buffer and avoids handing HALion the macOS `kInvalidArgument` value `2`, which the plugin immediately reused as a fake UTF-16 pointer in the next crash report.
- Hardened the maintainer-only raw `TUID*` host-callback handling so sentinel-like values such as `0xFFFFFFFF` are logged as `invalid_pointer(...)` rather than being dereferenced inside the probe logger, and `createInstance` can still succeed when HALion provides one valid IID and one bogus sentinel pointer.
- Hardened maintainer-only object-output pointer handling so invalid `obj` addresses such as `0x1` no longer panic the host during `createInstance` or `queryInterface`; `queryInterface` still returns `kNoInterface`, while `createInstance` now logs and returns a no-op `kResultOk` after the newest HALion rerun showed the plugin reusing `kNotImplemented == 3` as the bogus address `0x3`.
- Replaced the previous `FUnknown` alias treatment for the three HALion-only init-time GUIDs with dedicated opaque stub interfaces after `solo-compose-2026-04-12-134745.ips` showed a PC-alignment jump into Compose rodata, which is more consistent with a post-`FUnknown` virtual call than with another null-result crash.
- Follow-on planning now treats the private-GUID alias and opaque-stub experiments recorded here as diagnostic residue to replace or quarantine via real host-contract hardening, not as the intended long-term path to HALION support.

### File List

- `docs/architecture.md`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs`
- `_bmad-output/implementation-artifacts/9-2-extract-a-stable-backend-owned-vst3-host-core-from-the-poc.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Change Log

- 2026-04-11: Moved Story 9.2 to in-progress in sprint tracking.
- 2026-04-11: Extracted the shared backend-owned VST3 host core, added seam-local lifecycle regression coverage, documented the new owner seam, ran full validation, and moved the story to review.
- 2026-04-12: Used the retrieved HALion crash evidence to tighten the maintainer-only host-only controller-init profile so empty Steinberg wrapper-marker probes resolve through the host application's `FUnknown` surface and other unsupported init-time interface probes hand back a balanced non-null fallback `FUnknown` while still reporting `kNoInterface`.
- 2026-04-12: Confirmed that fallback path did not move the HALion crash site, then promoted the three observed HALion-only init-time GUID probes to the same investigation-only success-path alias used for the wrapper markers.
- 2026-04-12: Corrected the maintainer probe log classification so HALion-only GUID aliases report `supported_empty_marker_alias`, matching the actual host shim behavior instead of the stale wrapper-only diagnostic string.
- 2026-04-12: Confirmed the newest HALion rerun moved past the earlier plugin-side `0x40` wrapper-object crash and now reaches Compose's `IHostApplication::getName`; guarded the host shim so the observed invalid `0x60` name buffer returns `kInvalidArgument` instead of being dereferenced.
- 2026-04-12: Confirmed the next HALion rerun moved past the plugin-side UTF-16 `0x2` crash and into Compose's own `IHostApplication::createInstance` probe logging; guarded raw `TUID*` callback inputs so sentinel-like values such as `0xFFFFFFFF` are never dereferenced during maintainer-only logging or interface matching.
- 2026-04-12: Confirmed the latest HALion rerun moved past the earlier toxic `getName` and `createInstance` result-code crashes but then hit a PC-alignment `SIGBUS` inside Compose rodata at `src/playback/plugin_editor_poc.rs`; replaced the earlier empty-marker alias for the three HALion-only GUID probes with dedicated opaque stub interfaces that expose padded no-op slots for post-`FUnknown` virtual calls.
- 2026-04-12: Epic 9 follow-on planning now treats the private GUID alias and opaque-stub experiments as bounded diagnostic residue; Story 9.3 will replace or quarantine them through real host-owned contract hardening rather than extending them as the HALION fix path.
