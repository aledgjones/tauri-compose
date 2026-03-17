# Story 3.2: Render Tuplet Brackets and Numbers Correctly

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As an engraving-sensitive composer,
I want tuplets to display the right bracket and number treatment,
so that the score reads like real notation rather than placeholder annotation.

## Scope Note

- This story is renderer-first. It owns visible tuplet number and bracket treatment after Story 3.1 has made tuplet notation derivation backend-owned and directly testable.
- Story 3.2 must consume the derived parser output. It must not recompute tuplet structure in a parallel renderer-only path or rebuild missing tuplet grouping from frontend state.
- Treat tuplet start and end boundaries like metric split points, the same way Compose already treats barlines during notation derivation. By the time 3.2 renders a tuplet, the notation should already be split at those boundaries so the renderer can treat the tuplet as one isolated grouped unit.
- This story covers bracket visibility, number placement, and tuplet-specific hit targets or selection anchors needed for those adornments.
- This story does not own beam grouping, rest-shape coherence, tie shaping, or broader spacing refinement. Those remain Story 3.3 and Story 3.4 scope.
- MVP remains default-engraving only. Do not introduce user-facing display overrides for bracket style, ratio style, or custom number formatting.

## Acceptance Criteria

1. Given a derived tuplet notation result that requires visible tuplet marking, when the score is rendered, then the tuplet number is drawn in the correct relationship to its grouped content and bracket visibility follows the supported MVP readability rules while consuming the derived parser output rather than recomputing tuplet structure in a parallel renderer-only path.
2. Given a supported case where bracket chrome is not required for clarity, when the score is rendered, then the number remains legible and unnecessary bracket rendering is omitted according to the product's supported default notation rules.
3. Given tuplets are rendered beside surrounding notation, when the score surface refreshes, then tuplet markings remain visually attached to the correct content and they do not resolve as unrelated overlay annotations.
4. Given a derived passage crosses a tuplet start or end boundary, when tuplet markings are rendered, then the renderer consumes notation already split at the tuplet boundary using the same metric-boundary discipline Compose applies at barlines and the tuplet span is treated as one isolated grouped unit for bracket and number placement.

## Tasks / Subtasks

- [ ] Render tuplets from backend-owned wrapper metadata rather than overlay reconstruction (AC: 1, 3, 4)
  - [ ] Extend the score render pipeline so tuplet number and bracket draw instructions are emitted from parser-derived tuplet context carried through the existing instructions pipeline.
  - [ ] Keep rendering fully backend-owned in Rust; do not add React or overlay-layer rendering for tuplet chrome.
  - [ ] Reuse the existing composed-surface flow so tuplet markings stay attached to the notation they govern in shared renderer space.
- [ ] Implement MVP bracket-visibility and number-placement rules (AC: 1, 2, 4)
  - [ ] Draw the tuplet number in the correct relationship to the isolated tuplet unit produced by parser splitting at tuplet boundaries.
  - [ ] Omit bracket chrome in supported cases where the grouped content remains legible without it.
  - [ ] Preserve the product's default notation behavior only; do not add user-facing overrides or advanced engraving-option plumbing.
- [ ] Add tuplet adornment hit targets and selection plumbing needed for native score interaction (AC: 3)
  - [ ] Introduce native hit targets for tuplet brackets and/or numbers so score interaction can resolve the tuplet container without replacing tone-id authority for notes.
  - [ ] Ensure hit targets survive shared-surface composition and `renderer_hit_test` resolution the same way other native score objects do.
  - [ ] Keep selection authority backend-owned and aligned with the existing `Tuplet` selection payload contract.
- [ ] Add focused renderer and hit-test regressions for tuplet bracket/number behavior (AC: 1, 2, 3, 4)
  - [ ] Add draw-level tests for tuplet number and bracket instructions, including a case where bracket chrome is omitted.
  - [ ] Add composition or hit-test coverage proving tuplet adornments remain attached to the correct rendered content in shared renderer space.
  - [ ] Add a boundary case where a passage is split at a tuplet start or end and the resulting tuplet marking still renders as one isolated grouped unit rather than as detached annotation.
  - [ ] Keep visual bracket/number tests close to the Rust renderer seams; do not defer Story 3.2 proof to later end-to-end fixture work.
- [ ] Validate the touched renderer seams with the standard repo checks
  - [ ] Run focused Rust tests for draw, hit-test, and composed-surface behavior that cover the changed tuplet rendering paths.
  - [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml` after renderer changes.
  - [ ] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean is a repo rule when Rust files change.
- [ ] Review documentation impact before closing the story
  - [ ] Confirm whether tuplet bracket/number rendering or tuplet hit-target behavior changes developer-facing docs or guardrails.
  - [ ] Update the relevant docs and agent instructions in the same change if the durable contract changes materially.
  - [ ] If no docs change is needed, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 3 is about reading tuplets as real notation. Story 3.1 establishes backend-owned tuplet notation derivation; Story 3.2 is the first visible score-rendering story and should consume that parser output rather than re-proving it indirectly.
- The user concern resolved during planning was validation and boundary ownership. Story 3.2 now assumes tuplet boundaries are treated like barline-style metric split points during derivation, so rendering can handle the tuplet as an isolated unit.
- Later stories still own broader engraving coherence. Do not let 3.2 absorb beam grouping, tuplet rests, tie shaping, or spacing-polish work that belongs to 3.3 and 3.4.

### Technical Requirements

- Rendering should remain fully backend-owned: derive child notation and child local spacing normally, map child local x positions into the consumed host span, then draw the tuplet number and bracket from wrapper metadata.
- Consume parser-derived notation that is already split at tuplet boundaries. Renderer logic should not decide where a boundary split belongs; it should place chrome around the isolated tuplet unit produced by the parse layer.
- Tuplets are not a separate visual layer. They are normal notation rendered inside a warped container, with number and bracket draw instructions attached to that same unit.
- Keep tuplet-specific interaction native to the Rust renderer via hit targets rather than HTML overlay reconstruction.
- Preserve tone-id authority for notes while allowing tuplet container selection by `tuplet_id` for bracket/number hit-testing.

### Architecture Compliance

- The architecture explicitly recommends backend-owned tuplet rendering: derive child notation, map child local x positions into the consumed host span, and draw the tuplet number and bracket from wrapper metadata.
- The same architecture states parser, renderer, and playback projection should perform boundary-aware splits the way Compose already uses barlines and ties, with tuplet spans acting as another local metric boundary. Treat that as a hard prerequisite for 3.2 rather than re-solving the split in the renderer.
- Tuplets themselves should be selectable by `tuplet_id`; this supports native hit-testing for brackets and numbers without replacing tone-id-based authority.
- Shared-surface composition is already authoritative for combined draw instructions and hit targets across flows. Tuplet adornment hit targets must compose through that existing path.

### Library / Framework Requirements

- Work stays in Rust 2021 inside the existing Tauri/Skia/Metal renderer stack.
- Continue using existing renderer `DrawInstruction` and `HitTarget` patterns rather than inventing a second rendering path.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary likely implementation seams:
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/renderer/mod.rs`
  - `src-tauri/src/renderer/hit_test.rs`
- Existing draw modules to follow for text/line and hit-target patterns:
  - `src-tauri/src/instructions/draw/draw_tempo.rs`
  - `src-tauri/src/instructions/draw/draw_time_signature.rs`
  - `src-tauri/src/instructions/draw/draw_notehead.rs`
  - `src-tauri/src/instructions/draw/draw_rest.rs`
- Existing parser outputs or contracts to review if tuplet wrapper metadata needs one more renderer-facing field:
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
- If a focused tuplet draw helper is warranted, prefer a single new Rust draw module under `src-tauri/src/instructions/draw/` rather than scattering tuplet chrome logic across unrelated modules.

### Testing Requirements

- Add Rust-level renderer assertions for tuplet bracket and number draw instructions.
- Add at least one case where bracket chrome is omitted and only the number remains.
- Add hit-target coverage showing tuplet bracket/number selection resolves to the tuplet container instead of losing native hit-test fidelity.
- Add a boundary case proving tuplet adornments stay attached to an isolated tuplet unit when surrounding notation exists on both sides of the tuplet boundary.
- Keep Story 3.2 tests renderer-local and hit-test-local. Broader visual spacing fixtures can wait for Story 3.4.

### Established Code Patterns To Follow

- Extend existing backend-owned seams before introducing new ones.
- Keep native draw instructions and hit targets as the source of truth for score interaction.
- Prefer one focused Rust helper/module for a new score-render concern rather than scattering conditional logic through every draw file.
- Keep tuplets attached to the same notation pipeline as ordinary score content, not as overlay annotation.

### Previous Story Intelligence

- Story 3.1 now explicitly proves parser output independently of rendering. Story 3.2 should rely on that contract instead of repeating parser validation through visual guesswork.
- Story 2.5 and Story 2.4 both reinforced the current brownfield pattern: reuse authoritative seams, keep derived behavior backend-owned, and add focused regression coverage close to the module that owns the behavior.
- Existing docs already distinguish printed-score notation interactions from sequencer-only context. Tuplet bracket/number selection therefore belongs naturally in the native renderer path.

### Git Intelligence Summary

- Recent relevant commits:
  - `33a49140` `chore: epic 2 retro`
  - `21b576eb` `chore(2.5): mark story done`
  - `77c7fa2e` `feat(2.5): tuplet aware slice and delete`
  - `6407f59f` `fix: snap expectations.`
  - `ba93d2a7` `Complete Story 2.4 tuplet drag and snap work`
- The current implementation pattern is still incremental and seam-focused. Story 3.2 should introduce the minimum renderer surface needed for correct tuplet chrome without blending in 3.3 or 3.4 scope.

### Documentation Impact

- Review these docs during implementation and update them if the shipped tuplet-rendering contract changes materially:
  - `docs/architecture.md`
  - `docs/data-models.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` only if this work establishes a durable repo-wide rule
- If implementation only fulfills already-documented behavior, record that no documentation update was required in the completion notes rather than skipping the review silently.

### Project Structure Notes

- There is no current dedicated tuplet draw module under `src-tauri/src/instructions/draw/`; that is expected at this stage and makes Story 3.2 the right place to add one if needed.
- Existing draw modules demonstrate the current pattern for native text instructions, line instructions, and hit targets. Follow those patterns rather than inventing a new rendering abstraction.
- Shared-surface composition already preserves draw instructions and hit targets across flows. Tuplet adornments need to survive that same composition path unchanged except for expected coordinate offsets.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 / Story 3.2 acceptance criteria and adjacent story boundaries
- Source: `_bmad-output/planning-artifacts/prd.md` — FR14, FR18, notation credibility goals, and default-engraving-only MVP scope
- Source: `_bmad-output/planning-artifacts/architecture.md` — tuplets as local metric boundaries, wrapper-metadata rendering, spacing strategy, and tuplet selection/hit-testing guidance
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — score view should show empty rest-bearing tuplet content and maintain coherent boundary representation
- Source: `docs/data-models.md` — printed-score renderer owns visible notation materialization and selection targets
- Source: `docs/project-context.md` — native score rendering, shared-surface composition, and renderer/hit-test architecture conventions
- Source: `_bmad-output/implementation-artifacts/3-1-parse-tuplet-spanned-content-into-backend-owned-notation.md` — parser-first contract that Story 3.2 consumes

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Story 3.1 implementation artifact and Epic 3 story boundaries to preserve the parser-first to renderer-first handoff.
- Reviewed PRD notation requirements and MVP scope constraints for bracket and number treatment.
- Reviewed architecture sections covering boundary-aware splitting, wrapper-metadata rendering, spacing strategy, and tuplet hit targets.
- Inspected current renderer, hit-test, and draw-module seams to anchor Story 3.2 in the existing Rust-native rendering path.

### Completion Notes List

- Story 3.2 created with explicit renderer-only scope and an added planning note that tuplet boundaries should be treated like barline-style parse split points so rendering can handle the tuplet as an isolated unit.

### File List

- `_bmad-output/planning-artifacts/epics.md`
- `_bmad-output/implementation-artifacts/3-2-render-tuplet-brackets-and-numbers-correctly.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Change Log

- 2026-03-17: Added an Epic 3.2 acceptance criterion that treats tuplet boundaries like barline-style metric split points for renderer consumption.
- 2026-03-17: Created Story 3.2 implementation artifact and moved Story 3.2 to ready-for-dev.
