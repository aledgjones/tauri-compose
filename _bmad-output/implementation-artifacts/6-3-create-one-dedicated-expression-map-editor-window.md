# Story 6.3: Create One Dedicated Expression-Map Editor Window

Status: ready-for-dev

## Story

As a composer,
I want one dedicated editor for expression maps,
so that both instrument-driven fixes and library-driven authoring use the same durable workflow.

## Acceptance Criteria

1. Given I enter expression-map work from the Sequence menu library path or from the instrument-row cog, when the relevant window is opened, then both paths lead into the same dedicated Expression Map editor workflow, and the editor owns the selected Expression Map asset itself.
2. Given expression-map work is a reusable playback-asset workflow, when I use the dedicated editor, then the workflow is framed in musical playback terms where appropriate and technical trigger details where necessary, and the interface remains aligned with Compose's existing design language.
3. Given expression-map editing is app-scoped, when the library and editor windows are used, then the library behaves as the durable CRUD and browse surface, the editor behaves as the selected-map authoring surface, and both remain distinct from score-context panels and plugin-owned parameter editing.

## Tasks / Subtasks

- [ ] Task 1: Expand the 6.2 selected-map shell into a real Expression Map Editor window while adding a separate Expression Map Library window (AC: 1, 3)
  - [ ] Add a dedicated `Expression Map Library` app window launched from the Sequence menu and reused on repeated opens.
  - [ ] Expand the minimal 6.2 selected-map shell into a real `Expression Map Editor` app window for map authoring.
  - [ ] Keep the library and editor as two separate app windows, not a split-pane single surface.
  - [ ] Ensure instrument-row cog launch opens the editor window for the currently assigned map, while menu launch opens the library window with no forced map selection.

- [ ] Task 2: Make the library window the only CRUD and browse surface for reusable maps (AC: 1, 3)
  - [ ] Provide a searchable list of Expression Maps in the library window.
  - [ ] Support create, rename, duplicate, delete, import, export, multi-select export, and select-all from the library window only.
  - [ ] Keep create, duplicate, rename, and delete actions owned by the library list rather than duplicating them inside the editor.
  - [ ] Warn before delete but allow it, with the established contract that affected route links later resolve truthfully to `--`.

- [ ] Task 3: Build the selected-map editor around full rule authoring with auto-save (AC: 1, 2, 3)
  - [ ] Make the first editable field the Expression Map name so the user can immediately see and change which map is being authored.
  - [ ] Auto-save edits as the user works; do not introduce a separate manual save/apply workflow.
  - [ ] Support full editing of the current MVP rule model rather than metadata-only editing.
  - [ ] Allow multiple editor windows at once, one per map, to reinforce that maps are distinct reusable assets.

- [ ] Task 4: Define and surface rule evaluation semantics clearly and simply in the editor (AC: 2, 3)
  - [ ] Rules are evaluated top-down and stop at the first match.
  - [ ] Support drag-and-drop reordering using the repo's existing reorder interaction language.
  - [ ] Allow overlapping or shadowing rules in MVP without extra smart warnings.
  - [ ] Include a visible default fallback rule fixed at the end of the list, shown in a disabled/non-editable state, representing the current plugin default playback path.
  - [ ] Preserve that fallback rule even if all user-authored rules are removed.

- [ ] Task 5: Support the agreed MVP rule model in a way that stays understandable (AC: 2, 3)
  - [ ] Conditions use logical AND within a rule.
  - [ ] Support articulation conditions with multi-select because score articulations can combine.
  - [ ] Support note-length conditions with shorter-than, longer-than, and range forms.
  - [ ] Support multiple actions per rule, while keeping the editor presentation clear enough that users can still reason about what the rule does.
  - [ ] Support the agreed MVP action set for authored rules: keyswitch note, CC message, note-length transformation, and no-op.

- [ ] Task 6: Prove the two-window workflow, rule-order semantics, and documentation boundaries (AC: 1, 2, 3)
  - [ ] Add frontend tests for library window launch and reuse, editor window launch from both library and instrument-row paths, and multi-window-per-map editor behavior.
  - [ ] Add frontend tests for CRUD affordances living in the library only, rule reorder behavior, visible fixed fallback rule, and auto-save-driven editing behavior.
  - [ ] Add or extend backend/shared-contract tests as needed to prove rule order persistence, fallback preservation, and editor-safe map updates.
  - [ ] Update docs to record the two-window ownership model, first-match-wins rule semantics, library-only CRUD boundaries, and auto-save authoring behavior.

## Dev Notes

### Story Intent

- Story 6.3 is where Expression Maps become a real reusable workflow rather than a hidden model plus row linkage.
- The critical scope line is that CRUD and browsing live in one app-scoped library window, while actual map authoring lives in a separate dedicated editor window.
- This is intentionally not a plugin settings UI. It is Compose-owned authoring of playback semantics.

### User Model Clarified

- Users should think of the library as a catalogue of reusable Expression Maps and the editor as the focused workspace for one map.
- The Sequence menu launches the library window. The instrument-row cog opens the editor directly for the currently assigned map.
- Maps are generic. The editor should not ask the user what plugin the map belongs to, even if the user is practically building it for one plugin setup.
- Rule order matters. Rules closest to the top take precedence, evaluation stops at the first match, and the user is responsible for ordering more specific rules above broader ones.
- The default fallback rule must be visible so the model remains honest, but it should stay fixed and non-editable to avoid confusing the MVP contract.
- The editor should use musical language where that is the user's mental model, such as articulations, but technical trigger language where that is inherently required, such as keyswitch note or CC message.

### Previous Story Intelligence

- Story 6.1 established the durable model, route-address-aware linkage seam, and app-scoped library ownership contract.
- Story 6.2 intentionally kept the score-context row narrow and added only a minimal selected-map shell, explicitly deferring full editing and library workflow to Story 6.3.
- The repo already uses dedicated app-scoped windows such as Plugin Manager for reusable resource workflows, and uses compact row controls in Sequence for score-context relationship work. 6.3 should preserve that separation instead of collapsing the library into score context.

### This Story Owns

- A dedicated `Expression Map Library` window launched from the Sequence menu.
- A dedicated `Expression Map Editor` window opened from library selection or instrument-row cog.
- CRUD, import/export, search, multi-select export, and select-all in the library.
- Full MVP rule authoring in the editor with auto-save.
- Top-down first-match-wins rule evaluation with drag reorder and fixed fallback rule semantics.

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
  - Existing Sequence menu wiring pattern that should add the canonical `Expression Map Library...` entry.
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

- Treat library and editor as separate windows with separate responsibilities, not two tabs of the same screen.
- The library window should stay list-centric and operational: search, selection, CRUD, import/export, and launch into editor.
- The editor should stay focused on one map and immediately expose its name as the first input so the user always knows which map is being edited.
- Auto-save as edits occur. Do not create a manual save ceremony that complicates multi-window editing.
- Multiple editor windows are allowed, one per map. Reuse/focus by map identity rather than forcing one global editor window.
- Keep rule authoring visually simple even though actions are technical. A user should be able to scan a rule and understand both its conditions and its resulting playback actions.
- Support multiple actions in one rule, but present them clearly so the user does not lose track of execution order within the matching rule.
- The fixed fallback rule should be visually distinct and anchored last. It represents plugin default playback when no authored rule matches.

### Risks And Regression Traps

- The largest scope trap is letting the editor absorb library CRUD actions too aggressively. Keep CRUD in the library to preserve the two-window model.
- Another trap is trying to make the library window double as score-context assignment UI. That would violate the ownership split already established in 6.2.
- Rule stacking is intentionally out of scope. Do not drift from first-match-wins into layered action resolution.
- Auto-save plus multi-window editing can create state-coherence bugs if map identity and update propagation are weak. Treat map ID and current revision truth carefully.
- The fallback rule can easily become misleading if it appears editable. Keep it visible, fixed last, and non-editable.
- It is easy to overbuild warnings around shadowed rules or overlaps. MVP should trust the user and keep the model explainable, not paternalistic.

### Testing Requirements

- Add frontend tests for:
  - Sequence menu launch of the `Expression Map Library` window;
  - search, selection, multi-select export, and select-all behavior in the library;
  - create, rename, duplicate, delete, import, and export affordances in the library;
  - opening editor windows from both the library and the Sequence row cog;
  - multiple editor windows existing at once for different maps;
  - auto-save behavior as editor fields change;
  - drag-and-drop reorder persistence for rules;
  - visible fixed fallback rule that remains last and non-editable.
- Add backend or shared-contract tests for:
  - rule order persistence;
  - first-match-wins ordering semantics metadata;
  - fallback rule preservation;
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
  - the Sequence menu launches `Expression Map Library`;
  - the library and editor are separate app windows;
  - CRUD lives in the library, while full map authoring lives in the editor;
  - rules evaluate top-down and stop at first match;
  - the fallback rule is visible, fixed last, and non-editable;
  - authoring auto-saves as the user edits.

### Architecture Compliance

- App-scoped windows remain the owners of reusable Expression Map workflows.
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

- Existing app-scoped workflows such as Plugin Manager already establish the menu-launch and searchable-library pattern 6.3 should follow.
- Existing SortableJS usage in setup panels provides a repo-native reorder approach; 6.3 should reuse that interaction language for rule ordering.

### Latest Technical Information

- No external dependency change is required for 6.3. The important technical constraints are local: preserve the app-scoped ownership split, keep rule ordering deterministic, and keep multi-window editing coherent under auto-save.
- The repo already demonstrates both dedicated app-window management and narrow score-context launch points. 6.3 should build on that instead of inventing a new hybrid workflow.

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
- 2026-04-05: User clarification locked the 6.3 workflow: separate library and editor windows, library-only CRUD, editor-first map authoring with auto-save, first-match-wins rule ordering, fixed fallback rule, and drag-and-drop reorder.

### Completion Notes List

- Story context created and sprint status updated to ready-for-dev.
- Story 6.3 now owns the actual reusable Expression Map workflow through two app-scoped windows: a library and a dedicated editor.
- Rule semantics were tightened before writing: top-down first-match-wins, logical AND conditions, drag reorder, visible fixed fallback rule, and auto-save editing.
- Story 6.4 remains the first story that should make the authored rules audibly affect playback.

### File List

- `_bmad-output/implementation-artifacts/6-3-create-one-dedicated-expression-map-editor-window.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
