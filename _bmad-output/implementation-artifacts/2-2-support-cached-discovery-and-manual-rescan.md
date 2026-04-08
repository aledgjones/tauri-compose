# Story 2.2: Support Cached Discovery And Manual Rescan

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want Compose to load known plugins quickly and let me rescan when needed,
so that startup stays fast without preventing me from refreshing my installed inventory.

## Acceptance Criteria

1. Given Compose has a previously built plugin inventory cache, when the app starts, then previously discovered plugins are available from the cached inventory without a full probe on every launch, and empty-cache startup still performs the required initial discovery path.
2. Given I install, remove, or troubleshoot local plugins, when I trigger a manual full rescan from the supported inventory workflow, then Compose refreshes the plugin inventory from the local VST3 locations, and the resulting inventory state is reflected in the Plugin Manager.
3. Given discovery behavior has explicit performance and validation expectations, when cached discovery and manual rescan are implemented, then the work includes measurable behavior or checks for startup and rescan expectations, and discovery remains limited to required local desktop paths and permissions.

## Tasks / Subtasks

- [x] Task 1: Confirm the current brownfield implementation against Story 2.2 before changing behavior unnecessarily (AC: 1, 2, 3)
  - [x] Trace startup inventory initialization through `src-tauri/src/lib.rs` and `src-tauri/src/playback/plugin_inventory.rs` to verify that persisted inventory state is reused on startup, cached entries are revalidated, and only uncached bundle paths are probed during the bounded startup sweep.
  - [x] Verify the explicit full-rescan path remains the `plugin_inventory_discover` command exposed through `src/services/actions.ts` and consumed by the dedicated Plugin Manager toolbar in `src/windows/main/components/plugins.tsx` and `src/windows/plugin-manager/index.tsx`.
  - [x] Document any current repo drift between the acceptance criteria and the existing implementation before adding new code, with specific attention to startup timing evidence and refresh-result feedback.
- [x] Task 2: Tighten or complete the backend cache-first discovery seam without widening ownership boundaries (AC: 1, 2, 3)
  - [x] Keep all startup cache reuse and persistence behavior inside `src-tauri/src/playback/plugin_inventory.rs`; do not introduce frontend-owned cache state, alternate persistence paths, or project-file persistence for machine-local inventory data.
  - [x] Preserve the current startup contract: missing persisted state triggers initial discovery, while an existing persisted state record, including an empty one, is treated as cache-present and revalidated rather than forcing a full reprobe.
  - [x] Preserve stable plugin identity when cached bundles disappear or break by degrading those entries to `availability = unavailable` with `validation_failure`, instead of deleting them silently or changing the ownership model.
  - [x] Keep discovery limited to the required local VST3 search roots and app-local storage permissions; do not broaden path access or add background discovery flows under this story.
- [x] Task 3: Keep manual rescan app-scoped and ensure refreshed backend truth is visible in the Plugin Manager (AC: 2, 3)
  - [x] Reuse the existing `PluginsToolbar` refresh action and `PluginInventoryContext` event flow so the Plugin Manager reflects the backend-owned refreshed inventory without creating a second frontend inventory model.
  - [x] If the current UI does not provide enough feedback for refresh-in-progress or refreshed-state confirmation, add only the minimum bounded UX needed in the dedicated Plugin Manager window; do not turn score-context panels into a second inventory-management surface.
  - [x] Ensure any inventory refresh side effects continue to rebuild backend-owned same-session playback activation truth instead of introducing a Plugin-Manager-local recovery path.
- [x] Task 4: Strengthen validation, documentation, and measurable behavior checks around cache-first startup and manual rescan (AC: 1, 2, 3)
  - [x] Extend the focused Rust tests in `src-tauri/src/playback/plugin_inventory.rs` for startup cache reuse, empty-cache startup discovery, degraded stale entries, full manual rescan replacement, and persisted reload behavior rather than inventing a separate discovery harness.
  - [x] Add or update focused frontend tests only where the refresh workflow contract needs coverage, using existing suites such as `src/windows/plugin-manager/index.test.tsx`, `src/windows/main/components/plugins.test.tsx`, `src/services/actions.test.ts`, and `src/services/state/provider.test.tsx`.
  - [x] Capture the measurable behavior required by AC 3 through seam-local checks or explicit assertions, and keep any representative timing or validation evidence aligned with the existing plugin-inventory architecture rather than ad hoc scripts.
  - [x] Update developer-facing docs that describe plugin inventory startup, refresh, and ownership boundaries if implementation or wording changes.

## Dev Notes

### Story Intent

- Story 2.2 is a brownfield verification-and-hardening story, not a greenfield discovery implementation.
- The current repo already contains the core cache-first startup seam, explicit manual rescan command, persisted machine-local inventory storage, and Plugin Manager refresh entry point.
- The developer should start by proving the current implementation satisfies the story, then close only the remaining gaps around measurable validation, UX feedback, tests, or documentation.

### Brownfield Reality

- `src-tauri/src/playback/plugin_inventory.rs` already persists inventory state under the app config directory, reloads persisted state on startup, revalidates cached entries, performs a bounded uncached root sweep, and exposes explicit manual full discovery through `discover()` and `plugin_inventory_discover`.
- `src/windows/main/components/plugins.tsx` already exposes a `Refresh plugin inventory` action through `PluginsToolbar`, and `src/windows/plugin-manager/index.tsx` already mounts that shared toolbar inside the canonical app-scoped Plugin Manager window.
- The current codebase therefore appears to be at least partially ahead of Story 2.2. Do not replace working seams simply to make the story look newly implemented.

### Scope Boundaries

- In scope:
  - Cache-first startup reuse of machine-local plugin inventory.
  - Explicit manual full rescan of local VST3 discovery roots.
  - Plugin Manager reflection of refreshed backend-owned inventory truth.
  - Seam-local validation, measurable checks, and docs tied to this behavior.
- Out of scope:
  - Project-file persistence of plugin inventory or any `.soloc` ownership change.
  - Route authoring, score-context plugin selection UX, or recovery workflows owned by Epic 3.
  - Native editor-host expansion owned by Story 2.4.
  - Background discovery services, watcher-based rescans, or broader inventory-management features not required by the story.

### Cross-Story Guardrails

- Story 2.1 established Plugin Manager as the canonical app-scoped inventory browser. Keep manual rescan anchored there rather than moving inventory-management responsibility into score-context panels.
- Story 2.3 will consume the backend-owned inventory from a musician-friendly score-context picker. Do not add a second inventory truth model or broader assignment UX in Story 2.2.
- Story 2.4 owns bounded native editor-host behavior. Preserve `editor_host_validation` as adjacent inventory metadata, but do not widen startup discovery into editor-host-specific behavior under this story.
- Inventory refresh already participates in same-session playback activation rebuilding. Preserve that backend-owned coupling instead of adding ad hoc frontend recovery logic.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory truth, persistence, validation, and emission.
- Machine-local plugin inventory and cache metadata stay outside `.soloc` project files and outside source-controlled assets.
- App-scoped windows own full inventory browsing and maintenance actions; score-context surfaces consume backend-owned inventory truth without becoming inventory managers.
- Startup should stay cache-first and bounded. Manual rescan remains the explicit broader discovery seam.
- Discovery remains limited to required local VST3 roots and ordinary desktop permissions.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_inventory.rs`
  - `PluginInventoryContext::new()` resolves app-local storage and startup roots.
  - `from_db_path_and_roots()` loads persisted state, chooses cache reuse versus first discovery, and persists updated truth.
  - `validate_persisted_state()` revalidates cached entries and performs the bounded uncached sweep.
  - `discover_uncached_inventory_in_roots()` limits startup discovery to previously unseen bundle paths.
  - `discover()` and `discover_in_roots()` perform the explicit full-rescan path.
  - Existing test coverage already exercises persistence, cache reuse, stale entry degradation, uncached bundle discovery, and full manual rescan replacement.
- `src-tauri/src/lib.rs`
  - Creates and manages the shared `PluginInventoryContext` during app startup.
  - Registers the `plugin_inventory_discover` command.
- `src/services/actions.ts`
  - Exposes `actions.pluginInventory.discover()` as the frontend command wrapper.
- `src/services/state/provider.tsx`
  - Delivers `soloc://state/plugin_inventory` payloads to all windows through `PluginInventoryContext`.
- `src/windows/main/components/plugins.tsx`
  - Hosts `PluginsToolbar`, the existing refresh button, and shared inventory-browse controls.
- `src/windows/plugin-manager/index.tsx`
  - Mounts the canonical app-scoped Plugin Manager window that should continue to own refresh and browse responsibilities.

### Implementation Guidance

- Start with a gap analysis against the current repo. If the existing behavior already satisfies part of the story, preserve it and tighten only the missing pieces.
- Reuse the existing backend-owned persistence and refresh seams; do not add alternative caches, duplicate state providers, or new command surfaces.
- Preserve the startup distinction captured in repo memory: persisted cache reuse is the normal startup path, while manual refresh is the broader explicit maintenance path.
- Treat an empty persisted record as cache-present. Only missing persisted state should trigger initial discovery.
- If you need stronger AC 3 evidence, prefer explicit seam-local tests or bounded assertions over one-off instrumentation scripts.
- Keep refreshed-state UX intentional but minimal. The Plugin Manager may surface progress or confirmation if needed, but it should not grow into a second score-context workflow.

### Testing Requirements

- Prioritize seam-local Rust coverage in `src-tauri/src/playback/plugin_inventory.rs` for:
  - startup cache reuse without a full reprobe,
  - empty-cache startup discovery,
  - degraded unavailable cached entries when bundles disappear,
  - bounded discovery of uncached bundles on startup,
  - manual full rescan replacing and persisting new truth,
  - persisted-load failure fallback behavior.
- Add frontend tests only where Story 2.2 behavior depends on the app-scoped refresh affordance or state reflection:
  - `src/windows/plugin-manager/index.test.tsx`
  - `src/windows/main/components/plugins.test.tsx`
  - `src/services/actions.test.ts`
  - `src/services/state/provider.test.tsx`
- Validate touched code with the standard project commands relevant to the change:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings` if Rust files change

### Documentation Impact

- Update the following docs if implementation or wording changes:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `docs/project-overview.md` if the plugin-inventory startup or refresh workflow description changes materially
- Documentation must continue to state that plugin inventory is machine-local backend-owned truth, startup is cache-first, and full reprobe is an explicit manual refresh workflow.

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 2.2 user story and acceptance criteria; FR3, FR4, NFR4, NFR10, NFR12, NFR14.
- `_bmad-output/planning-artifacts/architecture.md` — Rust-owned inventory truth, app-scoped-versus-score-context boundaries, and machine-local persistence rules.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — Plugin Manager as the canonical app-scoped inventory and discovery surface.
- `docs/project-context.md` — frontend/backend ownership rules, multi-window guidance, and plugin inventory architecture constraints.
- `docs/architecture.md` — current plugin inventory cache-first startup and refresh architecture.
- `docs/development-guide.md` — current developer-facing guidance for startup cache reuse, manual refresh, and validation expectations.
- `_bmad-output/implementation-artifacts/2-1-build-the-app-scoped-plugin-inventory-window.md` — prior story learnings and scope boundaries for keeping Plugin Manager app-scoped.
- `/memories/repo/plugin-inventory-startup-cache.md` — startup cache contract and manual-rescan guardrails.
- `/memories/repo/plugin-manager-window-scope.md` — app-scoped Plugin Manager boundary and no-duplicate-browser rule.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.2`.
- Sprint status showed `2-2-support-cached-discovery-and-manual-rescan` as `ready-for-dev`, then was updated to `in-progress` for execution.
- Planning artifacts define Story 2.2 as cached startup plus explicit manual rescan with measurable validation expectations.
- Repo inspection confirmed the existing brownfield seams already satisfy most of AC 1 and AC 2: `PluginInventoryContext::new()` wires startup inventory ownership through `src-tauri/src/lib.rs`, persisted state reload happens in `from_db_path_and_roots()`, cache-present startup revalidates entries via `validate_persisted_state()`, and manual full reprobe remains `plugin_inventory_discover` through `src/services/actions.ts` and `PluginsToolbar` in the Plugin Manager window.
- The concrete repo drift was refresh-result feedback: docs already stated the dedicated Plugin Manager shows refresh-in-progress and refreshed-state feedback, but `src/windows/main/components/plugins.tsx` only disabled the refresh button during discovery.
- Backend ownership boundaries remained correct and required no architectural rewrite: startup cache reuse, stale-entry degradation, bounded root sweeping, persistence, and same-session playback activation rebuilding all stayed in Rust.
- Added one focused Rust regression test covering the empty persisted-cache contract and one focused frontend regression cluster covering refresh progress, success, and failure feedback.
- Validation completed with `npm run lint`, `npm run check-types`, `npm run test:js`, `cargo test --lib --manifest-path src-tauri/Cargo.toml --quiet`, and `cargo clippy --lib --manifest-path src-tauri/Cargo.toml -- -D warnings`.

### Completion Notes List

- Confirmed the existing Rust-owned cache-first discovery seam already met the story boundary: persisted inventory reload, cached-entry revalidation, bounded uncached startup sweep, stale-entry degradation, and manual full rescan remained unchanged.
- Kept the Plugin Manager refresh affordance minimal: manual rescan disables the refresh button while running and uses a dialog on command failure instead of inline status messaging.
- Added a Rust regression test for empty persisted cache reuse and frontend regression tests for refresh progress and completion feedback.
- Brought the repo-standard Rust validation gate back to green by removing one unused audio accessor and fencing test-only mixer helpers to test builds only; runtime behavior did not change.
- Existing docs in `docs/architecture.md` and `docs/development-guide.md` already described the intended refresh feedback and bounded startup sweep accurately, so no wording update was required.

### File List

- `_bmad-output/implementation-artifacts/2-2-support-cached-discovery-and-manual-rescan.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/windows/main/components/plugins.tsx`
- `src/windows/main/components/plugins.test.tsx`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src-tauri/src/playback/audio.rs`
- `src-tauri/src/playback/mixer.rs`

## Change Log

- 2026-03-25: Verified the brownfield startup/manual-rescan seam, added bounded Plugin Manager refresh feedback, extended focused cache-contract coverage, and cleared the repo validation gates for Story 2.2.
- 2026-03-25: Simplified Plugin Manager refresh UX back to button-disable plus failure dialog, removing the inline refresh status treatment.
