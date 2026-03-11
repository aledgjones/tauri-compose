# Story 3.1: Add Multi-Flow Render and Recalculation Visibility

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want render timing and diagnostic output to show which flows the unified render path processed and how much work each render or recalculation pass performed,
so that multi-flow rendering can be evolved through small changes with evidence rather than guesswork.

This increment is an observability story, not a notation or persistence story. The implementation should extend the existing render-timing and debugger surfaces that already expose flow-preparation and shared-surface diagnostics, and it should make later invalidation work measurable without inventing a parallel diagnostics stack or reintroducing a legacy single-flow renderer.

## Acceptance Criteria

1. Given the score view render pipeline uses the unified multi-flow composition path, when a score render completes, then timing output exposes which flows were prepared and composed and enough detail to understand which part of the render path was exercised.
2. Given a score with multiple flows, when a local update triggers recalculation, then the diagnostic output shows whether the work was scoped to one flow or multiple flows and the instrumentation is sufficient to support later invalidation improvements.
3. Given the renderer is provided any valid set of visible flows, when timing output is collected through the same unified render path, then maintainers still get useful baseline visibility for that pass and the instrumentation change does not alter score output behavior.
4. Given this observability increment is implemented, when it is validated, then no written-notation, written-duration, or persisted-score behavior changes are required and later optimization stories can rely on this visibility rather than introducing ad hoc measurement.

## Implied Observability Model

1. Reuse the current debugger timing pipeline.
   Existing events such as `render-timing-flow-preparation`, `render-timing-flow-preparation-details`, and `render-timing-surface-details` already flow from Rust into the debugger UI. Extend these contracts or add adjacent events in the same family instead of creating standalone logging paths, console-only output, or frontend-owned timing derivations.
2. Make render mode explicit.
   A maintainer looking at a completed render should be able to tell which flow ids were prepared, which were composed, how many flows participated, and whether the pass represented a normal render or a cleared/no-render state. The useful distinction is scope of work, not a conceptual split between renderer variants.
3. Make recalculation scope explicit.
   When a local edit or state change causes a recalculation, the diagnostics should show which visible flows were recalculated, whether the work matched the active flow only or a broader set, and whether the render path reused the existing shared-surface composition path.
4. Preserve the existing phase breakdown.
   Current per-phase timings for vertical spacing, layout, structure, notation, elements, spacing, stems, setup, barlines, stave, and GPU work remain useful. This story should add scope-awareness around them, not collapse or replace them.
5. Treat the debugger as a maintainer tool.
   The output should be optimized for development-time inspection in the existing debugger window and tests, not for end-user-facing score UI.

## User Journeys

### Journey 1: Compare narrow and broad render passes

1. The maintainer opens the debugger and renders a score.
2. The debugger shows the current timing rows plus the flow ids and counts involved in the pass.
3. The maintainer then renders a broader score state with more visible flows.
4. The debugger keeps the same timing categories but now shows the wider prepared and composed scope.
5. The maintainer can compare the passes without guessing from page counts or raw timing totals alone.

### Journey 2: Inspect a local update in one visible flow

1. The maintainer starts from a multi-flow score already visible in the main window.
2. A local edit affects one flow.
3. The render path recalculates and emits diagnostics.
4. The debugger shows whether the recalculation stayed local to the affected flow or broadened to more visible flows.
5. The maintainer can use that evidence to judge whether later invalidation work is actually narrowing scope.

### Journey 3: Trust observability changes not to alter score behavior

1. The maintainer exercises the updated render path across different visible-flow sets.
2. The score output remains visually and semantically unchanged.
3. Existing written-duration and notation-sensitive behaviors still pass their current tests.
4. The maintainer gains more visibility without paying for that visibility through behavior drift.

## Tasks / Subtasks

- [ ] Audit and extend the current render-timing contract rather than creating a new observability path (AC: 1, 2, 3, 4)
  - [ ] Verify the current emit sequence in `src-tauri/src/lib.rs`, `src-tauri/src/instructions/render_plan.rs`, and `src-tauri/src/instructions/mod.rs`, including where flow preparation, composition, and surface diagnostics are already emitted.
  - [ ] Identify the smallest backend-owned diagnostics shape that can describe prepared-flow scope, composed-flow scope, and recalculation scope without changing score output.
  - [ ] Keep cleared/no-render behavior explicit by continuing to emit reset-safe payloads when no render plan or surface is available.
- [ ] Add backend diagnostics for render mode and recalculation scope (AC: 1, 2, 3, 4)
  - [ ] Extend the existing render-plan and/or render-surface diagnostics so they state which flow ids were prepared, composed, and recalculated, plus any compact counts or scope labels that make the pass easy to interpret.
  - [ ] Ensure local updates can report whether recalculation was scoped to one flow or multiple flows, even if the current invalidation model still recomputes more than desired.
  - [ ] Preserve the current timing events so existing debugger consumers do not lose baseline visibility for any render pass.
  - [ ] Keep all FE↔BE diagnostics payloads in `snake_case` and Rust-owned.
- [ ] Surface the expanded diagnostics in the existing debugger window (AC: 1, 2, 3)
  - [ ] Update `src/windows/debugger/components/performance-timings.tsx` to show the new render-mode and recalculation-scope details inside the current layout timing panel or another clearly adjacent existing section.
  - [ ] Preserve the current performance timing rows and severity behavior so the debugger remains comparable before and after this story.
  - [ ] Avoid inventing user-facing score chrome, overlays, or React-owned interpretations of backend render truth.
- [ ] Add focused regression coverage around the observability contract (AC: 1, 2, 3, 4)
  - [ ] Add Rust tests around the diagnostics payload shape in `src-tauri/src/instructions/render_plan.rs` and any other backend seam changed by the implementation.
  - [ ] Extend debugger tests in `src/windows/debugger/components/performance-timings.test.tsx` to prove narrow-scope, broad-scope, and reset scenarios render the expected details.
  - [ ] Keep notation-sensitive regression coverage intact; if any render instrumentation touches parser-adjacent behavior, extend the nearest existing test support rather than creating a new framework.
- [ ] Document the new maintainer-facing instrumentation contract where it materially changes project expectations (AC: 4)
  - [ ] Update `docs/project-context.md` and `docs/architecture.md` if this story introduces a stable diagnostics contract or new required hook points for render-timing visibility.
  - [ ] Update AI-agent instructions only if this story establishes a new required implementation rule for future render instrumentation work.

## Dev Notes

### Story Foundation

- Epic 3 is about preserving notation trust while making multi-flow rendering faster, more isolated, and easier to evolve safely.
- Story 3.1 is the observability foundation for the rest of the epic. Stories 3.3 through 3.5 depend on maintainers being able to see whether recalculation scope is actually narrowing and whether multi-flow work is being measured consistently.
- The epic explicitly says this work must not change written notation behavior, written-duration behavior, or persisted score semantics.

### Developer Context Summary

- `src-tauri/src/lib.rs` already measures flow-preparation time before `generate_render_instructions(...)` and emits both `render-timing-flow-preparation` and `render-timing-flow-preparation-details`.
- `src-tauri/src/instructions/render_plan.rs` already computes `prepared_flow_ids`, `prepared_flow_count`, `composed_flow_ids`, and `composed_flow_count` through `FlowRenderPreparationDiagnostics`, but it does not explicitly describe recalculation scope or present the pass in the clearest maintainer-facing terms.
- `src-tauri/src/instructions/mod.rs` already emits `render-timing-surface-details` plus the current CPU timing breakdown after composing all visible flow segments into one shared surface.
- `src/windows/debugger/components/performance-timings.tsx` already listens to those events and renders flow-aware preparation plus shared-surface details in the debugger layout panel.
- `src/windows/debugger/components/performance-timings.test.tsx` already contains the right regression seam for backend payload changes, including multi-page surface cases and reset-to-null behavior.
- The current gap is not raw timing availability; it is that maintainers still need to infer actual work scope and recalculation scope from adjacent details instead of seeing that directly.

### Technical Requirements

- Preserve the current backend-owned event family for render timings and diagnostics.
- Add an explicit way to describe which flows participated in a pass and what recalculation scope was exercised.
- Add an explicit way to describe recalculation scope for local updates, including which flow ids were recalculated.
- Keep the diagnostics useful when only one flow is visible and when no render plan is available.
- Do not reintroduce a separate single-flow rendering branch, fallback path, or debugger model; the story is about observability over the current renderer design.
- Preserve current score output, hit-target behavior, written-notation behavior, written-duration behavior, and persistence behavior.
- Do not treat debugger-only observability as a reason to move render truth into React state.
- Do not introduce ad hoc console logging, temporary measurement flags, or a second diagnostics surface outside the existing debugger/event model.
- Keep the event payloads compact and scannable for both human maintainers and test assertions.
- Treat this story as a prerequisite for later invalidation work, not as the invalidation optimization itself.

### Architecture Compliance

- Rust remains the source of truth for score state, render orchestration, and diagnostics payload generation.
- Multi-flow presentation remains a derived composition layer over existing flow data; this story should observe that layer, not redesign it.
- Existing frontend/backend command and event boundaries remain in place; the debugger consumes emitted backend state rather than deriving render truth locally.
- Performance work must remain evidence-driven and support future isolated recalculation improvements rather than hard-coding assumptions that all visible flows are always one unit of work.
- No migration or `.soloc` format change is expected. If implementation discovery suggests otherwise, stop and reassess because that would contradict the story boundary.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep FE→BE payload fields in `snake_case` and maintain typed alignment between Rust diagnostics structs and TypeScript listener expectations.
- Reuse the existing debugger component, backend event names, and render pipeline seams before introducing new abstractions.
- No dependency upgrade is expected for this story.
- External web research was not available in this environment; repository-pinned versions and local project docs are the authoritative baseline.

### Project Structure Notes

- Backend render instrumentation belongs in the existing render pipeline under `src-tauri/src/lib.rs`, `src-tauri/src/instructions/render_plan.rs`, and `src-tauri/src/instructions/mod.rs`.
- Native render timing remains under the existing renderer backend, currently `src-tauri/src/renderer/backends/metal.rs`.
- Debugger presentation changes belong in `src/windows/debugger/components/performance-timings.tsx` and its existing tests.
- Shared frontend types should only be extended if a stronger typed contract is needed for the debugger listener code; do not create a feature-local observability store.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/instructions/render_plan.rs`
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/renderer/backends/metal.rs`
- Primary frontend debugger seams:
  - `src/windows/debugger/index.tsx`
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
  - `src/windows/debugger/index.test.tsx`
- Supporting documentation seams if behavior becomes a standing project rule:
  - `docs/project-context.md`
  - `docs/architecture.md`
- Likely out of scope unless implementation proves otherwise:
  - persisted-data migrations
  - score UI chrome in the main window
  - notation parser logic
  - new debugger windows or standalone observability dashboards

### Testing Requirements

- Add or extend Rust tests that prove diagnostics preserve visible flow ordering and correctly report prepared, composed, and recalculated flow scope.
- Add or extend Rust tests that prove reset/no-render paths still emit null-safe diagnostics without panicking or leaving stale frontend state behind.
- Extend `src/windows/debugger/components/performance-timings.test.tsx` to cover the new render-scope and recalculation-scope output for representative narrow and broad flow sets.
- Keep existing debugger coverage for shared-surface details and reset behavior intact.
- Confirm this observability story does not weaken notation-sensitive coverage or introduce a parallel parser test framework.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Git Intelligence Summary

- `4d0dadb8 feat(2.5): preserve active flow across score and sequence` shows the recent implementation pattern of tightening backend-owned contracts and pairing them with targeted UI tests rather than introducing parallel state.
- `7ee83c7c feat(2.4): verify multi-flow popover anchors` shows the repo already prefers focused regression coverage around debugger- and interaction-adjacent seams.
- `928b0376 feat(2.3): simplify selection clearing for deletes` reinforces the local pattern of making source-of-truth behavior explicit instead of relying on implicit inference or cleanup.
- `bca5d7e9 chore: epic 2 retro` indicates Epic 2 is complete, so Epic 3 should start with a narrow enabling story rather than a broad optimization jump.

### Project Context Reference

- `docs/project-context.md` defines the score as a native Rust-rendered surface with React acting as an overlay and debugger shell, and it already calls out the flow-preparation diagnostics and shared-surface render path.
- `_bmad-output/planning-artifacts/prd.md` says maintainers rely on debugger tools and performance visibility to evolve the rendering pipeline safely and requires isolated recalculation to remain evidence-driven.
- `_bmad-output/planning-artifacts/architecture.md` says operational safety for this initiative depends on render-timing instrumentation, targeted regression testing, fixture coverage, and debugger support.
- `_bmad-output/planning-artifacts/epics.md` defines this story as the observability increment that later invalidation and performance stories should build on.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.1 acceptance criteria)
- Source: `_bmad-output/planning-artifacts/prd.md` (Journey 4 maintainer needs, NFR2, NFR3, NFR4, NFR10, AC1, AC3)
- Source: `_bmad-output/planning-artifacts/architecture.md` (instrumentation expectations, Rust ownership, invalidation boundaries, debugger support)
- Source: `docs/project-context.md` (render pipeline stages, flow-preparation diagnostics, shared-surface render path)
- Source: `src-tauri/src/lib.rs` (flow-preparation emit seam)
- Source: `src-tauri/src/instructions/render_plan.rs` (current flow diagnostics payload)
- Source: `src-tauri/src/instructions/mod.rs` (surface diagnostics and per-phase timing emit seam)
- Source: `src/windows/debugger/components/performance-timings.tsx` (debugger timing UI)
- Source: `src/windows/debugger/components/performance-timings.test.tsx` (existing debugger regression coverage)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Existing debugger timing surface: `src/windows/debugger/components/performance-timings.tsx`
- Existing backend emit seams: `src-tauri/src/lib.rs`, `src-tauri/src/instructions/render_plan.rs`, `src-tauri/src/instructions/mod.rs`

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story is intentionally constrained to observability and debugger visibility; notation, persistence, and migration changes are out of scope.
- External web research was unavailable; local repository artifacts and current source were used as the authoritative baseline.

### File List

- `_bmad-output/implementation-artifacts/3-1-add-multi-flow-render-and-recalculation-visibility.md`
