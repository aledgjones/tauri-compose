# Story 3.4: Lay Out Tuplets Legibly Within the Existing Score Surface

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want tuplets to fit cleanly into the score layout,
so that numbers, brackets, and notes stay readable in normal work.

## Scope Note

- This story owns spacing and layout behavior for tuplets inside the existing native score surface after Stories 3.1 through 3.3 have established parser output, bracket/number chrome, and coherent note/rest/tie/beam rendering.
- Tuplets should fit the existing score spacing model, not create a parallel tuplet-only layout system.
- Reuse the existing slot patterns and spread them across the tuplet-governed span rather than adding new tuplet-specific spacing slots unless absolutely necessary.
- New spacing slots are a last resort here. The current project rule is that horizontal spacing slots are for flow spacing only and slot additions should be rare. Story 3.4 should assume tuplets normally do not need new slots.
- This story may widen ambient host spacing when the derived tuplet unit needs more room, or stretch child-local positions when ambient spacing is already sufficient.
- This story does not own new user-facing engraving controls, advanced ratio-specific overrides, or post-MVP layout tuning beyond what is needed for correct default readability.

## Acceptance Criteria

1. Given a score containing supported tuplets, when spacing is calculated, then tuplet-local layout can claim enough width for readable note spacing, brackets, and numbers and surrounding score spacing expands when required by the derived notation.
2. Given tuplets are laid out inside the existing score spacing system, when spacing slots are assigned, then the implementation reuses the existing slot patterns and spreads them across the tuplet-governed span rather than adding new tuplet-specific slots unless absolutely necessary and any new slot requires explicit justification because tuplets should normally fit the current slot model.
3. Given the ambient score context already provides sufficient width, when the tuplet is laid out, then local note positions stretch coherently within the available span and the notation remains aligned with the surrounding score surface.
4. Given tuplets begin or end next to non-tuplet material, when layout and hit targets are refreshed, then number placement, collision avoidance, and targeting remain legible and stable and the adjacent material remains distinguishable.

## Tasks / Subtasks

- [ ] Fit tuplet spacing into the existing horizontal-spacing model rather than adding a tuplet-only layout path (AC: 1, 2, 3, 4)
  - [ ] Review and extend `src-tauri/src/instructions/parse/get_horizontal_spacing.rs` so tuplet-derived content participates in the current widest-needed-spacing calculation.
  - [ ] Preserve the current `HorizontalSpacing` and `Slot` model as the default. Do not add tuplet-specific slots unless implementation proves an existing slot cannot support a required readable outcome.
  - [ ] If a new slot turns out to be unavoidable, document why existing slots could not carry the requirement and keep the addition as narrow as possible.
- [ ] Reuse existing slot positions and spread them through the tuplet-governed span (AC: 1, 2, 3)
  - [ ] Reuse `Slot::NoteSpacing` and other existing slot positions where tuplets need horizontal room, rather than treating tuplet markings as justification for generic new anchors.
  - [ ] Map or stretch local tuplet positions coherently through the consumed host span so the child notation remains aligned with the surrounding score.
  - [ ] Keep tuplet layout attached to the isolated tuplet unit already established in Stories 3.1 through 3.3.
- [ ] Preserve ambient-width expansion and in-span stretching behavior for tuplets (AC: 1, 3, 4)
  - [ ] Expand surrounding host spacing when the child notation plus bracket/number needs more width than the consumed host span provides.
  - [ ] Stretch local tuplet positions to fill the available span when ambient spacing is already wide enough.
  - [ ] Keep adjacent non-tuplet material distinguishable and stable when tuplet width changes ripple through the layout.
- [ ] Keep hit targets and layout refresh behavior stable around tuplet boundaries (AC: 4)
  - [ ] Ensure tuplet-related hit targets remain aligned after spacing recalculation and shared-surface composition refreshes.
  - [ ] Preserve readable number placement and collision stability near adjacent non-tuplet material.
  - [ ] Avoid introducing frontend-owned geometry repair logic; layout remains backend-owned.
- [ ] Add focused spacing and layout regressions for tuplets in the native score surface (AC: 1, 2, 3, 4)
  - [ ] Add tests proving tuplets can expand ambient spacing when their derived content needs more width.
  - [ ] Add tests proving tuplets stretch coherently inside already-wide ambient spacing.
  - [ ] Add tests proving the implementation reuses existing slots and does not add unnecessary tuplet-specific spacing slots.
  - [ ] Add a boundary-adjacent case proving nearby non-tuplet material remains distinguishable and hit targets stay aligned after spacing refresh.
- [ ] Validate the touched parser and renderer seams with the standard Rust checks
  - [ ] Run focused Rust tests for horizontal spacing, render orchestration, and any hit-target alignment paths affected by the layout change.
  - [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml` after implementation.
  - [ ] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean is a repo rule when Rust files change.
- [ ] Review documentation impact before closing the story
  - [ ] Confirm whether the shipped tuplet spacing/layout rules change developer-facing docs or guardrails.
  - [ ] Update the relevant docs and agent instructions in the same change if the durable contract changes materially.
  - [ ] If no docs change is needed, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 3 now forms a continuous chain: 3.1 derives an isolated tuplet unit, 3.2 renders bracket/number chrome around it, 3.3 makes notes/rests/ties/beams cohere, and 3.4 makes that same unit fit legibly into the score surface.
- Your added note belongs here directly: tuplets should not introduce new spacing slots unless absolutely needed. The expected implementation direction is to reuse the current slot model and spread that model over tuplet spacing.
- Story 3.4 should therefore finish the Epic 3 renderer track without opening a second spacing architecture.

### Technical Requirements

- The architecture already defines the spacing rule: if child notation plus bracket needs more width than the ambient consumed host span, host spacing expands; if surrounding host spacing is already wider, child local x positions stretch to fill the available width.
- The current host spacing model already behaves like a widest-needed-spacing system, and tuplets are expected to fit that model.
- Horizontal spacing slots are for flow spacing only; they are not a generic anchoring mechanism for every marking.
- Prefer reusing an existing slot position when an element should align with current notation but should not push later notation horizontally.
- Story 3.4 should therefore default to existing slots such as `Slot::NoteSpacing`, `Slot::PreNote`, `Slot::Accidentals`, and the current barline/directive slots where relevant, rather than inventing a `Tuplet*` slot family.

### Architecture Compliance

- The architecture's spacing interpretation already describes tuplets as child-local spacing mapped into the consumed host span under the same widest-needed-spacing principle used elsewhere in the score.
- Project context explicitly says slot additions should be rare, slots are for elements that consume horizontal flow space, and existing slots should be reused whenever a marking can align without pushing later notation.
- Story 3.4 must keep layout backend-owned in Rust and preserve shared-surface composition plus hit-target stability.
- This story should not overturn the earlier renderer assumption that tuplets are normal notation rendered inside a warped container rather than a separate visual layer.

### Library / Framework Requirements

- Work stays in Rust 2021 within the existing Tauri/Skia/Metal score-render pipeline.
- Continue using the existing `HorizontalSpacing`, `Slot`, and main instruction orchestration structures.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary likely spacing seams:
  - `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
  - `src-tauri/src/instructions/mod.rs`
- Existing draw consumers that rely on spacing spans and should be checked after layout changes:
  - `src-tauri/src/instructions/draw/draw_rest.rs`
  - `src-tauri/src/instructions/draw/draw_ties.rs`
  - `src-tauri/src/instructions/draw/draw_notehead.rs`
  - `src-tauri/src/instructions/draw/draw_beam.rs`
- Existing parser/render contracts to review if spacing needs one additional derived-data input:
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
- Avoid creating a tuplet-only spacing subsystem or a broad new slot family under `get_horizontal_spacing.rs` unless the acceptance criteria genuinely cannot be met otherwise.

### Testing Requirements

- Add focused Rust tests near `get_horizontal_spacing.rs` and any immediate render consumers changed by the spacing work.
- Cover at least these cases:
  - a tuplet whose derived content forces the ambient host span to expand
  - a tuplet laid out inside already-wide ambient spacing, where local positions stretch coherently
  - a case proving existing slots are reused rather than a new slot being added casually
  - a boundary-adjacent case proving nearby non-tuplet material remains legible and hit targets stay aligned
- Keep Story 3.4 validation close to spacing and renderer seams rather than deferring it to broad fixture-only checks.

### Established Code Patterns To Follow

- Extend existing backend-owned seams before adding new abstractions.
- Reuse current slot positions whenever possible.
- Treat new spacing slots as exceptional, not routine.
- Keep tuplets inside the same widest-needed-spacing model as the rest of the score.

### Previous Story Intelligence

- Story 3.2 established that tuplets should be rendered as isolated grouped units bounded by parser splits at tuplet start/end.
- Story 3.3 carried that same isolated-unit rule through beams, rests, and ties.
- Story 3.4 should preserve that unit while solving only the remaining layout and spacing legibility work.

### Git Intelligence Summary

- Recent relevant commits:
  - `33a49140` `chore: epic 2 retro`
  - `21b576eb` `chore(2.5): mark story done`
  - `77c7fa2e` `feat(2.5): tuplet aware slice and delete`
  - `6407f59f` `fix: snap expectations.`
  - `ba93d2a7` `Complete Story 2.4 tuplet drag and snap work`
- The repo pattern remains incremental and seam-focused. Story 3.4 should refine the existing spacing model rather than replacing it.

### Documentation Impact

- Review these docs during implementation and update them if the shipped tuplet spacing/layout contract changes materially:
  - `docs/architecture.md`
  - `docs/data-models.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` only if this work establishes a durable repo-wide rule
- If implementation only fulfills already-documented behavior, record that no documentation update was required in the completion notes rather than skipping the review silently.

### Project Structure Notes

- `get_horizontal_spacing.rs` already defines the `Slot` enum and the width-setting path for the host score. Story 3.4 should prefer adapting that logic instead of introducing a second spacing representation.
- `src-tauri/src/instructions/mod.rs` already visualizes the active slot bars in debugger-oriented code, which makes unnecessary new slots easy to spot during development.
- The current slot inventory ends at `Slot::NoteSpacing`; any addition beyond that should be treated as an exception that must be justified by an acceptance criterion, not by convenience.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 / Story 3.4 acceptance criteria and adjacent story boundaries
- Source: `_bmad-output/planning-artifacts/architecture.md` — spacing interpretation, widest-needed-spacing rule, and tuplet-local width behavior
- Source: `docs/project-context.md` — spacing-slot guardrails and the rule that slot additions should be rare
- Source: `_bmad-output/implementation-artifacts/3-2-render-tuplet-brackets-and-numbers-correctly.md` — isolated tuplet unit and renderer-boundary contract
- Source: `_bmad-output/implementation-artifacts/3-3-render-tuplet-notes-rests-ties-and-beams-coherently.md` — continuity of the isolated-unit rule through note/rest/tie/beam rendering

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed the Epic 3.4 acceptance boundary and the current 3.2 and 3.3 story artifacts to keep spacing work aligned with the isolated-tuplet-unit rule.
- Reviewed architecture spacing guidance for widest-needed-spacing and local-to-host span mapping.
- Reviewed project-context rules for spacing slots and confirmed slot additions are intended to be rare.
- Inspected current `Slot` and `HorizontalSpacing` seams plus the main instruction orchestration to anchor Story 3.4 in the existing spacing model.

### Completion Notes List

- Story 3.4 created with an explicit rule that tuplets should reuse existing spacing slots and only add a new slot if absolutely necessary and justified.

### File List

- `_bmad-output/planning-artifacts/epics.md`
- `_bmad-output/implementation-artifacts/3-4-lay-out-tuplets-legibly-within-the-existing-score-surface.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Change Log

- 2026-03-17: Added an Epic 3.4 acceptance criterion requiring tuplets to reuse existing slot patterns rather than introduce new tuplet-specific spacing slots unless absolutely necessary.
- 2026-03-17: Created Story 3.4 implementation artifact and moved Story 3.4 to ready-for-dev.
