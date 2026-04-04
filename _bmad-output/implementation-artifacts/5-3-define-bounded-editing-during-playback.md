# Story 5.3: Define Bounded Editing During Playback

Status: done

## Story

As a composer,
I want clear rules for what score edits apply during playback and when,
so that editing while listening feels predictable rather than fragile.

## Acceptance Criteria

1. Given Story 5.0 established a supported audible transport path, when playback is active, then Compose applies only these edits immediately in the current audible pass: `volume`, `pan`, `mute`, and `solo` on already-supported ready routes.
2. Given transport playback is driven from a backend-owned play-start snapshot, when playback is active and I create, delete, move, slice, retime, or repitch tones, or change accidentals, articulations, tuplet membership, routes, slots, or plugin assignments, then Compose saves the score intent immediately in canonical score state but defers the audible update until the next safe playback boundary.
3. Given a deferred edit occurs during playback, when that happens, then Compose preserves playback coherence, keeps the current pass running from the existing snapshot, and does not attempt mid-pass transport queue rewrites.
4. Given the supported safe boundaries are pause, seek restart, stop-then-play, natural playback end, and explicit backend transport resets, when any of those boundaries occurs after deferred edits, then the next transport snapshot reflects the updated score truth.
5. Given edit-during-playback behavior is part of real composition work, when representative scenarios are exercised, then the immediate-only set, deferred set, and safe-boundary triggers remain consistent, regression-tested, and explicit enough that unsupported live-edit behavior is not ambiguous.

## Tasks / Subtasks

- [x] Task 1: Define the backend-owned live-edit contract for active transport playback (AC: 1, 2, 3, 4)
  - [x] Encode the immediate live-update set explicitly as `volume`, `pan`, `mute`, and `solo` on supported ready routes.
  - [x] Encode the deferred set explicitly for tone-content edits, notation-affecting edits, and playback-topology edits.
  - [x] Keep the current transport snapshot model authoritative: no score-structural command may silently patch queued transport events or mutate engine-owned transport work mid-pass.
  - [x] Preserve immediate persistence of user intent in canonical score state even when the audible result must defer.

- [x] Task 2: Implement bounded behavior for score and playback-topology edits during playback (AC: 1, 2, 4)
  - [x] Keep the existing immediate live path for already-supported route-backed mix controls in `src-tauri/src/score/instruments/mod.rs`.
  - [x] Route score-structural edits such as tone create, remove, move, slice, pitch change, duration change, accidental change, articulation change, and tuplet-membership change through a shared backend policy that marks playback as needing a safe refresh instead of trying to rewrite the in-flight snapshot.
  - [x] Apply the same bounded policy to route assignment, route removal, slot reassignment, and plugin-topology mutations so playback coherence wins over partial mid-pass reconfiguration.
  - [x] Reuse the existing pause, stop, seek, natural-end, and next-play-start seams in `src-tauri/src/playback/transport.rs` as the supported safe boundaries where deferred playback truth becomes audible.

- [x] Task 3: Keep deferred-edit coordination backend-owned without creating parallel truth (AC: 3, 4)
  - [x] Keep deferred-edit state in Rust-owned models and existing general transport/runtime coordination paths rather than frontend-only inference.
  - [x] Avoid heavy-handed score-context messaging whose main purpose is to prove the contract rather than help composition flow.
  - [x] Avoid high-frequency event chatter or a second transport state cache; safe-boundary behavior remains backend-owned.

- [x] Task 4: Add seam-local regression coverage for bounded live-edit behavior (AC: 4)
  - [x] Extend `src-tauri/src/playback/transport.rs` tests to prove deferred edits do not corrupt the current transport pass and that pause, seek, stop, or restart rebuild from the updated score state.
  - [x] Extend `src-tauri/src/score/entries/tone.rs` and related backend seams only as needed to prove edit commands preserve score intent while transport remains coherent.
  - [x] Extend frontend tests only if visible score-context messaging or transport-trigger semantics change.

- [x] Task 5: Document the bounded editing contract for later Epic 5 and Epic 6 work (AC: 1, 2, 3, 4)
  - [x] Update developer-facing docs and agent instructions so later playback stories do not assume arbitrary mid-pass score rescheduling.
  - [x] Document the supported categories explicitly: what remains immediate today, what defers to the next safe boundary, and what is intentionally out of scope for MVP live editing.

### Review Findings

- [x] [Review][Patch] Pending topology refresh can be downgraded by a later score-structure edit [src-tauri/src/lib.rs:216]
- [x] [Review][Patch] Audio rebuild resets clear deferred topology work instead of reconciling it at the reset boundary [src-tauri/src/playback/audio.rs:212]
- [x] [Review][Patch] Regression coverage misses the mixed deferred-kind and audio-rebuild reset paths required by the bounded contract [src-tauri/src/playback/transport.rs:2574]

## Dev Notes

### Story Intent

- Story 5.3 defines the first explicit edit-during-playback contract on top of the now-trusted transport path.
- The objective is predictability, not generalized realtime score rescheduling. Users must be able to keep composing while playback runs, but the app must stay honest about when changes become audible.

### Previous Story Intelligence

- Story 5.1 established that supported live mix edits already have a safe in-flight path through backend-owned mix synchronization, so Story 5.3 should preserve that behavior rather than regressing it.
- Story 5.2 established seek as a fresh backend snapshot restart that clears old transport work before continuing from a new tick.
- Story 5.2a hardened the queue-window clocking model, reset semantics, and multi-target preparation contract, which means Story 5.3 should reuse transport-owned restart boundaries instead of inventing a mid-pass score-rescheduling seam.
- Recent commits `dcd661b1` and `2411a11a` concentrated the relevant work in `src-tauri/src/playback/transport.rs`, `src-tauri/src/playback/engine.rs`, `src-tauri/src/playback/audio.rs`, and transport-related docs; follow that same seam discipline.

### This Story Owns

- The explicit policy for immediate versus deferred edit behavior while transport is playing.
- Backend-owned tracking of deferred playback updates.
- Regression coverage that proves the bounded contract is stable and testable.

### This Story Does Not Own

- General realtime rescheduling of future score notes inside an already-running transport snapshot.
- A new frontend-owned transport model, pending-edit cache, or playback-topology mirror.
- Expression-map behavior, articulation realism, or broader playback authoring UX beyond bounded live-edit coordination.
- HALion-style multi-timbral, multi-slot, or arbitrary live-routing expansion.

### Explicit Live-Edit Contract

- Immediate in the current audible pass:
  - `volume`;
  - `pan`;
  - `mute`;
  - `solo`;
  - only when the edit targets an already-supported ready route that uses the existing live mix synchronization path.
- Deferred to the next safe playback boundary:
  - tone creation;
  - tone deletion;
  - tone slice;
  - tone move or shift;
  - tone pitch change;
  - tone duration change;
  - accidental change;
  - articulation change;
  - tuplet-membership or host-position change;
  - instrument route assignment;
  - instrument route removal;
  - slot reassignment;
  - plugin assignment or unassignment;
  - any other edit that changes score content, note timing, preparation truth, or playback topology.
- Safe boundaries where deferred audible changes become active:
  - `transport_pause`;
  - `transport_seek` restart;
  - `transport_play` after a stopped or paused state;
  - natural playback end followed by the next play-start;
  - explicit backend transport reset paths that already rebuild playback from current score truth.
- Not supported as immediate live updates in this story:
  - any mid-pass mutation of queued transport note events;
  - any mid-pass rebuild of prepared targets or playback topology;
  - any frontend-owned prediction of playback state that diverges from backend truth.

### Current Relevant Seams

- `src-tauri/src/playback/transport.rs`
  - Owns play, pause, seek, restart boundaries, shared transport atomics, and play-start snapshot creation.
- `src-tauri/src/playback/engine.rs`
  - Owns queued transport work and render-active slot state on the realtime path.
- `src-tauri/src/score/entries/tone.rs`
  - Owns score-content mutations that currently emit updated score state immediately without playback-policy checks.
- `src-tauri/src/score/instruments/mod.rs`
  - Already contains the supported live mix path and also owns route mutations that need explicit bounded behavior.
- `src-tauri/src/playback/audio.rs`
  - Owns stream rebuild resets; any deferred-edit state must stay coherent if transport is stopped through this seam.
- `src/services/actions.ts`
  - Remains the FE to BE mutation bridge; do not shift bounded-edit policy into the action layer.
- `src/services/types.ts`
  - Mirrors Rust-owned state; extend only if backend-owned deferred-edit state must be surfaced.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Existing score-edit trigger surface that already reads transport state and should stay thin.

### Implementation Guidance

- Preserve the current snapshot model: transport starts from a cloned score plus preparation snapshot, and score edits made after that point must not pretend to rewrite the running snapshot unless a new backend seam genuinely supports it.
- Prefer one shared backend helper or policy seam for edit-during-playback classification instead of scattering `if playing` checks across many commands.
- When an edit is deferred, update canonical score state immediately and keep the current pass coherent without trying to narrate the contract in the transport UI.
- Clear or resolve deferred-edit state through the same transport-owned restart boundaries that already rebuild playback truth. Do not create a second restart mechanism.
- Do not add user-facing messaging whose main job is to prove the bounded contract. The composition flow matters more than narrating backend state.
- Do not regress the existing live mix path. If an edit category is already proven immediate, preserve that path explicitly.
- Do not quietly broaden the immediate set during implementation. New immediate live edits require a new proven backend seam and dedicated regressions.

### Risks And Regression Traps

- Trying to patch `transport_playback` engine queues directly from score-edit commands risks stale-note leaks, queue corruption, and audio-thread unsafe behavior.
- Frontend-derived pending-edit badges or heuristics would create parallel playback truth and drift from backend reality.
- Auto-pausing on every edit would satisfy the acceptance criteria superficially but would degrade real composition flow unless deliberately chosen and documented; prefer a bounded deferred contract first.
- Route, slot, or plugin-topology edits during playback are especially dangerous because they can invalidate prepared targets; keep them deferred to the next safe boundary rather than partially reconfiguring the active pass.
- Audio rebuild resets and transport resets must clear or reconcile any deferred-edit state through the same backend-owned contract, or the UI will lie about whether playback is current.

### Testing Requirements

- Extend Rust tests near the owning seams in `src-tauri/src/playback/transport.rs`, `src-tauri/src/score/entries/tone.rs`, and `src-tauri/src/score/instruments/mod.rs` for:
  - deferred tone create, remove, move, pitch, duration, accidental, and articulation edits while transport is playing;
  - immediate live mix updates remaining immediate during playback;
  - route, slot, and plugin-topology changes during playback remaining deferred and not corrupting the current pass;
  - pause, seek, natural-end replay, and restart applying deferred score truth from a fresh snapshot.
- Use the existing transport test serialization pattern when tests inspect continuous-engine transport work.
- Extend frontend tests only if transport controls or shortcut behavior changes, likely in:
  - `src/services/actions.test.ts`
  - `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
  - any score-context playback panel tests that surface deferred-update messaging.
- Recommended validation commands:
  - `cargo test transport_ --manifest-path src-tauri/Cargo.toml`
  - `cargo test score_entries_tone_ --manifest-path src-tauri/Cargo.toml`
  - `cargo test score_instruments_set_ --manifest-path src-tauri/Cargo.toml`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `npm run lint`
  - `npm run check-types`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

### Documentation Impact

- Update docs in the same implementation change if Story 5.3 sharpens the live-edit contract, deferred-update wording, or transport restart boundaries.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `TESTING.md`
  - `.github/copilot-instructions.md`

### Architecture Compliance

- Rust remains the sole authoritative owner of score truth, playback-preparation truth, transport truth, and any deferred-edit playback status.
- The continuous engine remains the realtime playback-plane seam. Score-edit commands must not become ad hoc engine mutators.
- Frontend transport and score-edit surfaces stay thin and descriptive. No second transport or pending-edit model may be introduced in React.
- The supported product path remains rack-first and VST-owned. Do not reintroduce legacy playback fallback or compatibility residue to mask live-edit gaps.

### Library And Framework Requirements

- Keep the existing Tauri command pattern with `#[tauri::command(rename_all = "snake_case")]` handlers and Rust-owned managed state.
- Reuse the existing RAF-polled `transport_get_state` model for transport status; do not add event-heavy playback chatter for every in-flight tick.
- No new state-management library, scheduling dependency, or alternate playback-control surface should be introduced for this story.

### File Structure Notes

- Backend playback and transport policy work belongs under `src-tauri/src/playback/`.
- Score mutation policy hooks should live close to the existing score mutation seams under `src-tauri/src/score/`.
- Frontend transport and score surfaces should stay thin; do not add dedicated playback-debug messaging for deferred edit state.

### Git Intelligence Summary

- Recent Epic 5 work keeps transport ownership, engine cleanup, and reset semantics in backend seams rather than masking issues in the UI.
- The most recent transport commits updated both code and docs together, which is the correct pattern for this story because the core deliverable is an explicit behavioral contract.
- No current diff suggests an existing generalized live-edit seam; Story 5.3 should therefore document and implement a bounded contract, not assume hidden capability.

### Latest Technical Information

- Current repo versions remain the baseline for this story: Tauri 2.10.1, Rust 2021, React 19, TypeScript 5.9, and the existing backend-owned transport-plus-engine model.
- The current transport path already uses a bounded queue window and fresh play-start snapshots. That strengthens the case for deferred score edits rather than attempting mid-pass queue surgery.
- The current transport path already uses a bounded queue window and fresh play-start snapshots. That strengthens the case for deferred score edits rather than attempting mid-pass queue surgery, but it does not by itself justify extra transport messaging.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Story 5.3 acceptance criteria and Epic 5 context.
- `_bmad-output/planning-artifacts/prd.md`
  - MVP scope includes bounded edit-during-playback behavior and FR41 requires continued editing without workflow collapse.
- `_bmad-output/planning-artifacts/architecture.md`
  - Rust-owned playback truth, lock-safe transport behavior, and score-context versus app-scoped ownership boundaries.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Deferred live-edit feedback must be plain, compact, and score-contextual.
- `_bmad-output/implementation-artifacts/5-1-start-pause-and-resume-trusted-score-playback.md`
  - Existing live mix immediacy and transport reset guardrails.
- `_bmad-output/implementation-artifacts/5-2-seek-to-score-positions-without-losing-playback-trust.md`
  - Fresh-snapshot seek restart contract.
- `_bmad-output/implementation-artifacts/5-2a-harden-playback-clocking-and-runtime-reset-semantics.md`
  - Queue-window clocking and reset semantics that Story 5.3 must preserve.
- `/memories/repo/continuous-render-engine-ownership.md`
  - Continuous engine ownership boundary.
- `/memories/repo/transport-note-preroll-grouping.md`
  - Queue-window scheduling and canonical `supported_targets` snapshot reminder.
- `/memories/repo/transport-pause-last-queued-tick.md`
  - Pause and rebuild reset semantics around pending work.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-04: Story created after reviewing Epic 5 planning, PRD FR41, architecture constraints, UX deferred-update guidance, and the completed 5.1, 5.2, and 5.2a transport artifacts.
- 2026-04-04: Repo exploration confirmed that transport playback still runs from a backend-owned play-start snapshot, live mix edits already have a safe in-flight path, and score mutation commands currently have no shared playback-policy seam.
- 2026-04-04: Story guidance was anchored to the current transport, engine, score-entry, and score-instrument seams so the implementation can define a bounded contract without introducing frontend playback truth or audio-thread unsafe behavior.

### Completion Notes List

- Comprehensive developer context created for Story 5.3 with explicit immediate-versus-deferred edit rules, owning seams, regression targets, and documentation impact.
- Story prepared to guide a bounded live-edit implementation that preserves the existing transport snapshot and engine ownership model.
- Added a backend-owned deferred playback refresh contract and surfaced it through general state without creating frontend-owned playback truth.
- Reduced the user-facing surface after review feedback so the deferred-refresh contract remains backend-only instead of adding transport copy that mainly proved the story.
- Kept live mix updates immediate on supported ready routes while deferring score-structure and playback-topology edits to the next safe transport boundary.
- Tightened runtime readiness so stale live activation no longer reports `Ready` after score playback intent changes mid-session.
- Added seam-local Rust and frontend regressions covering deferred active-pass stability, fresh-snapshot rebuilds, stale-runtime truthfulness, and transport status messaging.
- Updated architecture, development, testing, project-context, and agent-instruction docs to make the bounded live-edit contract explicit for later Epic 5 and Epic 6 work.
- Fixed review follow-up issues by preserving playback-topology refresh precedence across mixed deferred edits, reconciling deferred topology work during audio rebuild reset boundaries, and adding regression coverage for both paths.

### File List

- `_bmad-output/implementation-artifacts/5-3-define-bounded-editing-during-playback.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/playback/live_edit.rs`
- `src-tauri/src/playback/mod.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/playback/runtime_activation.rs`
- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/playback/audio.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src-tauri/src/score/entries/tone.rs`
- `src-tauri/src/score/entries/tuplet.rs`
- `src-tauri/src/automation/registry.rs`
- `src-tauri/src/bin/performance.rs`
- `src-tauri/src/files.rs`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/score/flows.rs`
- `src-tauri/src/score/players.rs`
- `src/services/types.ts`
- `src/services/state/context.tsx`
- `src/windows/main/components/transport.tsx`
- `src/windows/main/components/transport.test.tsx`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `TESTING.md`
- `.github/copilot-instructions.md`

### Change Log

- 2026-04-04: Created Story 5.3 as a ready-for-dev artifact focused on bounded edit-during-playback rules, backend-owned deferred-update truth, and score-context feedback.
- 2026-04-04: Implemented the bounded live-edit contract, added deferred-refresh UI and regressions, validated Rust and TypeScript suites, and moved the story to review.
- 2026-04-04: Resolved code review findings, reran targeted and full validation, and moved the story to done.
