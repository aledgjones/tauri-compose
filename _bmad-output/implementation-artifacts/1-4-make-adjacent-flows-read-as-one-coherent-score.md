# Story 1.4: Make Adjacent Flows Read as One Coherent Score

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want adjacent rendered flows to feel visually consistent and intentionally separated,
so that the multi-flow score view reads like one document instead of stitched-together rendering fragments.

## Acceptance Criteria

1. Given two flows are rendered in the same score surface, when the score view displays them side by side, then staff sizing, clef presentation, instrument-name treatment, grouping marks, systemic starting barlines, and baseline spacing style remain visually consistent across both flows, and each visible flow is rendered as its own full white page on the shared surface with the fixed flow boundary exposing the gradient backdrop between them.
2. Given two adjacent flows in the score surface, when a user reads from the end of the first flow to the start of the second, then the transition is visually understandable without overlap or ambiguous boundary placement, and the visible backdrop gap makes the boundary legible at normal working zoom levels.
3. Given a score with one flow or with multi-flow rendering not yet extended beyond two visible flows, when the score view renders, then existing score presentation remains correct, and this refinement does not require interaction, popover, or active-flow logic changes.
4. Given this visual-coherence increment is implemented, when the output is validated, then notation-derivation and written-duration behavior remain unchanged, and the boundary treatment provides sufficient visual coherence for readable two-flow score presentation without altering interaction behavior.

## Tasks / Subtasks

- [x] Refine the shared-surface composition rules so adjacent flows stop reading like two isolated page starts (AC: 1, 2, 3, 4)
  - [x] Audit the current per-flow segment build in `src-tauri/src/instructions/mod.rs` to identify which page-start artifacts are being repeated on the second composed flow: instrument names, braces, brackets, systemic starting barline, left-edge padding, and any other segment-local chrome that makes the shared surface feel stitched together.
  - [x] Keep full native page-start treatment on both visible flows so the second visible flow still renders instrument names, grouping marks, the systemic starting barline, and the matching left-edge spacing while the fixed inter-flow gap provides the boundary cue.
  - [x] Keep the existing composed `renderer.width`, `renderer.height`, hit-target offsets, and Story 1.3 horizontal pan behavior authoritative for the final surface.
- [x] Preserve notation-consistent visual continuity without rewriting notation logic (AC: 1, 2, 4)
  - [x] Keep both visible flows on the same engrave-space, converter, vertical-spacing, and clef-rendering basis so coherence comes from shared rendering rules rather than flow-specific styling hacks.
  - [x] Preserve the existing parse/draw code path for both visible flows; this story should improve boundary readability through shared-surface composition and page backgrounds, not through flow-specific notation rendering branches.
  - [x] Preserve written durations, beaming, accidentals, spacing semantics, and broader notation-derivation behavior exactly as they work today.
- [x] Clarify and verify the inter-flow boundary treatment as an intentional document transition (AC: 1, 2, 4)
  - [x] Keep the fixed inter-flow separation deterministic and native-rendered, but refine the transition so it reads as a deliberate boundary between adjacent flows rather than two unrelated rendered panels.
  - [x] Replace the current single composed white page background with per-flow white page boxes so the existing gradient backdrop remains visible in the 20px inter-flow gap.
  - [x] Keep the page-start treatment native-rendered in the Rust pipeline; do not introduce React overlays, shell-level separators, or color-coded UI chrome to fake coherence.
  - [x] Confirm the boundary remains legible at normal working zoom and at compact desktop width where the score viewport is narrower.
- [x] Protect single-flow behavior and future layout evolution (AC: 3, 4)
  - [x] Preserve current single-flow presentation when fewer than two visible flows are composed.
  - [x] Do not take on selection, hit-testing, popovers, or active-flow semantics changes; those remain in later Epic 2 stories.
  - [x] Avoid hard-coding assumptions that would block Story 1.5 flow-orientation cues or future frame/page-layout work.
- [x] Add focused regression coverage and verification for coherent adjacent-flow presentation (AC: 1, 2, 3, 4)
  - [x] Add Rust tests near the shared-surface composition seam to verify adjacent-flow composition can suppress or preserve page-start elements intentionally without regressing single-flow output.
  - [x] Extend diagnostics or manual verification only where needed so maintainers can confirm the second visible flow no longer looks like a duplicated page fragment while renderer extents and scrolling remain correct.
  - [x] Validate with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- [x] Update docs if visual-coherence rules become part of the stable renderer contract
  - [x] Update `docs/architecture.md` and `docs/project-context.md` if adjacent-flow start treatment, boundary rules, or segment-role rendering become expected long-term behavior.

## Dev Notes

### Story Foundation

- Story 1.2 proved the first two visible flows can be composed into one shared native score surface with a deterministic fixed gap.
- Story 1.3 proved the combined surface width remains reachable through the existing horizontal pan model without creating a new frontend scroll architecture.
- Story 1.4 should improve readability and document-like coherence of that same shared surface. It is not a flow-count expansion, notation rewrite, or interaction-model change.
- The practical definition of done is: two visible flows still render through the shared surface, each flow reads as its own white page on the gradient backdrop, the gap between them remains visible and intentional, and single-flow behavior remains unchanged.

### Developer Context Summary

- `generate_render_instructions(...)` in `src-tauri/src/instructions/mod.rs` now builds up to the first two visible flow segments from `FlowRenderPlan`, composes them into one shared surface, and draws a single white background for the final composed renderer. If the intended Story 1.4 treatment is visible page separation on the gradient backdrop, this single composed background is one of the primary seams that must change.
- `build_flow_render_segment(...)` in `src-tauri/src/instructions/mod.rs` still computes full per-segment layout chrome for every composed flow: page padding, instrument-name placement, braces, brackets, sub-brackets, systemic starting barline, staves, and all notation elements. Those repeated system-start elements are the first place to inspect for the “stitched-together fragments” problem.
- `compose_render_segments(...)` in `src-tauri/src/instructions/mod.rs` currently applies only horizontal offsets plus the fixed inter-flow gap. Story 1.4 should keep that simple composition model and improve coherence through per-flow page backgrounds and boundary presentation rather than by suppressing native page-start drawing on the second visible flow.
- `prepare_flow_render_plan(...)` in `src-tauri/src/instructions/render_plan.rs` still deliberately caps the composed view at the first two visible flows. Keep Story 1.4 scoped to those same two flows rather than generalizing to arbitrary N-flow document layout.
- Story 1.3 established that the composed `renderer.width` is the single authoritative horizontal extent for native pan clamping and debugger verification. Do not regress that ownership by introducing a frontend-managed scroll model.
- The React overlay in `src/windows/main/components/renderer.tsx` remains an input/reporting layer over the native score surface. Visual coherence fixes for adjacent flows belong in Rust rendering, not in HTML overlays.

### Previous Story Intelligence

- Reuse the Story 1.2 shared-surface seam instead of introducing separate score panels, tabs, or alternate per-flow containers.
- Keep flow ordering anchored to `score.flows.order` and the prepared visible-flow plan. This story should refine how adjacent flows are presented, not reinterpret which flows are visible.
- Reuse the Story 1.3 debugger evidence path if additional diagnostics are needed. Do not invent a separate coherence-only tooling surface unless the existing debugger cannot verify the result.
- Preserve the current active-flow and interaction contract. Epic 1 is still focused on readable presentation; Epic 2 owns broader cross-flow interaction semantics.
- Treat documentation updates as part of the implementation if adjacent-flow start rules or boundary treatment become an expected renderer contract.

### Technical Requirements

- Keep Rust authoritative for score-surface composition, renderer extents, hit targets, and native score output. Do not move adjacent-flow presentation state into React.
- Treat second-flow start treatment as a presentation refinement over existing flow data, not as a persisted-data change or notation-model change.
- Do not suppress or trim the second visible flow's normal page-start visuals in this story. Instrument names, grouping marks, the systemic starting barline, and their matching left-edge spacing should render the same way they do for any other page start.
- Maintain the fixed boundary separation from Story 1.2 as a clear visual delimiter, but refine the start-of-second-flow presentation so the combined surface reads like one coherent score.
- If the visual design uses separate white page boxes per flow, keep those page backgrounds native-rendered in Rust and let the existing gradient backdrop show through the inter-flow gap instead of simulating the separation in the frontend shell.
- Preserve hit-target correctness and shared-surface offsets even if visual-only elements around the boundary are adjusted.
- Keep written-duration behavior, spacing derivation, clef logic, and broader notation semantics unchanged.
- No persistence or migration change is expected for this story.

### Architecture Compliance

- Multi-flow support remains a derived presentation/composition layer over existing flow data. This story must not introduce a new persisted score structure.
- The score must continue to render through Rust/Skia/Metal. Do not render notation, boundaries, or flow chrome in React/HTML.
- Preserve the existing frontend/backend command boundary and any snake_case diagnostics payloads if debugger emissions change.
- Active-flow semantics remain distinct from visible-flow presentation; Story 1.4 improves adjacent-flow readability, not editing scope.
- Keep future page-layout and music-frame evolution feasible, but do not introduce dormant page-layout abstractions yet. This story should stay with the current fixed two-flow composition rules until user-configurable page layout work is actually in scope.

### Library / Framework Requirements

- Stay within the existing stack already in the repo: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, Rust 2021, `skia-safe` 0.93.x, `rayon` 1.10, and `rustc-hash`.
- Preserve snake_case across the frontend/backend boundary and keep Tauri command/event payloads typed and minimal.
- Prefer extending existing renderer composition helpers before adding new abstractions.
- No dependency upgrade is expected for this story.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/instructions/render_plan.rs`
- Likely supporting Rust draw helpers only if the coherence refinement cannot stay in composition logic:
  - `src-tauri/src/instructions/draw/draw_barline.rs`
  - `src-tauri/src/instructions/draw/draw_braces.rs`
  - `src-tauri/src/instructions/draw/draw_brackets.rs`
  - `src-tauri/src/instructions/draw/draw_clef.rs`
  - `src-tauri/src/instructions/draw/draw_staves.rs`
- Supporting diagnostics only if verification needs to surface boundary/coherence evidence:
  - `src-tauri/src/lib.rs`
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
- Likely out of scope unless implementation proves otherwise:
  - persistence and migrations
  - selection/popover services
  - `src/windows/main/components/renderer.tsx`

### Testing Requirements

- Add Rust tests close to the shared-surface composition seam for per-flow page-box composition, fixed-gap boundary behavior, and unchanged single-flow output.
- Extend frontend tests only if debugger diagnostics or UI-facing verification surfaces change.
- Validate visually with a one-flow score and a two-flow score at normal working zoom and compact desktop width.
- Validation target:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Git Intelligence Summary

- The most relevant recent implementation commits are `022d3573 feat(1.3): renderer scrolling` and `b2f592b5 feat: render two flows on a shared score surface`.
- Those commits established the shared-surface composition and authoritative pan/clamp behavior that Story 1.4 should refine rather than replace.
- The remaining recent commits are planning/documentation oriented, so the strongest implementation precedent remains the current Story 1.2 and Story 1.3 render path already in the repository.

### Project Context Reference

- The project rules in `docs/project-context.md` require native Rust score rendering, Rust-owned authoritative state, snake_case FE/BE contracts, and keeping pan/zoom/render ownership in the backend.
- The architecture document treats multi-flow support as a derived composition layer over existing flows, with React remaining an orchestration shell rather than the owner of score truth.
- The current UX and PRD artifacts emphasize that the score should feel like one continuous professional document, not visually independent panels, while preserving precision over spectacle.
- Story 1.4 is the first increment explicitly tied to FR5 and FR18 pressure: visual coherence between adjacent flows plus a boundary treatment that remains clear and readable.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.4)
- Source: `_bmad-output/planning-artifacts/prd.md` (FR5, FR18, NFR5, MVP scope)
- Source: `_bmad-output/planning-artifacts/architecture.md` (derived multi-flow composition, Rust/React boundary, testing and instrumentation expectations)
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` (score-first hierarchy, continuous document feel, precision and trust goals)
- Source: `docs/project-context.md` (native renderer ownership, shared-surface render pipeline, backend-owned pan/zoom/render behavior)
- Source: `_bmad-output/implementation-artifacts/1-2-render-two-flows-in-one-score-surface-with-fixed-placement.md` (shared-surface composition and fixed-gap precedent)
- Source: `_bmad-output/implementation-artifacts/1-3-scroll-across-the-combined-multi-flow-score-width.md` (authoritative renderer width, shared pan path, debugger verification precedent)
- Source: `src-tauri/src/instructions/mod.rs` (`build_flow_render_segment`, `compose_render_segments`, `generate_render_instructions`)
- Source: `src-tauri/src/instructions/render_plan.rs` (two-flow composition scope)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Use the debugger window's `PerformanceTimings` panel as the primary verification surface if additional boundary/coherence diagnostics are added.
- Manual verification should include:
  - one-flow score: confirm presentation remains unchanged when only one flow is visible
  - two-flow score: confirm each flow sits on its own white page box, the gradient backdrop is visible in the inter-flow gap, and the transition remains readable at normal working zoom
  - compact desktop width: confirm the boundary remains understandable while Story 1.3 scrolling behavior continues to expose the full composed surface

### Implementation Plan

- Audit the per-segment render setup to confirm the page-start chrome and spacing that must remain intact on both visible flows.
- Keep the shared-surface renderer simple: use per-flow native page boxes and the fixed inter-flow gap for boundary readability without introducing dormant page-layout abstractions.
- Add focused regression coverage and only the minimum diagnostics needed to prove the coherence refinement.
- Update architecture and project-context docs if the adjacent-flow treatment becomes a documented renderer contract.

### Completion Notes List

- Story created automatically from the first backlog entry in `sprint-status.yaml` after Story 1.3 was marked done.
- Previous story learnings were incorporated from the Story 1.2 and Story 1.3 implementation artifacts, current renderer code, and recent commit history.
- No external dependency upgrade is expected for this story.
- No migration or persisted-data change is expected for this story.
- Kept the second visible flow's instrument names, grouping braces/brackets, systemic starting barline, and matching left-edge spacing as part of its normal native page-start rendering.
- Replaced the single composed white surface fill with per-flow white page rectangles in the native Rust composition path so the fixed inter-flow gap exposes the gradient backdrop while `renderer.width`, `renderer.height`, hit-target offsets, and Story 1.3 pan behavior remain authoritative.
- Added seam-level Rust tests for per-flow page-box composition and start-treatment rules; no React overlay or persisted-data changes were introduced.
- Updated `docs/architecture.md` and `docs/project-context.md` to record the page-per-flow shared-surface contract and the requirement to preserve native page-start chrome on each visible flow.
- Validation completed: `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- Manual visual verification remains documented in the Debug Log References checklist for one-flow, two-flow, and compact-width confirmation; no additional debugger diagnostics were required for this increment.

### File List

- src-tauri/src/instructions/mod.rs
- docs/architecture.md
- docs/project-context.md
- \_bmad-output/implementation-artifacts/1-4-make-adjacent-flows-read-as-one-coherent-score.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-03-11: Implemented page-per-flow shared-surface rendering for Story 1.4, then corrected a regression by restoring the second flow's native page-start chrome and spacing while keeping per-flow page boxes and seam-level Rust coverage.
