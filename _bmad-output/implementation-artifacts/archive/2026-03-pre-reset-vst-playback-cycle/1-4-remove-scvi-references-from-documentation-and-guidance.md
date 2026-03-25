# Story 1.4: Remove SCVI References From Documentation and Guidance

Status: done

## Story

As a maintainer,
I want the project documentation to stop describing SCVI as a current playback path,
so that contributors and reviewers see one unambiguous playback direction when working in the product.

## Acceptance Criteria

1. Given project documentation or developer guidance still describes SCVI as an active playback path, when this story is complete, then those references are removed or updated and the docs describe the intended VST-first direction instead.
2. Given docs previously pointed contributors to SCVI-specific scripts or workflows, when documentation cleanup is complete, then those instructions no longer appear as current guidance and no broken references remain to retired SCVI tooling.
3. Given this epic is a clean retirement rather than a compatibility transition, when the documentation is updated, then it does not introduce migration or coexistence guidance for SCVI and it treats SCVI as retired from the current product direction.

## Tasks / Subtasks

- [x] Task 1: Remove or reframe documentation that still presents SCVI as a current product capability (AC: 1, 3)
  - [x] Update `docs/project-overview.md` so the executive summary and feature inventory no longer describe the SCVI sampler engine as the current playback model.
  - [x] Rephrase any remaining high-level product wording so it points to the intended VST-first playback direction without falsely claiming broader shipped playback capability than the current product actually has.
  - [x] Do not add migration, coexistence, or fallback language that implies SCVI remains a supported path.
- [x] Task 2: Reconcile source-tree and developer-guidance docs with the post-Story-1.3 repository state (AC: 1, 2, 3)
  - [x] Update `docs/source-tree-analysis.md` so it no longer lists removed SCVI CLI binaries or wrapper scripts as current repo contents.
  - [x] If `src-tauri/src/playback/scvi.rs` still needs to appear in source-tree documentation, label it explicitly as a legacy runtime-only container reader rather than an active workflow/tooling surface.
  - [x] Review `docs/development-guide.md`, `docs/architecture.md`, and `docs/project-context.md` for wording that still implies active SCVI workflow or tooling, and update only the passages that remain misleading after Story 1.3.
- [x] Task 3: Preserve necessary legacy implementation context without presenting SCVI as supported workflow (AC: 1, 3)
  - [x] Keep bounded explanations of legacy `.scvi` runtime-only seams only where they remain necessary for maintainer understanding, such as attack-offset metadata flow or the retained container reader.
  - [x] Make sure any retained SCVI mention is clearly framed as legacy, runtime-only, or historical context rather than current product direction.
  - [x] Prefer removing stale references entirely over keeping explanatory text that no longer maps to an existing file, script, or workflow.
- [x] Task 4: Verify documentation no longer points contributors at retired SCVI tooling (AC: 2, 3)
  - [x] Run a targeted repository search across documentation and guidance files for `SCVI` / `scvi` and review every remaining hit.
  - [x] Confirm no current doc tells contributors to use retired entries such as `scripts/scvi-compress`, `scripts/scvi-inspect`, `src-tauri/src/bin/scvi_compress.rs`, or `src-tauri/src/bin/scvi_inspect.rs`.
  - [x] Leave the broad repo-wide final retirement audit and classification of intentionally historical mentions to Story 1.5.
- [x] Task 5: Validate the documentation cleanup outcome at the owning seam (AC: 1, 2, 3)
  - [x] Re-run the targeted docs/guidance SCVI search after edits and confirm the remaining hits are either intentionally legacy/runtime-only or part of future-story verification scope.
  - [x] Manually verify that every file, script, binary, or workflow named in updated documentation still exists and is described accurately.
  - [x] If implementation changes any code, commands, or agent instructions beyond documentation, run the relevant project validation commands for those touched seams; otherwise keep validation scoped to documentation accuracy.

## Dev Notes

### Story Intent

- Story 1.4 is the broad documentation and guidance retirement pass for SCVI.
- The required outcome is that contributors, reviewers, and agents reading the repo docs see one clear playback direction: VST-first, with any remaining SCVI references bounded as legacy internals only.
- This story is about documentation truth, not new runtime or tooling cleanup.
- The documentation should reflect a clean retirement, not a compatibility transition.

### Scope Boundaries

- In scope:
  - Project documentation and developer guidance that still presents SCVI as current product behavior or current maintainer workflow.
  - Stale source-tree or architecture descriptions that still point to removed SCVI scripts, binaries, or workflows.
  - Narrow agent/developer guidance updates if the existing instructions remain misleading after documentation cleanup.
- Out of scope:
  - Runtime removal or transport-path changes. Story 1.1 owns that.
  - User-facing workflow cleanup in product surfaces. Story 1.2 owns that.
  - CLI/build workflow removal. Story 1.3 owns that and is already complete.
  - Final repo-wide verification and classification of every residual SCVI mention. Story 1.5 owns that sweep.
  - Migration work, coexistence guidance, or any attempt to preserve SCVI as a supported alternate path.

### Previous Story Intelligence

- Story 1.1 removed SCVI from the supported playback runtime and made `transport_play` timing-only on the product path.
- Story 1.2 removed SCVI from user-facing workflow surfaces and kept any remaining sampler seams framed as legacy audition/tooling context rather than supported playback truth.
- Story 1.3 removed the SCVI CLI/build workflow surface from `package.json`, `src-tauri/Cargo.toml`, `scripts/`, and `src-tauri/src/bin/`, while keeping only bounded runtime-only legacy reader support where still needed.
- Story 1.4 should therefore clean up the repo narrative that still lags behind those completed changes, without broadening into another runtime/tooling refactor.

### Architecture Compliance

- Preserve the existing brownfield Tauri + React + TypeScript + Vite foundation.
- Keep Rust as the source of truth for runtime behavior and frontend as the display/control layer; this story should not invent new product claims that outrun the actual backend seams.
- Describe the supported playback direction as VST-first, but stay precise about current implementation status where the product is still foundation-building.
- If legacy SCVI runtime-only seams remain documented, keep them clearly bounded as maintainer-facing internals, not supported workflow.
- Do not introduce migration, fallback, or coexistence guidance for SCVI.

### Current Documentation Residue

- `docs/project-overview.md`
  - Still presents the "Sampler Engine" as a custom SCVI binary container playback engine in the high-level feature list.
  - This is the clearest remaining doc that presents SCVI as current product capability.
- `docs/source-tree-analysis.md`
  - Still lists removed SCVI CLI binaries (`src-tauri/src/bin/scvi_compress.rs`, `src-tauri/src/bin/scvi_inspect.rs`) and removed wrapper scripts (`scripts/scvi-compress`, `scripts/scvi-inspect`) as if they still exist.
  - Still describes `src-tauri/src/playback/scvi.rs` without clarifying that it is a legacy runtime-only container reader.
- `docs/development-guide.md`
  - Already says Compose no longer ships supported SCVI build or inspection commands; review for any remaining wording that still implies active workflow.
- `docs/project-context.md`
  - Already frames SCVI support as runtime-only and explicitly forbids treating CLI/build commands as supported workflow; keep this guidance unless any phrasing remains misleading.
- `docs/architecture.md`
  - Contains several SCVI references that appear intentionally bounded as legacy sampler/runtime context; only update if a passage still reads as current product path or points to removed tooling.

### Implementation Guidance

- Start with the docs that are objectively stale or misleading:
  - `docs/project-overview.md`
  - `docs/source-tree-analysis.md`
- Then audit the already-partially-cleaned guidance docs:
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `docs/architecture.md`
- Be precise about wording:
  - Replace "current SCVI playback path" claims with the intended VST-first direction.
  - Preserve only the minimum SCVI detail needed to explain bounded legacy internals that still exist.
  - Do not overcorrect into inaccurate claims that full VST playback workflow is already broader than what the repo currently supports.
- Prefer repo-state truth over historical completeness:
  - If a documented file or script no longer exists, remove the reference.
  - If a legacy file still exists, describe it as legacy/internal only.

### File Structure Requirements

- Primary documentation targets:
  - `docs/project-overview.md`
  - `docs/source-tree-analysis.md`
- Likely review/update targets depending on wording:
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `docs/architecture.md`
- Keep changes documentation-first.
- Only touch code or agent-instruction files if the documentation pass reveals current guidance there is still misleading.

### Testing Requirements

- Minimum validation target:
  - A targeted post-edit search confirms no documentation or guidance still presents SCVI as the active playback path.
  - No documentation points to removed SCVI CLI scripts or binaries as current workflow.
  - Remaining SCVI mentions are intentionally legacy/runtime-only or explicitly deferred to Story 1.5 verification.
- Preferred validation steps:
  - Search docs and guidance files for `SCVI` / `scvi` before and after edits.
  - Verify existence of any referenced file/script/path against the actual repo tree.
  - Run broader project checks only if this story changes non-doc files.
- Do not create a new repo-wide docs harness for this cleanup.

### Library / Framework Requirements

- Preserve the current pinned stack and repo conventions; this is documentation cleanup, not a dependency or architecture change.
- Relevant stack context for wording accuracy:
  - React `19.2.4`
  - TypeScript `5.9.3`
  - Vite `7.3.1`
  - Tauri frontend API `2.10.1`
  - Tauri Rust crate `2.10.3`
- Describe product direction and repo seams using the current architecture, not obsolete SCVI-first terminology.

### Documentation Impact

- This story is itself a documentation-impact story and must update the stale repo guidance directly.
- If the cleanup changes AI-agent implementation expectations beyond the current docs, update the relevant AI-agent instructions in the same change.
- Do not leave broken file references or outdated workflow guidance behind for Story 1.5 to silently absorb.
- Story 1.5 should validate the retirement outcome, not compensate for known stale docs that Story 1.4 can fix directly.

### Git Intelligence Summary

- Recent branch history confirms the cleanup sequence already underway:
  - `feat(1.3): continued scvi cleanup`
  - `feat(1.2): saving cleanup 2`
  - `feat(1.1): scvi cleanup part 1`
  - `chore: spec vst work`
- Story 1.4 should extend that sequence by making the documentation truthful about the current VST-first direction and the retired SCVI workflow/tooling path.

### Project Structure Notes

- The current repo state after Story 1.3 no longer contains:
  - `scripts/scvi-compress`
  - `scripts/scvi-inspect`
  - `src-tauri/src/bin/scvi_compress.rs`
  - `src-tauri/src/bin/scvi_inspect.rs`
- The repo still contains `src-tauri/src/playback/scvi.rs`, so documentation may retain it only as a bounded legacy runtime-only reader.
- `docs/project-overview.md` and `docs/source-tree-analysis.md` are the clearest mismatches between documentation and current repo state.
- `docs/project-context.md`, `docs/development-guide.md`, and much of `docs/architecture.md` already trend toward correct legacy framing; review them for accuracy, but avoid churn where the wording is already compliant.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 1, Story 1.4 acceptance criteria
- `_bmad-output/planning-artifacts/prd.md` — SCVI should be removed rather than carried forward; VST-first is the intended playback direction
- `_bmad-output/planning-artifacts/architecture.md` — preserved brownfield foundation and backend/frontend ownership rules
- `_bmad-output/planning-artifacts/ux-design-specification.md` — score-first workflow, avoid legacy or host-topology confusion in product communication
- `docs/project-overview.md` — still presents SCVI as a current feature
- `docs/source-tree-analysis.md` — still lists removed SCVI tooling files and needs repo-state reconciliation
- `docs/development-guide.md` — current maintainer guidance to verify for post-1.3 accuracy
- `docs/project-context.md` — current AI-agent rules with legacy runtime-only SCVI framing
- `docs/architecture.md` — architecture reference with bounded legacy SCVI runtime details
- `_bmad-output/implementation-artifacts/1-1-remove-scvi-from-the-active-playback-runtime.md` — runtime cleanup boundaries
- `_bmad-output/implementation-artifacts/1-2-remove-scvi-from-user-facing-playback-workflow.md` — workflow cleanup boundaries
- `_bmad-output/implementation-artifacts/1-3-remove-scvi-tooling-and-build-workflow-residue.md` — tooling-cleanup boundaries and current repo-state implications

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story target provided explicitly by user as `1.4`.
- Sprint tracker showed `1-4-remove-scvi-references-from-documentation-and-guidance` in `backlog` before story creation.
- Context grounded in Epic 1 planning docs, PRD, architecture, UX spec, project-context rules, Stories 1.1 through 1.3, current repo documentation hits, current filesystem state, and recent git history.
- Read-only exploration identified `docs/project-overview.md` and `docs/source-tree-analysis.md` as the primary stale documents, with `docs/development-guide.md`, `docs/project-context.md`, and `docs/architecture.md` already partially aligned and requiring only targeted wording review.
- Filesystem inspection confirmed Story 1.3 removed the SCVI wrapper scripts and SCVI Rust CLI binaries, while `src-tauri/src/playback/scvi.rs` remains as a legacy runtime-only file.
- No external web research was required because this story is repo-local documentation cleanup against the current pinned stack and completed Epic 1 cleanup sequence.
- Updated sprint tracking to `in-progress`, edited the two stale documentation files named in the story, and left `docs/development-guide.md`, `docs/project-context.md`, and `docs/architecture.md` unchanged because their remaining SCVI mentions were already bounded as legacy/runtime-only context.
- Targeted post-edit searches across `docs/**` confirmed no retained documentation reference to `scripts/scvi-compress`, `scripts/scvi-inspect`, `src-tauri/src/bin/scvi_compress.rs`, or `src-tauri/src/bin/scvi_inspect.rs`.
- Repo-state verification confirmed `src-tauri/src/playback/scvi.rs` still exists and is now described as a legacy runtime-only `.scvi` container reader in source-tree documentation.

### Completion Notes List

- Story context created with explicit boundaries between documentation cleanup (Story 1.4) and final verification sweep (Story 1.5).
- Story instructs the dev to update stale docs that still present SCVI as current while preserving only bounded legacy runtime explanations that remain technically accurate.
- Story identifies the highest-value stale files first so implementation can stay documentation-first and avoid unnecessary code churn.
- Updated `docs/project-overview.md` to replace the stale SCVI sampler-engine feature claim with VST-first product-direction wording that does not overstate currently shipped playback behavior.
- Updated `docs/source-tree-analysis.md` to remove retired SCVI CLI binary and wrapper-script entries and to relabel `src-tauri/src/playback/scvi.rs` as legacy runtime-only reader infrastructure.
- Validation stayed documentation-scoped because the change set did not touch code, commands, or agent instructions.
- Review completed and the story is now closed as done.

### File List

- `_bmad-output/implementation-artifacts/1-4-remove-scvi-references-from-documentation-and-guidance.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/project-overview.md`
- `docs/source-tree-analysis.md`

## Change Log

- 2026-03-21: Removed stale SCVI-as-current documentation claims, reconciled source-tree docs with post-1.3 repo state, and validated that remaining SCVI mentions are legacy/runtime-only context.
- 2026-03-21: Review completed and Story 1.4 was closed as done.
