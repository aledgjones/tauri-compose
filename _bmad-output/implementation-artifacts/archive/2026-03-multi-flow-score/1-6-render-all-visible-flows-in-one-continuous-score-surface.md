# Story 1.6: Render All Visible Flows in One Continuous Score Surface

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want score view to render every visible flow in sequence on the shared score surface,
so that the multi-flow reading model extends from the initial two-flow slice to the full score I chose to display.

## Acceptance Criteria

1. Given a layout contains more than two visible flows, when the score view renders through the updated multi-flow path, then all visible flows in the layout are composed into the shared score surface in `score.flows.order`, and no arbitrary two-flow cap truncates the visible reading path.
2. Given all visible flows are composed into one score surface, when the combined rendered extent exceeds the current visible viewport, then the user can continue reading across the full visible-flow sequence through the existing shared navigation model, and the score view does not require any top-bar flow picker for basic score reading.
3. Given a layout contains one or two visible flows, when the updated render path runs, then existing single-flow and two-flow behavior remains correct, and the earlier Epic 1 presentation refinements remain intact.
4. Given all visible flows are rendered in sequence, when the score view is shown at compact desktop width with secondary panels collapsed, then the score remains the primary work surface, and the shell does not switch to a different navigation model to reach later visible flows.
5. Given all-visible-flow rendering is validated, when notation and performance are reviewed end to end, then notation-derivation and persisted score semantics remain unchanged, and the expanded composed surface remains compatible with the broader multi-flow performance and recalculation guardrails planned in later epics.

## Tasks / Subtasks

- [x] Remove the hard two-flow composition cap from the render plan while preserving ordering semantics and correct one-flow behavior (AC: 1, 3, 5)
  - [x] Audit `src-tauri/src/instructions/render_plan.rs` and replace the current `MAX_COMPOSED_VISIBLE_FLOWS`-based composition limit so `FlowRenderPlan` composes every visible prepared flow in `score.flows.order`.
  - [x] Keep one-flow and hidden-active-flow cases deliberate while simplifying the render plan so visible prepared flows continue through the shared-surface composition path rather than a redundant fallback-only branch.
  - [x] Ensure `prepared_flow_ids`, `composed_flow_ids`, and composition diagnostics remain truthful for zero-flow, one-flow, two-flow, and 3+-flow layouts.
- [x] Extend the existing shared-surface composition path from two flows to all visible flows (AC: 1, 2, 3, 4, 5)
  - [x] Audit `generate_render_instructions(...)` in `src-tauri/src/instructions/mod.rs` so it builds one native render segment per composed flow and passes the full visible-flow list into `compose_render_segments(...)`.
  - [x] Reuse the existing segment-composition loop, page-per-flow white page boxes, inter-flow gap, and shared hit-target offset model instead of introducing per-flow panels, tabs, or a new frontend-owned layout concept.
  - [x] Keep `renderer.width` and `renderer.height` derived from the final composed surface so Story 1.3 pan and zoom constraints continue to treat the shared surface as the single authoritative horizontal extent.
- [x] Generalize shared-surface diagnostics and debugger verification beyond two page boxes (AC: 1, 2, 4, 5)
  - [x] Replace the current two-page-only surface diagnostics shape in `src-tauri/src/instructions/mod.rs` with a payload that can describe every composed page box rather than only `first_page_*` and `second_page_*` fields.
  - [x] Update `src/windows/debugger/components/performance-timings.tsx` and its tests so the debugger can display all composed page offsets and widths for 3+-flow layouts while preserving the existing surface-width, viewport, and inter-flow-gap evidence.
  - [x] Keep the debugger timing surface as the primary verification UI; do not add a separate flow-navigation diagnostics panel.
- [x] Protect established Epic 1 reading behavior while expanding flow count (AC: 2, 3, 4, 5)
  - [x] Preserve the Story 1.4 page-per-flow visual treatment and Story 1.3 shared pan path; later visible flows must become reachable through the same existing navigation model rather than a new picker or shell mode.
  - [x] Keep notation derivation, written-duration behavior, per-flow page-start chrome, and persisted score semantics unchanged.
  - [x] Keep Story 1.5's rejected flow-label and expanded bottom-bar direction out of scope; this story is strictly about removing the visible-flow count cap from the existing shared surface.
- [x] Add focused regression coverage and end-to-end verification for all-visible-flow composition (AC: 1, 2, 3, 4, 5)
  - [x] Add Rust tests near `src-tauri/src/instructions/render_plan.rs` for 3+-flow ordering, composed-flow diagnostics, and fallback behavior when only one visible flow exists or the active flow is hidden.
  - [x] Add Rust seam tests near `src-tauri/src/instructions/mod.rs` to verify page offsets, page box count, shared-surface extents, and hit-target offsets remain correct when composing more than two segments.
  - [x] Update `src/windows/debugger/components/performance-timings.test.tsx` for the generalized surface-details payload and 3+-page display.
  - [x] Validate with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- [x] Update docs if all-visible-flow composition becomes part of the stable renderer contract
  - [x] Update `docs/architecture.md` and `docs/project-context.md` if the removal of the two-flow cap and the generalized shared-surface diagnostics become documented engineering rules.

## Dev Notes

### Story Foundation

- Story 1.2 established the first shared native score surface by composing the first two visible flows into one renderer surface with a deterministic inter-flow gap.
- Story 1.3 proved the combined surface width remains reachable through the existing native pan path without introducing a separate frontend navigation model.
- Story 1.4 refined the presentation into page-per-flow white page boxes on the shared surface so adjacent flows read as one coherent document while keeping the inter-flow backdrop gap visible.
- Story 1.5 was closed and fully reverted after UX review rejected passive page labels and expanded bottom-bar flow context. Story 1.6 must not revive that direction; it should expand the successful shared-surface model rather than layer on rejected orientation chrome.
- The practical definition of done is: every visible flow in the current layout is rendered in `score.flows.order` on the same native shared surface, the existing pan path can reach the full composed width, one-flow and two-flow behavior remain intact, and debugger evidence remains trustworthy for the wider composition.

### Developer Context Summary

- Before Story 1.6, the live hard cap was in `src-tauri/src/instructions/render_plan.rs`: `MAX_COMPOSED_VISIBLE_FLOWS` was set to `2`, `FlowRenderPlan::composed_flow_ids()` used `.take(MAX_COMPOSED_VISIBLE_FLOWS)`, and a single-flow fallback branch handled the pre-shared-surface path.
- Story 1.6 intentionally removes that cap and simplifies the old fallback-only branch so the shared-surface composition seam remains authoritative even when only one visible flow is prepared. Reuse that seam; do not invent a second visible-flow registry.
- `compose_render_segments(...)` in `src-tauri/src/instructions/mod.rs` already iterates over an arbitrary `Vec<FlowRenderSegment>` and composes page backgrounds, draw instructions, hit targets, offsets, combined width, and height in one loop. The core composition helper is already N-flow capable; the main current limitation is which segments get passed into it and how diagnostics are reported.
- `build_flow_render_segment(...)` already computes each flow's full native page-start chrome, notation, and hit targets. Story 1.6 should keep those per-flow page starts intact for every visible flow rather than special-casing later flows.
- `render-timing-surface-details` is currently two-page-specific in both Rust and the debugger UI. `RenderSurfaceDiagnostics` in `src-tauri/src/instructions/mod.rs` exposes only `first_page_*` and `second_page_*` fields, and `src/windows/debugger/components/performance-timings.tsx` renders only page 1 and page 2 details. That shape must be generalized if 3+-flow composition is to remain verifiable.
- `RendererContext` pan and zoom behavior still depends on `renderer.width` and `renderer.height` from the composed native surface. Story 1.6 should continue to make backend-owned surface extents the single source of truth for reachability.
- The React renderer overlay remains an interaction/reporting layer only. Do not move flow composition, flow ordering, or scroll-range ownership into React.

### Previous Story Intelligence

- Reuse the Story 1.2 flow-plan and shared-surface composition seam instead of adding separate per-flow containers, tabs, or a top-bar flow picker.
- Preserve `score.flows.order` as the ordering contract. `layout.flow_ids` determines visibility, not display order.
- Preserve the Story 1.4 page-per-flow box treatment and the fixed native inter-flow gap while extending the number of composed flows.
- Keep the Story 1.3 shared pan model and debugger verification path. Wider composition should expose more of the same native surface, not create a second navigation architecture.
- Active-flow editing semantics remain separate from visible multi-flow presentation. Story 1.6 expands what is readable, not how editing scope or hit-testing semantics work.
- Do not reintroduce Story 1.5's rejected passive page labels or broader bottom-bar flow context while implementing this expansion.

### Technical Requirements

- Keep Rust authoritative for visible-flow composition, page offsets, draw instructions, hit targets, renderer extents, and diagnostics.
- Compose every visible prepared flow in `score.flows.order`, including one-flow layouts, so user-visible single-flow behavior remains correct without preserving a separate fallback-only render branch.
- Reuse the existing per-flow render-segment path and `compose_render_segments(...)` helper rather than cloning the parse/draw pipeline into separate branches.
- Preserve the Story 1.4 page-per-flow white page boxes and the existing inter-flow gap across all composed flows. Later flows should remain individual native pages on the same shared surface.
- Keep the existing one-surface navigation model. No new top-bar picker, no per-flow viewport switching, and no frontend-owned scroll state model.
- Keep written durations, spacing semantics, clef/key/time/tempo derivation, and persisted `.soloc` semantics unchanged. This story is a presentation-cap expansion, not a notation rewrite or data-model change.
- Treat diagnostics as part of the implementation contract. The wider composition path must remain reviewable through trustworthy debugger evidence, especially for page count, page offsets, surface width, and viewport reachability.
- Be mindful of later Epic 3 performance work: this story can increase composed-flow count, but it should not entangle that with a new invalidation architecture.
- No migration or persisted-data change is expected for this story.

### Architecture Compliance

- Multi-flow support remains a derived presentation/composition layer over existing flow data. Do not introduce a new persisted score structure or migration.
- The score must continue to render through Rust/Skia/Metal. Do not render notation, page boxes, or flow-order chrome in React/HTML.
- Preserve the existing frontend/backend command boundary and snake_case payload conventions if diagnostics payloads change.
- Keep active-flow semantics distinct from visible-flow presentation. This story removes a reading-path cap; it does not redefine editing ownership.
- Keep future page-layout and recalculation work feasible. Avoid hard-coding new assumptions that would block later invalidation/performance stories.

### Library / Framework Requirements

- Stay within the existing stack already in the repo: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, Rust 2021, `skia-safe` 0.93.x, `rayon` 1.10, and `rustc-hash`.
- Preserve snake_case across the frontend/backend boundary and keep event payloads typed and minimal.
- Prefer extending existing renderer/composition/debugger helpers before adding new abstractions.
- No dependency upgrade is expected for this story.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/instructions/render_plan.rs`
  - `src-tauri/src/instructions/mod.rs`
- Supporting debugger seams:
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
- Supporting docs if the contract changes:
  - `docs/architecture.md`
  - `docs/project-context.md`
- Likely out of scope unless implementation proves otherwise:
  - persistence and migrations
  - write/sequence shell navigation redesign
  - flow labels or bottom-bar flow-context UX
  - Epic 2 hit-testing or active-flow interaction work

### Testing Requirements

- Add Rust tests in `src-tauri/src/instructions/render_plan.rs` for 3+-flow prepared/composed ordering, visible-flow counts, and truthful composition diagnostics across zero-flow, one-flow, and hidden-active-flow cases.
- Add Rust seam tests in `src-tauri/src/instructions/mod.rs` for multi-segment page offsets, page-box counts, combined width/height, and hit-target offset correctness beyond two flows.
- Update debugger Vitest coverage in `src/windows/debugger/components/performance-timings.test.tsx` for generalized page diagnostics and 3+-flow rendering evidence.
- Validate visually with:
  - one visible flow to confirm unchanged single-flow behavior
  - two visible flows to confirm no regression in Stories 1.2 through 1.4 behavior
  - three or more visible flows at compact desktop width to confirm the later pages are reachable through the existing shared navigation model
- Validation target:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Git Intelligence Summary

- The most relevant recent implementation commits are `fc46ccdf feat(1.4): refine adjacent flow score coherence`, `1ee5350a feat(1.4): add per-flow page backgrounds`, and `022d3573 feat(1.3): renderer scrolling`.
- Those commits confirm the intended extension path for Story 1.6: build on the current shared-surface composition, authoritative renderer width, and page-per-flow background treatment rather than replacing them with a new UI model.
- The most recent commit, `83071eae chore(story): close rejected flow label direction`, is also relevant guardrail context: the rejected Story 1.5 orientation treatment should not leak back into this story.

### Project Context Reference

- `docs/project-context.md` requires native Rust score rendering, Rust-owned authoritative state, snake_case FE/BE contracts, and backend-owned pan/zoom/render behavior.
- `_bmad-output/planning-artifacts/architecture.md` treats multi-flow rendering as a derived presentation layer over existing flow data and explicitly separates composition concerns from notation semantics, interaction contracts, and future invalidation work.
- The current shared-surface pipeline already supports ordered visible-flow preparation, per-flow parse/draw work, and shared-surface composition; Story 1.6 should extend that same pipeline to all visible flows instead of creating a parallel presentation path.
- The debugger timing surface is the established maintainer verification UI for render-path changes. If Story 1.6 widens composition, the debugger evidence must widen with it.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.6)
- Source: `_bmad-output/planning-artifacts/architecture.md` (derived multi-flow composition, Rust/React boundary, instrumentation expectations, future invalidation concerns)
- Source: `docs/project-context.md` (native renderer ownership, flow-preparation phase, FE/BE boundary, shared-surface render pipeline)
- Source: `_bmad-output/implementation-artifacts/1-2-render-two-flows-in-one-score-surface-with-fixed-placement.md` (two-flow composition precedent)
- Source: `_bmad-output/implementation-artifacts/1-3-scroll-across-the-combined-multi-flow-score-width.md` (shared pan/reachability precedent)
- Source: `_bmad-output/implementation-artifacts/1-4-make-adjacent-flows-read-as-one-coherent-score.md` (page-per-flow box and boundary precedent)
- Source: `_bmad-output/implementation-artifacts/1-5-add-flow-labels-to-multi-flow-score-view.md` (rejected orientation direction to keep out of scope)
- Source: `src-tauri/src/instructions/render_plan.rs` (current two-flow cap and composed-flow diagnostics)
- Source: `src-tauri/src/instructions/mod.rs` (`build_flow_render_segment`, `compose_render_segments`, and current two-page surface diagnostics)
- Source: `src/windows/debugger/components/performance-timings.tsx` (current two-page debugger surface details)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Use the debugger window's `PerformanceTimings` panel as the primary verification surface for prepared-flow versus composed-flow evidence and page-box offsets.
- Manual verification should include one-flow, two-flow, and 3+-flow layouts at compact desktop width to confirm unchanged earlier behavior and newly reachable later visible flows.

### Implementation Plan

- Remove the two-flow cap only at the flow-planning seam so the rest of the shared-surface render pipeline can keep using the existing per-flow segment composition path.
- Keep single-flow and hidden-active-flow behavior correct while simplifying the render plan to a single shared-surface composition path rather than preserving the redundant pre-shared-surface branch indefinitely.
- Generalize the surface diagnostics payload to per-page-box entries instead of adding any new frontend-owned flow navigation concept.
- Cover the change with render-plan regressions, multi-segment seam tests, debugger Vitest assertions, and the full repo validation commands.

### Completion Notes List

- Story created from the explicit user-selected backlog item `1.6`.
- Previous story learnings were incorporated from Stories 1.2 through 1.5, the current architecture/project-context documents, and the live renderer/debugger seams in the repository.
- No dependency upgrade is expected for this story.
- No migration or persisted-data change is expected for this story.
- Story scope intentionally excludes reviving the rejected Story 1.5 flow-label and bottom-bar direction.
- Removed the hard two-flow cap from `FlowRenderPlan` so every visible prepared flow now composes in `score.flows.order`.
- Follow-up cleanup removed the explicit single-flow fallback path as well, so one visible flow now uses the same shared-surface composition path as multi-flow layouts.
- Kept the existing shared-surface composition path intact in Rust so each visible flow still contributes its own native page box, page-start chrome, hit-target offsets, and authoritative shared renderer extents.
- Replaced the two-page-only surface diagnostics payload with a generalized `page_boxes` list and updated the debugger timing panel to display every composed page offset and width.
- Added Rust regression coverage for 3+-flow ordering, hidden-active-flow behavior, and single-flow shared-surface composition, added seam tests for 3-page composition offsets and hit-target offsets, and expanded debugger Vitest coverage for generalized page-box diagnostics.
- Updated `docs/architecture.md` and `docs/project-context.md` so the documented renderer contract now reflects all-visible-flow shared-surface composition.
- Validation completed: `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Senior Developer Review (AI)

- 2026-03-11: Review clarified that removing the explicit single-flow fallback branch was intentional. The requirement was to preserve correct single-flow behavior after multi-flow expansion, not to keep the older redundant render path indefinitely.
- 2026-03-11: Story wording was updated during review to reflect the intended shared-surface simplification, truthful composition diagnostics, and the fact that no implementation change was required.

### Change Log

- 2026-03-11: Removed the visible-flow composition cap from the shared Rust render path, generalized surface diagnostics to per-page-box payloads, expanded Rust/Vitest regression coverage, and updated the renderer contract docs.
- 2026-03-11: Simplified single-flow rendering to use the same shared-surface composition path as multi-flow layouts and removed fallback-only debugger diagnostics.
- 2026-03-11: Clarified the story and review record so single-flow correctness is treated as a behavior-preservation check, not a requirement to retain the redundant legacy render branch.

### File List

- docs/architecture.md
- docs/project-context.md
- src-tauri/src/instructions/mod.rs
- src-tauri/src/instructions/render_plan.rs
- src/windows/debugger/components/performance-timings.test.tsx
- src/windows/debugger/components/performance-timings.tsx
- \_bmad-output/implementation-artifacts/1-6-render-all-visible-flows-in-one-continuous-score-surface.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
