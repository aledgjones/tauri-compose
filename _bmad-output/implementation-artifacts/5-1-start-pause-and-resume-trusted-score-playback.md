# Story 5.1: Start, Pause, And Resume Trusted Score Playback

Status: blocked

Blocked reason: Story 5.0 is now the explicit prerequisite for audible transport playback. Story 5.1 remains the trust-hardening pass on top of that baseline and should return to review only after Story 5.0 lands or after any prerequisite-driven follow-up fixes are applied.

## Story

As a composer,
I want playback to start, pause, and resume coherently,
so that I can re-audition my score during composition without losing trust in playback state.

## Acceptance Criteria

1. Given the score has a supported audible playback path from the earlier epic, when I start, pause, and resume playback from the supported workflow, then playback state transitions remain coherent, and the app avoids hangs, stuck notes, or corrupted session behavior during those actions.
2. Given repeated transport actions are common during composition, when I perform multiple start, pause, and resume cycles, then the supported playback path remains stable, and the behavior is suitable for representative regression validation rather than one-off demos.
3. Given some score instruments may still be non-ready, when playback is started or resumed, then ready routes continue through the supported playback path, and missing or unresolved routes remain recoverable without collapsing the session.

## Tasks / Subtasks

- [x] Task 1: Harden the backend-owned start, pause, and resume transport contract without widening scope beyond Story 5.1 (AC: 1, 2)
  - [x] Treat resume as the existing supported `transport_play` path from the current tick after `transport_pause`, not as a second frontend-owned or backend-owned transport mode.
  - [x] Keep `src-tauri/src/playback/transport.rs` as the owner of thread lifecycle, current tick, tempo snapshotting, and pause cleanup, while preserving `src-tauri/src/playback/engine.rs` as the realtime playback-plane seam.
  - [x] Ensure pause clears the active transport render target, flushes active audio, joins the prior transport thread, and republishes truthful tempo at the paused tick before the next resume.
  - [x] Ensure repeated play or resume actions remain idempotent and do not leak stale transport threads, stale engine render-target state, or corrupted shared transport flags.

- [x] Task 2: Preserve the existing trust-gated preparation path and mixed readiness behavior during transport actions (AC: 1, 3)
  - [x] Reuse `prepare_supported_playback(...)` in `src-tauri/src/playback/preparation.rs` as the canonical play-start handoff; do not create a transport-local preparation model, a probe-only model, or a frontend-owned readiness cache.
  - [x] Preserve the current single-target-first MVP boundary: at most one supported direct-plugin target is prepared for transport start or resume.
  - [x] Keep explicit `Unrouted` and `NeedsAttention` route truth intact so non-ready instruments remain recoverable without blocking the ready route that is allowed to play.
  - [x] Reject or no-op blocked starts through the existing backend trust gate without corrupting transport state, plugin-session state, or route-state presentation.

- [x] Task 3: Keep transport controls score-first and backend-driven on the frontend surfaces that already exist (AC: 1, 2)
  - [x] Reuse the existing transport surface in `src/windows/main/components/transport.tsx`, the action bridge in `src/services/actions.ts`, and the keyboard shortcuts in `src/windows/main/hooks/use-keyboard-shortcuts.ts` rather than adding a new playback control surface.
  - [x] Keep frontend behavior as a thin trigger and poll layer over backend state via `transport_get_state`; do not add event-heavy transport updates or frontend-owned pause or resume state.
  - [x] Preserve the current score-first workflow and top-bar boundaries: no DAW-like playback workspace, no new transport inspector, and no duplicate transport controls in score-context panels.
  - [x] If UI copy or enablement changes are needed to support trust, keep them compact and aligned with the existing playback-control grammar.

- [x] Task 4: Add regression coverage that proves stable repeated start, pause, and resume behavior on the supported path (AC: 1, 2, 3)
  - [x] Extend backend tests in `src-tauri/src/playback/transport.rs` for repeated play/pause cycles, pause-then-resume from the current tick, blocked-start no-op behavior, and mixed ready versus non-ready route handling.
  - [x] Extend backend tests around `src-tauri/src/playback/preparation.rs` or the owning transport seam to prove blocker semantics remain explicit and do not collapse the ready-route path.
  - [x] Extend frontend tests in `src/windows/main/components/transport.test.tsx` and `src/services/actions.test.ts` only as needed to prove the UI keeps triggering the correct backend commands and presents truthful paused or playing state.
  - [x] Keep assertions deterministic when flattening map-derived playback state in Rust tests; sort derived vectors before asserting when the source is an `FxHashMap`.

- [x] Task 5: Update documentation and agent guidance if Story 5.1 sharpens durable transport-trust rules (AC: 1, 2, 3)
  - [x] Update developer-facing docs if the supported pause or resume semantics, trust-gate rejection behavior, or repeated-cycle guarantees become more explicit after implementation.
  - [x] Update agent guidance if Story 5.1 establishes durable rules about transport-owned versus engine-owned responsibilities, or about how blocked starts should be represented and tested.

## Dev Notes

### Story Intent

- Epic 5 begins the shift from first audible playback proof into transport-trust behavior that composers can lean on during real work.
- Story 5.1 is not a new transport-feature expansion story. It is the trust-hardening pass for the supported start, pause, and resume loop on top of the current single-target-first audible path.
- The output of this story should be a bounded, repeatable, regression-tested transport contract, not a broader playback redesign.

### Epic And Story Context

- Epic 4 proved the first audible path, compact score-first workflow, continuous engine seam, and bounded mix-capability honesty.
- Epic 5 now asks whether that path can survive real composition behaviors such as start, pause, resume, later seek, bounded editing, reopen, and route recovery.
- Story 5.1 is the first trust gate in that sequence: if start, pause, and resume are not coherent, later seek and editing work rests on a weak foundation.

### Previous Story Intelligence

- Story 4.5 kept the playback workflow compact and score-first. Story 5.1 must preserve that shape rather than introducing a playback-specific workspace or explanatory surface.
- Story 4.4 established the current honest route-backed mix baseline. Story 5.1 should not reopen mix-capability scope; it should only preserve mix truth across transport transitions.
- Story 4.3 made `src-tauri/src/playback/engine.rs` the backend-owned realtime playback-plane seam. Transport must continue to own scheduling and state transitions, while the engine owns render-active slot state and queued event consumption.
- Story 4.1 and the follow-up foundation work established `prepare_supported_playback(...)` plus runtime activation as the canonical transport or audition preparation handoff. Story 5.1 must extend that seam, not fork it.

### This Story Owns

- Making the supported start, pause, and resume loop coherent and regression-testable.
- Preserving ready-route playback when some other routes remain non-ready.
- Proving repeated transport actions do not corrupt shared state, engine state, or the supported plugin-session path.
- Clarifying the current supported resume semantics if needed: resume is the play command from the current paused tick on a fresh play-start snapshot.

### This Story Does Not Own

- New seek semantics beyond preserving compatibility with the existing transport state model. Story 5.2 owns seek trust as a feature story.
- General live-edit-during-playback rules. Story 5.3 owns the bounded editing contract.
- Reopen restoration, missing-plugin recovery UX, or broader persistence changes. Stories 5.4 and 5.5 own those concerns.
- Multi-instrument or multi-target transport playback. The MVP remains bounded to one supported direct-plugin target on the current product path.

### Current Relevant Seams

- `src-tauri/src/playback/transport.rs`
  - Owns `transport_play`, `transport_pause`, `transport_seek`, `transport_get_state`, transport shared atomics, and the transport worker thread.
- `src-tauri/src/playback/preparation.rs`
  - Owns the canonical backend transport and audition preparation handoff via `prepare_supported_playback(...)`, including blocker semantics and supported-target resolution.
- `src-tauri/src/playback/runtime_activation.rs`
  - Owns the single-instrument trust-gate status and blocker model that transport start must respect.
- `src-tauri/src/playback/engine.rs`
  - Owns render-active slot tracking, queued live render events, and the continuous playback-plane behavior that transport toggles rather than replaces.
- `src-tauri/src/playback/audio.rs`
  - Owns OS stream lifetime and callback wiring; Story 5.1 must not move that responsibility into transport.
- `src/windows/main/components/transport.tsx`
  - Current top-bar playback control surface; keep it as the supported user-facing transport entry point.
- `src/services/actions.ts`
  - Thin frontend command bridge for `transport_play`, `transport_pause`, `transport_seek`, and `transport_get_state`.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Keyboard-driven transport entry point that must remain aligned with the backend-owned play or pause state.

### Implementation Guidance

- Preserve the current architectural contract that frontend transport controls trigger backend commands and poll lightweight transport state through `requestAnimationFrame`; do not add transport events during playback.
- Keep `transport_pause` non-destructive to tick position so resume can restart from the paused tick while still re-snapshotting score tempo data and playback preparation.
- Treat blocked starts as explicit backend outcomes. If a route is non-ready or the single-instrument trust gate is violated, do not collapse session state, silently invent fallback playback, or mutate score-owned route intent.
- Preserve the rack-first playback model: racks own slot lifecycle, slots own runtime sessions and targets, and instruments only own routes to those existing slots.
- If repeated-cycle bugs appear, fix them at the transport or engine ownership seam rather than masking them in React or action-layer state.
- Keep the current top bar limited to universal controls. Story 5.1 should not answer transport-trust concerns with extra panels or new workflow surfaces.
- Treat resume as a deliberate contract. The current architecture says pause retires the old thread and resume behaves like a fresh playback start from the current tick. Implement and test against that rule explicitly.

### Risks And Regression Traps

- Reintroducing frontend-owned playback truth, paused-state bookkeeping, or a parallel readiness cache would violate the current architecture and make trust bugs harder to reason about.
- Leaving stale transport threads or stale engine render-target state alive across repeated play/pause cycles risks hangs, double-processing, or stuck-note behavior.
- Letting blocked starts mutate engine state, route truth, or plugin-session state would break the recoverable non-ready model required by AC3.
- Broadening Story 5.1 into seek, editing, or multi-target transport work would blur epic sequencing and make regression evidence ambiguous.
- Depending on `FxHashMap` iteration order in transport tests will create false regressions; derived collections should be sorted before stable assertions.

### Testing Requirements

- Extend Rust tests near the owning seam in `src-tauri/src/playback/transport.rs` for:
  - repeated play/pause/resume cycles;
  - pause preserving tick while resume restarts from the paused tick with a fresh snapshot;
  - blocked starts leaving transport stopped and session state uncorrupted;
  - mixed ready and non-ready route scenarios preserving the supported ready path.
- Extend seam-local preparation coverage in `src-tauri/src/playback/preparation.rs` or transport tests if additional blocker or mixed-readiness assertions are required.
- Extend frontend tests in:
  - `src/windows/main/components/transport.test.tsx`
  - `src/services/actions.test.ts`
  - `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
    only when behavior visible at those seams changes.
- Recommended validation commands for implementation:
  - `cargo test transport_play_with_states --manifest-path src-tauri/Cargo.toml`
  - `cargo test prepare_supported_playback --manifest-path src-tauri/Cargo.toml`
  - `npm run test:js -- src/windows/main/components/transport.test.tsx src/services/actions.test.ts src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
  - `npm run lint`
  - `npm run check-types`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Documentation Impact

- Update docs in the same implementation change if Story 5.1 makes the supported pause or resume contract, blocked-start behavior, or repeated-cycle trust guarantees more explicit.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if transport-versus-engine ownership rules need sharper agent guidance

### Architecture Compliance

- Rust remains the sole authoritative owner of route truth, playback preparation, transport state, and transport lifecycle.
- The continuous engine remains the realtime playback-plane seam; transport may select or clear its render target, queue work, and manage scheduling, but it must not become the owner of stream lifetime or live plugin-session identity.
- Frontend transport surfaces stay as thin trigger and display layers over backend state. No second transport model may be introduced in React.
- The supported product path remains truly VST-owned. Do not reintroduce sampler-era or SCVI-era fallback playback, hidden readiness logic, or compatibility-residue ownership.
- Playback remains scoped to the authoritative active flow through `ui.flow_id`; Story 5.1 must not widen active-flow ownership or invent parallel selection state.

### Library And Framework Requirements

- Keep the existing Tauri 2 command bridge pattern: frontend `invoke(...)` calls into Rust `#[tauri::command(rename_all = "snake_case")]` handlers with managed state injected through `tauri::State`.
- Current external Tauri 2 guidance still supports this repo pattern: standard `Mutex`-wrapped managed state is appropriate when guards are not held across `await`, and commands remain the right typed bridge for Rust-owned transport operations.
- No new frontend state library, event bus, transport scheduler dependency, or audio framework should be introduced for Story 5.1.

### File Structure Notes

- Backend transport and playback trust work belongs under `src-tauri/src/playback/`.
- Frontend transport interaction stays under `src/windows/main/` and `src/services/`.
- Do not create new top-level playback folders, new windows, or a transport-specific app surface for this story.

### Git Intelligence Summary

- Recent history shows Epic 4 closing cleanly and transport-trust work opening on top of that baseline:
  - `8756f8dc chore: epic 4 retro`
  - `fdfdbdaf feat(4.5): cleanup`
  - `be1a05fb feat(4.4): mixer actions`
  - `14b8c3a8 chore: update bmad`
  - `f03085e2 chore: story creation`
- Story 5.1 should therefore preserve the compact score-first transport surface from Epic 4 and focus new code inside the existing backend playback seams.

### Latest Technical Information

- Current Tauri 2 documentation continues to endorse the repo’s transport command pattern: use `invoke(...)` from `@tauri-apps/api/core`, keep command names unique, and rely on managed `State` for application-owned backend state.
- Tauri 2 state-management guidance still supports standard `Mutex`-wrapped managed state for synchronous command access so long as guards are not held across async await points. That matches the repo’s current transport command style.
- There is no external framework change that justifies moving transport ownership out of Rust, replacing commands with event-driven control chatter, or replacing the current state-injection pattern for this story.

### References

- `_bmad-output/planning-artifacts/epics.md` - Epic 5 and Story 5.1 acceptance criteria.
- `_bmad-output/planning-artifacts/architecture.md` - transport trust, Rust-owned playback truth, realtime-safe playback-plane, and score-context versus app-scoped boundaries.
- `_bmad-output/planning-artifacts/prd.md` - transport-stable playback goals and technical trust gates.
- `docs/architecture.md` - current transport, preparation, engine, and pause/resume semantics.
- `docs/development-guide.md` - playback seam ownership and transport-preparation guidance.
- `docs/project-context.md` - transport polling, active-flow ownership, and playback-domain ownership rules.
- `/memories/repo/playback-rack-first-model.md` - rack-first route and slot ownership.
- `/memories/repo/continuous-render-engine-ownership.md` - engine ownership boundary and render-target regression trap.
- `/memories/repo/transport-test-ordering.md` - stable assertion guidance for `FxHashMap`-derived playback tests.
- `/memories/repo/epic4-playback-preparation-story-4-1.md` - preserve the shared preparation seam rather than forking a transport-local model.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story creation analysis covered Epic 5 acceptance criteria, Epic 4 carry-forward constraints, current transport and engine seams, readiness-gate ownership, frontend transport triggers, and transport test coverage gaps.
- 2026-04-04: Confirmed from the current codebase that `transport_play`, `transport_pause`, `transport_seek`, and `transport_get_state` already exist in `src-tauri/src/playback/transport.rs`, and that the continuous engine seam in `src-tauri/src/playback/engine.rs` remains the realtime playback-plane owner.
- 2026-04-04: Confirmed from current docs that the intended MVP resume contract is “fresh playback start from the current paused tick,” not a second dedicated resume mode.
- 2026-04-04: Confirmed from current Tauri 2 docs that the repo’s command plus managed-state pattern remains current and does not require a framework-level transport architecture change.
- 2026-04-04: Added `transport_pause_with_states(...)` in `src-tauri/src/playback/transport.rs` so pause cleanup remains transport-owned, always joins any prior transport thread, clears the transport render target, flushes active audio through the command seam, resets stale stop-signal state, and republishes truthful tempo at the paused tick.
- 2026-04-04: Hardened blocked `transport_play` no-op cleanup so failed preparation clears stale shared stop-signal state and transport render-target state while keeping transport stopped.
- 2026-04-04: Added backend regressions for repeated play/pause cycles, blocked-start stale-signal cleanup, resume-from-current-tick tempo snapshotting, pause cleanup, and mixed ready-plus-unrouted preparation behavior.
- 2026-04-04: Verified unchanged frontend transport seams by running the targeted transport/action/keyboard Vitest suite and the full JS suite without UI changes.
- 2026-04-04: Validation results: `npm run test:js` passed (98 files, 1200 tests), `cargo test --manifest-path src-tauri/Cargo.toml` passed (1039 lib tests + 12 release-bin tests), `npm run lint` passed, `npm run check-types` passed, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` passed.
- 2026-04-04: After Epic 5 planning was corrected, Story 5.1 was reclassified from `review` to `blocked` because the audible transport baseline it assumed is now explicitly owned by Story 5.0.

### Completion Notes List

- Hardened blocked-start cleanup in `src-tauri/src/playback/transport.rs` so stale `stop_signal` and transport render-target state are cleared on backend no-op starts.
- Centralized pause cleanup behind `transport_pause_with_states(...)` so pause always joins any prior transport thread, clears the active transport render target, flushes active audio through the command seam, and republishes truthful tempo at the paused tick.
- Proved repeated play/pause stability and resume-from-current-tick tempo snapshotting through new transport regressions, and locked the mixed ready-plus-unrouted preparation path with a seam-local preparation regression.
- Confirmed the existing frontend transport surface, action bridge, and keyboard shortcuts already satisfied Story 5.1 without introducing a new playback control surface or frontend-owned transport state.
- Updated `docs/architecture.md`, `docs/development-guide.md`, and `docs/project-context.md` to make the durable pause-cleanup and blocked-start no-op contract explicit for future agent and maintainer work.
- Blocked review progression until Story 5.0 establishes the explicit audible transport baseline that this story now formally depends on.

### File List

- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/5-1-start-pause-and-resume-trusted-score-playback.md`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `src-tauri/src/playback/preparation.rs`
- `src-tauri/src/playback/transport.rs`

### Change Log

- 2026-04-04: Hardened backend transport pause and blocked-start cleanup, added Story 5.1 regression coverage in the owning Rust seams, validated the unchanged frontend transport surface, and updated transport-trust documentation and guidance.
- 2026-04-04: Reclassified Story 5.1 from `review` to `blocked` after Epic 5 planning was corrected to insert Story 5.0 as the explicit audible transport prerequisite.
