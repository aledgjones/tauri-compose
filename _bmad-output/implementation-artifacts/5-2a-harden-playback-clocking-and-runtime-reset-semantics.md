# Story 5.2a: Harden Playback Clocking And Runtime Reset Semantics

Status: done

## Story

As a composer,
I want the playback platform to have predictable timing and reset behavior,
so that later realism and authoring features rest on a trustworthy engine foundation.

## Acceptance Criteria

1. Given supported score playback is running, when scheduling pressure or CPU load varies, then transport scheduling uses a bounded backend-owned clocking approach suitable for MVP trust rather than depending entirely on naive wall-clock sleeping.
2. Given playback is stopped, restarted, or reset after backend-owned transport actions, when transport ownership is rebuilt, then runtime queue state, transport-owned note state, and snapshot timing assumptions reset coherently before the next play-start.
3. Given audio device or sample-rate rebuild semantics interact with playback timing, when those transitions occur, then the backend-owned reset contract remains explicit, regression-tested, and free of stale sample-rate assumptions.
4. Given the product path now supports multiple prepared transport targets, when future stories extend playback, then legacy single-target compatibility residue is reduced or fenced so later work does not silently regress back to single-target behavior.

## Tasks / Subtasks

- [x] Task 1: Strengthen the transport clocking model for MVP trust (AC: 1)
  - [x] Audit the current sleep-thread scheduling path in `src-tauri/src/playback/transport.rs` and replace or harden it with a bounded backend-owned lookahead or equivalent clocking approach that is more resilient under load.
  - [x] Keep the audio callback, stream lifetime, and plugin-session ownership boundaries intact.

- [x] Task 2: Make runtime reset semantics explicit across transport and audio rebuild boundaries (AC: 2, 3)
  - [x] Consolidate reset behavior so stop, restart, and audio rebuild paths clear transport-owned queue state and timing assumptions consistently.
  - [x] Keep reset ownership inside the backend seams rather than spreading it into frontend state or UI recovery logic.

- [x] Task 3: Reduce multi-target compatibility residue that could mislead later stories (AC: 4)
  - [x] Fence or remove legacy single-target compatibility surfaces such as fallback fields that are easy for later work to read accidentally.
  - [x] Preserve the canonical multi-target play-start snapshot contract in `prepare_supported_playback(...)`.

- [x] Task 4: Add performance-sensitive regressions and documentation updates (AC: 1, 2, 3, 4)
  - [x] Extend backend tests and any representative timing or performance checks needed to keep the hardened clocking path honest.
  - [x] Update docs and agent guidance so later Epic 5 and Epic 6 work build on the hardened timing and reset platform rather than reintroducing weaker assumptions.

## Dev Notes

### Story Intent

- Story 5.2a is the platform hardening follow-up after pause/resume and seek trust. It owns infrastructure quality, not score-authoring features.

### This Story Owns

- Playback clocking hardening.
- Explicit runtime reset semantics across transport and audio rebuild boundaries.
- Cleanup or fencing of legacy single-target compatibility residue.

### This Story Does Not Own

- Expression maps, articulation realism, or automation.
- Slot exclusivity cleanup for multiple score instruments sharing a slot.
- New transport UI surfaces or frontend-owned playback state.

### Current Relevant Seams

- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/playback/engine.rs`
- `src-tauri/src/playback/audio.rs`
- `src-tauri/src/playback/preparation.rs`

### Documentation Impact

- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md`

### Dev Agent Record

#### Agent Model Used

GPT-5.4

#### Debug Log References

- 2026-04-04: Story created after the playback-platform audit to isolate clocking, reset semantics, and multi-target compatibility cleanup from the user-facing transport stories.
- 2026-04-04: Implemented bounded transport queue-window scheduling, shared transport reset-state cleanup, explicit audio-rebuild timing rebasing, and multi-target preparation cleanup.

#### Completion Notes List

- Hardened `src-tauri/src/playback/transport.rs` to queue near-future ticks through a bounded backend-owned queue window, using short wait quanta plus lead-frame offsets instead of one full sleep per tick.
- Consolidated transport reset-state cleanup so stop, pause, seek, and blocked start paths rebase `current_tick` and `last_queued_tick` together after engine-owned transport work is cleared.
- Made audio-rebuild reset behavior explicit in `src-tauri/src/playback/audio.rs` by rebasing queue timing state after the rebuild stop/resync wrapper.
- Removed the public `PlaybackPreparation.supported_target` compatibility field and kept `supported_targets` plus `supported_target_for_instrument(...)` as the explicit preparation contract.
- Updated architecture and agent guidance in `docs/architecture.md`, `docs/development-guide.md`, `docs/project-context.md`, and `.github/copilot-instructions.md`.
- Fixed review-found transport teardown regressions so stop/pause/rebuild paths preserve the last lookahead-queued tick for reset decisions and natural end-of-flow waits keep final queued transport work alive until teardown.
- Added regression coverage for lookahead pause rewind, natural end-of-flow pending work, and post-rebuild sample-rate refresh before the next playback snapshot.
- Validation passed with `npm run lint`, `npm run check-types`, `npm run test:js`, `cargo test --lib`, `cargo clippy --all-targets -- -D warnings`, and focused `cargo test playback::` regression coverage.

#### File List

- `_bmad-output/implementation-artifacts/5-2a-harden-playback-clocking-and-runtime-reset-semantics.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.github/copilot-instructions.md`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `src-tauri/src/playback/audio.rs`
- `src-tauri/src/playback/audition.rs`
- `src-tauri/src/playback/preparation.rs`
- `src-tauri/src/playback/transport.rs`

#### Change Log

- 2026-04-04: Created Story 5.2a to own playback clocking hardening, explicit runtime reset semantics, and cleanup of multi-target compatibility residue.
- 2026-04-04: Hardened transport queue-window scheduling, unified backend reset-state cleanup across transport and audio rebuild paths, removed public single-target preparation residue, and updated transport guidance/docs.
- 2026-04-04: Fixed code-review transport teardown regressions and added playback regressions for lookahead pause rewind, natural end drain, and rebuilt sample-rate snapshots.

### Review Findings

- [x] [Review][Patch] Lookahead stop/pause/rebuild teardown can resume from one tick too late after queued transport work is cleared [src-tauri/src/playback/transport.rs:257]
- [x] [Review][Patch] Natural end-of-flow teardown can clear the final lookahead-queued transport note before it sounds [src-tauri/src/playback/transport.rs:605]
- [x] [Review][Patch] Audio rebuild coverage still does not prove the next play-start snapshot uses the rebuilt sample rate instead of stale timing assumptions [src-tauri/src/playback/audio.rs:290]
