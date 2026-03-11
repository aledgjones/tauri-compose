# Story 2.2: Select Score Content in Either Visible Flow

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want to select score content in any visible flow on the shared score surface,
so that the multi-flow score view supports direct score work instead of passive reading only.

## Acceptance Criteria

1. Given two flows are visible in the same score surface, when a user selects a score element in either flow, then the correct element becomes selected, and the selection state identifies the correct flow context.
2. Given a new selection is made in one visible flow, when an earlier selection exists in another visible flow, then the selection updates predictably to the new target, and the UI does not present ambiguous multi-flow selection state unless explicitly supported.
3. Given a score with one visible flow, when selection runs through the updated path, then existing single-flow selection behavior remains correct, and current score-editing entry points remain usable.
4. Given selection works across the shared visible-flow surface, when this increment is validated, then composers can move a single clear selection between visible flows without losing target clarity, and written-notation behavior remains unchanged.

## Tasks / Subtasks

- [x] Verify the current multi-flow selection path from renderer hit-test result through sequencer state before changing behavior (AC: 1, 2, 3, 4)
  - [x] Review how `src/windows/main/components/renderer.tsx` converts flow-aware hit-test results into `actions.ui.sequencer.select*` calls and confirm every supported score-element selection variant already carries the correct `flow_id` from Rust.
  - [x] Trace the backend selection commands in `src-tauri/src/ui/sequencer.rs` to confirm each command replaces the prior selection with one authoritative sequencer selection instead of accumulating per-flow selection state.
  - [x] Verify the current selection contract against any number of visible flows on the shared score surface, not just the minimum two-flow acceptance slice.
- [x] Keep selection state singular, flow-aware, and backend-authoritative across the shared score surface (AC: 1, 2, 3, 4)
  - [x] Fix the root selection handoff only where verification shows a real gap: the selected payload crossing the renderer boundary, the backend sequencer selection update, or the UI consumers that derive details from the current selection.
  - [x] Preserve the existing rule that one new score selection replaces the prior one; do not introduce concurrent multi-flow selection state, frontend-owned flow disambiguation, or a second selection model.
  - [x] Ensure every supported selection variant continues to carry enough flow-aware identity for downstream UI and later popover work without reinterpreting which visible flow owns it.
- [x] Keep score-view selection consumers aligned with the multi-flow contract (AC: 1, 2, 3, 4)
  - [x] Review bottom-bar and properties-panel consumers such as `src/windows/main/components/selected-tone-info-display.tsx`, `src/windows/main/hooks/use-selected-tone-info.ts`, `src/windows/main/hooks/use-selected-tempo.ts`, `src/windows/main/hooks/use-selected-time-signature.ts`, `src/windows/main/hooks/use-selected-key-signature.ts`, `src/windows/main/hooks/use-selected-clef.ts`, and `src/windows/main/screens/write/components/properties-panel.tsx` so a selection from any visible flow resolves the correct flow-backed details.
  - [x] Extend the selection summary or related UI only as needed so switching selection between visible flows remains clear, without prematurely implementing the explicit active-flow affordance reserved for Story 2.5.
  - [x] Keep current single-flow behavior intact and avoid broad write-screen layout or navigation changes.
- [x] Add focused regression coverage for cross-flow selection replacement and consumer correctness (AC: 1, 2, 3, 4)
  - [x] Extend `src/windows/main/components/renderer.test.tsx` to prove clicks in later visible flows dispatch the correct flow-aware selection actions and that a miss still clears or preserves selection according to existing behavior.
  - [x] Add or update tests for selection-derived hooks and UI summaries, especially `src/windows/main/hooks/use-selected-tone-info.test.ts`, plus the existing per-selection hooks where flow-aware lookups could fail when the selected target belongs to another visible flow.
  - [x] Add Rust tests in `src-tauri/src/ui/sequencer.rs` if needed to prove the backend selection commands replace prior selection deterministically and preserve flow identity for each supported selection variant.
- [x] Update documentation only if the explicit selection contract or selection summary expectations become part of the documented architecture
  - [x] Update `docs/project-context.md` and `docs/architecture.md` only if implementation changes the documented selection ownership boundary, the singular-selection rule, or the way flow context is surfaced to downstream UI.

## Dev Notes

### Story Foundation

- Story 2.1 established that hit-testing on the shared native score surface already resolves flow-aware targets correctly and deterministically, with each supported selection payload carrying its originating `flow_id` before it reaches the frontend.
- Story 2.2 is the first increment that must prove the shared score surface is usable for actual score work. The goal is not multi-select or cross-flow editing semantics; it is one clear, authoritative selection that can move from one visible flow to another without ambiguity.
- The epic wording uses two visible flows as the minimum acceptance slice, but the implementation must stay correct for any number of visible flows rendered on the shared surface. Do not encode a two-flow ceiling into selection logic, tests, or UI summaries.
- Story 2.4 popovers and Story 2.5 explicit active-flow affordances both depend on this story preserving a clean contract: selection identifies exactly one target, carries the correct flow context, and updates predictably when the user selects a different visible flow.

### Developer Context Summary

- The current selection path already spans the shared multi-flow surface end to end: `src/windows/main/components/renderer.tsx` calls `hitTest(...)`, receives a flow-aware selection payload, and dispatches the corresponding `actions.ui.sequencer.select*` command without any frontend reinterpretation of flow ownership.
- Backend selection state is owned in `src-tauri/src/ui/sequencer.rs`, where each `ui_sequencer_selection_set_*` command writes a single `state.ui.sequencer.selection`. That is the authoritative place to preserve the one-selection-only rule when the user selects content in another visible flow.
- The UI already derives downstream context from the current sequencer selection rather than storing separate score-surface selection caches. `src/windows/main/hooks/use-selected-tone-info.ts` and the per-element hooks for tempo, time signature, key signature, and clef all resolve the selected object by following `flow_id` back into score state.
- `src/windows/main/screens/write/components/properties-panel.tsx` uses those hooks to choose the currently relevant editing panel. If flow-aware selection is wrong or incomplete, the properties panel will surface the wrong flow-backed object even when hit-testing itself was correct.
- `src/windows/main/components/selected-tone-info-display.tsx` currently presents bottom-bar context only for tone and rest selections. This story may need to make that summary more robust for cross-flow movement, but it should not turn into the explicit active-flow indicator planned for Story 2.5.
- The current architecture and project-context documents both require Rust to remain the source of truth for score selection and interaction state, with React acting as an overlay and consumer. Any fix should strengthen that boundary rather than introducing a frontend-owned selection map for visible flows.

### Technical Requirements

- Preserve one authoritative sequencer selection at a time. When a user selects a target in another visible flow, the new selection must replace the old one rather than accumulate cross-flow state.
- Keep `flow_id` present and correct on every supported selection variant crossing the Rust-to-TypeScript boundary.
- Preserve the existing selection command boundary in `src-tauri/src/ui/sequencer.rs`; do not move selection authority into React state or a new frontend cache.
- Keep selection behavior correct for any number of visible flows on the shared score surface, even though the minimum acceptance slice in the epic is two visible flows.
- Preserve the current single-flow selection path and all existing score-editing entry points.
- Keep written-notation behavior, persisted score semantics, and `.soloc` compatibility unchanged; this story should not require migration work.
- Avoid folding active-flow UX, contextual popover behavior, or selection-recovery behavior from later Epic 2 stories into this increment.

### Architecture Compliance

- Multi-flow interaction remains a derived presentation capability over existing flow data; do not create a second persisted or frontend-owned selection model.
- Rust remains the source of truth for score state, durable mutations, and sequencer selection state. React remains a renderer overlay plus UI consumer.
- Preserve `snake_case` fields and command arguments across the Tauri boundary.
- Extend the existing hit-test-to-selection pipeline instead of introducing parallel score-surface interaction logic.
- Keep explicit separation between selection correctness in this story and active-flow affordance work in Story 2.5.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep frontend/backend selection calls routed through `invoke()` via `actions.ts` with `snake_case` payloads.
- No dependency upgrade is expected for this story.
- External web research was not available in this environment; use repository-pinned versions and local docs as the authoritative baseline.

### Project Structure Notes

- Backend-owned selection state belongs under the existing UI command layer in `src-tauri/src/ui/`, while renderer hit-testing remains under `src-tauri/src/renderer/` and render composition remains under `src-tauri/src/instructions/`.
- Frontend interaction and score-view consumers should stay inside the existing write-screen/component/hook structure under `src/windows/main/`.
- Shared selection types and commands should continue to live in `src/services/types.ts` and `src/services/actions.ts`; do not create feature-local duplicates.
- No structural conflicts were found. The current architecture already has coherent seams for this work, so implementation should extend them rather than add new top-level feature folders.

### File Structure Requirements

- Primary frontend interaction seams:
  - `src/windows/main/components/renderer.tsx`
  - `src/services/hit-test.ts`
  - `src/services/actions.ts`
  - `src/services/types.ts`
- Primary backend selection seam:
  - `src-tauri/src/ui/sequencer.rs`
- Primary downstream UI consumer seams:
  - `src/windows/main/components/selected-tone-info-display.tsx`
  - `src/windows/main/hooks/use-selected-tone-info.ts`
  - `src/windows/main/hooks/use-selected-tempo.ts`
  - `src/windows/main/hooks/use-selected-time-signature.ts`
  - `src/windows/main/hooks/use-selected-key-signature.ts`
  - `src/windows/main/hooks/use-selected-clef.ts`
  - `src/windows/main/screens/write/components/properties-panel.tsx`
- Primary regression seams:
  - `src/windows/main/components/renderer.test.tsx`
  - `src/windows/main/components/selected-tone-info-display.test.tsx`
  - `src/windows/main/hooks/use-selected-tone-info.test.ts`
  - `src/windows/main/hooks/use-selected-tempo.test.ts`
  - `src/windows/main/hooks/use-selected-time-signature.test.ts`
  - `src/windows/main/hooks/use-selected-key-signature.test.ts`
  - `src/windows/main/hooks/use-selected-clef.test.ts`
- Likely out of scope unless verification proves otherwise:
  - persistence and migrations
  - playback transport
  - setup-screen flow management
  - explicit active-flow indicators

### Testing Requirements

- Extend `src/windows/main/components/renderer.test.tsx` to verify selection dispatch for targets in later visible flows and to confirm the existing miss-to-clear behavior remains intact.
- Add or extend tests for selection-derived UI summaries so a selection from a non-primary visible flow still resolves the correct instrument, stave, track, and entry details.
- Review the existing per-selection hook tests and add cross-flow coverage where a valid `flow_id` is required to resolve master-track or stave-backed data.
- Add Rust tests in `src-tauri/src/ui/sequencer.rs` if implementation changes backend selection-setting behavior, especially around replacement semantics and serialized shape for supported selection variants.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 2.1 already proved that backend hit-testing can resolve the correct target across the shared multi-flow surface and that selection payloads can preserve flow identity all the way to the frontend.
- The 2.1 implementation deliberately kept React thin: `renderer.tsx` dispatches the backend-owned selection payload directly into existing sequencer actions. Story 2.2 should preserve that pattern.
- 2.1 also hardened boundary behavior for any number of visible flows, not just two. This story should build on that guardrail rather than narrowing assumptions back down to a two-flow slice.
- The next stories depend on selection staying singular and trustworthy. If 2.2 adds ambiguity now, later popover and active-flow work will inherit that confusion.

### Git Intelligence Summary

- Recent work confirms the intended implementation path: `1079d4c6 feat(2.1): hit tet validation` touched the story artifact, `src-tauri/src/instructions/mod.rs`, `src-tauri/src/renderer/hit_test.rs`, `src/services/hit-test.test.ts`, and `src/windows/main/components/renderer.test.tsx`, reinforcing the pattern of backend-owned interaction correctness plus thin frontend dispatch.
- `4379a2af feat(1.6): render all visible flows on one surface` widened the renderer from a two-flow slice to all visible flows, so selection work must inherit that all-visible-flow model rather than reintroduce a capped assumption.
- The recent change pattern favors focused seam tests over broad architectural rewrites. Follow that approach here.

### Project Context Reference

- `docs/project-context.md` defines the score as a natively rendered Rust surface with a transparent React overlay that delegates hit-testing to Rust.
- `_bmad-output/planning-artifacts/architecture.md` requires Rust-owned score truth, derived multi-flow presentation, explicit command boundaries, and no duplicate frontend state for score-authoritative data.
- `_bmad-output/planning-artifacts/epics.md` places selection, popovers, and active-flow behavior in separate stories, which is an important scope guardrail for this increment.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.2 acceptance criteria and neighboring-story boundaries)
- Source: `_bmad-output/planning-artifacts/architecture.md` (Rust/React ownership boundaries, derived multi-flow presentation, testing expectations)
- Source: `docs/project-context.md` (native renderer overlay model, Tauri command conventions, stack versions)
- Source: `_bmad-output/implementation-artifacts/2-1-resolve-hit-targets-across-two-visible-flows.md` (previous-story learnings and all-visible-flow interaction guardrails)
- Source: `src/windows/main/components/renderer.tsx` (selection dispatch from hit-test results)
- Source: `src/services/actions.ts` (frontend selection command boundary)
- Source: `src/services/types.ts` (selection payload shape)
- Source: `src-tauri/src/ui/sequencer.rs` (authoritative backend selection state updates)
- Source: `src/windows/main/components/selected-tone-info-display.tsx` (bottom-bar selection summary)
- Source: `src/windows/main/hooks/use-selected-tone-info.ts` (tone/rest selection-derived summary)
- Source: `src/windows/main/hooks/use-selected-tempo.ts` (tempo selection flow lookup)
- Source: `src/windows/main/hooks/use-selected-time-signature.ts` (time-signature selection flow lookup)
- Source: `src/windows/main/hooks/use-selected-key-signature.ts` (key-signature selection flow lookup)
- Source: `src/windows/main/hooks/use-selected-clef.ts` (clef selection flow and stave lookup)
- Source: `src/windows/main/screens/write/components/properties-panel.tsx` (selection-driven properties-panel routing)
- Source: `src/windows/main/components/renderer.test.tsx` (frontend interaction regression seam)
- Source: `src/windows/main/components/selected-tone-info-display.test.tsx` (selection summary regression seam)
- Source: `src/windows/main/hooks/use-selected-tone-info.test.ts` (selection-derived hook regression seam)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Start verification at `src/windows/main/components/renderer.tsx` and `src-tauri/src/ui/sequencer.rs` to confirm whether the current path already satisfies the one-selection rule across visible flows before changing code.
- Use the hook and summary tests to catch subtle flow-resolution regressions after any change to selection payload handling or downstream lookup logic.
- If backend changes are required, keep them minimal and prove them with targeted Rust tests instead of broad state-model rewrites.
- Verified that `renderer.tsx` already forwards every flow-aware hit-test payload directly into backend-owned sequencer commands, and that `src-tauri/src/ui/sequencer.rs` already maintains a single authoritative `selection` slot for all supported variants.
- Confirmed with the user that surfacing flow labels in `selected-tone-info-display.tsx` is out of scope for Story 2.2, so the bottom-bar summary contract remains unchanged in this increment.
- Added targeted regressions for miss-to-clear behavior plus cross-flow lookup coverage in the tone, tempo, time-signature, key-signature, and clef selection consumers.

### Completion Notes List

- Story context prepared for story `2-2-select-score-content-in-either-visible-flow`.
- User requirement applied: the story is explicitly scoped to remain correct for any number of visible flows, not just two.
- Previous-story intelligence from Story 2.1 was incorporated so this story builds on the existing shared-surface hit-test contract rather than redefining interaction ownership.
- No persisted-data migration is expected for this story.
- No external web lookup was available; repository-pinned versions and local docs were used as the technical baseline.
- Verification showed no product-code gap in the selection handoff: the existing renderer-to-sequencer path and Rust-owned singular selection model already satisfy the story contract.
- Added regression coverage for later-flow selection dispatch, miss-to-clear behavior, and flow-aware consumer lookups without changing the existing bottom-bar selection summary contract.
- No documentation update was required because the ownership boundary, singular-selection rule, and surfaced summary contract remain unchanged.
- Validation passed with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- Final sanity check passed before closeout, and the story was moved from `review` to `done`.

### File List

- `_bmad-output/implementation-artifacts/2-2-select-score-content-in-either-visible-flow.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/windows/main/components/renderer.test.tsx`
- `src/windows/main/components/selected-tone-info-display.test.tsx`
- `src/windows/main/hooks/use-selected-tone-info.test.ts`
- `src/windows/main/hooks/use-selected-tempo.test.ts`
- `src/windows/main/hooks/use-selected-time-signature.test.ts`
- `src/windows/main/hooks/use-selected-key-signature.test.ts`
- `src/windows/main/hooks/use-selected-clef.test.ts`

## Change Log

- 2026-03-11: Verified the existing flow-aware singular-selection path and added focused cross-flow regression coverage without changing the bottom-bar summary contract.
- 2026-03-11: Final sanity check passed and Story 2.2 was marked done.
