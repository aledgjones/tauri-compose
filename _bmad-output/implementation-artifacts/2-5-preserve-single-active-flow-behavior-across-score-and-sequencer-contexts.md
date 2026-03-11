# Story 2.5: Preserve Single Active-Flow Behavior Across Score and Sequencer Contexts

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want score view multi-flow visibility to coexist with a single active flow for sequencer and piano-roll work,
so that the broader editing model stays coherent while the score view becomes more document-like.

This story uses the current Sequence workflow as the concrete single-flow editing surface already present in the product. The epic mentions piano-roll workflows more broadly, but implementation should extend the existing sequence and miniature-piano-roll surfaces rather than invent a new screen or navigation model in this increment.

## Acceptance Criteria

1. Given the score is showing multiple visible flows, when the user opens the Sequence workflow, then the Sequence workflow exposes one explicit active-flow switcher inside that workflow, and the shared top bar does not expose a workflow-specific flow selector.
2. Given the user changes the local Sequence active-flow switcher from Flow A to Flow B, when the change is applied, then `ui.flow_id` updates to Flow B and all Sequence-screen consumers that depend on the active flow update to Flow B without collapsing the score to a single visible flow.
3. Given the user deliberately selects score content that belongs to a different visible flow, when that selection is committed, then the shared active flow updates to the selected content's flow and the Sequence workflow follows that flow automatically.
4. Given the user pans, zooms, hovers, reads another visible flow, or clears selection by clicking empty score space, when no new score object is deliberately selected, then the shared active flow does not change.
5. Given the score shows multiple flows while Sequence remains a single-flow editing surface, when the active flow changes, then Sequence, miniature piano-roll views, instrument overview, and related single-flow controls all continue to show only one coherent active flow at a time.
6. Given the project contains only one flow, when score and Sequence workflows are used, then behavior matches the existing single-flow model and the new cross-flow interaction rules introduce no regression.
7. Given score and Sequence workflows are exercised together after this change, when active-flow handoff occurs between them, then notation behavior, playback coherence, and persisted score semantics remain unchanged.

## Implied Interaction Model

The current acceptance criteria imply the following user-facing behavior and should be treated as the intended journey unless implementation discovery proves a conflict:

1. Sequence owns the explicit active-flow switch.
   The user opens the Sequence workflow and sees a flow selector inside that workflow, not in the shared top bar. Changing that selector updates the one shared active flow for sequencer-style editing.
2. Score selection can hand off the active flow.
   When the user deliberately selects a note, rest, clef, key signature, time signature, or tempo marking in a different visible flow on the rendered score, that targeted selection is sufficient to move the shared active flow to that flow so the Sequence workflow follows it.
3. Passive score navigation does not hand off the active flow.
   Panning, zooming, hovering, reading another visible staff, or clearing selection by clicking empty space should not silently retarget the Sequence workflow to a different flow.
4. Score visibility and Sequence edit scope are intentionally different.
   The score may show multiple flows at once for reading and navigation, but Sequence and miniature piano-roll surfaces continue to show one active flow at a time.
5. Cross-surface updates should feel immediate.
   If the user changes the active flow in Sequence, the score remains multi-flow but any active-flow-derived screen state updates immediately. If the user deliberately selects score content in another flow, Sequence updates immediately to that flow without requiring a second manual switch.
6. Single-flow projects should feel unchanged.
   In a one-flow score, the local Sequence flow selector may be hidden or inert, and score or Sequence interaction should behave exactly as today's single-flow model does.

## User Journeys

### Journey 1: Switch flow from Sequence

1. The user opens the Sequence workflow while the score shows multiple visible flows.
2. The Sequence screen shows a local flow dropdown or equivalent switcher.
3. The user changes the selector from Flow A to Flow B.
4. `ui.flow_id` updates to Flow B.
5. Sequencer lanes, miniature piano-roll content, instrument overview, and any active-flow-derived controls update to Flow B.
6. The score remains in multi-flow view; it does not collapse to Flow B only.

### Journey 2: Select score content in another flow

1. The user is currently editing Flow A in Sequence.
2. In the rendered score, the user clicks a note in visible Flow B.
3. The app records a selection targeting Flow B.
4. Because this was a deliberate score selection, the shared active flow also moves to Flow B.
5. Sequence updates to Flow B automatically.
6. Any follow-on editing affordances remain aligned to Flow B without requiring a separate flow change.

### Journey 3: Read across flows without disrupting Sequence

1. The user is editing Flow A in Sequence while both Flow A and Flow B are visible in the score.
2. The user pans, zooms, or visually scans Flow B without selecting content in it.
3. The score view changes as expected for navigation.
4. The shared active flow stays on Flow A.
5. Sequence continues to show Flow A.

### Journey 4: Clear selection without changing active flow

1. The user has selected content in Flow B, and Sequence is currently showing Flow B.
2. The user clicks empty score space to clear the selection.
3. Selection clears.
4. The shared active flow remains Flow B unless another explicit flow change occurs.
5. Sequence stays on Flow B so the user does not lose editing context from an incidental deselect.

### Journey 5: Work in a single-flow score

1. The user opens a project with only one flow.
2. The score, Sequence workflow, and transport all resolve to that one flow.
3. No additional cross-flow behavior appears.
4. Existing single-flow editing behavior remains unchanged.

## Interaction Rules To Preserve

- One authoritative active flow exists across score, Sequence, and transport contexts.
- Explicit Sequence flow switching changes the active flow.
- Deliberate score-object selection may change the active flow.
- Passive score navigation must not change the active flow.
- Clearing selection must not implicitly revert to some default or first flow.
- Multi-flow score visibility must not imply multi-flow Sequence editing.
- Shared shell chrome should not own workflow-specific flow switching.

## Open Decisions Resolved By This Story Draft

- Selecting a score object in another visible flow should switch the shared active flow.
- Clearing selection should preserve the last active flow rather than reverting to a default flow.
- Passive score navigation should not switch the active flow.
- The flow switcher should live in the Sequence workflow, not in the shared top bar.
- The current Sequence workflow is the concrete implementation target for the epic's sequencer and piano-roll wording in this increment.

## Tasks / Subtasks

- [x] Verify the current authoritative active-flow seams before changing behavior (AC: 1, 2, 3, 4, 5, 6)
  - [x] Audit `src-tauri/src/ui/mod.rs`, `src/services/state/provider.tsx`, `src/windows/main/index.tsx`, `src/windows/main/components/renderer.tsx`, `src-tauri/src/ui/sequencer.rs`, `src/windows/main/hooks/use-bottom-panel-state.ts`, and `src/windows/main/screens/sequence/components/instrument-overview.tsx` to confirm where the current single active flow lives, how score selection and sequencing use it, and which UI surfaces still expose it in shared shell chrome.
  - [x] Confirm the current Sequence workflow is the real concrete implementation target for the epic's “sequencer and piano-roll” language, and avoid inventing a new workflow surface in this story.
  - [x] Verify the current transport synchronization path so active-flow changes remain coherent with playback state derived from `ui.flow_id`.
- [x] Keep one backend-owned active flow coherent across score and sequence surfaces (AC: 1, 2, 5, 6)
  - [x] Reuse the existing `state.ui.flow_id` as the one authoritative active-flow slot; do not create a second score-only or sequencer-only active-flow state.
  - [x] Ensure deliberate score-object selection in another visible flow updates that same authoritative slot, while passive reading, pan, zoom, hover, and miss-to-clear behavior do not fabricate contradictory state.
  - [x] Ensure sequence-screen consumers that currently derive their flow from `ui.flow_id` remain aligned with score interaction changes.
- [x] Move single-flow switching controls into workflow-local surfaces instead of the shared top bar (AC: 2, 3, 4, 5, 6)
  - [x] Remove the persistent active-flow selector from `src/windows/main/index.tsx` so the shared top bar returns to universal tools and information only.
  - [x] Add an explicit active-flow switch control inside the Sequence workflow using the current screen structure, so users can change the single active flow for sequencing without depending on the score-view reading model.
  - [x] Keep the score view readable without requiring any workflow-local selector for basic navigation across visible flows.
  - [x] Keep layout selection and transport placement coherent with the existing shell unless a focused adjustment is required by the moved flow control.
- [x] Keep sequence-screen editing surfaces explicitly single-flow while score view remains multi-flow (AC: 1, 2, 3, 5, 6)
  - [x] Ensure `use-bottom-panel-state` and the Sequence workflow continue to derive one concrete flow for sequencer editing, miniature piano rolls, instrument-overview filtering, and related controls.
  - [x] Keep `show_active_only` and related sequence-screen filters coherent with the active flow selected through the new workflow-local control.
  - [x] Avoid implying that multiple flows are editable simultaneously in sequencer-style surfaces.
- [x] Add focused regression coverage for cross-surface active-flow coherence (AC: 1, 2, 3, 4, 5, 6)
  - [x] Add frontend tests around the main shell to prove the shared top bar no longer owns the flow selector and that the Sequence workflow exposes the active-flow switch locally.
  - [x] Extend sequence-screen tests and any hook tests needed to prove active-flow changes propagate correctly through `use-bottom-panel-state`, instrument overview, and sequencer-derived controls.
  - [x] Extend renderer and interaction tests only as needed to prove deliberate score interactions can keep the authoritative active flow coherent with the sequence workflow without breaking passive reading behavior.
  - [x] Add backend tests around the authoritative active-flow update seam if implementation changes command behavior or `ui.flow_id` ownership semantics.
- [x] Update documentation only if this story formalizes a stable shell and active-flow rule (AC: 3, 4, 6)
  - [x] Update `docs/project-context.md`, `docs/architecture.md`, and AI-agent instructions if implementation formalizes that workflow-local single-flow selectors belong inside Sequence or related workflow surfaces rather than in shared top-bar chrome.

## Dev Notes

### Story Foundation

- Story 2.1 established flow-aware hit-testing on the shared score surface.
- Story 2.2 established that score selection is singular, flow-aware, and backend-owned.
- Story 2.3 established that selection stays trustworthy through updates and explicit invalidation.
- Story 2.4 established that contextual popovers remain derived from the same authoritative selection contract.
- With the deleted standalone active-flow-visibility story removed, this story is now the single remaining Epic 2 step for active-flow coherence. It should solve the real product problem: keeping one app-level active flow coherent across score, sequence, and playback surfaces without adding ornamental score-only state.
- Story 1.5 remains negative precedent: passive page labels and broadened bottom-bar flow summaries were rejected after live review. Do not resurrect those patterns while solving cross-surface active-flow coherence.

### Developer Context Summary

- `state.ui.flow_id` already exists as the app-level active-flow slot in `src-tauri/src/ui/mod.rs`, and `ui_flow_id_set` already synchronizes transport state through `sync_transport_to_app_state(...)`.
- `src/services/state/provider.tsx` already derives the frontend active flow from `ui.flow_id` with fallback to `score.flows.order[0]`, and that value drives transport tempo synchronization as well as other current single-flow consumers.
- `src/windows/main/index.tsx` still renders a global flow selector in the shared top bar. That directly conflicts with the UX direction that workflow-local single-flow selectors belong inside the relevant surface rather than in shared shell chrome.
- `src/windows/main/components/renderer.tsx` forwards flow-aware hit-test results into sequencer selection commands, but it does not itself update `ui.flow_id`; if score interaction should influence the shared active flow, that must happen through the authoritative backend-owned seam.
- `src-tauri/src/ui/sequencer.rs` already keeps one authoritative selection with flow-aware identity, which gives the implementation enough information to align the shared active flow with deliberate score editing when appropriate.
- The current Sequence workflow is the concrete single-flow editing surface already present in the app. `src/windows/main/hooks/use-bottom-panel-state.ts` derives one `flowId` from `ui.flow_id`, and `src/windows/main/components/sequencer.tsx` plus `src/windows/main/screens/sequence/components/instrument-overview.tsx` already assume one concrete active flow.
- `src-tauri/src/ui/screens/sequence.rs` exposes `show_active_only`, which is another signal that the Sequence workflow already assumes a single active flow. This story should strengthen that model rather than replace it.
- The repo does not currently expose a separate piano-roll screen beyond the Sequence workflow and miniature piano-roll views, so implementation should avoid speculative UI expansion.

### Technical Requirements

- Keep one backend-owned active-flow source of truth by reusing `ui.flow_id`.
- Deliberate score-object selection may update the shared active flow when needed for cross-surface coherence, but passive reading interactions must not fabricate contradictory state.
- The shared top bar should contain universal tools and information only; single-flow selectors should move into workflow-local surfaces.
- The Sequence workflow must continue to behave as a single-flow editing context even while score view shows multiple visible flows.
- Keep workflow-local active-flow switching explicit inside the relevant screen so users can control sequencer/piano-roll scope without using score-reading controls as an indirect proxy.
- Clearing score selection must not silently revert the active flow to a default or first visible flow.
- Preserve current single-flow behavior and keep score reading independent from the sequence workflow's local flow switcher.
- Preserve notation behavior, written-duration semantics, render composition, playback stability, persisted score semantics, and `.soloc` compatibility; this story should not require migration work.
- Do not introduce a second frontend-owned active-flow model, a duplicated transport-flow state, or score-surface ornamentation meant only to signal activeness.
- Treat the current Sequence workflow as the concrete implementation target for the epic's “piano-roll” wording unless a later product decision adds a distinct screen.

### Architecture Compliance

- Multi-flow presentation remains a derived view over existing flow data; active-flow semantics must stay distinct from visible-flow presentation while remaining coherent across surfaces.
- Rust remains the source of truth for score state, active-flow state, and durable mutations. React remains an orchestration shell and consumer of pushed state.
- Preserve `snake_case` fields and command arguments across the Tauri boundary.
- Extend the existing command and event model instead of introducing local frontend synchronization hacks between score, sequence, and playback surfaces.
- Keep workflow-local selector placement aligned with the UX rule that the shared top bar remains universal and workflow-specific controls live inside the relevant workflow surface.
- Avoid creating speculative “piano-roll” abstractions that do not match the current codebase; extend the existing Sequence workflow and miniature-piano-roll surfaces instead.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep FE→BE routing through `invoke()` in `src/services/actions.ts` with `snake_case` payloads.
- Reuse the existing shell, Sequence workflow, state-provider logic, and UI-state model before introducing new abstractions.
- No dependency upgrade is expected for this story.
- External web research was not available in this environment; use repository-pinned versions, local docs, and current source as the authoritative baseline.

### Project Structure Notes

- Backend-owned active-flow state belongs under the existing UI command layer in `src-tauri/src/ui/`.
- Shared shell changes belong in `src/windows/main/index.tsx` and the existing main-window screen/component structure.
- Workflow-local single-flow controls belong inside the Sequence workflow structure under `src/windows/main/screens/sequence/` and related hooks/components.
- Shared frontend state derivation should continue to live in `src/services/state/` and supporting hooks, not in feature-local caches.
- No new top-level feature folders are needed; this story should extend the existing main-window shell, Sequence workflow, and backend UI seams.

### File Structure Requirements

- Primary backend active-flow seams:
  - `src-tauri/src/ui/mod.rs`
  - `src-tauri/src/ui/sequencer.rs`
  - `src-tauri/src/ui/screens/sequence.rs`
- Primary shared shell and workflow-local UI seams:
  - `src/windows/main/index.tsx`
  - `src/windows/main/screens/sequence/index.tsx`
  - `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview.tsx`
  - `src/windows/main/components/sequencer.tsx`
  - `src/windows/main/hooks/use-bottom-panel-state.ts`
- Primary score-interaction seam:
  - `src/windows/main/components/renderer.tsx`
- Primary shared-state and command seams:
  - `src/services/state/provider.tsx`
  - `src/services/actions.ts`
  - `src/services/types.ts`
- Likely regression seams:
  - `src/windows/main/components/sequencer.test.tsx`
  - `src/windows/main/screens/sequence/components/bottom-panel.test.tsx`
  - `src/windows/main/components/renderer.test.tsx`
  - `src/services/actions.test.ts`
- Likely out of scope unless verification proves otherwise:
  - persistence and migrations
  - renderer-native active-flow badges
  - new standalone piano-roll screens
  - large shell redesign beyond moving the flow selector out of shared chrome

### Testing Requirements

- Add tests proving the shared top bar no longer owns the active-flow selector and that the Sequence workflow exposes a local active-flow switcher instead.
- Add tests proving `use-bottom-panel-state` and Sequence workflow surfaces continue to derive one coherent active flow from `ui.flow_id`.
- Add tests proving score interaction and workflow-local flow switching remain coherent with transport synchronization and do not create contradictory state.
- Add tests proving single-flow scenarios still behave exactly as before after the flow selector moves.
- Extend renderer tests only where needed to prove deliberate score interaction can keep the authoritative active flow coherent without making passive reading alter it.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 2.2 already proved score selection can move between visible flows without frontend-owned disambiguation.
- Story 2.3 proved selection should only change through explicit source-of-truth updates, which is relevant here because active flow should not flap during passive score movement.
- Story 2.4 confirmed popover editing remains tied to the selected target's flow and should stay aligned with whichever flow is active across surfaces.
- The current epic implementation pattern has been verification-first: inspect the existing backend-owned contract, then make the smallest coherent change that keeps the model trustworthy.
- Story 1.5 is still the key UX warning: do not solve coherence by adding passive labels, extra bottom-bar flow context, or other visual noise that the product has already rejected.

### Git Intelligence Summary

- `ae3e788e fix: epic cleanup` confirms the plan was intentionally simplified, so this story should stay focused on the remaining cross-surface active-flow problem instead of reintroducing split scope.
- `7ee83c7c feat(2.4): verify multi-flow popover anchors` shows the recent pattern of tightening the backend-owned interaction contract with focused tests rather than broad rewrites.
- `928b0376 feat(2.3): simplify selection clearing for deletes` reinforces that explicit source-of-truth updates are preferred over generalized normalization passes.
- `ee4f6ef4 test(2.2): cover multi-flow selection consumers` confirms existing selection consumers and screen seams already have useful regression coverage points for this story.

### Project Context Reference

- `docs/project-context.md` defines the score as a natively rendered Rust surface with a transparent React overlay and treats `ui.flow_id` as an existing shared active-flow input for downstream consumers.
- `_bmad-output/planning-artifacts/architecture.md` requires active-flow semantics to remain distinct from visible-flow presentation while preserving Rust ownership and avoiding duplicate frontend score state.
- `_bmad-output/planning-artifacts/ux-design-specification.md` explicitly says the shared top bar should contain universal tools and information only, while workflow-local single-flow selectors belong inside the relevant workflow surface.
- `_bmad-output/planning-artifacts/epics.md` now places the remaining active-flow work entirely in this story after removing the weaker standalone visibility story.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 2, Story 2.5 acceptance criteria and updated story boundaries)
- Source: `_bmad-output/planning-artifacts/architecture.md` (active-flow semantics distinct from visible-flow presentation, Rust ownership, shell orchestration)
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` (shared top-bar universality, workflow-local flow selectors, score-first shell)
- Source: `_bmad-output/planning-artifacts/prd.md` (FR10, FR11, FR17, FR32, NFR8, AC5, AC6)
- Source: `docs/project-context.md` (native renderer ownership, shared active-flow consumption, command-boundary rules)
- Source: `_bmad-output/implementation-artifacts/2-2-select-score-content-in-either-visible-flow.md` (singular selection groundwork)
- Source: `_bmad-output/implementation-artifacts/2-3-preserve-stable-selection-while-one-visible-flow-updates.md` (explicit source-of-truth update precedent)
- Source: `_bmad-output/implementation-artifacts/2-4-open-contextual-popovers-within-the-multi-flow-score-surface.md` (popover continuity groundwork)
- Source: `src-tauri/src/ui/mod.rs` (`ui_flow_id_set`, transport sync, shared UI state)
- Source: `src-tauri/src/ui/sequencer.rs` (flow-aware selection commands)
- Source: `src-tauri/src/ui/screens/sequence.rs` (`show_active_only` and sequence-screen UI state)
- Source: `src/windows/main/index.tsx` (current shared top-bar flow selector)
- Source: `src/windows/main/components/renderer.tsx` (score interaction handoff)
- Source: `src/windows/main/components/sequencer.tsx` (single-flow sequence editing surface)
- Source: `src/windows/main/screens/sequence/components/bottom-panel.tsx` (workflow-local sequence controls)
- Source: `src/windows/main/screens/sequence/components/instrument-overview.tsx` (current active-flow-derived sequence overview)
- Source: `src/windows/main/hooks/use-bottom-panel-state.ts` (sequence flow derivation from `ui.flow_id`)
- Source: `src/services/state/provider.tsx` (active-flow fallback and transport synchronization)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Keep `ui.flow_id` as the only shared active-flow seam and update it inside the backend sequencer-selection commands so deliberate score-object selection stays Rust-owned and transport-synchronized.
- Remove the shared-shell flow selector and keep flow switching in workflow-local sequencing surfaces, reusing `use-bottom-panel-state` rather than introducing feature-local active-flow state.
- Cover the change with backend tests plus focused main-window, Sequence-toolbar, and hook regressions, then update architecture and agent docs to codify the shell rule.

### Completion Notes List

- Verified the existing active-flow seam before changes: `ui.flow_id` remained backend-owned in `src-tauri/src/ui/mod.rs`, transport tempo sync already followed it, and Sequence consumers already derived one concrete flow from that slot.
- Updated backend sequencer-selection commands in `src-tauri/src/ui/sequencer.rs` so deliberate score-object selection now also updates `ui.flow_id` and re-synchronizes transport without changing clear-selection or passive navigation behavior.
- Removed the workflow-specific flow selector from `src/windows/main/index.tsx` and kept explicit active-flow switching in workflow-local sequencing surfaces while preserving the shared top bar as universal chrome.
- Extended `use-bottom-panel-state` so Sequence-local flow options are derived from the authoritative flow order while safely ignoring stale flow IDs, keeping miniature piano-roll and instrument-overview consumers aligned to one active flow.
- Added frontend regressions for main-window selector placement, Sequence local switching, hook-derived flow options, and shared toolbar rendering, plus backend coverage proving sequencer selection updates the shared active flow.
- Followed up on Sequence-screen UX by moving the flow picker into the sticky left overview header in `src/windows/main/screens/sequence/components/instrument-overview.tsx`, matching the main-window tick-bar height to the selector there only, and keeping the control visible across Sequence tabs without adding a separate header row.
- Addressed code-review follow-ups by restoring stale `ui.flow_id` fallback handling in `src/windows/main/screens/sequence/components/instrument-overview.tsx` and adding focused regressions around the Sequence overview and Write bottom panel behavior.
- Updated `docs/project-context.md`, `docs/architecture.md`, and `.github/copilot-instructions.md` to codify `ui.flow_id` ownership and the universal-top-bar/workflow-local-selector rule.
- Validation passed: `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- Follow-up validation for the Sequence overview-header refinement passed: `npm run lint`, `npm run check-types`, and `npm run test:js`.

### File List

- `.github/copilot-instructions.md`
- `docs/architecture.md`
- `docs/project-context.md`
- `src-tauri/src/ui/sequencer.rs`
- `src/windows/main/index.tsx`
- `src/windows/main/index.test.tsx`
- `src/windows/main/components/sequencer.tsx`
- `src/windows/main/components/sequencer.test.tsx`
- `src/windows/main/components/tick-bar.tsx`
- `src/windows/main/components/tick-bar.test.tsx`
- `src/windows/main/hooks/use-bottom-panel-state.ts`
- `src/windows/main/hooks/use-bottom-panel-state.test.ts`
- `src/windows/main/screens/sequence/components/instrument-overview.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview.test.tsx`
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
- `src/windows/main/screens/sequence/components/bottom-panel.test.tsx`
- `src/windows/main/screens/sequence/index.test.tsx`
- `src/windows/main/screens/write/components/bottom-panel.tsx`
- `src/windows/main/screens/write/components/bottom-panel.test.tsx`
- `src/ui/select.tsx`
- `_bmad-output/implementation-artifacts/2-5-preserve-single-active-flow-behavior-across-score-and-sequencer-contexts.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-03-11: Created Story 2.5 context for cross-surface active-flow coherence, anchored to the existing Sequence workflow and workflow-local selector placement rules.
- 2026-03-11: Implemented shared active-flow handoff from score selection, moved explicit flow switching into workflow-local Sequence controls, added regressions, and updated architecture/agent guidance.
- 2026-03-11: Moved the Sequence-screen flow picker into the sticky left overview header, matched the main overview tick-bar height to the selector, and tightened select truncation/alignment without changing the panel layout.
- 2026-03-11: Fixed code-review follow-ups by restoring stale-flow fallback in the Sequence overview header and reconciling the story file list with actual git changes.

## Senior Developer Review (AI)

### Review Date

2026-03-11

### Reviewer

GPT-5.4

### Findings

- Fixed: `src/windows/main/screens/sequence/components/instrument-overview.tsx` had lost the stale-`ui.flow_id` fallback already present in `use-bottom-panel-state`, which could blank the overview despite valid flows existing.
- Fixed: The story file list claimed source-file changes that were not present in git; the file list now reflects the actual changed files.

### Validation

- Passed after fix: `npm run lint`
- Passed after fix: `npm run check-types`
- Passed after fix: targeted JS regressions for the Sequence overview and Write bottom panel
- Passed after fix: full JS suite

### Notes

- Atlassian MCP search was attempted during review but failed with `401 Unauthorized`, so no external Jira/Confluence references were added.
