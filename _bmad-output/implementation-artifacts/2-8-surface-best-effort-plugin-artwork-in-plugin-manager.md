# Story 2.8: Surface Best-Effort Plugin Artwork in Plugin Manager

Status: backlog

## Story

As a maintainer,
I want Plugin Manager rows to show plugin artwork when Compose can derive it safely,
so that the inventory is easier to scan without depending on artwork availability for core usability.

## Acceptance Criteria

1. Given Plugin Manager rows reserve a square visual slot for plugin identity, when Compose has best-effort artwork or icon metadata for a plugin, then the Plugin Manager renders that image in the same square slot used by the fallback badge and the row layout remains consistent across plugins with and without artwork.
2. Given VST3 plugins do not provide deterministic cross-platform cover art in the current inventory seam, when no usable artwork is available for a plugin, then Compose falls back to the generated square name badge and inventory browsing remains legible and complete without missing-image errors or blank identity slots.
3. Given plugin artwork is a presentation enhancement rather than core inventory truth, when this story is implemented, then discovery, cached startup reuse, and manual rescan remain backend-owned and coherent and missing artwork does not change plugin availability classification, block inventory browsing, or alter `.soloc` persistence boundaries.

## Tasks / Subtasks

- [ ] Task 1: Define an optional backend-owned artwork metadata seam without pretending VST3 provides deterministic cover art (AC: 1, 2, 3)
  - [ ] Identify the bounded machine-local sources Compose may inspect for artwork, such as bundle icons or known resource files, and treat them as best-effort only.
  - [ ] Extend the Rust-owned inventory model with optional artwork metadata only if the source can be represented cleanly without changing availability semantics.
  - [ ] Keep artwork metadata outside `.soloc` and out of score-assignment or transport-owned state.
- [ ] Task 2: Reuse the existing square badge slot in Plugin Manager rows for artwork-first rendering with reliable fallback (AC: 1, 2)
  - [ ] Render artwork in the same square slot and dimensions currently used by the generated badge in the shared list-row component.
  - [ ] Fall back to the generated letter badge immediately when artwork is absent, fails to decode, or is not safe to render.
  - [ ] Preserve current availability chips, text, and icon cues so artwork does not become the primary status signal.
- [ ] Task 3: Preserve current inventory and cache boundaries while sourcing artwork opportunistically (AC: 2, 3)
  - [ ] Do not let artwork extraction failure block plugin discovery, startup cache reuse, or manual full rescan completion.
  - [ ] Keep inventory truth backend-owned and avoid frontend-only probing of plugin bundles.
  - [ ] Ensure artwork lookup remains bounded and does not silently widen the critical-path inventory workflow into a heavy media scan.
- [ ] Task 4: Add focused validation and documentation for the best-effort artwork behavior (AC: 1, 2, 3)
  - [ ] Add backend tests for optional artwork metadata classification or extraction helpers where practical.
  - [ ] Extend frontend tests for artwork rendering, fallback badge behavior, and row-layout consistency.
  - [ ] Update docs describing Plugin Manager capabilities and the optional nature of plugin artwork support.

## Dev Notes

### Story Intent

- This is a nice-to-have inventory presentation enhancement, not a blocker for the core Epic 2 plugin-library and editor-host work.
- The story exists to improve scanning and recognition in the app-scoped Plugin Manager while preserving the current backend-owned inventory model.
- The existing generated square badge is already the correct fallback identity slot; this story only upgrades that slot when optional artwork is available.

### Scope Boundaries

- In scope:
  - Best-effort artwork or icon extraction for Plugin Manager inventory rows.
  - Reuse of the current square badge slot and row layout.
  - Optional machine-local inventory metadata that supports artwork rendering.
- Out of scope:
  - Treating artwork as required plugin metadata.
  - Score-context assignment, readiness, or recovery UX.
  - Native plugin editor hosting, transport, playback activation, or `.soloc` persistence changes.
  - Claims of universal VST artwork support across platforms or plugin vendors.

### Cross-Story Alignment

- Story 2.3 established the dedicated Plugin Manager and the compact list-row presentation used today.
- Story 2.4 remains the owner of manual full rescan workflow; artwork extraction must not blur that boundary or make rescans feel required for ordinary browsing.
- Story 3.x assignment workflows should continue to treat Plugin Manager as an app-scoped inventory surface, not as the required path for score assignment.
- Because this is explicitly nice-to-have, it should not block the remaining Epic 2 critical path work if higher-priority plugin runtime or host stories are still active.

### Previous Story Intelligence

- `src/windows/main/components/plugins.tsx` now renders the Plugin Manager in a compact list view with a square generated badge based on plugin name.
- The current badge slot is already stable enough to act as the future artwork slot without changing row density or layout.
- Local code inspection confirmed `PluginInventoryEntry` currently exposes no artwork or image field; legacy sampler-era metadata has a `cover_image` field, but that seam is not the supported VST inventory path.
- Research into VST3 and macOS bundle metadata found platform-specific icons and resources may exist, but not a deterministic cross-platform cover-art field that Compose can assume is present.

### Architecture Compliance

- Rust remains the sole source of truth for plugin inventory metadata.
- Any artwork information must remain optional, machine-local, and outside `.soloc` project persistence.
- The frontend should render provided optional metadata and preserve immediate fallback behavior; it should not probe plugin bundles directly.
- Plugin availability and validation state remain separate from artwork presence.
- The Plugin Manager stays app-scoped, and artwork remains a browse enhancement rather than assignment truth.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_inventory.rs`
  - Owns plugin inventory discovery, persistence, and emitted inventory truth.
- `src/services/types.ts`
  - Mirrors the Rust-owned plugin inventory entry shape in TypeScript.
- `src/windows/main/components/plugins.tsx`
  - Owns the shared compact list-row renderer, including the current square generated badge.
- `src/windows/main/components/plugins.test.tsx`
  - Already covers list-row rendering and the current fallback badge behavior.
- `src/windows/plugin-manager/index.tsx`
  - Canonical app-scoped surface consuming the shared inventory list view.

### Implementation Guidance

- Treat artwork as opportunistic metadata, not as a guaranteed asset pipeline.
- Preserve the current square identity slot size and shape so artwork and fallback badge are visually interchangeable.
- Prefer backend-derived lightweight references or decoded assets only if they fit the existing inventory model and do not widen discovery cost materially.
- If multiple platform-specific artwork sources are possible, document the supported precedence order explicitly rather than letting it emerge accidentally.
- Fail closed: if artwork lookup is ambiguous, unsafe, or slow, keep the fallback badge.

### File Structure Requirements

- Backend work should stay centered in:
  - `src-tauri/src/playback/plugin_inventory.rs`
  - helper files under `src-tauri/src/playback/` only if extraction logic needs a focused seam
- Frontend work will likely touch:
  - `src/services/types.ts`
  - `src/windows/main/components/plugins.tsx`
  - `src/windows/main/components/plugins.test.tsx`
  - `src/windows/plugin-manager/index.tsx` only if window-level adjustments are needed
- Do not spread this story into score assignment panels, sampler-owned runtime state, or persistence/migration files.

### Testing Requirements

- Backend tests should prove:
  - optional artwork metadata extraction does not affect availability classification
  - missing or invalid artwork sources degrade cleanly to no-artwork metadata
  - discovery or cache flows do not fail solely because artwork is unavailable
- Frontend tests should prove:
  - artwork renders in the existing square slot when metadata is present
  - the generated badge appears when artwork is absent or unusable
  - list rows stay visually and structurally consistent across both states
- Validate with the project-standard commands relevant to touched files:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --lib -- -D warnings` if backend inventory files change

### Documentation Impact

- Update docs if Plugin Manager capabilities expand to include optional artwork support or if the inventory metadata model gains a new optional field.
- Likely touch points:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 2 Story 2.8 acceptance criteria
- `_bmad-output/planning-artifacts/architecture.md` — app-scoped inventory ownership and Rust-owned plugin truth
- `_bmad-output/planning-artifacts/ux-design-specification.md` — Plugin Manager as the canonical app-scoped inventory surface and accessibility rules
- `_bmad-output/implementation-artifacts/2-3-expose-plugin-inventory-and-availability-in-a-dedicated-plugin-manager.md` — current Plugin Manager scope and list-row reuse
- `docs/project-context.md` — FE/BE ownership and multi-window guardrails
- `src-tauri/src/playback/plugin_inventory.rs`
- `src/services/types.ts`
- `src/windows/main/components/plugins.tsx`
- `src/windows/main/components/plugins.test.tsx`
- `src/windows/plugin-manager/index.tsx`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story added explicitly as Epic 2 Story 2.8 at the user's request.
- Story scope is based on current Plugin Manager implementation and follow-up research into plugin artwork metadata feasibility.
- Research conclusion: VST3 plugins may expose platform-specific icons or resource assets, but there is no deterministic cross-platform cover-art field in the current inventory seam.
- The story is intentionally parked as `backlog` so it does not interrupt higher-priority Epic 2 implementation work.

### Completion Notes List

- Created a dedicated deferred story for optional plugin artwork support in Plugin Manager.
- Scoped the story around best-effort artwork extraction with fallback to the current generated square badge.
- Preserved existing architectural boundaries: backend-owned inventory truth, app-scoped Plugin Manager ownership, and no `.soloc` persistence impact.
- Recorded that this enhancement should not block the remaining Epic 2 critical path.

### File List

- `_bmad-output/implementation-artifacts/2-8-surface-best-effort-plugin-artwork-in-plugin-manager.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/planning-artifacts/epics.md`
