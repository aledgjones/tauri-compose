# Story 1.2: Remove SCVI From User-Facing Playback Workflow

Status: done

## Story

As a composer,
I want the app to stop presenting SCVI-specific playback workflow elements,
so that Compose communicates one clear playback model without legacy confusion.

## Acceptance Criteria

1. Given the app contains SCVI-specific playback labels, controls, entry points, settings, or workflow branches, when this story is complete, then those user-facing SCVI-specific elements are removed or updated and the UI no longer suggests SCVI is a supported playback path.
2. Given playback-related screens or flows previously assumed SCVI-specific concepts, when a user navigates those surfaces after implementation, then they encounter only the remaining intended playback direction and no broken or orphaned UI states remain.
3. Given a user inspects playback-related windows, panels, and settings surfaces, when this story is complete, then SCVI does not appear as an active option, explanation, or fallback concept and the remaining workflow language is internally consistent across those surfaces.

## Tasks / Subtasks

- [x] Task 1: Remove or reframe SCVI-specific user-facing playback entry points and labels (AC: 1, 2, 3)
  - [x] Audit the plugin browser and related playback surfaces in `src/windows/main/components/plugins.tsx` and any direct callers so empty states, button labels, and action copy no longer imply a legacy SCVI workflow.
  - [x] Audit the per-plugin window in `src/windows/plugin-settings/index.tsx`; if the surface is still fundamentally SCVI-articulation preview UI, remove or hide the entry point rather than merely relabeling it.
  - [x] Keep `src/windows/plugin-host-poc/index.tsx` and any related entry point consistent with the intended VST-first direction only; do not leave mixed messaging between supported product workflow and POC language.
- [x] Task 2: Keep the direct instrument assignment workflow coherent without exposing legacy playback concepts (AC: 1, 2, 3)
  - [x] Preserve score-instrument assignment as the primary user-facing playback workflow in `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` and related sequence/setup surfaces.
  - [x] Review whether `useSamplerState()`-backed plugin inventory exposure remains acceptable for Story 1.2 as an internal implementation seam; if the name leaks into user-visible UI or maintainer-facing surfaces, update the presentation layer without forcing a broad backend refactor.
  - [x] Prefer removing broken or misleading affordances over keeping a control alive behind legacy behavior.
- [x] Task 3: Align audition and preference surfaces with the post-Story-1.1 product direction (AC: 2, 3)
  - [x] Audit `src/windows/preferences/screens/note-input-and-editing.tsx`, `src/windows/main/hooks/use-keyboard-shortcuts.ts`, and `src/windows/main/components/sequencer-track.tsx` so audition wording and behavior remain coherent after SCVI runtime removal.
  - [x] Do not promise or imply an SCVI-backed playback result anywhere the user can reach from note input, note editing, or sequence workflows.
  - [x] If a surface still depends on sampler-only behavior that no longer matches the supported workflow, either bound it explicitly as preview-only or remove it from the product path.
- [x] Task 4: Clean up user-visible debug and maintainer-facing workflow residue without spilling into tooling cleanup (AC: 1, 2, 3)
  - [x] Review `src/windows/debugger/index.tsx` and similar user-reachable maintainer surfaces so they do not present SCVI as an active playback option or source of product truth.
  - [x] Update user-visible labels, section titles, and explanatory text before considering deeper refactors such as renaming `actions.sampler.*` or backend command identifiers.
  - [x] Leave backend sampler/scvi module retirement, CLI removal, and build-script cleanup to Story 1.3 unless a minimal compile-safety change is strictly required here.
- [x] Task 5: Add focused validation for the updated workflow surfaces (AC: 2, 3)
  - [x] Update or add targeted Vitest coverage for the affected playback UI surfaces and action wiring.
  - [x] Validate that removed or changed playback entry points do not leave orphaned buttons, windows, or failing render paths.
  - [x] Run the relevant frontend and backend checks needed to prove the product remains coherent after the user-facing cleanup.
- [x] Task 6: Capture narrow documentation impact only where this story makes current guidance inaccurate (AC: 1, 3)
  - [x] Update only the focused developer-facing docs that would become misleading for implementation or validation after the workflow cleanup.
  - [x] Leave the broad SCVI documentation and repo-guidance retirement pass to Story 1.4.

## Dev Notes

### Story Intent

- Story 1.2 is product-workflow cleanup, not backend tooling retirement.
- The required outcome is that users see one clear playback direction and no supported SCVI workflow path.
- Story 1.1 already removed SCVI from the supported transport runtime. Story 1.2 must make the UI and workflow language match that reality.
- Prefer deleting or hiding misleading user-facing controls over preserving them with weak legacy caveats.

### Scope Boundaries

- In scope:
  - User-facing playback labels, controls, windows, settings, and workflow branches.
  - Maintainer/debugger surfaces that a user can open from the product and that still present legacy playback messaging.
  - Minimal frontend/backend seam changes required to keep those surfaces coherent and compile-safe.
- Out of scope:
  - Removing `src-tauri/src/playback/sampler.rs`, `src-tauri/src/playback/scvi.rs`, SCVI CLI binaries, or build scripts. Story 1.3 owns that cleanup.
  - Broad documentation retirement of SCVI references across the repo. Story 1.4 owns that.
  - Repo-wide verification sweep. Story 1.5 owns that.
  - Persistence migration, compatibility scaffolding, or any attempt to preserve legacy SCVI behavior for old workflows.

### Previous Story Intelligence

- Story 1.1 made supported `transport_play` timing-only on the product path and removed SCVI runtime playback ownership.
- Story 1.1 explicitly left user-facing workflow cleanup to Story 1.2 and tooling cleanup to Story 1.3.
- Current developer guidance in `docs/project-context.md` and `docs/architecture.md` now treats `SamplerContext` as legacy sampler tooling/audition state rather than the supported transport runtime.
- Do not reintroduce SCVI fallback logic in the UI just to preserve legacy affordances.

### Architecture Compliance

- Rust remains the source of truth for playback state and assignment state; frontend changes must stay in the display/control layer.
- `src/services/actions.ts` remains the canonical frontend entry point for backend mutations.
- The main product playback direction is VST-first and score-first. UI language should describe plugins, assignments, readiness, and playback behavior in musical/product terms rather than surfacing legacy container/runtime terminology.
- Keep changes focused on user-facing workflow surfaces rather than renaming every internal sampler seam in one pass.
- If a rename is purely internal and not needed for AC compliance, defer it.

### Current User-Facing Legacy Surfaces

- `src/windows/main/components/plugins.tsx`
  - Plugin browser, rescan control, empty states, and native-host POC entry point.
  - Uses `useSamplerState()` and `actions.sampler.rescanPlugins()`.
- `src/windows/plugin-settings/index.tsx`
  - Per-plugin articulation/mini-keyboard surface.
  - Uses `useSamplerState()` and `actions.sampler.audition(...)`.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Direct assignment dropdown driven from sampler-backed plugin inventory.
- `src/windows/preferences/screens/note-input-and-editing.tsx`
  - Audition preference wording.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Note-edit audition path calls `actions.sampler.audition(...)` when enabled.
- `src/windows/main/components/sequencer-track.tsx`
  - Newly created/edit-adjusted notes can trigger audition.
- `src/windows/debugger/index.tsx`
  - Exposes sampler state directly in a user-reachable debugger window.
- `src/services/actions.ts`, `src/services/state/context.tsx`, and `src-tauri/src/lib.rs`
  - Contain the user-reachable command/state seams that back the above workflow surfaces.

### Implementation Guidance

- Start with presentation truth, not internal type purity:
  - Remove or update what the user can see first.
  - Only rename internal sampler APIs/types if the current names leak into user-visible or maintainer-visible workflow language.
- Treat the sequence instrument assignment flow as the core retained workflow.
- Be skeptical of the current plugin-settings window:
  - If it is still effectively a sampler-articulation preview surface, removing the entry point is safer than pretending it is already a production VST workflow.
- Keep the debugger honest:
  - If it still needs to show legacy sampler state for maintainers, label it as legacy tooling/audition state rather than product playback truth.
- Do not widen this story into SCVI asset deletion, CLI removal, or deep backend module surgery.
- Do not add fallback language such as "legacy", "old engine", or "SCVI still available" in product UI. The active workflow should read as singular and current.

### File Structure Requirements

- Frontend workflow cleanup should stay centered in:
  - `src/windows/main/components/plugins.tsx`
  - `src/windows/plugin-settings/index.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/preferences/screens/note-input-and-editing.tsx`
  - `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - `src/windows/main/components/sequencer-track.tsx`
  - `src/windows/debugger/index.tsx`
- Frontend seam adjustments should stay localized to:
  - `src/services/actions.ts`
  - `src/services/state/context.tsx`
  - any directly affected tests
- Backend command registration fallout should stay minimal and localized to:
  - `src-tauri/src/lib.rs`
  - only the directly affected command handlers or labels required to keep the workflow coherent

### Testing Requirements

- Minimum validation target:
  - Targeted Vitest coverage for affected UI surfaces and action wiring.
  - Type-check and lint validation after UI workflow cleanup.
  - Rust test/build validation only if backend command registration or command behavior changes.
- Prefer existing project checks:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust` if Rust seams change
- Do not create a new cross-cutting playback workflow harness for this cleanup.

### Library / Framework Requirements

- Preserve the existing Tauri + React + TypeScript + Vite stack.
- Current pinned versions relevant to this story:
  - React `19.2.4`
  - TypeScript `5.9.3`
  - Vite `7.3.1`
  - Tauri frontend API `2.10.1`
  - Tauri Rust crate `2.10.3`
- Use the repo's existing component/state patterns; this is not a redesign story.

### Documentation Impact

- If implementation changes narrow developer-facing workflow guidance or debug terminology, update only the specific docs that become inaccurate, likely `docs/project-context.md` or `docs/architecture.md`.
- Leave broad product/docs reference cleanup to Story 1.4.
- If the story changes any required agent implementation practice, update the AI-agent instructions as part of the work.

### Git Intelligence Summary

- Recent commits confirm the branch direction:
  - `feat(1.1): scvi cleanup part 1`
  - `chore: spec vst work`
- Story 1.2 should extend that direction by making the retained product workflow clearly VST-first and by removing user-facing ambiguity rather than preserving transitional mixed messaging.

### Project Structure Notes

- The architecture planning docs still describe `src-tauri/src/playback/sampler.rs` and `scvi.rs` as existing seams. That does not make them supported user workflows.
- The UX spec favors direct instrument assignment, compact surfaces, and clear playback-readiness language over host-topology exposure.
- The PRD explicitly says SCVI should be removed rather than carried forward and that missing or unavailable playback resources should degrade gracefully instead of blocking score work.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 1, Story 1.2 acceptance criteria
- `_bmad-output/planning-artifacts/prd.md` — SCVI replacement, VST-first product direction, graceful degraded states
- `_bmad-output/planning-artifacts/architecture.md` — file placement, frontend/backend ownership, retained sampler seam boundaries
- `_bmad-output/planning-artifacts/ux-design-specification.md` — direct assignment UX, assignment clarity, avoid host-topology mental model
- `_bmad-output/implementation-artifacts/1-1-remove-scvi-from-the-active-playback-runtime.md` — prior-story boundaries and runtime learnings
- `docs/project-context.md` — current agent rules, retained legacy sampler/audition seam notes
- `src/services/actions.ts` — current `actions.sampler.*` user-reachable frontend API surface
- `src/services/state/context.tsx` — `SamplerContext` / `useSamplerState()` exposure
- `src/windows/main/components/plugins.tsx` — plugin browser, rescan, native-host POC entry point
- `src/windows/plugin-settings/index.tsx` — per-plugin articulation/keyboard preview surface
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` — score-instrument assignment dropdown
- `src/windows/preferences/screens/note-input-and-editing.tsx` — audition preference surface
- `src/windows/main/hooks/use-keyboard-shortcuts.ts` — note-edit audition path
- `src/windows/main/components/sequencer-track.tsx` — creation/edit audition path
- `src/windows/debugger/index.tsx` — maintainer-facing sampler state surface
- `src-tauri/src/lib.rs` — command registration and managed sampler state seam
- `package.json` — available validation scripts

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story target provided explicitly by user as `1.2`.
- Sprint tracker showed `1-2-remove-scvi-from-user-facing-playback-workflow` in `backlog` before story creation.
- Story context grounded in Epic 1 planning docs, PRD, architecture, UX spec, project-context rules, Story 1.1 output, current codebase seams, and recent git history.
- Independent exploration confirmed the likely in-scope user-facing surfaces are concentrated in plugin browser, plugin settings, sequence assignment, audition preference, sequencer audition paths, debugger, and frontend action/state seams.
- Removed the plugin-settings launch affordance and the experimental host-proof entry point from the plugin browser, and tightened plugin empty-state copy so the product path no longer advertises a legacy playback branch.
- Reframed the retained assignment and audition wording around plugin assignment and preview behavior, and relabeled debugger sampler inspection as legacy plugin audition state instead of product playback truth.
- Validation run: targeted Vitest coverage for touched surfaces passed; `npm run lint`, `npm run check-types`, and full `npm run test:js` passed after correcting two stale `actions.score.layout` flow-assignment assertions in `src/services/actions.test.ts`.

### Completion Notes List

- Story context created with explicit separation between user-facing workflow cleanup (Story 1.2) and backend/tooling retirement (Story 1.3).
- Story instructs the dev to prefer removing misleading SCVI-driven surfaces over preserving them with transitional caveats.
- Story keeps direct instrument assignment as the retained product workflow and treats internal sampler naming changes as optional unless user-visible.
- Narrow documentation impact is included per project rules.
- Plugin tiles no longer expose the sampler-era plugin-settings window, and the plugin browser no longer surfaces the experimental host proof on the product path.
- Sequence assignment now presents `Assign plugin` as the retained user-facing playback action, while note input/editing surfaces describe audition as preview behavior.
- Debugger sampler state remains available for maintainers but is explicitly labeled as legacy plugin audition state, avoiding product-path ambiguity without broad backend renames.
- Updated focused Vitest coverage for the plugin browser, assignment row, preferences screen, and debugger, plus refreshed the shared actions test assertions needed for a green repo-wide JS suite.

### File List

- `_bmad-output/implementation-artifacts/1-2-remove-scvi-from-user-facing-playback-workflow.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/development-guide.md`
- `src/services/actions.test.ts`
- `src/windows/debugger/index.test.tsx`
- `src/windows/debugger/index.tsx`
- `src/windows/main/components/plugins.test.tsx`
- `src/windows/main/components/plugins.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
- `src/windows/plugin-host-poc/index.tsx`
- `src/windows/preferences/screens/note-input-and-editing.test.tsx`
- `src/windows/preferences/screens/note-input-and-editing.tsx`

### Change Log

- 2026-03-20: Created Story 1.2 context file with concrete scope boundaries, file targets, prior-story learnings, and validation guidance.
- 2026-03-20: Removed the plugin-settings product entry point, removed the experimental host-proof product affordance, redacted sampler path data from the debugger, clarified empty assignment states, updated focused docs/tests, and validated with lint, typecheck, and full JS tests.
