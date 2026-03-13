# Story 1.2: Render Two Flows in One Score Surface with Fixed Placement

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want the score view to show two flows in the same rendered surface,
so that multi-flow reading begins as a concrete visible capability without taking on final layout behavior too early.

## Acceptance Criteria

1. Given a score with at least two flows, when the score view renders after the flow-aware composition work is in place, then the first two flows appear in the same score surface, and each flow is rendered in sequence using a simple deterministic horizontal placement rule.
2. Given the first two flows appear in the same score surface, when a composer reads from the end of the first flow toward the start of the second, then the transition is visually understandable as one continuous score workspace, and the user does not need a separate flow-switching action to continue reading.
3. Given the first two flows are rendered in the same score surface, when the score is displayed, then the second flow is visually distinct from the first, and the placement is sufficient to avoid overlap between the two rendered flows.
4. Given a score with only one flow, when the score view renders through the updated path, then the score output remains correct and usable, and the existing single-flow rendering path remains available through the same score-view workflow.
5. Given two flows are rendered in the same score surface, when this first visible multi-flow slice is validated, then written-duration and notation-derivation behavior remains unchanged, and the rendered score shows a fixed visible gap between the two flows that is sufficient to distinguish the boundary at normal working zoom.

## Tasks / Subtasks

- [x] Compose the first two prepared flows into one backend-rendered score surface (AC: 1, 2, 3, 4)
  - [x] Extend the Rust render composition seam introduced in Story 1.1 so `generate_render_instructions` can render up to the first two prepared flows from `FlowRenderPlan` into one shared `RendererContext` surface.
  - [x] Keep flow ordering deterministic by taking visible flows from `prepared_flows` in `score.flows.order`, never from `layout.flow_ids` iteration order.
  - [x] Preserve the existing single-flow fallback when fewer than two visible flows are available.
- [x] Introduce a simple fixed-placement rule with a verifiable boundary gap (AC: 1, 2, 3, 5)
  - [x] Define a single fixed inter-flow gap that is at least 20 CSS pixels at normal zoom, but express the placement in renderer space so the native pipeline remains authoritative.
  - [x] Position the second rendered flow to the right of the first flow's rendered extent plus the fixed gap, without overlapping page backgrounds, staves, names, braces, or score symbols.
  - [x] Keep the placement rule deliberately simple and deterministic for this story; do not take on final spacing-balancing or page-layout behavior yet.
- [x] Refactor the single-flow renderer into reusable per-flow composition units (AC: 1, 3, 4, 5)
  - [x] Extract the current single-flow parse/draw body in `src-tauri/src/instructions/mod.rs` into an internal helper that can build one flow segment's instructions, hit targets, width, and height without immediately clearing the whole renderer.
  - [x] Merge two flow segments into one final instruction list by offsetting the second segment's draw instructions and any associated hit-target/popover coordinates into the combined surface coordinate space.
  - [x] Set final `renderer.width` and `renderer.height` from the combined surface extents so existing pan and zoom constraints automatically use the wider score page.
- [x] Preserve current interaction and active-flow contracts while making the second flow visible (AC: 2, 4, 5)
  - [x] Keep Rust as the source of truth for visible-flow composition and active-flow semantics; do not mirror multi-flow composition state into React.
  - [x] Preserve the existing active-flow editing model from Story 1.1. If second-flow hit targets are emitted, ensure they remain flow-correct and deterministic; if interaction stays active-flow-only for now, make that behavior an intentional, documented constraint rather than an accident.
  - [x] Avoid introducing broader cross-flow selection, popover, or active-flow behavior changes that belong to Epic 2.
- [x] Extend diagnostics and verification for visible two-flow composition (AC: 1, 3, 4, 5)
  - [x] Extend the existing flow-preparation/debug timing output so maintainers can verify which visible flows were actually composed into the surface and whether single-flow fallback was used.
  - [x] Keep the debugger timing surface as the primary verification UI rather than creating a separate diagnostics panel.
  - [x] Add manual verification notes covering a one-flow score and a two-flow score at normal zoom.
- [x] Add focused regression coverage and validation (AC: 1, 2, 3, 4, 5)
  - [x] Add Rust tests around two-flow composition order, fixed-gap placement, combined width/height calculation, and single-flow fallback behavior.
  - [x] Add or update hit-test and/or renderer tests to prove any composed-flow coordinate offsets stay consistent with the combined surface.
  - [x] Update debugger/Vitest coverage if preparation/composition diagnostics change.
  - [x] Validate with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- [x] Update docs if stable render-surface composition contracts change
  - [x] Update `docs/architecture.md` and `docs/project-context.md` if two-flow surface composition, fixed-gap expectations, or debugger verification become part of the stable development contract.

## Dev Notes

### Story Foundation

- Epic 1 is moving from backend-only flow preparation to the first visible multi-flow slice.
- This story should make two flows readable in one continuous native score surface, but it should still avoid final layout sophistication, interaction redesign, or active-flow UX changes.
- The safest definition of done is: the first two visible flows are painted into one wider renderer surface with a deterministic gap, single-flow behavior still works, and maintainers can verify the composed result through existing diagnostics.

### Developer Context Summary

- Story 1.1 already established `FlowRenderPlan` and `PreparedFlowRenderInput` in `src-tauri/src/instructions/render_plan.rs`; the plan preserves visible-flow order from `score.flows.order` and keeps `drawn_flow_id` explicit.
- `AppContext::render()` in `src-tauri/src/lib.rs` now always goes through `prepare_flow_render_plan(...)`, emits `render-timing-flow-preparation`, and passes the plan into `generate_render_instructions(...)`.
- `generate_render_instructions` in `src-tauri/src/instructions/mod.rs` still renders only one flow. It clears `renderer.instructions` and `renderer.hit_targets`, computes one flow's width/height, and stores those dimensions on `RendererContext`.
- `RendererContext` pan and zoom limits in `src-tauri/src/renderer/mod.rs` already derive from `renderer.width` and `renderer.height`, so widening the backend-rendered surface is the seam that later scrolling stories will build on.
- `renderer_hit_test` in `src-tauri/src/renderer/hit_test.rs` assumes hit-target coordinates are already in the shared renderer-space coordinate system. Any second-flow target or popover anchor must be offset into that same combined space.
- The React overlay in `src/windows/main/components/renderer.tsx` already forwards bounds, pan, zoom, and hit-testing to Rust. This story should not move score rendering or composition logic into React.

### Previous Story Intelligence

- Reuse the `FlowRenderPlan` seam instead of inventing a second multi-flow registry or frontend-owned render state.
- Keep flow ordering tied to `score.flows.order`; Story 1.1 added tests specifically to prevent nondeterministic `layout.flow_ids` ordering leaks.
- Extend the existing `render-timing-flow-preparation` diagnostics and debugger surface rather than adding ad hoc logs or a new panel.
- Preserve active-flow rendering fallback behavior when the active flow is hidden from the layout or when no prepared flows are visible.
- Treat documentation changes as part of the implementation whenever stable renderer/debugger expectations shift.

### Technical Requirements

- Keep notation derivation, written durations, spacing analysis, and draw-module behavior unchanged in semantics. This story is about composing already-derived flow outputs into one visible surface, not rewriting notation logic.
- Prefer extracting a reusable single-flow render-segment helper from `generate_render_instructions(...)` rather than duplicating the full parse/draw pipeline twice in two separate code paths.
- Represent the fixed inter-flow boundary in renderer space and derive any pixel guarantee from the converter, so the native renderer stays authoritative and the 20px minimum remains verifiable.
- Compose the first two visible flows only. Do not generalize to arbitrary N-flow layout in a way that obscures the simple rule this story is supposed to validate.
- Make the second flow visibly distinct via spatial separation, not via new colors, panels, or shell-level UI.
- Keep the final combined surface rectangular and deterministic so later stories can extend scrolling, visual coherence, and orientation cues from the same foundation.

### Architecture Compliance

- Rust remains the single source of truth for score state, visible-flow composition, and native score rendering.
- The score must continue to render through Rust/Skia/Metal. Do not render notation in React/HTML.
- Multi-flow rendering remains a derived presentation layer over existing flow data. Do not introduce persisted score-structure changes or migrations for this story.
- Active-flow semantics and visible multi-flow presentation remain separate concerns. This story adds visibility, not a new editing model.
- Existing interaction contracts should not be broadened casually. Any second-flow hit-target behavior must be deliberate and flow-correct; broader cross-flow interaction work belongs to Epic 2.
- Keep page-layout and frame-layout evolution feasible. Do not bake in assumptions that the fixed two-flow placement is the permanent final layout model.

### Library / Framework Requirements

- Stay within the current stack already in the repo: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, Rust 2021, `skia-safe` 0.93.x, `rayon` 1.10, and `rustc-hash`.
- Preserve snake_case across the frontend/backend boundary and keep Tauri command/event payloads typed and minimal.
- Use existing native-renderer helpers, converter utilities, and `RendererContext` composition patterns before introducing new abstractions.
- No dependency upgrade is expected for this story.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/instructions/render_plan.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/renderer/mod.rs`
  - `src-tauri/src/renderer/hit_test.rs`
- Supporting backend/UI seams if needed for pan/zoom or diagnostics verification:
  - `src-tauri/src/ui/renderer.rs`
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
  - `src/windows/main/components/renderer.tsx`
- Likely out of scope for Story 1.2 unless implementation proves otherwise:
  - persistence and migrations
  - popover workflow changes
  - write-screen properties UX
  - sequencer/piano-roll active-flow semantics

### Testing Requirements

- Add Rust tests close to the render composition seam for first-two-flow selection, offset calculations, combined renderer extents, and single-flow fallback.
- Add or update renderer/hit-test tests to verify composed coordinate offsets remain correct in shared renderer space.
- If diagnostics payloads change, extend the existing debugger Vitest coverage rather than creating a parallel test surface.
- Validate that a one-flow score still renders exactly as before through the same workflow.
- Validation target:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Git Intelligence Summary

- Recent commits remain planning-oriented: `chore: sprint prep`, `chore: epics docs`, `chore: architecture docs`, `chore: ux docs setup`, `chore: PRD review`.
- The most relevant implementation precedent is the current Story 1.1 code already present in the working tree, not the recent commit history.
- Story 1.2 should therefore establish the first visible multi-flow rendering pattern cleanly enough that Stories 1.3 through 1.5 can extend it without reworking the core composition seam.

### Project Context Reference

- The project rules in `docs/project-context.md` require native Rust score rendering, Rust-owned authoritative state, snake_case FE/BE contracts, and careful avoidance of unnecessary new horizontal-spacing slots.
- The current architecture document already treats multi-flow support as a derived presentation/composition layer over existing flow data rather than a persisted model change.
- `RendererContext` caching and pan/zoom behavior already assume a single shared page surface. Story 1.2 should extend that page surface, not create multiple frontend panels.
- The debugger timing surface is the established maintainer verification point for render-path changes.
- Documentation updates are required when renderer/debugger behavior becomes part of the shared engineering contract.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.2)
- Source: `_bmad-output/planning-artifacts/prd.md` (FR1-FR5, FR16, FR18-FR19, FR35-FR36; NFR1-NFR6, NFR10)
- Source: `_bmad-output/planning-artifacts/architecture.md` (multi-flow presentation composition, active-flow separation, render instrumentation, project structure mapping)
- Source: `docs/project-context.md` (native renderer ownership, flow-preparation phase, FE/BE boundary, render pipeline rules)
- Source: `docs/architecture.md` (renderer caching, flow-preparation seam, native score render pipeline)
- Source: `src-tauri/src/instructions/render_plan.rs` (prepared-flow ordering and current single-flow fallback)
- Source: `src-tauri/src/instructions/mod.rs` (current single-flow render body and renderer dimension assignment)
- Source: `src-tauri/src/lib.rs` (`AppContext::render` flow-preparation seam and diagnostics emission)
- Source: `src-tauri/src/renderer/mod.rs` (`RendererContext` width/height, pan constraints, cached instruction surface)
- Source: `src-tauri/src/renderer/hit_test.rs` (shared renderer-space hit-target contract)
- Source: `src-tauri/src/ui/renderer.rs` (pan/zoom commands constrained by renderer dimensions)
- Source: `src/windows/main/components/renderer.tsx` (native-score overlay, bounds updates, hit-test delegation)
- Source: `_bmad-output/implementation-artifacts/1-1-expose-a-verifiable-flow-aware-render-path-for-multi-flow-delivery.md` (previous story learnings and established diagnostics seam)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Use the debugger window's `PerformanceTimings` panel as the primary verification surface for composed-flow diagnostics and fallback mode.
- Manual verification should include a one-flow score and a two-flow score at normal zoom to confirm visible gap, surface width growth, and single-flow compatibility.
- Manual verification notes:
  - One-flow score: confirm the render path still produces a single readable page, the debugger shows `Composed flow count: 1`, and fallback mode remains `active-flow-only`.
  - Two-flow score: confirm the first two visible flows appear on one wider surface, the debugger shows `Composed flow count: 2`, and the visible inter-flow gap remains clearly readable at normal zoom.

### Implementation Plan

- Refactor `generate_render_instructions(...)` into reusable per-flow segment building plus a small composition step that merges up to two visible flows into one shared surface.
- Keep ordering and fallback decisions in `FlowRenderPlan`, and extend diagnostics so the debugger can show both prepared flows and the specific flow IDs actually composed.
- Add focused Rust coverage for flow selection, fixed-gap sizing, combined extents, and coordinate offsets; update debugger Vitest assertions if the diagnostics payload changes.
- Update architecture/project context docs only for stable renderer-surface composition and debugger-verification contract changes introduced by this story.

### Completion Notes List

- Story created automatically from the first backlog entry in `sprint-status.yaml` after Story 1.1 was marked done.
- Previous story learnings were incorporated from the Story 1.1 implementation artifact and current renderer code.
- No external dependency upgrade is expected for this story.
- No migration or persisted-data change is expected for this story.
- Refactored the Rust score render path into reusable per-flow segment construction, then composed up to the first two visible flows into one shared renderer surface with a deterministic renderer-space gap.
- Kept flow ordering anchored to `score.flows.order`, preserved active-flow single-surface fallback when fewer than two visible flows are available, and widened `RendererContext` extents from the composed surface bounds.
- Added renderer-space draw-instruction and hit-target offset helpers plus Rust coverage for fixed-gap sizing, segment composition offsets, shared-surface hit testing, and plan diagnostics.
- Extended debugger diagnostics to show prepared versus composed flow IDs and counts so maintainers can verify fallback versus two-flow surface composition without adding a new panel.
- Validation passed with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- Code review follow-up: renamed the debugger draw-target diagnostic to a fallback-only field so multi-flow reviews no longer imply that a non-composed active flow was drawn into the shared surface.

### Senior Developer Review (AI)

- 2026-03-11: Requested changes during review because the debugger exposed `drawn_flow_id` even when the active flow was not part of the composed surface, which made the verification panel misleading for 3+ visible flows.
- 2026-03-11: Fixed during review by changing flow-preparation diagnostics to emit `fallback_draw_flow_id` only when single-flow fallback is active, then updated the debugger and focused coverage to match.

### File List

- \_bmad-output/implementation-artifacts/1-2-render-two-flows-in-one-score-surface-with-fixed-placement.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- docs/architecture.md
- docs/project-context.md
- src-tauri/src/instructions/mod.rs
- src-tauri/src/instructions/render_plan.rs
- src-tauri/src/renderer/hit_test.rs
- src-tauri/src/renderer/mod.rs
- src/windows/debugger/components/performance-timings.test.tsx
- src/windows/debugger/components/performance-timings.tsx

## Change Log

- 2026-03-11: Created Story 1.2 with implementation guidance for fixed-gap two-flow surface composition, active-flow guardrails, diagnostics expectations, and focused regression coverage.
- 2026-03-11: Implemented fixed-gap two-flow surface composition in Rust, extended debugger diagnostics for composed-flow verification, added composition/offset regression coverage, and updated stable renderer contract docs.
- 2026-03-11: Fixed review feedback so fallback diagnostics only name a draw target in single-flow fallback mode, preventing misleading multi-flow verification output.
