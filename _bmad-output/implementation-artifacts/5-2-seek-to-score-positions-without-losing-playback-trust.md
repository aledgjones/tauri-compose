# Story 5.2: Seek To Score Positions Without Losing Playback Trust

Status: backlog

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

- [ ] Task 1: Define the supported seek contract for the backend-owned transport path (AC: 1, 2, 3)
  - [ ] Keep seek backend-owned inside `src-tauri/src/playback/transport.rs`; do not introduce a frontend-owned transport model or a second playback controller.
  - [ ] Treat in-flight seek as a transport rebase event that clears stale timeline work before continuing from the new tick.
  - [ ] Preserve score-first active-flow ownership through `ui.flow_id`.

- [ ] Task 2: Clear stale timeline work before continuing from the new seek position (AC: 1, 3, 4)
  - [ ] Extend `src-tauri/src/playback/engine.rs` and `src-tauri/src/playback/transport.rs` so seek cancels queued transport events and active transport-owned notes from the old timeline.
  - [ ] Ensure the next scheduled work comes from a fresh play-start-style snapshot based on the seek target, not from stale relative frame offsets.
  - [ ] Keep readiness evaluation snapshot-based rather than per-render-loop.

- [ ] Task 3: Keep paused seek truthful and non-destructive (AC: 2)
  - [ ] Ensure paused or stopped seeks update `current_tick` and effective tempo truth immediately without starting playback.
  - [ ] Keep the next play-start snapshot coherent after paused seek.

- [ ] Task 4: Add regression coverage and documentation for the seek contract (AC: 1, 2, 3, 4)
  - [ ] Extend backend tests in `src-tauri/src/playback/transport.rs` and `src-tauri/src/playback/engine.rs` for seek-while-playing, seek-while-paused, repeated seek cycles, and mixed readiness cases.
  - [ ] Update docs if Story 5.2 clarifies durable backend seek semantics or the pause-versus-seek interaction contract.

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

#### Completion Notes List

- Story created and not yet implemented.

#### File List

- `_bmad-output/implementation-artifacts/5-2-seek-to-score-positions-without-losing-playback-trust.md`

#### Change Log

- 2026-04-04: Created Story 5.2 to own seek rebase correctness, old-timeline queue clearing, and truthful paused-seek behavior.
