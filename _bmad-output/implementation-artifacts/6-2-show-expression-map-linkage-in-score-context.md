# Story 6.2: Show Expression-Map Linkage In Score Context

Status: ready-for-dev

## Story

As a composer,
I want to see whether my current routed instrument has an expression map,
so that I can understand and adjust the current score-context linkage without leaving the instrument workflow.

## Acceptance Criteria

1. Given an instrument is associated with a playback route, when I inspect the instrument overview row, then I can see which Expression Map is linked to that route through a compact row control, and `--` remains the truthful unassigned state.
2. Given Expression Maps are reusable assets rather than selection-owned data, when linkage is displayed in score context, then the instrument row shows only the current route relationship and reassignment control, and it does not imply that the row or bottom panel owns the Expression Map definition itself.
3. Given a linked map may need deeper work, when I use the instrument overview row, then Compose provides an adjacent edit affordance that opens the selected map in an app-scoped window shell, and that shell remains intentionally minimal in this story so Story 6.3 can expand it into the real editor and library workflow.

## Tasks / Subtasks

- [ ] Task 1: Extend the current instrument-row contract to carry Expression Map linkage alongside playback route state (AC: 1, 2)
  - [ ] Add or extend backend and shared frontend payloads so the instrument overview row can resolve the currently assigned Expression Map display name or the truthful unassigned `--` state.
  - [ ] Keep the linkage contract aligned with the route-address-aware backend model from Story 6.1 while surfacing it as the route's Expression Map in MVP UI.
  - [ ] Preserve the current backend ability to distinguish richer linkage conditions if needed later, but collapse the user-facing row state to assigned-name or `--` only.

- [ ] Task 2: Add a compact row-level Expression Map dropdown without changing the overview row's structural feel (AC: 1, 2)
  - [ ] Add an inline dropdown in `InstrumentOverviewRow` using the same compact interaction language as the existing slot selector.
  - [ ] Populate the dropdown from the current reusable Expression Map library and include `--` as the clear option.
  - [ ] Keep the control bounded to the row only; do not add Expression Map linkage to the bottom `PlaybackRackPanel` in this story.
  - [ ] Preserve the current instrument-overview height and alignment expectations so the row remains visually matched with the overview track layout rather than growing into a second-tier panel.

- [ ] Task 3: Support assignment, reassignment, and clearing through the row control only (AC: 1, 2)
  - [ ] Selecting a map from the dropdown assigns that Expression Map to the current route linkage seam.
  - [ ] Selecting `--` clears the linked map and returns playback to the current direct plugin path with no extra guarantee of musically sensible behavior beyond today's default behavior.
  - [ ] A linked map with zero rules still displays as assigned by name; assignment truth is separate from whether later playback behavior is rich.
  - [ ] If a previously referenced map no longer exists, resolve the row truthfully to `--` rather than inventing warning-heavy recovery UI.

- [ ] Task 4: Add a row-adjacent edit button that opens a minimal selected-map window shell (AC: 3)
  - [ ] Add a compact cog-style button adjacent to the Expression Map dropdown, matching the current plugin-row control language.
  - [ ] Keep the button disabled when the row shows `--`.
  - [ ] Opening the button should launch or focus a dedicated app-scoped Expression Map window shell for the currently assigned map, but that shell should remain intentionally minimal in Story 6.2.
  - [ ] Do not broaden the shell into full map authoring, browse/import/export workflows, or full library management; Story 6.3 owns that expansion.

- [ ] Task 5: Prove the row UX boundaries and sequencing contract with seam-local tests and docs (AC: 1, 2, 3)
  - [ ] Add or extend frontend tests around `InstrumentOverviewRow` to prove assigned-name display, `--` clearing, disabled edit button when unassigned, and minimal-window launch when assigned.
  - [ ] Add or extend backend or shared-contract tests only as needed to prove row payload truth for assigned and unassigned linkage states.
  - [ ] Update documentation to record that Story 6.2 adds row-level linkage selection and a minimal selected-map shell, while Story 6.3 still owns the full Expression Map editor and library workflow.

## Dev Notes

### Story Intent

- Story 6.2 is the first user-visible Expression Map surface, but it must stay narrow.
- The core outcome is not editing maps. The core outcome is that the instrument overview row can show and change which Expression Map is currently assigned to the route, using the same compact score-context language already established for routing.
- The minimal selected-map window shell exists only to preserve the app-scoped ownership model. It is not the full editor yet.

### User Model Clarified

- Expression Map linkage belongs in score context, but not in the bottom rack panel. The row is the right level because it sits in the user's mental space between the instrument and its current routed playback behavior.
- The row should show the current map name or `--` only. Richer backend linkage nuance is valid for product evolution, but it is noise in MVP UI.
- The row control is both state display and assignment mechanism, just like the existing route dropdown.
- Clearing to `--` means Compose stops applying Expression Map trigger mapping and falls back to the current direct plugin path. The product should not promise that unassigned playback remains musically sensible beyond current behavior.
- A map with zero rules is still a real assigned map. Assignment truth and playback richness are not the same thing.
- The cog button exists to open the selected map at the app level, reinforcing that Expression Maps are reusable assets outside the score selection itself.

### Previous Story Intelligence

- Story 6.1 established the durable Expression Map model, route-address-aware linkage seam, app-scoped library ownership, and quiet `--` handling for missing references.
- Story 5.5 reinforced the repo rule that backend truth may stay richer than user-facing warning chrome. 6.2 should preserve that same discipline by collapsing the row UI to assigned-name or `--`.
- Story 3.2 established the current Sequence row versus rack-panel boundary. Route assignment stays in the row, while the rack panel owns slot-level context. Expression Map linkage should follow the row-side pattern rather than expanding the rack panel into a second ownership surface.

### This Story Owns

- Row-level Expression Map visibility for the current instrument route.
- Row-level assignment, reassignment, and clearing through a compact dropdown.
- A disabled-when-unassigned cog button that opens a minimal selected-map app window shell.
- The MVP user-facing collapse from richer backend linkage semantics down to assigned-name or `--`.
- Regression proof that the instrument row keeps its compact structure while adding the new controls.

### This Story Does Not Own

- Bottom-panel Expression Map surfaces. Do not add linkage UI to `PlaybackRackPanel` in this story.
- Full Expression Map authoring, map creation, rename, duplicate, delete, import/export, browse/library management, or deep editor UI. Story 6.3 owns those workflows.
- Audible playback changes. Story 6.4 owns the first playback effect.
- A broad user-facing ready, missing, incomplete taxonomy. Those distinctions stay backend-capable but do not belong in the MVP row UI.
- Factory maps, auto-assignment, compatibility heuristics, or plugin-smart defaults.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Canonical compact row surface for route assignment and adjacent plugin-level controls. This is the main 6.2 UI seam.
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - Existing regression home for compact row semantics and adjacent buttons.
- `src/services/actions.ts`
  - Current home of app-scoped window-opening helpers such as `openPluginManager`; reuse this pattern for the minimal Expression Map shell.
- `src/services/types.ts`
  - Shared payload and type mirror that must stay aligned with Rust if route linkage data is exposed to the row.
- `src-tauri/src/score/playback.rs`
  - Owns persisted route and route-address model that the UI linkage should project from.
- `src-tauri/src/files.rs`
  - Owns truthful load behavior when linked map references are missing.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Important non-target seam for 6.2. This story should avoid adding Expression Map ownership cues there.

### Implementation Guidance

- Treat the new dropdown as a sibling to the route selector, not a new panel or explanatory block. The row must stay compact and aligned with the overview track height.
- Prefer extending existing row-control patterns over inventing a new bespoke layout language.
- Keep the user-facing label stable and explicit: `Expression Map`.
- The dropdown should list currently available reusable maps by display name and include `--` to clear.
- If the currently linked map cannot be resolved from the app library, display `--` and disable the edit button.
- The cog button should open a dedicated Expression Map window label/instance keyed for selected-map work, but the content can be a bounded shell or placeholder surface that clearly preserves app-level ownership for Story 6.3 to build on.
- Do not add Expression Map controls to the rack panel or imply that maps are slot-owned objects, even though the backend linkage seam is route-address aware.
- Avoid wrapping the row onto an additional visual tier or increasing its apparent density enough to break alignment with the overview track.

### Risks And Regression Traps

- The largest UX trap is leaking too much backend nuance into the row. Assigned-name or `--` is the intended user-facing contract here.
- Another trap is placing the control in the bottom panel because the data model references slot address. That would imply the wrong ownership level to users.
- A third trap is accidentally pulling full editor or library scope into the 6.2 cog action. Keep the window intentionally shallow until Story 6.3.
- Layout drift is a real product risk. If the row grows vertically or visually stops matching the overview track rhythm, the compact Sequence surface degrades.
- Clearing an assigned map must remain truthful and quiet. Do not introduce extra warning banners, recovery badges, or fallback language beyond `--`.

### Testing Requirements

- Extend `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` for:
  - assigned map name display;
  - `--` as the clear option;
  - clearing behavior through the dropdown;
  - disabled cog button when no map is assigned;
  - selected-map window-shell launch when a map is assigned.
- Add or extend frontend tests only as needed to confirm the row control does not introduce unexpected warning chrome or secondary panel behavior.
- Add shared-contract or backend tests only where required to prove the row receives truthful assigned or unassigned linkage data.
- Recommended validation commands once implemented:
  - `npm run test:js`
  - `npm run check-types`
  - `npm run lint`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

### Documentation Impact

- Update docs when the first user-visible Expression Map linkage surface lands.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
- Document explicitly that:
  - the first visible Expression Map control lives in the Sequence instrument row;
  - the bottom rack panel still does not own Expression Map UI;
  - the row exposes assignment by dropdown plus a disabled-when-unassigned edit button;
  - the 6.2 window is a minimal selected-map shell, not yet the full editor/library workflow.

### Architecture Compliance

- Rust remains the owner of linkage truth and persistent route-address-aware map references.
- The frontend row is a compact projection of that backend truth, not an independent ownership surface.
- App-scoped windows remain the place for reusable asset ownership. The 6.2 cog action exists to preserve this boundary, not to blur it.
- Do not expand the rack panel into an Expression Map management surface.
- Keep frontend and backend contracts aligned if linkage payloads or window state are added.

### Library And Framework Requirements

- Stay on the repo baselines already established in the project context: Rust 2021, Tauri 2.10.1, React 19.2.4, and TypeScript 5.9.3.
- Reuse existing row control patterns, `actions.ts` window-opening conventions, and shared Rust/TypeScript payload mirrors.

### File Structure Notes

- Row UI changes belong in `src/windows/main/screens/sequence/components/`.
- New selected-map shell window scaffolding should follow the existing multi-window structure under `src/windows/` and the app-scoped HTML entry pattern already used elsewhere.
- Shared linkage payload changes belong in the existing Rust/TypeScript score and services seams.
- Keep tests close to the row component and shared seam owners; do not invent a new cross-cutting Expression Map test harness for this story.

### Git Intelligence Summary

- The current row already combines compact dropdown-based route assignment with an adjacent plugin cog button. Story 6.2 should follow that established interaction pattern instead of adding a new linkage layout.
- Existing app-scoped window launch patterns are already present in `actions.ts`, so the minimal selected-map shell should reuse those conventions and labels.

### Latest Technical Information

- No external-library change is required for 6.2. The important technical constraint is local: keep Sequence row UX narrow and aligned with the repo's established playback-readiness presentation rules.
- The current codebase already demonstrates the “rich backend, narrow UI” pattern for playback routes. Expression Map linkage should follow that same contract rather than creating a second, noisier UX model.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 6 and Story 6.2 baseline direction.
- `_bmad-output/implementation-artifacts/6-1-model-expression-maps-as-durable-playback-assets.md`
  - Durable Expression Map model, quiet missing-reference handling, and route-address-aware linkage foundations.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Panel-versus-window ownership model and Expression Map relationship guidance.
- `_bmad-output/planning-artifacts/architecture.md`
  - App-scoped reusable asset windows and backend-owned state flow.
- `/memories/repo/playback-readiness-ui-semantics.md`
  - Narrow user-facing state presentation contract.
- `/memories/repo/sequence-rack-panel-story-3-2.md`
  - Current row-versus-rack-panel ownership boundary in Sequence.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Existing row dropdown and adjacent-button interaction language.
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - Existing regression surface for row behavior.
- `src/services/actions.ts`
  - Existing app-scoped window launch pattern.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-05: Story prepared after reviewing Story 6.1, Epic 6 planning, current Sequence row seams, row-versus-rack-panel repo memory, and current app-scoped window-opening patterns.
- 2026-04-05: User clarification locked the 6.2 UX boundaries: row only, compact dropdown assignment, `--` as the truthful clear state, disabled cog when unassigned, and a minimal selected-map shell that Story 6.3 will deepen.

### Completion Notes List

- Story context created and sprint status updated to ready-for-dev.
- Story 6.2 stays intentionally narrow: row-level linkage display and assignment plus a minimal selected-map shell only.
- Bottom-panel Expression Map UI and richer state taxonomies were explicitly excluded to preserve the intended ownership model and avoid UX noise.
- Story 6.3 remains the owner of the real editor and library workflow.

### File List

- `_bmad-output/implementation-artifacts/6-2-show-expression-map-linkage-in-score-context.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
