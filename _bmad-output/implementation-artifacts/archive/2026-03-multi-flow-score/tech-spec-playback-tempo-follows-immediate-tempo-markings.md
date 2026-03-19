---
title: "Playback Tempo Follows Immediate Tempo Markings"
slug: "playback-tempo-follows-immediate-tempo-markings"
created: "2026-03-08"
status: "Completed"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
	- "Rust 2021 + Tauri 2 transport/playback backend"
	- "TypeScript 5 frontend action/state layer"
	- "Vitest frontend unit tests and inline Rust #[cfg(test)] module tests"
files_to_modify:
	- "src-tauri/src/playback/transport.rs"
	- "src-tauri/src/score/entries/tempo.rs"
	- "src-tauri/src/lib.rs"
	- "src/services/actions.ts"
	- "src/services/actions.test.ts"
	- "src/services/state/context.tsx"
  - "src/services/state/provider.tsx"
	- "docs/architecture.md"
  - "tests/fixtures/README.md"
code_patterns:
	- "Playback snapshots score and sampler state once at transport start, then advances tick-by-tick on a dedicated thread"
	- "Transport tempo is exposed through TransportShared atomics and surfaced to the frontend via transport_get_state"
	- "Score-level tempo directives live on the flow master track and are queried by tick through Track tempo helpers"
	- "Frontend transport commands are thin invoke wrappers in actions.ts and are verified with payload assertions in actions.test.ts"
	- "Transport defaults are mirrored in frontend initial state, so display defaults should stay aligned with backend fallback semantics"
test_patterns:
	- "Frontend transport wrappers use Vitest invoke-mock assertions in src/services/actions.test.ts"
	- "Frontend transport context defaults are covered with provider/hook tests near src/services/state/context.tsx"
	- "Rust score entry behavior is tested inline next to the entry module"
	- "Rust playback behavior should use focused inline tests in transport.rs for tempo resolution and fallback semantics"
---

# Tech-Spec: Playback Tempo Follows Immediate Tempo Markings

**Created:** 2026-03-08

## Overview

### Problem Statement

Immediate tempo markings now exist in score data and render correctly, but playback still runs from a single transport tempo value instead of following score tempo entries. This makes playback inaccurate when a score defines an absolute tempo at tick 0, contains later absolute tempo changes, or starts playback from a non-zero tick where the active tempo should be inherited from an earlier marking.

Because this logic runs in the transport path that feeds note scheduling for live playback, any solution that adds avoidable overhead, blocking, lock contention, or repeated expensive lookups risks timing instability and audible jitter. Performance on the playback path is therefore a non-negotiable requirement.

### Solution

Update playback to resolve tempo from score tempo entries on the flow master track, applying immediate absolute tempo changes as exact step changes at tick boundaries. On playback start or seek, resolve the active tempo from the most recent applicable score tempo at-or-before the current tick. When no score tempo applies, synthesize a temporary default tempo for transport use only so playback follows one consistent tempo-resolution path without mutating score data.

The implementation must keep tempo resolution cheap enough for the live playback path: avoid blocking the audio pipeline, avoid unnecessary allocations or repeated full-score scans in the transport loop, and keep timing updates deterministic so tempo-following does not introduce audible jitter.

### Scope

**In Scope:**

- Playback following immediate absolute tempo entries already stored in score data
- Resolving initial playback tempo from tick 0 or earlier applicable tempo state at the current playback tick
- Applying exact step tempo changes when playback crosses an immediate absolute tempo tick
- Accurate tempo resolution when playback begins from the middle of a flow
- Synthesized default fallback tempo behavior when no score tempo entry applies
- Truthful transport tempo reporting during stopped seek, stop, play, pause/resume, and active playback
- Backend architecture choices that leave room for future relative immediate tempos and gradual tempos
- Automated coverage for tempo resolution, transport timing behavior, and defensive fallback cases
- Documentation updates describing playback now following score tempo markings

**Out of Scope:**

- Relative immediate tempos such as `più mosso`, `meno mosso`, `a tempo`, or `Tempo I`
- Gradual tempo changes such as `rit.`, `accel.`, or interpolated transitions
- Live playback reacting immediately to score edits while already playing
- Broad transport UX redesign beyond any minimal consistency cleanup required by this slice
- A full reactive playback architecture refactor

## Context for Development

### Codebase Patterns

- Score tempo entries already live on the flow master track and should remain the single source of truth for score-driven playback tempo.
- Playback currently snapshots score and sampler data at play start, then runs a transport thread that advances one tick at a time.
- The transport currently derives tick duration from one fixed BPM captured at playback start, so tempo-following work must move timing resolution into the tick loop without overcomplicating the current proof-of-concept transport architecture.
- Existing tempo entry helpers should be reused or lightly extended rather than duplicating tempo lookup logic in playback.
- This slice should stay narrowly focused on immediate absolute tempos while keeping extension points clear for future relative and gradual tempo families.
- `TransportShared` already exposes the live transport tempo through atomics, so score-driven playback should update that shared value instead of inventing a second reporting path.
- The backend currently keeps a simple default tempo constant in the transport layer, and the frontend mirrors that value in `initialTransportState`, so fallback semantics need a single clear source of truth.
- The current `Track::get_tempo_before_tick()` helper linearly scans backward by tick; it is functionally correct for ad hoc lookup, but it must not become the steady-state playback-loop algorithm on long flows.
- Existing transport commands are already registered and consumed through stable Tauri command names, so any removal of obsolete manual tempo override APIs must update the command surface cleanly.
- Transport state should remain truthful even while stopped, so seek/stop transitions need to publish the effective tempo for the current tick rather than leaving stale tempo state in place.
- This slice should centralize effective playback tempo resolution behind one helper/resolver seam instead of scattering absolute-tempo logic across the transport loop.
- Playback timing decisions ultimately affect when work is handed to the live audio path, so transport-side tempo following must minimize lock contention, repeated score traversal, and per-tick overhead to avoid audible timing instability.
- `src/services/state/provider.tsx` currently patches transport actions to refresh polled state after `play`, `stop`, `seek`, and `setTempo()`, so removal of manual tempo override must update that provider wiring as well.
- Fixture docs currently state that tempo markings are visual-only; that guidance becomes stale once playback begins consuming score tempo data.

### Files to Reference

| File                                                                          | Purpose                                                                                               |
| ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `src-tauri/src/playback/transport.rs`                                         | Current playback thread, transport state, tick timing, play/pause/seek/set-tempo commands             |
| `src-tauri/src/score/entries/tempo.rs`                                        | Immediate tempo entry model and existing lookup helpers that playback should consume                  |
| `src-tauri/src/lib.rs`                                                        | Current Tauri transport command registration surface                                                  |
| `src/services/actions.ts`                                                     | Frontend transport action wrappers, including the current manual tempo command surface                |
| `src/services/actions.test.ts`                                                | Existing invoke-wrapper tests for transport and tempo command payloads                                |
| `src/services/state/context.tsx`                                              | Frontend initial transport state that currently mirrors the backend default tempo                     |
| `src/services/state/provider.tsx`                                             | Provider wrapper that currently refreshes transport state after tempo/seek/stop mutations             |
| `_bmad-output/implementation-artifacts/tech-spec-immediate-tempo-markings.md` | Prior completed spec for current absolute tempo entry/rendering behavior and future-tempo constraints |
| `docs/project-context.md`                                                     | Project rules for score/render/playback architecture and agent implementation constraints             |
| `docs/architecture.md`                                                        | Architecture reference that must be updated for the new playback behavior                             |
| `tests/fixtures/README.md`                                                    | Fixture guidance that currently says tempo markings do not affect playback                            |

### Technical Decisions

- Playback should resolve an effective tempo-shaped value for every relevant transport tick. When no applicable score tempo exists, the resolver should synthesize a non-persisted fallback tempo using the same runtime shape/path as a real tempo entry rather than inserting anything into score data.
- The synthesized fallback tempo behaves as the effective tempo at tick 0 and remains inherited until the first real score tempo entry takes effect.
- Playback starting from any tick must resolve the effective tempo from the latest tempo entry at-or-before that tick.
- Immediate absolute tempo changes apply as exact step changes on the matching tick boundary; no ramping or interpolation is introduced in this slice.
- Live score edits during active playback remain out of scope because the current transport architecture snapshots score state at play start.
- The implementation should avoid locking the codebase into an absolute-only tempo model; tempo resolution should be structured so future relative and gradual tempo behaviors can slot in without replacing the whole transport path.
- The best current insertion point is the transport tick loop in `transport_thread()`, because that is where both note duration-in-seconds and wall-clock tick pacing are currently derived from one fixed BPM.
- Tempo should be resolved once per playback tick before note collection/render work begins; per-note code should consume that already-resolved tempo rather than querying tempo repeatedly.
- Playback start should resolve and publish the active tempo for the current tick before the thread begins so `transport_get_state()` and the frontend show the correct BPM immediately even when starting from the middle of a flow.
- Initial transport state after app/provider startup should reflect the effective tick-0 tempo semantics, so idle UI state is truthful before any user interaction.
- Stopped seek should immediately resolve and publish the effective tempo for the destination tick so the UI remains truthful before `play` is pressed.
- `stop()` should reset `current_tick` to 0 and immediately publish the effective tempo at tick 0, using a real tick-0 score tempo when present or the synthesized fallback tempo otherwise.
- Pause/resume should behave like a fresh play-start from the current tick, re-resolving tempo from the latest score snapshot so playback stays accurate if score edits happened while paused.
- Missing user-authored tempo is a normal case and should silently use the synthesized fallback tempo.
- Malformed tempo data or broken tempo invariants are not normal missing-data cases; in this development-stage app they should fail hard and explicitly rather than being silently treated as absent tempo.
- Variable-tempo scheduling should advance transport deadlines from the prior deadline using per-tick durations, rather than re-basing each tick purely on `now`, to reduce drift and jitter.
- The current reverse-tick scan helper must not become the steady-state playback-loop algorithm on long flows; the resolver layer should use a bounded lookup strategy suitable for repeated transport-tick evaluation.
- Performance is a hard requirement: the tempo-following path must avoid avoidable blocking, avoid transport-thread work that could starve or destabilize the audio pipeline, and prefer precomputed or incrementally maintained tempo state over repeated expensive lookup inside the hot loop.
- The hot loop must not acquire `AppContext`/score mutexes or perform full score/master-track rescans once playback has started; all tempo data needed for repeated playback evaluation should come from playback-start snapshots and bounded runtime state.
- Manual transport tempo override commands are superseded for score playback in this slice. If no current non-score consumer needs them, remove the obsolete public transport tempo override surface rather than leaving ambiguous precedence in place.
- No live-edit synchronization should be added in this slice; playback continues to honor the play-start snapshot limitation explicitly called out by the user.

## Implementation Plan

### Tasks

- [x] Task 1: Add a dedicated effective-tempo resolver for playback
  - File: `src-tauri/src/score/entries/tempo.rs`
  - Action: Introduce a shared effective-tempo resolver that returns the applicable immediate absolute tempo at a playback tick or a synthesized fallback tempo-shaped value when no score tempo applies.
  - Notes: Keep the resolver seam explicit for future tempo-family expansion so later relative and gradual tempo logic can plug into the same path. The spec intentionally does not mandate one implementation strategy, but the chosen approach must provide bounded hot-loop lookup cost, resolve tempo once per tick, and avoid unbounded reverse tick-by-tick scans during active playback.

- [x] Task 2: Refactor transport timing to resolve tempo per playback tick
  - File: `src-tauri/src/playback/transport.rs`
  - Action: Replace the fixed-run BPM assumption with per-tick effective-tempo resolution sourced from the master-track tempo resolver, including synthesized fallback behavior when no score tempo exists.
  - Notes: The transport loop should compute wall-clock tick duration from the resolved BPM for the current tick, apply immediate changes exactly at the tempo tick boundary, and use the same resolved BPM when converting note durations from ticks to seconds. Update deadline scheduling so per-tick durations accumulate from the prior deadline instead of rebasing on `now` every iteration. The hot path must stay lean enough that tempo-following does not introduce audible jitter or destabilize downstream audio work.

- [x] Task 3: Publish correct tempo state on play, seek, stop, pause/resume, and transport polling
  - File: `src-tauri/src/playback/transport.rs`
  - Action: Ensure playback start resolves the active tempo for the current tick before spawning the playback thread, updates `TransportShared` with that BPM immediately, and keeps `transport_get_state()` aligned with score-following transport tempo during playback and stopped-state navigation.
  - Notes: Starting playback from the middle must show the inherited BPM immediately. Seeking while stopped should immediately publish the destination tick tempo. `stop()` should publish tick-0 tempo semantics immediately after resetting tick. Resume after pause should re-resolve from the current tick using a fresh playback start path, and the old transport thread must be fully stopped/retired before the resumed run starts.

- [x] Task 4: Remove obsolete manual transport tempo override surface and align frontend defaults
  - File: `src/services/actions.ts`
  - Action: Audit the frontend/backend transport tempo override surface, confirm whether any real non-score consumer still exists, and remove the public manual transport tempo override wrapper if none is found.
  - Notes: This slice should not leave an ambiguous `setTempo()` API behind if score tempo is now the real playback source. The implementation must explicitly verify existing call sites before removal rather than assuming the API is unused.
  - File: `src/services/actions.test.ts`
  - Action: Remove or update transport wrapper tests to match the final public transport action surface after manual override cleanup.
  - Notes: Keep the frontend transport contract explicit and minimal.
  - File: `src-tauri/src/lib.rs`
  - Action: Remove obsolete Tauri command registration if the manual transport tempo override command is eliminated.
  - Notes: Keep the public command surface aligned with actual supported transport behavior.
  - File: `src/services/state/context.tsx`
  - Action: Keep the frontend initial transport tempo aligned with the backend-defined fallback tempo semantics while relying on transport polling to publish truthful seek/stop state.
  - Notes: This avoids UI drift before the first transport poll resolves real state.
  - File: `src/services/state/provider.tsx`
  - Action: Remove or adjust provider-level `setTempo()` patching and related transport refresh logic so the provider matches the final transport action surface.
  - Notes: If `setTempo()` is removed, cleanup here is required to avoid stale wrappers and tests.

- [x] Task 5: Add focused automated coverage for score-driven transport tempo behavior
  - File: `src-tauri/src/playback/transport.rs`
  - Action: Add inline Rust tests for synthesized fallback tempo, tick-0 tempo resolution, inherited tempo resolution at non-zero start ticks, exact step changes when crossing an immediate absolute tempo entry, truthful stopped-seek tempo publication, stop reset to tick-0 tempo, pause/resume re-resolution, and explicit malformed-data failure behavior.
  - Notes: Prefer small deterministic helpers that can be tested without spinning a real audio thread. Add targeted tests for variable-BPM deadline accumulation behavior where practical, and structure the implementation so the hottest tempo-resolution pieces can be profiled or micro-benchmarked if needed.
  - File: `src-tauri/src/score/entries/tempo.rs`
  - Action: Add or refine helper tests that lock down the lookup semantics playback depends on.
  - Notes: Cover at-tick, at-or-before, synthesized fallback, and explicit malformed-data failure behavior explicitly.
  - File: `src/services/actions.test.ts`
  - Action: Update transport wrapper tests only to reflect the final transport action surface after obsolete manual override cleanup.
  - Notes: Keep frontend transport coverage intentionally thin and contract-focused.

- [x] Task 6: Update architecture documentation for score-driven playback tempo
  - File: `docs/architecture.md`
  - Action: Update the architecture notes to reflect that immediate absolute score tempos now drive transport tempo during playback, including truthful stopped-state tempo reporting and the current play-start snapshot limitation.
  - Notes: Replace the existing deferred-playback statement for tempo markings with the new behavior, document stop/seek/pause-resume semantics, and record that this slice does not change persisted score schema and therefore requires no migration.
  - File: `tests/fixtures/README.md`
  - Action: Update fixture guidance that currently says tempo markings are visual-only so it no longer contradicts score-driven playback tempo behavior.
  - Notes: Clarify that tempo fixtures remain useful for notation coverage, but tempo data is no longer playback-agnostic at the architectural level.

### Acceptance Criteria

- [x] AC 1: Given a flow with no tempo entries, when playback starts at tick 0, then transport playback uses a synthesized fallback tempo and reports that BPM through transport state.
- [x] AC 1a: Given the app or provider initializes with tick 0 selected and no user interaction yet, when transport state is first observed, then the reported idle tempo reflects the effective tick-0 tempo semantics rather than an unrelated stale value.
- [x] AC 2: Given a flow with an immediate absolute tempo at tick 0, when playback starts, then transport uses that tempo instead of the synthesized fallback from the first playback tick.
- [x] AC 3: Given a flow with an immediate absolute tempo earlier than the current playback tick, when playback starts from the middle of the flow, then transport resolves and reports the most recent tempo at-or-before that tick before any notes are played.
- [x] AC 4: Given a flow with multiple immediate absolute tempo entries, when playback crosses a tempo-entry tick, then the transport changes to the new BPM exactly on that tick boundary with no interpolation.
- [x] AC 5: Given a note that starts before a later tempo change, when the note is rendered at its start tick, then its audio duration in seconds is derived from the BPM active at that start tick rather than from a stale global tempo.
- [x] AC 6: Given playback is stopped and the user seeks to a later tick, when the seek completes, then `transport_get_state()` immediately reports the effective tempo for the destination tick before playback starts.
- [x] AC 7: Given score tempo entries exist, when playback is running, then `transport_get_state()` reports the score-resolved current BPM rather than a stale or manually overridden transport BPM.
- [x] AC 8: Given the user edits tempo markings during active playback, when playback is already running, then the current run continues using its play-start score snapshot and the documentation states this limitation explicitly.
- [x] AC 9: Given playback is stopped and the user triggers `stop()`, when the transport resets to tick 0, then reported transport tempo immediately reflects the effective tempo at tick 0 rather than the last played tempo.
- [x] AC 10: Given playback was paused at a non-zero tick, when playback resumes, then tempo is re-resolved from the current tick so intervening score edits made while paused are reflected in the resumed run.
- [x] AC 11: Given no user-authored tempo exists yet for the active playback range, when transport resolves playback tempo, then playback silently uses the synthesized fallback tempo.
- [x] AC 11a: Given tempo data violates expected invariants or is otherwise malformed, when transport attempts to resolve playback tempo, then the failure is explicit and hard rather than silently treated as missing tempo.
- [x] AC 12: Given the transport evaluates tempo repeatedly across many ticks, when playback runs on long flows, then the implementation does not rely on an unbounded reverse tick-by-tick scan as the steady-state lookup path.
- [x] AC 13: Given manual transport tempo override is superseded by score-driven playback tempo, when this slice is completed, then no ambiguous public transport tempo override path remains unless a confirmed non-score use case is explicitly documented.
- [x] AC 14: Given future tempo families are still planned, when this slice is implemented, then the code introduces or preserves a dedicated effective-tempo resolver seam that does not hard-code playback logic irreversibly to immediate absolute tempos only.
- [x] AC 15: Given the transport uses variable BPM over time, when tempo changes occur during playback, then wall-clock scheduling advances with bounded drift by accumulating per-tick deadlines from prior timing state rather than rebasing every tick from the current instant.
- [x] AC 16: Given the feature changes playback behavior and architecture guidance, when the work is completed, then [docs/architecture.md](docs/architecture.md) is updated to describe score-driven playback tempo, stopped-state tempo reporting, pause/resume semantics, and the current snapshot limitation.
- [x] AC 17: Given this slice changes playback behavior but not persisted score schema, when the work is completed, then the spec and documentation explicitly state that no migration is required.
- [x] AC 18: Given tempo-following runs on the live playback path, when the implementation is complete, then tempo resolution avoids unbounded per-tick score scans, avoidable blocking, and other hot-path work likely to cause audible jitter.
- [x] AC 19: Given playback has started from a snapshot, when the transport hot loop advances ticks, then tempo is resolved once per tick before note collection and no new `AppContext`/score mutex access is introduced into that hot loop.

### Documentation Impact

- Update [docs/architecture.md](docs/architecture.md) to replace the current statement that tempo markings do not affect transport tempo and document the new immediate-absolute playback behavior.
- Document that playback tempo is now score-driven during playback, that stopped seek/stop state also publishes truthful effective tempo for the current tick, and that pause/resume re-resolves from the current tick.
- Document that playback still uses a play-start score snapshot while running and therefore does not react live to edits until a fresh play/resume cycle begins.
- Explicitly note that this slice does not change persisted tempo schema or `.soloc` format behavior, so no migration update is required.
- Update `tests/fixtures/README.md` so fixture documentation no longer claims tempo markings are playback-neutral/visual-only.
- No update to AI-agent instructions is required for this slice because no standing implementation rule or workflow convention changes; the work changes product behavior and architecture notes, not project-wide agent operating rules.

## Additional Context

### Dependencies

- Depends on the already-implemented immediate absolute tempo entry/data/rendering slice documented in `_bmad-output/implementation-artifacts/tech-spec-immediate-tempo-markings.md`.
- Depends on existing flow master-track tempo storage and helper lookups in `src-tauri/src/score/entries/tempo.rs`.
- Depends on the current transport polling model (`transport_get_state()`) and playback-thread architecture in `src-tauri/src/playback/transport.rs`.
- Depends on introducing a reusable effective-tempo resolver seam that can later absorb relative and gradual tempo behavior without changing the transport contract again.
- No new external libraries or services are required.
- No persistence or migration dependency is introduced because this slice consumes existing tempo data only.

### Testing Strategy

- Add Rust unit tests around effective-tempo resolver helpers so the active BPM at a tick is deterministic and independently verifiable, including synthesized fallback and explicit malformed-data failure cases.
- Add Rust transport-focused tests for: fallback default with no tempos, tick-0 score tempo override, inherited at-or-before resolution for non-zero playback starts, immediate stopped-seek tempo publication, stop reset to tick-0 tempo, pause/resume re-resolution, and exact step tempo changes at tempo-entry ticks.
- Add targeted transport timing tests for variable-BPM deadline accumulation behavior where practical, or isolate the scheduling helper enough to verify drift-minimizing logic deterministically.
- Add targeted performance-oriented validation for the transport hot path, such as isolated tests around resolver complexity and scheduling helpers, so expensive per-tick behavior is caught before it risks audible jitter.
- Update `src/services/actions.test.ts` only to reflect the final transport action surface after obsolete manual override cleanup.
- Update provider-level transport tests/mocks if `setTempo()` removal changes the provider wrapper behavior.
- Manual verification: create a short score with no tempo entry, a tick-0 tempo, and multiple later tempos; seek while stopped, stop after playback, pause/edit/resume, and start from mid-flow positions to confirm audible tempo and reported transport BPM match score expectations.

### Notes

- Keep this slice small and implementation-ready as part of the wider tempo-system epic.
- Prefer a design that is simple now but does not make relative or gradual tempo support a rewrite later.
- The highest-risk area is coupling wall-clock pacing and note-duration conversion to two different BPM sources; the implementation should centralize tempo resolution so both use the same effective tempo for a given tick.
- The most important non-functional constraint is transport-path performance: any approach that adds enough overhead to create playback jitter is unacceptable, even if functionally correct.
- Another risk is leaving obsolete manual tempo override APIs behind after score tempo becomes authoritative; this slice should remove or explicitly retire those paths so transport behavior stays unambiguous.
- The synthetic fallback tempo should be treated as a transport-only implementation detail: it must never be persisted back into score state or emitted as a real score mutation.
- Missing tempo is normal and should be satisfied by synthesized fallback; malformed tempo data is a development-stage invariant failure and should fail hard with explicit diagnostics.
- If implementation discovers a meaningful non-score consumer of the old manual transport tempo override surface, that exception must be documented explicitly before retaining the API.
- Future follow-up slices should cover immediate relative tempos, gradual tempos, and live playback reaction to score edits instead of stretching this implementation beyond immediate absolute tempo support.

## Review Notes

- Adversarial review completed
- Findings: 10 total, 8 fixed, 2 skipped
- Resolution approach: auto-fix
