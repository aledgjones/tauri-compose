# Story 1.5: Verify Clean SCVI Retirement Across Product and Repo Surfaces

Status: done

## Story

As a maintainer,
I want to verify that SCVI has been cleanly retired from the active product and repository workflow,
so that future playback work can proceed on one unambiguous foundation.

## Acceptance Criteria

1. Given SCVI runtime code, user-facing workflow residue, tooling, and docs have been cleaned up, when the agreed verification checks are run, then repository search confirms no active SCVI-specific runtime, UI, tooling, or guidance path remains and the remaining SCVI references, if any, are only intentionally historical or explanatory.
2. Given the cleanup has been completed, when the app build and relevant validation checks are run, then the product remains buildable and the checks do not rely on SCVI-specific behavior to pass.
3. Given maintainers inspect the repository for the current playback direction, when Epic 1 is complete, then they can treat SCVI as retired from active product behavior and workflow and future playback stories can proceed without dual-path assumptions, migration scaffolding, or legacy fallback requirements.

## Tasks / Subtasks

- [x] Task 1: Build a repo-wide SCVI verification inventory and classification map (AC: 1, 3)
  - [x] Run a repo-wide search for `SCVI`, `scvi`, and `.scvi` across active source, docs, tests, config, scripts, release history, and planning artifacts.
  - [x] Classify each remaining hit as one of: active-path defect, allowed legacy runtime-only seam, allowed historical/release record, or prior-story/planning artifact.
  - [x] Treat `_bmad-output/**`, sprint tracking, and `releases.md` as historical records unless a retained line still falsely presents SCVI as a current supported workflow.
- [x] Task 2: Verify active product and workflow surfaces no longer expose SCVI as current behavior (AC: 1, 2, 3)
  - [x] Confirm the supported transport path in `src-tauri/src/playback/transport.rs` remains non-SCVI and does not snapshot sampler or streamer state as active runtime truth.
  - [x] Confirm active maintainer workflow surfaces such as `package.json`, `src-tauri/Cargo.toml`, `scripts/`, `.vscode/`, and current docs do not advertise SCVI commands, binaries, or supported operational paths.
  - [x] Confirm frontend and user-facing source under `src/**` does not present SCVI labels, controls, or workflow copy; the current expected frontend-side search hit is only bounded debugger test data if it remains intentionally legacy.
- [x] Task 3: Reconcile any residual active-path SCVI residue discovered by the audit (AC: 1, 3)
  - [x] Update or remove any doc, comment, test, or code path that still presents SCVI as a supported runtime, UI, tooling, or guidance path.
  - [x] Preserve only minimal legacy or explanatory references needed for bounded sampler or audition internals such as `src-tauri/src/playback/scvi.rs`, `src-tauri/src/playback/sampler.rs`, and `src-tauri/src/playback/streamer.rs`, and keep that framing explicit.
  - [x] Do not introduce fallback logic, migration scaffolding, compatibility shims, or new abstractions merely to preserve SCVI.
- [x] Task 4: Run verification commands that must pass without SCVI-specific behavior (AC: 2)
  - [x] Run `npm run lint`.
  - [x] Run `npm run check-types`.
  - [x] Run `npm run build`.
  - [x] Run `npm run test:js`.
  - [x] Run `npm run test:rust`.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`.
- [x] Task 5: Record the final retirement outcome for maintainers (AC: 1, 2, 3)
  - [x] Capture which remaining SCVI references are intentionally historical or explanatory and why they do not represent an active product or workflow path.
  - [x] If the audit passes without functional changes, keep repo churn minimal and record the clean verification outcome in the story completion notes and any touched maintainer-facing doc.
  - [x] Ensure future playback stories can treat SCVI as retired from active product behavior and workflow without dual-path assumptions.

## Dev Notes

### Story Intent

- Story 1.5 is the final Epic 1 verification pass for SCVI retirement.
- The goal is not to erase every historical string `SCVI` from the repo. The goal is to prove that no active product, runtime, UI, tooling, or maintainer workflow still depends on or advertises SCVI as current.
- Historical release notes, sprint artifacts, and bounded legacy-runtime explanations may remain if they are clearly not active workflow guidance.
- This story should leave the repo in a state where future VST-first playback work can proceed without dual-path ambiguity.

### Scope Boundaries

- In scope:
  - Repo-wide classification of remaining SCVI references in active code, docs, tests, configs, scripts, and project artifacts.
  - Targeted cleanup of any misleading reference that still implies SCVI is a supported path.
  - Validation proving the current build and test workflow does not depend on SCVI-specific commands or runtime behavior.
- Out of scope:
  - Removing every historical or release-note mention of SCVI.
  - New VST feature work or playback capability expansion.
  - Release/tag/publish workflow changes.
  - Migration work, compatibility preservation, or introducing new fallback behavior.

### Previous Story Intelligence

- Story 1.1 removed SCVI from the supported transport runtime path and left `transport_play` timing-only on the supported product path.
- Story 1.2 removed SCVI from user-facing workflow surfaces so the app no longer presents SCVI as the supported playback direction.
- Story 1.3 removed SCVI package scripts, generated wrappers, and Rust CLI binaries while retaining bounded `.scvi` reader and sampler seams only as legacy internals.
- Story 1.4 removed stale documentation and source-tree guidance that still presented SCVI as current product direction, while intentionally preserving bounded legacy/runtime-only explanations where the code still exists.
- Story 1.4 is currently in `review`, so Story 1.5 must verify against the actual current workspace state rather than relying only on sprint-status labels.

### Architecture Compliance

- Preserve the existing Tauri + Rust + React + TypeScript + Vite brownfield foundation.
- Keep Rust as the owner of runtime truth and the frontend as a thin display and control layer.
- Supported playback must remain non-SCVI on the product path; do not reintroduce SCVI runtime ownership through transport, UI, or workflow validation.
- If `.scvi` support remains anywhere, it must stay bounded to legacy sampler or audition seams and be clearly framed as unsupported product-path behavior.
- Do not create a second verification harness or parallel architecture just to classify SCVI residue; reuse the existing repo search, build, test, and documentation seams.

### Current Verification Surface

- Current source-code SCVI hits are concentrated in backend legacy runtime files:
  - `src-tauri/src/playback/scvi.rs`
  - `src-tauri/src/playback/sampler.rs`
  - `src-tauri/src/playback/streamer.rs`
  - `src-tauri/src/playback/mod.rs`
- Current frontend-side search hits appear bounded to debugger test data rather than active UI workflow:
  - `src/windows/debugger/index.test.tsx`
- Current docs hits already trend toward bounded legacy framing and need verification rather than automatic removal:
  - `docs/project-overview.md`
  - `docs/development-guide.md`
  - `docs/architecture.md`
  - `docs/source-tree-analysis.md`
- Expected historical or non-product hits include:
  - `releases.md`
  - `_bmad-output/implementation-artifacts/*.md`
  - `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `package.json` no longer exposes SCVI-specific scripts, which is an important verification baseline for this story.

### Implementation Guidance

- Start with the search surface, not with editing. Determine exactly which remaining SCVI mentions are defects and which are expected legacy or historical references.
- Prioritize active workflow files first:
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `scripts/**`
  - `.vscode/**`
  - `README.md`
  - `TESTING.md`
  - `docs/**`
  - `src/**`
  - `src-tauri/src/**`
- Verify that supported runtime behavior stays clean:
  - `transport.rs` should remain timing-only on the supported product path.
  - Legacy sampler files may still parse `.scvi` assets, but that must not imply supported playback ownership.
- If a remaining hit is only historical or test-fixture context, do not remove it merely to make search output emptier.
- Prefer minimal, precise fixes over broad churn. The story is about trustworthy classification and cleanup, not rewriting half the repo.

### File Structure Requirements

- Primary verification targets:
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `src-tauri/src/playback/transport.rs`
  - `src-tauri/src/playback/mod.rs`
  - `src-tauri/src/playback/scvi.rs`
  - `src-tauri/src/playback/sampler.rs`
  - `src-tauri/src/playback/streamer.rs`
  - `src/windows/debugger/index.test.tsx`
  - `docs/project-overview.md`
  - `docs/development-guide.md`
  - `docs/architecture.md`
  - `docs/source-tree-analysis.md`
- Secondary verification targets if search surfaces them:
  - `README.md`
  - `TESTING.md`
  - `.vscode/settings.json`
  - `releases.md`
  - `_bmad-output/implementation-artifacts/*.md`
- Keep edits tightly scoped to any file that still misrepresents SCVI as current.

### Testing Requirements

- Minimum verification target:
  - Repo search finds no SCVI-specific runtime, UI, tooling, or guidance path still presented as active.
  - The remaining hits are explicitly historical, explanatory, or bounded legacy-runtime context.
  - Build, tests, and clippy pass without requiring SCVI-specific commands or supported playback behavior.
- Required validation commands:
  - `npm run lint`
  - `npm run check-types`
  - `npm run build`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- Include a final targeted search after any edits so the completion notes can classify the remaining SCVI references explicitly.
- Do not create a bespoke repo-wide audit framework for this story.

### Library / Framework Requirements

- Preserve the current pinned stack and treat this as verification and cleanup on the existing foundation, not a dependency-upgrade story.
- Relevant current stack context:
  - React `19.2.4`
  - TypeScript `5.9.3`
  - Vite `7.3.1`
  - Tauri frontend API `2.10.1`
  - Tauri Rust crate `2.10.3`
- No external API migration or web research is required for this story; the work is entirely repo-local.

### Documentation Impact

- This story may require no documentation edits if the current docs already frame SCVI correctly as retired or bounded legacy context.
- If the audit finds misleading language, update the exact owning documentation file rather than broadening into an unnecessary docs rewrite.
- If maintainers need a durable summary of what remaining SCVI references still mean, prefer recording that in an existing maintainer-facing doc or in the story completion notes instead of creating a standalone audit document.
- Do not leave behind ambiguous language that would force later VST stories to keep re-litigating whether SCVI is still considered active.

### Git Intelligence Summary

- Recent branch history confirms the cleanup sequence already completed before this verification pass:
  - `feat(1.4): documentation cleanup`
  - `feat(1.3): continued scvi cleanup`
  - `feat(1.2): saving cleanup 2`
  - `feat(1.1): scvi cleanup part 1`
  - `chore: spec vst work`
- Story 1.5 should close that sequence by proving the cleanup outcome is coherent across runtime, docs, tooling, and repo search surfaces.

### Project Structure Notes

- The active repo no longer advertises SCVI build or inspection commands from `package.json`.
- The remaining SCVI-aware backend files are currently legacy-runtime seams, not supported transport or build workflow surfaces.
- The main risk for this story is false positives: prior story artifacts, release history, and intentionally bounded legacy explanations can remain as search hits without violating the acceptance criteria.
- The real defect condition is any remaining source, doc, or workflow surface that still teaches maintainers or users that SCVI is part of the active Compose playback path.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 1, Story 1.5 acceptance criteria
- `_bmad-output/planning-artifacts/architecture.md` — backend ownership, validation, and brownfield architecture rules
- `_bmad-output/planning-artifacts/ux-design-specification.md` — score-first playback UX direction and avoidance of host-topology confusion
- `docs/project-context.md` — agent rules for legacy SCVI runtime-only support and non-supported workflow guidance
- `_bmad-output/implementation-artifacts/1-1-remove-scvi-from-the-active-playback-runtime.md` — supported runtime cleanup boundaries
- `_bmad-output/implementation-artifacts/1-2-remove-scvi-from-user-facing-playback-workflow.md` — user-facing cleanup boundaries
- `_bmad-output/implementation-artifacts/1-3-remove-scvi-tooling-and-build-workflow-residue.md` — tooling cleanup boundaries and remaining legacy-runtime classification
- `_bmad-output/implementation-artifacts/1-4-remove-scvi-references-from-documentation-and-guidance.md` — documentation cleanup boundaries and expected remaining legacy framing
- `package.json` — active maintainer workflow commands
- `src-tauri/src/playback/transport.rs` — supported playback path verification target
- `src-tauri/src/playback/scvi.rs` — legacy `.scvi` container reader
- `src-tauri/src/playback/sampler.rs` — legacy sampler inventory and cache seam
- `src-tauri/src/playback/streamer.rs` — legacy sampler disk-streaming seam
- `docs/project-overview.md` — current high-level product-direction framing
- `docs/development-guide.md` — current maintainer guidance on legacy sampler notes
- `docs/architecture.md` — current architecture framing for legacy sampler/runtime seams
- `docs/source-tree-analysis.md` — current source-tree classification of legacy SCVI files
- `src/windows/debugger/index.test.tsx` — current debugger test fixture reference to a legacy `.scvi` path
- `releases.md` — expected historical release-note references

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story target provided explicitly by user as `1.5`.
- Sprint tracker showed `1-5-verify-clean-scvi-retirement-across-product-and-repo-surfaces` in `backlog` before story creation.
- Context grounded in Epic 1 planning docs, architecture, UX spec, project-context rules, and Stories 1.3 and 1.4.
- Current search results show the remaining SCVI references concentrated in legacy backend runtime seams, bounded documentation references, debugger test data, release history, and prior-story artifacts.
- `package.json` no longer exposes SCVI-specific scripts, which confirms Story 1.3 cleanup at the active npm workflow surface.
- `transport.rs` remains the key verification seam for proving supported playback is no longer SCVI-owned.
- No external web research is required because this story is repo-local verification and cleanup against the current pinned stack.
- Targeted active-workflow searches returned no SCVI hits in `package.json`, `src-tauri/Cargo.toml`, `scripts/**`, `.vscode/**`, `README.md`, or `TESTING.md`.
- Final active-path search classified remaining hits as: legacy runtime-only backend seams in `src-tauri/src/playback/{scvi,sampler,streamer}.rs` and `mod.rs`; bounded debugger fixture data in `src/windows/debugger/index.test.tsx`; explanatory legacy framing in current docs; and historical planning/story records under `_bmad-output/**`.
- No active-path defect was discovered, so no functional source or documentation cleanup was required beyond this verification record.
- Verification commands completed successfully: `npm run lint`, `npm run check-types`, `npm run build`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`.

### Completion Notes List

- Completed the final Epic 1 SCVI retirement audit without needing functional code or documentation edits.
- Confirmed the supported transport path remains non-SCVI and timing-only on the product path; no active workflow surface advertises SCVI commands, binaries, UI, or guidance.
- Classified remaining SCVI references as either bounded legacy sampler/audition internals, bounded debugger fixture data, explanatory maintainer documentation, or historical planning/story artifacts.
- Verified the active toolchain and product checks pass without SCVI-specific behavior: `npm run lint`, `npm run check-types`, `npm run build`, full JS tests (1113 passed), Rust library tests (1100 passed), and clippy with warnings denied.
- Future playback stories can treat SCVI as retired from active product behavior and repository workflow without dual-path assumptions, migration scaffolding, or legacy fallback requirements.
- User accepted the verification pass as sufficient review and approved the story for closure as done.

### File List

- `_bmad-output/implementation-artifacts/1-5-verify-clean-scvi-retirement-across-product-and-repo-surfaces.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-03-21: Completed the SCVI retirement verification audit, recorded the remaining legacy/historical classifications, validated the repo without SCVI-specific workflow dependencies, and moved Story 1.5 to review.
- 2026-03-21: User accepted the verification outcome as the effective review and closed Story 1.5 as done.
