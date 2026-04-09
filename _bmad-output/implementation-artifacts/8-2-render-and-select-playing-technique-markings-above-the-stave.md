# Story 8.2: Render And Select Playing-Technique Markings Above The Stave

Status: done

## Story

As a composer,
I want playing-technique markings to appear and be selectable in the score,
so that the new score entries behave like real authored notation rather than invisible playback metadata.

## Acceptance Criteria

1. Given `PlayingTechnique` entries are stave-master-owned score markings, when they are rendered in the score, then they appear above their owning stave using the same established visual text-positioning approach as tempo markings, and they do not inherit tempo's flow-master ownership or top-stave-only behavior.
2. Given the user must be able to work with these markings like other score directives, when a visible playing-technique marking is shown in the score, then it emits native renderer hit targets and participates in shared selection behavior, and the selected state is owned by the same backend-driven score interaction model as other directive-style entries.
3. Given the first shipped scope is intentionally bounded, when `pizz` and `arco` markings are displayed, then the rendered text is clear and notation-like, and the draw path reuses proven text-rendering patterns instead of inventing a separate visual system for technique markings.
4. Given the visible score workflow must include authoring and removal for the first shipped markings, when the user opens the playing-technique popover from the current score selection, then Compose reuses the existing free-text popover pattern with `Shift+P`, accepts only `pizz` and `arco`, creates the new marking at the selection tick, selects the created marking, and if the current selection has no `stave_id` it attaches the new marking to the first stave in the score for that flow.
5. Given same-tick technique coexistence is already part of the durable score model, when multiple visible playing-technique markings exist at the same tick on one stave, then they all draw at the regular canonical text anchor for that tick and stave even if the text overlaps, and deleting a selected playing-technique marking through `Delete` or `Backspace` removes only the selected entry without confirmation.

## Tasks / Subtasks

- [x] Task 1: Render visible playing-technique markings above the owning stave with tempo-style text positioning (AC: 1, 3, 5)
  - [x] Add the native draw path for `Entry::PlayingTechnique` under the existing Rust renderer pipeline; do not render technique markings in React or HTML.
  - [x] Reuse the proven text-measurement, font, color, and above-stave positioning approach already used for tempo-style directive text, while keeping ownership on stave-master tracks instead of flow-master scope.
  - [x] Respect existing hidden-entry rules so hidden playing-technique entries do not draw or emit hit targets even though the `Hidden` editing workflow lands in Story 8.3.
  - [x] Render multiple visible same-tick playing-technique markings at the regular canonical text anchor for that tick and stave, explicitly deferring any collision-avoidance, spacing, or smarter stacking behavior.

- [x] Task 2: Extend the shared hit-target and backend-driven selection model to include playing-technique markings (AC: 2, 5)
  - [x] Add a `PlayingTechnique` selection variant to the Rust and TypeScript selection models with the fields needed for directive-style shared selection, including `flow_id`, `stave_id`, `playing_technique_id`, and `tick`.
  - [x] Emit native renderer hit targets for visible playing-technique markings and wire them through the existing hit-test path, selection-coordinate path, and shared renderer overlay selection handling.
  - [x] Add the narrow sequencer selection command and frontend action wrapper for playing-technique selection using the same backend-owned flow as other directive-style entries.
  - [x] Keep selected styling driven from the shared backend selection state rather than from a frontend-only shadow model.

- [x] Task 3: Add the bounded popover create and keyboard delete workflow for visible playing-technique markings (AC: 4, 5)
  - [x] Extend the existing popover system with one playing-technique entry path that reuses the free-text parser pattern and opens from `Shift+P`.
  - [x] Allow the popover to open from any current selectable score entry that carries a tick value; when the current selection already has `stave_id`, create on that stave, and when it only carries a tick such as tempo, key signature, or time signature, resolve creation to the first stave in the score for that flow.
  - [x] Keep the accepted vocabulary intentionally bounded to `pizz` and `arco` only, with no additional aliases, no mouse-only create affordance, and no technique-editing mutation flow beyond create plus delete.
  - [x] When the popover is submitted from an already selected playing-technique marking, create the new technique at the same tick under the existing same-tick coexistence rules instead of replacing the selected marking.
  - [x] Extend the shared score keyboard-delete path so `Delete` and `Backspace` remove only the currently selected playing-technique entry via the existing remove-by-id seam instead of clearing all techniques at the tick.

- [x] Task 4: Add seam-local regression coverage and update the user-facing workflow docs (AC: 1, 2, 3, 4, 5)
  - [x] Add Rust coverage around the draw path, hidden-entry suppression, hit-target emission, selection highlighting, and any same-tick row-layout behavior owned by the renderer seam.
  - [x] Extend hit-test and UI selection coverage so playing-technique markings participate in shared backend-owned selection the same way other directive-style entries do.
  - [x] Add TypeScript coverage for the new popover shortcut, accepted-token parsing, first-stave fallback for tick-only selections, renderer click-to-select behavior, and keyboard deletion of selected playing-technique entries.
  - [x] Update `docs/development-guide.md` or the nearest maintained user/developer workflow doc to record the bounded playing-technique authoring path, including `Shift+P`, accepted tokens, same-tick row behavior, first-stave fallback for tick-only selections, and keyboard deletion semantics.

### Review Findings

- [x] [Review][Defer] Same-tick playing-technique siblings are not individually selectable [src-tauri/src/instructions/mod.rs:909] — deferred for now because this story is proving the concept; proper text collision and coexistence handling will be addressed later as a larger holistic change.
- [x] [Review][Patch] Tick-only playing-technique fallback uses unordered flow `player_ids` [src/ui/popover.tsx:101]

## Dev Notes

### Story Intent

- Story 8.2 turns the durable playing-technique entries from Story 8.1 into visible score directives that behave like authored notation inside the shared score interaction model.
- The scope is intentionally linked: render, native hit targets, backend-owned selection, popover creation, and keyboard deletion all belong to the same visible score workflow for the first shipped markings.
- The highest-risk failure mode is widening into a parallel UI system or properties workflow instead of reusing the existing renderer, hit-test, selection, popover, and keyboard-delete seams.

### User Clarifications

- Story 8.2 should keep the split agreed during refinement: render, shared selection, popover creation, and keyboard deletion belong here, while the `Hidden` properties panel remains Story 8.3.
- The playing-technique popover should reuse the existing free-text parser pattern and open from `Shift+P`.
- The first shipped popover vocabulary stays bounded to `pizz` and `arco` only.
- Any currently selectable score entry with a tick value may act as the popover entry point.
- If the current selection already has `stave_id`, create the new playing-technique marking on that stave.
- If the current selection only carries a tick and no `stave_id`, such as tempo, key signature, or time signature, create the playing-technique marking on the first stave in the score for that flow instead of rejecting creation.
- If the user opens the popover from an already selected playing-technique marking and submits a different technique, the new entry should be created at the same tick under the existing coexistence rules instead of replacing the selected entry.
- Multiple visible same-tick playing-technique markings should all draw at the regular canonical x and y position for this story, even if they overlap; any collision-avoidance pass is future work.
- `Delete` and `Backspace` should remove only the selected playing-technique entry with no confirmation.

### This Story Owns

- Visible score rendering for `PlayingTechnique` entries above their owning stave.
- Native hit targets and shared backend-owned selection behavior for visible playing-technique markings.
- One bounded keyboard-first popover authoring path for `pizz` and `arco`.
- Keyboard deletion for selected playing-technique markings through the shared score shortcut model.
- Seam-local regression coverage and workflow documentation for the visible playing-technique authoring path.

### This Story Does Not Own

- Bottom-panel properties surfaces or the `Hidden` toggle workflow; those belong to Story 8.3.
- Broader playing-technique vocabulary, aliases, technique-specific configuration, or edit-in-place kind mutation.
- Mouse-only creation affordances or a separate context menu just for playing techniques.
- A sophisticated text-collision, horizontal spreading, or vertical stacking system beyond the canonical-anchor draw path for this story.
- Derived playing-technique context for notes, expression-map matching, or playback behavior; those remain in Stories 8.4 through 8.6.

### Current Relevant Seams

- `src-tauri/src/score/entries/playing_technique.rs`
  - Owner of the playing-technique entry family, create/remove/update seams, and same-tick duplicate/coexistence rules established in Story 8.1.
- `src-tauri/src/instructions/draw/draw_tempo.rs`
  - Best reference for above-stave directive text rendering, selected coloring, and hit-target emission anchored to popover coordinates.
- `src-tauri/src/instructions/mod.rs`
  - Shared render-plan composition and selection-coordinate plumbing that already knows how directive hit targets map back to popover anchors.
- `src-tauri/src/renderer/mod.rs`
  - Owner of `HitTarget` and shared renderer context state used by native score hit testing.
- `src-tauri/src/renderer/hit_test.rs`
  - Shared hit-test behavior for native renderer targets; extend this instead of inventing playing-technique-specific pointer logic.
- `src-tauri/src/ui/sequencer.rs`
  - Owner of the backend selection enum and narrow Tauri selection commands for score items.
- `src/services/types.ts`
  - Frontend mirror of the backend selection union and entry types; keep it aligned with Rust.
- `src/services/actions.ts`
  - Existing action wrappers already include playing-technique create/remove/update seams and need the selection command wrapper added in the same style.
- `src/services/popovers.ts`
  - Existing free-text popover parser seam used by clef, tempo, key signature, time signature, barline, and tuplet authoring.
- `src/ui/popover.tsx`
  - Shared shortcut registration, popover-opening, and submission flow for keyboard-first score authoring.
- `src/windows/main/components/renderer.tsx`
  - Shared frontend overlay that consumes hit-test results and dispatches backend-owned selection commands.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Shared delete and edit keyboard workflow; extend the existing directive-style deletion path rather than adding a second removal model.
- `docs/development-guide.md`
  - Best current home for documenting the new bounded playing-technique authoring workflow unless implementation reveals a better maintained workflow reference.

### Architecture Compliance

- Keep backend-owned score truth and backend-owned selection canonical. React must not become the owner of visible playing-technique state or selected-state truth.
- Render playing-technique markings through the native Rust/Skia/Metal score renderer only; do not recreate them in the HTML overlay.
- Keep playing-technique ownership on stave-master tracks even when the visual positioning borrows tempo-style text layout.
- Reuse the existing hit-target, selection, popover, and keyboard-delete seams instead of inventing one-off playing-technique workflows.
- Hidden entries must stay aligned with shared directive behavior: no draw, no hit target, no frontend-only hidden-state workaround.
- The tick-only-selection fallback to the first stave must be resolved from backend-pushed score truth for the selected flow, not from a fabricated frontend-owned alternate ordering model.

### Implementation Guidance

- Prefer a dedicated draw seam for playing-technique markings, parallel to other directive-style draw modules, instead of embedding one-off logic into unrelated renderer code.
- Use the current tempo draw path as the reference for text measurement, selected coloring, and hit-target anchoring, while keeping the y-position and stave ownership tied to the actual owning stave.
- Add one new `Selection::PlayingTechnique` variant in Rust and mirror it in TypeScript, then thread it through selection setters, hit-test results, and selection coordinate lookup.
- Keep the first-stave fallback bounded and deterministic: when the current selection has a tick but no `stave_id`, resolve the first stave from the selected flow's existing score order and use that stave for creation.
- After popover submission, select the created playing-technique entry using the id returned from the existing create command.
- Use the remove-by-id playing-technique command for keyboard deletion so deleting one selected technique does not clear sibling same-tick techniques on the same stave.
- Do not add token aliases such as `pizz.` in this story unless scope is explicitly revisited.

### Risks And Regression Traps

- Treating playing-technique markings like tempo flow-master entries or top-stave-only directives, which would violate the established score ownership model.
- Implementing visible marking behavior in React rather than in the native renderer and shared hit-test path.
- Extending delete behavior with a clear-at-tick operation that removes sibling same-tick techniques instead of only the selected entry.
- Rejecting tick-only directive selections outright instead of honoring the agreed first-stave fallback.
- Widening this story into the `Hidden` properties workflow, bottom-panel authoring UI, or playback semantics that belong to later Epic 8 stories.
- Overengineering collision handling instead of shipping the agreed canonical-anchor draw path first.

### Testing Requirements

- Add Rust tests for the playing-technique draw path covering visible rendering, hidden suppression, hit-target creation, and selected-state coloring.
- Extend native hit-test coverage so playing-technique hit targets resolve to the new selection variant and preserve the shared best-target behavior.
- Add or extend UI selection tests around the new sequencer selection command and any selection-coordinate logic required for popover anchoring.
- Add TypeScript tests for `Shift+P` popover opening, accepted token parsing, tick-only-selection first-stave fallback, renderer click selection, and `Delete`/`Backspace` deletion of a selected playing-technique entry.
- Keep tests seam-local: renderer behavior near the Rust draw and hit-test seams, popover parsing near `src/services/popovers.ts`, and delete behavior near the shared keyboard-shortcuts hook.

### Documentation Impact

- Update `docs/development-guide.md` or the nearest maintained workflow doc to describe the bounded playing-technique authoring flow in the score.
- The documentation update should explicitly cover `Shift+P`, accepted tokens (`pizz`, `arco`), same-tick canonical-anchor overlap behavior, the first-stave fallback for tick-only selections, and keyboard deletion semantics.
- Update `docs/project-context.md` only if implementation establishes a durable repo-wide rule that future agents must follow beyond this feature-specific workflow.
- No AI-agent instruction update is required unless implementation turns the first-stave fallback or selection behavior into a broader repository rule.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- No new dependencies should be introduced for Story 8.2.

### File Structure Notes

- Keep Rust rendering, hit-test, and selection changes near the current score-render pipeline and UI-selection seams under `src-tauri/src/`.
- Keep shared popover and keyboard workflow changes in the existing frontend seams under `src/services/`, `src/ui/`, and `src/windows/main/`.
- Keep story tracking under `_bmad-output/implementation-artifacts/` and update sprint status in the same pass.

### Git Intelligence Summary

- The latest commit, `feat(8.1): playing technique setup`, confirms that Story 8.2 should build directly on the new `PlayingTechnique` entry family and existing create/remove/update actions rather than inventing a second playing-technique model.
- The recent migration cleanup and reseed commits reinforce that 8.2 is a render-and-interaction story, not a persistence-schema or migration story.

### Latest Technical Information

- No external library research is needed for Story 8.2.
- The decisive current information is repo truth as of 2026-04-09: there is already a shared popover authoring system, a shared directive-style selection model, and a shared keyboard delete path that Story 8.2 should extend rather than duplicate.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 8 and Story 8.2 acceptance criteria.
- `_bmad-output/implementation-artifacts/8-1-add-durable-playing-technique-score-entries.md`
  - Previous-story constraints, same-tick coexistence rules, and the bounded `PlayingTechniqueKind` vocabulary.
- `docs/project-context.md`
  - Backend-owned state, native render ownership, selection, and score-entry architecture rules.
- `_bmad-output/planning-artifacts/architecture.md`
  - Rust-owned playback and UI-boundary rules that still apply to score interaction work.
- `_bmad-output/planning-artifacts/prd.md`
  - Product intent around score-first workflows and existing desktop architecture boundaries.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Scope model for contextual score surfaces versus deeper configuration surfaces.
- `src-tauri/src/score/entries/playing_technique.rs`
  - Playing-technique entry owner seam from Story 8.1.
- `src-tauri/src/instructions/draw/draw_tempo.rs`
  - Reference draw path for above-stave directive text and hit targets.
- `src-tauri/src/renderer/hit_test.rs`
  - Shared native hit-test behavior.
- `src-tauri/src/ui/sequencer.rs`
  - Backend selection model and selection command owner seam.
- `src/services/popovers.ts`
  - Existing free-text score-authoring parser seam.
- `src/ui/popover.tsx`
  - Shared keyboard shortcut and popover UI flow.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Shared score keyboard delete path.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-09: Story prepared from Epic 8 planning, Story 8.1 output, repo architecture and project-context docs, shared popover workflow, shared selection seams, and current keyboard-delete behavior.
- 2026-04-09: Scope refined with the user before story creation: 8.2 owns render, shared selection, popover creation, and keyboard deletion; 8.3 keeps the `Hidden` properties workflow.
- 2026-04-09: Tick-only selection refinement recorded: when the selected score entry has a tick but no `stave_id`, playing-technique creation should target the first stave in the score for that flow.

### Completion Notes List

- Story context created for Story 8.2.
- Story 8.2 moved to `ready-for-dev` in sprint tracking.
- Story scope explicitly includes render, hit targets, shared selection, popover creation, and keyboard deletion for visible playing-technique markings.
- Story scope explicitly excludes the `Hidden` properties panel, broader vocabulary, mouse-only creation, and playback semantics.
- Implemented native Rust rendering for visible playing-technique markings with hit targets, selected styling, hidden suppression, and the current same-tick canonical-anchor draw behavior.
- Added backend and TypeScript selection support for playing-technique directives, including hit-test dispatch, selection-coordinate lookup, and a dedicated sequencer selection command.
- Added the bounded `Shift+P` popover workflow for `pizz` and `arco`, including deterministic first-stave fallback for tick-only selections and remove-by-id keyboard deletion.
- Added seam-local Rust and TypeScript regression coverage for rendering, hit testing, selection, popover parsing, shortcut opening, first-stave fallback, renderer click selection, and deletion.
- Updated the development guide with the shipped playing-technique authoring guardrails.

### File List

- `_bmad-output/implementation-artifacts/8-2-render-and-select-playing-technique-markings-above-the-stave.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/development-guide.md`
- `src/services/actions.test.ts`
- `src/services/actions.ts`
- `src/services/popovers.test.ts`
- `src/services/popovers.ts`
- `src/services/types.ts`
- `src/ui/popover.test.tsx`
- `src/ui/popover.tsx`
- `src/windows/main/components/renderer.test.tsx`
- `src/windows/main/components/renderer.tsx`
- `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
- `src/windows/main/hooks/use-selected-tone-info.ts`
- `src-tauri/src/instructions/draw/draw_playing_technique.rs`
- `src-tauri/src/instructions/draw/mod.rs`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/renderer/hit_test.rs`
- `src-tauri/src/score/entries/playing_technique.rs`
- `src-tauri/src/ui/mod.rs`
- `src-tauri/src/ui/sequencer.rs`

### Change Log

- 2026-04-09: Implemented Story 8.2 native playing-technique rendering, shared selection, bounded popover authoring, delete workflow, regression coverage, and workflow documentation updates.
