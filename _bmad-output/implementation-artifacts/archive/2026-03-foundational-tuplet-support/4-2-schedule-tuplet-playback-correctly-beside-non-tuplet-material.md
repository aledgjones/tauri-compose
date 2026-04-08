# Story 4.2: Schedule Tuplet Playback Correctly Beside Non-Tuplet Material

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want tuplets to play in the right relationship to surrounding notes,
so that mixed passages sound rhythmically correct.

## Scope Note

- This story owns mixed-passage ordering and adjacency behavior after Story 4.1 established the exact host-time projection seam.
- Story 4.2 must consume the existing playback projection contract rather than redefining tuplet timing locally in transport code.
- The goal is deterministic ordering of projected tuplet events beside surrounding host events on the supported playback path, not a new playback architecture.
- Exhaustive proof that all existing non-tuplet playback remains unchanged belongs to Story 4.3. Story 4.2 should add targeted mixed-passage and adjacency regressions only where they are needed to prove correct interleaving.
- MVP playback scope remains the active flow only. Cross-flow scheduling expansion and workflow-level playback changes remain Story 4.4 or future work.
- Nested tuplet playback remains out of scope.

## Acceptance Criteria

1. **Given** tuplet material appears before, after, or between ordinary non-tuplet notes
   **When** transport ordering is resolved
   **Then** tuplet events are scheduled in the correct host-time order relative to surrounding material
   **And** adjacent non-tuplet playback remains rhythmically correct

2. **Given** a passage mixes tuplets and non-tuplet material on the same supported playback path
   **When** playback runs
   **Then** the tuplet timing does not drift relative to neighboring events
   **And** the result matches the persisted rhythmic intent

## Tasks / Subtasks

- [x] Define the mixed-passage scheduling contract on top of the existing 4.1 projection seam (AC: 1, 2)
  - [x] Treat `src-tauri/src/playback/tuplet_projection.rs` as the authority for exact projected host positions; do not add a second tuplet interpretation inside `transport.rs`.
  - [x] Document and encode the ordering rule for host events and projected tuplet events that share or straddle the same integer host tick bucket.
  - [x] Preserve the existing active-flow transport boundary and current transport command surface while refining event ordering.
- [x] Audit and refine scheduled-note collection so projected tuplets and surrounding host events interleave deterministically (AC: 1, 2)
  - [x] Inspect `collect_scheduled_notes(...)` and the runtime scheduling loop together rather than patching one side in isolation.
  - [x] Ensure bucket assignment, per-bucket ordering, and fractional in-tick deadlines produce the same order implied by projected host musical time.
  - [x] Confirm adjacent events remain correct when host notes occur immediately before a tuplet, immediately after it, and between projected tuplet onsets on the same track or supported playback path.
- [x] Prove mixed passages stay exact without drift over real transport execution paths (AC: 1, 2)
  - [x] Add focused tests for host-only, tuplet-only, and mixed passages that exercise before/after/between adjacency rather than isolated tuplet projection alone.
  - [x] Add at least one case where projected tuplet onsets and ordinary host onsets land in neighboring positions within the same integer tick window, so per-bucket ordering is explicit.
  - [x] Add at least one representative passage showing that later surrounding host notes still land on the correct host tick after projected tuplet notes have been scheduled.
- [x] Keep the 4.1 projection contract intact while extending transport behavior (AC: 1, 2)
  - [x] Reuse the existing `PlaybackTick` rational timing model and avoid collapsing mixed-passage timing back to binary-tick approximation.
  - [x] If a gap appears in scheduling behavior, fix the shared projection or collection seam rather than embedding transport-only tuplet exceptions.
  - [x] Do not broaden playback scope, alter `ui.flow_id` ownership, or introduce frontend-owned playback ordering state.
- [x] Validate the touched playback seams with standard repo checks
  - [x] Run focused Rust tests for the updated mixed-passage scheduling and transport behavior.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` after the playback changes.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean remains a repo rule for Rust changes.
- [x] Review documentation impact before closing the story
  - [x] Confirm whether mixed-passage scheduling changes any durable developer-facing playback contract beyond the already documented shared-model rule.
  - [x] Update the relevant docs and agent instructions in the same change if the shipped contract changes materially.
  - [x] If no docs change is required, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 4 now moves from exact projection into exact interleaving. Story 4.1 established how tuplet-owned and boundary-crossing tones project into host musical time; Story 4.2 must ensure those projected events schedule correctly beside ordinary host material.
- The Epic 3 retrospective explicitly called for early mixed-passage playback regressions so transport ordering mistakes are caught before they spread. This story is the first place that action item becomes concrete.
- The product requirement is not merely that isolated tuplets project correctly, but that mixed passages sound right without asking the transport to infer a second semantics model.

### Carry-Forward Invariants

- Playback is a consumer of the shared tuplet model, not a second semantics engine.
- Tuplets remain one shared semantic model across persistence, parsing, rendering, and playback.
- Projection into host musical time is already owned by the 4.1 seam; Story 4.2 consumes that projection and proves deterministic ordering beside non-tuplet material.
- Tuplet geometry is compressed on the host track, but crossing duration is literal across tuplet boundaries on the canonical expanded timeline.
- Active-flow playback scope remains unchanged.
- MVP remains single-level only.

### Technical Requirements

- Ordering must be determined by projected host musical time, including fractional in-tick offsets, rather than by raw storage order or flat host-tick assumptions.
- Mixed passages must remain exact without drift. A later surrounding host event must still land at its intended host position after projected tuplet events are interleaved.
- The rational `PlaybackTick` model introduced in Story 4.1 should remain the precision seam for mixed scheduling.
- Story 4.2 should avoid broad transport redesign. Extend the existing scheduled-note collection and runtime loop only as far as needed to make mixed-passage ordering explicit and correct.
- This story does not own exhaustive non-tuplet regression coverage or active-flow scope expansion.

### Current Codebase Reality

- `src-tauri/src/playback/tuplet_projection.rs` now emits `PlaybackToneEvent`s with exact rational `PlaybackTick` starts and durations, already sorted by projected host position.
- `src-tauri/src/playback/transport.rs` buckets those projected events in `collect_scheduled_notes(...)` by `start.floor()` and sorts per-bucket by exact `start`, then pitch.
- The runtime transport loop reads `scheduled_notes` for the current integer host tick and sleeps fractional in-tick offsets derived from `note.start.fractional_within_tick()` before rendering each note.
- Story 4.1 already proved isolated projection and one mixed-passage start-order case. Story 4.2 should focus on the remaining transport-level risk: whether collection plus runtime scheduling preserves correct adjacency for ordinary host notes before, after, and between projected tuplet events.
- The likely implementation seam remains narrow: `src-tauri/src/playback/transport.rs` plus focused additions or refinements in `src-tauri/src/playback/tuplet_projection.rs` only if the shared event collection contract is missing information.

### Architecture Compliance

- The architecture already defines the MVP playback seam: collect child note events from tuplets together with host note events outside tuplets, project them into host time using the tuplet warp, then order them alongside host events by projected host position.
- This story should therefore refine or verify that seam rather than bypassing it.
- Boundary-derived continuity rules remain shared with parser and rendering work. If a mixed passage exposes a gap, extend the shared seam instead of adding transport-local heuristics.
- Tempo remains a host-time concern. Story 4.2 should not move tempo logic into tuplet-local reasoning.

### Library / Framework Requirements

- Work stays in Rust 2021 within the current Tauri playback stack.
- Reuse the existing playback modules and exact arithmetic support already introduced in Story 4.1.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary implementation seam:
  - `src-tauri/src/playback/transport.rs`
- Shared projection seam to extend only if the contract is incomplete:
  - `src-tauri/src/playback/tuplet_projection.rs`
- Current durable rule references:
  - `docs/project-context.md`
  - `docs/development-guide.md`
  - `.github/copilot-instructions.md`

### Testing Requirements

- Add focused Rust playback regressions close to the existing playback seam.
- Cover at least these cases:
  - host note immediately before tuplet material
  - host note immediately after tuplet material
  - host note positioned between projected tuplet onsets in a mixed passage
  - mixed passages that cross integer tick buckets without drift
  - unchanged active-flow scheduling assumptions while tuplets are present
- Prefer tests that exercise `collect_scheduled_notes(...)` and the runtime-facing ordering assumptions together, not only isolated projection helpers.

### Established Code Patterns To Follow

- Extend and refine existing APIs and seams rather than introducing a parallel playback path.
- Keep Rust as the source of truth. Do not add frontend-owned playback ordering state.
- Preserve the 4.1 projection seam as the single source of tuplet playback timing.
- Keep story scope disciplined: mixed-passage ordering here, broad flat-playback regression proof in Story 4.3, active-flow playback scope in Story 4.4.

### Previous Story Intelligence

- Story 4.1 already delivered `src-tauri/src/playback/tuplet_projection.rs` and routed transport through exact projected starts and durations instead of raw `Track::get_tones_at_tick(...)` lookups.
- The 4.1 story record explicitly left broader event ordering beside surrounding non-tuplet material to Story 4.2. Do not reopen projection ownership or semantic-model questions that 4.1 already settled.
- Epic 3 retrospective guidance is directly relevant here: add mixed-passage regressions early and preserve the no-parallel-model discipline.

### Git Intelligence Summary

- Recent relevant commits:
  - `75e814a2` `fix: app hang due to stat access`
  - `bacb9174` `fix: mcp alignment`
  - `83b127e9` `feat(4.1): tuple playback`
  - `60fc01c0` `chore: epic 3 retro`
  - `b7a2bf12` `feat(3.2, 3.3, 3.4): triplet notation`
- The key follow-on pattern is clear: recent work keeps strengthening durable rules, then implementing the next seam against those rules rather than inventing local exceptions.

### Project Context Reference

- `docs/project-context.md` states that playback is another consumer of the shared tuplet model and must not locally infer ownership, boundary meaning, projection, or continuity.
- `docs/development-guide.md` phrases the carry-forward rule explicitly: playback is a consumer of the shared tuplet model, not a second semantics engine.
- `.github/copilot-instructions.md` preserves the same rule and keeps MVP playback scoped to the authoritative active flow.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 / Story 4.2 acceptance criteria and scope boundary with Stories 4.1, 4.3, and 4.4
- Source: `_bmad-output/planning-artifacts/prd.md` — FR19, FR20, FR21, FR22 and the playback/sequencing integrity requirements for tuplets beside ordinary material
- Source: `_bmad-output/planning-artifacts/architecture.md` — MVP playback seam, exact host-time projection, ordering alongside host events, and host-time tempo separation
- Source: `docs/project-context.md` — durable rule that playback consumes the shared tuplet model instead of becoming a second semantics engine
- Source: `docs/development-guide.md` — carry-forward playback rule and `ui.flow_id` active-flow guardrail
- Source: `.github/copilot-instructions.md` — shared tuplet semantic-model rule and active-flow playback guardrails
- Source: `_bmad-output/implementation-artifacts/4-1-project-tuplet-timing-into-host-musical-time.md` — previous-story intelligence and explicit handoff that mixed ordering remains Story 4.2 scope
- Source: `_bmad-output/implementation-artifacts/epic-3-retro-2026-03-18.md` — action item to add mixed-passage playback regressions early and preserve no-parallel-model discipline
- Source: `/memories/repo/tuplet-timing-invariant.md` — canonical reminder that container geometry and crossing-duration consumption are distinct
- Source: `/memories/repo/tuplet-tie-projection.md` — carry-forward reminder that projected continuity must remain boundary-aware without mutating persisted ownership

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 4 / Story 4.2 acceptance criteria in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed PRD playback integrity requirements for mixed tuplet and non-tuplet passages.
- Reviewed architecture playback direction, exact host-time projection seam, and required host-event ordering alongside projected tuplet events.
- Reviewed durable project rules in `.github/copilot-instructions.md`, `docs/project-context.md`, and `docs/development-guide.md`.
- Reviewed Story 4.1 to inherit the existing projection seam and preserve the explicit handoff of mixed-ordering scope into Story 4.2.
- Reviewed the Epic 3 retrospective to carry forward the action item about early mixed-passage playback regressions.
- Inspected `src-tauri/src/playback/tuplet_projection.rs` and `src-tauri/src/playback/transport.rs` to identify the concrete mixed-ordering seam and runtime fractional scheduling path.
- Reviewed recent commit history, including `83b127e9 feat(4.1): tuple playback`, to capture current implementation patterns.
- Added an explicit transport comparator for scheduled notes so shared-bucket ordering is documented in code and stays deterministic after projection.
- Added host-only and mixed-passage transport regressions covering unchanged ordinary-note ordering, same-bucket host-versus-triplet ordering, and before/between/after adjacency without drift.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml --lib collect_scheduled_notes` and `cargo test --manifest-path src-tauri/Cargo.toml` after adding the missing host-only regression coverage.

### Implementation Plan

- Keep `tuplet_projection.rs` as the single source of exact host-time projection.
- Make transport's bucket ordering rule explicit with a dedicated comparator instead of repeating ad hoc sort logic.
- Prove mixed passage ordering with focused transport tests that flatten scheduled starts across buckets and validate no-drift adjacency.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story 4.2 is explicitly framed as a mixed-passage transport-ordering story that consumes, rather than redefines, the 4.1 projection seam.
- The story narrows the likely implementation seam to `src-tauri/src/playback/transport.rs` with targeted support from `src-tauri/src/playback/tuplet_projection.rs` only if the shared event-collection contract proves incomplete.
- Documentation impact should remain a conscious closeout check, but no upfront durable docs change is assumed.
- Encoded the mixed-passage transport ordering rule in a dedicated `compare_scheduled_notes(...)` helper so exact projected host time remains the primary sort key and simultaneous starts use stable fallback fields.
- Added focused regressions that prove host-only passages stay in tick order and that host notes immediately before, between, and after projected triplet onsets stay in the expected host-time order without drift.
- Confirmed the durable developer-facing playback contract did not change beyond the already documented shared-model rule, so no doc or agent-instruction update was required for this story.

### File List

- `_bmad-output/implementation-artifacts/4-2-schedule-tuplet-playback-correctly-beside-non-tuplet-material.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/playback/transport.rs`

## Change Log

- 2026-03-18: Created Story 4.2 with implementation guardrails focused on mixed-passage playback ordering beside non-tuplet material.
- 2026-03-18: Encoded deterministic shared-bucket transport ordering and added focused mixed-passage playback regressions for Story 4.2.
- 2026-03-18: Added the missing host-only transport regression so Story 4.2 covers host-only, tuplet-only, and mixed scheduling on the transport seam.
