# Story 4.4: Keep MVP Playback Scoped to the Active Flow

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want tuplets to play back correctly in the active flow without expanding MVP playback scope,
so that the feature ships with the right boundaries.

## Scope Note

- This story owns the active-flow scope boundary for playback after Stories 4.1, 4.2, and 4.3 established projection, mixed ordering, and host-only regression proof.
- Story 4.4 must treat the current single-flow transport path as the active playback architecture. It should keep tuplet playback correct inside that boundary, not broaden transport into cross-flow playback.
- The goal is to prove and preserve that the chosen playback flow is the only flow scheduled, tempo-resolved, clamped, and snapshotted at play start.
- `ui.flow_id` remains the authoritative active-flow slot when present. The existing first-flow fallback when no flow is active remains a narrow safeguard, not a license to merge playback across flows.
- Cross-flow playback expansion, workflow-level playback UX changes, and shared-top-bar flow switching remain out of scope.
- Nested tuplets remain out of scope.

## Acceptance Criteria

1. **Given** MVP playback remains single-flow oriented
   **When** tuplets exist in the active flow
   **Then** playback works correctly for that flow
   **And** the feature does not require broader cross-flow playback changes

2. **Given** tuplets are implemented for active-flow playback
   **When** the playback story is completed
   **Then** the system honors the current single-flow playback boundary
   **And** later playback expansion remains a separate concern

## Tasks / Subtasks

- [x] Audit the current active-flow playback boundary end to end and encode the contract explicitly (AC: 1, 2)
  - [x] Treat `src-tauri/src/playback/transport.rs` as the boundary owner for active-flow selection, playback snapshot construction, flow-length clamping, and transport tempo publication.
  - [x] Confirm that `active_flow_id(...)`, `sync_transport_to_app_state(...)`, `resolve_app_tempo_bpm(...)`, and `transport_play(...)` all follow the same one-flow selection rule.
  - [x] Preserve the current fallback behavior when `ui.flow_id` is `None`: select only the first ordered flow, without widening playback to multiple flows.
- [x] Add focused Rust regressions that prove tuplet playback stays scoped to one flow (AC: 1, 2)
  - [x] Add a multi-flow scenario where the active flow contains tuplets and a sibling flow also contains notes or tuplets, and prove that `collect_scheduled_notes(...)` only schedules the active flow's events.
  - [x] Add proof that non-active flows do not affect flow length, clamped transport position, or tempo publication while active-flow playback is running.
  - [x] Add or extend a fallback test showing that when no active flow is selected, playback still chooses only the first ordered flow rather than combining flows.
  - [x] Reuse the existing transport test helpers such as `make_app_context(...)`, `add_flow_with_tempos(...)`, `make_flow_score_with_track(...)`, `make_flow_score_with_tracks(...)`, and `flatten_scheduled_starts(...)` before introducing any new fixture helpers.
- [x] Fix any uncovered active-flow scope leaks at the narrowest shared seam (AC: 1, 2)
  - [x] If playback scope leaks across flows, tighten the flow-selection and snapshot seams in `transport.rs` rather than adding flow-filtering patches elsewhere.
  - [x] Keep `collect_track_playback_events(...)` and the projected `PlaybackTick` model intact unless the active-flow boundary cannot be enforced without a shared-seam adjustment.
  - [x] Do not add frontend-owned playback state, cross-flow event merging, or transport-local reinterpretation of tuplet semantics.
- [x] Preserve current workflow boundaries while finishing Epic 4 (AC: 2)
  - [x] Keep the current transport command surface unchanged.
  - [x] Avoid UI work such as shared-top-bar flow controls, Sequence workflow redesign, or cross-flow playback toggles.
  - [x] Keep later playback expansion explicitly deferred instead of partially implemented behind hidden conditionals.
- [x] Validate the touched playback seams with standard repo checks
  - [x] Run focused Rust tests for the new active-flow playback regressions.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml` after the playback changes.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` because clippy-clean remains a repo rule for Rust changes.
- [x] Review documentation impact before closing the story
  - [x] Confirm whether preserving active-flow-only playback changes any durable developer-facing contract beyond the already documented `ui.flow_id` and shared-playback-model rules.
  - [x] Update the relevant docs and agent instructions in the same change if the shipped behavior changes materially.
  - [x] If no docs change is required, record that decision explicitly in the completion notes.

## Dev Notes

### Story Foundation

- Epic 4 is closing on scope discipline, not new playback breadth. Story 4.4 protects FR22 by proving that tuplet playback works on the current active-flow path without turning MVP into a broader transport initiative.
- NFR5 still matters here: preserving playback behavior includes preserving the current one-flow boundary while tuplets exist in the score.
- Stories 4.1 through 4.3 already established exact projection, deterministic mixed ordering, and host-only regression proof. Story 4.4 should reuse that architecture and finish the epic by preventing accidental cross-flow expansion.

### Carry-Forward Invariants

- Playback is a consumer of the shared tuplet model, not a second semantics engine.
- Tuplets remain one shared semantic model across persistence, parsing, rendering, and playback.
- `ui.flow_id` is the authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport-facing tempo state.
- Passive score interactions must not change the active flow.
- The transport may fall back to the first ordered flow when no active flow is selected, but that fallback still chooses exactly one flow.
- MVP playback remains single-level only.

### Technical Requirements

- `transport_play(...)` must build a playback snapshot for exactly one flow: the active flow when `ui.flow_id` is set, otherwise the current first-flow fallback.
- The chosen flow must drive all of the following together: `scheduled_notes`, `flow_length`, transport tick clamping, and tempo resolution.
- `collect_scheduled_notes(...)` must continue to walk only the selected flow's player, stave, track, and instrument graph, even when other flows also contain tuplets or ordinary notes.
- Non-active flows must not contribute scheduled events, tempo changes, or length limits to active-flow playback.
- Preserve the exact projected `PlaybackTick` timing model and current note-ordering rules inside the selected flow.
- Do not add cross-flow scheduling, mixed-flow transport snapshots, or workflow-specific flow selectors as part of this story.

### Current Codebase Reality

- `src-tauri/src/playback/transport.rs` already centralizes flow selection through `active_flow_id(app_ctx)` and currently uses that result for app-tempo resolution, transport clamping, and play-start snapshot creation.
- `active_flow_id(...)` currently resolves `app_ctx.ui.flow_id.or_else(|| app_ctx.score.flows.order.first().copied())`, so the transport already has a one-flow fallback contract that should be preserved or tightened, not broadened.
- `transport_play(...)` snapshots `collect_scheduled_notes(&app_ctx.score, flow_id)`, `flow.length`, and `flow_tempo_resolver(&app_ctx.score, flow_id)` from the same chosen flow.
- `collect_scheduled_notes(...)` already iterates the selected flow's `player_ids`, `staves`, and `track_ids`, then gathers projected playback events per track through `collect_track_playback_events(...)`.
- Existing tests already prove some multi-flow transport behavior for tempo publication and first-flow fallback, but they do not yet explicitly prove tuplet-bearing sibling flows stay silent while the active flow plays.

### Architecture Compliance

- The architecture's MVP playback direction is still host-time scheduling over one exact projection seam, with tempo remaining a host-time concern.
- The durable architecture docs also keep `ui.flow_id` as the single authoritative active-flow slot across score, Sequence, and transport tempo resolution.
- Story 4.4 must therefore preserve a one-flow playback snapshot and avoid any design that would merge multiple flows or introduce a second transport-owned flow-selection model.
- Shared-top-bar workflow rules also apply: playback scope changes do not belong in the shared chrome during this story.

### Library / Framework Requirements

- Work stays in Rust 2021 inside the current Tauri playback stack.
- Reuse `transport.rs`, `tuplet_projection.rs`, `PlaybackTick`, and the existing transport test helpers.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary implementation and test seam:
  - `src-tauri/src/playback/transport.rs`
- Shared projection seam to touch only if active-flow isolation requires a small shared-contract adjustment:
  - `src-tauri/src/playback/tuplet_projection.rs`
- Durable rule references:
  - `docs/project-context.md`
  - `docs/development-guide.md`
  - `docs/architecture.md`
  - `.github/copilot-instructions.md`

### Testing Requirements

- Add focused Rust regressions close to the transport seam.
- Cover at least these cases:
  - active flow contains tuplets while a sibling flow also contains notes or tuplets, and only the active flow is scheduled
  - transport clamping and tempo publication stay aligned to the chosen active flow on a multi-flow score
  - fallback with `ui.flow_id = None` still selects only the first ordered flow
  - existing host-only and mixed-passage ordering behavior inside the selected flow remains unchanged
- Prefer extending the current `transport.rs` tests rather than building a disconnected playback test harness.

### Established Code Patterns To Follow

- Extend and refine existing APIs and seams rather than introducing a parallel playback path.
- Keep Rust as the source of truth. Do not add frontend-owned playback flow state.
- Preserve the shared projection seam from Stories 4.1 and 4.2, and the host-only regression proof from Story 4.3.
- Keep story scope disciplined: active-flow boundary proof here, broader playback expansion later.

### Previous Story Intelligence

- Story 4.1 established exact host-time projection and moved transport off flat raw-track note lookup.
- Story 4.2 made mixed ordering explicit and kept MVP playback scoped to the active flow while proving host and tuplet events interleave correctly inside one flow.
- Story 4.3 proved the projected scheduling seam preserves legacy host-only playback behavior and extracted a local audible-note filter helper without broadening transport scope.
- The correct starting point for 4.4 is therefore not a new playback design. It is regression-first verification that the existing one-flow transport seam still holds when multiple flows and tuplets coexist.

### Git Intelligence Summary

- Recent relevant commits:
  - `3c60c495` `feat(4.3): preserve host-only playback behavior`
  - `9e867590` `feat(4.2): finalize tuplet playback ordering`
  - `75e814a2` `fix: app hang due to state access.`
  - `bacb9174` `fix: mcp alignment`
  - `83b127e9` `feat(4.1): tuple playback`
- The current pattern is seam hardening in sequence: projection, then ordering, then host-only regression proof. Story 4.4 should finish that line by hardening the active-flow boundary instead of introducing a broader transport surface.

### Project Context Reference

- `docs/project-context.md` states that `ui.flow_id` is the one authoritative active-flow slot and that playback consumes the shared tuplet model rather than inferring semantics locally.
- `docs/development-guide.md` preserves the same active-flow and shared-playback-model rules in the developer guardrails.
- `docs/architecture.md` records the active-flow ownership boundary and the host-time playback direction that this story must preserve.
- `.github/copilot-instructions.md` reinforces that MVP playback must stay scoped to the active flow.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 4 / Story 4.4 acceptance criteria and scope boundary with Stories 4.1, 4.2, and 4.3
- Source: `_bmad-output/planning-artifacts/prd.md` — FR22 requirement that MVP playback remain single-flow even when tuplets exist in the active flow
- Source: `_bmad-output/planning-artifacts/architecture.md` — MVP playback direction and host-time scheduling seam
- Source: `docs/project-context.md` — authoritative `ui.flow_id` rule and shared playback-model guardrail
- Source: `docs/development-guide.md` — tuplet authoring and playback carry-forward guardrails
- Source: `docs/architecture.md` — active-flow ownership rule across score, Sequence, and transport
- Source: `.github/copilot-instructions.md` — active-flow playback scope rule and shared semantic-model rule
- Source: `_bmad-output/implementation-artifacts/4-1-project-tuplet-timing-into-host-musical-time.md` — previous-story handoff on exact playback projection
- Source: `_bmad-output/implementation-artifacts/4-2-schedule-tuplet-playback-correctly-beside-non-tuplet-material.md` — previous-story handoff on mixed ordering and current scope boundaries
- Source: `_bmad-output/implementation-artifacts/4-3-preserve-existing-non-tuplet-playback-behavior.md` — previous-story handoff on host-only regression proof and stable transport seams
- Source: `/memories/repo/tuplet-timing-invariant.md` — reminder that container geometry and crossing duration remain distinct in shared playback semantics

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 4 / Story 4.4 acceptance criteria in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed the PRD and architecture material for FR22 and the MVP playback boundary.
- Reviewed durable active-flow and playback-model rules in `docs/project-context.md`, `docs/development-guide.md`, `docs/architecture.md`, and `.github/copilot-instructions.md`.
- Reviewed Stories 4.2 and 4.3 to inherit the current playback seam, ordering contract, and regression-proofing pattern.
- Inspected `src-tauri/src/playback/transport.rs` to confirm the current one-flow transport selection path, fallback behavior, snapshot construction, and existing tests.
- Checked recent git history through `feat(4.3): preserve host-only playback behavior` to keep this story aligned with the current implementation sequence.
- Reviewed `/memories/repo/tuplet-timing-invariant.md` to keep the shared timing semantics explicit while defining playback-scope guardrails.
- Added focused transport regressions for active-flow-only note scheduling, active-flow clamp plus tempo publication, and first-flow fallback scheduling in multi-flow scores.
- Ran focused Rust regressions for the new transport coverage, then ran full `cargo test --manifest-path src-tauri/Cargo.toml` and full `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Story 4.4 is explicitly framed as an active-flow boundary-preservation story on top of the existing projection and scheduling seams.
- Current code already selects a single flow for tempo, clamping, and playback snapshot creation; implementation should begin with regression proof and only tighten that seam if a leak is found.
- No upfront durable documentation change is assumed; update docs only if the shipped active-flow contract changes materially.
- Added an explicit transport comment documenting that playback selects exactly one flow: `ui.flow_id` when present, otherwise only the first ordered flow.
- Added multi-flow regressions proving sibling flows stay silent for scheduled notes, do not extend active-flow clamping, and do not override active-flow tempo publication.
- No active-flow scope leak was found in the shared transport seam, so no transport behavior change was required beyond documenting the contract and extending regression coverage.
- No durable documentation or agent-instruction update was required because the shipped behavior did not change materially beyond reinforcing the already documented active-flow-only playback rule.
- Story marked done after focused Rust regressions, full Rust test coverage, and clippy validation passed for the active-flow playback seam.

### File List

- `_bmad-output/implementation-artifacts/4-4-keep-mvp-playback-scoped-to-the-active-flow.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/playback/transport.rs`

## Change Log

- 2026-03-18: Created Story 4.4 with implementation guardrails focused on preserving MVP active-flow playback scope for tuplet-aware transport.
- 2026-03-18: Added active-flow playback regressions in `transport.rs`, documented the one-flow transport contract explicitly, and validated with full Rust test and clippy passes.
- 2026-03-18: Marked Story 4.4 done and committed the validated active-flow playback boundary coverage.
