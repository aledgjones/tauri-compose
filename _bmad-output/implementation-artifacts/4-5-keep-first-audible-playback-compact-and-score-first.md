# Story 4.5: Keep First Audible Playback Compact And Score-First

Status: done

## Story

As a composer,
I want the first audible playback workflow to stay compact and tied to score-context controls,
so that hearing the score feels like part of composition instead of a detour into playback tooling.

## Acceptance Criteria

1. Given first audible playback is being proved through score-context auditioning and related controls, when the UI for this epic is implemented, then the useful workflow is available through compact score-context controls, and the design avoids extra explanatory surfaces or duplicate controls that exist mainly to prove acceptance criteria.
2. Given deeper inventory and plugin editing already belong to app-scoped windows, when first audible playback is used, then the core audible workflow stays in the score context, and it escalates outward only when a real user need requires it.
3. Given Compose uses a custom compact design language, when first audible playback surfaces are added or refined, then they preserve the existing panel and window grammar, and readiness and control states remain legible without relying on color alone.

## Tasks / Subtasks

- [x] Task 1: Audit and refine the current score-context audible workflow around existing Sequence surfaces (AC: 1, 2, 3)
  - [x] Start from the current mature score-context seams in `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`, `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`, `src/windows/main/screens/sequence/components/playback-readiness.tsx`, `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`, `src/windows/main/components/mixer.tsx`, `src/windows/main/components/mixer-channel.tsx`, `src/windows/main/components/transport.tsx`, and `src/windows/main/utils/audition.ts` rather than inventing a new playback workspace.
  - [x] Keep the core loop compact: understand route/readiness, adjust only the bounded supported control set already proven by backend `supported_mix_controls`, audition from score interaction, and start supported score playback from the same score-first context.
  - [x] Remove or avoid explanatory or duplicate controls whose main purpose is to demonstrate acceptance criteria literally rather than solve a real user need.
  - [x] Preserve the current direct mental model that a score instrument routes to a playback destination; do not teach plugin-host topology as the user workflow.
- [x] Task 2: Preserve the architectural boundary between score-context workflow and app-scoped escalation surfaces (AC: 1, 2)
  - [x] Keep Plugin Manager as the canonical app-scoped inventory browser, while preserving the score-context rack editor action as the bounded secondary handoff for native plugin editing rather than making either surface the default home of first audible playback.
  - [x] Ensure any escalation from score context happens only for a real workflow need such as browsing inventory or opening a plugin’s native editor, not because the core audible path is incomplete, duplicated, or cluttered.
  - [x] Avoid duplicating route, readiness, or mix controls across score-context and app-scoped windows just to make the system feel “complete.”
  - [x] Keep the top bar limited to universal controls; workflow-specific playback context should remain inside the owning score or Sequence surfaces.
- [x] Task 3: Keep readiness and control states legible within the existing design grammar (AC: 1, 3)
  - [x] Reuse the current compact panel and window language instead of creating a playback-only visual language or DAW-style surface density.
  - [x] Preserve non-color cues for readiness and control state, including label text, iconography, structure, and compact state copy where needed; do not rely on color-only distinctions.
  - [x] Keep the current warning semantics narrow: no warning chrome for plain `--` or `N. --`, hidden presentation for ready or neutral route states, and compact warning treatment only for broken assigned plugin routes.
  - [x] Make any transport-boundary messaging concise and embedded in the existing workflow surface rather than a separate explainer panel.
- [x] Task 4: Build directly on the completed Story 4.4 mix-capability baseline (AC: 1, 2, 3)
  - [x] Treat Story 4.4’s bounded route-first mix behavior as the current baseline: reuse the shipped `supported_mix_controls` seam and the existing honest score-context mixer surface instead of redefining capability rules here.
  - [x] Keep Story 4.5 focused on workflow compactness, surface placement, escalation boundaries, and state legibility rather than expanding into deeper mix capability design.
  - [x] If 4.5 reveals a UI need for a control or indicator that the current backend-owned capability seam does not yet support honestly, leave it out rather than inventing placeholder semantics.
- [x] Task 5: Add focused UI regression coverage and documentation updates for compact score-first playback (AC: 1, 2, 3)
  - [x] Extend frontend tests around the owning score-context surfaces so control placement, escalation paths, and readiness semantics remain compact and consistent.
  - [x] Add or refine tests that prove the user can stay in score context for the core first-audible path while deeper inventory workflows remain menu-driven and native editor workflows remain optional rack-origin handoffs.
  - [x] Update developer-facing docs and agent guidance if Story 4.5 sharpens durable rules about compact score-context playback surfaces, escalation boundaries, or non-color state legibility.

## Dev Notes

### Story Intent

- Story 4.5 is the UX-shaping companion to the audible-playback foundation established in Stories 4.1 through 4.4.
- The goal is not “add more UI.” The goal is to keep the first audible playback workflow calm, direct, and visibly part of score work.
- This story should reduce the chance that Epic 4 drifts into proof-oriented surfaces, duplicate controls, or app-scoped detours that make playback feel heavier than the underlying product need.

### Epic And Story Context

- Epic 4 is proving that Compose can move from written score material to believable sound without breaking the notation-first product shape.
- Story 4.2 established the score-context audition contract.
- Story 4.3 established the concrete continuous engine path behind that contract.
- Story 4.4 defines which bounded route-first mix controls are honest for the active route path.
- Story 4.5 should make those capabilities feel native to the existing score workflow rather than like a separate playback mode.

### Previous Story Intelligence

- Story 4.4 is now complete and establishes the current honest mix baseline: `volume`, `mute`, `solo`, and `pan` are surfaced only when the active ready direct-plugin route proves them through backend `supported_mix_controls`.
- Story 4.4 also kept the score-context mixer compact and backend-owned in meaning. Story 4.5 should refine the workflow around those shipped controls rather than reopen mix-capability design.
- Story 4.3 confirmed that editor hosting, transport, and audition are all secondary consumers of one backend-owned playback engine. Story 4.5 should not answer UX concerns by creating a second workflow surface or second domain owner.
- Earlier playback work in Epics 2 and 3 kept Plugin Manager app-scoped, route authoring score-context, and the rack-origin native editor path bounded. That split is an architectural boundary, not just a presentational choice.
- The repo memory for playback-readiness UI semantics is explicit: plain unrouted and empty-slot states are not warnings, while broken assigned plugin routes are.

### This Story Owns

- Keeping the first audible workflow available through compact score-context controls.
- Preserving the current score-context versus app-scoped window boundary while making escalation paths deliberate.
- Ensuring playback readiness and bounded control state remain legible within the current Compose design grammar.
- Preventing proof-only UI growth that would make the workflow feel DAW-like or operationally heavier than necessary.

### This Story Does Not Own

- New app-scoped windows, dashboards, inspectors, or playback workspaces.
- Deep plugin browsing, plugin validation, or native editor lifecycle changes beyond preserving the correct escalation boundary.
- Mixer-capability semantics that belong to Story 4.4.
- Broader transport trust, seek, pause, resume, or multi-route playback behavior that belongs to Epic 5 and later.

### Architecture Compliance

- Score-context playback surfaces remain compact and direct, while app-scoped windows own plugin inventory and deep editor workflows.
- Rust remains the authoritative playback-domain owner; Story 4.5 must not solve UX clarity by inventing frontend-owned route, readiness, transport, or capability truth.
- The shared main-window top bar stays limited to universal controls; workflow-specific playback interaction belongs inside the owning score or Sequence workflow surface.
- Keep backend playback-state richness and user-facing warning chrome separate. UI presentation should stay intentionally narrower than backend internals.
- Preserve the existing bounded native-editor handoff from the score-context rack surface; it is a targeted secondary action, not a reason to shift routine playback work out of score context.

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Current route-state and readiness presentation in the score-context workflow.
- `src/windows/main/screens/sequence/components/playback-readiness.tsx`
  - Current compact warning and non-warning semantics for route readability.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Current presentation mapping that keeps ready and neutral states hidden while surfacing compact warning treatment only for broken assigned routes.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Current score-context rack surface for slot preparation, plugin assignment, and bounded native-editor handoff.
- `src/windows/main/components/mixer.tsx`
  - Current bounded score-context mixer surface.
- `src/windows/main/components/mixer-channel.tsx`
  - Current per-instrument control presentation and the strongest place to keep the workflow compact.
- `src/windows/main/components/transport.tsx`
  - Current universal transport surface that must not absorb workflow-specific playback explanation.
- `src/windows/main/utils/audition.ts`
  - Current ready-route audition gate that should remain a score-context trigger rather than becoming a second playback workflow.
- `src/windows/main/screens/sequence/`
  - Owning workflow surface for route authoring and score-context playback control placement.
- `src/services/actions.ts`
  - Command bridge that should remain thin; Story 4.5 should not add frontend-owned workflow state here.

### Implementation Guidance

- Prefer auditing and refining existing Sequence and score-context surfaces over adding new ones; this story starts from a mature baseline, not an empty UI seam.
- Keep the primary questions answerable at a glance inside score context: what is this instrument routed to, is it ready, what can I change here, and can I hear it now.
- Reuse the shipped route-row, rack-panel, mixer, and audition seams before adding any new control placement.
- If a real workflow needs deeper interaction, escalate outward deliberately to the Plugin Manager window or through the existing rack-origin native editor handoff. Do not create secondary in-between panels that duplicate those workflows.
- Preserve non-color legibility through text, iconography, spacing, titles, and control state, especially for readiness and actionability.
- Keep any explanation of the current single-instrument transport boundary minimal and embedded in the current workflow, not in a dedicated help surface.

### Risks And Regression Traps

- Adding a playback-specific control panel, floating inspector, or extra toolbar just to prove the story would violate the compact score-first requirement.
- Repeating route and readiness controls across multiple surfaces would create scope confusion about what the user should trust.
- Using color alone to distinguish important states would violate AC3 and weaken accessibility.
- Moving routine first-audible tasks into Plugin Manager or plugin-editor workflows would invert the intended architectural boundary.
- Reopening the already-shipped 4.4 capability contract inside 4.5 would blur ownership and create duplicate semantics.

### Testing Requirements

- Add frontend coverage proving:
  - the core first-audible workflow remains achievable from score-context surfaces without detouring into app-scoped windows;
  - route/readiness state remains legible with the bounded warning semantics already established;
  - compact controls do not regress into duplicate or proof-only surfaces;
  - non-color cues remain present for key readiness and control states.
- Extend tests around existing owning seams such as:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
  - `src/windows/main/components/mixer.test.tsx`
  - `src/windows/main/components/mixer-channel.test.tsx`
  - `src/windows/main/components/transport.test.tsx` if placement or wording changes there become necessary
  - `src/windows/main/utils/audition.test.ts` when score-context trigger or ready-route gating changes are part of the implementation
- Validation commands for implementation should include:
  - `npm run test:js -- src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx src/windows/main/components/mixer.test.tsx src/windows/main/components/mixer-channel.test.tsx src/windows/main/components/transport.test.tsx src/windows/main/utils/audition.test.ts`
  - `npm run lint`
  - `npm run check-types`

### Documentation Impact

- Update docs in the same implementation change if Story 4.5 sharpens durable guidance for compact score-context playback surfaces, escalation boundaries, or non-color state legibility.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if the “minimum useful surface” rule or the rack-origin editor handoff boundary needs sharper agent guidance for playback UX

### Git Intelligence Summary

- Recent Epic 4 work is still tightly sequenced around backend playback truth and compact score-context surfaces:
  - `be1a05fb feat(4.4): mixer actions`
  - `f03085e2 chore: story creation`
  - `3fe6a7e4 feat(4.3): audio playback`
  - `258a2775 chore: update plan to fix audtion holistically`
- Story 4.5 should preserve that direction and avoid introducing a new UI workflow that implies a different playback architecture.

### Latest Technical Information

- Repo baseline remains aligned with current external guidance:
  - Tauri 2 continues to support a thin command bridge with one `generate_handler!` registration surface and frontend `invoke(...)` calls, which fits the repo’s existing React-to-Rust interaction pattern.
  - No external UI framework or design-system migration is required for this story; the repo’s current custom design language remains the intended implementation path.
  - Current repo docs already codify the honest score-context mix baseline and warning semantics, so Story 4.5 should consume those project rules rather than reinterpret them locally.
- Story 4.5 is therefore a workflow-refinement story inside the current architecture, not a technology-change story.

### Project Structure Notes

- Keep implementation within existing frontend workflow seams under `src/windows/main/` plus any thin bridge updates in `src/services/`.
- Do not add new windows, new app roots, or new design-system layers for this story.
- If the work starts to require deeper route-capability semantics or live playback-engine changes, hand that concern back to Story 4.4 or the owning backend seam rather than solving it with extra UI.

### References

- `_bmad-output/planning-artifacts/epics.md` - Epic 4 and Story 4.5 acceptance criteria.
- `_bmad-output/planning-artifacts/ux-design-specification.md` - direct routing, score-first mental model, progressive disclosure, and compact-design guidance.
- `_bmad-output/planning-artifacts/architecture.md` - score-context versus app-scoped boundary, compact playback surface rules, and backend-owned playback truth.
- `_bmad-output/planning-artifacts/prd.md` - first-audible playback goals and bounded MVP framing.
- `docs/project-context.md` - score-context playback ownership, top-bar constraints, and warning-state semantics.
- `docs/architecture.md` - current score-context rack and route surfaces, transport boundary, and engine-backed playback model.
- `docs/development-guide.md` - compact warning semantics, playback-surface guardrails, and validation commands.
- `/memories/repo/playback-readiness-ui-semantics.md` - compact warning rules for route states.
- `/memories/repo/playback-rack-first-model.md` - rack-first ownership and score-context route semantics.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story creation analysis covered Epic 4 acceptance criteria, Sequence score-context playback surfaces, readiness-badge semantics, score-context versus app-scoped boundaries, UX-design guidance, and recent Epic 4 commit sequencing.
- 2026-04-03: Confirmed with the user that the current `--` unrouted presentation and the existing score-context rack versus app-scoped editor boundary should remain unchanged for Story 4.5.
- 2026-04-03: Added regression coverage locking app-scoped inventory actions out of the Sequence score-context rack surface and Instruments panel header.
- 2026-04-03: Targeted tests passed for `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` and `src/windows/main/screens/sequence/components/bottom-panel.test.tsx`; `npm run lint` and `npm run check-types` passed.
- 2026-04-03: Investigated the full-suite blocker in `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`; root cause was that automatic selected-tone audition reused the general ready-route gate and ignored `single_instrument_transport_ready`, so a selected tone was incorrectly marked as already auditioned before transport readiness flipped true.
- 2026-04-03: Fixed `src/windows/main/hooks/use-keyboard-shortcuts.ts` so automatic selection audition waits for `single_instrument_transport_ready` while explicit audition actions keep the existing ready-route gate; targeted keyboard-shortcuts tests, full JS suite, `npm run lint`, and `npm run check-types` now pass.
- 2026-04-03: Added a compact non-color legibility pass across icon-only playback controls by moving icon button labels/titles onto the button element, exposing `aria-pressed` for selected toggle state, and naming the Sequence bottom-panel tabs, transport actions, and score-context mix controls explicitly.
- 2026-04-03: Added regression coverage for those compact legibility cues in `transport.test.tsx`, `mixer-channel.test.tsx`, `instrument-overview-row.test.tsx`, and `bottom-panel.test.tsx`; full JS suite, `npm run lint`, and `npm run check-types` pass after the change.
- 2026-04-03: Completed the remaining compact-legibility audit slice by extending `mixer.test.tsx` for toolbar labels and pressed-state semantics, and by updating `docs/development-guide.md`, `docs/architecture.md`, and `docs/project-context.md` to codify the non-color score-context playback-control rule.
- 2026-04-03: Full JS suite, `npm run lint`, and `npm run check-types` pass after the final regression/doc updates.
- 2026-04-03: User approved the current Story 4.5 implementation shape and requested final bookkeeping to mark the story done.

### Completion Notes List

- Refreshed Story 4.5 against the post-4.4 baseline so the implementation guidance reflects the shipped honest mix controls, current score-context seams, and current escalation boundaries.
- Added focused regression tests for Story 4.5's compact score-first boundary without changing product behavior: the Sequence rack surface and Instruments panel header do not surface app-scoped inventory actions such as Plugin Manager or refresh controls.
- Left the current unrouted `--` route presentation unchanged per user direction.
- Left the current score-context rack versus app-scoped native editor boundary unchanged per user direction.
- Investigated and fixed an unrelated full-suite blocker in `src/windows/main/hooks/use-keyboard-shortcuts.ts` so automatic selection audition now honors `single_instrument_transport_ready` before replaying a selected tone.
- Full JS validation is green again after the keyboard-shortcuts fix.
- Added a small 4.5 implementation slice for AC3 without changing layout: icon-only transport, tab, and score-context mix controls now expose explicit labels/titles and pressed state, so compact legibility does not depend on color alone.
- Added focused regression tests locking that non-color legibility into the owning transport, sequence, and mix surfaces.
- Extended the remaining mixer-toolbar regression coverage so both row-level and toolbar-level compact mix controls now prove the same non-color legibility contract.
- Updated the developer-facing docs and project-context guidance so the compact score-context playback-control rule is now durable project guidance rather than implicit behavior.
- User approved the final compact score-first outcome and the story bookkeeping was updated to done.

### File List

- `_bmad-output/implementation-artifacts/4-5-keep-first-audible-playback-compact-and-score-first.md`
- `src/ui/buttons/icon-button.tsx`
- `src/windows/main/components/mixer.test.tsx`
- `src/windows/main/components/mixer-channel.test.tsx`
- `src/windows/main/components/mixer-channel.tsx`
- `src/windows/main/components/mixer.tsx`
- `src/windows/main/components/transport.test.tsx`
- `src/windows/main/components/transport.tsx`
- `src/windows/main/screens/sequence/components/bottom-panel.test.tsx`
- `src/windows/main/screens/sequence/components/bottom-panel.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
