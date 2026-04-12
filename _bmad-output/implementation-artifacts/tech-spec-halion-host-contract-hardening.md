---
title: "HALION Host Contract Hardening"
type: "implementation-spec"
created: "2026-04-12"
status: "approved"
context:
  - "../../docs/project-context.md"
  - "../../docs/architecture.md"
  - "../../docs/development-guide.md"
  - "../planning-artifacts/epics.md"
  - "../planning-artifacts/sprint-change-proposal-2026-04-12.md"
  - "tech-spec-investigate-multi-instrument-vst-support-with-halion.md"
---

## Intent

Story 9.2 extracted a real backend-owned host core and the 2026-04-12 HALION reruns removed several genuine host bugs, but the remaining failure boundary no longer looks like a timing-only or return-code-only problem. The maintainer-only private-GUID aliases, opaque stubs, and other success-path shims were useful diagnostics, yet they are not converging on a credible long-term fix.

The next slice must therefore harden the real host-owned VST3 contract that HALION-class plugins reasonably expect from public Steinberg guidance and comparator hosts. HALION remains the acceptance bar. Ordinary product support remains blocked until the hardened seam produces stronger evidence.

## Boundaries And Constraints

**Always:**

- Keep `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` as the canonical lifecycle owner.
- Preserve current blocked-bundle posture, direct-root route emission, unsupported non-root activation, and no shared-slot or multi-instrument product claim.
- Prefer public Steinberg-visible contracts and established host prior art over guessed private-interface behavior.
- Clean up in place: remove or quarantine superseded probe-era shim branches instead of leaving parallel ownership paths behind.

**Ask First:**

- If a proposed fix would broaden user-facing support claims, route-choice behavior, or capability status beyond bounded load/init hardening.
- If a new private or vendor-specific interface appears necessary without public evidence or a strong comparator-host basis.

**Never:**

- Do not treat guessed private-GUID aliases, opaque success stubs, or fake-success return paths as the main implementation path.
- Do not broaden this slice into part/channel capability probing, shared-slot workflows, or multi-instrument UX.

## Focused Outcome

The implementation should leave Compose with one truthful host-owned initialization seam that:

- owns the init-time host service graph beside the extracted host core;
- serves public host behavior through real lifecycle and service objects rather than maintainer-only shims;
- keeps unknown or private init-time GUIDs as logged diagnostic evidence unless public evidence justifies real implementation;
- reruns HALION and one known-good control bundle through the same seam to prove either bounded non-crashing initialization or a narrower remaining blocker.

## Code Map

- `../../src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` - canonical lifecycle owner and the primary seam for real init-contract hardening.
- `../../src-tauri/src/playback/plugin_editor_poc.rs` - current maintainer-only probe profile, callback logging, and probe-era residue that must be cleaned up or quarantined.
- `../../src-tauri/src/playback/plugin_inventory.rs` - backend-owned validation payloads that record maintainer probe evidence.
- `../../src-tauri/src/automation/registry.rs` and `../../src-tauri/src/automation/mcp.rs` - automated probe entry points that must keep reusing the same backend-owned seam.
- `../../docs/architecture.md` - durable host-ownership and seam-boundary documentation.
- `../../docs/development-guide.md` - maintainer workflow and automated HALION rerun guidance.
- `../../docs/playback-support-baseline.md` - support posture and blocked-reason evidence after the rerun.

## Tasks And Acceptance

**Execution:**

- [ ] `../../src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` and adjacent owner files - promote a real host-owned init contract around the extracted host core, including the public host services and lifecycle behavior that ordinary bounded editor-host opens and maintainer probes should truly share. This is the seam that should grow; do not create a new parallel host owner.
- [ ] `../../src-tauri/src/playback/plugin_editor_poc.rs` - reduce the maintainer-only controller-init profile to observation, comparison, and bounded public-contract experiments. Remove or quarantine superseded private-GUID aliases, opaque stubs, and other fake-success branches that no longer align with the real host path.
- [ ] `../../src-tauri/src/playback/plugin_inventory.rs` and the existing automation entry points - keep failure classification and saved maintainer evidence backend-owned while reusing the hardened host core; do not invent a second diagnostics model or a probe-only lifecycle.
- [ ] HALION rerun through the existing automated probe flow - validate one currently working single-instrument control bundle plus HALION Sonic against the hardened seam, and record whether the remaining blocker is cleared or narrowed.
- [ ] Documentation and planning artifacts - update `../../docs/architecture.md`, `../../docs/development-guide.md`, `../../docs/playback-support-baseline.md`, `../planning-artifacts/epics.md`, `sprint-status.yaml`, and `../../.github/copilot-instructions.md` in the same change so the new implementation rule is explicit.

**Acceptance Criteria:**

1. Given Story 9.2 extracted the backend-owned host core, when this hardening slice lands, then init-time host services used by both ordinary bounded editor-host opens and maintainer probes come from the real host core rather than probe-era private-GUID success paths or parallel lifecycle owners.
2. Given HALION-class plugins still query unknown or private init-time interfaces, when the hardened seam handles them, then Compose logs them and preserves truthful unsupported behavior by default, and any still-retained diagnostic residue is clearly isolated from ordinary product behavior.
3. Given one known-good single-instrument control bundle and HALION are rerun through the shared automated probe flow, when results are recorded, then the control bundle remains stable and HALION either reaches a bounded non-crashing load/init state or yields a narrower remaining blocker rooted in the hardened public-contract seam.
4. Given this work changes host architecture, maintainer workflow, and developer expectations, when it completes, then the relevant docs and AI-agent instructions are updated in the same change and current user-facing support posture remains blocked until later proof explicitly changes it.

## Design Notes

- Start from public-contract gaps and comparator-host structure, not from HALION-private GUID behavior.
- Treat unknown/private init-time GUIDs as evidence to compare against the public contract. They may justify later work only if public or external evidence explains what real host-owned behavior they correspond to.
- Use the existing automated probe path and persisted logs as the verification harness. Do not add a separate scanner or compatibility lane in this slice.
- Preserve cleanup-in-place: when a real host-owned seam replaces a probe-era branch, remove the superseded branch or explicitly quarantine it behind maintainer-only diagnostics.

## Verification

**Commands:**

- `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml plugin_host_compatibility --lib -- --nocapture`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- `npm run check-types`
- `npm run lint`

**Maintainer runbook:**

1. Start Compose with `npm run start:dev`.
2. Prepare the persisted probe log through the Debugger or `compose_action_plugin_editorHost_prepareLoadInitProbeLog`.
3. Run HALION Sonic through `compose_action_plugin_editorHost_probeLoadInit` and compare it with one known-good single-instrument control bundle.
4. Read the persisted log through `compose_action_plugin_editorHost_readProbeLog` after each rerun, and inspect the latest macOS crash report only if bounded non-crashing init is still not reached.

## Documentation Impact

- Update `../../docs/architecture.md` if the host service graph or backend ownership boundary changes materially.
- Update `../../docs/development-guide.md` so the maintainer flow stops implying private-GUID success-path shims are the intended direction.
- Update `../../docs/playback-support-baseline.md` when the HALION rerun changes the blocked reason or reaches `load_init_safe`.
- Keep `../planning-artifacts/epics.md` and `sprint-status.yaml` aligned with the host-hardening sequence in the same pass.
- Update `../../.github/copilot-instructions.md` whenever the repo's required host-implementation practice changes, because that rule is now durable.
