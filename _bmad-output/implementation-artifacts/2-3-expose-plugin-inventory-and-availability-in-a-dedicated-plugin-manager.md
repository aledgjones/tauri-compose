# Story 2.3: Expose Plugin Inventory and Availability in a Dedicated Plugin Manager

Status: review

## Story

As a maintainer,
I want a dedicated Plugin Manager window that shows the discovered plugin library and availability state,
so that I can inspect what Compose can use and what needs attention.

## Acceptance Criteria

1. Given Compose has a discovered or cached plugin inventory, when the user opens the Plugin Manager, then they can see the plugin inventory in a dedicated app-scoped window and that window is clearly separate from score-context assignment surfaces.
2. Given some plugins may be usable while others are unavailable or problematic, when the inventory is displayed, then the Plugin Manager communicates plugin availability state clearly and status is not conveyed by color alone.
3. Given the Plugin Manager is an app-scoped surface, when the user reaches it from canonical app entry points or contextual launch paths, then the experience remains consistent with the project's panel-versus-window scope model and it uses the existing Compose design system.

## Tasks / Subtasks

- [x] Task 1: Add the dedicated Plugin Manager window scaffold and canonical Sequence-menu launch path (AC: 1, 3)
  - [x] Add a dedicated window entry point for Plugin Manager in the existing multi-window Vite/Tauri setup instead of overloading score-context panels.
  - [x] Add the canonical Plugin Manager Sequence-menu item in `src-tauri/src/lib.rs` and route it through the existing menu-event pattern used by Preferences.
  - [x] Add the corresponding frontend listener and window-open flow using the established `WebviewWindow` pattern.
- [x] Task 2: Render Rust-owned plugin inventory truth in the dedicated window without creating a second frontend domain model (AC: 1, 2)
  - [x] Reuse the existing `PluginInventoryState` and `soloc://state/plugin_inventory` provider flow rather than fetching or reconstructing separate manager-local truth.
  - [x] Reuse shared inventory presentation logic from the current Sequence `Plugins` surface where practical, but converge on one stable browse presentation instead of keeping parallel views of the same inventory data.
  - [x] Keep plugin inventory browsing app-scoped only; do not turn this story into score assignment, picker behavior, or plugin-editor hosting.
- [x] Task 3: Communicate availability accessibly and reuse only the inventory controls that fit this story's scope (AC: 2, 3)
  - [x] Reuse the current search and `Available only` filter in the dedicated window because they already support inventory browsing.
  - [x] Ensure availability is legible through text and/or iconography, not color alone, while preserving the current Compose visual language.
  - [x] Do not add broader inventory-management features such as hiding, favoriting, or assignment-specific controls unless they are already required by a later story.
- [x] Task 4: Preserve the score-context versus app-scoped UX boundary for later stories (AC: 1, 3)
  - [x] Keep the Sequence bottom-panel inventory surface as a temporary contextual confirmation/browsing seam until later score-context assignment stories refine that workflow.
  - [x] Explicitly defer contextual launch into the Plugin Manager to later score-context stories that own recovery and assignment UX, rather than forcing it into this story.
  - [x] Do not require users to manage the full Plugin Manager window in order to complete later score-context assignment work.
- [x] Task 5: Add focused validation and documentation updates for the new app-scoped surface (AC: 1, 2, 3)
  - [x] Add frontend tests for the new Plugin Manager window rendering, inventory-state consumption, menu launch handling, and accessible availability labeling.
  - [x] Update developer-facing docs if the canonical app-scoped Plugin Manager entry point or inventory-surface scope model is described differently after implementation.
  - [x] Validate the story with the project-standard commands relevant to touched areas.

## Dev Notes

### Story Intent

- Story 2.1 established backend-owned discovery and machine-local plugin inventory persistence.
- Story 2.2 established cache-first startup inventory visibility through the existing provider/event path.
- Story 2.3 is the first dedicated app-scoped inventory UI story. Its job is to give that already-existing inventory truth a proper window-level home.
- This story should make the Plugin Manager a real app-scoped destination without pulling score assignment, reassignment, or recovery workflows out of their later score-context stories.

### User Decisions Captured

- For Story 2.3, implement the canonical Sequence-menu entry now.
- Do not require the contextual launch path in this story. Preserve that as later work so the full score-context UX can be designed where assignment and recovery are actually owned.
- Reuse the current search and `Available only` filter in the first Plugin Manager window because they already support inventory browsing.
- Do not expand this story into broader organisation or management controls just because the dedicated window exists. If a richer control is clearly owned by later work, leave it for that follow-up.

### Scope Boundaries

- In scope:
  - A dedicated Plugin Manager window for browsing the discovered or cached inventory.
  - A canonical Sequence-menu launch path for that window.
  - Reuse of existing inventory state, search, and availability filtering where it helps the dedicated manager browse the inventory.
  - Clear accessible availability communication for `available` versus `unavailable` entries.
- Out of scope:
  - Score-context plugin assignment, reassignment, or readiness UX. Epic 3 owns that work.
  - Contextual launch into Plugin Manager from score surfaces. Keep the requirement visible for later stories, but do not force it into 2.3.
  - Manual full rescan UX. Story 2.4 owns that workflow.
  - Native plugin editor hosting or sampler/plugin-settings behavior.
  - New plugin-inventory domain truth in React, frontend caching, or `.soloc` persistence changes.
  - Broader inventory organisation features such as favorites, hide rules, or categorisation beyond the currently reused controls.

### Cross-Story Alignment

- Epic-level UX requires both canonical entry points and contextual launch paths for app-scoped windows, but the score-context handoff is better aligned with later score-facing stories than with this first manager-window story.
- Story 3.2 owns score-context readiness and the primary contextual recovery path. That is the correct seam for opening the relevant score-context UI rather than defaulting users into an unrelated app-scoped workflow.
- Story 3.3 owns the contextual assignment picker and explicitly says users should not need the full Plugin Manager window just to assign a plugin.
- Story 2.3 should therefore establish the dedicated app-scoped window and leave the future contextual handoff requirement explicit in notes, not implicit in unfinished UI.

### Previous Story Intelligence

- Story 2.1 introduced the backend-owned inventory seam in `src-tauri/src/playback/plugin_inventory.rs`, the `soloc://state/plugin_inventory` event channel, and the shared `PluginInventoryState` / `PluginInventoryEntry` frontend model.
- Story 2.2 confirmed that startup already emits cache-first inventory state through the existing provider flow, so the Plugin Manager should consume that same state rather than adding an alternate fetch path.
- The current Sequence `Plugins` surface in `src/windows/main/components/plugins.tsx` already implements:
  - inventory browse rendering
  - search
  - `Available only` filtering
- compact availability messaging
- Reuse that behavior where practical, but do not preserve a second temporary presentation of the same inventory data just because an older debug-oriented surface exists.
- Existing tests also confirm there is currently no plugin-settings entry point on inventory tiles. Preserve that separation here unless a later dedicated story changes it.

### Architecture Compliance

- Rust remains the sole source of truth for plugin inventory state and availability classification.
- The frontend stays presentation-only and must not create a second inventory model for the manager window.
- App-scoped inventory belongs in a dedicated window; score-context relationships stay in panels.
- The Sequence menu owns the canonical launch point for Plugin Manager in the current product state.
- Machine-local plugin inventory remains outside `.soloc` and outside score persistence.
- Keep all FE/BE contracts aligned to existing `snake_case` Rust-owned models.
- VST3 plugins remain untrusted third-party code; the manager window only reports inventory truth and health state, it does not alter host-safety boundaries.
- The older `plugin-host-poc` window is a proof seam rather than a canonical product destination, so replacing its standalone default capability slot with `plugin-manager` is acceptable as the inventory surface becomes the named app-scoped window.

### Current Relevant Seams

- `src-tauri/src/lib.rs`
  - Owns menu construction and menu-event emission.
  - This is the place to add the canonical Plugin Manager Sequence-menu item and emit a dedicated frontend event for it.
- `src/services/state/provider.tsx`
  - Already listens for menu events in the main window and opens secondary windows such as Preferences.
  - Reuse this pattern for the Plugin Manager launch path.
- `src/services/actions.ts`
  - Already contains `WebviewWindow` helpers and plugin-inventory commands.
  - If a shared `openPluginManager` helper improves clarity, keep it here rather than inventing one-off window logic in multiple components.
- `src/windows/main/components/plugins.tsx`
  - Current temporary inventory browse surface and toolbar.
  - This is the most likely place to converge on one shared Plugin Manager row presentation while keeping Sequence-specific wiring out of the app-scoped window.
- `src/services/state/context.tsx`
  - Already defines initial plugin inventory state used across windows.
- `vite.config.ts`
  - Owns the multi-page HTML entrypoint list. A dedicated Plugin Manager window needs to be added here if a new entrypoint is created.
- Existing non-manager plugin window seams:
  - `plugin-settings.html`
  - `src/windows/plugin-settings/`
  - `src-tauri/src/playback/plugin_editor_poc.rs`
  - These remain sampler- or proof-oriented surfaces rather than the app-scoped inventory manager.
  - Do not silently repurpose sampler-owned windows into Plugin Manager. The older plugin-host POC may be superseded at the capability/spec level without turning its experimental host behavior into Plugin Manager scope.

### Implementation Guidance

- Build the Plugin Manager as a dedicated app-scoped window that mounts its own `AppStateProvider`, matching the repo's existing multi-window pattern.
- Prefer converging on one shared inventory row presentation from the current Sequence plugin surface rather than preserving duplicate tile/grid markup for the same plugin data.
- Keep Sequence-only concerns out of shared components. The manager window should not inherit bottom-panel state, Sequence tab assumptions, or score-context framing.
- Reuse the current search and `Available only` filter because they already support the dedicated window's immediate browsing value and do not conflict with later stories.
- Keep availability semantics aligned to the current backend model:
  - `available`
  - `unavailable` with `validation_failure` detail
- Improve accessibility by pairing textual status labels and/or icons with current styling rather than relying on background or text color alone.
- Keep the manager browse-focused. Do not add assignment buttons, score selection coupling, or plugin editor launch affordances unless later story scope explicitly requires them.
- Treat the manager as the canonical home for inventory browsing, not as the default answer for common score-context assignment tasks.

### File Structure Requirements

- Backend and app-shell work should stay centered in:
  - `src-tauri/src/lib.rs` for menu item creation and event emission
- Frontend work will likely touch:
  - `src/services/state/provider.tsx` for menu-event listening and window opening
  - `src/services/actions.ts` only if extracting a reusable open-window helper adds clarity
  - `src/windows/main/components/plugins.tsx` for shared inventory UI extraction or cleanup
  - `src/windows/plugin-manager/` for the dedicated manager window implementation if a new window is added
  - `plugin-manager.html` and `vite.config.ts` if a new dedicated entrypoint is created
- Do not spread this story into score assignment panels, sampler state, transport state, or persistence/migration files.

### Testing Requirements

- Frontend tests should prove:
  - the canonical Sequence-menu launch path opens the dedicated Plugin Manager window
  - the manager window consumes `PluginInventoryState` through the existing provider path
  - reused search and `Available only` filter behavior still work in the dedicated window
  - availability state remains legible without color alone
  - shared inventory rendering changes do not regress the temporary Sequence `Plugins` surface
- Existing tests around `src/windows/main/components/plugins.test.tsx`, `src/services/state/provider.test.tsx`, and window-level tests are likely the best owning seams to extend.
- Validate with the project-standard commands relevant to touched files:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings` if Rust menu/app-shell changes trigger backend lint exposure

### Library / Framework Requirements

- Preserve the current brownfield stack and existing dependencies:
  - Tauri Rust crate `2.10.3`
  - Tauri JS API `2.10.1`
  - React `19.2.4`
  - TypeScript `5.9.3`
- Use the existing Tauri `WebviewWindow` multi-window pattern; do not introduce a new windowing approach.
- Do not add a new state-management library, window-management library, or alternate inventory-fetching client.

### Documentation Impact

- Update developer-facing docs if implementation changes the accurate description of:
  - where the canonical Plugin Manager entry point lives
  - the distinction between app-scoped inventory browsing and score-context assignment workflows
  - the availability of the dedicated Plugin Manager window in the current product state
- Likely touch points if behavior changes materially:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`

### Git Intelligence Summary

- Recent history shows Story 2.1 landed as `02d92556 feat(2.1): vst discovery` and Story 2.2 followed as `d6d10743 feat(2.2): plugin cache behaviour`.
- Story 2.3 should continue that pattern: extend the already-established inventory seam and existing window/menu infrastructure rather than reworking inventory architecture.

### Latest Technical Information

- Current Tauri 2 app-menu and secondary-window patterns in this repo already route canonical menu actions through `src-tauri/src/lib.rs` plus frontend listeners in `AppStateProvider`. Reusing that pattern for Plugin Manager keeps window ownership consistent with the existing brownfield shell.
- Current Tauri 2 menu and secondary-window patterns in this repo already route canonical menu actions through `src-tauri/src/lib.rs` plus frontend listeners in `AppStateProvider`. Reusing that pattern for Plugin Manager keeps window ownership consistent with the existing brownfield shell.
- The repo's Vite build already uses separate HTML entrypoints for app windows. A dedicated Plugin Manager can follow that same pattern cleanly without frontend router complexity.

### Project Structure Notes

- The existing Sequence `Plugins` surface remains temporary and contextual. It should not become the final definition of app-scoped inventory management.
- The existing `plugin-settings` window appears to be sampler/plugin-specific and should not be conflated with the Plugin Manager without an explicit cleanup decision.
- Keep the full UX promise visible in notes: later score-context stories still need to add contextual launch and recovery handoff paths so the product does not strand the Plugin Manager behind the menu bar only.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 2 Story 2.3, Epic 3 Story 3.2, and Epic 3 Story 3.3 requirements
- `_bmad-output/planning-artifacts/architecture.md` — app-scoped window ownership, menu-entry expectations, and frontend/backend boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — panels-versus-windows scope model, Plugin Manager role, and future contextual-launch expectation
- `_bmad-output/implementation-artifacts/2-1-discover-installed-vst3-plugins-into-a-local-inventory.md` — original inventory seam and machine-local ownership
- `_bmad-output/implementation-artifacts/2-2-reuse-a-cached-plugin-inventory-on-startup.md` — startup cache reuse, provider-path continuity, and current temporary surface notes
- `docs/project-context.md` — AI-agent implementation guardrails and FE/BE ownership rules
- `src-tauri/src/lib.rs` — current app menu and menu-event wiring
- `src/services/state/provider.tsx` — existing menu listener and secondary-window launch pattern
- `src/services/actions.ts` — `WebviewWindow` helpers and plugin inventory commands
- `src/services/state/context.tsx` — shared initial plugin inventory state across windows
- `src/windows/main/components/plugins.tsx` — current inventory renderer and reusable browse controls
- `src/windows/main/components/plugins.test.tsx` — current inventory-surface behavior and separation guardrails
- `vite.config.ts` — multi-page entrypoint wiring for secondary windows
- `plugin-settings.html` and `src/windows/plugin-settings/` — existing plugin-specific window scaffold that should not be confused with the app-scoped Plugin Manager without explicit cleanup

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.3`.
- Sprint status showed `2-3-expose-plugin-inventory-and-availability-in-a-dedicated-plugin-manager` in `ready-for-dev` under Epic 2.
- Cross-story review confirmed the user decision to ship the canonical Sequence-menu launch now while preserving contextual launch for later score-context UX stories.
- Wider epic analysis confirmed Story 3.2 owns contextual recovery paths and Story 3.3 owns contextual assignment controls, so 2.3 should not widen into those seams.
- Repo analysis confirmed the existing inventory state path is already window-safe through `AppStateProvider`, and that the current Sequence `Plugins` surface plus tests provide reusable manager-window UI building blocks.
- Repo analysis also confirmed an existing `plugin-settings` window scaffold is sampler/plugin-specific, so the story explicitly guards against silently rebranding that older surface into the Plugin Manager without intentional cleanup.
- Repo analysis confirmed the existing `plugin-host-poc` window remains a bounded proof seam for Stories 2.5 through 2.7, so the Plugin Manager work keeps that window distinct and preserves its capability coverage instead of silently deleting it.
- Added a dedicated `plugin-manager.html` entrypoint and `src/windows/plugin-manager/` surface so inventory browsing remains app-scoped and separate from score assignment panels.
- Added the `Plugin Manager...` Sequence-menu item in `src-tauri/src/lib.rs`, emitted `soloc://menu/plugin-manager`, and opened the window through `actions.openPluginManager()` from the main-window provider listener.
- Reused the shared inventory provider path and the existing `Plugins` / `PluginsToolbar` browse UI, while hiding the refresh control in the dedicated manager window to keep Story 2.4's manual full rescan flow deferred.
- Updated inventory status chips to pair text with iconography and explicit `aria-label` values so availability is legible without color alone.
- Validation run completed successfully with `npm run test:js`, `npm run lint`, `npm run check-types`, and `cd src-tauri && cargo clippy --lib -- -D warnings`.

### Completion Notes List

- Story context created with the Sequence-menu launch decision locked in for 2.3.
- Future contextual-launch responsibility is explicitly preserved for later score-context stories rather than silently dropped.
- The story favors extracting and reusing current inventory UI behavior instead of building a second inconsistent inventory renderer.
- The story explicitly prevents common failure modes: turning Plugin Manager into an assignment surface, introducing a second frontend inventory model, or over-expanding the first dedicated window with future-management features.
- Implemented a dedicated Plugin Manager window with a canonical Sequence-menu launch path and a dedicated Vite/Tauri entrypoint.
- Reused the backend-owned inventory provider flow and existing browse controls so the manager window and Sequence panel stay aligned on inventory semantics without introducing a second frontend model.
- Kept the manager browse-focused by omitting assignment controls and the manual refresh button from the dedicated window while preserving the Sequence panel as the temporary contextual browse seam.
- Added focused tests for the new action helper, main-window menu launch handling, accessible availability labels, and the dedicated manager window rendering/filter behavior.
- Updated `docs/architecture.md`, `docs/development-guide.md`, `docs/project-context.md`, and `docs/project-overview.md` to reflect the canonical Plugin Manager entry point, the retained `plugin-host-poc` proof seam, and the 10-entrypoint multi-window app layout.
- Refined the dedicated Plugin Manager presentation to a compact list view with only display name, status, and binary-path subtitle, and removed the window-local title/description block.
- Refined the inventory status presentation into color-coded chips while preserving text and icon cues for accessibility.
- Follow-up review fix: reused the existing `plugin-manager` window by label on repeated opens, aligned the story/docs to the Sequence-menu launch decision, and restored compact unavailable-plugin failure messaging in the minimal list rows.

### Retro Note

- Retro note for Epic 2: avoid keeping parallel views of the same plugin inventory data, especially when one survives only as a temporary debug-oriented renderer. Story 2.3 briefly drifted into that shape, which created stale props, stale spec language, and unnecessary UI divergence for one backend-owned dataset.

### File List

- `_bmad-output/implementation-artifacts/2-3-expose-plugin-inventory-and-availability-in-a-dedicated-plugin-manager.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `plugin-manager.html`
- `src/services/actions.test.ts`
- `src/services/actions.ts`
- `src/services/state/provider.test.tsx`
- `src/services/state/provider.tsx`
- `src/windows/main/components/plugins.test.tsx`
- `src/windows/main/components/plugins.tsx`
- `src/windows/plugin-manager/boot.tsx`
- `src/windows/plugin-manager/index.test.tsx`
- `src/windows/plugin-manager/index.tsx`
- `src-tauri/src/lib.rs`
- `vite.config.ts`

## Change Log

- 2026-03-21: Added the dedicated app-scoped Plugin Manager window, canonical app-menu launch wiring, shared inventory-browser reuse, focused tests, and developer-facing documentation updates.
- 2026-03-21: Refined the Plugin Manager UI to a compact list layout with reduced metadata and no extra header copy.
- 2026-03-21: Updated Plugin Manager status badges to color-coded chips that still retain text and icon cues.
