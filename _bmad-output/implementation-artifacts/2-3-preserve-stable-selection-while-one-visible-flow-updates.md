# Story 2.3: Preserve Stable Selection While One Visible Flow Updates

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want score interaction state to remain stable while one visible flow updates,
so that local edits do not make multi-flow score interaction feel fragile or unpredictable.

This story uses two visible flows as the minimum acceptance slice from the epic, but the implementation and verification must remain correct for any number of visible flows rendered on the shared score surface.

## Acceptance Criteria

1. Given two visible flows and an active selection in one of them, when a local update occurs in that selected flow, then the selection remains stable if the target still exists, and unrelated visible flow interaction state does not become invalidated unnecessarily.
2. Given two visible flows and an active selection in one flow, when the other visible flow updates, then the existing selection remains coherent, and the user does not see contradictory selection or target state across flows.
3. Given a local update removes or invalidates the selected target, when the update completes, then selection recovery follows a defined predictable rule, and the UI does not leave stale selection state behind.
4. Given a selected target remains valid after a local update, when the user continues with keyboard-driven editing or navigation, then focus remains on the surviving score context, and subsequent commands continue to apply to the retained selected target.
5. Given this interaction-stability increment is validated, when multi-flow updates are exercised, then selection recovery follows the same defined rule across repeated update scenarios, and the implementation preserves existing notation behavior and recalculation boundaries.

## Tasks / Subtasks

- [x] Verify the current selection lifecycle across local score updates before changing behavior (AC: 1, 2, 3, 4, 5)
  - [x] Trace the current selection write path in `src-tauri/src/ui/sequencer.rs` and the score-mutation paths in `src-tauri/src/score/entries/tone.rs`, `src-tauri/src/score/entries/tempo.rs`, `src-tauri/src/score/entries/time_signature.rs`, `src-tauri/src/score/entries/key_signature.rs`, and `src-tauri/src/score/entries/clef.rs` to determine whether the backend already preserves, clears, or leaves stale selection after recalculation.
  - [x] Review how the current selection is consumed during render and anchor lookup in `src-tauri/src/instructions/mod.rs` and `src-tauri/src/ui/mod.rs`, with special attention to what happens when the selected target still exists, changes in-place, or disappears after an update.
  - [x] Verify the contract against any number of visible flows on the shared surface, covering both “selected flow updates” and “other visible flow updates” scenarios.
- [x] Keep selection recovery explicit in the mutation flows that actually invalidate the selected target (AC: 1, 2, 3, 4, 5)
  - [x] Preserve the current sequencer selection unchanged when the selected target still resolves cleanly to the same flow-owned entry after the mutation and re-render.
  - [x] When a concrete mutation flow removes or invalidates the selected target, clear the sequencer selection explicitly in that flow rather than via a generalized render-time validity pass.
  - [x] Do not add frontend-owned recovery heuristics, cross-flow fallback selection, or automatic jumps to a nearby target.
- [x] Keep downstream UI and keyboard workflows coherent when selection survives or clears (AC: 2, 3, 4)
  - [x] Verify the selection-derived hooks in `src/windows/main/hooks/use-selected-tone-info.ts`, `use-selected-tempo.ts`, `use-selected-time-signature.ts`, `use-selected-key-signature.ts`, and `use-selected-clef.ts` continue to resolve the correct flow-backed data after same-flow and other-flow updates.
  - [x] Confirm `src/windows/main/hooks/use-keyboard-shortcuts.ts` continues to target the retained selected item when it survives, and stops acting on stale selection state once the backend recovery rule clears it.
  - [x] Keep popover-driven invalidation flows coherent when selection clears as part of submission, without adding a generalized popover invalidation watcher or new interaction model.
- [x] Add focused regression coverage for shared-surface selection survival and invalidation (AC: 1, 2, 3, 4, 5)
  - [x] Add or update service-level tests around explicit invalidating mutation flows so directive removals clear selection deterministically, including guarded tick-zero clear attempts.
  - [x] Extend frontend regression coverage in `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`, `src/windows/main/hooks/use-selected-tone-info.test.ts`, and the existing per-selection hook tests to cover later-flow selections that survive updates, later-flow selections invalidated by updates, and unchanged single-flow behavior.
  - [x] Keep existing backend mutation seams under audit so same-flow edits, other-flow edits, and selected-target removals continue to exercise the expected selection outcome without introducing a generalized normalization pass.
- [x] Update documentation only if selection recovery becomes an explicit architectural rule
  - [x] Update `docs/project-context.md` and `docs/architecture.md` only if implementation formalizes a documented backend selection-normalization contract after recalculation.

## Dev Notes

### Story Foundation

- Story 2.1 established that hit-testing on the shared native score surface already returns flow-owned targets with stable `flow_id` context.
- Story 2.2 established that the application keeps one authoritative sequencer selection at a time and that downstream UI resolves details by following that selection back into score state.
- Story 2.3 is therefore not a new interaction model. It is a trust-and-coherence story: after local score mutations and recalculation, the authoritative selection must either still point at a valid target or be cleared predictably at the source of truth.
- This story is likely verification-heavy with a small root-cause fix, because several frontend consumers already guard against missing entries while a few concrete mutation paths still needed explicit selection clearing.
- Story 2.4 depends on this story’s outcome because popover anchoring is derived from the current selection. These two stories can be implemented in one validation/test batch, but they must retain separate acceptance criteria and sprint tracking.

### Developer Context Summary

- The current renderer overlay in `src/windows/main/components/renderer.tsx` remains intentionally thin: it delegates hit-testing to Rust and writes the returned flow-aware payload into backend-owned sequencer selection commands.
- Every `ui_sequencer_selection_set_*` command in `src-tauri/src/ui/sequencer.rs` writes a single `state.ui.sequencer.selection` and calls `state.emit(..., true)`, so score mutations and selection writes both currently feed the same recalculation path.
- `src-tauri/src/instructions/mod.rs` uses the current selection during draw generation to decide which notation elements render as selected, and `src-tauri/src/ui/mod.rs` resolves popover coordinates by matching the authoritative selection against `renderer.hit_targets` after composition.
- The selection-derived hooks in `src/windows/main/hooks/` already treat invalid references defensively by returning `null` if a selected object can no longer be found or no longer matches the expected tick or entry type.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts` also derives edit and delete actions from the current selection. If backend selection is stale while frontend consumers null out, the user can end up in a contradictory state where some UI looks cleared but the source of truth still claims something is selected.
- Other-flow updates should not disturb a valid selection because selection identity already includes flow-owned IDs and ticks. The only time recovery should change the selection is when the selected target no longer resolves cleanly after the update.

### Technical Requirements

- Keep backend-owned selection identity authoritative, while preserving the existing explicit action/invoke selection management model instead of introducing a generalized render-time normalization pass.
- Preserve the current selection unchanged when the selected target still exists and still resolves to the same flow-owned item after recalculation.
- When the selected target no longer exists or no longer resolves cleanly, clear selection predictably rather than leaving stale state behind.
- Ensure updates in another visible flow do not clear or rewrite a valid selection in the non-edited flow.
- Keep current single-flow selection behavior intact.
- Preserve notation behavior, written-duration semantics, recalculation boundaries, and `.soloc` semantics; this story should not require migration work.
- Do not fold active-flow affordances from Story 2.5 or broader cross-surface active-flow behavior from Story 2.6 into this story.

### Architecture Compliance

- Multi-flow interaction remains a derived presentation capability over existing flow data; do not introduce a second selection model in React or in persisted state.
- Rust remains the source of truth for score state, durable mutations, and sequencer selection state. React remains a renderer overlay plus consumer of backend-derived state.
- Preserve `snake_case` fields and command arguments across the Tauri boundary.
- Keep selection recovery explicit in the mutation flows that invalidate selection rather than layering frontend heuristics or a generalized render-time validity pass on top.
- Keep selection stability distinct from active-flow UI and distinct from contextual-popover feature work, except where stale selection would otherwise break popover coherence.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep FE→BE selection and mutation calls routed through `invoke()` via `src/services/actions.ts` with `snake_case` payloads.
- No dependency upgrade is expected for this story.
- External web research was not available in this environment; use repository-pinned versions and local docs as the authoritative baseline.

### File Structure Requirements

- Primary backend selection seam:
  - `src-tauri/src/ui/sequencer.rs`
- Primary backend mutation seams:
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/score/entries/tempo.rs`
  - `src-tauri/src/score/entries/time_signature.rs`
  - `src-tauri/src/score/entries/key_signature.rs`
  - `src-tauri/src/score/entries/clef.rs`
- Primary render and selection-consumer seams:
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/ui/mod.rs`
  - `src/windows/main/components/renderer.tsx`
  - `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - `src/windows/main/hooks/use-selected-tone-info.ts`
  - `src/windows/main/hooks/use-selected-tempo.ts`
  - `src/windows/main/hooks/use-selected-time-signature.ts`
  - `src/windows/main/hooks/use-selected-key-signature.ts`
  - `src/windows/main/hooks/use-selected-clef.ts`
- Likely out of scope unless verification proves otherwise:
  - persistence and migrations
  - active-flow indicators
  - sequencer/piano-roll active-flow switching

### Testing Requirements

- Add backend tests that prove selection remains intact when the selected target survives a same-flow update.
- Add backend tests that prove updates in another visible flow do not clear or mutate a valid existing selection.
- Add backend tests that prove removal or invalidation of the selected target clears selection deterministically instead of leaving stale state behind.
- Extend `src/windows/main/hooks/use-keyboard-shortcuts.test.ts` so keyboard-driven edit and delete flows behave correctly when selection survives and when it is invalidated.
- Extend the existing selection-hook tests so later-flow selections still resolve after unrelated updates and collapse to `null` only when the backend has intentionally cleared or invalidated them.
- Reuse Story 2.4 popover tests where helpful, but keep this story’s pass condition centered on authoritative selection stability rather than popover features.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 2.1 proved the shared-surface hit-target contract is already backend-owned and flow-aware, including composed popover anchor coordinates for later visible flows.
- Story 2.2 proved the application can move one authoritative selection between visible flows without introducing concurrent multi-flow selection state.
- Story 2.2 also hardened the selection-derived hooks and renderer tests, which makes this story a natural verification-first follow-on rather than a greenfield selection rewrite.
- The next popover story depends on the same invariant: selection must either still mean one valid target or be cleared cleanly.

### Git Intelligence Summary

- `ee4f6ef4 test(2.2): cover multi-flow selection consumers` shows the recent implementation pattern: add focused seam tests around existing flow-aware selection behavior before widening scope.
- `1079d4c6 feat(2.1): hit tet validation` and `4379a2af feat(1.6): render all visible flows on one surface` confirm the current interaction model is already shared-surface and all-visible-flow aware.
- Recent history suggests the correct approach here is to verify first, then make the smallest backend correction needed to keep selection trustworthy during recalculation.

### Project Context Reference

- `docs/project-context.md` defines the score as a natively rendered Rust surface with a transparent React overlay and backend-owned authoritative state.
- `_bmad-output/planning-artifacts/architecture.md` requires multi-flow work to remain a derived presentation layer, preserve Rust ownership of score truth, and avoid duplicate frontend state.
- `_bmad-output/planning-artifacts/epics.md` separates selection stability, contextual popovers, active-flow affordances, and cross-surface active-flow behavior into distinct increments. Preserve those boundaries.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.3 acceptance criteria and neighboring-story boundaries)
- Source: `_bmad-output/planning-artifacts/architecture.md` (Rust ownership, derived multi-flow presentation, testing expectations)
- Source: `docs/project-context.md` (native renderer overlay model, repo-pinned stack, command-boundary rules)
- Source: `_bmad-output/implementation-artifacts/2-1-resolve-hit-targets-across-two-visible-flows.md` (flow-aware hit-target and popover-anchor contract)
- Source: `_bmad-output/implementation-artifacts/2-2-select-score-content-in-either-visible-flow.md` (singular backend-owned selection model and consumer coverage)
- Source: `src-tauri/src/ui/sequencer.rs` (authoritative selection slot and selection commands)
- Source: `src-tauri/src/instructions/mod.rs` (selection-aware draw path and shared-surface composition)
- Source: `src-tauri/src/ui/mod.rs` (`ui_get_selection_coordinates` selection-to-hit-target lookup)
- Source: `src-tauri/src/score/entries/tone.rs` (tone mutation paths)
- Source: `src-tauri/src/score/entries/tempo.rs` (tempo mutation paths)
- Source: `src-tauri/src/score/entries/time_signature.rs` (time-signature mutation paths)
- Source: `src-tauri/src/score/entries/key_signature.rs` (key-signature mutation paths)
- Source: `src-tauri/src/score/entries/clef.rs` (clef mutation paths)
- Source: `src/windows/main/components/renderer.tsx` (selection handoff from hit-test to backend)
- Source: `src/windows/main/hooks/use-keyboard-shortcuts.ts` (selection-driven keyboard edit and delete flows)
- Source: `src/windows/main/hooks/use-selected-tone-info.ts` (tone/rest selection validation)
- Source: `src/windows/main/hooks/use-selected-tempo.ts` (tempo selection validation)
- Source: `src/windows/main/hooks/use-selected-time-signature.ts` (time-signature selection validation)
- Source: `src/windows/main/hooks/use-selected-key-signature.ts` (key-signature selection validation)
- Source: `src/windows/main/hooks/use-selected-clef.ts` (clef selection validation)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Completion Notes List

- Story context prepared from the user-specified paired batch covering Stories 2.3 and 2.4.
- This story is framed as verification-first and likely test-heavy, with the accepted fix centered on explicit invalidation handling in concrete mutation flows rather than a generalized normalization seam.
- Story 2.3 and Story 2.4 can be implemented together in one branch and validation run because popover anchoring depends on the same selection-trust contract, but their sprint statuses remain separate.
- No persisted-data migration is expected for this story.
- No external web lookup was available; local docs, current repo state, recent implementation artifacts, and recent git history were used as the technical baseline.
- The workflow referenced `_bmad/core/tasks/validate-workflow.xml`, but that helper file is not present in this repo; the create-story checklist was applied manually during story preparation.
- Preserved the existing explicit-selection-management model for same-target updates and replaced the generalized render-time normalization experiment with explicit selection clears only in invalidating mutation flows.
- Added explicit selection clears for popover-driven directive removals in `src/services/popovers.ts`, including tick-zero clear attempts so the UI still reflects that the guarded removal path ran.
- Kept defensive selection consumers and added regression coverage for later-flow selection survival plus explicit clear behavior in popover mutation paths.
- Validation passed: `npm run lint`, `npm run check-types`, full JS test suite (`1016` passed), `npm run test:rust` (`932` passed), and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- Documentation remained unchanged because the final implementation did not introduce a new documented architecture rule; it stayed within existing explicit action/invoke selection management patterns.

### File List

- `_bmad-output/implementation-artifacts/2-3-preserve-stable-selection-while-one-visible-flow-updates.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/services/popovers.test.ts`
- `src/services/popovers.ts`
- `src/ui/popover.test.tsx`
- `src/ui/popover.tsx`
- `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
- `src/windows/main/hooks/use-selected-tone-info.test.ts`
