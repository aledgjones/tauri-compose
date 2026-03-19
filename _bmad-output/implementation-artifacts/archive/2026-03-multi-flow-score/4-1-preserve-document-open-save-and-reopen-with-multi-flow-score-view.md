# Story 4.1: Preserve Document Open, Save, and Reopen with Multi-Flow Score View

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want scores to open, save, and reopen correctly while multi-flow score view is introduced,
so that the new score presentation does not put normal document work at risk.

This story is the document-safety gate for Epic 4. Multi-flow rendering is already established as a derived presentation layer over existing score data. This increment must prove that opening an existing project, saving it back to disk, and reopening it later still round-trip through the current `.soloc` pipeline without inventing a new persisted multi-flow format or weakening the current migration and file-integrity guarantees.

## Acceptance Criteria

1. Given a score that can be rendered in multi-flow score view, when the user opens the document through the existing desktop workflow, then the document loads correctly and the score can render through the updated multi-flow-capable path without corrupting document state.
2. Given a score viewed in the updated score view, when the user saves and later reopens the document, then the document reopens correctly and the score remains usable without requiring a new persisted multi-flow document format.
3. Given a single-flow score, when the same open, save, and reopen workflow is exercised, then existing document behavior remains correct and no regression is introduced by the multi-flow presentation work.
4. Given this document-safety increment is validated, when maintainers review the implementation, then multi-flow rendering remains a presentation-layer capability and no migration is required for this story.

## Tasks / Subtasks

- [x] Audit the existing load/save pipeline before changing behavior (AC: 1, 2, 3, 4)
  - [x] Inspect `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, and `src-tauri/src/migrations/` to confirm where project payloads are deserialized, migrated, reapplied to app state, and serialized back to disk.
  - [x] Confirm the persisted project DTO remains `{ ui, score, next_id }` and that multi-flow rendering is still derived from existing flow/layout state rather than a separate saved representation.
  - [x] Identify any current renderer-reset, sampler-sync, or transport-sync behavior triggered during load so the reopened project lands in a coherent post-load state.
- [x] Preserve document round-tripping while multi-flow rendering is active (AC: 1, 2, 3)
  - [x] Keep `load_project_from_file` and `save_project_to_file` authoritative for project I/O; do not add a parallel multi-flow-specific save path.
  - [x] Ensure the multi-flow-capable render path can rebuild from existing score/layout data immediately after load without storing presentation-only state in the `.soloc` payload.
  - [x] Preserve current single-flow round-tripping behavior and error handling for malformed or unsupported files.
- [x] Add regression coverage around open/save/reopen with multi-flow-visible data (AC: 1, 2, 3, 4)
  - [x] Extend Rust tests near `src-tauri/src/files.rs` and related persistence seams to verify load/save round-trips still preserve the current project model when multiple flows and layouts are present.
  - [x] Extend frontend command tests in `src/services/actions.test.ts` for open/save wiring only if command usage changes materially.
  - [x] Add manual verification guidance using representative `.soloc` fixtures or sample projects that exercise both single-flow and multi-flow-visible layouts.
- [x] Keep migration and persistence scope honest (AC: 2, 4)
  - [x] Do not add a migration unless implementation discovery proves that older files would fail to load correctly after the change.
  - [x] If a real persisted-data or load/save semantic change becomes unavoidable, add the pending migration in `src-tauri/src/migrations/next.rs` and document the impact explicitly.
- [x] Update docs only if the stable file-behavior contract changes (AC: 4)
  - [x] Update `docs/architecture.md`, `docs/data-models.md`, `docs/project-overview.md`, and `TESTING.md` if the implementation changes maintainer expectations for load/save verification or `.soloc` semantics.

## Dev Notes

### Story Foundation

- Epic 1 through Epic 3 deliberately treated multi-flow as a derived render/composition feature over existing score data rather than a new persisted score model.
- Epic 4 Story 4.1 is the first explicit regression guard for that decision: the project must continue to open, save, and reopen through the current desktop document pipeline without inventing a multi-flow file format.
- The story should stay narrowly focused on file integrity, migration safety, and rehydration correctness. It is not permission to redesign the `.soloc` container, broaden persistence scope, or persist presentation-only render artifacts.

### Developer Context Summary

- `src-tauri/src/files.rs` owns `load_project_from_file`, `save_project_to_file`, recent-file helpers, current-file-path state, and the migration-aware `.soloc` parse/apply flow.
- `src-tauri/src/files.rs` already deserializes the persisted project through the `ProjectFile` DTO (`ui`, `score`, `next_id`) and validates the authoritative container `app_version` against `score.meta.app_version` before applying migrations.
- `src-tauri/src/soloc.rs` owns the container build/decompress helpers, so any persistence work should preserve that module as the single source of truth for the binary wrapper rather than inventing a second save format.
- `docs/project-overview.md`, `docs/architecture.md`, and `docs/data-models.md` all document `.soloc` as the stable project format and the migration chain as app-version keyed. Story 4.1 should preserve that contract.
- The recent multi-flow work lives in render-plan and composition seams, not in file format code. That is the intended boundary this story should keep intact.

### Technical Requirements

- Preserve the persisted project shape as `{ ui, score, next_id }`; do not serialize multi-flow render products, page-box offsets, or other presentation-only state.
- Keep load/save behavior authoritative in `src-tauri/src/files.rs` and `.soloc` container behavior authoritative in `src-tauri/src/soloc.rs`.
- Preserve migration review rules. If older files would not load correctly without transformation, add pending migration steps in `src-tauri/src/migrations/next.rs`; otherwise, do not touch migrations.
- Preserve current integrity checks for invalid `.soloc` magic, unsupported chunk flags, version mismatches, and malformed headers.
- Preserve single-flow behavior. Multi-flow-capable code must not cause regressions in projects that only have one visible flow.
- Keep any post-load render preparation derived from current score/layout state; reopening a file should reconstruct the multi-flow surface from existing data, not from saved renderer state.
- No network dependency, sync assumption, or account-coupled persistence behavior may be introduced.

### Architecture Compliance

- Rust remains the single source of truth for durable document state and file I/O.
- Multi-flow remains a derived presentation layer over existing flows/layouts, not a new persisted score structure.
- The frontend continues to invoke document commands through `actions.ts`; it must not become a second persistence authority.
- The current migration model remains app-version keyed and `.soloc`-container based; do not introduce a second schema-version system.

### Library / Framework Requirements

- Stay within the current repo stack: Tauri 2.10.1, Rust 2021, React 19.2.4, TypeScript 5.9.3.
- Preserve `snake_case` for all FE-to-BE command payloads and typed alignment between Rust persistence payloads and TypeScript state types.
- Reuse existing persistence helpers and tests rather than adding ad hoc save/load scripts.
- External web research was not available in this environment; repository docs and current source are the authoritative baseline.

### Project Structure Notes

- Primary persistence seams:
  - `src-tauri/src/files.rs`
  - `src-tauri/src/soloc.rs`
  - `src-tauri/src/migrations/mod.rs`
  - `src-tauri/src/migrations/next.rs`
- Primary frontend document-command seams:
  - `src/services/actions.ts`
  - `src/services/actions.test.ts`
  - `src/services/state/provider.tsx`
- Supporting documentation seams:
  - `docs/project-overview.md`
  - `docs/architecture.md`
  - `docs/data-models.md`
  - `TESTING.md`

### File Structure Requirements

- Extend the existing file I/O and migration modules before creating any new persistence abstraction.
- Keep renderer-specific work out of persistence code unless a post-load render reset is already part of the established file-open path.
- Avoid touching unrelated score or UI modules unless the file-open path already routes through them.
- If new tests are added, colocate them with the existing persistence tests in `src-tauri/src/files.rs` or adjacent command tests.

### Testing Requirements

- Add or extend Rust tests proving that projects with multiple flows/layouts still load, save, and reload through the current `.soloc` pipeline without data-shape drift.
- Validate that single-flow projects still round-trip unchanged.
- If command wiring changes, extend `src/services/actions.test.ts` and `src/services/state/provider.tsx` coverage for open/save menu or command flows.
- Run manual verification with representative `.soloc` files that exercise multi-flow-visible layouts after save and reopen.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Epic 3 kept render observability, recalculation, and performance work strictly on the derived render path. Story 4.1 should not collapse that clean boundary by persisting renderer-owned state.
- The recent `.soloc` container work already strengthened file integrity and migration handling. Reuse that foundation rather than reworking the container model.
- No previous Epic 4 story exists yet, so this story should establish the pattern that desktop workflow preservation is mostly about guarding boundaries, not adding new product surface area.

### Git Intelligence Summary

- `063efd68 feat(3.5): set current performance baseline warning` reinforces the current pattern of tightening multi-flow guardrails without broadening feature scope.
- `ae16e160 chore: refactor render trigger API` suggests render-path changes are still active; Story 4.1 should therefore verify that file-open rehydration continues to trigger the right derived render behavior without persisting extra state.
- `09e9b6d5 feat(3.3): isolate recalculation to affected flow` and `700e7dda test(3.2): add multi-flow notation safety regressions` both reinforce the repo habit of keeping multi-flow work additive and test-backed rather than rewriting foundational data contracts.

### Project Context Reference

- `docs/project-context.md` says Rust owns score state, durable document mutations, and native score rendering.
- `docs/architecture.md` and `docs/project-overview.md` document `.soloc` as the app's project format and the migration chain as the authoritative compatibility path.
- `_bmad-output/planning-artifacts/architecture.md` explicitly maps desktop document workflow work to `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, and the migrations directory.
- `_bmad-output/planning-artifacts/epics.md` defines this story as presentation-layer safety work with no planned migration.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 4, Story 4.1)
- Source: `_bmad-output/planning-artifacts/architecture.md` (desktop document workflow and persistence boundaries)
- Source: `docs/project-context.md` (Rust-owned persistence and render boundaries)
- Source: `docs/project-overview.md` (`.soloc` container overview)
- Source: `docs/architecture.md` (file format and persistence architecture)
- Source: `docs/data-models.md` (migration system and persisted project semantics)
- Source: `src-tauri/src/files.rs` (project load/save, migration, recent files, current file path)
- Source: `src-tauri/src/soloc.rs` (container build/decompress helpers)
- Source: `src/services/actions.ts` (frontend load/save command wiring)
- Source: `src/services/state/provider.tsx` (menu-driven open/save command listeners)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Audit the load/save boundary in `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, and `src-tauri/src/migrations/` to verify the persisted project shape and post-load rehydration behavior remain unchanged.
- Add persistence regression tests in `src-tauri/src/files.rs` that round-trip representative single-flow and multi-flow-visible project payloads through the existing `.soloc` container helpers.
- Prove the reopened multi-flow project still rebuilds its visible-flow render plan from restored `score` and `ui` state alone, without any persisted renderer-owned state or migration changes.

### Completion Notes List

- Story 4.1 was created directly from Epic 4 planning artifacts and current persistence code seams.
- The story keeps multi-flow rendering explicitly on the derived presentation path and treats any persistence change as an exception requiring migration review.
- The repository does not contain `_bmad/core/tasks/validate-workflow.xml`, so the BMAD validator could not be invoked; the create-story checklist was applied manually while preparing this artifact.
- External web research was not available in this environment; repository docs and source were used as the authoritative baseline.
- Audited `load_project_from_file` and `save_project_to_file` and confirmed they remain the sole project I/O path; the persisted DTO is still `{ ui, score, next_id }`, the `.soloc` wrapper still lives in `src-tauri/src/soloc.rs`, and load still stops playback, syncs transport, emits a full-visible-flow render recalculation, and then syncs the sampler.
- Added Rust regression coverage in `src-tauri/src/files.rs` for the authoritative project DTO shape, multi-flow-visible `.soloc` round-tripping, and single-flow round-trip non-regression.
- No frontend command changes were required because `src/services/actions.ts` already routes open/save through `load_project_from_file` and `save_project_to_file`; the existing command tests remain sufficient for this story.
- No migration was added because the persisted project model and load/save semantics did not change.
- Manual verification guidance: open a representative multi-flow-visible `.soloc` project, confirm the visible layout still composes the expected flows, save, reopen, and confirm the same layout and active-flow context render correctly; repeat once with a single-flow project to confirm unchanged behavior.
- Manual verification completed on 2026-03-12: user confirmed the open/save/reopen workflow behaved correctly.
- Validation completed with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### File List

- `_bmad-output/implementation-artifacts/4-1-preserve-document-open-save-and-reopen-with-multi-flow-score-view.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/files.rs`

### Change Log

- 2026-03-12: Audited the existing `.soloc` load/save pipeline, confirmed no migration or persistence-shape change was needed for multi-flow score view, and added Rust regression coverage for multi-flow and single-flow round-tripping.
- 2026-03-12: Story accepted after manual verification; status updated from review to done.
