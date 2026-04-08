# Story 3.3: Render Tuplet Notes, Rests, Ties, and Beams Coherently

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As an engraving-sensitive composer,
I want tuplet note grouping to read correctly,
so that beams, rests, and ties reinforce the tuplet instead of contradicting it.

## Scope Note

- This story owns the renderer and parser-consumer work needed for tuplet noteheads, derived rests, ties, and beam groups to read as one musically coherent tuplet passage.
- Story 3.3 assumes Story 3.1 already derives parser-owned tuplet notation and Story 3.2 already renders tuplet bracket/number chrome from that derived unit.
- Story 3.3 assumes the Story 3.1 contract carries local semantic address, written duration, host occupancy, and the isolated anchored local `NotationTrack` distinctly enough that beam, tie, and rest consumers do not reconstruct tuplet behavior from host tick arithmetic.
- Story 3.3 should continue the tuplet carry-forward invariants already refined in Story 2.4 and Story 2.5 via the Story 3.1 parser contract rather than restating those rules from scratch.
- Treat tuplet start and end boundaries like barline-style metric split points here as well. By the time 3.3 makes beaming, rest, and tie decisions visible, the passage should already be split at tuplet boundaries so the tuplet can be treated as an isolated rhythmic unit.
- This story may refine parser-consumer logic and rendering behavior for beams, rests, and ties, but it must not reintroduce a continuous unsplit host-only view of the passage.
- This story does not own final spacing and collision polish across the whole score surface. That remains Story 3.4 scope.
- MVP remains single-level only. Do not absorb nested tuplet rendering or advanced engraving overrides.

## Acceptance Criteria

1. Given supported tuplet note groupings, when notation is rendered, then beam decisions reinforce the tuplet grouping and the rendered result does not contradict the tuplet's rhythmic structure.
2. Given tuplets contain rests or partially filled spans, when the score is rendered, then rests appear in positions consistent with the derived tuplet structure and the result remains legible within MVP engraving scope.
3. Given a created tuplet container has no entered notes yet, when the score is rendered, then the governed local span reads as empty rest-bearing tuplet content using base-note-value rests for the tuplet frame and the bracket and number treatment remain coherent with those derived rests.
4. Given a tie enters, exits, or passes through supported tuplet content, when the score is rendered, then tie presentation remains musically coherent and the rendering does not imply approximate binary timing.
5. Given a tone crosses into or out of a tuplet boundary, when notation is rendered, then the boundary split is represented with the same musical logic Compose uses around barlines and ties and the tuplet remains readable as one coherent rhythmic unit.
6. Given tuplet notes, rests, ties, and beams are rendered after derivation, when the renderer consumes the passage, then tuplet start and end boundaries are treated as isolated metric split points in the same way barlines constrain grouping decisions and beaming, rest placement, and tie continuity are resolved from that isolated tuplet unit rather than from a continuous unsplit host view.
7. Given MVP tuplets are non-nestable, when rendering support is implemented, then rendering only needs to cover single-level tuplet content.

## Tasks / Subtasks

[x] Make parser-consumer beaming decisions tuplet-aware without breaking existing non-tuplet grouping (AC: 1, 5, 6, 7)

- [x] Extend `src-tauri/src/instructions/parse/get_beams.rs` and any immediate beam consumers so tuplet-local note groupings are resolved from the isolated tuplet unit produced by parser boundary splitting.
- [x] Preserve existing non-tuplet beam behavior outside tuplets and across unaffected passages.
- [x] Keep MVP scope single-level only; do not introduce nested beaming logic for tuplets.
      [x] Render derived tuplet rests from absence-of-tone content coherently inside the tuplet unit (AC: 2, 3, 5, 6)
- [x] Reuse the existing rest rendering path so derived tuplet rests remain backend-owned and not persisted as explicit rest entries.
- [x] Ensure partially filled tuplets and empty tuplet containers render rest-bearing content that aligns with the derived tuplet unit rather than with unsplit host spacing assumptions.
- [x] Keep rest placement coherent with the bracket/number treatment established in Story 3.2.
      [x] Preserve tie continuity across tuplet boundaries and through derived fragments (AC: 4, 5, 6)
- [x] Reuse existing tie-drawing and tie-identity patterns so tones crossing tuplet boundaries remain one stored event while rendering as coherent tied fragments.
- [x] Ensure the score output does not imply approximate binary timing when a tie enters, exits, or spans a tuplet context.
- [x] Keep boundary-aware tie behavior derived from parser splits, not from renderer-only reconstruction.
      [x] Keep the main Rust draw pipeline coherent when combining tuplet notes, rests, ties, and beams (AC: 1, 2, 3, 4, 5, 6)
- [x] Review the orchestration in `src-tauri/src/instructions/mod.rs` so `get_beams`, `draw_rest`, `draw_ties`, and `draw_beam` consume the tuplet-aware notation track consistently.
- [x] Preserve current notehead, accidental, articulation, and stem-direction semantics except where tuplet-aware grouping requires a targeted change.
- [x] Do not fold broad spacing or collision-polish work into this story.
      [x] Add focused regression coverage for tuplet notes, rests, ties, and beams (AC: 1, 2, 3, 4, 5, 6, 7)
- [x] Add parser-consumer or parse-layer tests for tuplet-local beam grouping decisions where that seam owns the rule.
- [x] Add draw-level tests covering tuplet rests and tie rendering through supported boundary-crossing cases.
- [x] Add at least one empty-tuplet and one partially filled-tuplet case proving derived rests remain coherent with tuplet grouping.
- [x] Add at least one boundary case proving the isolated tuplet unit rule affects beam/rest/tie behavior the same way metric boundaries affect ordinary notation around barlines.
- [x] Add a representative non-tuplet regression so current note, rest, tie, and beam behavior outside tuplets remains unchanged.
      [x] Validate the touched parser and renderer seams with the standard Rust checks
- [x] Run focused Rust tests for the changed parse and draw modules.
- [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` after implementation.
- [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean is a repo rule when Rust files change.
      [x] Review documentation impact before closing the story
- [x] Confirm whether tuplet beam/rest/tie semantics change developer-facing docs or guardrails.
- [x] Update the relevant docs and agent instructions in the same change if the durable contract changes materially.
- [x] If no docs change is needed, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 3 is staged intentionally: Story 3.1 proves parser-owned tuplet derivation, Story 3.2 makes bracket/number chrome visible, and Story 3.3 completes the musical reading by making notes, rests, ties, and beams agree with the tuplet unit.
- The same boundary rule continues here: tuplet starts and ends should constrain grouping the way barlines do. This is especially important in 3.3 because beam grouping, rest placement, and tie continuity can become ambiguous if the renderer falls back to an unsplit host-only view.
- Story 3.3 should therefore complete a continuous contract across the tickets rather than introducing a fresh interpretation.

### Technical Requirements

- Rests remain derived from absence of tones inside the child frame, exactly as they do on host tracks. No explicit rest entries are required in tuplet storage.
- A completely empty triplet should read in the local tuplet metric frame using one base-note rest per local slot. Outside that special empty-triplet case, partially filled tuplets should follow the existing local split-unit behavior for the equivalent local metric frame.
- A tone crossing a tuplet boundary remains one stored event. The split happens when notation, rendering, or playback derives the tone through the tuplet boundary, preserving continuity through ties and projection.
- Beaming, rest placement, and tie behavior must consume the isolated tuplet unit produced by derivation rather than reconstructing grouping from host time alone.
- Tie continuity should continue to rely on tone identity as the continuity mechanism while consuming parser-derived fragment adjacency and semantic span information rather than assuming the continuation lies at host `tick + duration`.
- Preserve current note/chord rendering semantics inside the child frame except where tuplet-local grouping requires a targeted change.
- Do not change persisted-data semantics in this story.

### Story 3.1 Contract Dependency

Story 3.3 assumes Story 3.1 exposes an isolated anchored local `NotationTrack` whose local notations already encode:

- local semantic position through local ticks
- written duration through ordinary `Notation.duration`
- host occupancy through the enclosing tuplet anchor and projection helpers
- tone identity continuity across boundary-derived notation

Story 3.3 should consume that anchored track for beaming, rest placement, and tie continuity rather than translating the passage back into one continuous host-only notation stream.

### Architecture Compliance

- The architecture says tuplets supply the local rhythmic context used when deriving notation fragments, hit targets, ties, and playback timing.
- It also says parser, renderer, and playback projection perform the same kind of boundary-aware split Compose already uses around barlines and ties, with tuplet spans acting as another local metric boundary.
- The architecture explicitly states rests remain derived from absence of tones inside the child frame, and the child frame is enough to support written-duration interpretation, beaming decisions, and spacing layout.
- Story 2.4's address/projection split remains relevant here too: consumers should resolve coherence from the parser's local semantic contract plus host mapping, not by flattening tuplets back into one host-only time scale.
- Story 3.3 must keep those statements true end-to-end for renderer-visible results.

### Library / Framework Requirements

- Work stays in Rust 2021 within the current Tauri/Skia/Metal score-render pipeline.
- Continue using the existing parse and draw module structure.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary likely parser-consumer seams:
  - `src-tauri/src/instructions/parse/get_beams.rs`
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
  - `src-tauri/src/instructions/parse/get_stem_directions.rs`
  - `src-tauri/src/instructions/parse/get_stem_lengths.rs`
- Primary likely draw seams:
  - `src-tauri/src/instructions/draw/draw_beam.rs`
  - `src-tauri/src/instructions/draw/draw_rest.rs`
  - `src-tauri/src/instructions/draw/draw_ties.rs`
  - `src-tauri/src/instructions/mod.rs`
- Existing parser test support to extend when scenario construction is needed:
  - `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`
- Keep Story 3.3 out of broad spacing ownership:
  - do not turn this into a `get_horizontal_spacing.rs` redesign unless a narrow follow-up is unavoidable and directly tied to the accepted ACs

### Testing Requirements

- Add focused Rust tests near the owning seams for beam grouping, derived rests, and tie continuity in tuplet contexts.
- Cover at least these cases:
  - a tuplet with notes that should beam together as one tuplet-local group
  - a partially filled tuplet that derives rest content in the expected local positions
  - an empty tuplet that renders as rest-bearing tuplet content
  - a boundary-crossing tone whose tied fragments remain coherent across host and tuplet contexts
  - a non-tuplet regression preserving current grouping behavior outside tuplets
- Keep Story 3.3 proof close to parser and draw modules rather than deferring it to broad visual fixture work.

### Established Code Patterns To Follow

- Extend the existing parser and draw seams before adding new abstractions.
- Keep derived behavior backend-owned and authoritative.
- Treat tuplet boundaries as grouping boundaries in the same family as barlines for derivation and rendering.
- Preserve one stored tone per event while allowing multiple derived notation fragments at metric boundaries.

### Previous Story Intelligence

- Story 3.1 established the parser-first contract and direct structural validation of tuplet notation output.
- Story 3.2 established that visible tuplet chrome should consume that already split, isolated tuplet unit rather than re-deriving it.
- Story 3.3 should carry the same rule through beams, rests, and ties so the Epic reads as one continuous implementation thread.

### Git Intelligence Summary

- Recent relevant commits:
  - `33a49140` `chore: epic 2 retro`
  - `21b576eb` `chore(2.5): mark story done`
  - `77c7fa2e` `feat(2.5): tuplet aware slice and delete`
  - `6407f59f` `fix: snap expectations.`
  - `ba93d2a7` `Complete Story 2.4 tuplet drag and snap work`
- The repo pattern remains seam-first and incremental. Story 3.3 should refine the existing parser/draw modules rather than introducing a separate tuplet engraving subsystem.

### Documentation Impact

- Review these docs during implementation and update them if the shipped tuplet beam/rest/tie contract changes materially:
  - `docs/architecture.md`
  - `docs/data-models.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` only if this work establishes a durable repo-wide rule
- If implementation only fulfills already-documented behavior, record that no documentation update was required in the completion notes rather than skipping the review silently.

### Project Structure Notes

- Existing `get_beams` and draw helpers already provide the non-tuplet beam/tie/rest path. Story 3.3 should adapt those seams to consume tuplet-aware notation rather than inventing a separate tuplet-specific engraving pipeline.
- `src-tauri/src/instructions/mod.rs` already orchestrates `get_beams`, `draw_ties`, `draw_beam`, and `draw_rest`; review that orchestration carefully before changing any one module in isolation.
- The architecture already contains worked examples where a crossing tone derives a tuplet-side quarter plus tuplet-side rest with tie continuity preserved. That example should anchor expectations for 3.3 behavior.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 3 / Story 3.3 acceptance criteria and adjacent story boundaries
- Source: `_bmad-output/planning-artifacts/prd.md` — FR15, FR16, and notation-coherence expectations for tuplets
- Source: `_bmad-output/planning-artifacts/architecture.md` — boundary-aware splitting, rests-from-absence, tie continuity, and beaming/child-frame guidance
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — empty tuplet containers should read as rest-bearing score content and boundary representation should remain coherent
- Source: `docs/data-models.md` — score renderer owns visible notation materialization for rests and other notation objects
- Source: `docs/project-context.md` — current parse/draw module structure and Rust-owned rendering conventions
- Source: `_bmad-output/implementation-artifacts/3-1-parse-tuplet-spanned-content-into-backend-owned-notation.md` — parser-first contract Story 3.3 consumes
- Source: `_bmad-output/implementation-artifacts/3-2-render-tuplet-brackets-and-numbers-correctly.md` — renderer-boundary contract Story 3.3 continues

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Re-read the current Story 3.1 artifact to preserve continuity with the parser-first contract after recent edits.
- Reviewed Story 3.2 and Epic 3 boundaries so Story 3.3 continues the same isolated-tuplet-unit rule.
- Reviewed architecture guidance for boundary-aware splitting, rests-from-absence, tie continuity, and beaming support.
- Inspected current parse and draw seams around `get_beams`, `draw_beam`, `draw_rest`, `draw_ties`, and the main instruction orchestration.
- Implemented projected tuplet-local render contexts in `src-tauri/src/instructions/mod.rs` so anchored notation now reuses the main draw path for rests, beams, stems, dots, noteheads, articulations, and tuplet chrome.
- Extended `src-tauri/src/instructions/draw/draw_ties.rs` so same-track ties no longer assume continuation always exists at `tick + duration`, and added cross-context tie drawing for host-to-tuplet continuity.
- Added focused parse and draw regressions for tuplet-local beam grouping, empty tuplet rests, projected tuplet noteheads/beams, and host-to-tuplet tie curves.
- Ran focused Rust tests, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Completion Notes List

- Story 3.3 created with the same barline-style boundary rule carried into beams, rests, and ties so Epic 3 remains a continuous implementation thread across 3.1, 3.2, and 3.3.
- Tuplet-local notation now renders through projected child-track spacing instead of bracket-only chrome, so empty and partially filled tuplets emit real rest-bearing and note-bearing score content inside the governed host span.
- Tie rendering now bridges host and anchored tuplet fragments by tone identity and projected fragment order, which preserves boundary continuity without assuming the continuation lives at host `tick + duration`.
- Documentation review completed. No doc or agent-instruction update was required because the shipped behavior matches the existing Story 3.1/3.2 architecture contract rather than changing it materially.

### File List

- `_bmad-output/planning-artifacts/epics.md`
- `_bmad-output/implementation-artifacts/3-3-render-tuplet-notes-rests-ties-and-beams-coherently.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/instructions/draw/draw_ties.rs`
- `src-tauri/src/instructions/parse/get_beams.rs`
- `src-tauri/src/instructions/parse/get_dots.rs`
- `src-tauri/src/instructions/parse/get_articulations.rs`

### Change Log

- 2026-03-17: Added an Epic 3.3 acceptance criterion making tuplet boundaries explicit grouping split points for beam, rest, and tie behavior.
- 2026-03-17: Created Story 3.3 implementation artifact and moved Story 3.3 to ready-for-dev.
- 2026-03-17: Rendered anchored tuplet notation through projected local spacing so tuplets now emit notes, rests, beams, stems, dots, articulations, and bracket chrome as one coherent unit.
- 2026-03-17: Added cross-context tie drawing between host and tuplet fragments and added focused parser/draw regressions plus full Rust validation.
- 2026-03-18: Closed review and marked Story 3.3 done; the documentation review outcome remains that no additional durable doc update was required.
