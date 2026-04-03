# Story 4.5: Keep First Audible Playback Compact And Score-First

Status: ready-for-dev

## Story

As a composer,
I want the first audible playback workflow to stay compact and tied to score-context controls,
so that hearing the score feels like part of composition instead of a detour into playback tooling.

## Acceptance Criteria

1. Given first audible playback is being proved through score-context auditioning and related controls, when the UI for this epic is implemented, then the useful workflow is available through compact score-context controls, and the design avoids extra explanatory surfaces or duplicate controls that exist mainly to prove acceptance criteria.
2. Given deeper inventory and plugin editing already belong to app-scoped windows, when first audible playback is used, then the core audible workflow stays in the score context, and it escalates outward only when a real user need requires it.
3. Given Compose uses a custom compact design language, when first audible playback surfaces are added or refined, then they preserve the existing panel and window grammar, and readiness and control states remain legible without relying on color alone.

## Tasks / Subtasks

- [ ] Task 1: Audit and refine the current score-context audible workflow around existing Sequence surfaces (AC: 1, 2, 3)
  - [ ] Start from the current score-context seams in `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`, `src/windows/main/components/mixer.tsx`, `src/windows/main/components/mixer-channel.tsx`, `src/windows/main/components/transport.tsx`, and the Sequence bottom-panel rack surfaces rather than inventing a new playback workspace.
  - [ ] Keep the core loop compact: understand route/readiness, adjust the bounded supported control set, audition from score interaction, and start supported score playback from the same score-first context.
  - [ ] Remove or avoid explanatory or duplicate controls whose main purpose is to demonstrate acceptance criteria literally rather than solve a real user need.
  - [ ] Preserve the current direct mental model that a score instrument routes to a playback destination; do not teach plugin-host topology as the user workflow.
- [ ] Task 2: Preserve the architectural boundary between score-context workflow and app-scoped escalation surfaces (AC: 1, 2)
  - [ ] Keep Plugin Manager as the app-scoped inventory browser and the bounded native plugin editor as a secondary deep-edit handoff, not as the default home of first audible playback.
  - [ ] Ensure any escalation from score context happens only for a real workflow need such as browsing inventory or opening a plugin’s native editor, not because the core audible path is incomplete or cluttered.
  - [ ] Avoid duplicating route, readiness, or mix controls across score-context and app-scoped windows just to make the system feel “complete.”
  - [ ] Keep the top bar limited to universal controls; workflow-specific playback context should remain inside the owning score or Sequence surfaces.
- [ ] Task 3: Keep readiness and control states legible within the existing design grammar (AC: 1, 3)
  - [ ] Reuse the current compact panel and window language instead of creating a playback-only visual language or DAW-style surface density.
  - [ ] Preserve non-color cues for readiness and control state, including label text, iconography, structure, and compact state copy where needed.
  - [ ] Keep the current warning semantics narrow: no warning chrome for plain `--` or `N. --`, and compact warning treatment only for broken assigned plugin routes.
  - [ ] Make any transport-boundary messaging concise and embedded in the existing workflow surface rather than a separate explainer panel.
- [ ] Task 4: Coordinate explicitly with Story 4.4 without assuming it has already shipped (AC: 1, 2, 3)
  - [ ] Treat Story 4.4’s bounded route-first mix behavior as an adjacent prerequisite or sibling slice, but do not assume its implementation details are already present when refining Story 4.5.
  - [ ] Keep Story 4.5 focused on workflow compactness, surface placement, and state legibility rather than expanding into deeper mix capability design.
  - [ ] If 4.5 reveals a UI need for a control or indicator that 4.4 does not yet support honestly, leave it out rather than inventing placeholder semantics.
- [ ] Task 5: Add focused UI regression coverage and documentation updates for compact score-first playback (AC: 1, 2, 3)
  - [ ] Extend frontend tests around the owning score-context surfaces so control placement, escalation paths, and readiness semantics remain compact and consistent.
  - [ ] Add or refine tests that prove the user can stay in score context for the core first-audible path while deeper inventory or editor workflows remain optional escalation paths.
  - [ ] Update developer-facing docs and agent guidance if Story 4.5 sharpens durable rules about compact score-context playback surfaces, escalation boundaries, or non-color state legibility.

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

- Story 4.3 confirmed that editor hosting, transport, and audition are all secondary consumers of one backend-owned playback engine. Story 4.5 should not answer UX concerns by creating a second workflow surface or second domain owner.
- Earlier playback work in Epics 2 and 3 kept Plugin Manager app-scoped and route authoring score-context. That split is an architectural boundary, not just a presentational choice.
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

### Current Relevant Seams

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Current route-state and readiness presentation in the score-context workflow.
- `src/windows/main/screens/sequence/components/playback-readiness.tsx`
  - Current compact warning and non-warning semantics for route readability.
- `src/windows/main/components/mixer.tsx`
  - Current bounded score-context mixer surface.
- `src/windows/main/components/mixer-channel.tsx`
  - Current per-instrument control presentation and the strongest place to keep the workflow compact.
- `src/windows/main/components/transport.tsx`
  - Current universal transport surface that must not absorb workflow-specific playback explanation.
- `src/windows/main/screens/sequence/`
  - Owning workflow surface for route authoring and score-context playback control placement.
- `src/services/actions.ts`
  - Command bridge that should remain thin; Story 4.5 should not add frontend-owned workflow state here.

### Implementation Guidance

- Prefer refinement of existing Sequence and score-context surfaces over adding new surfaces.
- Keep the primary questions answerable at a glance inside score context: what is this instrument routed to, is it ready, what can I change here, and can I hear it now.
- If a real workflow needs deeper interaction, escalate outward deliberately to existing app-scoped windows. Do not create secondary in-between panels that duplicate those workflows.
- Preserve non-color legibility through text, iconography, spacing, and control state, especially for readiness and actionability.
- Keep any explanation of the current single-instrument transport boundary minimal and embedded in the current workflow, not in a dedicated help surface.

### Risks And Regression Traps

- Adding a playback-specific control panel, floating inspector, or extra toolbar just to prove the story would violate the compact score-first requirement.
- Repeating route and readiness controls across multiple surfaces would create scope confusion about what the user should trust.
- Using color alone to distinguish important states would violate AC3 and weaken accessibility.
- Moving routine first-audible tasks into Plugin Manager or plugin-editor workflows would invert the intended architectural boundary.
- Treating the sibling Story 4.4 control set as already implemented could lead Story 4.5 to design around unavailable or dishonest controls.

### Testing Requirements

- Add frontend coverage proving:
  - the core first-audible workflow remains achievable from score-context surfaces without detouring into app-scoped windows;
  - route/readiness state remains legible with the bounded warning semantics already established;
  - compact controls do not regress into duplicate or proof-only surfaces;
  - non-color cues remain present for key readiness and control states.
- Extend tests around existing owning seams such as:
  - `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `src/windows/main/components/mixer.test.tsx`
  - `src/windows/main/components/mixer-channel.test.tsx`
  - `src/windows/main/components/transport.test.tsx` if placement or wording changes there become necessary
- Validation commands for implementation should include:
  - `npm run test:js -- src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx src/windows/main/components/mixer.test.tsx src/windows/main/components/mixer-channel.test.tsx`
  - `npm run lint`
  - `npm run check-types`

### Documentation Impact

- Update docs in the same implementation change if Story 4.5 sharpens durable guidance for compact score-context playback surfaces, escalation boundaries, or non-color state legibility.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if the “minimum useful surface” rule needs sharper agent guidance for playback UX

### Git Intelligence Summary

- Recent Epic 4 work is still tightly sequenced around backend playback truth and compact score-context surfaces:
  - `3fe6a7e4 feat(4.3): audio playback`
  - `258a2775 chore: update plan to fix audtion holistically`
  - `3ce3e57f feat(4.2): auditioning`
  - `a98b6cc4 feat(4.1): playback prep`
- Story 4.5 should preserve that direction and avoid introducing a new UI workflow that implies a different playback architecture.

### Latest Technical Information

- Repo baseline remains aligned with current external guidance:
  - Tauri 2 continues to support a thin command bridge with one `generate_handler!` registration surface and frontend `invoke(...)` calls, which fits the repo’s existing React-to-Rust interaction pattern.
  - No external UI framework or design-system migration is required for this story; the repo’s current custom design language remains the intended implementation path.
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

### Completion Notes List

- Created Story 4.5 as an implementation-ready guide for keeping first audible playback compact, score-first, and aligned with existing Compose workflow grammar.

### File List

- `_bmad-output/implementation-artifacts/4-5-keep-first-audible-playback-compact-and-score-first.md`
