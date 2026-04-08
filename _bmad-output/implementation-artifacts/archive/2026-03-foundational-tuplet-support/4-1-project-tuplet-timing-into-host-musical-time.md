# Story 4.1: Project Tuplet Timing Into Host Musical Time

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want tuplet playback timing to come from the same exact model as notation and persistence,
so that playback reflects the real rhythmic structure I entered.

## Scope Note

- This story is a playback-contract story first and an implementation story second. It owns the host-time projection seam that Epic 4 will build on.
- Story 4.1 must carry forward the same shared tuplet semantic model already established for persistence, parsing, rendering, and spacing. Playback is another consumer of that model, not a second semantics engine.
- The story must not treat parser output or renderer output as literal playback input. The requirement is semantic reuse, not notation-structure reuse.
- Transport code may implement scheduling mechanics, but it must not locally re-derive tuplet ownership, boundary meaning, projection, or continuity through transport-specific inference.
- If playback work exposes a semantic gap, resolve that gap in a shared Rust seam rather than patching `transport.rs` in isolation.
- MVP remains single-level only. Do not absorb nested tuplet timing, cross-flow playback expansion, or broader transport redesign here.
- Story 4.1 owns projection into host musical time. It does not own broader event ordering beside surrounding non-tuplet material beyond what is required to prove the projection seam is correct; that broader ordering work remains Story 4.2 scope.

## Acceptance Criteria

1. **Given** a score containing supported tuplets
   **When** playback scheduling is prepared
   **Then** tuplet child onsets and durations are projected into host musical time from the tuplet container's local frame
   **And** the projection preserves exact timing rather than binary-tick approximation

2. **Given** a tuplet contains multiple notes at different local positions
   **When** their playback timing is resolved
   **Then** each onset and duration is computed from the same backend-owned timing model used by save/load and notation derivation
   **And** no separate playback-only timing interpretation is introduced

3. **Given** a tone extends into or out of a tuplet across a supported boundary
   **When** playback timing is resolved
   **Then** each owned segment is scheduled in the correct context with preserved continuity across the boundary
   **And** the result remains rhythmically exact relative to surrounding non-tuplet material

4. **Given** MVP tuplets are structurally non-nestable
   **When** playback projection is implemented
   **Then** host-time projection only needs to support single-level tuplet content
   **And** nested tuplet timing projection is deferred

## Tasks / Subtasks

- [x] Define and implement one shared playback-facing tuplet projection seam (AC: 1, 2, 3, 4)
  - [x] Encode the carry-forward semantic invariants explicitly: ownership-by-start, local tuplet frame, local-to-host projection, one stored tone per event, and boundary-derived continuity.
  - [x] Keep the semantic model shared with persistence, parsing, and rendering; do not let `transport.rs` become the authority for tuplet meaning.
  - [x] If a new helper or module is needed, give it an obvious Rust-owned domain home and keep it focused on shared tuplet projection semantics rather than transport-only convenience logic.
- [x] Project tuplet-local playback timing into host musical time without binary approximation (AC: 1, 2, 4)
  - [x] Resolve tuplet child tone onsets from local tuplet ticks into host musical time using the established tuplet warp.
  - [x] Resolve projected host durations with exact arithmetic so triplet timing stays exact under host tempo mapping.
  - [x] Keep playback host-time based after projection so existing tempo handling remains a host-level concern.
- [x] Preserve continuity for tones that cross tuplet boundaries (AC: 2, 3)
  - [x] Respect ownership-by-start: host-owned tones remain host-owned, tuplet-owned tones remain tuplet-owned, and crossing the boundary does not force stored-tone splitting.
  - [x] Derive any playback fragments needed at the tuplet boundary from the shared semantic model rather than from raw tick heuristics inside transport.
  - [x] Ensure crossing duration remains literal on the canonical expanded timeline; only container geometry is compressed on the host track.
- [x] Integrate the projection seam into the existing transport path without creating a parallel playback model (AC: 1, 2, 3)
  - [x] Replace or extend the current flat `collect_notes_at_tick(...)` path so playback can see tuplet child content and boundary-aware projected fragments.
  - [x] Preserve the current active-flow transport boundary and existing frontend transport commands; do not introduce a new frontend-owned playback state model or expand playback scope.
  - [x] Keep `transport_play`, `transport_pause`, `transport_stop`, `transport_seek`, and `transport_get_state` stable unless a change is required by the shared projection seam.
- [x] Add focused playback regressions close to the owning seams (AC: 1, 2, 3, 4)
  - [x] Add transport or projection-seam tests for isolated tuplet child notes projected into host time.
  - [x] Add tests for tones that start outside a tuplet and continue into it, and for tones that start inside a tuplet and continue out.
  - [x] Add a representative non-tuplet regression proving the new projection path does not alter existing flat playback behavior.
  - [x] Add at least one mixed-passage case proving projected tuplets remain exact relative to neighboring host material without drift.
- [x] Validate the touched playback seams with standard repo checks
  - [x] Run focused Rust tests for any new projection helper and for the impacted transport tests.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` after the playback changes.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean remains a repo rule for Rust changes.
- [x] Review documentation impact before closing the story
  - [x] Confirm whether the implemented playback projection seam changes durable developer-facing expectations beyond the current newly documented semantic-model rule.
  - [x] Update the relevant docs and agent instructions in the same change if the shipped playback contract changes materially.
  - [x] If no further docs change is required, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 4 begins the playback half of foundational tuplets. Story 4.1 should establish the exact host-time projection contract so Story 4.2 can focus on broader transport ordering and adjacency behavior instead of rediscovering tuplet semantics.
- Epic 3 left the project with a coherent semantic model across parser, renderer, hit-testing, and spacing. The key carry-forward lesson from the Epic 3 retrospective is that playback must inherit that same model explicitly rather than trying to infer it locally from transport-facing raw ticks.
- The user concern to preserve here is knowledge transfer, not literal code reuse. A new-context dev agent should be able to implement Story 4.1 by applying the documented rules without having to re-learn tuplet boundary semantics from scratch.

### Carry-Forward Invariants

- Playback is a consumer of the shared tuplet model, not a second semantics engine.
- Tuplets remain one shared semantic model across persistence, parsing, rendering, and playback.
- Ownership stays with where a tone starts.
- Tuplet-local reasoning and host projection remain distinct concepts.
- Crossing a tuplet boundary does not force a stored-tone split; boundary-aware fragments are derived when a consumer needs them.
- Tuplet geometry is compressed on the host track, but crossing duration is literal across tuplet boundaries on the canonical expanded timeline.
- If playback implementation exposes a semantic gap, fix that gap in a shared Rust seam rather than in transport-local logic.

### Technical Requirements

- Playback should continue to think in host musical time, but tuplet child note events must be projected into host time before scheduling.
- Projection must use exact arithmetic. Binary-tick approximation is explicitly not acceptable for supported triplets.
- Tempo stays pinned to host timeline positions. Tuplets map local notation time into host musical time; tempo maps host musical time into wall-clock time.
- The implementation should preserve one stored tone per event while allowing playback derivation to expose multiple projected segments where boundary continuity requires it.
- Do not make parser-owned notation structs into the hidden playback API. The contract to reuse is semantic, not notation-structure based.
- Preserve the current active-flow playback boundary and `ui.flow_id` authority. Story 4.1 should not broaden playback scope or introduce a parallel flow-selection model.
- MVP remains non-nestable. Nested tuplet timing projection is out of scope.

### Current Codebase Reality

- `src-tauri/src/playback/transport.rs` currently snapshots the score and calls `collect_notes_at_tick(...)`, which walks instrument tracks and reads raw tones via `Track::get_tones_at_tick(&tick)`.
- `Track::get_tones_at_tick(...)` in `src-tauri/src/score/entries/tone.rs` only returns `Entry::Tone` values stored directly at that host tick. It does not expose tuplet child content or any boundary-aware projection.
- Epic 3 already established coherent tuplet timing semantics in parser land. `src-tauri/src/instructions/parse/get_written_durations.rs` carries anchored local tuplet tracks plus host projection metadata, but Story 4.1 must reuse the same semantics without making playback depend on parser-owned notation output.
- This means the current risk is architectural drift: playback can easily fork into its own flat host-tick interpretation unless Story 4.1 explicitly replaces the flat tone-lookup assumption with a shared projection seam.

### Architecture Compliance

- The architecture's playback direction is already explicit: playback remains host-time based, but tuplet child note events must be projected into host time before scheduling.
- The same architecture says boundary splits are introduced only by notation, rendering, or playback derivation. Playback must therefore derive boundary-aware fragments from the same ownership and projection model rather than inventing transport-only exceptions.
- The projection model is already defined: local tuplet ticks map into host time by the tuplet's consumed host span over local printed length.
- The architecture also states that parser, renderer, and playback projection perform the same kind of boundary-aware split Compose already uses around barlines and ties, with tuplet spans acting as another local metric boundary.
- Story 4.1 should therefore extend a shared semantic seam, not couple transport to parser representation and not bury meaning inside scheduling conditionals.

### Library / Framework Requirements

- Work stays in Rust 2021 within the current Tauri playback stack.
- Continue using the existing `transport.rs`, mixer, sampler, and active-flow transport commands unless the projection seam requires a targeted change.
- No new dependency is expected for this story. If exact arithmetic support needs to expand beyond current integer handling, prefer a focused Rust-native approach consistent with the existing codebase over pulling in a large new abstraction casually.

### File Structure Requirements

- Primary likely implementation seam:
  - `src-tauri/src/playback/transport.rs`
- Existing score-domain seam currently used by transport:
  - `src-tauri/src/score/entries/tone.rs`
- Existing shared semantic reference seam to study and align with:
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
- Possible focused home for a shared playback-facing projection helper if required:
  - an existing playback Rust module or one narrowly scoped new Rust file under `src-tauri/src/playback/`
- Frontend transport action surface to preserve unless strictly necessary:
  - `src/services/actions.ts`

### Testing Requirements

- Add playback regression coverage close to the transport or shared projection seam rather than inventing a disconnected playback framework.
- Cover at least these cases:
  - isolated tuplet child note projection into host musical time
  - host-owned tone crossing into a tuplet
  - tuplet-owned tone crossing out of a tuplet
  - mixed tuplet and non-tuplet material with no drift relative to surrounding host notes
  - unchanged non-tuplet playback behavior
- If a shared projection helper is introduced, give it direct unit tests for projection arithmetic and ownership/boundary behavior, then add transport tests proving the helper is actually consumed correctly.

### Established Code Patterns To Follow

- Extend and refine existing APIs and seams rather than introducing a parallel playback path.
- Keep Rust as the source of truth. Do not add frontend-owned timing reconstruction.
- Keep active-flow playback semantics authoritative and centralized.
- Prefer one shared semantic seam for tuplet projection over transport-local ad hoc conditionals.

### Previous Story Intelligence

- Story 3.4 confirmed the Epic 3 rule that tuplets should continue to consume established shared seams instead of creating subsystem-specific alternatives.
- Story 3.1 and its review fixes are directly relevant: boundary-aware projection is easy to regress when one seam starts making local assumptions. Story 4.1 should treat that as a warning for playback.
- Epic 3 retrospective conclusion: playback must consume the same exact backend-owned timing model instead of inventing a transport-only interpretation.
- The newly added durable documentation now states that tuplets are one shared semantic model across persistence, parsing, rendering, and playback, and that playback is a consumer of that model rather than a second semantics engine.

### Git Intelligence Summary

- Recent relevant commits:
  - `60fc01c0` `chore: epic 3 retro`
  - `b7a2bf12` `feat(3.2, 3.3, 3.4): triplet notation`
  - `ef91c9c8` `feat(3.2): splttiing`
  - `7f3b427e` `fix: tuplet timings encoded in agent instructions`
  - `bf7a1808` `chore: update stories to be more procise`
- Recent work reinforced the pattern of codifying durable tuplet rules in project instructions and then consuming those rules incrementally across story seams. Story 4.1 should follow that same discipline.

### Project Context Reference

- `docs/project-context.md` now explicitly states that playback is another consumer of the shared tuplet model and must not locally infer ownership, boundary meaning, projection, or continuity.
- `docs/development-guide.md` now phrases the playback carry-forward rule as: "Playback is a consumer of the shared tuplet model, not a second semantics engine."
- `.github/copilot-instructions.md` now treats tuplets as one shared semantic model across persistence, parsing, rendering, and playback and forbids local subsystem-specific semantic reinference.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 / Story 4.1 acceptance criteria and adjacent playback story boundaries
- Source: `_bmad-output/planning-artifacts/prd.md` — technical success criteria, FR19/FR20, NFR4/NFR7, and the requirement that playback stay aligned with notation and persistence under one exact model
- Source: `_bmad-output/planning-artifacts/architecture.md` — projection model, playback interpretation, MVP playback direction, and playback regression strategy
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — product requirement that notation, editing, and playback emerge from one coherent model and that boundary-crossing tones remain one user-perceived tone across score and playback
- Source: `docs/project-context.md` — transport-facing guardrail that playback must consume the shared tuplet model rather than becoming a second semantics engine
- Source: `docs/development-guide.md` — tuplet authoring and playback carry-forward guardrails
- Source: `.github/copilot-instructions.md` — durable AI-agent rule that tuplets are one shared semantic model across persistence, parsing, rendering, and playback
- Source: `_bmad-output/implementation-artifacts/3-4-lay-out-tuplets-legibly-within-the-existing-score-surface.md` — previous-story intelligence and Epic 3 contract continuity
- Source: `_bmad-output/implementation-artifacts/epic-3-retro-2026-03-18.md` — retrospective outcome that Epic 4 must begin from the established semantic model rather than rediscover it in transport code
- Source: `/memories/repo/tuplet-timing-invariant.md` — canonical reminder that crossing duration is literal across tuplet boundaries while host container geometry remains compressed

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 4 / Story 4.1 acceptance criteria in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed PRD exact-timing, shared-model, and regression requirements for playback.
- Reviewed architecture playback direction, projection model, ownership rules, and regression guidance.
- Reviewed UX guidance that playback behavior should emerge from the same coherent semantic model as editing and notation.
- Reviewed current durable project rules in `.github/copilot-instructions.md`, `docs/project-context.md`, and `docs/development-guide.md`.
- Reviewed Story 3.4 and the Epic 3 retrospective to carry forward the established contract-first seam discipline.
- Inspected current transport and score seams in `src-tauri/src/playback/transport.rs`, `src-tauri/src/score/entries/tone.rs`, and `src/services/actions.ts`.
- Identified the current flat-timing risk: transport still collects raw host-track tones directly and must be updated to consume shared tuplet projection semantics.
- Added `src-tauri/src/playback/tuplet_projection.rs` as the playback-scoped projection seam with exact rational host-time math and boundary-aware fragment derivation.
- Replaced the transport's raw host-tick tone lookup with preprojected scheduled notes grouped by integer host tick plus fractional in-tick offsets.
- Added projection-seam regression coverage for isolated tuplet children, host-owned entry into tuplets, tuplet-owned exits to host time, mixed passages, and flat non-tuplet material.
- Added a transport regression proving projected tuplet events are bucketed by exact host tick rather than the old flat `get_tones_at_tick(...)` path.
- Ran focused Rust tests for `score::tuplet_projection::tests` and `playback::transport::tests::collect_scheduled_notes_groups_projected_tuplet_events_by_exact_host_tick`.
- Ran full Rust validation: `cargo test --manifest-path src-tauri/Cargo.toml` and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story 4.1 is explicitly framed as a shared-semantics playback projection story rather than a transport-local implementation story.
- The story now encodes the carry-forward invariant that playback is a consumer of the shared tuplet model, not a second semantics engine.
- Implemented exact host-time projection in `src-tauri/src/playback/tuplet_projection.rs`, including ownership-by-start handling and boundary-derived fragment splits for tones that enter or exit tuplets.
- Updated `src-tauri/src/playback/transport.rs` to snapshot projected note starts once at play-start, retain host-time tempo handling, and schedule fractional in-tick note onsets without re-deriving tuplet semantics inside transport.
- Added projection and transport regressions covering isolated triplet children, host-owned boundary entry, tuplet-owned boundary exit, mixed passages, and unchanged flat-track playback behavior.
- Documentation impact review: no further docs or agent-instruction updates were required because the shipped behavior matches the already documented shared semantic-model rule rather than changing it materially.

### File List

- `_bmad-output/implementation-artifacts/4-1-project-tuplet-timing-into-host-musical-time.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/playback/mod.rs`
- `src-tauri/src/playback/tuplet_projection.rs`

## Change Log

- 2026-03-18: Added a shared score-domain tuplet playback projection seam, integrated exact projected note scheduling into transport, and added focused playback regressions plus full Rust validation.
