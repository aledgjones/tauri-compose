# Story 2.3: Expose Musician-Friendly Plugin Selection Inputs

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want score workflows to show only usable plugin choices in a clear musical form,
so that I can choose playback options without dealing with raw host-inventory complexity.

## Acceptance Criteria

1. Given Compose has a backend-owned plugin inventory with health status, when a score-context routing surface needs plugin choices, then it receives a filtered view of currently usable plugins and targets, and unavailable or invalid inventory entries are not presented as ordinary ready-to-route choices.
2. Given inventory management is app-scoped but routing is score-context, when I need more information about missing or unavailable options, then the score-context workflow continues to rely on the existing app-scoped Plugin Manager path, and it does not duplicate the full inventory-management workflow locally.
3. Given the rack-first model is the canonical playback mental model, when plugin options are presented in contextual pickers, then they are framed as usable musical playback choices rather than host-topology internals, and the UI stays compact rather than expanding into proof-oriented control sprawl.

## Tasks / Subtasks

- [x] Task 1: Audit the current brownfield routing surfaces before changing behavior (AC: 1, 2, 3)
  - [x] Trace the existing Sequence score-context seams in `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` and `src/windows/main/screens/sequence/components/playback-rack-panel.tsx` to confirm which picker already consumes rack slots, which picker already consumes plugin inventory, and where current labels or filters drift from Story 2.3.
  - [x] Confirm the backend-owned inventory and route-choice inputs already available through `usePluginInventoryState()`, `route_target_options_by_slot_id`, and `ResolvedPlaybackRoute` before adding any new frontend state or commands.
  - [x] Document any current repo drift against the story, with specific attention to score-context wording, current unusable-option filtering, and whether the score-context surface duplicates any Plugin Manager responsibilities.
- [x] Task 2: Tighten the score-context choice contract without creating a second inventory model (AC: 1, 3)
  - [x] Reuse Rust-owned plugin inventory truth from `src-tauri/src/playback/plugin_inventory.rs` and Rust-owned route-choice truth from `route_target_options_by_slot_id`; do not introduce a frontend-owned cache, duplicate filtering model, or `.soloc` persistence for machine-local inventory state.
  - [x] Keep ordinary ready-to-route choices limited to backend-valid options. If current filtering is inferred from the wrong frontend fields, extend or reuse the backend-owned contract rather than layering more local heuristics onto React components.
  - [x] Do not conflate editor-host posture with routing usability. `editor_host_validation` is adjacent inventory metadata for bounded native-editor hosting; if routing needs a stricter usability signal, expose that through the authoritative backend-owned picker contract instead of treating editor-host limits as the routing truth by default.
  - [x] Preserve the current single-target-first boundary. If a slot has more than one backend-provided `PlaybackRouteChoice`, expose only the supported bounded choice UX for this story and do not invent speculative multi-target, part-routing, or host-topology controls.
- [x] Task 3: Make the picker language musician-friendly and keep escalation app-scoped (AC: 1, 2, 3)
  - [x] Present plugin and target choices in rack-first, musical language using the existing score-context surfaces; avoid raw ids, host-topology terminology, or explanatory control sprawl.
  - [x] Reuse the existing `Select`-based compact picker patterns where they remain sufficient; if a richer filtered picker is genuinely required, keep it bounded to the current musical task and preserve keyboard and labeling accessibility.
  - [x] Keep escalation app-scoped: reuse the canonical Plugin Manager workflow when deeper inventory inspection is needed, and do not embed the full inventory browser, refresh workflow, or management affordances inside the Sequence score-context surfaces.
  - [x] Preserve the current separation of responsibilities: Sequence row and bottom-panel surfaces are score-contextual, while Plugin Manager remains the app-scoped inventory window.
- [x] Task 4: Lock the story down with focused tests, docs, and contract checks (AC: 1, 2, 3)
  - [x] Extend frontend tests around the Sequence routing surfaces in `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` and `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` for filtered usable choices, compact musical labels, and current-selection preservation.
  - [x] Add or update tests in `src/services/actions.test.ts` and `src/services/state/provider.test.tsx` if Story 2.3 changes how the existing Plugin Manager launch seam is invoked from score-context flows.
  - [x] Add or update Rust tests only if the authoritative inventory or route-choice contract changes; keep those regressions at the owning seam instead of creating a cross-cutting picker harness.
  - [x] Update developer-facing docs to describe the score-context picker boundary, the app-scoped Plugin Manager escalation path, and any clarified routing-usability contract.

## Dev Notes

### Story Intent

- Story 2.3 is a brownfield refinement story, not a greenfield routing implementation.
- The repo already contains bounded score-context routing surfaces in Sequence and an app-scoped Plugin Manager window. The work here is to make those score-context inputs clearer, more musician-friendly, and correctly filtered against backend-owned truth.
- The goal is not to expose the full plugin inventory inside score context. The goal is to let composers make the next useful routing choice quickly while keeping the deeper inventory workflow app-scoped in Plugin Manager.

### Story Foundation

- Epic 2 positions Story 2.3 between the app-scoped inventory work in Story 2.1 and bounded native editor work in Story 2.4.
- The three acceptance criteria center on: filtered usable choices, app-scoped inventory ownership, and compact rack-first framing.
- UX planning explicitly states that routing should feel like choosing a usable sound path for a musical part, not configuring a host graph.

### Brownfield Reality

- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx` already shows score-context slot cards with a compact plugin picker sourced from backend-owned plugin inventory.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` already exposes the bounded route-authoring picker for current score instruments by selecting a slot from the score rack or switching back to `--`.
- `src/services/actions.ts` already owns the current slot and route mutation commands, and the broader app still owns the canonical Plugin Manager window-launch path.
- `src/services/state/provider.tsx` already listens for `soloc://menu/plugin-manager`, so any future escalation must reuse the same app-scoped seam rather than inventing a second launch path.
- The repo is therefore partially ahead of Story 2.3. Preserve the working seams and close only the gaps around filtering correctness, language, and compactness without duplicating Plugin Manager workflow inside Sequence.

### Previous Story Intelligence

- Story 2.2 confirmed that plugin inventory is machine-local backend-owned truth and that manual refresh remains an explicit Plugin Manager workflow. Do not move refresh or inventory ownership into Sequence score-context surfaces.
- Story 2.2 also confirmed the canonical app-scoped inventory scope: Plugin Manager remains the place for browsing the full inventory, health, and refresh behavior, while score-context UI should consume backend-owned truth without becoming a second inventory browser.
- Repo memory reinforces that Sequence instrument rows and the Sequence bottom panel are the score-context surfaces for playback relationships, while Plugin Manager stays app-scoped and should not be silently replaced by older sampler-era window scaffolding.

### Git Intelligence Summary

- Recent commits show Epic 2 work already landed in the brownfield seams rather than in isolated prototypes:
  - `d1eebda1` — `feat(2.2): plugin refrest check`
  - `a1df6832` — `feat(2.1): plugin registry`
- Treat Story 2.3 as an extension and tightening pass over those existing seams, not as justification to relocate the workflow or introduce a second selection model.

### Scope Boundaries

- In scope:
  - Filtered, backend-truth-driven plugin and target choices in score-context routing surfaces.
  - Musician-friendly labels and compact choice presentation.
  - Preserving the app-scoped Plugin Manager boundary while keeping score-context pickers focused on the current musical task.
  - Focused frontend and seam-local contract tests for those behaviors.
- Out of scope:
  - Full inventory browsing, rescan, or plugin health management inside Sequence.
  - New machine-local inventory persistence behavior or `.soloc` ownership changes.
  - Native editor-host expansion owned by Story 2.4.
  - Broad multi-target or multi-part routing UX beyond the bounded backend-proven contract.

### Architecture Compliance

- Rust remains the sole authoritative owner of plugin inventory truth, route choice truth, resolved readiness state, and persistence semantics.
- Score-context surfaces must consume backend-owned inventory and route-choice state without inventing parallel frontend-owned routing truth.
- The rack-first model remains canonical: racks own slot lifecycle, slots own plugin assignment or unassignment, and instruments own routes into existing slots.
- Plugin Manager remains an app-scoped window. Sequence rows and bottom-panel surfaces remain score-context panels and do not embed inventory-management affordances.
- If any implementation changes persisted playback-domain structures or `.soloc` behavior, migration impact must be assessed and pending steps updated in `src-tauri/src/migrations/next.rs`. This story should not need persistence changes, so treat any such change as exceptional and justify it explicitly.

### Technical Requirements

- Keep the authoritative plugin inventory seam in `src-tauri/src/playback/plugin_inventory.rs` and its emitted frontend contract in `src/services/types.ts` synchronized if new routing-usability metadata is added.
- Reuse the existing frontend mutation seams:
  - `actions.score.playback.slot.assignPlugin()` / `unassignPlugin()`
  - `actions.score.instrument.assignSlotRoute()` / `unassignRoute()`
- Do not revive legacy `score_instruments_set_plugin_route`-style ownership or sampler-era terminology.
- If the current picker contract cannot distinguish routing-usable versus merely discovered plugins cleanly enough, extend backend-owned metadata rather than hard-coding more UI-only exclusion logic.
- Preserve the existing `PlaybackRouteChoice.label` and `route_target_options_by_slot_id` backend contract as the authority for target wording. React must not invent speculative part or MIDI-channel options.
- Preserve compact score-first phrasing. Users should understand “this instrument routes to this playable choice” without needing host-engine jargon.

### File Structure Requirements

- Likely frontend seams:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - `src/services/actions.ts`
  - `src/services/types.ts`
  - `src/services/state/provider.tsx` only if score-context behavior needs to react to existing app-scoped Plugin Manager events
- Likely backend seams only if the authoritative picker contract must change:
  - `src-tauri/src/playback/plugin_inventory.rs`
  - The Rust score/playback state emitter that populates `route_target_options_by_slot_id` and resolved route state
- Do not repurpose the legacy plugin-settings window scaffold into another inventory browser.

### UI And Accessibility Guidance

- Prefer the existing compact `Select` pattern when it is sufficient. Story 2.3 does not justify a new dependency or a full custom browser just to make the choices look more sophisticated.
- If a custom filtered picker becomes necessary, keep it explicitly bounded to the current task and follow current accessibility guidance:
  - React controlled select inputs require a stable `value` plus synchronous `onChange` updates.
  - Pickers need an accessible label.
  - If a true editable combobox is introduced, it must implement correct popup state, focus management, keyboard behavior, and labeling rather than approximating those semantics.
- Do not add decorative controls or explanatory text that turns the Sequence surface into a proof UI.

### Testing Requirements

- Frontend tests should cover:
  - filtering out invalid or unavailable ordinary choices,
  - preserving the current selection when a route still exists but is non-ready,
  - compact, musician-friendly labels in score-context pickers,
  - any new empty or degraded state wording introduced by the story.
- Extend existing suites first:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
  - `src/services/actions.test.ts`
  - `src/services/state/provider.test.tsx`
- Add Rust tests only when the backend-owned routing-usability or route-choice contract changes.
- Validate touched code with the standard project commands relevant to the final change set:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo test --manifest-path src-tauri/Cargo.toml --quiet` and `cargo clippy --lib --manifest-path src-tauri/Cargo.toml -- -D warnings` if Rust changes

### Documentation Impact

- Update the following docs if implementation or wording changes materially:
  - `docs/development-guide.md`
  - `docs/data-models.md`
  - `docs/component-inventory.md`
  - `docs/project-context.md` if Story 2.3 establishes a new enduring rule for routing-usability filtering or score-context versus app-scoped boundaries
  - `docs/architecture.md` if the backend-owned picker contract changes materially
- Documentation must continue to state that full inventory management is app-scoped, score-context pickers consume backend-owned truth, and the rack-first model remains the mental model.

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 2.3 acceptance criteria and Epic 2 sequencing.
- `_bmad-output/planning-artifacts/ux-design-specification.md` — Sections 3.4 to 3.7 on contextual plugin selection, panels versus windows, and compact score-first escalation.
- `_bmad-output/planning-artifacts/architecture.md` — Rust ownership, score-context versus app-scoped boundaries, and persistence rules.
- `docs/project-context.md` — project-wide implementation rules for rack-first playback, Rust ownership, and compact workflow-specific UI.
- `docs/development-guide.md` — current score-context rack and route workflow guidance.
- `docs/data-models.md` — ownership split for rack slots, plugin targets, routes, and resolved route state.
- `_bmad-output/implementation-artifacts/2-2-support-cached-discovery-and-manual-rescan.md` — previous-story learnings about inventory ownership, refresh boundaries, and Plugin Manager scope.
- `/memories/repo/plugin-manager-window-scope.md` — app-scoped Plugin Manager boundary and no-duplicate-browser rule.
- `/memories/repo/plugin-inventory-startup-cache.md` — backend-owned inventory truth and manual refresh boundary.
- `/memories/repo/playback-rack-first-model.md` — rack-first ownership model and route semantics.
- React `<select>` reference — controlled select requirements and labeling guidance.
- WAI-ARIA Authoring Practices combobox guidance — use native/select-only semantics when sufficient, and only introduce full combobox behavior with the required keyboard and popup semantics.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.3`.
- Sprint status showed `2-3-expose-musician-friendly-plugin-selection-inputs` in `backlog` before story creation.
- Planning artifacts define Story 2.3 as a score-context filtering and presentation story, not a Plugin Manager expansion or a broader routing-architecture rewrite.
- Repo inspection confirmed the current brownfield seams already include Sequence score-context slot and route controls, Plugin Manager app-scoped inventory ownership, and backend-owned inventory emission.
- The remaining likely implementation risk is contract drift between “inventory is present” and “choice is usable in the current score workflow”; Story 2.3 must keep the score-context picker bounded to the approved usable-plugin contract without growing a second inventory-management surface.
- Audit result: `instrument-overview-row.tsx` already consumed rack slots and backend route choices, and the symbolic `--` unrouted state remains the intended compact score-context wording. `playback-rack-panel.tsx` already consumed backend plugin inventory, but its ordinary picker filtering incorrectly treated `editor_host_validation` as routing truth.
- Backend contract check: the existing Rust-owned `route_target_options_by_slot_id` and `ResolvedPlaybackRoute` seams were sufficient for Story 2.3, so no Rust, migration, or `.soloc` persistence changes were required.
- Validation run completed with `npm run lint`, `npm run check-types`, and `npm run test:js` all passing after the Story 2.3 changes.
- User follow-up refinement: unavailable plugins and editor-host-blocked plugins must stay out of the rack-panel picker itself, the rack panel should stay minimal rather than growing extra escalation UI, and the compact instrument-row `--` wording stays intact.

### Completion Notes List

- Tightened the rack-panel picker to treat usable plugins as entries that are both inventory-available and not editor-host blocked.
- Kept unavailable and editor-host-blocked plugins out of the ordinary score-context picker while leaving inventory browsing and refresh in the separate app-scoped Plugin Manager workflow.
- Kept the instrument-row unrouted picker in its compact `--` form and refreshed the developer docs for the clarified score-context boundary.
- No `actions.test.ts`, `provider.test.tsx`, or Rust test updates were required because the existing app-scoped boundaries and Rust-owned contracts were reused unchanged.

### File List

- `_bmad-output/implementation-artifacts/2-3-expose-musician-friendly-plugin-selection-inputs.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/component-inventory.md`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`

## Change Log

- 2026-03-26: Created the Story 2.3 implementation guide and moved sprint tracking to `ready-for-dev`.
- 2026-03-26: Implemented score-context routing-usability filtering, kept the compact `--` row picker wording, kept unavailable or editor-host-blocked plugins out of the rack-panel picker, added focused Sequence picker tests, aligned the docs to the final minimal score-context UI, and moved the story to `done`.
