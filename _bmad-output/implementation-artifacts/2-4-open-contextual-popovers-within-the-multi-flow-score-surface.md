# Story 2.4: Open Contextual Popovers Within the Multi-Flow Score Surface

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want contextual score popovers to work in either visible flow,
so that multi-flow score view supports the same local directive and editing workflows as single-flow score view.

This story uses two visible flows as the minimum acceptance slice from the epic, but the implementation and verification must remain correct for any number of visible flows rendered on the shared score surface.

## Acceptance Criteria

1. Given two visible flows in the score surface, when a user invokes a contextual popover for a supported score interaction in either flow, then the correct popover opens for the selected target, and the popover is anchored to the correct visual flow context.
2. Given a target near the boundary between visible flows, when a contextual popover is opened, then the popover resolves against the correct underlying target, and it does not appear associated with the wrong flow.
3. Given a score with one visible flow, when contextual popovers run through the updated path, then existing single-flow popover behavior remains correct, and current directive workflows remain usable.
4. Given a contextual popover is opened from the multi-flow score surface, when the user navigates by keyboard, then focus moves predictably into the popover and back to the originating score context when the popover closes, and the visible score context remains clear while the popover is open.
5. Given this popover increment is validated, when popovers open across the visible multi-flow surface, then composers can complete supported popover-based directive workflows in either visible flow without leaving score context, and notation-derivation and persisted score behavior remain unchanged.

## Tasks / Subtasks

- [x] Verify the current popover invocation and anchor contract on the shared score surface before changing behavior (AC: 1, 2, 3, 4, 5)
  - [x] Review `src/ui/popover.tsx`, `src/services/popovers.ts`, `src/services/actions.ts`, `src-tauri/src/ui/mod.rs`, and the shared-surface hit-target composition in `src-tauri/src/instructions/mod.rs` to confirm how a selected target becomes an anchored popover.
  - [x] Verify that `ui_get_selection_coordinates` already resolves correct anchor coordinates for selections in later visible flows and near inter-flow boundaries, including current zoom and pan.
  - [x] Confirm the story remains valid for any number of visible flows rather than baking in a two-flow ceiling.
- [x] Keep contextual popover anchoring backend-owned and flow-aware (AC: 1, 2, 3, 5)
  - [x] Reuse the current flow-aware selection plus `ui_get_selection_coordinates` contract rather than computing popover anchors in React.
  - [x] If validation exposes a gap, fix it at the root seam so the authoritative selection resolves exactly one correct composed hit target and anchor across the shared surface.
  - [x] Preserve current single-flow behavior and current directive-entry workflows while extending correctness across visible flows.
- [x] Preserve keyboard-focus and score-context continuity while a popover is open (AC: 4, 5)
  - [x] Ensure focus moves predictably into the popover input when it opens and returns to the originating score interaction context, renderer overlay, or previously focused score-adjacent control when the popover closes.
  - [x] Keep the visible score context clear while the popover is open without taking on the later active-flow coherence work.
  - [x] Ensure wheel/pan changes, selection changes, or invalidated selections do not leave orphaned popover state anchored to a stale target.
- [x] Add focused regression coverage for multi-flow popover correctness and continuity (AC: 1, 2, 3, 4, 5)
  - [x] Add Rust tests around `src-tauri/src/ui/mod.rs` so later-flow selections and boundary-adjacent selections resolve the correct composed coordinates after zoom and pan are applied.
  - [x] Extend `src/ui/popover.test.tsx` to cover later-flow selections, keyboard invocation for the supported popover types, close behavior when selection becomes invalid, and focus-return expectations on close.
  - [x] Extend `src/windows/main/components/renderer.test.tsx` only as needed to prove selection plus popover invocation remains coherent on the shared surface.
- [x] Update documentation only if the popover anchor contract becomes an explicit architectural guarantee
  - [x] Update `docs/project-context.md` and `docs/architecture.md` only if implementation formalizes backend-owned popover anchor guarantees or focus-return rules beyond what is already implied by the existing renderer model.

## Dev Notes

### Story Foundation

- Story 2.1 established the shared-surface hit-target contract, including flow-aware popover anchor coordinates on hit targets after multi-flow composition.
- Story 2.2 established that score selection is singular, flow-aware, and backend-owned, which is the prerequisite for opening popovers against the correct selected target.
- Story 2.4 is therefore primarily a correctness-and-continuity story, not a new editing surface. The expected implementation is likely verification-heavy, with small fixes only where anchor lookup or focus restoration proves incomplete.
- The current product already uses keyboard-triggered contextual popovers (`Shift+B`, `Shift+C`, `Shift+K`, `Shift+M`, `Shift+T`) anchored from the current selection. That existing interaction model should be extended across multi-flow score view rather than replaced.
- Story 2.3 and Story 2.4 can be implemented together because popover trust depends on stable selection, but keep the acceptance criteria and sprint tracking separate.

### Developer Context Summary

- `src/ui/popover.tsx` opens contextual popovers from keyboard shortcuts only when there is a current sequencer selection and `actions.ui.renderer.getSelectionCoordinates()` returns anchor coordinates.
- `src-tauri/src/ui/mod.rs` implements `ui_get_selection_coordinates` by matching the authoritative sequencer selection against `renderer.hit_targets`, then converting the selected hit target’s `popover_x` and `popover_y` through the current zoom and pan.
- `src-tauri/src/instructions/mod.rs` already composes all visible flows into one shared surface and offsets hit targets, including popover anchor coordinates, into shared renderer space.
- `src/services/popovers.ts` parses the user’s input and routes the resulting command using the current `flow_id`, `tick`, and, for clefs, `stave_id` derived from the current selection.
- `src/ui/popover.tsx` already autofocuses the input and closes on blur or Escape, but explicit focus restoration to the originating score context is not currently documented and should be verified or hardened as needed.
- `src/windows/main/components/renderer.tsx` closes any open popover on wheel-driven pan or zoom. If selection is invalidated while a popover is open, that stale UI state should close coherently rather than remaining attached to a dead target.

### Technical Requirements

- Keep popover anchor calculation backend-owned through `ui_get_selection_coordinates`; do not calculate score-space anchor positions in React.
- Preserve the current flow-aware selection contract so popover parsing and directive creation run against the correct `flow_id`, `tick`, and `stave_id`.
- Ensure composed later-flow targets and boundary-adjacent targets resolve to the correct visual flow context.
- Keep current single-flow popover workflows intact.
- Ensure keyboard focus moves into the popover on open and returns predictably to score context on close.
- Close or suppress stale popovers when selection is invalidated or when the current score context no longer supports the anchored target.
- Preserve notation behavior, persisted score semantics, and `.soloc` compatibility; this story should not require migration work.
- Do not expand scope into explicit active-flow UI, mouse-only context menus, or large modal editing flows.

### Architecture Compliance

- Multi-flow popovers remain a derived interaction capability over backend-owned selection and renderer-owned hit-target geometry.
- Rust remains the source of truth for score state, selection identity, and anchor coordinates. React remains responsible for lightweight popover rendering and input handling only.
- Preserve `snake_case` fields and commands across the Tauri boundary.
- Extend the current popover contract instead of introducing a parallel frontend geometry system or a second selection model.
- Keep contextual popover correctness distinct from the later active-flow coherence work.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep FE→BE routing through `invoke()` in `src/services/actions.ts` with `snake_case` payloads.
- No dependency upgrade is expected for this story.
- External web research was not available in this environment; use repository-pinned versions and local docs as the authoritative baseline.

### File Structure Requirements

- Primary frontend popover seams:
  - `src/ui/popover.tsx`
  - `src/ui/popover.test.tsx`
  - `src/services/popovers.ts`
  - `src/services/actions.ts`
- Primary backend anchor and composition seams:
  - `src-tauri/src/ui/mod.rs`
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/renderer/hit_test.rs`
- Primary shared interaction seam:
  - `src/windows/main/components/renderer.tsx`
- Dependent selection-stability seam:
  - `src-tauri/src/ui/sequencer.rs`
- Likely out of scope unless verification proves otherwise:
  - active-flow chrome
  - persistence and migrations
  - alternate context-menu systems

### Testing Requirements

- Add Rust tests proving `ui_get_selection_coordinates` resolves correct anchor positions for later visible flows and boundary-adjacent targets after composition, zoom, and pan.
- Extend `src/ui/popover.test.tsx` to verify supported popovers open and submit correctly from selections in later visible flows, not just primary-flow fixtures.
- Add tests that verify popovers close cleanly when selection is cleared or becomes invalid, and that focus restoration remains predictable after Escape, blur, and successful submission.
- Extend renderer seam tests only where needed to keep the selection-to-popover bridge trustworthy.
- Validate together with Story 2.3 if both are implemented in one batch, because popover anchoring depends on the same selection-trust contract.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 2.1 already proved the backend can keep flow-owned popover anchor coordinates aligned with shared-surface hit targets across visible flows.
- Story 2.2 proved selection can move cleanly between visible flows without frontend-owned disambiguation, which is the key prerequisite for trustworthy contextual popovers.
- Story 2.3 is the adjacent stability story and should be treated as a dependency for any stale-selection cleanup or focus-continuity fixes.
- The prior implementation pattern in this epic is to prove the current contract first, then tighten only the minimal backend or seam behavior that fails the new regression tests.

### Git Intelligence Summary

- `ee4f6ef4 test(2.2): cover multi-flow selection consumers` reinforces that recent work has favored targeted seam tests around existing flow-aware behavior rather than broad rewrites.
- `1079d4c6 feat(2.1): hit tet validation` confirms popover correctness should continue to rely on backend-owned flow-aware hit targets and anchors.
- `4379a2af feat(1.6): render all visible flows on one surface` means popover verification must assume all visible flows can be present on the shared surface, not just the initial two-flow slice.

### Project Context Reference

- `docs/project-context.md` defines the score as a natively rendered Rust surface with a transparent React overlay and backend-owned hit-testing.
- `_bmad-output/planning-artifacts/architecture.md` requires interaction mapping, selection, and popover behavior to extend existing backend/frontend contracts rather than introduce a parallel model.
- `_bmad-output/planning-artifacts/epics.md` keeps popovers, active-flow affordances, and cross-surface active-flow semantics in separate stories. Preserve that separation.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.4 acceptance criteria and neighboring-story boundaries)
- Source: `_bmad-output/planning-artifacts/architecture.md` (interaction-contract guidance, Rust ownership, testing expectations)
- Source: `docs/project-context.md` (native renderer overlay model, shared-surface flow composition, Tauri command conventions)
- Source: `_bmad-output/implementation-artifacts/2-1-resolve-hit-targets-across-two-visible-flows.md` (flow-aware hit-target and popover-anchor groundwork)
- Source: `_bmad-output/implementation-artifacts/2-2-select-score-content-in-either-visible-flow.md` (backend-owned singular selection groundwork)
- Source: `_bmad-output/implementation-artifacts/2-3-preserve-stable-selection-while-one-visible-flow-updates.md` (selection-stability pairing and stale-state guardrails)
- Source: `src/ui/popover.tsx` (current keyboard-triggered popover behavior)
- Source: `src/ui/popover.test.tsx` (existing frontend popover regression seam)
- Source: `src/services/popovers.ts` (directive parse-and-dispatch behavior)
- Source: `src/services/actions.ts` (selection-coordinate IPC and directive command routing)
- Source: `src-tauri/src/ui/mod.rs` (`ui_get_selection_coordinates`)
- Source: `src-tauri/src/instructions/mod.rs` (shared-surface hit-target and anchor composition)
- Source: `src-tauri/src/renderer/hit_test.rs` (flow-aware target resolution before popover anchoring)
- Source: `src/windows/main/components/renderer.tsx` (renderer overlay and popover-close behavior during pan/zoom)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Completion Notes List

- Story context prepared from the user-specified paired batch covering Stories 2.3 and 2.4.
- This story is framed as verification-first and likely test-heavy, with any implementation expected to tighten existing anchor and focus continuity seams rather than introduce a new popover model.
- Story 2.3 and Story 2.4 can be implemented together in one branch and validation run because popover behavior depends on the same backend-owned selection contract, but their sprint statuses remain separate.
- No persisted-data migration is expected for this story.
- No external web lookup was available; local docs, current repo state, recent implementation artifacts, and recent git history were used as the technical baseline.
- The workflow referenced `_bmad/core/tasks/validate-workflow.xml`, but that helper file is not present in this repo; the create-story checklist was applied manually during story preparation.
- Verified the shared-surface popover contract remains backend-owned by reusing `ui_get_selection_coordinates` and keeping React on the existing keyboard-triggered popover path.
- Simplified the frontend back to event-local popover behavior after verification, retaining the existing open, submit, blur, Escape, and wheel-close flow instead of layering continuous anchor sync and explicit focus restoration state.
- Added Rust regression coverage for later-flow and boundary-adjacent selection coordinate resolution and kept focused frontend regression coverage around keyboard-triggered popover invocation and renderer continuity.
- Validation passed with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### File List

- `_bmad-output/implementation-artifacts/2-4-open-contextual-popovers-within-the-multi-flow-score-surface.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/ui/popover.tsx`
- `src/ui/popover.test.tsx`
- `src/windows/main/components/renderer.tsx`
- `src/windows/main/components/renderer.test.tsx`
- `src-tauri/src/ui/mod.rs`

## Change Log

- 2026-03-11: Verified the multi-flow popover anchor path, kept anchor resolution backend-owned, and simplified the frontend back to the existing event-driven popover flow.
- 2026-03-11: Added Rust regression coverage for later-flow anchor resolution and boundary-adjacent targets, plus focused frontend regression coverage for keyboard-triggered popovers and renderer wheel continuity.
