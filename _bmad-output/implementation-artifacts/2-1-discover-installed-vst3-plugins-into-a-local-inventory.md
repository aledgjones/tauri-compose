# Story 2.1: Discover Installed VST3 Plugins Into a Local Inventory

Status: ready-for-dev

## Story

As a composer,
I want Compose to discover installed VST3 plugins on macOS,
so that I can see which playback tools are available without configuring them manually.

## Acceptance Criteria

1. Given Compose is launched on a supported macOS environment, when plugin discovery runs, then the app identifies installed VST3 plugins from the supported local discovery paths and stores the resulting inventory as machine-local application state rather than project data.
2. Given valid and invalid plugin candidates may both exist on disk, when discovery completes, then Compose records enough information to distinguish discovered plugins from undiscoverable or unusable ones and the discovery result is available to the app as inventory truth.
3. Given this is the first VST inventory story, when it is complete, then no score-assignment workflow is required for the story to be valuable and users or maintainers can confirm that Compose knows what plugins exist locally.

## Tasks / Subtasks

- [ ] Task 1: Introduce a Rust-owned VST3 inventory domain model and machine-local persistence seam (AC: 1, 2)
  - [ ] Add a dedicated playback inventory module under `src-tauri/src/playback/` instead of extending the legacy SCVI sampler model in place.
  - [ ] Define explicit Rust-owned inventory records for discovered plugins, availability state, and discovery failure detail using `snake_case` fields that can cross the Rust ↔ TypeScript boundary unchanged.
  - [ ] Persist the discovered inventory in app-local storage outside `.soloc` files, reusing the existing `sled` + `app.path().app_config_dir()` style already used by preferences and recent files.
- [ ] Task 2: Implement bounded macOS VST3 discovery and validation for the approved MVP search roots (AC: 1, 2)
  - [ ] Discover candidate bundles only from `~/Library/Audio/Plug-Ins/VST3` and `/Library/Audio/Plug-Ins/VST3` for this story's MVP scope.
  - [ ] Reuse and extract the existing VST3 path-validation and bundle-resolution helpers from `src-tauri/src/playback/plugin_editor_poc.rs` where practical instead of duplicating filesystem logic.
  - [ ] Validate candidates enough to separate usable bundles from unavailable/problematic entries without widening this story into full runtime loading, assignment, or editor-host lifecycle work.
  - [ ] Keep discovery ordering deterministic so repeated scans produce stable inventory ordering and stable identity records.
- [ ] Task 3: Expose inventory truth through backend commands and the existing confirmation surface (AC: 2, 3)
  - [ ] Add or update backend command seams so discovery can run and emit Rust-owned inventory state to the frontend.
  - [ ] Keep the frontend as a presentation layer only; do not introduce a frontend-owned plugin inventory model.
  - [ ] Reuse the existing Sequence bottom-panel `Plugins` surface as the temporary confirmation UI for Story 2.1 rather than creating the dedicated Plugin Manager window early.
  - [ ] Make the reused surface capable of showing discovered versus unavailable/problematic plugin state clearly enough for confirmation, without trying to complete Story 2.3's full manager workflow.
- [ ] Task 4: Add focused validation close to the owning seams (AC: 1, 2, 3)
  - [ ] Add Rust tests for discovery-root handling, deterministic ordering, candidate validation, and persistence/load behavior for machine-local inventory state.
  - [ ] Extend frontend tests only where needed to prove the reused `Plugins` surface consumes the new inventory shape correctly.
  - [ ] Run the relevant validation commands needed to prove the new story remains build-safe and testable.
- [ ] Task 5: Capture narrow documentation impact for the new inventory seam (AC: 1, 2)
  - [ ] Update developer-facing docs that describe playback/plugin state ownership if they become inaccurate after the new inventory seam is added.
  - [ ] Do not document a dedicated Plugin Manager workflow yet; Story 2.3 owns that app-scoped window.

## Dev Notes

### Story Intent

- This story establishes the first real VST3 inventory truth in the product.
- The goal is discovery plus machine-local persistence, not assignment, playback, cached-startup optimization, full rescan UX, or editor-host lifecycle behavior.
- This story starts the clean replacement path away from SCVI-era seams. Remaining sampler/runtime code is temporary legacy scaffolding, and the explicit project end state is full removal of that subsystem in Epic 7 Story 7.2 after the VST replacement path is complete.
- The user resolved two otherwise ambiguous decisions for this story:
  - Supported discovery roots for MVP are the user and system VST3 folders only.
  - Confirmation should reuse the existing Sequence bottom-panel `Plugins` surface until Story 2.3 introduces the dedicated Plugin Manager window.

### Scope Boundaries

- In scope:
  - Rust-owned VST3 inventory domain model.
  - Discovery of installed VST3 bundles from the approved macOS roots.
  - Machine-local persistence outside `.soloc`.
  - Minimal command/state/UI wiring required to confirm inventory results in the existing plugin list surface.
  - Availability state rich enough to distinguish usable from unavailable/problematic candidates.
- Out of scope:
  - Startup cache reuse as the primary fast path. Story 2.2 owns that optimization.
  - A dedicated Plugin Manager window. Story 2.3 owns that app-scoped surface.
  - Manual full rescan workflow. Story 2.4 owns explicit rescan UX.
  - Native editor host lifecycle. Stories 2.5 through 2.7 own that seam.
  - Score-instrument assignment, auto-assignment, or playback activation. Epic 3 owns those workflows.
  - Persisting inventory into `.soloc`, adding migrations for project files, or inventing compatibility scaffolding.

### Architecture Compliance

- Rust remains the sole source of truth for plugin inventory, assignment state, playback preparation state, and transport state.
- `.soloc` persists portable project intent and assignment references only; machine-local plugin discovery/cache state must remain outside project files.
- Internal app communication stays command-oriented and backend-owned through typed Tauri commands using `snake_case` names and payloads.
- React remains a thin control/presentation layer. Do not add a second frontend-owned inventory truth.
- Do not build the new VST inventory seam on top of the legacy sampler domain. `SamplerContext`, `soloc://state/sampler`, and `actions.sampler.*` are legacy sampler/audition surfaces, not the target architecture for Epic 2.
- App-scoped playback inventories belong in dedicated windows long term, but Story 2.1 is explicitly using the existing bottom-panel surface as a temporary confirmation surface until Story 2.3.
- Plugin discovery is limited to explicit local VST3 locations and validated loading paths.
- VST3 plugins are untrusted third-party code from a host-safety perspective, so discovery failures must degrade predictably rather than crash or poison shared state.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Already contains macOS-oriented VST3 helper logic for search roots, path validation, bundle binary resolution, and deterministic root scanning.
  - Current constants only cover the user and system VST3 roots, which matches the story's approved MVP path scope.
  - This file is a useful source of reusable helper logic, but its editor-host session code is intentionally a bounded POC and should not become the inventory domain model.
- `src-tauri/src/playback/sampler.rs`
  - Still models legacy SCVI plugin scanning, loading, cache state, and audition behavior.
  - The existing `SamplerState` and `PluginMeta` shape show how FE state emission works today, but Story 2.1 should avoid forcing VST3 inventory truth into the SCVI sampler domain if a dedicated inventory seam is clearer.
- `src-tauri/src/lib.rs`
  - Registers current playback-related command seams and state contexts.
  - Will likely need new inventory command/state registration or a carefully updated existing seam.
- `src/services/actions.ts`
  - Exposes sampler actions such as `rescanPlugins()` and `openPluginEditorPoc()`.
  - Any new inventory action should preserve the command-oriented pattern and `snake_case` backend naming.
- `src/windows/main/components/plugins.tsx`
  - Existing bottom-panel plugin list UI that can serve as the temporary confirmation surface for Story 2.1.
  - Currently assumes legacy sampler plugin data and an “active only” filter, so it will likely need bounded adaptation for VST3 inventory status display.
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
  - Mounts the existing `Plugins` surface in the Sequence bottom panel.
  - Reuse this surface for confirmation instead of introducing a new app-scoped window prematurely.
- `src-tauri/src/preferences.rs` and `src-tauri/src/files.rs`
  - Show the current project pattern for app-local `sled` storage under `app.path().app_config_dir()`.
  - This is the preferred local persistence pattern to reuse before inventing a different storage stack.

### Implementation Guidance

- Prefer a new Rust inventory seam under `src-tauri/src/playback/` such as `plugin_inventory.rs` or a similarly named module, rather than overloading the legacy SCVI sampler runtime with VST3 responsibilities.
- Reuse the existing bottom-panel `Plugins` view only as a temporary presentation shell if helpful. Do not treat its current sampler-backed state shape, event channel, or command names as the final VST inventory contract.
- Reuse helper logic from `plugin_editor_poc.rs` where it is already correct:
  - root expansion and search ordering
  - VST3 bundle-path validation
  - bundle binary resolution rules
- Keep discovery deterministic:
  - sort roots and directory entries consistently
  - produce stable inventory ordering
  - produce stable plugin identity so later stories can build cache reuse, manager UI, and assignment logic on top of it
- Availability state should be explicit and backend-owned. The story should not collapse everything to a simple boolean if the product needs to distinguish “discovered and usable” from “found but unavailable/problematic.”
- Do not widen Story 2.1 into full plugin instantiation or editor-host startup just to determine availability. A bounded, defensible validation bar is enough for this story as long as it cleanly distinguishes usable versus unavailable/problematic entries.
- Do not serialize discovery results into project persistence or add project migrations. This is machine-local state only.
- Do not create the dedicated Plugin Manager window early. The temporary confirmation path is the existing Sequence bottom-panel plugin surface.
- Keep naming crisp:
  - use `inventory` for machine-local discovered plugin availability
  - reserve `assignment` for future score-instrument relationships
  - reserve `recovery` for future reopen and missing-plugin resolution flows

### File Structure Requirements

- Backend ownership should stay centered in:
  - `src-tauri/src/playback/`
  - `src-tauri/src/lib.rs`
  - any directly related command-registration or backend state files needed to emit inventory truth
- Local persistence should follow the existing backend pattern already demonstrated in:
  - `src-tauri/src/preferences.rs`
  - `src-tauri/src/files.rs`
- Frontend fallout should stay minimal and localized to:
  - `src/services/actions.ts`
  - `src/services/types.ts`
  - `src/windows/main/components/plugins.tsx`
  - `src/windows/main/screens/sequence/components/bottom-panel.tsx` only if needed for bounded display integration
- Avoid spreading Story 2.1 concerns into score persistence, assignment UI, transport runtime, or plugin-host window code unless a narrow helper extraction is required.

### Testing Requirements

- Rust coverage should stay near the owning seam and include:
  - discovery-root resolution for the approved user + system paths
  - deterministic sort behavior for discovered candidates
  - rejection/flagging behavior for invalid or unusable candidates
  - machine-local persistence round trips for inventory records
  - command-level behavior that proves the inventory becomes app-available state
- Frontend coverage should stay focused on the reused `Plugins` surface:
  - inventory entries render from the new backend-driven shape
  - problematic/unavailable state is visible without relying on color alone
  - existing search/filter behavior is either preserved intentionally or deliberately narrowed with tests updated accordingly
- Prefer existing project validation commands:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --lib -- -D warnings` when backend edits introduce Rust lint exposure

### Library / Framework Requirements

- Preserve the current brownfield Tauri + Rust + React + TypeScript + Vite foundation.
- Relevant current versions and dependencies:
  - Tauri Rust crate `2.10.3`
  - Tauri JS API `2.10.1`
  - React `19.2.4`
  - TypeScript `5.9.3`
  - `sled = 0.34.7` for current local app persistence patterns
  - `libloading = 0.8.9` and `vst3 = 0.3.0` already present for current VST3 host/POC work
- Do not add a new persistence library or frontend store for inventory state unless the existing patterns prove insufficient.

### Documentation Impact

- Update developer-facing docs if this story changes the accurate description of plugin inventory ownership or playback-related local state, especially:
  - `docs/project-context.md`
  - `docs/architecture.md`
  - `docs/development-guide.md` if it documents plugin-related backend seams
- Do not add or imply final Plugin Manager UX documentation here; Story 2.3 owns the dedicated window and its app-scope guidance.
- No `.soloc` migration or persistence-format documentation should change for this story because inventory remains machine-local only.

### Git Intelligence Summary

- Recent branch history shows Epic 1 cleanup has been completed and the branch is positioned to start VST inventory work:
  - `55384ca6 chore: epic 1 retro`
  - `dee33eeb chore(1.5): verification pass`
  - `05da1741 feat(1.4): documentation cleanup`
  - `2d9d844c feat(1.3): continued scvi cleanup`
  - `974ce5a2 feat(1.2): saving cleanup 2`
- Use Story 2.1 to establish a clean first VST inventory seam rather than reintroducing hybrid SCVI/VST ownership.

### Latest Technical Information

- Steinberg’s current VST3 macOS guidance lists these standard locations: user, global, network, and application-bundled VST3 folders. For Story 2.1, the user explicitly chose the narrower MVP scope of the user and global system folders only.
- Steinberg’s guidance also states VST3 discovery does not require registry-style plugin registration and that hosts should scan predefined folders deterministically.
- Tauri 2.10.x continues to expose app-scoped config/data/local-data directories. The current repo already uses `app.path().app_config_dir()` successfully for `sled`-backed local state, which is the preferred reuse path for Story 2.1.

### Project Structure Notes

- The repo still contains legacy sampler/plugin concepts tied to SCVI. Story 2.1 should not interpret those names as the desired long-term VST architecture.
- The existing `Plugins` bottom-panel UI is good enough as a temporary confirmation surface, but it is not the long-term app-scoped Plugin Manager. Keep the distinction explicit so Story 2.3 remains cleanly scoped.
- Because this story is the foundation for Stories 2.2 through 2.4 and Epic 3 assignment work, developer choices around identity, availability-state naming, and persistence boundaries must be conservative and reusable.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 2, Story 2.1 acceptance criteria and downstream story context
- `_bmad-output/planning-artifacts/prd.md` — VST3 product direction, offline/local-state constraints, machine-local inventory expectations
- `_bmad-output/planning-artifacts/architecture.md` — Rust-owned inventory truth, command-oriented FE/BE seams, machine-local persistence rules, scope model
- `_bmad-output/planning-artifacts/ux-design-specification.md` — plugin inventory scope model, temporary contextual surfaces versus future app-scoped Plugin Manager
- `docs/project-context.md` — project conventions, `snake_case` boundary rules, Tauri/Rust ownership, validation commands
- `src-tauri/src/playback/plugin_editor_poc.rs` — existing VST3 root discovery, bundle validation, and host helper logic
- `src-tauri/src/playback/sampler.rs` — legacy SCVI plugin state/emission pattern and current `sampler_rescan_plugins` seam
- `src-tauri/src/lib.rs` — active backend command/state registration
- `src-tauri/src/preferences.rs` — local `sled` persistence pattern via `app.path().app_config_dir()`
- `src-tauri/src/files.rs` — additional local `sled` persistence pattern and app-config-dir usage
- `src/services/actions.ts` — current command-oriented frontend action pattern
- `src/services/types.ts` — current frontend playback/plugin type patterns
- `src/windows/main/components/plugins.tsx` — temporary confirmation surface to reuse in Story 2.1
- `src/windows/main/screens/sequence/components/bottom-panel.tsx` — existing mount point for the temporary confirmation surface
- `docs/component-inventory.md` — existing documentation naming the `Plugins` panel

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.1` rather than auto-discovered from sprint status.
- No previous story exists in Epic 2, so there are no prior implementation learnings within the epic to import.
- User clarified two ambiguous decisions during story preparation: limit Story 2.1 discovery to the user + system macOS VST3 roots, and reuse the existing Sequence bottom-panel `Plugins` surface as the confirmation UI until Story 2.3.
- External verification checked current Steinberg VST3 location guidance and current Tauri 2.10.x app-local path guidance before finalizing implementation notes.

### Completion Notes List

- Story context created with explicit path-scope and confirmation-surface decisions so the dev agent does not have to infer product scope.
- Guidance explicitly prevents common failure modes: persisting inventory into `.soloc`, overloading the SCVI sampler as the long-term VST domain model, or creating the Plugin Manager window early.
- Story 2.1 is positioned as the reusable inventory foundation for Stories 2.2 through 2.4 and Epic 3 assignment work.

### File List

- `_bmad-output/implementation-artifacts/2-1-discover-installed-vst3-plugins-into-a-local-inventory.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
