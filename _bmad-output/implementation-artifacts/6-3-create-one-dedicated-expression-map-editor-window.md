# Story 6.3: Create One Dedicated Expression-Map Editor Window

Status: done

## Story

As a composer,
I want one dedicated editor for expression maps,
so that both instrument-driven fixes and library-driven authoring use the same durable workflow.

## Acceptance Criteria

1. Given I enter expression-map work from the Sequence menu library path or from the instrument-row action, when the relevant window is opened, then both paths lead into the same dedicated Expression Map workflow window, and that window owns the selected Expression Map asset itself.
2. Given expression-map work is a reusable playback-asset workflow, when I use the dedicated editor, then the workflow is framed in musical playback terms where appropriate and technical trigger details where necessary, and the interface remains aligned with Compose's existing design language.
3. Given expression-map editing is app-scoped, when the combined workflow window is used, then the left pane behaves as the durable CRUD and browse surface, the right pane behaves as the selected-map authoring surface, and the overall workflow remains distinct from score-context panels and plugin-owned parameter editing.

## Tasks / Subtasks

- [x] Task 1: Expand the 6.2 selected-map shell into one real combined Expression Map workflow window (AC: 1, 3)
  - [x] Reuse the existing app-scoped Expression Map window as the single workflow window launched from both the Sequence menu and the instrument-row action.
  - [x] Build the window as a split layout with a left library pane and a right selected-map editor pane.
  - [x] Ensure instrument-row launch opens the same window with the current map selected, while menu launch opens the same window and auto-selects the first available map when one exists.

- [x] Task 2: Make the left pane the only CRUD and browse surface for reusable maps (AC: 1, 3)
  - [x] Provide a searchable list of Expression Maps in the left pane.
  - [x] Support the intentionally simplified library workflow in the left pane: search, create, selection, and delete.
  - [x] Keep create and delete owned by the left library list; map renaming remains part of the right selected-map authoring surface.
  - [x] Warn before delete but allow it, with the established contract that affected route links later resolve truthfully to `--`.

- [x] Task 3: Build the right selected-map editor pane around full rule authoring with auto-save (AC: 1, 2, 3)
  - [x] Make the first editable field in the right pane the Expression Map name so the user can immediately see and change which map is being authored.
  - [x] Auto-save edits as the user works; do not introduce a separate manual save/apply workflow.
  - [x] Support full editing of the current MVP rule model rather than metadata-only editing.
  - [x] Keep authoring in the same app-scoped window rather than spawning one editor window per map.

- [x] Task 4: Define and surface rule evaluation semantics clearly and simply in the editor (AC: 2, 3)
  - [x] Rules are evaluated top-down and stop at the first match.
  - [x] Support drag-and-drop reordering using the repo's existing reorder interaction language.
  - [x] Allow overlapping or shadowing rules in MVP without extra smart warnings.
  - [x] Keep the editor scoped to authored rules only; plugin-default fallback behavior remains implicit rather than rendered as a fixed row in the 6.3 window.

- [x] Task 5: Support the agreed MVP rule model in a way that stays understandable (AC: 2, 3)
  - [x] Conditions use logical AND within a rule.
  - [x] Support articulation conditions as single primitive rows with `==` and `!=` comparators.
  - [x] Support written-note-length conditions as single primitive rows with `<`, `<=`, `==`, `!=`, `>=`, and `>` comparators over one note duration value.
  - [x] Express compound articulation or note-length intent through multiple condition rows rather than compound controls inside one row.
  - [x] Support multiple actions per rule, while keeping the editor presentation clear enough that users can still reason about what the rule does.
  - [x] Support the agreed MVP action set for authored rules: keyswitch note, CC message, note-length transformation, and no-op.

- [x] Task 6: Prove the one-window workflow, rule-order semantics, and documentation boundaries (AC: 1, 2, 3)
  - [x] Add frontend tests for single-window launch and reuse from both the Sequence menu and the instrument-row action, plus selection handoff into the same workflow window.
  - [x] Add frontend tests for left-pane CRUD affordances, right-pane rule reorder behavior, and auto-save-driven editing behavior.
  - [x] Add or extend backend/shared-contract tests as needed to prove rule order persistence, fallback preservation, and editor-safe map updates.
  - [x] Update docs to record the one-window split-pane ownership model, first-match-wins rule semantics, left-pane CRUD boundaries, and auto-save authoring behavior.

## Dev Notes

### Story Intent

- Story 6.3 is where Expression Maps become a real reusable workflow rather than a hidden model plus row linkage.
- The critical scope line is that CRUD and browsing live in the left pane of one app-scoped workflow window, while actual map authoring lives in the right editor pane of that same window.
- The current shipped 6.3 workflow intentionally narrows the left pane to lightweight library actions: search, create, selection, and delete. Rename remains part of the right selected-map authoring surface.
- This is intentionally not a plugin settings UI. It is Compose-owned authoring of playback semantics.

### User Model Clarified

- Users should think of the left pane as a catalogue of reusable Expression Maps and the right pane as the focused workspace for one map.
- Users should think of the left pane as a lightweight catalogue and launch surface, not a full asset-management console.
- The Sequence menu launches the combined workflow window. The instrument-row action opens that same window with the currently assigned map selected.
- Maps are generic. The editor should not ask the user what plugin the map belongs to, even if the user is practically building it for one plugin setup.
- Rule order matters. Rules closest to the top take precedence, evaluation stops at the first match, and the user is responsible for ordering more specific rules above broader ones.
- The editor should use musical language where that is the user's mental model, such as articulations, but technical trigger language where that is inherently required, such as keyswitch note or CC message.
- The current editor models one primitive condition per row. Compound intent is authored by adding more rows instead of opening compound multiselect or range controls inside a single row.

### Previous Story Intelligence

- Story 6.1 established the durable model, route-address-aware linkage seam, and app-scoped library ownership contract.
- Story 6.2 intentionally kept the score-context row narrow and added only a minimal selected-map shell, explicitly deferring full editing and library workflow to Story 6.3.
- The repo already uses dedicated app-scoped windows such as Plugin Manager for reusable resource workflows, and uses compact row controls in Sequence for score-context relationship work. 6.3 should preserve that separation instead of collapsing the library into score context.

### This Story Owns

- One dedicated `Expression Map` workflow window launched from the Sequence menu or from the instrument row.
- Search, create, selection, and delete in the left pane.
- Full MVP rule authoring in the right pane with auto-save.
- Top-down first-match-wins rule evaluation with drag reorder and authored-rule editing only.

### This Story Does Not Own

- Audible playback behavior changes. Story 6.4 owns proving that linked rules affect playback.
- Plugin-specific ownership fields, plugin autodetection, or default map assignment heuristics.
- Smart analysis or warning systems for shadowed or overlapping rules.
- Broad trigger expansion beyond the currently agreed MVP authored actions.
- Score-context CRUD surfaces. The row still owns linkage only, not library management.

### Current Relevant Seams

- `src/services/actions.ts`
  - Existing app-scoped window launch patterns to reuse for both library and editor windows.
- `src/services/state/provider.tsx`
  - Existing menu event listening pattern used for Plugin Manager and other app-scoped windows.
- `src-tauri/src/lib.rs`
  - Existing Sequence menu wiring pattern that should add the canonical `Expression Map...` entry.
- `src/windows/plugin-manager/`
  - Current reference for an app-scoped searchable library window.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing score-context cog launch point for the selected-map editor.
- `src/windows/main/screens/setup/components/player-panel.tsx`
  - Existing SortableJS reorder pattern that should inform rule drag-and-drop behavior.
- `src/services/types.ts`
  - Shared frontend mirror for any new Expression Map window and rule payload types.
- `src-tauri/src/score/playback.rs`
  - Persistence and rule-order truth must stay aligned with backend ownership.

### Implementation Guidance

- Treat the workflow as one split-pane window with separate responsibilities by pane, not as two app windows or two tabs.
- The left pane should stay list-centric and operational, but intentionally lightweight for the current shipped scope: search, selection, create, and delete.
- The right pane should stay focused on one map and immediately expose its name as the first input so the user always knows which map is being edited.
- Auto-save as edits occur. Do not create a manual save ceremony that complicates multi-window editing.
- Multiple editor windows are allowed, one per map. Reuse/focus by map identity rather than forcing one global editor window.
- Keep rule authoring visually simple even though actions are technical. A user should be able to scan a rule and understand both its conditions and its resulting playback actions.
- Support multiple actions in one rule, but present them clearly so the user does not lose track of execution order within the matching rule.
- Keep plugin-default fallback behavior implicit in this workflow rather than rendering a fixed non-editable fallback row.

### Risks And Regression Traps

- The largest scope trap is letting the right editor pane absorb library CRUD actions too aggressively. Keep CRUD in the left pane to preserve the split ownership model.
- Another trap is trying to make the library window double as score-context assignment UI. That would violate the ownership split already established in 6.2.
- Rule stacking is intentionally out of scope. Do not drift from first-match-wins into layered action resolution.
- Auto-save plus multi-window editing can create state-coherence bugs if map identity and update propagation are weak. Treat map ID and current revision truth carefully.
- The plugin-default fallback can easily become misleading if it is over-explained as editable UI. Keep that fallback implicit in the shipped 6.3 workflow.
- It is easy to overbuild warnings around shadowed rules or overlaps. MVP should trust the user and keep the model explainable, not paternalistic.

### Testing Requirements

- Add frontend tests for:
  - Sequence menu launch of the combined `Expression Map` workflow window;
  - search and selection behavior in the left pane;
  - create and delete affordances in the left pane, including native confirmation before delete;
  - opening the same workflow window from both the menu and the Sequence row action;
  - selection handoff from the left pane into the right editor pane;
  - auto-save behavior as editor fields change;
  - drag-and-drop reorder persistence for rules;
  - rule-row authoring with the current primitive condition and multi-action model.
- Add backend or shared-contract tests for:
  - rule order persistence;
  - first-match-wins ordering semantics metadata;
  - multi-action rule serialization;
  - safe delete behavior where later route linkage resolves to `--`.
- Recommended validation commands once implemented:
  - `npm run test:js`
  - `npm run check-types`
  - `npm run lint`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

### Documentation Impact

- Update docs when the real Expression Map workflow lands.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
- Document explicitly that:
  - the Sequence menu launches the combined `Expression Map` workflow window;
  - the left pane and right pane divide library and editor responsibilities inside one app window;
  - the left pane owns search, create, selection, and delete, while full map authoring including rename lives in the right pane;
  - rules evaluate top-down and stop at first match;
  - the current editor models one primitive condition per row and expresses compound intent through multiple rows;
  - authoring auto-saves as the user edits.

### Architecture Compliance

- The app-scoped Expression Map window remains the owner of the reusable workflow.
- Score-context UI still owns only linkage and launch affordances, not map CRUD.
- Rust remains the owner of durable map truth, rule order, and serialized action/condition data.
- Frontend windows project and mutate backend-owned state through existing action/state patterns.
- Keep the editor distinct from plugin-owned parameter editing even when actions express technical trigger data.

### Library And Framework Requirements

- Stay on the repo baselines already established in the project context: Rust 2021, Tauri 2.10.1, React 19.2.4, and TypeScript 5.9.3.
- Reuse existing `WebviewWindow` patterns, Sequence menu event wiring, searchable list UI patterns, and SortableJS reorder seams already present in the repo.

### File Structure Notes

- Add the new app windows under `src/windows/` following the existing multi-window boot and index pattern.
- Window launch helpers belong in `src/services/actions.ts` and related tests.
- Sequence menu event wiring belongs in the existing app-state provider and backend menu setup seams.
- Library and editor UI tests should remain close to their owning windows/components rather than creating a separate expression-map-only harness.

### Git Intelligence Summary

- Existing app-scoped workflows such as Plugin Manager already establish the menu-launch and searchable-library pattern the left pane should follow.
- Existing SortableJS usage in setup panels provides a repo-native reorder approach; 6.3 should reuse that interaction language for rule ordering.

### Latest Technical Information

- No external dependency change is required for 6.3. The important technical constraints are local: preserve the app-scoped ownership split, keep rule ordering deterministic, and keep multi-window editing coherent under auto-save.
- The repo already demonstrates narrow score-context launch points and searchable app-scoped manager windows. 6.3 should combine those patterns into one split-pane Expression Map workflow window rather than proliferating more windows.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 6 and Story 6.3 baseline direction.
- `_bmad-output/implementation-artifacts/6-1-model-expression-maps-as-durable-playback-assets.md`
  - Durable model, linkage, and app-scoped ownership foundation.
- `_bmad-output/implementation-artifacts/6-2-show-expression-map-linkage-in-score-context.md`
  - Row-level linkage UI and minimal selected-map shell handoff.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Expression Map app-scoped ownership and panel-versus-window rules.
- `src/services/actions.ts`
  - Existing app-window launch pattern.
- `src/services/state/provider.tsx`
  - Existing menu event wiring pattern.
- `src-tauri/src/lib.rs`
  - Existing Sequence menu definition and event emission.
- `src/windows/plugin-manager/index.tsx`
  - Existing searchable library-window reference.
- `src/windows/main/screens/setup/components/player-panel.tsx`
  - Existing SortableJS reorder interaction pattern.
- `/memories/repo/plugin-manager-window-scope.md`
  - App-scoped manager-window scope discipline.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-05: Story prepared after reviewing Story 6.1, Story 6.2, Epic 6 planning, current app-window/menu seams, Plugin Manager scope memory, and existing SortableJS reorder usage.
- 2026-04-05: User clarification updated the 6.3 workflow to one combined Expression Map window with a left library pane and right editor pane, while preserving the existing CRUD, authoring, auto-save, and first-match-wins rule semantics.
- 2026-04-05: Implemented the split-pane Expression Map workflow window, expanded the reusable rule model for multi-articulation and note-length comparators, and added backend create/duplicate/delete/import/export seams for the app-scoped reusable map library.
- 2026-04-05: Validation completed with `npm run check-types`, `npm run lint`, `npm run test:js`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`.
- 2026-04-05: Post-review bug fix released the expression-map library mutex before emitting refreshed general state so create/duplicate/delete/import and targeted expression-map updates no longer hang after successful persistence.
- 2026-04-05: Follow-up editor refinements added persisted expression-map metadata fields (`creator`, `version`), replaced the first editor section with structured Expression Map Data inputs, and added explicit input aria-label support for stable accessibility and test coverage.
- 2026-04-06: Scope was intentionally narrowed after implementation review: the shipped left pane remains search/create/delete plus selection, rename stays in the right editor pane, plugin-default fallback remains implicit instead of a rendered fixed row, and rule authoring now uses primitive one-row conditions rather than compound multiselect or range controls.
- 2026-04-06: Follow-up architecture cleanup gave rules, conditions, and actions backend-owned stable ids, moved nested add/update/remove and type-reset operations onto dedicated Rust commands keyed by identity, and kept React as a thin control layer instead of rebuilding nested arrays for whole-map upserts.
- 2026-04-06: Follow-up architecture cleanup removed the remaining wide map upsert and rekey seams from the editor, switched expression-map creation and duplication to backend-generated UUID ids, and made the map ID field read-only in the window.

### Completion Notes List

- Story context created and sprint status updated to ready-for-dev.
- Story 6.3 now owns the actual reusable Expression Map workflow through one app-scoped split-pane window with library and editor responsibilities separated by pane.
- Rule semantics were tightened before writing: top-down first-match-wins, logical AND conditions, drag reorder, and auto-save editing.
- Story 6.4 remains the first story that should make the authored rules audibly affect playback.
- Implemented a dedicated left-pane library workflow for search, selection, create, rename, duplicate, delete, import, single export, multi-select export, and select-all while keeping score-context rows limited to linkage and launch.
- The shipped left pane is intentionally simpler than the earlier draft story: search, selection, create, and delete only. Rename is owned by the right pane as part of selected-map authoring.
- Expanded and then intentionally simplified the reusable Expression Map rule model into primitive one-row conditions with equality and relational comparators, while preserving multiple ordered actions per rule.
- Added frontend coverage for one-window launch reuse, selection handoff, CRUD affordances, auto-save, and drag reorder persistence, plus backend coverage for safe delete behavior.
- Added native confirmation before left-pane delete so map removal stays deliberate while preserving the later `--` route-resolution contract.
- Added persisted expression-map metadata editing for name, id, creator, and version in the first editor section.
- Expression-map IDs are now backend-generated UUIDs shown as read-only reference data in the editor rather than user-authored or rekeyable metadata.
- Retro note: the drag-and-drop complexity came from treating one bug as a Sortable problem when it was really three concerns mixed together: React list identity, editor selection ownership, and drag-clone styling. Complexity crept in because early fixes chased drag symptoms first, while the important invariant that the rendered rule order must visibly change and preserve the selected rule was not asserted directly enough.
- Follow-up cleanup removed the last index-based nested authoring seam in the editor: rules, conditions, and actions now mutate through backend-owned ids and targeted Rust commands instead of whole-map nested array replacement from React.

### File List

- `_bmad-output/implementation-artifacts/6-3-create-one-dedicated-expression-map-editor-window.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `docs/data-models.md`
- `src-tauri/Cargo.toml`
- `src/services/actions.test.ts`
- `src/services/actions.ts`
- `src/services/state/provider.test.tsx`
- `src/services/types.ts`
- `src/ui/input/index.tsx`
- `src/ui/input/number-input.test.tsx`
- `src/ui/input/number-input.tsx`
- `src/ui/input/text-input.tsx`
- `src/windows/expression-map/expression-map-editor-pane.tsx`
- `src/windows/expression-map/expression-map-editor-pane.css`
- `src/windows/expression-map/expression-map-library-pane.tsx`
- `src/windows/expression-map/expression-map-rule-card.tsx`
- `src/windows/expression-map/index.test.tsx`
- `src/windows/expression-map/index.tsx`
- `src/windows/expression-map/model.ts`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
- `src-tauri/src/lib.rs`
- `src-tauri/src/playback/expression_maps.rs`

### Change Log

- 2026-04-05: Implemented the dedicated Expression Map split-pane workflow window, expanded the reusable rule model and backend library commands, updated row launch wording, and documented the shipped one-window authoring workflow.
- 2026-04-05: Fixed a backend deadlock in expression-map mutation commands by emitting refreshed state only after releasing the library lock.
- 2026-04-05: Added persisted expression-map metadata fields and reworked the first editor section into settings-style Expression Map Data inputs with accessible names.
- 2026-04-05: Moved the default rule into the Rules section, removed the separate fallback section, and dropped metadata compatibility defaults for omitted fields.
- 2026-04-06: Rebased Story 6.3 documentation to the intentionally simplified shipped scope: left pane search/create/delete only, rename in the right pane, native confirmation before delete, implicit plugin-default fallback, and primitive one-row condition authoring.
- 2026-04-06: Review-fix pass simplified the expression-map editor update flow away from the optimistic library cache, reset menu launches back to the default library selection, added numeric default fallback handling, preserved selected-rule intent across drag reorder, corrected the Tauri confirm tests, and refreshed dependent test fixtures for the expanded expression-map metadata contract.
- 2026-04-06: Follow-up DnD ghost fix gave the rule list a dedicated ghost style and moved the Sortable fallback clone onto `body` so the drag ghost tracks the pointer correctly inside the scrolling expression-map editor pane.
- 2026-04-06: Follow-up DnD alignment removed the custom rule-list fallback-clone path and matched the instrument-panel Sortable configuration so the expression-map rule drag now behaves the same way as the existing setup-panel drag flow.
- 2026-04-06: Follow-up DnD drop-commit fix now commits rule reorders only after drop, hides the fallback drag clone explicitly, and lets the local reordered map win immediately so the list order updates without waiting on the backend state echo.
- 2026-04-06: Follow-up DnD event fix records the last sortable move during drag and commits that move on drop, avoiding reliance on `onEnd` index payloads alone when fallback dragging is active.
- 2026-04-06: Follow-up DnD identity fix replaced index-based React rule keys with stable per-rule keys and restored the normal sortable sort flow so reordered rules render in their new positions instead of reconciling by array slot.
- 2026-04-06: Follow-up DnD visual alignment removed the hidden fallback-clone styling and gave rule rows the same stable border-box pattern as the player list so the visible drag ghost matches existing setup behavior and the dashed placeholder no longer grows on drag start.
- 2026-04-06: Follow-up DnD positioning fix moved the rule-list fallback clone onto `body` to escape the editor pane scroll container, restored left-border-only row styling, and kept the dashed ghost size stable with `box-sizing: border-box`.
- 2026-04-06: Follow-up DnD clone-motion fix removed the generic row transition from rule items and added a dedicated visible fallback-clone class with `transition: none`, `pointer-events: none`, and a high z-index so the dragged rule can track pointer transforms directly instead of lagging behind them.
- 2026-04-06: Follow-up reorder cleanup extracted the selected-index remap into one helper and removed the extra moved-rule branch so the rule-list drag handler now states its intent directly instead of carrying redundant selection bookkeeping.
- 2026-04-06: Follow-up reorder simplification removed the ref-based latest-value plumbing from the rule-list drag path, switched persistence back to a single `onEnd` save, and preserved both implicit default selection and explicit rule selection through one current-selection helper.
- 2026-04-06: Follow-up rule-identity fix added persisted expression-map rule ids, moved rule-id generation fully into the Rust backend, switched the editor to selection-by-rule-id, and documented rule ids as internal reusable-map model identity rather than score `UniqueId` ownership.

### Review Findings

- [x] [Review][Patch] Menu launch reuses a stale selected map instead of restoring the menu-owned default selection [src/services/actions.ts:66]
- [x] [Review][Patch] Early local-cache handling around create and editable IDs was replaced by backend-generated read-only UUID ids plus targeted metadata setters, removing the stale rekey path entirely [src/windows/expression-map/index.tsx:119]
- [x] [Review][Patch] Clearing numeric inputs can propagate `undefined`/`NaN` into persisted expression-map fields [src/ui/input/number-input.tsx:60]
- [x] [Review][Patch] Rule selection is tracked by array index, so drag reorder can retarget subsequent edits to the wrong rule [src/windows/expression-map/expression-map-editor-pane.tsx:36]
- [x] [Review][Patch] Delete-confirmation tests still mock `window.confirm`, so the shipped Tauri confirm path is not covered and the current test file fails [src/windows/expression-map/index.test.tsx:233]
- [x] [Review][Patch] Expression-map schema docs still omit `creator` and `version` even though the staged contract now treats both metadata fields as required [docs/data-models.md:448]
- [x] [Review][Patch] The editor preferred a local expression-map draft in ways that could outlive the backend echo, so the window now follows shared state directly and only keeps a narrow created-map fallback before the store catches up [src/windows/expression-map/index.tsx:24]
- [x] [Review][Patch] The editor carried edit-time and reorder-time optimistic state even though backend updates are effectively immediate, so those paths were removed in favor of backend-owned updates only [src/windows/expression-map/index.tsx:126]
- [x] [Review][Defer] Legacy expression-map library payloads that lack the new nested ids or metadata fields now fail deserialization and are silently replaced with an empty library instead of being migrated [src-tauri/src/playback/expression_maps.rs:194] — deferred: no legacy expression maps exist while the feature remains development-only
- [x] [Review][Patch] The story artifact still contained visible-fallback and import/export claims that contradicted the narrowed shipped 6.3 scope [/_bmad-output/implementation-artifacts/6-3-create-one-dedicated-expression-map-editor-window.md:52]
