# Story 1.3: Scroll Across the Combined Multi-Flow Score Width

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want horizontal scrolling to move across the combined rendered width of visible flows,
so that the first multi-flow score surface can be read as one continuous workspace instead of separate fixed panels.

## Acceptance Criteria

1. Given a score where two flows are rendered in the same score surface, when the combined rendered width exceeds the current visible viewport, then horizontal scrolling moves across the full combined score width, and both flows remain reachable through the existing score-view navigation pattern.
2. Given a score with only one flow, when the score view renders through the updated scrolling path, then horizontal scrolling remains correct for the existing single-flow case, and the existing score-navigation inputs continue to work through the same score-view path.
3. Given a score with two rendered flows, when a user scrolls from the first flow toward the second, then the movement feels continuous across the shared score surface, and the horizontal scroll range reaches the full rendered extent of both visible flows without truncating the second flow.
4. Given the score view is shown at compact desktop width with secondary panels collapsed, when the combined rendered width exceeds the available viewport, then both visible flows remain reachable through the same horizontal navigation model, and the shell does not require an alternate navigation pattern to access the second flow.
5. Given the combined multi-flow score width is scrollable, when this increment is validated, then rendering remains stable while scrolling, and written-notation behavior and persisted score semantics remain unchanged.

## Tasks / Subtasks

- [x] Make renderer pan constraints respect the full composed surface width for multi-flow scores (AC: 1, 2, 3, 4, 5)
  - [x] Audit the Story 1.2 composition path so the final `renderer.width` coming from the composed surface is the single authoritative horizontal extent used by pan clamping and native rendering.
  - [x] Adjust `RendererContext::constrain_pan(...)` and any supporting renderer-bound calculations only where needed so a viewport narrower than the combined surface can pan from the leftmost readable starting position to the full rightmost extent of the second flow.
  - [x] Preserve the current centering/clamping behavior when the rendered score width is smaller than the viewport, so single-flow layouts and wide monitor cases do not regress.
- [x] Preserve one continuous navigation model in the frontend overlay (AC: 1, 2, 3, 4)
  - [x] Keep wheel and trackpad scrolling routed through the existing `actions.ui.renderer.setPan(...)` path; do not add per-flow panels, tabs, or alternate navigation controls.
  - [x] Confirm pointer-drag panning and wheel horizontal deltas operate against the same shared renderer surface so moving from flow one into flow two feels continuous.
  - [x] Keep React as an overlay only; do not mirror composed-flow width or scroll state into new frontend-owned score models.
- [x] Maintain render and interaction stability while scrolling the wider surface (AC: 2, 3, 5)
  - [x] Verify Story 1.2 shared-surface hit targets remain valid as pan offsets change across the extended horizontal range.
  - [x] Preserve the current zoom-around-center behavior and vertical panning rules while enabling the wider horizontal travel.
  - [x] Avoid introducing new notation spacing, layout-balancing, or active-flow semantics changes; this story is about reachability across the existing shared surface.
- [x] Extend diagnostics and manual verification for scrollable multi-flow width (AC: 1, 3, 4, 5)
  - [x] Extend the existing debugger timing/details surface only if needed to expose the combined renderer width, viewport bounds, or other data maintainers need to confirm the second flow is reachable without truncation.
  - [x] Add manual verification notes for a two-flow score at compact desktop width and a one-flow score to confirm unchanged single-flow behavior.
  - [x] Keep the debugger surface as the verification UI rather than introducing a dedicated scrolling diagnostics panel.
- [x] Add focused regression coverage and validation (AC: 1, 2, 3, 4, 5)
  - [x] Add Rust tests around horizontal pan clamping for viewports narrower than the composed multi-flow width, including left-edge, right-edge, and centered single-flow cases.
  - [x] Add or update frontend and/or backend tests covering wheel or pan interactions only where current coverage would otherwise miss regressions in the shared navigation path.
  - [x] Validate with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- [x] Update docs if scrollability becomes part of the stable renderer contract
  - [x] Update `docs/architecture.md` and `docs/project-context.md` if the renderer-width or pan-clamping behavior becomes a documented implementation rule for multi-flow presentation.

## Dev Notes

### Story Foundation

- Epic 1 now has a shared two-flow renderer surface from Story 1.2, but that visible composition only becomes useful if the user can traverse the entire composed width through the same score navigation model they already use.
- This story should extend the current shared-surface implementation into a genuinely readable horizontal workspace, without taking on later visual-coherence, orientation-cue, or cross-flow interaction stories.
- The safest definition of done is: the combined renderer width is fully reachable through the existing pan/scroll path, single-flow behavior remains intact, and maintainers can verify the wider travel range without a separate UI model.

### Developer Context Summary

- Story 1.2 already composes up to the first two prepared flows into one shared `RendererContext` surface in `src-tauri/src/instructions/mod.rs`, with combined `renderer.width`, `renderer.height`, shared draw instructions, and shared hit targets.
- `RendererContext::constrain_pan(...)` in `src-tauri/src/renderer/mod.rs` is the current clamp seam for both wheel scrolling and pointer-drag panning. It converts renderer-space page dimensions to CSS pixels, applies zoom, and clamps pan independently on each axis.
- `ui_renderer_pan_set(...)` and `ui_renderer_zoom_set(...)` in `src-tauri/src/ui/renderer.rs` already delegate pan and zoom constraints to the native renderer context. This story should keep that backend-owned constraint model intact.
- The React overlay in `src/windows/main/components/renderer.tsx` already sends wheel deltas and pointer drag updates through `actions.ui.renderer.setPan(...)`, using the same surface for single-flow and multi-flow navigation.
- Current clamping behavior centers content when the scaled rendered size plus margins fits inside the viewport. The multi-flow case likely needs validation or refinement at the boundary where the composed width exceeds the viewport while the single-flow case must keep the existing centering behavior.
- Story 1.2 also established debugger diagnostics for flow preparation/composition counts. If scroll verification needs more evidence, extend that existing surface instead of inventing a new diagnostics path.

### Previous Story Intelligence

- Reuse the Story 1.2 shared-surface seam instead of introducing separate scroll containers or frontend-owned flow panels.
- Keep flow ordering tied to `score.flows.order` and the prepared/composed flow plan. Scrolling should expose more of the already composed surface, not reinterpret which flow comes next.
- Extend existing renderer/debugger verification paths only where they help confirm reachability or clamping behavior.
- Preserve single-flow fallback behavior and the current active-flow model; Epic 2 owns broader interaction changes.
- Treat documentation updates as part of implementation if renderer-width and scroll-clamping semantics become stable engineering rules.

### Technical Requirements

- Keep Rust authoritative for score-surface size, pan limits, and native render output. Do not move scroll-range ownership into React.
- Treat the composed surface width produced by Story 1.2 as the primary input to scrolling behavior; avoid duplicating width calculations in frontend code.
- Preserve the existing one-surface navigation model for wheel, trackpad, and pointer-drag input.
- Ensure the rightmost extent of the second composed flow becomes reachable when the viewport is narrower than the combined width.
- Preserve current single-flow centering/clamping semantics when content does not overflow horizontally.
- Keep written durations, notation spacing semantics, and persisted score behavior unchanged; this story must not become a notation-layout rewrite.
- Avoid taking on visual boundary refinements, active-flow cues, selection updates, or popover behavior changes that belong to later stories.

### Architecture Compliance

- Rust remains the single source of truth for score state, renderer extents, pan constraints, and native score rendering.
- The score must continue to render through Rust/Skia/Metal. Do not render notation or scrolling surfaces in React/HTML.
- Multi-flow support remains a derived presentation layer over existing flow data. No persisted score-structure change or migration is expected for this story.
- The existing frontend/backend command boundary remains in force: the frontend sends input deltas, while the backend decides the valid pan result.
- Active-flow semantics remain distinct from visible-flow presentation; this story improves reachability, not editing scope.
- Keep future page-layout and broader multi-flow evolution feasible. Do not hard-code a scrolling model that assumes the current two-flow composition is the final layout architecture.

### Library / Framework Requirements

- Stay within the existing stack already in the repo: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, Rust 2021, `skia-safe` 0.93.x, `rayon` 1.10, and `rustc-hash`.
- Preserve snake_case across the frontend/backend boundary and keep Tauri command/event payloads typed and minimal.
- Use existing renderer converter, pan-clamp, and overlay interaction helpers before introducing new abstractions.
- No dependency upgrade is expected for this story.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/renderer/mod.rs`
  - `src-tauri/src/ui/renderer.rs`
  - `src-tauri/src/lib.rs`
- Primary frontend seam:
  - `src/windows/main/components/renderer.tsx`
- Supporting diagnostics/tests if verification changes are needed:
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
- Likely out of scope for Story 1.3 unless implementation proves otherwise:
  - persistence and migrations
  - flow ordering logic in `render_plan.rs`
  - selection/popover workflows
  - write-screen properties UX and active-flow indicators

### Testing Requirements

- Add Rust tests close to the renderer clamp/composition seam for horizontal overflow behavior, right-edge reachability, and non-overflow centering.
- Add or update tests only where needed to prove the existing wheel/pan path still works for both one-flow and two-flow surfaces.
- Validate that a one-flow score still behaves correctly through the same pan/zoom path.
- Validation target:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Git Intelligence Summary

- The most relevant recent implementation commit is `feat: render two flows on a shared score surface`, which established the combined renderer surface this story must now make fully navigable.
- Earlier recent commits are planning/documentation oriented, so the main implementation precedent remains the current Story 1.2 code already in the working tree.
- Story 1.3 should therefore deepen the shared-surface pattern rather than changing course toward a different navigation architecture.

### Project Context Reference

- The project rules in `docs/project-context.md` require native Rust score rendering, Rust-owned authoritative state, snake_case FE/BE contracts, and preserving existing pan/zoom/render ownership in the backend.
- The current architecture document treats multi-flow support as a derived presentation/composition layer over existing flow data, with React remaining an orchestration shell rather than the owner of score truth.
- `RendererContext` already caches instructions and applies pan/zoom without recalculation on every movement. This story should preserve that model while expanding the usable horizontal range.
- The debugger timing surface is the established maintainer verification point for render-path changes.
- Documentation updates are required if wider-surface pan constraints become part of the stable engineering contract.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.3)
- Source: `_bmad-output/planning-artifacts/architecture.md` (multi-flow presentation composition, React/Rust boundary, render instrumentation, project structure mapping)
- Source: `docs/project-context.md` (native renderer ownership, flow composition pipeline, FE/BE boundary, renderer cache and pan/zoom rules)
- Source: `src-tauri/src/instructions/mod.rs` (shared two-flow surface composition and combined renderer extents)
- Source: `src-tauri/src/renderer/mod.rs` (`RendererContext::constrain_pan`, renderer width/height ownership, cached instruction surface)
- Source: `src-tauri/src/ui/renderer.rs` (backend-owned pan/zoom commands)
- Source: `src/windows/main/components/renderer.tsx` (wheel and drag input routed through the renderer pan path)
- Source: `_bmad-output/implementation-artifacts/1-2-render-two-flows-in-one-score-surface-with-fixed-placement.md` (previous story learnings and existing diagnostics contract)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Use the debugger window's `PerformanceTimings` panel as the primary verification surface for composed-flow diagnostics and any added reachability evidence.
- Manual verification should include a one-flow score and a two-flow score at compact desktop width to confirm unchanged single-flow behavior and full second-flow reachability.
- Manual verification notes:
  - One-flow score: confirm the renderer remains centered/clamped correctly when content fits in the viewport and the existing score navigation path behaves unchanged.
  - Two-flow score: confirm wheel/trackpad or pointer-drag scrolling can reach the far edge of the second flow without truncation, while rendering remains visually stable during movement.

### Implementation Plan

- Audit the renderer-width and pan-clamp seam from Story 1.2, then tighten the clamp logic only where the composed surface currently fails to expose its full horizontal extent.
- Preserve the existing frontend input path so all scrolling continues to flow through the same backend-owned pan commands.
- Add focused regression coverage around overflow versus non-overflow pan behavior and extend debugger evidence only if needed to verify reachability.
- Update architecture/project context docs only if this story makes wider-surface pan semantics a documented stable contract.

### Completion Notes List

- Story created automatically from the first backlog entry in `sprint-status.yaml` after Story 1.2 was marked done.
- Previous story learnings were incorporated from the Story 1.2 implementation artifact and the current renderer/navigation code.
- No external dependency upgrade is expected for this story.
- No migration or persisted-data change is expected for this story.
- Audit result: the Story 1.2 shared-surface path already kept the composed `renderer.width` authoritative for native pan clamping and rendering, so this increment focused on regression proof rather than a clamp rewrite.
- Added debugger surface diagnostics for shared-surface width and viewport width so maintainers can confirm horizontal reachability from the existing debugger UI.
- Cleared shared-surface debugger diagnostics whenever the backend resets render-plan diagnostics so stale viewport/surface values do not persist after the renderer falls back to an empty state.
- Added Rust clamp coverage for wide composed surfaces and non-overflow centering, plus frontend coverage proving wheel and pointer-drag input both stay on the existing `actions.ui.renderer.setPan(...)` path.
- Validation passed with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- Manual verification notes retained for review: use a one-flow score to confirm centering remains stable when content fits, and a two-flow score at compact desktop width to confirm the second flow remains reachable through the shared horizontal navigation path.
- Senior review follow-up fixes applied: the debugger reset path now clears `render-timing-surface-details`, the story File List now includes `src-tauri/src/instructions/render_plan.rs`, and the File List no longer claims an untouched `src-tauri/src/ui/renderer.rs` change.

### File List

- \_bmad-output/implementation-artifacts/1-3-scroll-across-the-combined-multi-flow-score-width.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- docs/architecture.md
- docs/project-context.md
- src-tauri/src/instructions/mod.rs
- src-tauri/src/instructions/render_plan.rs
- src-tauri/src/lib.rs
- src-tauri/src/renderer/mod.rs
- src/windows/debugger/components/performance-timings.test.tsx
- src/windows/debugger/components/performance-timings.tsx
- src/windows/main/components/renderer.test.tsx

## Senior Developer Review (AI)

### Reviewer

GPT-5.4

### Date

2026-03-11

### Outcome

Approved after fixes.

### Findings

- Fixed stale debugger diagnostics by clearing `render-timing-surface-details` whenever render-plan diagnostics reset in `src-tauri/src/lib.rs`.
- Corrected the Dev Agent Record File List to match the actual implementation change set, including the formatter-touched `src-tauri/src/instructions/render_plan.rs` test update and removing the unmodified `src-tauri/src/ui/renderer.rs` entry.
- Re-ran validation claims for lint, typecheck, JS tests, Rust tests, and clippy in the current workspace.

## Change Log

- 2026-03-11: Created Story 1.3 with implementation guidance for shared-surface horizontal reachability, backend-owned pan constraints, stability guardrails, and focused regression coverage.
- 2026-03-11: Verified the shared composed renderer width remains the authoritative native pan-clamp extent, added debugger surface diagnostics, expanded Rust and React regression coverage for shared scroll/pan behavior, and documented the stable renderer-width contract.
- 2026-03-11: Applied senior-review follow-up fixes to clear stale shared-surface debugger diagnostics, corrected the story File List to include `src-tauri/src/instructions/render_plan.rs`, and marked the story done after revalidation.
