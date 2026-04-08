# Story 1.1: Remove Legacy Playback Path Ownership

Status: done

## Story

As a composer,
I want Compose to use only the supported VST-owned playback path,
so that playback behavior is predictable and no hidden sampler or SCVI path changes what I hear.

## Acceptance Criteria

1. Given the app contains sampler-era or SCVI-era playback entry points, state surfaces, routing hooks, or supporting code in the active or near-active product path, when the VST playback path is invoked through the supported UI and backend actions, then the active playback flow uses only VST-owned runtime seams, and superseded sampler or SCVI product-path behavior and code are removed rather than preserved as fallback, bridge, or compatibility ownership.
2. Given existing score and playback actions call backend playback seams, when those actions are executed after the reset, then they resolve through one authoritative VST-owned backend path, and no frontend-facing action or state surface continues to present legacy playback ownership as active behavior.
3. Given legacy removal changes persisted or shared playback-domain structures, when the implementation changes load or save semantics or persisted data shape, then migration impact is assessed and pending migration steps are updated as required, and portable project intent remains intact.
4. Given some sampler-backed behavior may not yet exist in the new VST system, when removal work reaches those seams, then the legacy sampler behavior is still removed instead of being retained for temporary parity, and any resulting functional gap is treated as an accepted part of the reset rather than a reason to keep old seams alive.

## Tasks / Subtasks

- [x] Remove backend sampler ownership from active playback activation flow (AC: 1, 2, 4)
  - [x] Delete `AppContext::reconcile_playback_activation_runtime_with_sampler` and `AppContext::reconcile_playback_activation_runtime_with_app` from `src-tauri/src/lib.rs` and stop calling them during initialization or refresh flows.
  - [x] Remove `sync_sampler_with_playback_runtime(...)` calls from score mutation seams in `src-tauri/src/score/instruments/mod.rs`, player mutation seams in `src-tauri/src/score/players.rs`, project load in `src-tauri/src/files.rs`, and plugin inventory discovery in `src-tauri/src/playback/plugin_inventory.rs`.
  - [x] Keep `PlaybackActivationRuntime::from_score_and_inventory(...)` as the authoritative readiness gate so route activation stays inventory-owned rather than sampler-cache-owned.
  - [x] Delete any remaining sampler code whose purpose is to support or influence the supported playback path, even if it currently props up behavior not yet rebuilt in VST-owned seams.
- [x] Remove legacy sampler-backed frontend ownership and audition surfaces (AC: 1, 2, 4)
  - [x] Remove `SamplerContext`, `initialSamplerState`, and `useSamplerState()` from `src/services/state/context.tsx`.
  - [x] Remove the `soloc://state/sampler` listener and `SamplerContext.Provider` wrapping from `src/services/state/provider.tsx`.
  - [x] Remove `actions.sampler.*` from `src/services/actions.ts` and delete or disconnect the corresponding Tauri command usage from product UI.
  - [x] Remove sampler-backed audition entry points from `src/windows/plugin-settings/index.tsx`, `src/windows/main/hooks/use-keyboard-shortcuts.ts`, `src/windows/main/components/sequencer-track.tsx`, and any other active product surfaces that still trigger legacy sampler note preview, without preserving temporary sampler-based parity behavior.
  - [x] Remove sampler diagnostics from debugger-facing product surfaces rather than relabeling them as active playback state.
- [x] Remove the remaining legacy sampler implementation rather than leaving inert scaffolding behind (AC: 1, 4)
  - [x] Delete `src-tauri/src/playback/sampler.rs`, `src-tauri/src/playback/scvi.rs`, `src-tauri/src/playback/streamer.rs`, and related glue only if they are no longer required by any supported VST-owned path after the reset; otherwise continue removal until no old sampler runtime remains in the app.
  - [x] Remove obsolete sampler state types, events, commands, tests, fixtures, and helper utilities instead of leaving them dormant in the main codebase.
  - [x] Treat loss of unreplaced sampler-only functionality as acceptable within story 1.1 when that is the cost of achieving a true clean slate.
- [x] Clean up legacy backend seams and confirm bounded persistence impact (AC: 1, 3, 4)
  - [x] Remove or disconnect legacy sampler/scvi ownership code that only exists to support the active playback path; do not preserve a parallel fallback path once the VST-owned seam is authoritative.
  - [x] Assess whether any persisted-data or load/save semantics changed; if they did, update `src-tauri/src/migrations/next.rs`, and if they did not, record that the story intentionally made no persisted-data migration change.
  - [x] Verify `.soloc` portability remains unchanged and machine-local plugin inventory or cache state is still excluded from project files.
- [x] Add regression coverage for the VST-only ownership reset (AC: 1, 2, 3, 4)
  - [x] Add or extend Rust tests around playback activation so route readiness is proven to come from plugin inventory and score routes, not sampler cache state.
  - [x] Add or extend load/save or file-open tests proving project reopen rebuilds playback activation without sampler reconciliation and still degrades unavailable plugins to recoverable non-ready route state.
  - [x] Update frontend tests and provider helpers to remove sampler-context assumptions.
  - [x] Remove or replace legacy sampler action and debugger tests that only prove the deleted path.
- [x] Update developer-facing documentation and agent guidance (AC: 1, 2, 3)
  - [x] Update `docs/architecture.md`, `docs/project-context.md`, `docs/development-guide.md`, and `TESTING.md` so they no longer describe `SamplerContext`, sampler events, or SCVI-backed audition as part of the active playback path.
  - [x] Update AI-agent instructions if implementation rules change as part of this seam deletion work.

## Dev Notes

### Story Intent

This story is the reset point for playback ownership. The dev agent must remove sampler and SCVI-era ownership from the product, not hide it behind compatibility wording or keep it around as dormant scaffolding. The rack-first and inventory-owned VST path stays; the legacy sampler system goes away, even where that temporarily reduces feature coverage.

### Confirmed Product Decisions

- Remove sampler-backed note audition surfaces in this story rather than preserving them behind a temporary bridge.
- Remove sampler diagnostics from debugger-facing product surfaces in this story rather than keeping them as legacy-only displays.
- Prefer a true clean slate over legacy parity. If old sampler functionality has no VST replacement yet, remove it anyway instead of letting it keep shaping the architecture.

### Technical Requirements

- Rust remains the sole source of truth for playback-domain state.
- `PlaybackActivationRuntime` must be rebuilt from score state plus plugin inventory only.
- Do not introduce a new adapter layer that preserves sampler-era ownership under different naming.
- Do not preserve legacy sampler or SCVI fallback behavior, dormant helper code, or temporary parity paths once this reset begins.
- Any persistence-impact assessment must follow the repo rule: implement change, update `src-tauri/src/migrations/next.rs` only if semantics changed, and explicitly note migration impact.

### Architecture Compliance

- Preserve the existing Tauri + Rust + React + TypeScript + Vite brownfield foundation.
- Keep playback-critical mutations backend-owned and command-oriented.
- Treat machine-local plugin inventory as the only availability source for route activation; `.soloc` continues to store portable intent only.
- Keep score-context versus app-scoped surface boundaries intact while deleting legacy playback ownership.
- Do not add UI whose main purpose is to prove acceptance criteria literally. Removal should simplify the surface area.

### Library / Framework Requirements

- React state providers remain display-layer only; remove the sampler provider rather than replacing it with another frontend-owned playback truth source.
- Tauri commands remain `snake_case`; if sampler commands are removed, remove their frontend call sites in the same slice.
- Keep shared FE/BE payload naming aligned with Rust-owned models.

### File Structure Requirements

- Backend ownership changes belong in `src-tauri/src/lib.rs`, `src-tauri/src/playback/`, `src-tauri/src/score/`, and file-load seams.
- Frontend ownership cleanup belongs in `src/services/` and the affected window components under `src/windows/`.
- Do not move playback-domain truth into React contexts, utilities, or window-local state.

### Testing Requirements

- Prefer seam-local regression coverage.
- Cover runtime activation, project load or reopen, inventory refresh, and score mutation seams where sampler ownership is being removed.
- Update JS tests that currently require `useSamplerState()` or `actions.sampler.*`.
- Validate that transport-facing playback still consumes backend-owned activation state after the sampler path is removed.

### Implementation Risks And Guardrails

- The highest-risk mistake is deleting the sampler bridge without proving that inventory-owned activation still drives transport and route readiness correctly.
- A second risk is leaving hidden call sites behind, especially in keyboard audition and sequencer preview flows.
- A third risk is stopping at disconnection and leaving the old sampler runtime in the tree, which would keep influencing future work even if it no longer runs.
- A fourth risk is updating code but not docs, which would leave future agents free to reintroduce the deleted path.
- Do not claim a migration is unnecessary without checking load/save semantics and shared payload shape explicitly.

### Relevant Code Seams

- `src-tauri/src/lib.rs`: playback activation refresh and legacy sampler reconciliation.
- `src-tauri/src/score/instruments/mod.rs`: slot and route mutation seams currently syncing sampler state after VST activation refresh.
- `src-tauri/src/score/players.rs`: player mutations with sampler sync side effects.
- `src-tauri/src/files.rs`: project load path.
- `src-tauri/src/playback/plugin_inventory.rs`: inventory discovery currently followed by sampler sync.
- `src/services/state/context.tsx` and `src/services/state/provider.tsx`: legacy sampler frontend state surface.
- `src/services/actions.ts`: sampler command entry points.
- `src/windows/plugin-settings/index.tsx`, `src/windows/main/hooks/use-keyboard-shortcuts.ts`, `src/windows/main/components/sequencer-track.tsx`: sampler-backed audition entry points to remove.

### Documentation Impact

The implementation must update docs that still describe sampler state as part of the active product path. Minimum expected updates:

- `docs/architecture.md`
- `docs/project-context.md`
- `docs/development-guide.md`
- `TESTING.md`
- Any AI-agent instruction file whose rules still normalize sampler-era active-path ownership

### Git Intelligence Summary

- Recent repo direction already reflects the reset: `feat: course correction`, `feat(3.8): vst management.`, `chore: epic 3 redirection`, and blocked-story commits reinforce that this story is a cleanup and trust-restoration slice, not an additive feature slice.

### Project Structure Notes

- No prior story file exists in `_bmad-output/implementation-artifacts/`, so this story establishes the baseline format for subsequent implementation-ready stories in this sprint output.
- Current repo memory confirms the rack-first model is authoritative and that `SamplerContext` is compatibility residue only, not a valid owner for routed VST activation.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic 1: Trust One VST-Only Playback Foundation]
- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-03-24.md#1. Issue Summary]
- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-03-24.md#4.3 Architecture changes]
- [Source: _bmad-output/planning-artifacts/architecture.md#Core Architectural Decisions]
- [Source: docs/project-context.md#Critical Don't-Miss Rules]
- [Source: docs/architecture.md#Frontend Side (Display Layer)]
- [Source: /memories/repo/playback-rack-first-model.md]
- [Source: /memories/repo/scvi-retirement-verification.md]

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Recent git history reviewed for reset context.
- Repo exploration completed for active sampler coupling, test coverage, and docs impact.
- Validation completed with `npm run lint`, `npm run check-types`, `vitest run`, and `npm run test:rust`.

### Completion Notes List

- Removed backend sampler reconciliation, sampler commands, sampler state providers, sampler-backed debugger output, and the plugin-settings window from the supported product path.
- Deleted the legacy sampler, SCVI container reader, and streamer modules; moved the remaining shared `StreamBuffer` helper into `playback/mixer.rs` so no dead playback module had to stay exported.
- Removed the last dead sampler-era helper and regression from `playback/runtime_activation.rs` after review so the VST-owned runtime seam no longer carries unused sampler gating residue.
- Deleted the lingering orphaned `playback/sampler.rs` file and removed the unused `PluginInventoryContext::available_plugin_ids()` helper plus its self-only test so story 1.1 no longer leaves dead sampler cleanup residue in-tree.
- Migration impact: none. No `.soloc` payload shape, load/save semantics, or machine-local inventory persistence rules changed, so `src-tauri/src/migrations/next.rs` was intentionally left untouched.
- AI-agent instructions already enforced rack-first VST ownership and did not require changes for this story.

### Change Log

- 2026-03-25: Removed sampler/SCVI playback ownership from active backend and frontend paths, deleted the sampler-era window and runtime modules, updated docs, and validated with lint, typecheck, full JS tests, and Rust library tests.
- 2026-03-25: Post-review cleanup removed the last unused sampler-era runtime helper and its regression coverage from `src-tauri/src/playback/runtime_activation.rs`.
- 2026-03-25: Follow-up cleanup deleted the lingering orphaned `src-tauri/src/playback/sampler.rs` file and removed the unused `available_plugin_ids` helper/test from `src-tauri/src/playback/plugin_inventory.rs`.

### File List

- \_bmad-output/implementation-artifacts/1-1-remove-legacy-playback-path-ownership.md
- TESTING.md
- docs/architecture.md
- docs/development-guide.md
- docs/project-context.md
- docs/project-overview.md
- docs/source-tree-analysis.md
- plugin-settings.html
- src/services/actions.test.ts
- src/services/actions.ts
- src/services/state/context.test.tsx
- src/services/state/context.tsx
- src/services/state/provider.test.tsx
- src/services/state/provider.tsx
- src/services/test-utils.tsx
- src/services/types.ts
- src-tauri/capabilities/default.json
- src-tauri/src/automation/registry.rs
- src-tauri/src/bin/performance.rs
- src-tauri/src/files.rs
- src-tauri/src/lib.rs
- src-tauri/src/playback/mixer.rs
- src-tauri/src/playback/mod.rs
- src-tauri/src/playback/plugin_inventory.rs
- src-tauri/src/playback/runtime_activation.rs
- src-tauri/src/playback/sampler.rs
- src-tauri/src/playback/scvi.rs
- src-tauri/src/playback/streamer.rs
- src-tauri/src/score/instruments/mod.rs
- src-tauri/src/score/players.rs
- src/windows/debugger/index.test.tsx
- src/windows/debugger/index.tsx
- src/windows/main/components/sequencer-track.tsx
- src/windows/main/components/sequencer.tsx
- src/windows/main/hooks/use-keyboard-shortcuts.test.ts
- src/windows/main/hooks/use-keyboard-shortcuts.ts
- src/windows/plugin-settings/boot.tsx
- src/windows/plugin-settings/index.test.tsx
- src/windows/plugin-settings/index.tsx
- vite.config.ts
