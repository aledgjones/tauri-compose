# Story 7.3: Validate Transport Trust Against Representative Composition Scenarios

Status: done

## Story

As a maintainer,
I want transport trust validated through representative score scenarios,
so that release decisions are based on composition-grade behavior rather than isolated happy paths.

## Acceptance Criteria

1. Given the supported playback path now includes real transport behavior, when representative play, pause, resume, seek, and edit-during-playback scenarios are exercised, then the resulting validation shows whether playback remains coherent, dropout-free, and trustworthy, and the checks are strong enough to catch regressions that matter to composition work.
2. Given transport trust is release-gating, when the validation set is run, then the results distinguish between demo-capable behavior and composition-trustworthy behavior, and the product is not considered release-credible if the representative scenarios fail.
3. Given transport validation must be explicit rather than anecdotal, when the scenarios are documented and executed, then the scenario set is repeatable and specific, and it can be reused as an ongoing regression baseline.

## Tasks / Subtasks

- [x] Task 1: Execute the canonical Story 7.3 transport-trust matrix through real owner seams instead of a new proof harness (AC: 1, 3)
  - [x] Reuse `docs/playback-support-baseline.md` as the single scenario matrix for Story 7.3; do not create a second transport checklist in `TESTING.md`, the UI, or a one-off validation note.
  - [x] Build representative score scenarios with the existing transport test helpers in `src-tauri/src/playback/transport.rs`, especially the helpers for multi-track, multi-tempo, and expression-map-aware setup, so the validation looks like real composition work rather than isolated single-note proofs.
  - [x] Capture evidence from backend-owned seams only: `supported_targets` at play-start, transport tick and tempo truth, engine transport debug state, and any bounded manual product evidence needed by the baseline.
  - [x] Keep the scenario set compact but representative: routed play start, repeated pause/resume, stopped and active seek, bounded edit-during-playback, and expression-map-aware routed playback on the current MVP path.

- [x] Task 2: Prove backend-owned snapshot trust across play, pause, resume, and seek boundaries (AC: 1, 2, 3)
  - [x] Extend seam-local regressions in `src-tauri/src/playback/transport.rs`, `src-tauri/src/playback/engine.rs`, and `src-tauri/src/playback/audio.rs` to validate repeated play/pause/resume cycles, active-seek rebases, paused and stopped seek truth, and audio-rebuild reset boundaries under representative transport work.
  - [x] Preserve the existing play-start snapshot contract from `prepare_supported_playback_with_expression_maps(...)`: readiness is evaluated at snapshot boundaries, not polled per tick and not rebuilt from frontend state.
  - [x] Reconfirm the `last_queued_tick` pause and rebuild semantics so reset and resume decisions stay aligned with pending engine-owned transport work instead of only the advanced cursor.
  - [x] Keep unresolved or unsupported routes as explicit backend no-ops during representative scenarios rather than blockers that hide whether the supported path itself remains trustworthy.

- [x] Task 3: Validate composition-grade behavior for bounded edits and expression-map-aware playback without widening scope (AC: 1, 2, 3)
  - [x] Reuse Story 5.3's bounded live-edit contract exactly: `volume`, `pan`, `mute`, and `solo` on already-ready supported routes remain immediate; score-structure and playback-topology edits remain deferred to the next transport-owned reset boundary.
  - [x] Reuse Story 6.4's transport-owned expression-map path exactly: play-start snapshots resolve expression-map actions through the existing preparation and transport seams rather than a second playback model.
  - [x] Include at least one representative composition-shaped scenario with non-trivial musical content, such as multiple instruments, tempo changes, articulations, or single-level tuplets already supported by the backend-owned playback model, while keeping validation inside current MVP scope.
  - [x] Validate that pause, seek, natural-end replay, and other reset boundaries reconcile deferred playback truth cleanly without mid-pass queue surgery or stale-note leaks.

- [x] Task 4: Publish release-review evidence and verdict boundaries without overstating support (AC: 2, 3)
  - [x] Update `docs/playback-support-baseline.md` with Story 7.3 results for each executed transport-trust scenario, including pass, fail, bounded risk, or unresolved status.
  - [x] Distinguish clearly between demo-capable and composition-trustworthy behavior in the baseline artifact; failing representative scenarios must remain release blockers rather than being rounded up to "good enough."
  - [x] Keep bundle-level verdicts honest: Story 7.2a repaired bounded host structure, but `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3` must not be promoted by Story 7.3 unless current validation evidence supports the full claim.
  - [x] Update `TESTING.md`, `docs/development-guide.md`, and any directly linked maintainer guidance only where the transport-validation workflow or release guidance materially changes.

## Dev Notes

### Story Intent

- Story 7.3 is the transport-evidence pass for Epic 7.
- Story 7.1 defined the baseline, Story 7.2 validated discovery and bounded host behavior, and Story 7.2a repaired concrete host-lifecycle defects. Story 7.3 now determines whether the actual routed transport path is trustworthy enough for composition work.
- The goal is not to make transport richer. The goal is to measure whether the existing backend-owned transport contract remains coherent under representative composition scenarios that matter for release confidence.

### Previous Story Intelligence

- Story 7.1 established `docs/playback-support-baseline.md` as the canonical Epic 7 support artifact. Reuse it; do not fork a second transport matrix.
- Story 7.2 proved discovery and blocked-host posture through the real seams, but manual validation still left `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3` short of a clean `known-good` verdict.
- Story 7.2a fixed bounded host structural faults in `src-tauri/src/playback/plugin_editor_poc.rs`, but its completion notes still require evidence before those bundles are promoted. Story 7.3 must not quietly assume host trust is already settled.
- Story 5.1 established pause, stop, resume, live mix truth, and audio-rebuild reset behavior on the real product path.
- Story 5.2a hardened queue-window scheduling, explicit runtime reset semantics, and the canonical `supported_targets` snapshot contract.
- Story 5.3 established the bounded live-edit contract: only `volume`, `pan`, `mute`, and `solo` are immediate during playback; score-structure and playback-topology edits defer to transport-owned reset boundaries.
- Story 6.4 kept expression playback transport-owned through `prepare_supported_playback_with_expression_maps(...)` and per-note action resolution in `transport.rs`; Story 7.3 should validate that path, not replace it.

### This Story Owns

- The repeatable transport-trust validation set for representative composition scenarios.
- Seam-local regression coverage and maintainer evidence for play, pause, resume, seek, bounded edit-during-playback, and expression-map-aware transport.
- The release-facing distinction between demo-capable playback and composition-trustworthy playback.
- Documentation updates that record actual transport-trust evidence in the canonical Epic 7 baseline.

### This Story Does Not Own

- Discovery or bounded native editor-host validation. Those were handled by Stories 7.2 and 7.2a.
- General transport redesign, new playback UI, or frontend-owned validation state.
- Broader plugin-family support claims, universal VST3 compatibility claims, or marketing-style support language.
- New live-edit categories beyond the bounded Story 5.3 contract.

### Current Relevant Seams

- `docs/playback-support-baseline.md`
  - Canonical Story 7.3 transport-trust scenarios and support vocabulary.
- `TESTING.md`
  - Command guide and the rule that the baseline doc owns the Epic 7 scenario matrix.
- `docs/development-guide.md`
  - Maintainer-facing playback guidance that may need transport-verdict updates.
- `src-tauri/src/playback/transport.rs`
  - Owner seam for play, pause, resume, seek, queue-window scheduling, transport thread lifecycle, play-start snapshot creation, and the largest body of existing transport trust tests.
- `src-tauri/src/playback/engine.rs`
  - Owner seam for queued transport work, active transport-note cleanup, `clear_transport_playback()`, `transport_playback_has_pending_work()`, and debug inspection of engine-owned transport state.
- `src-tauri/src/playback/audio.rs`
  - Owner seam for audio rebuild resets that must preserve transport truth and queue-state rebasing.
- `src-tauri/src/playback/preparation.rs`
  - Owner seam for `supported_targets`, route-backed mix truth, and expression-map-aware play-start preparation.
- `src-tauri/src/playback/expression_maps.rs`
  - Owner seam for durable rule resolution and current expression-map action semantics.
- `src-tauri/src/playback/tuplet_projection.rs`
  - Owner seam for carrying articulations and written durations into transport playback events when representative scenarios include tuplets.

### Repo Memory Guardrails

- Pause and rebuild reset must follow `last_queued_tick` whenever engine-owned transport work is still pending; `current_tick` alone is not trustworthy enough for resume boundaries.
- Pause cleanup must fully drain transport-owned note-off work before adjacent editor-open or manual validation steps, or stale playback can survive pause and contaminate transport-trust evidence.
- Expression-map transport proof remains snapshot-based: rules resolve at play-start, actions are applied per scheduled note, and CC actions remain structurally supported but intentionally no-op for the bounded current proof path.
- Transport tests that inspect derived values from `FxHashMap` must sort those vectors before asserting order; hash iteration order is not a transport-trust signal.

### Implementation Guidance

- Start from the existing transport test infrastructure in `src-tauri/src/playback/transport.rs`; it already includes helpers for multi-track, multi-tempo, and expression-map-aware scenarios. Story 7.3 should scale those helpers into representative scenarios instead of inventing a new harness.
- Keep validation backend-truth-driven. The strongest automated evidence should come from transport and engine state, not from UI timing or anecdotal manual listening alone.
- Treat transport trust as a release gate, not a polish check. If representative scenarios still expose stale queued work, resume drift, seek corruption, or inconsistent deferred-edit boundaries, the story should record that honestly rather than hiding it inside a softer verdict.
- Preserve the current snapshot model exactly. Do not add mid-pass readiness probes, mid-pass expression-map re-resolution, or queue rewrites to make scenarios pass.
- If representative scenarios use tuplets, articulations, or tempo changes, reuse the existing shared playback model and projection seams rather than transport-local reinterpretation.
- Keep bundle claims narrow. If transport trust is only proven on a subset of concrete bundles or scenarios, record that exact subset and leave the rest as unresolved or future expansion.

### Risks And Regression Traps

- Mistaking a clean demo path for composition-trustworthy playback.
- Treating host stabilization from Story 7.2a as automatic proof that the same bundles are now release-credible for transport trust.
- Reintroducing frontend-owned readiness, timing, or support state to paper over backend transport defects.
- Allowing score or topology edits to rewrite engine-owned transport work mid-pass instead of following the existing deferred contract.
- Missing pause-related stale-note cleanup, especially in scenarios that pause and then immediately reopen or interact with plugin windows.
- Using unsorted `FxHashMap`-derived vectors in transport assertions and turning map iteration order into a flaky regression signal.

### Testing Requirements

- Prefer seam-local Rust coverage first:
  - `cd src-tauri && cargo test transport_ --lib -- --nocapture`
  - `cd src-tauri && cargo test prepare_supported_playback --lib -- --nocapture`
  - `cd src-tauri && cargo test expression_maps --lib -- --nocapture`
  - `cd src-tauri && cargo test playback::engine --lib -- --nocapture`
- If Story 7.3 changes code, also run:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
- Manual validation should follow the canonical Story 7.3 matrix in `docs/playback-support-baseline.md` and capture explicit evidence for:
  - routed play start on supported targets;
  - repeated pause and resume cycles;
  - stopped and active seek behavior;
  - bounded edit-during-playback refresh boundaries;
  - expression-map-aware routed playback on the current MVP path.
- If representative manual validation still includes `BBC Symphony Orchestra.vst3` or `Originals - Intimate Grand Piano.vst3`, record bundle-specific verdicts explicitly and do not treat them as `known-good` without the supporting evidence.

### Documentation Impact

- Update `docs/playback-support-baseline.md` with Story 7.3 scenario results and verdicts.
- Update `TESTING.md` only if the recommended transport-validation workflow or command guidance changes materially.
- Update `docs/development-guide.md` if the maintainer-facing transport runbook or support posture changes materially.
- Update AI instructions only if Story 7.3 establishes a durable repo-wide validation rule beyond this Epic 7 evidence pass.

### Architecture Compliance

- Backend ownership remains intact: Rust owns route truth, preparation truth, transport truth, engine truth, and validation evidence.
- The transport and expression-map paths remain snapshot-based and backend-owned; React remains a thin trigger and display surface.
- The current MVP claim remains macOS-only, VST3-only, rack-first, and bounded to the explicit support baseline.
- Legacy sampler, SCVI, or frontend-owned playback fallbacks must not reappear to satisfy transport-trust acceptance criteria.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- Do not add dependencies for Story 7.3 unless a narrowly justified seam-local test or evidence-format need appears.

### File Structure Notes

- Keep transport and engine validation work under `src-tauri/src/playback/`.
- Keep score-scenario or projection work near the existing playback or score seams that already own it.
- Keep support-verdict evidence in the canonical maintainer docs under `docs/`; do not add a second validation app or a frontend-owned status matrix.

### Git Intelligence Summary

- Recent Epic 7 commits followed the right pattern: baseline definition first (`feat(7.1): playback support docs.`), then bounded validation (`feat(7.2): validation`), then seam-local host hardening (`feat(7.2a): natuve plugin ui fixes`). Story 7.3 should continue that sequence by producing transport evidence, not by redefining scope.
- The relevant transport hardening already landed in Epic 5. Story 7.3 should lean on those seams and tests rather than treating transport trust as greenfield work.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 7 and Story 7.3 acceptance criteria.
- `_bmad-output/planning-artifacts/prd.md`
  - FR37 through FR41 plus the release-critical performance and reliability requirements for transport trust.
- `_bmad-output/planning-artifacts/architecture.md`
  - Backend-owned playback truth, transport trust, and realtime safety constraints.
- `_bmad-output/implementation-artifacts/5-1-start-pause-and-resume-trusted-score-playback.md`
  - Pause/resume, live mix truth, and audio-rebuild reset guardrails.
- `_bmad-output/implementation-artifacts/5-2a-harden-playback-clocking-and-runtime-reset-semantics.md`
  - Queue-window clocking, reset semantics, and `supported_targets` contract.
- `_bmad-output/implementation-artifacts/5-3-define-bounded-editing-during-playback.md`
  - Immediate-versus-deferred live-edit contract.
- `_bmad-output/implementation-artifacts/7-1-define-the-supported-plugin-and-scenario-baseline.md`
  - Canonical support-vocabulary and scenario-baseline origin.
- `_bmad-output/implementation-artifacts/7-2-validate-plugin-discovery-and-host-behavior-against-the-baseline.md`
  - Discovery and bounded-host evidence posture.
- `_bmad-output/implementation-artifacts/7-2a-stabilize-bounded-native-editor-host-for-real-library-control-bundles.md`
  - Current host-structure repair status and bundle-verdict caution.
- `docs/playback-support-baseline.md`
  - Canonical Story 7.3 transport-trust scenario matrix.
- `TESTING.md`
  - Repo testing commands and the rule that Epic 7 scenario matrices live in the baseline doc.
- `src-tauri/src/playback/transport.rs`
  - Transport owner seam and representative scenario test helpers.
- `src-tauri/src/playback/engine.rs`
  - Engine transport cleanup and debug inspection seam.
- `src-tauri/src/playback/audio.rs`
  - Audio rebuild reset seam.
- `src-tauri/src/playback/preparation.rs`
  - `supported_targets` and expression-map-aware preparation seam.
- `src-tauri/src/playback/expression_maps.rs`
  - Durable expression-map rule evaluation seam.
- `/memories/repo/expression-map-transport-proof-story-6-4.md`
  - Current expression-map transport proof boundary.
- `/memories/repo/transport-pause-last-queued-tick.md`
  - Pause and rebuild rewind semantics.
- `/memories/repo/transport-pause-editor-open-cleanup.md`
  - Pause cleanup trap adjacent to manual validation.
- `/memories/repo/transport-test-ordering.md`
  - Transport assertion ordering guardrail.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-08: Story prepared from Epic 7 planning, PRD transport-trust requirements, architecture ownership rules, the canonical support baseline, prior Epic 5 transport stories, and the completed Epic 7 host-validation artifacts.
- 2026-04-08: Independent exploration confirmed that the strongest existing Story 7.3 implementation seam is the mature transport test surface in `src-tauri/src/playback/transport.rs`, plus engine debug inspection and audio-rebuild reset seams.
- 2026-04-08: Story context explicitly carries forward the unresolved bundle-verdict caution from Story 7.2a so Story 7.3 does not accidentally broaden support claims while validating transport trust.
- 2026-04-08: Added a representative multi-track, multi-tempo, expression-map-aware transport regression that proves snapshot truth, active seek rebasing, and unresolved-route no-op behavior through existing owner seams.
- 2026-04-08: Added complementary engine and audio regressions for pending transport work cleanup and rebuild reset stop-boundary truth, then stabilized an existing pause regression test with explicit shared-engine isolation.
- 2026-04-08: Updated the canonical playback support baseline with Story 7.3 results, the automated transport verdict, repo-wide validation evidence, and the explicit bundle-level boundary that keeps BBC/Originals unpromoted pending maintainer manual proof.

### Completion Notes List

- Added `make_flow_score_with_tracks_and_tempos(...)` plus `representative_transport_scenario_rebases_expression_map_playback_without_widening_route_truth()` in `src-tauri/src/playback/transport.rs` to validate supported-target snapshots, active seek rebasing, expression-map-aware routed playback, and unresolved-route no-op behavior through real transport seams.
- Added `pending_transport_work_tracks_multiple_slots_until_transport_cleanup_runs()` in `src-tauri/src/playback/engine.rs` and `reset_transport_for_audio_rebuild_keeps_the_rewound_stop_boundary_tick_over_stale_last_queued_tick()` in `src-tauri/src/playback/audio.rs` to tighten queue-cleanup and rebuild-boundary trust coverage.
- Stabilized `transport_pause_with_states_republishes_tempo_at_the_paused_tick_and_joins_the_thread()` with explicit shared-engine isolation so Story 7.3 transport-trust evidence remains deterministic under parallel test execution.
- Updated `docs/playback-support-baseline.md` with Story 7.3 scenario outcomes, the automated backend verdict, repo-wide validation commands, and the explicit boundary that keeps `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3` at `discovered but unvalidated` until maintainer manual proof is captured.
- Left `TESTING.md` and `docs/development-guide.md` unchanged because the canonical workflow and command guidance did not materially change beyond the baseline evidence update.
- Validation passed: `cargo test transport_ --lib -- --nocapture`, `cargo test prepare_supported_playback --lib -- --nocapture`, `cargo test expression_maps --lib -- --nocapture`, `cargo test playback::engine --lib -- --nocapture`, `npm run lint`, `npm run check-types`, `npm run test:js`, and `npm run test:rust`.
- 2026-04-08: Maintainer accepted the review pass as sufficient for final completion and requested the story be marked `done`.

### File List

- `_bmad-output/implementation-artifacts/7-3-validate-transport-trust-against-representative-composition-scenarios.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/playback-support-baseline.md`
- `src-tauri/src/playback/audio.rs`
- `src-tauri/src/playback/engine.rs`
- `src-tauri/src/playback/transport.rs`

### Change Log

- 2026-04-08: Created the Story 7.3 implementation artifact and advanced sprint tracking to `ready-for-dev`.
- 2026-04-08: Added representative transport-trust regressions across transport, engine, and audio seams; documented Story 7.3 results in the canonical playback support baseline; and advanced the story to `review`.
- 2026-04-08: Review accepted as final and story marked `done`.
