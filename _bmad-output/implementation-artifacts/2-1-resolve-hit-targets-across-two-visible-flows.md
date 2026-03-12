# Story 2.1: Resolve Hit Targets Across Two Visible Flows

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want score interactions to resolve the correct target in either visible flow,
so that multi-flow score view can support precise interaction without ambiguity.

This story uses two visible flows as the minimum acceptance slice from the epic, but the implementation and verification must remain correct for any number of visible flows on the shared score surface.

## Acceptance Criteria

1. Given two flows are rendered in the same score surface, when a user points to an interactive score element in either visible flow, then hit-testing resolves the correct score target for that flow, and the result identifies a single unambiguous target.
2. Given two visible flows with distinct rendered positions, when hit-testing runs near the boundary between flows, then the resolved target belongs to the visually corresponding flow, and adjacent flow content does not steal the interaction.
3. Given a score with only one visible flow, when hit-testing runs through the updated multi-flow-aware path, then existing single-flow target resolution remains correct, and current score interactions remain usable.
4. Given the multi-flow hit-target path is implemented, when this increment is validated, then repeated hit-tests at the same visible location resolve to the same flow-correct target, and downstream selection or popover logic can consume that target without reinterpreting which flow it belongs to.

## Tasks / Subtasks

- [x] Verify the existing shared-surface hit-target contract across composed flows and only tighten implementation where verification exposes a real gap (AC: 1, 2, 3, 4)
  - [x] Review how `build_flow_render_segment(...)`, `compose_render_segments(...)`, and `HitTarget::offset(...)` currently place per-flow hit targets into shared renderer space in `src-tauri/src/instructions/mod.rs` and `src-tauri/src/renderer/mod.rs`.
  - [x] Confirm every interactive notation element continues to carry the correct `Selection` payload with the originating `flow_id` after composition, rather than inferring flow ownership later in the frontend.
  - [x] Verify the contract against any number of visible flows on the shared surface, even though the minimum acceptance slice for this story is two visible flows.
  - [x] If verification exposes ordering- or overlap-based ambiguity near inter-flow boundaries, fix that at the backend hit-target contract rather than by adding frontend heuristics.
- [x] Keep Rust hit-testing authoritative and deterministic for shared-surface layouts of any size (AC: 1, 2, 3, 4)
  - [x] Update `src-tauri/src/renderer/hit_test.rs` and any supporting renderer data structures only if verification shows the current path is not deterministic or flow-correct.
  - [x] Preserve the current renderer-space conversion path from CSS pixels to `Space` units and keep hit-testing backend-owned; do not move target resolution into React.
  - [x] Preserve popover anchor coordinates (`popover_x`, `popover_y`) and flow-aware `Selection` payloads so later Epic 2 stories can build on the same contract.
- [x] Preserve the existing selection handoff from renderer overlay to sequencer state (AC: 1, 3, 4)
  - [x] Keep `src/services/hit-test.ts`, `src/services/types.ts`, and `src/windows/main/components/renderer.tsx` aligned with the Rust selection payload shape and existing `snake_case` command parameters.
  - [x] Ensure the renderer overlay still dispatches the correct selection action for Tone, Rest, Tempo, TimeSignature, KeySignature, and Clef results without adding a parallel multi-flow selection model.
  - [x] Keep current single-flow click, pan, and zoom behavior intact while extending correctness to the shared two-flow surface.
- [x] Add focused regression coverage for hit-target correctness and stability (AC: 1, 2, 3, 4)
  - [x] Add Rust tests near `src-tauri/src/renderer/hit_test.rs` covering second-flow hits, boundary-adjacent hits, deterministic repeated hits, and unchanged single-flow behavior.
  - [x] Add Rust seam tests near `src-tauri/src/instructions/mod.rs` or `src-tauri/src/renderer/mod.rs` to verify composed-flow hit targets are offset into the correct shared-space positions and preserve flow identity.
  - [x] Update `src/services/hit-test.test.ts` and `src/windows/main/components/renderer.test.tsx` so the frontend contract remains stable for flow-aware hit-test results.
- [x] Update documentation if the shared hit-target contract becomes part of the explicit renderer architecture
  - [x] Update `docs/architecture.md` and `docs/project-context.md` if implementation changes the documented expectations for shared-surface hit-target ordering, flow ownership, or popover-anchor guarantees.

## Dev Notes

### Story Foundation

- Epic 1 already moved the score into a single shared native surface for multiple visible flows; Story 2.1 is the first interaction story that must make that surface trustworthy for editing work rather than passive reading.
- The practical definition of done is not just “a click returns something.” It is: a click in any visible flow resolves exactly one correct flow-aware target, boundary-adjacent clicks do not bleed into neighboring flow content, and the existing single-flow interaction path still works unchanged.
- This story is the interaction foundation for Story 2.2 selection and Story 2.4 contextual popovers. Do not solve this with ad hoc frontend heuristics that later stories would have to undo.
- Because Epic 1 already introduced the shared-surface composition, this may turn out to be primarily a verification and regression-hardening story rather than a large implementation story. The developer should prove that first, then make the smallest backend correction needed if proof fails.

### Developer Context Summary

- The current shared-surface composition already offsets hit targets into renderer space during `compose_render_segments(...)` in `src-tauri/src/instructions/mod.rs`. Each later visible flow inherits its horizontal offset through `HitTarget::offset(...)` in `src-tauri/src/renderer/mod.rs`, and that composition path now supports any number of visible flows.
- The current backend hit-test entry point in `src-tauri/src/renderer/hit_test.rs` converts CSS pixel coordinates into renderer `Space` units, then linearly scans `renderer.hit_targets` and returns the first target whose bounds contain the point.
- That implementation already proves the backend owns hit-testing, but it also means correctness depends on target ordering and non-overlapping bounds being trustworthy after shared-surface composition. Boundary mistakes here will propagate directly into selection and popover workflows.
- Frontend interaction in `src/windows/main/components/renderer.tsx` is intentionally thin: it computes element-relative coordinates, calls `hitTest(...)`, and dispatches the returned `Selection` to the existing sequencer actions. Keep it that way.
- `src/services/types.ts` defines the cross-boundary `Selection` union. The returned payload already includes `flow_id` for every supported selection variant, so this story should preserve and strengthen that backend contract rather than adding extra frontend disambiguation state.
- `src/ui/popover.tsx` derives flow, tick, and stave context from the current sequencer selection. If hit-testing resolves the wrong flow, later popover stories will be wrong by construction.

### Previous Story Intelligence

- Story 1.6 confirmed that all visible flows now compose into one shared native surface, and that hit targets are already offset into shared renderer space instead of being limited to a two-flow cap.
- Story 1.4 established the page-per-flow treatment and the fixed inter-flow gap. Story 2.1 should use that visible separation as a correctness boundary for hit-testing rather than inventing a different interaction geometry model.
- The rejected Story 1.5 flow-label direction is still out of scope. This story is about target correctness, not adding new orientation chrome.
- The implementation stance should therefore be verify-first: assume the broad architecture may already be correct, then close only the specific correctness gaps that validation reveals.

### Technical Requirements

- Keep Rust authoritative for hit-target generation, shared-surface target offsets, and hit-test resolution.
- Preserve the current CSS-pixel to renderer-`Space` conversion path in `renderer_hit_test`; any correctness fix must still honor pan and zoom.
- Preserve `Selection` payloads with the correct `flow_id` for every supported hit-target variant.
- Ensure boundary-adjacent hits resolve to the visually corresponding flow and remain deterministic across repeated clicks at the same location.
- Treat support for two visible flows as the minimum acceptance check, not as a maximum supported flow count. Verification and any fix must remain valid for any number of visible flows already supported by the shared-surface renderer.
- Keep single-flow hit-testing behavior intact. This story extends the shared-surface path; it must not regress established single-flow score interactions.
- Preserve popover anchor data on `HitTarget` so later contextual popover work can consume the same backend-owned geometry.
- Do not introduce persisted-data changes, `.soloc` semantics changes, or migration work for this story.
- Do not move score interaction ownership into React or create a frontend-owned map of flow bounds.

### Architecture Compliance

- Multi-flow interaction remains a derived presentation-layer capability over existing flow data; do not add a new persisted interaction model.
- The score remains rendered and hit-tested natively in Rust/Skia/Metal. React remains an overlay and orchestration layer only.
- Preserve `snake_case` across the Tauri command boundary and keep Rust and TypeScript selection types aligned.
- Keep active-flow semantics distinct from visible-flow hit-target correctness. This story resolves “what did the user click,” not “which flow is active for editing.”
- Keep later Epic 2 and Epic 3 work feasible by improving the existing hit-target contract rather than special-casing only the current two-flow scenario.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, Rust 2021, `skia-safe` 0.93.0, `rayon` 1.10, and `rustc-hash`.
- Preserve Tauri `invoke()` usage in `src/services/hit-test.ts` with `snake_case` parameters.
- No dependency upgrade is expected for this story.
- External web research is not available in this environment; use repository-pinned versions and local docs as the authoritative implementation baseline.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/renderer/hit_test.rs`
  - `src-tauri/src/renderer/mod.rs`
  - `src-tauri/src/instructions/mod.rs`
- Primary frontend seams:
  - `src/services/hit-test.ts`
  - `src/services/types.ts`
  - `src/windows/main/components/renderer.tsx`
- Primary regression seams:
  - `src/services/hit-test.test.ts`
  - `src/windows/main/components/renderer.test.tsx`
- Downstream consumer to keep compatible:
  - `src/ui/popover.tsx`
- Likely out of scope unless implementation proves otherwise:
  - persistence and migrations
  - active-flow UX indicators
  - sequencer or piano-roll active-flow switching

### Testing Requirements

- Add Rust tests in `src-tauri/src/renderer/hit_test.rs` for:
  - second-flow hit resolution on the composed surface
  - boundary-adjacent hits resolving to the visually correct flow
  - deterministic repeated hit-testing at the same location
  - representative multi-page or later-flow cases proving the contract still holds when more than two flows are visible
  - unchanged single-flow behavior
- Add backend seam coverage near `src-tauri/src/instructions/mod.rs` or `src-tauri/src/renderer/mod.rs` to verify composed hit targets preserve correct offsets and `flow_id` values after shared-surface composition.
- Update `src/services/hit-test.test.ts` to preserve the frontend IPC contract for flow-aware selections.
- Update `src/windows/main/components/renderer.test.tsx` to cover flow-aware click dispatch and to confirm non-hit interactions still fall back to pan behavior.
- Prefer verification-oriented assertions over speculative rewrites: first prove the existing path passes or fails, then make the smallest change necessary.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Git Intelligence Summary

- Recent history confirms the intended extension path: `4379a2af feat(1.6): render all visible flows on one surface` widened the shared-surface composition, while `fc46ccdf feat(1.4): refine adjacent flow score coherence` preserved per-flow page boxes and inter-flow separation.
- `83071eae chore(story): close rejected flow label direction` remains a useful guardrail: do not mix target-resolution work with the previously rejected orientation-label direction.
- Treat the current renderer state as a continuation of Epic 1’s shared-surface design, not as a license to introduce a second interaction system.

### Project Context Reference

- `docs/project-context.md` defines the score page as natively rendered in Rust and the React renderer as a transparent interaction overlay only.
- `_bmad-output/planning-artifacts/architecture.md` requires multi-flow work to remain a derived composition layer over existing flow data and explicitly warns against duplicating score-authoritative state in the frontend.
- The current shared-surface render pipeline already composes every visible flow into one renderer-owned surface and keeps combined hit targets in shared renderer space. Story 2.1 should strengthen the correctness of that contract, not replace it.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.1)
- Source: `_bmad-output/planning-artifacts/architecture.md` (derived multi-flow interaction architecture, Rust/React boundaries, testing expectations)
- Source: `docs/project-context.md` (native render pipeline, shared-surface composition, backend-owned hit-testing)
- Source: `_bmad-output/implementation-artifacts/1-6-render-all-visible-flows-in-one-continuous-score-surface.md` (all-visible-flow shared-surface precedent)
- Source: `src-tauri/src/instructions/mod.rs` (`compose_render_segments(...)`, per-flow segment composition, shared hit-target offsets)
- Source: `src-tauri/src/renderer/mod.rs` (`HitTarget`, `HitTarget::offset(...)`, renderer-owned hit-target storage)
- Source: `src-tauri/src/renderer/hit_test.rs` (current backend hit-test scan and existing tests)
- Source: `src/services/hit-test.ts` (frontend IPC contract)
- Source: `src/services/types.ts` (`Selection` union and flow-aware payload shape)
- Source: `src/windows/main/components/renderer.tsx` (overlay click path from pointer event to selection action)
- Source: `src/windows/main/components/renderer.test.tsx` (frontend renderer interaction regression seam)
- Source: `src/ui/popover.tsx` (downstream consumer of flow-aware selection context)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Use targeted Rust tests around `renderer_hit_test` as the primary correctness proof for boundary behavior and repeated-hit determinism.
- Use `src/windows/main/components/renderer.test.tsx` as the main frontend seam to verify the overlay still dispatches the correct selection actions after backend changes.
- Verified the composed-surface contract preserved `flow_id` ownership and popover anchors, then tightened backend hit resolution so overlapping shared-surface targets resolve to the visually closest containing target instead of relying on insertion order.

### Implementation Plan

- Verify whether ambiguity is caused by composed target offsets, target ordering, or both before changing resolution logic.
- Treat the existing implementation as potentially already correct for most cases; use tests and targeted manual checks to determine whether this story is mainly verification or needs a small backend fix.
- Fix the issue at the backend hit-target contract so the returned selection is already flow-correct when it crosses the Tauri boundary.
- Keep the frontend overlay thin and update only the tests and typed contracts needed to preserve the existing selection handoff.

### Completion Notes List

- Story auto-created from the first backlog story in `sprint-status.yaml`: `2-1-resolve-hit-targets-across-two-visible-flows`.
- Epic 2 was marked `in-progress` because this is its first story.
- Story context was built from the planning artifacts, architecture and project-context docs, recent Epic 1 implementation artifacts, current renderer/hit-test code, and recent git history.
- The story wording was tightened after review so “two visible flows” is treated as the minimum acceptance slice from the epic, not a maximum supported flow count, and so the work is framed as verification-first.
- No persisted-data migration is expected for this story.
- No external web lookup was available; repository-pinned versions and local docs were used as the authoritative technical baseline.
- Added a backend hit-test fix so overlapping shared-surface targets are resolved by closest containing target with stable tie-breaking, instead of first-match ordering.
- Added Rust regression tests covering inter-flow gaps, overlapping boundary hits, repeated-hit determinism under competing targets, pan/zoom conversion on the shared surface, later-flow offsets, unchanged single-flow behavior, and composed-hit-target flow identity.
- Added frontend regression tests proving flow-aware Tone and Rest IPC payloads pass through unchanged and that the renderer overlay still dispatches the correct selection actions for flow-aware multi-flow results.
- Full validation passed with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- Documentation review concluded that the existing architecture docs already described the backend-owned shared-surface hit-target contract accurately, so no doc edits were required for this story.

### File List

- \_bmad-output/implementation-artifacts/2-1-resolve-hit-targets-across-two-visible-flows.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- src-tauri/src/renderer/hit_test.rs
- src-tauri/src/instructions/mod.rs
- src/services/hit-test.test.ts
- src/windows/main/components/renderer.test.tsx

## Senior Developer Review (AI)

### Reviewer

GPT-5.4 on 2026-03-11

### Outcome

Approved after fixes

### Review Notes

- Fixed backend order-dependence in `renderer_hit_test` by resolving overlapping containing hit targets to the visually closest match with stable tie-breaking.
- Added Rust regression coverage for the real boundary cases the original review found missing: gap clicks between flows, overlapping boundary hits, repeated overlapping hits, and pan/zoom-aware shared-surface resolution.
- Re-ran validation with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

## Change Log

- 2026-03-11: Added backend and frontend regression coverage for shared-surface hit-target flow ownership, boundary correctness, repeated-hit determinism, and unchanged single-flow selection handoff.
- 2026-03-11: Fixed backend shared-surface hit resolution to prefer the visually closest containing target during overlaps, added pan/zoom and seam regression coverage, and approved the story after full validation.
