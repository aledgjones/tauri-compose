# Story 5.2: Seek To Score Positions Without Losing Playback Trust

Status: done

## Story

As a composer,
I want seek to reposition playback coherently,
so that I can navigate the score without stale audio from the old timeline leaking through.

## Acceptance Criteria

1. Given transport is playing with one or more supported routes, when I seek to a new score position, then stale queued transport events and active transport-owned notes from the old timeline are cancelled before playback continues from the new tick.
2. Given transport is paused or stopped, when I seek to a new score position, then tick and tempo truth update immediately to the target position without starting playback or corrupting future play-start snapshots.
3. Given some supported routes are ready and others are not, when I seek during playback, then playback continues from a fresh backend snapshot at the new position for supported routes only, while unresolved routes remain recoverable scheduling no-ops.
4. Given repeated seeks are common during composition, when I perform multiple seek actions while playing or paused, then transport state remains stable, stuck notes do not accumulate, and the supported playback path remains regression-testable.

## Tasks / Subtasks

- [x] Task 1: Define the supported seek contract for the backend-owned transport path (AC: 1, 2, 3)
  - [x] Keep seek backend-owned inside `src-tauri/src/playback/transport.rs`; do not introduce a frontend-owned transport model or a second playback controller.
  - [x] Treat in-flight seek as a transport rebase event that clears stale timeline work before continuing from the new tick.
  - [x] Preserve score-first active-flow ownership through `ui.flow_id`.

- [x] Task 2: Clear stale timeline work before continuing from the new seek position (AC: 1, 3, 4)
  - [x] Extend `src-tauri/src/playback/engine.rs` and `src-tauri/src/playback/transport.rs` so seek cancels queued transport events and active transport-owned notes from the old timeline.
  - [x] Ensure the next scheduled work comes from a fresh play-start-style snapshot based on the seek target, not from stale relative frame offsets.
  - [x] Keep readiness evaluation snapshot-based rather than per-render-loop.

- [x] Task 3: Keep paused seek truthful and non-destructive (AC: 2)
  - [x] Ensure paused or stopped seeks update `current_tick` and effective tempo truth immediately without starting playback.
  - [x] Keep the next play-start snapshot coherent after paused seek.

- [x] Task 4: Add regression coverage and documentation for the seek contract (AC: 1, 2, 3, 4)
  - [x] Extend backend tests in `src-tauri/src/playback/transport.rs` and `src-tauri/src/playback/engine.rs` for seek-while-playing, seek-while-paused, repeated seek cycles, and mixed readiness cases.
  - [x] Update docs if Story 5.2 clarifies durable backend seek semantics or the pause-versus-seek interaction contract.

### Review Findings

- [x] [Review][Patch] Active seek can race with a concurrent restart and orphan a transport thread [src-tauri/src/playback/transport.rs:717]
- [x] [Review][Patch] Clearing shared-slot transport playback can leave old transport notes sounding after seek [src-tauri/src/playback/engine.rs:104]
- [x] [Review][Patch] Paused-seek regression coverage is still missing despite the story claiming it [src-tauri/src/playback/transport.rs:2008]

## Dev Notes

### Story Intent

- Story 5.2 sits directly on top of Story 5.1. Once play/pause/resume hardening is in place, seek becomes the next transport-trust feature that must guarantee old timeline work cannot leak into the new one.

### This Story Owns

- Seek rebase correctness.
- Clearing stale queued transport work and active transport-owned notes on seek.
- Truthful paused seek behavior.

### This Story Does Not Own

- Playback clock-model redesign. That belongs to the follow-up platform hardening story.
- General live-edit rules during playback. Story 5.3 owns that contract.
- Slot exclusivity cleanup for shared-slot routing.

### Current Relevant Seams

- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/playback/engine.rs`
- `src-tauri/src/playback/preparation.rs`
- `src/windows/main/components/transport.tsx`
- `src/services/actions.ts`

### Testing Requirements

- Recommended validation commands:
  - `cargo test transport_seek --manifest-path src-tauri/Cargo.toml`
  - `cargo test transport_events_for_tick --manifest-path src-tauri/Cargo.toml`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

### Documentation Impact

- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`

### Dev Agent Record

#### Agent Model Used

GPT-5.4

#### Debug Log References

- 2026-04-04: Story created after the post-5.0 playback audit to isolate seek correctness from pause/resume hardening.
- 2026-04-04: Implemented `transport_seek_with_states(...)` so active seek reuses the backend-owned play-start snapshot seam after cancelling stale transport-owned engine work.
- 2026-04-04: Added transport and engine regressions covering stopped seek, active seek rebasing, mixed readiness no-ops, repeated seek cycles, and preserving non-transport render activity during transport clears.
- 2026-04-04: Review fixes serialized seek/play handoff under the transport mutex, forced shared-slot runtime stops during transport clears, and added paused-seek plus join-before-restart regressions.

#### Completion Notes List

- Active seek now behaves as a transport-owned rebase: `transport_seek` stops and joins the in-flight transport thread, clears stale queued transport events plus active transport-owned notes, clamps through the active flow, and resumes only from a fresh `prepare_supported_playback(...)` snapshot.
- Stopped and paused seek now republish truthful `current_tick` and effective tempo without arming playback, keeping the next play-start snapshot coherent.
- Added backend regression coverage for stopped seek, active seek restart, mixed readiness handling, repeated seek stability, and engine clearing that preserves non-transport render activity on the same slot.
- Validation passed with `cargo test transport_seek --manifest-path src-tauri/Cargo.toml`, `cargo test transport_pause_with_states_rewinds_to_the_last_queued_tick_when_work_is_still_pending --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`.
- Code review fixes closed the remaining gaps: restart handoff is serialized, transport clears now stop shared runtimes before rebasing, and paused seek has explicit regression coverage.

#### File List

- `_bmad-output/implementation-artifacts/5-2-seek-to-score-positions-without-losing-playback-trust.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `src-tauri/src/playback/engine.rs`
- `src-tauri/src/playback/transport.rs`

#### Change Log

- 2026-04-04: Created Story 5.2 to own seek rebase correctness, old-timeline queue clearing, and truthful paused-seek behavior.
- 2026-04-04: Implemented the backend-owned seek rebase contract, added transport/engine regression coverage, and documented the durable seek-versus-pause semantics.
