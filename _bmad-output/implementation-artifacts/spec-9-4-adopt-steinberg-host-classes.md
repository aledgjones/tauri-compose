---
title: "Adopt Steinberg SDK host classes for the bounded VST3 host seam"
type: "refactor"
created: "2026-04-13"
status: "in-progress"
baseline_commit: "8db3c1f07ee8678fb78edba9417f0f567671bc3d"
context:
  [
    "docs/project-context.md",
    "docs/architecture.md",
    "docs/development-guide.md",
  ]
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The current bounded macOS VST3 host still hand-writes the host application, handler, and host-created message objects even though the lifecycle now mirrors Steinberg PlugProvider order. That leaves Compose on a thinner, less canonical host contract than MuseScore, which uses Steinberg hosting classes directly, and it keeps HALION remediation blocked on custom COM behavior that we should not maintain long term.

**Approach:** Replace the handwritten host-application support path with a bounded Steinberg-SDK-backed shim that owns HostApplication and PlugProvider-style setup for the ordinary product seam. Keep the existing backend-owned lifecycle, policy blocking, and macOS editor attachment behavior, but make Rust call into one narrow SDK-backed host bridge instead of continuing to author the host ABI surface by hand.

## Boundaries & Constraints

**Always:** Preserve the current backend-owned product path and Story 9.4 scope; keep the shared host core as the one owner of component/controller setup and teardown; prefer a narrow SDK-backed shim over guessed private HALION interfaces; preserve current blocked-target policy and current supported-plugin behavior; include documentation impact for any changed host architecture or maintainer workflow.

**Ask First:** If using Steinberg host classes requires vendoring new SDK source into the repo, introducing a new external binary dependency, or widening support claims beyond the current bounded posture, stop and ask before proceeding.

**Never:** Do not add fake-success private GUID implementations; do not keep parallel handwritten and SDK-backed ordinary host paths alive longer than needed; do not broaden multi-instrument, HALION-ready, or user-facing support claims as part of this refactor; do not move canonical playback or score state ownership into JavaScript.

## I/O & Edge-Case Matrix

| Scenario             | Input / State                                                          | Expected Output / Behavior                                                                                                                                                                                                           | Error Handling                                                               |
| -------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- |
| HAPPY_PATH           | macOS bounded editor-host session for an already-supported VST3 bundle | The host core builds the session through the SDK-backed host bridge, preserves current provider ordering, and the plugin reaches the same or better initialization point without Rust-authored host COM objects on the ordinary path | N/A                                                                          |
| BRIDGE_BUILD_FAILURE | build or runtime setup lacks the required Steinberg bridge inputs      | Compose fails explicitly at build-time or host-session construction with a truthful bounded error instead of silently falling back to the handwritten host ABI surface                                                               | Return a targeted host-contract failure and keep unsupported targets blocked |

</frozen-after-approval>

## Code Map

- `src-tauri/Cargo.toml` -- current Rust crate manifest; will need bridge/build dependencies if an SDK-backed shim is introduced.
- `src-tauri/build.rs` -- currently only runs `tauri_build::build()`; will need bounded C++/bridge compilation wiring.
- `src-tauri/src/playback/plugin_editor_poc.rs` -- owns the handwritten HostApplication, handler, createInstance path, and related tests that this refactor is meant to replace or shrink.
- `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` -- backend-owned session lifecycle seam that must remain the single owner while switching host implementation strategy.
- `docs/architecture.md` -- documents the host-core ownership model and needs an update if the host ABI implementation moves behind an SDK-backed bridge.
- `docs/development-guide.md` -- maintainer workflow doc that should explain any new bridge/build requirement for the bounded host seam.

## Tasks & Acceptance

**Execution:**

- [x] `src-tauri/Cargo.toml` and `src-tauri/build.rs` -- add the minimum build plumbing for a bounded Steinberg-SDK-backed host bridge -- the repo currently has only raw `vst3` bindings and no C++/bridge integration.
- [x] `src-tauri/src/playback/plugin_editor_poc.rs` -- replace or sharply reduce the handwritten HostApplication/createInstance implementation so the ordinary seam uses the SDK-backed host classes for host context and host-created support objects -- this is the core intent of the refactor.
- [x] `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` -- route session construction through the new bridge while preserving the existing backend-owned provider order, failure reporting, and teardown behavior -- lifecycle ownership must not split again.
- [x] `src-tauri/src/playback/plugin_editor_poc.rs` tests and any new bridge-local tests -- cover the SDK-backed host path, especially host object creation, lifecycle preservation, and failure behavior when bridge setup is unavailable -- the refactor changes the ABI implementation seam and needs local regression coverage.
- [x] `docs/architecture.md` and `docs/development-guide.md` -- document the new host implementation boundary, build requirements, and maintainer expectations -- docs impact is required for this behavior and architecture change.

**Acceptance Criteria:**

- Given a bounded macOS VST3 host session build, when Compose allocates the ordinary host context, then the session uses the SDK-backed Steinberg host classes instead of the handwritten Rust HostApplication/createInstance path.
- Given the shared host core already owns provider-style ordering, when the SDK-backed host bridge is introduced, then component/controller initialization order, failure reporting, and reverse teardown remain backend-owned in `vst3_host_core.rs` rather than moving into parallel logic.
- Given a supported plugin that currently works on the bounded path, when the new host bridge is used, then the plugin still mounts through the ordinary seam without regressing existing policy blocking or supported-path behavior.
- Given missing or invalid bridge prerequisites, when a bounded host session is attempted, then Compose returns a truthful host-contract failure instead of silently falling back to the handwritten ordinary host ABI implementation.
- Given the host implementation boundary has changed, when the work lands, then `docs/architecture.md` and `docs/development-guide.md` describe the SDK-backed seam and any maintainer-visible build implications.

## Spec Change Log

## Design Notes

The repo is already close to Steinberg PlugProvider order at the lifecycle level, so this spec intentionally does not redesign ownership again. The change is narrower: move the host ABI implementation behind Steinberg’s hosting classes where MuseScore relies on them, and keep Rust responsible for session orchestration, product policy, telemetry, and macOS view ownership.

The preferred shape is a bounded bridge layer rather than a broad rewrite:

```text
Rust host core -> narrow SDK-backed bridge -> Steinberg HostApplication/PlugProvider
```

That keeps the current `vst3_host_core.rs` seam as the product owner while removing the need for Compose to keep hand-authoring the ordinary host COM surface.

## Verification

**Commands:**

- `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` -- expected: host-seam tests pass on the new SDK-backed path.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` -- expected: bridge/build changes remain warning-free.
- `npm run check-types` -- expected: frontend type checks remain green after any command or payload adjustments.
- `npm run lint` -- expected: frontend lint remains green.

**Results:**

- `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` -- passed, `58 passed; 0 failed`.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` -- passed.
- `npm run check-types` -- not run; the refactor is limited to Rust/build/docs paths.
- `npm run lint` -- not run; the refactor is limited to Rust/build/docs paths.
