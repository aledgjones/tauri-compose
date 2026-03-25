# Story 1.3: Remove SCVI Tooling and Build Workflow Residue

Status: done

## Story

As a maintainer,
I want the repository to stop carrying SCVI-specific scripts and build-time support paths,
so that future playback work proceeds on one clear implementation path instead of splitting effort across live and dead tooling.

## Acceptance Criteria

1. Given the repository contains SCVI-specific build, inspect, compression, or support scripts, when this story is complete, then those SCVI-specific scripts are removed and no active build or developer workflow depends on them.
2. Given package scripts or internal workflow entry points reference SCVI-specific tooling, when the cleanup is finished, then those references are removed or updated and the remaining workflow surface reflects the VST-first direction only.
3. Given a maintainer reviews the repository's active tooling after this story, when they inspect package scripts, developer entry points, and playback-related support scripts, then they do not find SCVI-specific operational paths presented as current workflows and the repo remains buildable with the remaining supported toolchain.

## Tasks / Subtasks

- [x] Task 1: Remove SCVI-specific workflow entry points from the active toolchain surface (AC: 1, 2, 3)
  - [x] Remove the SCVI package scripts from `package.json`, specifically the `generate-scvi`, `generate-scvi:build`, `inspect-scvi`, and `inspect-scvi:build` entries.
  - [x] Remove the SCVI binary declarations from `src-tauri/Cargo.toml` so the active Cargo manifest no longer advertises SCVI tooling binaries.
  - [x] Retire the generated script entry points in `scripts/scvi-compress` and `scripts/scvi-inspect` if they are no longer part of any supported workflow.
- [x] Task 2: Remove SCVI-only source binaries and any compile-time support path that exists solely for the retired tooling workflow (AC: 1, 2, 3)
  - [x] Remove `src-tauri/src/bin/scvi_compress.rs` and `src-tauri/src/bin/scvi_inspect.rs` if no supported workflow still requires them.
  - [x] Audit `src-tauri/src/playback/mod.rs`, `src-tauri/src/playback/scvi.rs`, and `src-tauri/src/playback/sampler.rs` for compile-time or maintainer-facing dependencies on the retired SCVI container tooling path.
  - [x] If a remaining backend seam still depends on `.scvi` container parsing only to support the retired tooling workflow, remove or replace that dependency rather than keeping dead SCVI support paths compiled by default.
- [x] Task 3: Keep the remaining playback and maintainer seams coherent after tooling removal (AC: 2, 3)
  - [x] Preserve the current supported VST-first runtime direction established by Story 1.1 and the user-facing cleanup direction established by Story 1.2.
  - [x] Do not reintroduce SCVI fallback logic, compatibility shims, or hidden operational paths merely to keep old tooling code alive.
  - [x] If legacy sampler or debugger seams still need bounded compile-safe state during the transition, keep that state clearly scoped to legacy audition/tooling behavior and not to the supported playback workflow.
- [x] Task 4: Update focused developer guidance that becomes inaccurate immediately because of the tooling removal (AC: 2, 3)
  - [x] Remove or rewrite the SCVI tooling command table and sample attack alignment workflow guidance in `docs/development-guide.md` if those commands are retired by this story.
  - [x] Update `docs/project-context.md` if it still instructs agents or maintainers to treat SCVI CLI binaries or SCVI container tooling as active developer workflow.
  - [x] Update `docs/architecture.md` only where the current architecture description would otherwise claim active SCVI tooling or support binaries that no longer exist.
  - [x] Leave the broad repo-wide SCVI documentation retirement sweep to Story 1.4.
- [x] Task 5: Add validation close to the affected build and tooling seams (AC: 3)
  - [x] Run `npm run lint`.
  - [x] Run `npm run check-types`.
  - [x] Run `npm run build`.
  - [x] Run `npm run test:js`.
  - [x] Run `npm run test:rust`.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` if Rust files or Cargo target declarations change.

## Dev Notes

### Story Intent

- Story 1.3 is repository and build-workflow cleanup, not a broad product redesign.
- The required outcome is that maintainers no longer see SCVI build, inspect, or compression tooling presented as part of the active Compose workflow.
- This story should remove the retired workflow at the root, not merely relabel it.
- The cleanup must preserve one clear direction: VST-first playback with the remaining supported toolchain only.

### Scope Boundaries

- In scope:
  - `package.json` scripts and any generated script entry points that present SCVI tooling as current.
  - Cargo binary declarations and Rust source binaries dedicated to SCVI compression or inspection.
  - Compile-time dependencies or support paths that exist only to keep retired SCVI tooling workflows alive.
  - Narrow developer-facing documentation and agent guidance that would become immediately inaccurate after the tooling cleanup.
- Out of scope:
  - Broad user-facing workflow cleanup. Story 1.2 owns that and is already ahead of this story.
  - Broad documentation retirement across the entire repo. Story 1.4 owns that full sweep.
  - Repo-wide final verification of every residual SCVI reference. Story 1.5 owns that verification pass.
  - Migration work, compatibility preservation, or any attempt to keep SCVI as a supported secondary path.

### Previous Story Intelligence

- Story 1.1 disconnected SCVI from the supported transport runtime and explicitly left SCVI build scripts, inspection commands, and repo tooling cleanup to Story 1.3.
- Story 1.2 cleaned up user-facing workflow language and explicitly left backend sampler/SCVI module retirement, CLI removal, and build-script cleanup to Story 1.3 unless a minimal compile-safety change was required.
- The repo is therefore in an intentionally transitional state: supported playback is no longer SCVI-owned, but SCVI tooling and container-oriented seams still exist in maintainer-facing and build-facing areas.
- Story 1.3 should resolve that mismatch rather than leaving dead workflow branches in place.

### Architecture Compliance

- Preserve the existing brownfield Tauri + React + TypeScript + Vite foundation.
- Rust remains the source of truth for backend state and runtime behavior; frontend surfaces should not invent a second tooling truth while this cleanup happens.
- Keep active build and developer workflow surfaces aligned to the supported VST-first direction only.
- If removing SCVI tooling exposes a dead backend seam, prefer deleting or isolating that seam over preserving a second operational path.
- Do not add new abstractions just to preserve the shape of a retired SCVI workflow.

### Current Tooling And Build Residue

- `package.json`
  - Active npm scripts still expose `generate-scvi`, `generate-scvi:build`, `inspect-scvi`, and `inspect-scvi:build`.
- `src-tauri/Cargo.toml`
  - Active binary targets still include `scvi-compress` and `scvi-inspect`.
- `scripts/`
  - The repository still carries `scvi-compress` and `scvi-inspect` developer entry points.
- `src-tauri/src/bin/`
  - `scvi_compress.rs` and `scvi_inspect.rs` remain in the active source tree as SCVI-only binaries.
- `src-tauri/src/playback/mod.rs`
  - Still declares `pub mod scvi;`.
- `src-tauri/src/playback/sampler.rs`
  - Still imports `ScviContainer`, scans `/Library/Audio/Plug-Ins/SCVI`, opens `.scvi` containers, and builds cache metadata from them.
- `docs/development-guide.md`
  - Still documents SCVI generation and inspection commands as current maintainer workflow.
- `docs/project-context.md`
  - Still describes SCVI CLI binaries and SCVI container format as active tooling guidance.

### Implementation Guidance

- Start with active workflow surfaces first:
  - Remove the npm and Cargo entry points that advertise SCVI tooling as current.
  - Remove the dedicated SCVI binaries and generated script wrappers if nothing supported still depends on them.
- Then audit backend compile dependencies:
  - If `playback/scvi.rs` and `playback/sampler.rs` only survive to support the retired SCVI tooling path, remove or refactor them in the same story.
  - If a legacy sampler seam must remain temporarily for bounded maintainership or audition behavior, make sure it no longer implies a supported SCVI build or operational workflow.
- Keep the outcome unambiguous:
  - No SCVI build script.
  - No SCVI inspect script.
  - No SCVI binary targets in the active manifest.
  - No documentation telling maintainers to use retired SCVI tooling as part of current development.
- Avoid partial cleanup that leaves dead references scattered across package scripts, Cargo bins, generated wrappers, and docs.

### File Structure Requirements

- Primary file targets:
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `scripts/scvi-compress`
  - `scripts/scvi-inspect`
  - `src-tauri/src/bin/scvi_compress.rs`
  - `src-tauri/src/bin/scvi_inspect.rs`
- Likely backend fallout targets if compile dependencies remain:
  - `src-tauri/src/playback/mod.rs`
  - `src-tauri/src/playback/scvi.rs`
  - `src-tauri/src/playback/sampler.rs`
  - `src-tauri/src/lib.rs`
- Likely focused documentation targets:
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `docs/architecture.md`
- Keep frontend fallout minimal and localized; only touch `src/services/actions.ts`, `src/services/state/context.tsx`, debugger surfaces, or related tests if removing SCVI support paths breaks compile safety.

### Testing Requirements

- Minimum validation target:
  - Active npm/Cargo workflow no longer exposes SCVI tooling commands or SCVI binary targets.
  - The repo still type-checks, builds, and passes the relevant JS and Rust checks.
  - Any Rust target cleanup remains clippy-clean.
- Prefer these checks:
  - `npm run lint`
  - `npm run check-types`
  - `npm run build`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` when Rust/Cargo targets change
- Do not create a new repo-wide harness for this cleanup.

### Library / Framework Requirements

- Preserve the current pinned stack and treat this as repo-local cleanup rather than a dependency-upgrade story.
- Current relevant versions:
  - React `19.2.4`
  - TypeScript `5.9.3`
  - Vite `7.3.1`
  - Tauri frontend API `2.10.1`
  - Tauri Rust crate `2.10.3`
  - `vst3 = 0.3.0`
- No external API migration or latest-version adoption is required for this story; focus on removing retired SCVI tooling from the current stack.

### Documentation Impact

- This story must update the narrow developer docs that would become wrong as soon as SCVI tooling is removed.
- Minimum expected docs review:
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `docs/architecture.md` if it still presents removed tooling binaries or container workflow as active
- If implementation changes project rules for agents or maintainers beyond those docs, update the relevant AI-agent instructions in the same change.
- Do not broaden this into the full repo-wide SCVI wording sweep; Story 1.4 owns that.

### Git Intelligence Summary

- Recent branch history confirms the cleanup sequence already underway:
  - `feat(1.1): scvi cleanup part 1`
  - `chore: spec vst work`
- Story 1.3 should extend that direction by removing the retired tooling layer, not by preserving a second operational branch for legacy containers.

### Project Structure Notes

- The planning and developer docs still describe SCVI container binaries and SCVI-backed sampler seams as if they are part of the active maintainer workflow.
- That is now architectural residue, not a supported product direction.
- Story 1.3 should either remove the dead seams or leave them clearly bounded as non-workflow legacy internals with no exposed build or developer entry point.
- Story 1.5 will perform the final repo-wide verification sweep, so this story should focus on concrete operational residue rather than trying to classify every historical mention in one pass.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 1, Story 1.3 acceptance criteria
- `_bmad-output/planning-artifacts/prd.md` — SCVI should be removed rather than carried forward; VST-first direction is the active product goal
- `_bmad-output/planning-artifacts/architecture.md` — preserved brownfield foundation, build tooling expectations, and existing playback module structure
- `_bmad-output/implementation-artifacts/1-1-remove-scvi-from-the-active-playback-runtime.md` — runtime cleanup boundaries and explicit deferral of tooling cleanup to Story 1.3
- `_bmad-output/implementation-artifacts/1-2-remove-scvi-from-user-facing-playback-workflow.md` — user-facing cleanup boundaries and explicit deferral of CLI/build cleanup to Story 1.3
- `package.json` — active npm workflow still exposes SCVI build and inspect commands
- `src-tauri/Cargo.toml` — active Cargo manifest still declares `scvi-compress` and `scvi-inspect`
- `scripts/scvi-compress` and `scripts/scvi-inspect` — generated developer entry points for retired SCVI tooling
- `src-tauri/src/bin/scvi_compress.rs` and `src-tauri/src/bin/scvi_inspect.rs` — SCVI-only Rust binaries
- `src-tauri/src/playback/mod.rs` — still declares the SCVI playback module
- `src-tauri/src/playback/sampler.rs` — current `.scvi` scan/load/audition seam and `/Library/Audio/Plug-Ins/SCVI` dependency
- `docs/development-guide.md` — still documents SCVI tooling commands and container inspection workflow
- `docs/project-context.md` — still treats SCVI container tooling as part of current agent guidance

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story target provided explicitly by user as `1.3`.
- Sprint tracker showed `1-3-remove-scvi-tooling-and-build-workflow-residue` in `backlog` before story creation.
- Context grounded in Epic 1 planning docs, PRD, architecture, UX spec, project-context rules, Story 1.1 and Story 1.2 outputs, current package/Cargo manifests, and the active SCVI-related source tree.
- Read-only exploration confirmed that the active SCVI tooling surface is concentrated in npm scripts, Cargo bin targets, generated script wrappers, dedicated Rust binaries, the SCVI playback module declaration, the sampler container-loading seam, and focused developer docs.
- No external web research was required because this story is repo-local tooling retirement against the current pinned stack, not a dependency-upgrade or API-migration task.
- Implementation removed the SCVI npm scripts, Cargo bin targets, generated wrapper executables, and dedicated Rust CLI binaries.
- Backend audit confirmed `.scvi` parsing still exists through the legacy sampler/audition seam, so the remaining SCVI reader was kept bounded as runtime-only legacy infrastructure instead of an active build workflow.
- Validation results: `npm run lint`, `npm run check-types`, `npm run build`, `npm run test:js`, and `npm run test:rust` passed.
- Follow-up implementation cleared the remaining clippy warnings in `src/automation/registry.rs`, `src/instructions/parse/get_horizontal_spacing.rs`, and `src/playback/plugin_editor_poc.rs`, and the repo now passes `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`.

### Completion Notes List

- Story context created with explicit scope boundaries between runtime cleanup (Story 1.1), user-facing workflow cleanup (Story 1.2), tooling cleanup (Story 1.3), docs sweep (Story 1.4), and final verification (Story 1.5).
- Story instructs the dev to remove active SCVI workflow entry points at the root rather than merely relabeling them.
- Story highlights the likely hidden root-cause dependency: `sampler.rs` still loads `.scvi` containers and may block full tooling retirement unless removed or isolated.
- Story includes narrow documentation impact requirements so removed tooling is not left documented as current maintainer workflow.
- Story keeps the validation bar explicit: lint, typecheck, build, JS tests, Rust tests, and clippy when Rust/Cargo targets change.
- Removed SCVI workflow entry points from `package.json`, `src-tauri/Cargo.toml`, `.vscode/settings.json`, `scripts/`, and `src-tauri/src/bin/` so the active repo toolchain no longer exposes SCVI compression or inspection commands.
- Reframed the remaining SCVI container reader and documentation as legacy sampler/audition runtime support only, preserving the VST-first supported path while avoiding new fallback logic.
- Rewrote the focused developer docs to remove SCVI command guidance and retain only bounded legacy attack-alignment/runtime notes.
- Validation passed for lint, typecheck, build, JS tests, Rust tests, and clippy.
- Final cleanup also fixed the remaining repo clippy blockers uncovered by the validation pass so Story 1.3 can move to review cleanly.

### File List

- `.vscode/settings.json`
- `_bmad-output/implementation-artifacts/1-3-remove-scvi-tooling-and-build-workflow-residue.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `package.json`
- `scripts/scvi-compress` (deleted)
- `scripts/scvi-inspect` (deleted)
- `src-tauri/src/automation/registry.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/src/bin/scvi_compress.rs` (deleted)
- `src-tauri/src/bin/scvi_inspect.rs` (deleted)
- `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/scvi.rs`

### Change Log

- 2026-03-20: Created Story 1.3 context file with concrete tooling targets, compile-dependency guardrails, validation commands, and documentation impact.
- 2026-03-20: Removed SCVI CLI/build workflow residue from package scripts, Cargo targets, generated wrappers, dedicated Rust binaries, and focused maintainer docs; validation passed except for unrelated pre-existing clippy failures outside the story seam.
- 2026-03-20: Cleared the remaining clippy warnings surfaced during validation, reran the full validation suite, and moved Story 1.3 to review.
