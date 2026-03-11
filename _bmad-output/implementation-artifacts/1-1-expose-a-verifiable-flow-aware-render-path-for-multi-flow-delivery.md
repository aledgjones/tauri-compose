# Story 1.1: Expose a Verifiable Flow-Aware Render Path for Multi-Flow Delivery

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want the score renderer to expose a verifiable flow-aware composition path,
so that I can confirm multi-flow delivery is safe to build and debug before visible layout changes land.

## Acceptance Criteria

1. Given the existing score view render pipeline, when rendering runs for a score with one or more flows, then the renderer builds its presentation from flow-aware composition inputs, and maintainers can confirm through diagnostics that the flow-aware path was exercised while single-flow output remains unchanged.
2. Given a score containing multiple flows, when the updated render pipeline runs, then it can enumerate and prepare flow-aware composition inputs for more than one flow, and each prepared flow input is available as an explicit, inspectable unit for later score-surface composition work.
3. Given the updated flow-aware composition path, when a score renders, then existing written-duration and notation-derivation behavior remains unchanged, and no persisted score-format or migration change is required.
4. Given a maintainer validates this delivery step in normal debugging tools, when the score render completes, then the implementation exposes enough timing or diagnostic visibility to confirm the new composition path is being exercised, and the render path is ready for visible multi-flow placement work without another hidden backend reshaping step.

## Tasks / Subtasks

- [ ] Add a flow-aware preparation boundary in the Rust render orchestration layer (AC: 1, 2)
  - [ ] Refactor the current single-flow render entry seam in `AppContext::render()` so it prepares ordered flow-aware inputs before handing work to the existing draw pipeline.
  - [ ] Enumerate visible flows from the active layout using `layout.flow_ids`, but preserve ordering from `score.flows.order` rather than hash-set iteration order.
  - [ ] Keep `ui.flow_id` as the active flow for this story, but route rendering through the new preparation path even when only one flow is actually drawn.
- [ ] Expose explicit prepared flow units without changing current score output (AC: 1, 2, 3)
  - [ ] Introduce an internal render-plan/prepared-flow type or equivalent helper that captures the data later stories will need for score-surface composition.
  - [ ] Preserve the current user-visible single-flow output by continuing to draw only the active flow in Story 1.1.
  - [ ] Make the prepared flow units inspectable in debugging output or timing diagnostics rather than duplicating score state in React.
- [ ] Extend diagnostics so maintainers can verify the new path is active (AC: 1, 4)
  - [ ] Emit additional timing or diagnostic signals that show flow-aware preparation ran before the existing per-flow parse/draw phases.
  - [ ] Update the debugger timing surface to display the new signal in the existing timing UI rather than creating a separate debug panel.
  - [ ] Include enough information to distinguish active-flow drawing from ordered multi-flow preparation.
- [ ] Preserve notation, persistence, and renderer contracts (AC: 3)
  - [ ] Keep written-duration and notation derivation logic unchanged; this story is orchestration only, not notation logic.
  - [ ] Avoid `.soloc` semantic changes and do not add a migration unless persisted behavior truly changes.
  - [ ] Preserve the native render pipeline and current hit-test/selection contracts for this story.
- [ ] Add focused regression coverage and verification (AC: 1, 2, 3, 4)
  - [ ] Add Rust tests around ordered flow enumeration and prepared-flow generation.
  - [ ] Add or update debugger/Vitest coverage for any new timing event wiring.
  - [ ] Validate with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- [ ] Update docs if implementation changes documented renderer/debugger contracts
  - [ ] Update `docs/architecture.md` and `docs/project-context.md` if new flow-aware timing phases, render-plan terminology, or debugger expectations become part of the stable development contract.

## Dev Notes

### Story Foundation

- Epic 1 is about making multiple flows read as one score surface, but Story 1.1 is intentionally backend-first.
- The goal is to create the composition seam that later visible-placement stories can reuse, not to ship two-flow rendering yet.
- The safest definition of done is: ordered visible flows can be prepared as explicit units, the current active flow still renders identically, and the debugger can prove the new path ran.

### Developer Context Summary

- The current render path is still single-flow. `src-tauri/src/lib.rs` resolves one `layout_id` and one `flow_id`, then calls `generate_render_instructions(app, self, renderer, layout_id, flow_id)`.
- `src-tauri/src/instructions/mod.rs` already receives both `layout_id` and `flow_id`, but immediately dereferences a single flow and produces one page of instructions and hit targets.
- Layouts already know about multiple flows. `src-tauri/src/score/layouts.rs` stores `flow_ids` as an inclusion set, with ordering explicitly coming from `score.flows.order`.
- Flow creation and removal already keep layouts synchronized in `src-tauri/src/score/flows.rs` and `src-tauri/src/score/layouts.rs`. Reuse this plumbing instead of inventing a second visible-flow registry.
- The debugger already listens to `render-timing-*` and `native-render-timing-*` events in `src/windows/debugger/components/performance-timings.tsx`, so diagnostics should extend that surface rather than creating another one.

### Technical Requirements

- Introduce a flow-aware preparation concept near the Rust render pipeline. A small internal type such as `PreparedFlowRenderInput` or `FlowRenderPlan` is appropriate if it helps isolate later multi-flow composition work.
- Prepare ordered flow units from the active layout's visible flows. Preserve active-flow semantics by still drawing only the current active flow in Story 1.1.
- Keep diagnostics primitive and easy to consume. If you add timing events, follow the existing `render-timing-*` naming pattern and keep payloads simple.
- Expose enough debugging context to answer: which layout rendered, which flow is active, which flows were prepared, how many prepared flow units exist, and whether single-flow draw fallback was used.
- Do not introduce a React-owned copy of render-plan state. Rust remains the source of truth.

### Architecture Compliance

- Rust remains the single source of truth for score state, durable mutations, and render orchestration.
- The score must continue to render natively through Rust/Skia/Metal. Do not move notation rendering into React/HTML.
- Multi-flow support must remain a derived presentation/composition layer over existing flow data, not a new persisted score structure.
- Active-flow semantics and visible-flow composition are separate concerns. Story 1.1 should preserve that separation.
- Preserve existing notation behavior, especially written durations. This story must not become a notation rewrite.
- Preserve current hit-target and selection behavior. Multi-flow interaction changes belong to later stories.
- Avoid adding new horizontal spacing slots. This story is about orchestration and diagnostics, not visible layout spacing.

### Library / Framework Requirements

- Stay within the current stack and versions already in the repo: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.x, Rust 2021, `skia-safe` 0.93.x, `rayon` 1.10, and `rustc-hash` for map/set usage.
- Preserve snake_case across the frontend/backend boundary.
- Preserve existing Tauri command patterns and typed selection payloads.
- Use `FxHashMap`/`FxHashSet` in Rust where new hash collections are needed.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/instructions/mod.rs`
  - a small new helper under `src-tauri/src/instructions/` only if the current module becomes too crowded
- Existing model/plumbing references:
  - `src-tauri/src/score/layouts.rs`
  - `src-tauri/src/score/flows.rs`
  - `src-tauri/src/renderer/mod.rs`
- Debugger/UI seams only if needed for diagnostics exposure:
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
- Likely out of scope for Story 1.1 unless the implementation proves otherwise:
  - persistence and migrations
  - popover behavior
  - write-screen interaction UX
  - visible multi-flow placement/hit-testing changes

### Testing Requirements

- Add Rust coverage close to the new flow-aware orchestration helper or instruction module.
- Cover ordered flow enumeration explicitly so hash-set storage does not leak nondeterministic ordering into rendering.
- If debugger events change, extend the existing Vitest coverage in `src/windows/debugger/components/performance-timings.test.tsx`.
- Keep parser-local tests local if any notation-adjacent helper is touched.
- Validation target:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Git Intelligence Summary

- Recent commits are planning-document commits only: `chore: epics docs`, `chore: architecture docs`, `chore: ux docs setup`, `chore: PRD review`, `feat(bmad): PRD`.
- There is no prior implementation commit history for multi-flow rendering yet, so this story should establish the first code-level pattern cleanly.

### Project Context Reference

- The score render pipeline is native: parse modules build notation data, draw modules emit `Vec<DrawInstruction>` and `Vec<HitTarget>`, and the Metal backend paints the score surface.
- `RendererContext` caches draw instructions and hit targets for pan/zoom re-rendering. Preserve that cache model.
- The frontend renderer overlay delegates hit testing to Rust via `renderer_hit_test`; Story 1.1 should not replace or approximate that contract in React.
- Existing debugger timing categories already cover layout, render, and GPU phases. Extend those categories rather than creating ad hoc logging.
- Documentation rules in this repo require updating shared docs whenever stable implementation expectations change.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.1)
- Source: `_bmad-output/planning-artifacts/prd.md` (Functional requirements FR1, FR12-FR16, FR35-FR36; Non-functional requirements NFR1-NFR6, NFR10)
- Source: `_bmad-output/planning-artifacts/architecture.md` (Core architectural decisions and implementation sequence)
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` (maintainer/debugging user, score-first shell, debugger continuity)
- Source: `docs/project-context.md` (AI-agent implementation rules and score render pipeline)
- Source: `docs/architecture.md` (native render pipeline, flow-wide directives, caching)
- Source: `src-tauri/src/lib.rs` (`AppContext::render` current single-flow seam)
- Source: `src-tauri/src/instructions/mod.rs` (`generate_render_instructions` current single-flow orchestration and timing events)
- Source: `src-tauri/src/score/layouts.rs` (`Layout.flow_ids` inclusion semantics)
- Source: `src-tauri/src/score/flows.rs` (flow creation/removal and layout synchronization)
- Source: `src-tauri/src/renderer/mod.rs` (`RendererContext`, cached instructions, cached hit targets)
- Source: `src/windows/debugger/components/performance-timings.tsx` (existing timing UI)
- Source: `src/windows/main/components/renderer.tsx` (existing native-score overlay and hit-test contract)
- Source: `src/services/types.ts` and `src-tauri/src/ui/sequencer.rs` (selection and FE/BE contract shape)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Use the debugger window's `PerformanceTimings` panel as the primary verification surface for any new render-path timing or diagnostic events.
- Keep any new diagnostics compatible with the existing debugger event-listener pattern.

### Completion Notes List

- Story created automatically from the first backlog entry in `sprint-status.yaml`.
- No previous story intelligence exists yet because this is the first implementation story in Epic 1.
- Recent git history provides planning context only, not implementation precedent.
- No external dependency upgrade is expected for this story.
- No migration or persisted-data change is expected for this story.

### File List

- None yet. Implementation pending.
