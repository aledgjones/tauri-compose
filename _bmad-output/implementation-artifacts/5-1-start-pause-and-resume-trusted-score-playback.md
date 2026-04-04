# Story 5.1: Start, Pause, And Resume Trusted Score Playback

Status: ready-for-dev

## Story

As a composer,
I want playback to start, pause, and resume coherently,
so that repeated transport actions remain trustworthy during real score work.

## Acceptance Criteria

1. Given transport is playing with one or more supported direct-plugin root routes, when I pause or stop playback from the supported workflow, then all transport-owned queued events and active transport notes are cancelled deterministically, no stale note output survives, and tick plus tempo truth remain coherent at the paused position.
2. Given transport is paused, when I resume playback from the current transport surface, then playback restarts from the paused tick on a fresh backend snapshot without duplicate transport threads, stale engine events, stuck notes, or corrupted render-target state.
3. Given repeated transport actions and in-flight mix edits are common during composition, when I perform multiple play/pause/resume cycles and adjust supported volume, pan, mute, or solo values, then the supported playback path remains stable and audible output follows current backend mix truth without requiring a transport restart.
4. Given some routes may still be `Unrouted` or `NeedsAttention`, when playback is started or resumed, then transport state remains coherent, supported routes continue through the product path, and unsupported routes remain recoverable scheduling no-ops rather than collapsing the session.
5. Given the audio output device or sample rate may rebuild the stream, when that happens during or around supported playback, then transport-owned playback resets coherently, stale timing assumptions do not survive the rebuild, and the next play-start uses a fresh backend snapshot.

## Tasks / Subtasks

- [ ] Task 1: Make pause and stop a hard transport-owned reset of engine-owned playback work (AC: 1, 2)
  - [ ] Extend `src-tauri/src/playback/transport.rs` and `src-tauri/src/playback/engine.rs` so pause and stop clear queued transport events, retire active transport-owned notes deterministically, and leave no stale transport work armed after the thread joins.
  - [ ] Preserve the current backend ownership split: transport owns lifecycle and state transitions, while the continuous engine owns queued event consumption and render-active slot state.
  - [ ] Ensure the hard-reset path covers multi-target transport playback, not just one slot.

- [ ] Task 2: Harden repeated start, pause, and resume behavior on the current multi-target transport path (AC: 1, 2, 4)
  - [ ] Keep resume as the existing `transport_play` path from the current tick after `transport_pause`, not as a second mode.
  - [ ] Ensure repeated play/pause/resume cycles remain idempotent and do not leak stale transport threads, stale render-target state, stale queued events, or corrupted shared transport flags.
  - [ ] Reuse `prepare_supported_playback(...)` as the canonical play-start handoff and keep readiness evaluation snapshot-based at play-start rather than per-tick or per-render-loop.
  - [ ] Preserve the current multi-target product path: all supported direct-plugin root routes present at play-start may participate, while unresolved routes remain explicit backend truth and schedule nothing.

- [ ] Task 3: Keep supported live mix truth coherent during active playback (AC: 3)
  - [ ] Ensure `volume`, `pan`, `mute`, and `solo` mutations for supported routes update the actual engine mix state used by in-flight playback rather than only a transport-side mirror.
  - [ ] Keep route-backed mix honesty intact: unsupported route shapes must not pretend to expose live controls they cannot honor.
  - [ ] Prove the supported live-mix path through seam-local backend tests rather than UI-only assertions.

- [ ] Task 4: Define coherent transport behavior across audio device and sample-rate rebuilds (AC: 5)
  - [ ] Treat device or sample-rate changes as transport-owned reset events: clear transport-owned playback state, clear engine transport work, and require a fresh play-start snapshot after rebuild.
  - [ ] Do not allow stale sample-rate assumptions from an earlier play-start snapshot to survive an audio stream rebuild.
  - [ ] Keep `src-tauri/src/playback/audio.rs` as the owner of stream lifetime and rebuild wiring rather than moving that responsibility into transport.

- [ ] Task 5: Add regression coverage and documentation for the durable start/pause/resume contract (AC: 1, 2, 3, 4, 5)
  - [ ] Extend backend tests in `src-tauri/src/playback/transport.rs` and `src-tauri/src/playback/engine.rs` for pause clearing queued work, repeated cycles, resume from paused tick, multi-target hard resets, live mix updates, and device/sample-rate reset behavior.
  - [ ] Extend backend tests around `src-tauri/src/playback/preparation.rs` only as needed to prove the play-start snapshot contract remains stable and readiness is still not polled in the render loop.
  - [ ] Update `docs/architecture.md`, `docs/development-guide.md`, `docs/project-context.md`, and `.github/copilot-instructions.md` so transport-reset and multi-target playback rules stay explicit for later Epic 5 and Epic 6 work.

## Dev Notes

### Story Intent

- Story 5.1 is now the transport-trust hardening pass for the actual product path that emerged from Story 5.0: multi-target transport starts are allowed, unsupported routes schedule nothing, and start/pause/resume must now behave like a professional playback loop rather than a first-audible proof.
- The main objective is hard transport-state correctness. If pause or resume still allows stale queued events, stuck notes, stale sample-rate assumptions, or mix truth drift, later seek and editing work will rest on a weak foundation.

### Previous Story Intelligence

- Story 5.0 established audible transport playback, moved readiness checks to play-start snapshots, and widened the product path to schedule all currently supported direct-plugin root targets together.
- The post-5.0 audit identified the remaining infrastructure gaps this story now owns:
  - pause does not yet guarantee that engine-queued transport events are cancelled;
  - repeated pause/resume cycles need explicit hard-reset guarantees across all transport targets;
  - live mix edits must update the engine path, not just a transport-side mirror;
  - device/sample-rate rebuilds need explicit transport reset semantics.

### This Story Owns

- Hard-stop pause and stop behavior for transport-owned queued work.
- Stable repeated play/pause/resume behavior on the supported multi-target playback path.
- Live supported mix truth during active playback.
- Coherent transport reset semantics around audio device or sample-rate rebuilds.

### This Story Does Not Own

- Seek semantics beyond preserving the foundation needed for Story 5.2.
- General live-edit-during-playback rules. Story 5.3 owns the bounded editing contract.
- Slot exclusivity cleanup for multiple score instruments routed to the same slot. That remains deferred cleanup rather than a gate for this story.
- Expression maps, articulation realism, or broader playback authoring UX.

### Current Relevant Seams

- `src-tauri/src/playback/transport.rs`
  - Owns `transport_play`, `transport_pause`, `transport_seek`, play-start snapshot creation, transport shared atomics, and transport worker lifecycle.
- `src-tauri/src/playback/engine.rs`
  - Owns render-active slot state, queued transport and audition events, and the real-time render path on the audio callback.
- `src-tauri/src/playback/preparation.rs`
  - Owns the canonical play-start snapshot of resolved routes, supported transport targets, route-backed mix truth, and blocker semantics.
- `src-tauri/src/playback/audio.rs`
  - Owns device and sample-rate stream rebuilds; this story must define how transport resets around that seam rather than reassigning ownership.
- `src-tauri/src/score/instruments/mod.rs`
  - Owns score-side `volume`, `pan`, `mute`, and `solo` mutations that must stay coherent with the engine during in-flight playback.

### Implementation Guidance

- Preserve the current backend-owned play-start snapshot contract. Route readiness must continue to be evaluated once at play-start, not in the transport loop and not in the audio callback.
- Fix reset and stuck-note bugs at the transport or engine seam rather than masking them in React or the action layer.
- Treat pause and stop as transport-owned control actions with engine-owned queue cleanup responsibilities. Those actions must leave no latent transport note work behind.
- Keep the frontend transport surface thin: the existing transport UI, action bridge, and RAF polling remain the only supported control surface.
- If device/sample-rate rebuild handling requires stopping playback, make that behavior explicit and deterministic rather than attempting to preserve stale in-flight timing assumptions.

### Risks And Regression Traps

- Clearing render targets without clearing queued transport events will leave stale note work armed after pause or stop.
- Updating only transport-side mix mirrors without updating engine slot mix will make the UI appear truthful while audio output remains stale.
- Letting audio stream rebuilds preserve old play-start sample-rate assumptions risks incorrect event durations and offsets after device changes.
- Reintroducing per-tick or per-render-loop route readiness checks would undermine the performance benefit of the current snapshot model.

### Testing Requirements

- Extend Rust tests near the owning seam in `src-tauri/src/playback/transport.rs` and `src-tauri/src/playback/engine.rs` for:
  - pause clearing queued transport work;
  - repeated play/pause/resume cycles;
  - resume from the paused tick on a fresh snapshot;
  - multi-target hard reset behavior;
  - live mix updates affecting in-flight audio;
  - audio device or sample-rate rebuild reset semantics.
- Extend frontend transport tests only if visible trigger or display semantics change.
- Recommended validation commands:
  - `cargo test transport_pause_with_states --manifest-path src-tauri/Cargo.toml`
  - `cargo test queue_transport_note --manifest-path src-tauri/Cargo.toml`
  - `cargo test prepare_supported_playback --manifest-path src-tauri/Cargo.toml`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `npm run lint`
  - `npm run check-types`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Documentation Impact

- Update docs in the same implementation change if Story 5.1 sharpens pause or stop reset rules, multi-target resume guarantees, live mix truth expectations, or device/sample-rate rebuild behavior.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md`

### Dev Agent Record

#### Agent Model Used

GPT-5.4

#### Debug Log References

- 2026-04-04: Story 5.1 was rewritten after the post-5.0 playback-platform audit to align with the current multi-target transport path and the remaining infrastructure gaps.

#### Completion Notes List

- Story rewritten and not yet implemented.

#### File List

- `_bmad-output/implementation-artifacts/5-1-start-pause-and-resume-trusted-score-playback.md`

#### Change Log

- 2026-04-04: Reopened and rewrote Story 5.1 so it now owns play/pause/resume hardening, live mix truth during active playback, and coherent device/sample-rate reset semantics for the current multi-target transport path.
