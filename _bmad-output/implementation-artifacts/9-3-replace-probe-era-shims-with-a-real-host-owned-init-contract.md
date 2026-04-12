# Story 9.3: Replace Probe-Era Shims With A Real Host-Owned Init Contract

Status: review

## Story

As a maintainer,
I want the HALion-class initialization path to rely on a real host-owned VST3 contract rather than maintainer-only success-path shims,
so that Compose can test and fix the remaining failure boundary through one truthful implementation seam.

## Acceptance Criteria

1. Given Story 9.2 extracted the shared host core, when Story 9.3 completes, then ordinary bounded editor-host opens and maintainer reruns use the same host-owned init contract for factory host context, component and controller initialization, component-handler timing, peer wiring, view preparation, and any newly implemented public host services, and no probe-era private-GUID success-path shim remains the main owner of HALion progress.
2. Given current maintainer probes still contain wrapper-marker aliases, opaque stubs, or toxic-result guards introduced during diagnosis, when the real host-owned contract is implemented, then superseded shim branches are removed or clearly quarantined behind maintainer-only diagnostics, and unknown or private init-time GUIDs default to truthful unsupported or logged behavior unless public evidence justifies a real host implementation.
3. Given the single-instrument baseline and blocked-bundle posture are already proven, when Story 9.3 lands, then the known-good single-instrument control path remains stable, and direct-root route emission, unsupported non-root activation, and current blocked user-facing posture remain unchanged.

## Tasks / Subtasks

- [x] Task 1: Make the shared host core own one truthful controller-init host contract (AC: 1, 3)
  - [x] Use the backend-owned host service graph from `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` for ordinary bounded editor-host opens and maintainer reruns during controller initialization.
  - [x] Keep lifecycle-profile timing differences bounded to observation or comparison work; do not let the maintainer profile swap in a fake-success controller-init host surface.
  - [x] Preserve current blocked-bundle posture, direct-root route emission, and unsupported non-root activation behavior.

- [x] Task 2: Remove or quarantine probe-era fake-success controller-init shims (AC: 1, 2)
  - [x] Delete or isolate private-GUID aliases, opaque stub surfaces, and non-null fallback return paths so they no longer influence the canonical host contract.
  - [x] Keep any remaining maintainer-only observation code limited to logging, comparison, or bounded public-contract experiments.
  - [x] Ensure unknown or private init-time GUIDs are recorded as diagnostic evidence rather than silently treated as supported.

- [x] Task 3: Extend seam-local regression coverage around truthful host behavior (AC: 1, 2, 3)
  - [x] Replace tests that assert opaque-stub or fallback success-path behavior with tests that assert shared host-contract usage and truthful unsupported responses.
  - [x] Keep lifecycle-profile and host-service coverage close to `src-tauri/src/playback/plugin_editor_poc.rs` and `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs`.
  - [x] Preserve regression coverage for the supported single-instrument control path and unchanged blocked-bundle posture.

- [x] Task 4: Update documentation and tracking artifacts with the hardened-contract rule (AC: 2, 3)
  - [x] Update `docs/architecture.md` to describe the real host-owned init contract and the cleanup-in-place removal or quarantine of probe-era fake-success shims.
  - [x] Update `docs/development-guide.md` to describe the reduced maintainer comparison profile and the existing automated rerun flow without implying private-GUID or opaque-stub success paths are the intended direction.
  - [x] Update `docs/playback-support-baseline.md` only as far as the hardened-contract work changes the current maintainer evidence wording while keeping user-facing support posture blocked.

## Dev Notes

### Story Intent

- Story 9.3 is the approved follow-on to Story 9.2 and the 2026-04-12 Epic 9 course correction.
- The goal is not to prove multi-instrument support. The goal is to harden the real public-contract host seam so HALion-class failures are exercised through one truthful backend-owned path.
- HALION remains the acceptance bar, but current user-facing support posture stays blocked until later proof explicitly changes it.

### This Story Owns

- Making `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` the effective owner of one truthful controller-init host contract.
- Removing or clearly quarantining the probe-era wrapper-marker aliases, opaque stubs, and non-null fallback behavior that were introduced only for diagnosis.
- Keeping maintainer diagnostics backend-owned and on the same host-core seam rather than inventing a second lifecycle owner.
- Updating docs and tracking artifacts in the same change.

### This Story Does Not Own

- Reclassifying any blocked bundle as supported.
- Broadening route-choice behavior, `PluginPart`, `MidiChannel`, shared-slot workflows, or multi-instrument UX.
- Claiming that unknown or private init-time GUIDs are supported without public evidence.
- Publishing the post-hardening HALION verdict; that belongs to Story 9.4.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs`
  - Canonical lifecycle owner for factory host context, component and controller initialization, handler timing, connection-point wiring, view ownership, and deterministic teardown.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Current home of the maintainer-only controller-init wrapper, probe logging, and probe-era fake-success residue that must be removed or isolated.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Backend-owned maintainer evidence payload storage.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Ordinary blocked-bundle posture that must remain unchanged in this story.

### Implementation Guidance

- Start from the real public host service graph already exposed by `PluginHostApplication` and the shared host core.
- Prefer one truthful controller-init host surface for ordinary and maintainer flows.
- If a maintainer-only comparison profile remains, keep it limited to public-contract timing or observation differences rather than interface-shape fakery.
- Unknown or private init-time interface requests should be logged and treated as unsupported by default.
- Clean up in place: do not leave dead fake-success branches active beside the hardened seam.

### Risks And Regression Traps

- Leaving the default controller-init path on a diagnostics wrapper that still alters interface results.
- Removing maintainer logging entirely instead of reducing it to truthful observation.
- Accidentally widening product support claims or blocked-bundle posture while hardening the host contract.
- Reintroducing a second lifecycle owner beside the extracted host core.

### Testing Requirements

- Add or update seam-local Rust tests that prove:
  - the default product path and maintainer comparison path share the same truthful controller-init host contract;
  - unknown or private init-time GUIDs are logged or treated as unsupported rather than reported as supported through fake-success surfaces;
  - the supported single-instrument path and blocked-bundle posture remain unchanged.
- Recommended validation commands after implementation:
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib -- --nocapture`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_host_compatibility --lib -- --nocapture`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- If runtime truth is needed, rerun the existing automated probe flow rather than relying on prior terminal history.

### Documentation Impact

- Update `docs/architecture.md` if the host service graph or lifecycle ownership wording changes materially.
- Update `docs/development-guide.md` so maintainer guidance stops describing private-GUID aliases, opaque stubs, or fallback `FUnknown` behavior as the current path.
- Update `docs/playback-support-baseline.md` only for evidence wording that changes because of the hardened contract work.

### References

- `_bmad-output/planning-artifacts/epics.md`
- `_bmad-output/planning-artifacts/sprint-change-proposal-2026-04-12.md`
- `_bmad-output/implementation-artifacts/tech-spec-halion-host-contract-hardening.md`
- `_bmad-output/implementation-artifacts/9-2-extract-a-stable-backend-owned-vst3-host-core-from-the-poc.md`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/playback-support-baseline.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-12: `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture`, `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib -- --nocapture`, `cargo test --manifest-path src-tauri/Cargo.toml plugin_host_compatibility --lib -- --nocapture`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `npm run check-types`, and `npm run lint` all passed after the host-contract hardening edits.
- 2026-04-12: Maintainer automation reruns through `plugin.editorHost.prepareLoadInitProbeLog` and `plugin.editorHost.probeLoadInit` kept `ProEQ.vst3` at `load_init_safe` on the default shared contract, while `HALion Sonic.vst3` still crashed during `controller_initialize` on both probe profiles after truthful unsupported wrapper/private-interface queries were logged.

### Completion Notes List

- The shared host core now owns one truthful controller-init host contract for both ordinary bounded opens and maintainer reruns; the legacy-named comparison profile only changes timing, not interface shape.
- Probe-era private-GUID aliases, HALION-only opaque stubs, non-null fallback `FUnknown` returns, and fake-success invalid-pointer paths were removed from the canonical controller-init seam.
- Seam-local regression coverage now asserts truthful unsupported responses and the shared host-contract model instead of the old shim behavior.
- Post-hardening maintainer reruns preserved the known-good control path (`ProEQ.vst3` reached `load_init_safe`) while keeping HALION blocked after crashes during `controller_initialize` on both profiles.
- No `.github/copilot-instructions.md` change was needed because the durable no-fake-success host-contract rule was already captured there.

### File List

- `_bmad-output/implementation-artifacts/9-3-replace-probe-era-shims-with-a-real-host-owned-init-contract.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/playback-support-baseline.md`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs`

### Change Log

- 2026-04-12: Created the Story 9.3 implementation artifact from the approved Epic 9 story and tech spec, and marked it in-progress for development.
- 2026-04-12: Hardened the shared controller-init host contract, removed probe-era fake-success paths, updated seam-local tests and maintainer docs, and reran the existing automated probe flow on `ProEQ.vst3` and `HALion Sonic.vst3`.
