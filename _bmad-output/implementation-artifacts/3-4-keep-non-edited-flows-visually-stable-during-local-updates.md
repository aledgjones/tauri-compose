# Story 3.4: Keep Non-Edited Flows Visually Stable During Local Updates

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want visible flows that I am not editing to remain visually stable,
so that local score changes do not make the broader multi-flow document feel jittery or fragile.

Story 3.3 narrowed score-mutation recalculation for many flow-local edits, but some local update paths still drive the broad visible-flow pass. In particular, deliberate sequencer selection changes in `src-tauri/src/ui/sequencer.rs` still call `state.emit(..., true)`, which means a purely local visual update can re-run all visible flows and make unaffected flows appear to churn without cause. Story 3.4 should close that gap by keeping non-edited flows visually stable during local updates while preserving the shared-surface render model, authoritative backend diagnostics, and existing notation behavior.

## Acceptance Criteria

1. Given multiple flows are visible in score view, when a local update occurs in one visible flow, then non-edited visible flows remain visually stable and the user does not see unnecessary layout shifts or redraw artifacts in unaffected flows.
2. Given a local update changes the edited flow visibly, when the render completes, then the visible update is limited to the relevant flow and any directly affected shared presentation areas, and unaffected flows do not appear to flicker or move without cause.
3. Given a score with one visible flow, when the same update path runs, then current single-flow rendering behavior remains intact and no regression is introduced by multi-flow stabilization logic.
4. Given this visual-stability increment is validated, when users perform local edits in multi-flow score view, then the rendered document feels more dependable and notation semantics remain unchanged.

## Tasks / Subtasks

- [ ] Audit the remaining local update paths that still trigger broad visible-flow recomposition and identify the minimum affected-flow boundary for each one (AC: 1, 2, 3, 4)
  - [ ] Trace the selection and clear-selection commands in `src-tauri/src/ui/sequencer.rs` that still call `state.emit(&app, &mut renderer, true)` and confirm which of those paths only need flow-local visual updates.
  - [ ] Verify whether any other local UI-driven update paths still bypass the Story 3.3 recalculation request seam in `src-tauri/src/lib.rs` and broaden to a whole-visible-flow pass without a real shared-surface dependency.
  - [ ] Separate truly shared presentation changes from edited-flow-only changes so the story narrows visible churn only where the backend can remain truthful.
- [ ] Extend the backend-owned recalculation and reuse path so local visual updates preserve unaffected flow stability (AC: 1, 2, 3)
  - [ ] Route flow-local selection and related local update commands through the explicit recalculation request path introduced in Story 3.3 instead of the legacy broad boolean path.
  - [ ] Preserve truthful `render-timing-flow-preparation-details` diagnostics by recalculating only the flow that actually changed unless the backend can prove a broader shared-surface update is required.
  - [ ] Reuse cached non-edited flow segments and keep the composed shared surface ordered in `score.flows.order` without introducing a frontend-owned stability model.
- [ ] Keep shared presentation behavior coherent when only one visible flow updates (AC: 1, 2, 4)
  - [ ] Preserve hit-target correctness, selection highlight behavior, popover anchor lookup, and `ui.flow_id` semantics when local updates narrow to one flow.
  - [ ] Define and implement the rule for directly affected shared presentation areas so any required shared redraws stay minimal, deterministic, and backend-owned.
  - [ ] Avoid changes to notation derivation, written-duration semantics, persistence, or multi-flow page-order rules while stabilizing visual updates.
- [ ] Add focused regression coverage for visual stability and narrowed update scope (AC: 1, 2, 3, 4)
  - [ ] Add Rust tests around `src-tauri/src/ui/sequencer.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/instructions/render_plan.rs`, and `src-tauri/src/instructions/mod.rs` proving local selection and related local updates only recalculate the affected visible flow when reuse is available.
  - [ ] Add or extend tests that prove unaffected flow draw instructions, hit targets, and composed positions remain stable while another visible flow updates.
  - [ ] Extend debugger-facing frontend tests only if diagnostics payload shape or display rules change materially.
- [ ] Validate the visual-stability path end to end (AC: 1, 2, 3, 4)
  - [ ] Run `npm run lint`.
  - [ ] Run `npm run check-types`.
  - [ ] Run `npm run test:js`.
  - [ ] Run `npm run test:rust`.
  - [ ] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

## Dev Notes

### Story Foundation

- Epic 3 is sequencing multi-flow performance work deliberately: Story 3.1 added observability, Story 3.2 added notation-safety guardrails, and Story 3.3 narrowed recalculation for flow-local score mutations.
- Story 3.4 is the follow-on stabilization increment. It should remove unnecessary visible churn that remains after Story 3.3, especially for local interaction paths such as deliberate selection changes that are flow-local in practice.
- This story is still a presentation/orchestration change over existing flow data. It is not permission to rewrite notation logic, persistence, or frontend ownership boundaries.

### Developer Context Summary

- `src-tauri/src/ui/sequencer.rs` is still an important broad-pass seam: the `ui_sequencer_selection_set_*` commands and `ui_sequencer_selection_clear` currently call `state.emit(&app, &mut renderer, true)` after updating authoritative backend selection state.
- Story 3.3 already introduced the backend-owned recalculation seam in `src-tauri/src/lib.rs`, including `emit_with_recalculation`, `emit_affected_flow`, and track-to-flow mapping, plus truthful recalculation diagnostics in `src-tauri/src/instructions/render_plan.rs` and segment reuse in `src-tauri/src/instructions/mod.rs` / `src-tauri/src/renderer/mod.rs`.
- Selection identity is already flow-aware in `src-tauri/src/ui/sequencer.rs`, and Story 2.3 established that backend-owned selection state should remain explicit and predictable during local updates rather than being normalized by a generalized frontend heuristic.
- The renderer and popover seams already depend on backend-owned selection and hit targets: `src-tauri/src/instructions/mod.rs` uses selection during draw generation, and `src-tauri/src/ui/mod.rs` resolves anchor coordinates from composed hit targets after render.
- `src/windows/debugger/components/performance-timings.tsx` remains the intended maintainer-facing surface for verifying narrowed recalculation scope and shared-surface stability. Do not create a second observability path.

### Technical Requirements

- Keep the recalculation boundary truthful. Do not claim unaffected flows remained stable if the backend still recomputed them unnecessarily.
- Narrow local interaction-driven updates through the same backend-owned recalculation request seam used for score mutations, unless a specific shared-surface dependency requires a broader pass.
- Preserve the shared-surface composition model, `score.flows.order`, existing page-box behavior, hit-target composition, and native pan/zoom authority.
- Keep `ui.flow_id` as the only authoritative active-flow slot, and preserve deliberate selection handoff semantics established by Story 2.5.
- Preserve selection, popover, and keyboard-edit behavior while avoiding a second frontend-owned cache or stability model.
- No persistence, migration, or `.soloc` semantic change is expected. Do not touch `src-tauri/src/migrations/next.rs` unless implementation discovery proves a persisted-data impact, which would contradict the planned story boundary.

### Architecture Compliance

- Rust remains the single source of truth for score state, sequencer selection state, render orchestration, and diagnostics generation.
- The native score surface remains the only notation render path; React stays an overlay and consumer of backend-derived state.
- Multi-flow stabilization must extend the existing render-plan and composition seams rather than invent a parallel per-flow frontend render model.
- Shared presentation updates that truly affect more than one visible flow may still broaden recalculation, but the rule for doing so must be explicit, minimal, and backend-owned.
- Keep visual stability distinct from broader performance characterization work in Story 3.5.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep frontend/backend payload fields and command arguments in `snake_case` and preserve typed alignment between Rust diagnostics structs and TypeScript debugger consumers.
- Preserve current performance-oriented backend choices such as `FxHashMap`, the native Rust/Skia/Metal render path, and renderer-side segment reuse.
- No dependency upgrade is expected for this story.
- External web research was not available in this environment; repository docs and source are the authoritative baseline.

### Project Structure Notes

- Primary local-update seams to audit and likely extend:
  - `src-tauri/src/ui/sequencer.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/instructions/render_plan.rs`
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/renderer/mod.rs`
- Existing selection and interaction seams that must remain coherent:
  - `src-tauri/src/ui/mod.rs`
  - `src/windows/main/components/renderer.tsx`
  - `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - `src/windows/main/hooks/use-selected-tone-info.ts`
  - `src/windows/main/hooks/use-selected-tempo.ts`
  - `src/windows/main/hooks/use-selected-time-signature.ts`
  - `src/windows/main/hooks/use-selected-key-signature.ts`
  - `src/windows/main/hooks/use-selected-clef.ts`
- Existing debugger seam for diagnostics validation:
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
- Supporting docs that already describe the intended render contract:
  - `docs/project-context.md`
  - `docs/architecture.md`

### Testing Requirements

- Add Rust tests that prove local selection and clear-selection passes report only the affected flow in `recalculated_flow_ids` when reuse is available and broaden only when a real shared dependency requires it.
- Add Rust tests that prove unaffected visible flows keep stable composed instructions, hit targets, and offsets while another visible flow receives a local visual update.
- Keep single-flow behavior covered so the narrowed visual-stability path does not regress the current baseline.
- Extend debugger UI tests only if the diagnostics display contract changes.
- Extend frontend selection-consumer tests only if implementation discovery changes any selection or popover contract visible to the React shell.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 2.3 established that selection stability should be handled through explicit backend-owned updates rather than generalized normalization passes, which is directly relevant because selection changes are one of the remaining broad recalculation seams.
- Story 2.5 established that deliberate score-object selection may update `ui.flow_id`, while passive navigation and read-only interaction must not fabricate competing active-flow state.
- Story 3.1 established the maintainer-facing diagnostics contract for prepared, composed, and recalculated visible flows.
- Story 3.3 introduced the recalculation request model and cached unaffected-flow reuse; Story 3.4 should build on those seams instead of inventing a separate optimization path.
- Story 3.5 is the follow-on validation increment, so this story should focus on concrete visual-stability behavior rather than broad benchmark reporting.

### Git Intelligence Summary

- `700e7dda test(3.2): add multi-flow notation safety regressions` reinforces the repo pattern of adding guardrail coverage before or alongside render-path changes.
- `61d9a08c chore(3.1): prep` reflects the deliberate sequencing of Epic 3 from observability into targeted optimization work.
- `4d0dadb8 feat(2.5): preserve active flow across score and sequence` confirms active-flow semantics are already centralized and must not be duplicated while stabilizing local updates.
- `7ee83c7c feat(2.4): verify multi-flow popover anchors` and `928b0376 feat(2.3): simplify selection clearing for deletes` show that selection and anchor behavior are already treated as explicit backend-owned contracts, not incidental frontend effects.

### Project Context Reference

- `docs/project-context.md` defines the shared-surface render pipeline, backend-owned diagnostics contract, native renderer overlay model, and `ui.flow_id` authority rules.
- `docs/architecture.md` defines the Rust-owned state model, shared score-render pipeline, caching expectations, and selection/hit-target interaction boundaries.
- `_bmad-output/planning-artifacts/architecture.md` requires multi-flow work to remain a derived presentation capability over existing score data, preserve backend ownership boundaries, and keep invalidation/caching rules explicit.
- `_bmad-output/planning-artifacts/epics.md` defines Story 3.4 as the visual-stability increment that follows recalculation isolation and precedes broad performance validation.
- `_bmad-output/implementation-artifacts/2-3-preserve-stable-selection-while-one-visible-flow-updates.md` and `_bmad-output/implementation-artifacts/2-5-preserve-single-active-flow-behavior-across-score-and-sequencer-contexts.md` provide the selection and active-flow contracts that this story must preserve.
- `_bmad-output/implementation-artifacts/3-3-isolate-recalculation-to-the-affected-flow.md` provides the recalculation request and cached segment reuse seams that this story should extend.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.4 acceptance criteria)
- Source: `_bmad-output/planning-artifacts/architecture.md` (invalidating/caching boundaries, backend ownership, derived multi-flow presentation)
- Source: `docs/project-context.md` (render pipeline, diagnostics ownership, `ui.flow_id` rule, stack versions)
- Source: `docs/architecture.md` (Rust-owned state, selection/hit-target boundaries, caching and recalculation expectations)
- Source: `_bmad-output/implementation-artifacts/2-3-preserve-stable-selection-while-one-visible-flow-updates.md` (selection-stability precedent)
- Source: `_bmad-output/implementation-artifacts/2-5-preserve-single-active-flow-behavior-across-score-and-sequencer-contexts.md` (authoritative active-flow behavior)
- Source: `_bmad-output/implementation-artifacts/3-3-isolate-recalculation-to-the-affected-flow.md` (recalculation request and reuse seam)
- Source: `src-tauri/src/ui/sequencer.rs` (authoritative selection commands still using broad recalc)
- Source: `src-tauri/src/lib.rs` (render entry and recalculation request seam)
- Source: `src-tauri/src/instructions/render_plan.rs` (prepared/composed/recalculated flow reporting)
- Source: `src-tauri/src/instructions/mod.rs` (shared-surface composition, selection-aware draw path, flow-segment reuse)
- Source: `src-tauri/src/renderer/mod.rs` (segment cache storage)
- Source: `src-tauri/src/ui/mod.rs` (selection coordinate lookup against hit targets)
- Source: `src/windows/debugger/components/performance-timings.tsx` (maintainer-facing diagnostics consumer)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Completion Notes List

- Story 3.4 was created through the BMAD create-story flow using the first Epic 3 backlog item in `sprint-status.yaml`, which aligns with the existing plan sequence after Story 3.3.
- The story is framed as a follow-on to Story 3.3, with deliberate sequencer selection updates identified as the clearest remaining broad recalculation seam to narrow.
- Story context was grounded in Epic 3 planning, the architecture/planning artifacts, the current Rust render pipeline, Story 2.3 selection-stability rules, Story 2.5 active-flow rules, and the completed Story 3.3 invalidation work.
- The repository does not contain `_bmad/core/tasks/validate-workflow.xml`, so the workflow validation step could not run through the expected helper and was applied manually while preparing the artifact.
- No persisted-data migration is expected for this story.

### File List

- `_bmad-output/implementation-artifacts/3-4-keep-non-edited-flows-visually-stable-during-local-updates.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
