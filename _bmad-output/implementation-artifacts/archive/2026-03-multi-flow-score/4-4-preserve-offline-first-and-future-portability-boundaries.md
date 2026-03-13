# Story 4.4: Preserve Offline-First and Future Portability Boundaries

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want multi-flow score view to avoid introducing network assumptions or unnecessary persistence coupling,
so that the feature stays aligned with the project's offline-first and future-portable architecture.

This story closes Epic 4 by turning the architectural boundaries into explicit implementation guardrails. Multi-flow score view must remain a derived local capability over existing score data, without adding cloud assumptions, sync coupling, account requirements, or persistence work that would make future portability harder instead of easier.

## Acceptance Criteria

1. Given multi-flow score view is implemented as part of the desktop application, when the feature is reviewed for architecture impact, then it operates without any network dependency and core composition, rendering, interaction, and playback workflows remain offline-capable.
2. Given the feature introduces new score-view behavior, when maintainers review persistence impact, then multi-flow rendering remains derived from existing score data rather than requiring account, sync, or cloud assumptions and future score portability remains possible without introducing new MVP sync behavior.
3. Given current persisted score documents, when they are used with the updated score view, then they remain usable without a forced change to document semantics and any future persistence work remains separable from this presentation feature.
4. Given this boundary-preservation increment is validated, when Epic 4 is reviewed as a whole, then multi-flow score view remains compatible with the product's offline-first desktop direction and no unnecessary migration or cloud-related scope has been introduced.

## Tasks / Subtasks

- [x] Audit current architectural boundaries that this story must preserve (AC: 1, 2, 3, 4)
  - [x] Inspect `docs/project-context.md`, `docs/architecture.md`, `docs/project-overview.md`, `docs/data-models.md`, and the planning artifacts to confirm the current offline-first, persistence, and migration rules.
  - [x] Inspect `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, `src-tauri/src/migrations/`, and relevant multi-flow render seams to confirm where presentation ends and persistence begins.
  - [x] Audit any new dependencies or service usage introduced by the multi-flow feature to confirm no network assumptions have leaked into core workflows.
- [x] Keep multi-flow rendering derived and local (AC: 1, 2, 3)
  - [x] Preserve multi-flow score view as a render/composition concern derived from existing `score` and `ui` state rather than a new persisted document model.
  - [x] Do not add account, sync, cloud-storage, or remote API requirements to make multi-flow score view function.
  - [x] Preserve current `.soloc` document semantics unless a real persisted-data need is discovered and explicitly migration-gated.
- [x] Make portability and migration boundaries explicit in implementation and docs (AC: 2, 3, 4)
  - [x] Document any persistence-impact review performed as part of the implementation so it is clear why no migration is needed, or what migration is required if discovery proves otherwise.
  - [x] If a persisted-data change becomes necessary, add the pending migration in `src-tauri/src/migrations/next.rs` and update the relevant docs in the same change.
  - [x] Keep any future portability discussion framed as a separate follow-on capability rather than silently coupling it to this presentation feature.
- [x] Add regression coverage or audits that guard the boundary honestly (AC: 1, 2, 3, 4)
  - [x] Add or update tests only where implementation touches persistence or load/save semantics.
  - [x] If no code changes are needed beyond documentation or audits, record the explicit validation evidence rather than inventing low-value code churn.
  - [x] Add maintainer-facing verification guidance for dependency review, offline exercise, and migration-impact review.
- [x] Update docs and agent instructions if this story changes stable project rules (AC: 4)
  - [x] Update `docs/project-context.md`, `docs/architecture.md`, `docs/project-overview.md`, `docs/data-models.md`, and `.github/copilot-instructions.md` if implementation clarifies or strengthens stable offline-first, portability, or migration expectations.

## Dev Notes

### Story Foundation

- Stories 4.1 through 4.3 protect concrete user-facing workflows: document round-tripping, native desktop behavior, and playback preservation.
- Story 4.4 is the architectural capstone that keeps those protections honest by preventing hidden cloud, sync, or persistence coupling from slipping in under the banner of multi-flow support.
- The intended outcome is explicit restraint: multi-flow score view should remain a local, derived presentation capability over the existing desktop application.

### Developer Context Summary

- `docs/project-context.md` and `_bmad-output/planning-artifacts/architecture.md` both state that the product is macOS-first, desktop-first, and offline-first, with Rust owning durable state and `.soloc` remaining the project format.
- `docs/project-overview.md` and `docs/data-models.md` document the current `.soloc` container, app-version-keyed migrations, and the rule that persistence changes require migration review.
- `src-tauri/src/files.rs` and `src-tauri/src/soloc.rs` are the concrete persistence seams; multi-flow rendering work should stay outside them unless a true document-semantics issue is discovered.
- The current checked-in Tauri config exposes local asset protocol scope for app data but no application-level network dependency for core workflows. Story 4.4 should preserve that posture.
- The multi-flow feature itself has so far lived in render-plan, composition, selection, and active-flow seams, which is consistent with the intended derived-presentation model.

### Technical Requirements

- Multi-flow score view must function without network access.
- Core composition, rendering, interaction, playback, open, save, and reopen workflows must remain offline-capable.
- Multi-flow rendering must remain derived from existing score/layout/UI state instead of introducing a new persisted score-view model.
- Do not add sync, account, or cloud assumptions to make the feature work.
- Preserve `.soloc` semantics and the current migration chain unless a real persisted-data change is required.
- If older files would fail without transformation, add the pending migration in `src-tauri/src/migrations/next.rs` and document the impact explicitly.
- Preserve future portability by keeping presentation behavior separable from persistence behavior.

### Architecture Compliance

- The feature remains aligned with the project's offline-first desktop direction.
- Rust remains the source of truth for durable document state; React remains the orchestration shell.
- Persistence, migration, and document semantics remain separate concerns from score-surface presentation.
- Future sync or portability features remain optional follow-on work, not hidden dependencies of this MVP feature.

### Library / Framework Requirements

- Do not introduce new network clients, cloud SDKs, or remote service dependencies for this story.
- Reuse the existing `.soloc` persistence, migration, and local file workflows already present in the repo.
- Preserve the current Tauri/Rust/React stack and avoid platform-expansion work.
- External web research was not available in this environment; repository docs and source were used as the authoritative baseline.

### Project Structure Notes

- Primary boundary and persistence seams:
  - `src-tauri/src/files.rs`
  - `src-tauri/src/soloc.rs`
  - `src-tauri/src/migrations/mod.rs`
  - `src-tauri/src/migrations/next.rs`
- Primary multi-flow presentation seams that should stay derived:
  - `src-tauri/src/instructions/render_plan.rs`
  - `src-tauri/src/instructions/mod.rs`
  - `src/windows/main/components/renderer.tsx`
- Primary documentation and rules seams:
  - `docs/project-context.md`
  - `docs/architecture.md`
  - `docs/project-overview.md`
  - `docs/data-models.md`
  - `.github/copilot-instructions.md`

### File Structure Requirements

- Keep presentation changes in existing render/interaction modules instead of spreading persistence logic into feature-specific files.
- Keep migration work, if any, in `src-tauri/src/migrations/next.rs` until a human release flow promotes it.
- Avoid creating repo-wide portability or sync scaffolding as part of this story.
- Prefer documentation updates over low-value code churn if the story outcome is primarily validation of already-correct boundaries.

### Testing Requirements

- If implementation touches persistence or migration behavior, add or extend Rust tests in the relevant file-loading and migration seams.
- Otherwise, validate the boundary through explicit dependency review, offline workflow checks, and persistence-impact review notes.
- Confirm that representative multi-flow projects still open, render, save, and play back without network access.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 4.1 already frames persistence changes as exceptional and migration-gated.
- Story 4.2 preserves the desktop shell and native document posture.
- Story 4.3 preserves the existing single-active-flow playback contract without expanding scope.
- Story 4.4 should unify those constraints into an explicit architectural boundary: no hidden cloud, sync, or persistence coupling.

### Git Intelligence Summary

- The recent Epic 3 commits reinforce a consistent project pattern: add guardrails, preserve boundaries, and avoid unnecessary scope expansion.
- Story 4.4 should follow that pattern by recording and enforcing the offline-first/persistence boundary rather than inventing new functionality.

### Project Context Reference

- `_bmad-output/planning-artifacts/epics.md` defines this story as a boundary-preservation increment, not a new feature surface.
- `_bmad-output/planning-artifacts/architecture.md` and `docs/project-context.md` both state that the product is offline-first and that persistence changes should be avoided where possible.
- `docs/data-models.md` and `docs/project-overview.md` define the current `.soloc` and migration contract that this story should preserve.
- `.github/copilot-instructions.md` already requires migration review for persisted-data or load/save semantic changes and should stay aligned if this story clarifies project rules further.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 4, Story 4.4)
- Source: `_bmad-output/planning-artifacts/architecture.md` (offline-first and persistence boundaries)
- Source: `docs/project-context.md` (offline-first, persistence, and active architectural rules)
- Source: `docs/architecture.md` (file format, migration, and desktop architecture)
- Source: `docs/project-overview.md` (`.soloc` and desktop product framing)
- Source: `docs/data-models.md` (migration system and persisted document semantics)
- Source: `src-tauri/src/files.rs` (load/save and migration application)
- Source: `src-tauri/src/soloc.rs` (container helpers)
- Source: `.github/copilot-instructions.md` (migration and persistence rules)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Audit the existing documentation, persistence seams, migration chain, and multi-flow render-plan path to confirm Story 4.4 closes as a boundary-preservation review rather than a new implementation slice.
- Verify that no network clients, sync assumptions, or feature-specific persisted state were introduced while shipping Epic 4.
- Prefer explicit audit evidence and maintainer notes over speculative code or test churn if the current architecture already satisfies the story.

### Completion Notes List

- Story 4.4 was created as the architectural closeout story for Epic 4, consolidating the project's existing offline-first and persistence-boundary rules into explicit implementation guardrails.
- The story intentionally prefers validation and documentation over speculative code churn when the correct outcome is to preserve already-correct boundaries.
- The repository does not contain `_bmad/core/tasks/validate-workflow.xml`, so the BMAD validator could not be invoked; the create-story checklist was applied manually while preparing this artifact.
- External web research was not available in this environment; repository docs and source were used as the authoritative baseline.
- Audited `docs/project-context.md`, `docs/architecture.md`, `docs/project-overview.md`, `docs/data-models.md`, and `.github/copilot-instructions.md` and confirmed the repo already states the required offline-first, migration-gated, `.soloc`-centric architectural rules clearly.
- Audited `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, `src-tauri/src/migrations/mod.rs`, and `src-tauri/src/migrations/next.rs` and confirmed persisted project semantics remain the authoritative `{ ui, score, next_id }` payload, the migration chain remains version-gated, and there is no pending migration needed for Epic 4 multi-flow work.
- Audited `src-tauri/src/instructions/render_plan.rs` and the main renderer seam and confirmed multi-flow score view still resolves as a derived local render/composition concern from existing score/layout/UI state instead of a separate persisted view model.
- Audited `package.json` and `src-tauri/Cargo.toml` for new dependency drift and found no application network client or cloud SDK requirement added for Epic 4; the current dependency set remains aligned with a local desktop app using Tauri file/dialog/opener capabilities and embedded storage.
- No product code, persistence code, migrations, tests, docs, or agent-instruction changes were required because the current implementation already preserves the intended offline-first and future-portable boundaries for multi-flow score view.
- Maintainer verification guidance: continue treating any future sync, account, portability, or cloud-storage capability as a follow-on project; if a future score-view enhancement changes `.soloc` semantics or load/save behavior, that work must remain migration-gated and documented separately rather than being folded silently into presentation work.
- No additional validation commands were run because this story closed through architectural audit only and intentionally avoided low-value code churn.
- Story accepted on 2026-03-12 after review confirmed Epic 4 introduced no hidden network dependency, persistence coupling, or migration work beyond the existing documented project boundaries.

### File List

- `_bmad-output/implementation-artifacts/4-4-preserve-offline-first-and-future-portability-boundaries.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Change Log

- 2026-03-12: Audited offline-first, persistence, migration, and dependency boundaries for Epic 4 and confirmed the multi-flow feature remains a local derived presentation capability with no extra persistence or network scope.
- 2026-03-12: Story accepted and status updated from review to done.
