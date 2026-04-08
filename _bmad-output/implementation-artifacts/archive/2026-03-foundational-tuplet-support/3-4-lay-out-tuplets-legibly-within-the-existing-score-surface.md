# Story 3.4: Lay Out Tuplets Legibly Within the Existing Score Surface

Status: done

Supersession note 2026-03-19: the original Story 3.4 implementation and acceptance text describe even distribution of tuplet-only extra width across host and local ticks. That historical delivery remains recorded below, but the live contract is now the Gould/LCM proportional distribution defined in the follow-up tech spec `tech-spec-gould-based-tuplet-spacing-distribution.md` and enforced in the Rust spacing seam.

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want tuplets to fit cleanly into the score layout,
so that numbers, brackets, and notes stay readable in normal work.

## Scope Note

- This story owns spacing and layout behavior for tuplets inside the existing native score surface after Stories 3.1 through 3.3 have established parser output, bracket/number chrome, and coherent note/rest/tie/beam rendering.
- Tuplets should fit the existing score spacing model, not create a parallel tuplet-only layout system.
- Story 3.4 assumes the parser and renderer contract already distinguishes written duration from projected host occupancy, and that the isolated anchored local `NotationTrack` from Story 3.1 contains the stable local notation structure needed for spacing to perform the local spacing pass here without redesigning the parser contract.
- Story 3.4 also inherits the tuplet carry-forward invariants via Story 3.1; spacing should consume that established boundary/projection model rather than invent a separate layout-time interpretation.
- Story 3.4 resolves tuplet width by comparing two widths for the governed span: the ambient host spacing already implied by surrounding score layout across that span, and the width required by the tuplet's local content. The implementation should then widen the host span if the tuplet needs more room, or stretch the tuplet-local positions to fill the ambient span if the ambient span is already wider.
- Reuse the existing slot patterns and spread them across the tuplet-governed span rather than adding new tuplet-specific spacing slots unless absolutely necessary.
- New spacing slots are a last resort here. The current project rule is that horizontal spacing slots are for flow spacing only and slot additions should be rare. Story 3.4 should assume tuplets normally do not need new slots.
- This story may widen ambient host spacing when the derived tuplet unit needs more room, or stretch child-local positions when ambient spacing is already sufficient.
- This story does not own new user-facing engraving controls, advanced ratio-specific overrides, or post-MVP layout tuning beyond what is needed for correct default readability.

## Acceptance Criteria

1. Given a score containing supported tuplets, when spacing is calculated, then the implementation compares the ambient host spacing already implied across the governed span against the width required by the tuplet's local content and chooses the wider of the two as the rendered span width.
2. Given tuplets are laid out inside the existing score spacing system, when spacing slots are assigned, then the implementation reuses the existing slot patterns and spreads them across the tuplet-governed span rather than adding new tuplet-specific slots unless absolutely necessary and any new slot requires explicit justification because tuplets should normally fit the current slot model.
3. Given the tuplet-required local width is greater than the ambient host spacing across the governed span, when the tuplet is laid out, then the difference is added back into the host span by distributing the extra width evenly across the governed host ticks so surrounding tracks and adjacent notation stay aligned.
4. Given the ambient host spacing across the governed span is already wider than the tuplet-required local width, when the tuplet is laid out, then the excess width is distributed evenly across the tuplet-local ticks so child notation stretches coherently within the available span and remains aligned with the surrounding score surface.
5. Given tuplets begin or end next to non-tuplet material, when layout and hit targets are refreshed, then number placement, collision avoidance, and targeting remain legible and stable and the adjacent material remains distinguishable.

## Tasks / Subtasks

- [x] Fit tuplet spacing into the existing horizontal-spacing model rather than adding a tuplet-only layout path (AC: 1, 2, 3, 4, 5)
  - [x] Review and extend `src-tauri/src/instructions/parse/get_horizontal_spacing.rs` so tuplet-derived content participates in the current widest-needed-spacing calculation.
  - [x] Calculate the ambient host width already implied across the governed span from the existing spacing model before deciding whether the tuplet needs to widen that span.
  - [x] Run the local spacing pass here against the stable anchored `NotationTrack` exposed by the tuplet contract when comparing tuplet-required width against ambient host width.
  - [x] Preserve the current `HorizontalSpacing` and `Slot` model as the default. Do not add tuplet-specific slots unless implementation proves an existing slot cannot support a required readable outcome.
  - [x] If a new slot turns out to be unavoidable, document why existing slots could not carry the requirement and keep the addition as narrow as possible.
- [x] Reuse existing slot positions and spread them through the tuplet-governed span (AC: 1, 2, 3, 4)
  - [x] Reuse `Slot::NoteSpacing` and other existing slot positions where tuplets need horizontal room, rather than treating tuplet markings as justification for generic new anchors.
  - [x] When the tuplet-required width exceeds the ambient host span width, distribute the extra width evenly across the governed host ticks so the widened base spacing remains shared by aligned tracks.
  - [x] When the ambient host span width already exceeds the tuplet-required width, distribute the excess evenly across the tuplet-local ticks so child notation stretches coherently within the available span.
  - [x] Keep tuplet layout attached to the isolated tuplet unit already established in Stories 3.1 through 3.3.
- [x] Preserve ambient-width expansion and in-span stretching behavior for tuplets (AC: 1, 3, 4, 5)
  - [x] Expand surrounding host spacing when the child notation plus bracket/number needs more width than the ambient governed host span provides.
  - [x] Stretch local tuplet positions to fill the available span when ambient host spacing is already wide enough.
  - [x] Keep adjacent non-tuplet material distinguishable and stable when tuplet width changes ripple through the layout.
- [x] Keep hit targets and layout refresh behavior stable around tuplet boundaries (AC: 5)
  - [x] Ensure tuplet-related hit targets remain aligned after spacing recalculation and shared-surface composition refreshes.
  - [x] Preserve readable number placement and collision stability near adjacent non-tuplet material.
  - [x] Avoid introducing frontend-owned geometry repair logic; layout remains backend-owned.
- [x] Add focused spacing and layout regressions for tuplets in the native score surface (AC: 1, 2, 3, 4, 5)
  - [x] Add tests proving the implementation compares ambient host span width against tuplet-required local width and selects the wider result.
  - [x] Add tests proving tuplets can expand ambient host spacing by distributing the extra width across the governed host ticks when their derived content needs more width.
  - [x] Add tests proving tuplets stretch coherently inside already-wide ambient spacing by distributing the excess across tuplet-local ticks.
  - [x] Add tests proving the implementation reuses existing slots and does not add unnecessary tuplet-specific spacing slots.
  - [x] Add a boundary-adjacent case proving nearby non-tuplet material remains distinguishable and hit targets stay aligned after spacing refresh.
- [x] Validate the touched parser and renderer seams with the standard Rust checks
  - [x] Run focused Rust tests for horizontal spacing, render orchestration, and any hit-target alignment paths affected by the layout change.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` after implementation.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean is a repo rule when Rust files change.
- [x] Review documentation impact before closing the story
  - [x] Confirm whether the shipped tuplet spacing/layout rules change developer-facing docs or guardrails.
  - [x] Update the relevant docs and agent instructions in the same change if the durable contract changes materially.
  - [x] If no docs change is needed, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 3 now forms a continuous chain: 3.1 derives an isolated tuplet unit, 3.2 renders bracket/number chrome around it, 3.3 makes notes/rests/ties/beams cohere, and 3.4 makes that same unit fit legibly into the score surface.
- Your added note belongs here directly: tuplets should not introduce new spacing slots unless absolutely needed. The expected implementation direction is to reuse the current slot model and spread that model over tuplet spacing.
- The width rule is comparative rather than absolute: first determine the ambient host span width already implied by the broader layout, then compare it with the tuplet-local required width, widening the host span only when the tuplet needs more room and otherwise stretching the tuplet content to fill the existing span.
- Story 3.4 should therefore finish the Epic 3 renderer track without opening a second spacing architecture.

### Technical Requirements

- The architecture already defines the spacing rule: if child notation plus bracket needs more width than the ambient consumed host span, host spacing expands; if surrounding host spacing is already wider, child local x positions stretch to fill the available width.
- The implementation should calculate the ambient host width for the governed span from the existing cross-track spacing result, calculate the width required by the tuplet's local content, and then choose the wider span.
- If the tuplet-required width is wider, the added width should be spread evenly across the governed host ticks so the widened base spacing remains shared by aligned tracks.
- If the ambient host width is already wider, the excess should be spread evenly across the tuplet-local ticks so the child notation fills the span coherently without changing the surrounding base spacing.
- The current host spacing model already behaves like a widest-needed-spacing system, and tuplets are expected to fit that model.
- Spacing must consume projected host occupancy explicitly. Story 3.4 must not size tuplet material by treating local written duration as if it were the occupied host width.
- Horizontal spacing slots are for flow spacing only; they are not a generic anchoring mechanism for every marking.
- Prefer reusing an existing slot position when an element should align with current notation but should not push later notation horizontally.
- Story 3.4 should therefore default to existing slots such as `Slot::NoteSpacing`, `Slot::PreNote`, `Slot::Accidentals`, and the current barline/directive slots where relevant, rather than inventing a `Tuplet*` slot family.

### Story 3.1 Contract Dependency

Story 3.4 assumes Story 3.1 exposes a stable anchored local `NotationTrack` that can be passed through a later spacing pass without redesigning parser output.

Story 3.4 therefore owns:

- running the spacing pass against that local `NotationTrack`
- comparing tuplet-required width against ambient host width
- mapping the spaced local unit back into the governed host span

Story 3.4 does not require Story 3.1 to precompute spacing; it only requires Story 3.1 to preserve the local `NotationTrack` and host-span anchor needed to do so later.

### Architecture Compliance

- The architecture's spacing interpretation already describes tuplets as child-local spacing mapped into the consumed host span under the same widest-needed-spacing principle used elsewhere in the score.
- Story 2.4's address-span projection rule is the direct precedent here: layout should project from the local semantic unit into the consumed host span, not recover width from host-tick approximations after flattening.
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
  - a tuplet whose derived content forces the ambient host span to expand by distributing extra width across the governed host ticks
  - a tuplet laid out inside already-wide ambient spacing, where excess width is distributed coherently across the tuplet-local ticks
  - a case proving the implementation compares ambient host width and tuplet-required width before choosing the rendered span width
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
- Extended the shared horizontal-spacing pass so anchored tuplet notation computes a local spacing width and widens only the governed host span when local demand exceeds ambient host width.
- Replaced uniform anchored-track projection with resolved local spacing plus even in-span stretch, and aligned tuplet brackets/hit targets to that resolved spacing map.
- Added focused spacing and tuplet-layout regressions, then ran focused `cargo test` filters, full `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Completion Notes List

- Story 3.4 created with an explicit rule that tuplets should reuse existing spacing slots and only add a new slot if absolutely necessary and justified.
- Implemented ambient-vs-local tuplet width comparison in the shared spacing pass and reused existing `Slot` values only; no tuplet-specific slot family was added.
- Updated anchored tuplet rendering so local note/bracket geometry uses the resolved spacing map, with excess ambient width distributed evenly across local tuplet ticks.
- Added focused regressions for even host-span widening, even local stretching, resolved bracket alignment, and boundary-adjacent tuplet hit-target stability.
- Documentation review completed: no durable docs or agent-instruction update was required because the implementation fulfilled the existing spacing contract rather than changing it.

### File List

- `_bmad-output/implementation-artifacts/3-4-lay-out-tuplets-legibly-within-the-existing-score-surface.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/instructions/draw/draw_tuplet.rs`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`

### Change Log

- 2026-03-17: Added an Epic 3.4 acceptance criterion requiring tuplets to reuse existing slot patterns rather than introduce new tuplet-specific spacing slots unless absolutely necessary.
- 2026-03-17: Created Story 3.4 implementation artifact and moved Story 3.4 to ready-for-dev.
- 2026-03-17: Clarified the Story 3.4 width rule so tuplets compare ambient host span width against tuplet-required local width, widening host spacing only when needed and otherwise stretching local tuplet spacing across the available span.
- 2026-03-17: Implemented tuplet-aware shared horizontal spacing, replaced uniform anchored projection with resolved local spacing, added focused regressions, and validated with Rust tests plus clippy.
- 2026-03-18: Closed review and marked Story 3.4 done; no additional project docs or agent instructions needed updating beyond the recorded documentation review outcome.
- 2026-03-19: Annotated Story 3.4 as historically superseded on tuplet extra-width distribution; the live rule is now Gould/LCM proportional spacing shared across host widening and anchored local stretch.
