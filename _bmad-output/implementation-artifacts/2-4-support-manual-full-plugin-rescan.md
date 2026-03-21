# Story 2.4: Support Manual Full Plugin Rescan

Status: ready-for-dev

## Story

As a maintainer,
I want to trigger a full VST3 rescan on demand,
so that Compose can refresh its plugin inventory when the local environment changes.

## Acceptance Criteria

1. Given the local plugin environment changes after cached inventory was created, when the user requests a full rescan, then Compose performs a full refresh of the supported VST3 discovery paths and updates the inventory state used by the app.
2. Given the inventory is shown in the Plugin Manager, when a full rescan completes, then the updated discovery results are reflected in that inventory view and the user can tell that the inventory has been refreshed.
3. Given a full rescan is a distinct workflow from cached startup reuse, when this story is complete, then both behaviors coexist cleanly and the app does not require a full rescan for ordinary startup visibility.

## Tasks / Subtasks

- [ ] Task 1: Keep manual full rescan as the explicit full-reprobe seam in the Rust-owned inventory domain (AC: 1, 3)
  - [ ] Reuse `plugin_inventory_discover` as the explicit full discovery command and keep startup cache reuse bounded.
  - [ ] Ensure manual full rescan refreshes the complete supported root set, persists the resulting inventory, and emits updated inventory truth through `soloc://state/plugin_inventory`.
  - [ ] Do not route manual full rescan through the legacy sampler rescan seam or any frontend-owned cache state.
- [ ] Task 2: Add the explicit rescan UX to the dedicated Plugin Manager and keep the user-visible result legible (AC: 1, 2)
  - [ ] Add a clear rescan action in the dedicated Plugin Manager window built by Story 2.3.
  - [ ] Surface refresh-in-progress and refresh-complete feedback using existing Compose patterns so users can tell the inventory was updated.
  - [ ] Keep the temporary Sequence inventory surface coherent if it still reuses shared inventory components during the transition, but treat the Plugin Manager as the canonical app-scoped home for the workflow.
- [ ] Task 3: Preserve the Story 2.2 startup-cache boundary while allowing broad manual refresh (AC: 1, 3)
  - [ ] Do not widen startup validation into a hidden full reprobe.
  - [ ] Keep startup semantics cache-first and bounded, with manual full rescan remaining the broader maintenance workflow.
  - [ ] Keep machine-local inventory persistence outside `.soloc` and preserve stable inventory availability semantics after rescan.
- [ ] Task 4: Add focused validation and documentation for the explicit rescan workflow (AC: 1, 2, 3)
  - [ ] Add backend tests proving manual full rescan remains broader than startup validation and updates persisted inventory truth.
  - [ ] Extend frontend tests for Plugin Manager refresh interaction, refreshed inventory rendering, and user-visible completion state.
  - [ ] Update developer-facing docs where startup cache reuse versus manual full rescan behavior is described.

## Dev Notes

### Story Intent

- Story 2.1 introduced the Rust-owned inventory seam.
- Story 2.2 established bounded startup cache reuse.
- Story 2.3 gives inventory a dedicated app-scoped Plugin Manager window.
- Story 2.4 owns the explicit broad refresh workflow and must keep that workflow distinct from startup cache behavior.

### Scope Boundaries

- In scope:
  - Explicit manual full reprobe of the supported VST3 roots.
  - Plugin Manager rescan control and user-visible refresh feedback.
  - Persistence and event emission of the refreshed machine-local inventory.
- Out of scope:
  - Changing startup cache reuse semantics beyond preserving the 2.2 boundary.
  - Score-context assignment workflows or contextual recovery UX.
  - Native plugin editor hosting. Stories 2.5 through 2.7 own that seam.
  - `.soloc` persistence or project migrations.

### Cross-Story Alignment

- Story 2.3 should provide the dedicated Plugin Manager surface that Story 2.4 extends with explicit rescan UX.
- Story 3.3 later owns contextual score assignment and explicitly should not require users to manage the full Plugin Manager just to assign a plugin.
- Keep the temporary Sequence inventory surface secondary; the canonical app-scoped refresh workflow belongs in the Plugin Manager.

### Previous Story Intelligence

- `src-tauri/src/playback/plugin_inventory.rs` already contains the broad discovery seam via `plugin_inventory_discover` and the bounded startup seam via `PluginInventoryContext::new()`.
- `src/services/actions.ts` already exposes `actions.pluginInventory.discover()`.
- The current Sequence `PluginsToolbar` already has a refresh trigger labeled `Refresh plugin inventory`; reuse the successful wording and command seam where practical.
- Repository guardrail: do not confuse `plugin_inventory_discover` with the legacy `sampler_rescan_plugins` command.

### Architecture Compliance

- Rust remains the sole source of truth for inventory state and error classification.
- Manual full rescan stays machine-local and outside `.soloc`.
- Startup stays bounded; manual rescan is the only broad explicit reprobe seam.
- Frontend remains presentation-only and consumes `soloc://state/plugin_inventory` rather than reconstructing inventory truth locally.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_inventory.rs`
  - `PluginInventoryContext::new()` for bounded startup reuse.
  - `PluginInventoryContext::discover()` and `plugin_inventory_discover` for explicit full reprobe.
- `src/services/actions.ts`
  - `actions.pluginInventory.discover()` already wraps the backend command.
- `src/windows/main/components/plugins.tsx`
  - Existing refresh control and inventory presentation semantics.
- `src/windows/plugin-manager/` or equivalent Story 2.3 window files
  - Canonical app-scoped home for the refresh workflow once 2.3 lands.

### Implementation Guidance

- Treat Story 2.4 as explicit wiring and validation of the broad refresh path, not as a reinvention of discovery logic.
- Prefer reusing the existing command and shared inventory rendering rather than adding a second refresh path or manager-local state model.
- Give the user a clear sense of completion through existing timestamp/state language such as refreshed labels or updated discovery time; do not add speculative inventory-management features beyond this story.
- Keep naming crisp:
  - user-facing: `Refresh plugin inventory`
  - backend seam: `plugin_inventory_discover`
  - avoid sampler-era rescan terminology in new VST-owned code

### File Structure Requirements

- Backend work should stay centered in:
  - `src-tauri/src/playback/plugin_inventory.rs`
- Frontend work will likely touch:
  - `src/services/actions.ts`
  - shared inventory UI under `src/windows/main/components/plugins.tsx` if reused
  - the dedicated Plugin Manager window created in Story 2.3
- Do not spread Story 2.4 into sampler runtime, score assignment panels, or persistence/migration files.

### Testing Requirements

- Backend tests should prove:
  - manual full rescan refreshes all supported roots
  - refreshed state persists and re-emits correctly
  - startup remains bounded and is still distinct from manual full reprobe
- Frontend tests should prove:
  - Plugin Manager refresh invokes the correct command
  - refreshed inventory results are shown after completion
  - refresh feedback is user-visible and not color-only
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --lib -- -D warnings` if backend edits require it

### Documentation Impact

- Update docs that describe plugin inventory startup behavior, manual refresh, or the Plugin Manager workflow if implementation changes those descriptions.
- Likely touch points:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 2.4 acceptance criteria
- `_bmad-output/planning-artifacts/architecture.md` — app-scoped window ownership and backend-owned inventory truth
- `_bmad-output/planning-artifacts/ux-design-specification.md` — Plugin Manager as canonical inventory-management surface
- `_bmad-output/implementation-artifacts/2-2-reuse-a-cached-plugin-inventory-on-startup.md` — bounded startup/cache guardrails
- `_bmad-output/implementation-artifacts/2-3-expose-plugin-inventory-and-availability-in-a-dedicated-plugin-manager.md` — dedicated window scope and menu-entry decisions
- `src-tauri/src/playback/plugin_inventory.rs`
- `src/services/actions.ts`
- `src/windows/main/components/plugins.tsx`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.4`.
- Wider repo analysis confirmed the broad reprobe command already exists and is distinct from the bounded startup path.
- Cross-story review kept Story 2.4 focused on explicit rescan behavior in the dedicated Plugin Manager instead of widening Story 2.3 or regressing Story 2.2 semantics.

### Completion Notes List

- Story context preserves the critical boundary: startup stays bounded, manual rescan stays broad.
- Story context explicitly avoids the legacy sampler rescan seam.
- Story context keeps the dedicated Plugin Manager as the canonical app-scoped refresh surface while preserving later score-context boundaries.

### File List

- `_bmad-output/implementation-artifacts/2-4-support-manual-full-plugin-rescan.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
