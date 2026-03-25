# Story 2.2: Reuse a Cached Plugin Inventory on Startup

Status: done

## Story

As a composer,
I want Compose to reuse a cached plugin inventory on startup,
so that available plugins are shown quickly without forcing a full rescan every launch.

## Acceptance Criteria

1. Given Compose has previously discovered a local VST3 inventory, when the app starts normally, then it can load the cached plugin inventory for immediate use and startup does not require a full reprobe to expose known plugins.
2. Given cached inventory data exists and the user has not requested a full rescan, when the app starts normally, then Compose may perform a bounded startup validation pass that rechecks cached bundle paths and related cached entry state, performs one cheap discovery-root sweep for uncached bundle paths, probes only newly seen bundles, and does not require a full discovery reprobe to keep ordinary startup visibility coherent.
3. Given some cached plugin entries may no longer be present or valid on disk, when the startup sanity check runs, then Compose marks those entries consistently as unavailable, stale, or otherwise non-usable according to the current inventory model and cached startup behavior remains consistent with the current local environment.
4. Given no cached inventory exists yet, when the app starts normally, then Compose performs initial discovery so plugin surfaces are not left permanently empty and the resulting inventory becomes the machine-local cache for later startups.
5. Given plugin inventory is machine-local state, when cache reuse is implemented, then that cache remains outside `.soloc` project data and project persistence is not used as a substitute for local plugin discovery state.

## Tasks / Subtasks

- [x] Task 1: Add backend-owned startup cache reuse to the VST3 inventory seam (AC: 1, 4, 5)
  - [x] Extend `PluginInventoryContext` startup initialization so persisted inventory state becomes immediately available during app initialization without requiring an explicit frontend action.
  - [x] Keep cached inventory loading inside `src-tauri/src/playback/plugin_inventory.rs`; do not route startup cache truth through the legacy SCVI sampler state or a frontend store.
  - [x] When no persisted inventory exists, trigger initial discovery from the backend-owned startup path so the Sequence plugin surface does not remain empty until manual refresh.
- [x] Task 2: Implement a bounded startup validation pass that does not widen into Story 2.4 rescan behavior (AC: 2, 3)
  - [x] Add a cached-entry validation path that checks the persisted entries already known to the inventory seam.
  - [x] Restrict startup validation to cached bundle-path checks plus one cheap discovery-root sweep for uncached bundle paths; do not run a full candidate reprobe during ordinary startup.
  - [x] Reuse the shared VST3 helper seam where practical so missing bundles, missing binaries, and file-read failures degrade into the existing inventory failure model instead of inventing a second taxonomy.
  - [x] Mark stale or no-longer-usable cached entries consistently in the existing inventory model so downstream UI and later recovery stories can rely on stable semantics.
- [x] Task 3: Preserve the existing frontend state flow while proving cache-first startup visibility (AC: 1, 2, 4)
  - [x] Keep `soloc://state/plugin_inventory` as the event channel and `PluginInventoryState` as the Rust-aligned shared model.
  - [x] Ensure `initialize` emits cached or newly discovered inventory state on startup so the existing `Plugins` surface shows inventory truth without requiring the refresh button.
  - [x] Keep frontend fallout minimal and presentation-only; do not add a frontend-owned startup cache or client-side reconstruction logic.
- [x] Task 4: Add focused regression coverage near the owning seams (AC: 1, 2, 3, 4, 5)
  - [x] Add backend tests for persisted cache load on startup, empty-cache fallback discovery, bounded stale-entry validation, and stable unavailability/error classification.
  - [x] Extend frontend/provider tests only where needed to prove startup-emitted inventory state is consumed correctly without an explicit manual discovery action.
  - [x] Validate the story with the project-standard commands relevant to touched areas.
- [x] Task 5: Capture documentation impact for startup cache behavior (AC: 2, 4, 5)
  - [x] Update developer-facing docs that describe plugin inventory startup behavior, machine-local cache ownership, or the distinction between cache reuse and manual full rescan if implementation changes those descriptions.
  - [x] Keep docs explicit that startup cache reuse is not `.soloc` persistence and not a substitute for the later manual full rescan workflow.

## Dev Notes

### Story Intent

- Story 2.1 already established a dedicated Rust-owned VST3 inventory seam with machine-local persistence and frontend state emission.
- Story 2.2 owns the startup fast path that reuses that persisted inventory instead of forcing discovery every launch.
- The outcome should be cache-first startup visibility, not a new manager UI, not manual rescan UX, and not score-assignment behavior.
- The startup path must remain compatible with the longer Epic 2 sequence:
  - Story 2.3 adds the dedicated Plugin Manager window.
  - Story 2.4 adds the explicit full rescan workflow.
  - Epic 3 later consumes inventory truth for score assignment and recovery.

### Scope Boundaries

- In scope:
  - Backend-owned reuse of persisted plugin inventory during startup.
  - Empty-cache fallback discovery during startup.
  - Bounded startup validation that keeps startup coherent with the current machine state while also surfacing newly added bundles.
  - Minimal frontend proof that startup-emitted inventory appears without manual refresh.
- Out of scope:
  - Manual full rescan controls or root-wide reprobe behavior. Story 2.4 owns that workflow.
  - Dedicated Plugin Manager UI. Story 2.3 owns that app-scoped window.
  - Project persistence, `.soloc` schema changes, or migration work for plugin inventory. Inventory remains machine-local only.
  - Score-instrument assignment, auto-assignment, plugin loading, or transport behavior.
  - Any widening of startup validation into a full compatibility probe, repeated full-root probing, or editor-host lifecycle test.

### Previous Story Intelligence

- Story 2.1 introduced the durable seam the dev agent should extend rather than replace:
  - `src-tauri/src/playback/plugin_inventory.rs` already owns `PluginInventoryContext`, persisted inventory load/save, discovery, and `soloc://state/plugin_inventory` emission.
  - `src-tauri/src/playback/vst3.rs` already centralizes macOS VST3 root expansion, bundle validation, and bundle binary resolution helpers.
  - The frontend already consumes `PluginInventoryState` through `src/services/state/provider.tsx` and the temporary Sequence `Plugins` surface.
- Story 2.1 guardrails still apply here:
  - do not move inventory truth into the SCVI sampler domain
  - do not persist machine-local inventory into `.soloc`
  - keep React presentation-only
  - keep terminology distinct between `inventory`, `assignment`, and `recovery`
- Story 2.1 completion notes already confirmed focused tests around persistence round trips, explicit runtime error surfacing, and bottom-panel display integration. Story 2.2 should extend those tests rather than invent a new harness.

### Architecture Compliance

- Rust remains the sole source of truth for plugin inventory state, startup cache reuse, and inventory error classification.
- Machine-local plugin discovery/cache state lives outside `.soloc` and must never become canonical project state.
- Startup behavior must stay offline-first and local-desktop only; no network assumptions or external registries.
- Shared models and commands crossing the Rust to TypeScript boundary must remain `snake_case`.
- Frontend startup handling stays event-driven through the existing provider pattern; do not add a second startup-fetch flow that reconstructs backend truth in React.
- Any degraded startup outcome must preserve a recoverable inventory state instead of clearing the cache silently or poisoning project state.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_inventory.rs`
  - Already opens the local `sled` database, loads any persisted inventory payload, discovers roots on demand, persists inventory state, and emits `soloc://state/plugin_inventory`.
  - This is the primary seam for cache reuse, bounded startup validation, bounded uncached-bundle discovery, and empty-cache fallback discovery.
- `src-tauri/src/playback/vst3.rs`
  - Already holds shared macOS VST3 helper logic for search roots, bundle validation, and bundle binary resolution.
  - Reuse this seam for cached-entry validation and one-pass uncached-bundle listing where it stays bounded to known roots.
- `src-tauri/src/lib.rs`
  - Owns state initialization and app startup wiring.
  - The startup sequence already emits plugin inventory state during `initialize`; this story should preserve that shape while making the emitted state cache-first useful.
- `src/services/state/provider.tsx`
  - Already listens to `soloc://state/plugin_inventory` and updates frontend state during initialization.
  - This story should not require a new manual fetch or alternate startup state path.
- `src/services/state/context.tsx`
  - Defines `initialPluginInventoryState` and the provider context used across windows.
- `src/windows/main/components/plugins.tsx`
  - Current temporary confirmation surface for plugin inventory.
  - Story 2.2 should let this surface show cached startup inventory immediately, without turning it into the final Plugin Manager.

### Implementation Guidance

- Treat startup reuse as an extension of the existing inventory seam, not a second inventory system.
- Preserve the current behavior where persisted state loads synchronously into `PluginInventoryContext` and emits during startup, but add the missing business behavior that distinguishes:
  - cache present: emit cached state immediately, revalidate cached entries, and append any newly seen bundles from one bounded root sweep
  - cache absent: run initial discovery and persist the result for future startups
- For this story, define the bounded startup sanity check narrowly:
  - inspect cached entries first
  - check whether the cached bundle path still exists and still looks like a valid bundle candidate under the existing helper rules
  - reuse cached entry metadata where possible
  - do one cheap root listing sweep to spot uncached `.vst3` bundles
  - probe only newly seen uncached bundle paths
  - do not treat startup sanity checking as a substitute for Story 2.4's explicit full rescan
- Keep stale-entry handling inside the current inventory model. The current enum only distinguishes `available` vs `unavailable`, so represent stale or missing entries through `availability: unavailable` plus existing failure detail rather than inventing a parallel frontend-only status model.
- Preserve stable plugin identity for still-valid cached entries so later assignment and recovery stories can build on this seam.
- If startup validation hits file I/O or bundle-read issues, degrade those entries predictably in the inventory state and continue startup; ordinary startup should not fail hard because one cached plugin is missing or broken.
- Do not introduce audio-thread, transport-thread, or renderer-adjacent behavior for this story. This is startup inventory work only.

### File Structure Requirements

- Backend work should stay centered in:
  - `src-tauri/src/playback/plugin_inventory.rs`
  - `src-tauri/src/playback/vst3.rs` only if a small helper extraction or reuse point is needed
  - `src-tauri/src/lib.rs` only for bounded startup wiring changes
- Frontend fallout should stay localized to:
  - `src/services/state/provider.tsx`
  - `src/services/state/context.tsx` if startup defaults need adjustment
  - `src/services/types.ts` only if the backend-owned shared model changes
  - `src/windows/main/components/plugins.tsx` only if startup-state messaging needs a minimal display update
- Do not spread Story 2.2 concerns into project persistence, score domain files, transport runtime, or new window scaffolding.

### Testing Requirements

- Backend tests should stay close to `plugin_inventory.rs` and prove:
  - persisted inventory reloads during startup without a full discovery pass
  - empty cache falls back to bounded initial discovery
  - cached entries missing on disk degrade to `unavailable` with stable failure detail
  - startup sanity checking remains bounded and appends newly seen uncached bundles without turning into a full reprobe
  - storage/open/load failures still surface explicitly without corrupting persisted truth
- Frontend/provider tests should stay narrow and prove:
  - startup-emitted plugin inventory populates state without clicking refresh
  - stale/unavailable entries from startup still render coherently in the reused `Plugins` surface
  - existing alert behavior for inventory errors remains stable if startup validation surfaces errors
- Prefer the existing validation commands used by Story 2.1:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --lib -- -D warnings` when backend edits expose Rust lint risk

### Library / Framework Requirements

- Preserve the current brownfield stack and existing dependencies:
  - Tauri Rust crate `2.10.3`
  - Tauri JS API `2.10.1`
  - React `19.2.4`
  - TypeScript `5.9.3`
  - `sled = 0.34.7`
- Do not add a new persistence library, frontend cache library, or alternate startup-state store for this story.

### Documentation Impact

- Update developer-facing docs if implementation changes the accurate description of:
  - startup plugin inventory behavior
  - machine-local cache ownership
  - the distinction between cache reuse and explicit full rescan
- Most likely documentation touch points if behavior changes materially:
  - `docs/project-context.md`
  - `docs/development-guide.md`
  - `docs/architecture.md`
- Keep documentation explicit that `.soloc` remains portable project state only and does not carry plugin discovery cache data.

### Git Intelligence Summary

- Recent branch history shows Story 2.1 just landed as `02d92556 feat(2.1): vst discovery`.
- That makes Story 2.2 a direct follow-on story: extend the new VST inventory seam rather than reworking it.
- The repo is already on Epic 2 with `2-1-discover-installed-vst3-plugins-into-a-local-inventory` in `review` and `2-2-reuse-a-cached-plugin-inventory-on-startup` still in `backlog`, so this story should assume the 2.1 implementation exists but may still receive minor review-driven cleanup.

### Latest Technical Information

- Current Tauri 2 path guidance still distinguishes app-scoped config, data, local-data, and cache directories. The existing repo pattern already uses `app.path().app_config_dir()` successfully for machine-local `sled` storage, so Story 2.2 should reuse that backend storage seam rather than inventing a new path strategy.
- Current Steinberg VST3 guidance still lists macOS user and global VST3 directories among the standard locations and notes that VST3 discovery is folder-based rather than registry-based. Story 2.1 already constrained MVP discovery to the user and global system roots only, and Story 2.2 should preserve that scope instead of expanding startup behavior to network or app-bundled roots.

### Project Structure Notes

- The current `Plugins` bottom-panel remains a temporary contextual confirmation surface until Story 2.3 introduces the dedicated app-scoped Plugin Manager.
- The current `PluginInventoryAvailability` shared model is intentionally simple. Story 2.2 should use that existing backend model plus `validation_failure` detail rather than bloating the frontend with a richer temporary status taxonomy.
- This story is foundational for later assignment and recovery work. Conservative choices around startup identity stability, stale-entry degradation, and machine-local cache boundaries are more important than squeezing in extra manager or rescan behavior early.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 2 and Story 2.2 acceptance criteria
- `_bmad-output/planning-artifacts/prd.md` — cache-first startup requirement and explicit distinction between cached startup and manual full rescan
- `_bmad-output/planning-artifacts/architecture.md` — Rust-owned inventory truth, machine-local cache boundaries, and command/state ownership rules
- `_bmad-output/planning-artifacts/ux-design-specification.md` — panel vs window scope model and accessibility rules for inventory state display
- `_bmad-output/implementation-artifacts/2-1-discover-installed-vst3-plugins-into-a-local-inventory.md` — previous story learnings, guardrails, and exact seams established in Story 2.1
- `docs/project-context.md` — project workflow rules, FE/BE ownership constraints, and validation expectations
- `src-tauri/src/playback/plugin_inventory.rs` — owning backend seam for persisted inventory load/save and startup cache reuse
- `src-tauri/src/playback/vst3.rs` — reusable VST3 helper seam for bounded cached-entry validation
- `src-tauri/src/lib.rs` — app startup initialization and state emission
- `src/services/state/provider.tsx` — frontend startup listener for plugin inventory state
- `src/services/state/context.tsx` — plugin inventory initial context state
- `src/services/types.ts` — Rust-aligned frontend plugin inventory model
- `src/windows/main/components/plugins.tsx` — temporary startup confirmation surface for inventory visibility

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.2`.
- Sprint status identified the story key as `2-2-reuse-a-cached-plugin-inventory-on-startup` and showed Epic 2 already `in-progress`.
- Previous-story analysis confirmed Story 2.1 already delivered the dedicated `plugin_inventory` seam, local `sled` persistence, frontend event wiring, and focused tests.
- Architecture, PRD, and UX analysis all aligned on the same boundaries: Rust-owned inventory truth, machine-local cache outside `.soloc`, cache-first startup, and explicit separation between bounded startup reuse and later manual full rescan.
- External verification confirmed current Tauri 2 app-local path guidance and current Steinberg macOS VST3 location guidance before finalizing implementation notes.
- The story narrows the ambiguous “bounded file-lookup sanity check” to cached-entry validation plus one cheap uncached-bundle sweep so startup remains fast and Story 2.4 retains ownership of root-wide reprobe behavior.
- `PluginInventoryContext::new()` now resolves standard VST3 roots once, reuses any persisted inventory during startup, validates cached entries in place, appends any newly seen uncached bundles from one bounded sweep, and falls back to initial discovery only when no persisted cache exists.
- Startup validation remains bounded: it reuses the cache, surfaces stale cached entries, and only probes new bundles when the one-pass root listing finds them, preserving Story 2.4 ownership of broader reprobe behavior.
- Rust regression coverage now includes startup cache reuse, empty-cache startup discovery, stale cached entry degradation, and load-failure preservation of the persisted payload.
- Review follow-up fixed the corrupt persisted-cache path so startup now surfaces the storage error, falls back to bounded discovery when roots are available, and rewrites the cache with recovered valid inventory state.
- Follow-up behavior refinement now lets startup surface newly installed bundles automatically through one cheap root sweep while keeping broader rescan ownership on the manual refresh path.
- Frontend provider coverage now proves `soloc://state/plugin_inventory` startup payloads are consumed by existing consumers without a manual refresh action.
- Validation completed with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --lib -- -D warnings`.

### Completion Notes List

- Story context created with direct continuation from Story 2.1, minimizing reinvention risk for the dev agent.
- Guidance explicitly prevents common failure modes: turning startup validation into a hidden full rescan, moving cache truth into `.soloc`, or introducing a frontend-owned startup cache model.
- The story keeps the temporary Sequence `Plugins` panel as the confirmation surface until Story 2.3 adds the dedicated Plugin Manager.
- Documentation impact is included so startup cache semantics remain accurate in developer-facing docs if implementation changes behavior.
- Startup now emits backend-owned cache-first inventory truth without requiring a frontend discovery action, while preserving the existing `soloc://state/plugin_inventory` event path.
- Cached entries that disappear or become invalid now degrade to `availability: unavailable` with stable `validation_failure` detail and retained identity instead of forcing a full startup reprobe.
- Newly added bundles under the standard discovery roots now appear on startup without a manual refresh, but startup still probes only those uncached bundle paths instead of rebuilding the whole inventory.
- When no persisted inventory record exists, startup performs initial backend discovery and persists the resulting machine-local cache for later launches.
- If the persisted inventory payload is corrupt, startup now recovers through bounded discovery instead of leaving the inventory empty until manual refresh.
- Developer-facing docs now state explicitly that startup cache reuse is machine-local state outside `.soloc` and remains distinct from the manual refresh/rescan workflow.

### File List

- `_bmad-output/implementation-artifacts/2-2-reuse-a-cached-plugin-inventory-on-startup.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/development-guide.md`
- `_bmad-output/planning-artifacts/prd.md`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src/services/state/provider.test.tsx`

## Change Log

- 2026-03-21: Implemented backend-owned startup cache reuse and cached-entry validation for plugin inventory, added focused Rust and frontend regression coverage, and updated developer docs for startup cache semantics.
- 2026-03-21: Addressed review feedback by recovering from corrupt persisted inventory payloads via bounded startup discovery and revalidated the full Rust suite.
- 2026-03-21: Refined startup validation to do one cheap uncached-bundle sweep, probe only newly seen bundles, and updated the planning/developer records to match.
