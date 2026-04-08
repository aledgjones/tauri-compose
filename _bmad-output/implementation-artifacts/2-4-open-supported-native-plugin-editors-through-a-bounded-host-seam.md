# Story 2.4: Open Supported Native Plugin Editors Through A Bounded Host Seam

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want to open a supported plugin's native editor from Compose,
so that I can adjust plugin-specific controls without Compose taking over that responsibility.

## Acceptance Criteria

1. Given a supported VST plugin is available through the bounded native editor-host seam, when I invoke the supported open-editor action from Compose, then the plugin's native editor opens through the backend-owned host path, and Compose treats it as a secondary handoff surface rather than a primary playback workflow.
2. Given native editor support may be limited for some plugins or scenarios, when the editor cannot be opened or a supported interaction is unavailable, then Compose preserves overall playback usability, and the product reports the limitation without implying that core routing or inventory state is lost.
3. Given the native editor host seam has explicit lifecycle expectations, when the editor is opened, focused, resized, closed, or reopened, then the supported lifecycle behavior follows the bounded host design, and it does not expand scope into transport, routing, preset-management, or broader production-host ownership.

## Tasks / Subtasks

- [x] Task 1: Confirm and formalize the existing bounded score-context open-editor path instead of inventing a new UX surface (AC: 1)
  - [x] Audit the current score-context rack affordance in `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`, `src/services/actions.ts`, and the Rust command registration path to confirm whether the existing slot-level editor-open button already satisfies the supported user-facing entry point.
  - [x] Reuse the current Sequence score-context rack surface if it is already sufficient. Do not add extra buttons, duplicate launch points, or Plugin Manager driven editor-open UX just to literalize the acceptance criteria.
  - [x] If the current affordance needs refinement, keep the change minimal and score-contextual: one deliberate handoff action on the relevant routed slot, not a broader editor-management workflow.
  - [x] Preserve the rack-first model and slot identity semantics: editor-open must continue to resolve through the selected `slot_id` as well as plugin identity so same-bundle slots remain distinct backend-owned targets.

- [x] Task 2: Keep the editor host bounded and clearly secondary to Compose-owned playback workflows (AC: 1, 2, 3)
  - [x] Reuse the existing backend-owned host seam in `src-tauri/src/playback/plugin_editor_poc.rs`; this story should tighten or promote the supported contract around that seam, not build a second host architecture.
  - [x] Preserve the existing bounded-host policy posture from `src-tauri/src/playback/plugin_host_compatibility.rs` and the phase-aware `editor_host_validation` metadata in the plugin inventory contract.
  - [x] Ensure unsupported, blocked, or failed editor-host attempts remain local to the host action. They must not mutate routing truth, rack truth, inventory truth, or slot-owned runtime ownership beyond the supported bounded lifecycle behavior.
  - [x] Keep editor-host messaging framed as a limitation of the native editor seam, not as loss of the route, rack slot, or plugin inventory entry.

- [x] Task 3: Harden or preserve the explicit lifecycle contract without widening story scope (AC: 2, 3)
  - [x] Verify open, focus, resize, close, and same-session reopen continue through the existing backend lifecycle path and remain backend-owned rather than frontend-coordinated.
  - [x] Preserve bounded resize behavior: only treat arbitrary host-window resize as supported when the plugin editor reports resize capability; plugin-driven resize requests must continue to follow the current backend-owned rules.
  - [x] Keep the host seam macOS-only and bounded to supported current-session behavior. Do not broaden this story into transport integration, preset workflows, automation, multi-window orchestration, or universal plugin compatibility claims.
  - [x] If lifecycle fixes are required, keep them inside the owning Rust host seam and existing score-context entry point rather than adding compensating UI state or workaround flows in React.

- [x] Task 4: Lock the story down with focused tests and documentation updates (AC: 1, 2, 3)
  - [x] Extend existing frontend tests in `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` for the supported open action, disabled or hidden states for blocked targets, and failure messaging that preserves score-context usability.
  - [x] Extend Rust tests at the owning seams when host-policy or lifecycle behavior changes, especially in `src-tauri/src/playback/plugin_host_compatibility.rs` and the existing `plugin_editor_poc` test coverage.
  - [x] Update `docs/development-guide.md`, `TESTING.md`, and any other touched documentation so they describe the supported bounded editor-host handoff, the remaining limitations, and the fact that Plugin Manager remains the app-scoped inventory surface rather than the routine editor-launch workflow.
  - [x] Update `docs/project-context.md` or `docs/architecture.md` only if this story establishes a new enduring implementation rule or materially changes the bounded host contract.

## Dev Notes

### Story Intent

- Story 2.4 is not a greenfield native-host build. The repo already contains a substantial bounded editor-host seam plus a score-context rack entry point.
- The work here is to turn that existing seam into a clear, supported, minimal product path for opening a plugin editor from the right score-context surface.
- The user explicitly wants this kept intentionally minimal. Do not create UI for the sake of proving acceptance criteria separately.

### Story Foundation

- Epic 2 positions Story 2.4 after the inventory and picker work in Stories 2.1 through 2.3.
- The acceptance criteria are about bounded editor opening, limitation handling, and lifecycle correctness, not about expanding Compose into a production host or moving ordinary playback workflows into native plugin UI.
- UX planning says native plugin UI is a deliberate secondary handoff for plugin-specific depth, while Compose remains the primary shell for routing, readiness, inventory, and playback meaning.

### Brownfield Reality

- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx` already exposes a slot-level editor-open affordance through the cog button and routes the action through `actions.pluginInventory.openEditorPoc(plugin.id, { slot_id })`.
- `src/services/actions.ts` already bridges the frontend to the backend-owned `plugin_editor_poc_open` command.
- `src-tauri/src/playback/plugin_editor_poc.rs` already owns the host lifecycle path, backend window creation or focus, slot-target resolution, and native editor attachment behavior.
- `src-tauri/src/playback/plugin_host_compatibility.rs` already enforces a bounded unsupported-target list for the current seam.
- The repo is therefore ahead of the story technically. Treat Story 2.4 as a bounded integration and hardening pass, not as justification to add a new host surface or broaden Compose's plugin-host responsibilities.

### Previous Story Intelligence

- Story 2.3 deliberately kept score-context plugin selection compact and musician-friendly, and kept Plugin Manager app-scoped. Story 2.4 must preserve that boundary.
- The current Sequence rack panel is the score-context surface for slot preparation. Plugin Manager remains the app-scoped inventory browser and should not become the default native-editor launch workflow.
- Repo guidance also says not to add a user-facing native plugin-editor launch button to unrelated surfaces just because the host seam exists. If the existing rack action suffices, keep it.

### Git Intelligence Summary

- Recent Epic 2 commits show incremental tightening rather than feature sprawl:
  - `d1f64ab8` - `feat(2.3): cleanup plugin assignment ux`
  - `d1eebda1` - `feat(2.2): plugin refrest check`
  - `a1df6832` - `feat(2.1): plugin registry`
- Continue that pattern. Prefer refining the existing rack entry point, host seam, tests, and docs over introducing a new window or a second editor-launch flow.

### Scope Boundaries

- In scope:
  - Formalizing or tightening the existing score-context editor-open action.
  - Preserving clear limitation handling for blocked, failed, or unsupported editor-host attempts.
  - Lifecycle fixes inside the existing backend-owned host seam.
  - Focused tests and documentation updates for the supported bounded contract.
- Out of scope:
  - New inventory-management UX in Sequence or Plugin Manager.
  - New proof-oriented buttons, banners, panels, or editor-management dashboards.
  - Transport, playback mixing, preset management, automation, or broader DAW-style host behavior.
  - Universal plugin compatibility claims, multi-target hosting, or non-macOS support.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory truth, slot-owned runtime sessions, editor-host lifecycle, and failure semantics.
- Native editor attach must remain a transient view on the slot-owned runtime session, not the act that creates the canonical live plugin instance.
- Score-context surfaces may trigger the open action, but must not grow into a second host-state owner or lifecycle coordinator.
- Plugin Manager remains an app-scoped inventory surface. This story must not collapse the score-context versus app-scoped boundary.
- If any implementation touches persisted playback-domain structures or `.soloc` semantics, migration impact must be assessed and pending steps updated in `src-tauri/src/migrations/next.rs`. This story should avoid persistence changes unless strictly required by the existing slot-owned restore path.

### Technical Requirements

- Reuse `actions.pluginInventory.openEditorPoc()` as the existing FE to BE mutation seam unless there is a concrete reason to rename or narrow it.
- Preserve the `PluginInventoryEntry.editor_host_validation` contract in `src/services/types.ts` as the authoritative frontend-facing source for bounded editor-host posture.
- Keep blocked or failed editor-host posture separate from inventory validity. A discovered available bundle may still be unsupported for the bounded host seam.
- Preserve macOS-native backend window ownership for the host window. Do not replace it with a frontend-owned `WebviewWindow` flow.
- Slot-scoped editor opens must continue to target the selected slot identity so same-bundle slots remain distinct when reopened.
- If the current code or docs still overstate the feature as a generic host or a routine workflow, narrow that language to “bounded native editor handoff” and keep the product claim honest.

### File Structure Requirements

- Likely frontend seams:
  - `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - `src/services/actions.ts`
  - `src/services/types.ts`
- Likely backend seams:
  - `src-tauri/src/playback/plugin_editor_poc.rs`
  - `src-tauri/src/playback/plugin_host_compatibility.rs`
  - `src-tauri/src/playback/plugin_inventory.rs`
  - `src-tauri/src/lib.rs` only if command wiring changes
- Likely documentation seams:
  - `docs/development-guide.md`
  - `TESTING.md`
  - `docs/architecture.md`
  - `docs/project-context.md`

### UI And Accessibility Guidance

- Prefer the existing score-context rack button if it already communicates the action adequately. Do not add more UI unless there is a user-facing gap that the current control cannot cover.
- The editor-open action should read as a deliberate deeper handoff, not as the default solution for routine routing or readiness issues.
- Any error or limitation text must preserve conceptual clarity: the route still exists, the slot still exists, and the inventory entry may still be valid even when the bounded editor host cannot attach.
- Do not move this action into Plugin Manager just because that window is app-scoped. The inventory browser and the score-context editor handoff have different responsibilities.

### Testing Requirements

- Frontend tests should cover:
  - the supported score-context open action on a valid slot-backed available plugin,
  - disabled or hidden editor-open affordances for blocked targets,
  - local failure reporting that does not imply route or inventory loss,
  - any minimal wording or accessibility updates around the bounded handoff.
- Extend existing suites first:
  - `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
  - `src/windows/main/components/plugins.test.tsx` only if user-facing bounded-host wording changes there
- Rust tests should stay at the owning seam:
  - `src-tauri/src/playback/plugin_host_compatibility.rs`
  - existing `plugin_editor_poc` tests
- Validate touched code with the standard project commands relevant to the final change set:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo test --manifest-path src-tauri/Cargo.toml --quiet plugin_editor_poc` and `cargo clippy --lib --manifest-path src-tauri/Cargo.toml -- -D warnings` if Rust changes

### Documentation Impact

- Update the following docs if the implementation or terminology changes materially:
  - `docs/development-guide.md`
  - `TESTING.md`
  - `docs/architecture.md`
  - `docs/project-context.md`
- Documentation must keep three boundaries explicit:
  - Compose owns routing, readiness, and inventory meaning.
  - Native plugin UI is a secondary bounded handoff for plugin-specific internals.
  - Plugin Manager remains the app-scoped inventory browser, not a catch-all editor-host workflow.

### References

- `_bmad-output/planning-artifacts/epics.md` - Story 2.4 acceptance criteria and Epic 2 sequencing.
- `_bmad-output/planning-artifacts/ux-design-specification.md` - Sections 2.5 and 3.1 to 3.7 on score-context routing, panels versus windows, and native plugin UI as a deliberate secondary handoff.
- `_bmad-output/planning-artifacts/architecture.md` - Rust-owned playback truth, slot-owned runtime sessions, and bounded native host direction.
- `docs/project-context.md` - project-wide implementation rules for slot-owned sessions, app-scoped Plugin Manager scope, and score-context rack surfaces.
- `docs/development-guide.md` - current bounded host behavior, limitations, and manual validation workflow.
- `TESTING.md` - runtime-activation and native editor-host validation expectations.
- `_bmad-output/implementation-artifacts/2-3-expose-musician-friendly-plugin-selection-inputs.md` - previous-story guidance on keeping score-context UI compact and avoiding Plugin Manager duplication.
- `/memories/repo/plugin-editor-host-session-lifecycle.md` - backend lifecycle lessons for attach, resize, close, and same-session reopen behavior.
- `/memories/repo/plugin-manager-window-scope.md` - app-scoped Plugin Manager boundary and no-duplicate-browser rule.
- `/memories/repo/playback-rack-first-model.md` - rack-first ownership model and slot versus route semantics.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.4`.
- Sprint status showed `2-4-open-supported-native-plugin-editors-through-a-bounded-host-seam` in `backlog` before story creation.
- Planning artifacts define Story 2.4 as a bounded native-editor handoff story, not a broader plugin-host or playback-workflow expansion.
- Repo inspection confirmed the current brownfield seams already include a score-context rack editor-open affordance, a backend-owned host command, a bounded unsupported-target policy seam, and supporting tests and docs.
- The highest implementation risk is scope creep: adding new UI, widening Plugin Manager responsibility, or treating the native editor as a primary playback workflow instead of a deliberate secondary handoff.
- Existing docs and code already treat the host seam as bounded and macOS-only; Story 2.4 should preserve that honesty while making the supported path clearer and more durable.
- Audited `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`, `src/services/actions.ts`, `src-tauri/src/playback/plugin_editor_poc.rs`, and `src-tauri/src/playback/plugin_host_compatibility.rs`; the existing slot-local rack action and slot-scoped backend target resolution already satisfied the intended entry point and bounded lifecycle contract.
- The discarded rack-panel wording refinement was not kept. Final story closure relies on the existing score-context editor-open button, existing local failure alert, and the backend-owned bounded host seam already present in the codebase.
- Validation re-run on the final codebase completed with `npm run check-types`, `npm run lint`, and full `npm run test:js` passing. Full JS output still includes pre-existing React `act(...)` warnings and a mocked `setZoom` stderr trace in unrelated suites, but no failing tests.

### Completion Notes List

- Created the Story 2.4 implementation guide with explicit minimal-scope guardrails.
- Directed the developer to reuse the current Sequence rack entry point and backend host seam unless a concrete user-facing gap requires a small refinement.
- Captured the distinction between inventory validity and bounded editor-host support so blocked or failed editor opens do not imply route or inventory loss.
- Closed the story against the existing code path: the Sequence rack still provides the slot-local editor-open action, `actions.pluginInventory.openEditorPoc(...)` still bridges to the backend command, and the Rust host seam remains slot-scoped and bounded.
- Kept the documentation changes that match the current codebase and removed implementation-record claims that depended on discarded UI wording changes.
- Re-ran the standard frontend validation commands on the final codebase before marking the story done.

### File List

- `_bmad-output/implementation-artifacts/2-4-open-supported-native-plugin-editors-through-a-bounded-host-seam.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/development-guide.md`
- `TESTING.md`
- `docs/architecture.md`
- `docs/project-context.md`

## Change Log

- 2026-03-26: Created the Story 2.4 implementation guide and moved sprint tracking to `ready-for-dev`.
- 2026-03-26: Formalized the existing Sequence rack native-editor handoff, tightened blocked/failure messaging, updated bounded-host documentation, and moved the story to `review`.
- 2026-03-26: Aligned the documentation and story record to the existing codebase after discarding the draft rack-panel wording change, then marked the story `done`.
