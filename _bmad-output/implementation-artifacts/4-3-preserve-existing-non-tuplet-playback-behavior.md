# Story 4.3: Preserve Existing Non-Tuplet Playback Behavior

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want existing playback to keep working after tuplets are added,
so that the new timing support does not regress older scores.

## Scope Note

- This story owns regression proof and any narrowly scoped fixes required to keep host-only playback behavior unchanged after Stories 4.1 and 4.2 introduced exact projected scheduling.
- Story 4.3 must treat the current projected-note collection path as the active playback architecture. It should verify and preserve legacy non-tuplet behavior through that path, not reopen pre-tuplet transport architecture.
- The story should focus on host-only passages and existing playback concerns that can regress because transport now consumes `PlaybackTick` projection and per-bucket scheduling.
- Active-flow playback scope remains unchanged and belongs to Story 4.4. Cross-flow expansion remains out of scope.
- Mixed-passage ordering beside tuplets was established in Story 4.2. Story 4.3 should reuse those guarantees and broaden proof only where host-only behavior still lacks coverage.
- Nested tuplets remain out of scope.

## Acceptance Criteria

1. **Given** a score with no tuplets
   **When** playback runs after foundational tuplet support is introduced
   **Then** existing non-tuplet playback behavior remains correct
   **And** the added tuplet timing path does not alter legacy rhythmic output

2. **Given** tuplets are added to the playback implementation
   **When** the transport handles existing non-tuplet scenarios
   **Then** it continues to use the correct current behavior for those passages
   **And** the feature does not force unrelated playback changes

## Tasks / Subtasks

- [x] Audit the current host-only playback path end to end and encode the non-regression contract explicitly (AC: 1, 2)
  - [x] Treat `src-tauri/src/playback/tuplet_projection.rs` and `src-tauri/src/playback/transport.rs` together as the current host-only path, because non-tuplet passages now flow through the same projection seam as tuplet-aware passages.
  - [x] Confirm and document the invariants that must stay true for host-only material: integer-timed `PlaybackTick`s, unchanged bucket ownership via `start.floor()`, deterministic ordering, and unchanged active-flow selection behavior.
  - [x] Keep the story framed as preservation of existing behavior under the new seam, not a transport redesign or a reintroduction of the removed flat `get_tones_at_tick(...)` scheduling path.
- [x] Add focused Rust regressions that prove host-only playback stays unchanged under the projected scheduling seam (AC: 1, 2)
  - [x] Extend the existing playback tests close to the owning seams to cover pure non-tuplet passages across multiple onsets, including at least one case with same-bucket or same-start ordering pressure.
  - [x] Add proof that host-only projection remains integer timed and does not introduce fractional in-tick drift or unexpected re-bucketing.
  - [x] Add at least one host-only scenario that crosses tempo boundaries or starts mid-flow so the projected scheduling seam is verified under current tempo resolution behavior.
  - [x] Add at least one host-only scenario that exercises live mix constraints relevant to legacy playback behavior, such as mute/solo filtering or instrument separation, if current tests do not already prove that seam adequately.
- [x] Fix any uncovered host-only regressions at the narrowest shared seam (AC: 1, 2)
  - [x] If host-only playback differs under the projected path, prefer tightening the shared projection or scheduling seam rather than adding a separate legacy-only playback branch.
  - [x] Preserve `compare_scheduled_notes(...)`, tempo resolution, and the runtime scheduling loop unless a concrete host-only regression proves a change is required.
  - [x] Do not broaden playback scope, introduce frontend-owned playback state, or add transport-local tuplet exceptions while preserving host-only behavior.
- [x] Preserve current transport-facing behavior for legacy passages without unrelated feature movement (AC: 2)
  - [x] Keep the existing transport command surface and active-flow playback boundary stable.
  - [x] Avoid changing unrelated sampler, mixer, SCVI, or file-format behavior.
  - [x] If a safeguard requires additional assertions or helpers, keep them local to the current playback modules and consistent with existing Rust test patterns.
- [x] Validate the touched playback seams with standard repo checks
  - [x] Run focused Rust tests for the updated non-tuplet playback regressions.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` after the playback changes.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean remains a repo rule for Rust changes.
- [x] Review documentation impact before closing the story
  - [x] Confirm whether preserving legacy host-only playback changes any durable playback contract beyond the already documented shared-model rule.
  - [x] Update the relevant docs and agent instructions in the same change if the shipped behavior changes materially.
  - [x] If no docs change is required, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 4 now shifts from establishing exact tuplet projection and mixed-passage ordering to proving that older host-only playback still behaves the same after those changes.
- FR21 and NFR5 are the center of gravity here: adding tuplets must not regress existing non-tuplet playback.
- Story 4.1 already routed playback through exact `PlaybackTick` projection, and Story 4.2 already made mixed ordering explicit. Story 4.3 should treat those seams as established and verify that pure legacy passages still pass through them unchanged.

### Carry-Forward Invariants

- Playback is a consumer of the shared tuplet model, not a second semantics engine.
- Tuplets remain one shared semantic model across persistence, parsing, rendering, and playback.
- Host-only passages should remain integer timed under the shared projection seam; the existence of tuplets elsewhere must not force fractional timing into legacy passages.
- Bucket ownership and runtime order must still be determined by exact projected host musical time, but host-only passages should continue to map cleanly onto their original integer tick schedule.
- Active-flow playback scope remains unchanged.
- MVP remains single-level only.

### Technical Requirements

- Preserve the current projected scheduling architecture introduced in Story 4.1 instead of reviving a parallel legacy transport path.
- Prove that non-tuplet passages still emit `PlaybackTick::from_tick(...)`-equivalent starts and durations where appropriate, so host-only playback keeps integer timing.
- Verify that `collect_scheduled_notes(...)` continues to bucket and order host-only notes correctly now that all playback flows through projected events.
- Verify that tempo resolution for legacy passages remains correct when playback starts at tick zero, mid-flow, or across existing tempo changes.
- Verify that transport runtime concerns relevant to legacy playback, especially mute/solo filtering and instrument separation, still behave correctly for non-tuplet passages under the projected scheduling path.
- Avoid unrelated playback changes. If a fix is required, it should be the smallest shared-seam correction that preserves both host-only and tuplet-aware behavior.

### Current Codebase Reality

- `src-tauri/src/playback/tuplet_projection.rs` is now the shared collection seam for both host-owned tones and tuplet-owned tones. Host-only material already passes through `collect_track_playback_events(...)`, which emits integer `PlaybackTick`s for flat passages.
- `src-tauri/src/playback/transport.rs` now collects projected note events through `collect_scheduled_notes(...)`, buckets them by `start.floor()`, and sorts them with `compare_scheduled_notes(...)`.
- Current transport tests already cover several important seams: flat host-only ordering, shared-bucket ordering beside tuplets, mixed before/between/after adjacency, tempo resolution helpers, and active-flow fallback behavior.
- The remaining likely gap is breadth, not architecture: legacy host-only behavior needs fuller proof across tempo, mix filtering, and deterministic ordering scenarios now that transport consumes projected events universally.

### Architecture Compliance

- The architecture already commits playback to host-time scheduling over a shared exact projection seam. Story 4.3 must preserve non-tuplet behavior within that seam rather than creating a host-only exception path.
- This story is therefore about regression-proofing and narrow seam corrections, not about revisiting the architectural decision to project playback events into host musical time before scheduling.
- Tempo remains a host-level concern, and `ui.flow_id` remains the authoritative active-flow slot.

### Library / Framework Requirements

- Work stays in Rust 2021 inside the current Tauri playback stack.
- Reuse the existing `PlaybackTick`, projected event collection, transport scheduling, and tempo resolver seams.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary implementation and test seam:
  - `src-tauri/src/playback/transport.rs`
- Shared projection seam to adjust only if a host-only regression proves the current contract is incomplete:
  - `src-tauri/src/playback/tuplet_projection.rs`
- Durable rule references:
  - `docs/project-context.md`
  - `docs/development-guide.md`
  - `.github/copilot-instructions.md`

### Testing Requirements

- Add focused Rust regressions close to the existing playback seam.
- Cover at least these cases:
  - host-only passages remain in integer tick order
  - host-only passages do not gain fractional `PlaybackTick` drift
  - host-only playback remains correct across tempo changes or mid-flow start positions
  - host-only playback still respects current mute/solo or instrument-separation behavior where applicable
  - active-flow playback assumptions remain unchanged while tuplets exist elsewhere in the codebase
- Prefer extending the existing helpers in `transport.rs` tests, such as `make_flow_score_with_track(...)`, `make_score_with_tempos(...)`, and `flatten_scheduled_starts(...)`, instead of building a disconnected test harness.

### Established Code Patterns To Follow

- Extend and refine existing APIs and seams rather than introducing a parallel playback path.
- Keep Rust as the source of truth. Do not add frontend-owned playback ordering or timing state.
- Keep `tuplet_projection.rs` as the shared playback collection seam and `transport.rs` as the scheduling/runtime seam.
- Preserve story-scope discipline: host-only regression proof here, active-flow scope boundaries in Story 4.4.

### Previous Story Intelligence

- Story 4.1 established the exact host-time projection seam and explicitly included a representative flat non-tuplet regression, but it did not fully prove broader legacy behavior.
- Story 4.2 finalized deterministic ordering for projected tuplet and host events beside one another, encoded the scheduling comparator explicitly, and added targeted host-only and mixed-passage transport regressions.
- Story 4.2's narrow-seam lesson is the right starting point here: preserve the shared projection and scheduling architecture, then add the host-only proof needed to satisfy FR21 and NFR5.
- Epic 3 retrospective guidance still applies indirectly: prevent playback from drifting into a parallel semantics model, and add focused regressions early where a seam can fail.

### Git Intelligence Summary

- Recent relevant commits:
  - `9e867590` `feat(4.2): finalize tuplet playback ordering`
  - `75e814a2` `fix: app hang due to state access.`
  - `bacb9174` `fix: mcp alignment`
  - `83b127e9` `feat(4.1): tuple playback`
  - `60fc01c0` `chore: epic 3 retro`
- The current pattern is contract-first refinement: 4.1 established the projection seam, 4.2 hardened ordering, and 4.3 should now prove that the same seam preserves older host-only behavior without reopening architecture.

### Project Context Reference

- `docs/project-context.md` states that playback is another consumer of the shared tuplet model and must not locally infer ownership, boundary meaning, projection, or continuity.
- `docs/development-guide.md` phrases the playback carry-forward rule explicitly: "Playback is a consumer of the shared tuplet model, not a second semantics engine."
- `.github/copilot-instructions.md` preserves the same rule and keeps MVP playback scoped to the authoritative active flow.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 / Story 4.3 acceptance criteria and scope boundary with Stories 4.1, 4.2, and 4.4
- Source: `_bmad-output/planning-artifacts/prd.md` — FR21 and NFR5 requirements that existing non-tuplet playback remain correct after tuplets ship
- Source: `_bmad-output/planning-artifacts/architecture.md` — playback remains host-time based through one exact projection seam rather than separate transport semantics
- Source: `docs/project-context.md` — durable rule that playback consumes the shared tuplet model instead of becoming a second semantics engine
- Source: `docs/development-guide.md` — carry-forward playback rule and `ui.flow_id` active-flow guardrail
- Source: `.github/copilot-instructions.md` — shared tuplet semantic-model rule and active-flow playback guardrails
- Source: `_bmad-output/implementation-artifacts/4-1-project-tuplet-timing-into-host-musical-time.md` — previous-story handoff that 4.1 established the projection seam but not exhaustive host-only proof
- Source: `_bmad-output/implementation-artifacts/4-2-schedule-tuplet-playback-correctly-beside-non-tuplet-material.md` — previous-story intelligence on the explicit transport ordering seam and current regression corpus
- Source: `_bmad-output/implementation-artifacts/epic-3-retro-2026-03-18.md` — action item to add focused playback regressions early and preserve the no-parallel-model discipline
- Source: `/memories/repo/tuplet-timing-invariant.md` — canonical reminder that host geometry and literal crossing-duration consumption remain distinct

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 4 / Story 4.3 acceptance criteria in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed PRD and architecture requirements for preserving existing non-tuplet playback behavior while keeping playback on the shared exact-timing seam.
- Reviewed durable playback guardrails in `.github/copilot-instructions.md`, `docs/project-context.md`, and `docs/development-guide.md`.
- Reviewed Story 4.1 to inherit the exact host-time projection seam and Story 4.2 to inherit the deterministic ordering seam and current transport regression baseline.
- Reviewed the Epic 3 retrospective to keep the playback work contract-first and regression-driven.
- Inspected `src-tauri/src/playback/transport.rs` and `src-tauri/src/playback/tuplet_projection.rs` to identify the current host-only playback path, existing tests, and likely regression gaps.
- Reviewed recent commit history through `feat(4.2): finalize tuplet playback ordering` to capture the latest implementation pattern and narrow the likely change surface.
- Ran a dedicated code-exploration pass over the playback modules to confirm that host-only passages already flow through projected event collection and that Story 4.3 should expand regression proof rather than introduce a second transport path.
- Added host-only playback regressions in `src-tauri/src/playback/tuplet_projection.rs` and `src-tauri/src/playback/transport.rs` covering integer timing, same-start deterministic ordering, tempo-onset alignment, and live mute/solo filtering.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml --lib collect_scheduled_notes`, `cargo test --manifest-path src-tauri/Cargo.toml --lib collect_track_playback_events_keeps_host_only_passages_integer_timed`, and `cargo test --manifest-path src-tauri/Cargo.toml --lib filter_audible_scheduled_notes_respects_mute_and_solo_for_host_only_passages` during the focused regression pass.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml` and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` after the changes.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story 4.3 is explicitly framed as a host-only playback preservation story on top of the existing projected scheduling seam from Stories 4.1 and 4.2.
- Added direct host-only regression proof at the projection seam for integer `PlaybackTick` starts and durations, plus transport regressions for same-start ordering pressure, tempo-onset alignment, and live mute/solo filtering.
- Extracted a local `filter_audible_scheduled_notes(...)` helper inside `src-tauri/src/playback/transport.rs` so runtime filtering and host-only regression coverage share the same seam without changing transport scope or command surfaces.
- No host-only playback defect was uncovered in the shared projection or scheduling architecture; the implementation remained on the existing seam and did not reintroduce a legacy transport path.
- No durable documentation or agent-instruction update was required because the shipped playback contract did not change materially; this story only tightened regression proof and extracted a local transport helper.

### File List

- `_bmad-output/implementation-artifacts/4-3-preserve-existing-non-tuplet-playback-behavior.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/playback/tuplet_projection.rs`

## Change Log

- 2026-03-18: Created Story 4.3 with implementation guardrails focused on preserving legacy host-only playback behavior under the shared projected scheduling seam.
- 2026-03-18: Added host-only playback regression coverage across projection, scheduling, tempo-onset alignment, and live mix filtering; extracted a local audible-note filter helper; validated with focused Rust tests, full `cargo test`, and full `cargo clippy`.
