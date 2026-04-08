# Story 5.4: Restore Playback State On Project Reopen

Status: done

## Story

As a composer,
I want valid playback routes and rack state to restore when I reopen a project,
so that playback remains part of the normal project lifecycle.

## Acceptance Criteria

1. Given Story 5.0 established a supported audible transport playback path, and a saved project contains valid playback intent and the required plugins are available, when I reopen the project, then Compose restores the relevant rack slots and instrument routes, and playback-relevant state remains coherent with the backend-owned model.
2. Given project files must remain portable, when reopen restoration occurs, then restoration resolves saved project intent against machine-local plugin inventory, and project files do not contain machine-local cache or discovery truth.
3. Given reopened playback state is part of trusted composition workflow, when reopen scenarios are validated, then state integrity across save, close, and reopen is preserved, and restoration behavior is suitable for regression validation.

## Tasks / Subtasks

- [x] Task 1: Audit the existing save and reopen seams against Story 5.4 before changing behavior (AC: 1, 2, 3)
  - [x] Confirm `save_project_to_file` persists rack slots, instrument routes, and portable slot target state only.
  - [x] Confirm `load_project_from_file` stops any active transport work, resets ephemeral UI state, restores the score payload, re-syncs transport state to reopened score truth, and recomputes runtime readiness from current machine-local inventory.
  - [x] Confirm no frontend-owned reopen orchestration or parallel playback truth is required for the supported path.

- [x] Task 2: Close only the remaining backend gaps discovered by the seam audit (AC: 1, 2)
  - [x] Prefer refining `src-tauri/src/files.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/playback/runtime_activation.rs`, or closely related seams over introducing any new reopen workflow.
  - [x] Keep rack-first ownership intact: racks keep slot lifecycle, slots keep target intent, and instruments restore only routes to existing slots.
  - [x] Do not persist transport play state, queue contents, machine-local inventory truth, or any live runtime host object into `.soloc` files.

- [x] Task 3: Strengthen regression proof for reopened playback readiness and transport preparation (AC: 1, 2, 3)
  - [x] Extend seam-local load/save tests around `src-tauri/src/files.rs` to prove valid reopen restores slot targets, routes, and portable plugin state while recomputing readiness from inventory.
  - [x] Add or extend preparation-focused tests near `src-tauri/src/playback/preparation.rs` or transport-owned seams to prove reopened score state feeds the supported playback preparation path without special-case frontend coordination.
  - [x] Keep one unavailable-inventory regression that proves persisted intent remains intact while runtime truth degrades honestly, without broadening Story 5.4 into Story 5.5 recovery UX.

- [x] Task 4: Document the verified reopen contract and scope boundaries (AC: 2, 3)
  - [x] Update developer-facing docs if the reopen contract, persistence boundary, or validation expectations are clarified during implementation.
  - [x] Record explicitly that this story restores playback rack and route intent on reopen, but does not persist active transport playback, current queue state, or broader missing-plugin recovery UX.

## Dev Notes

### Story Intent

- Initial seam review says this story is verification-first, not a greenfield implementation story.
- The existing load and runtime-refresh path already appears to satisfy most of the acceptance criteria for valid-plugin reopen behavior.
- The likely remaining work is to prove the transport-era contract end to end, tighten any small backend seams that the audit exposes, and avoid rediscovering or duplicating Epic 1 reopen groundwork.

### Previous Story Intelligence

- Story 5.3 reinforced that transport playback runs from a backend-owned snapshot and that pause, stop, seek, and transport resets are hard backend-owned boundaries; Story 5.4 must preserve that contract on reopen rather than persisting in-flight transport state.
- Story 5.3 also kept live playback truth backend-owned and documentation-aligned, which is the right pattern here: sharpen the reopen proof at the owning seams instead of inventing frontend state to narrate it.
- Recent commits `2abd4919`, `dcd661b1`, and `2411a11a` kept Epic 5 transport hardening concentrated in backend playback seams and adjacent docs. Follow the same discipline.

### Existing Seams Already Doing Important Work

- `src-tauri/src/files.rs`
  - `save_project_to_file(...)` already serializes the project through the `.soloc` container after syncing slot-target state from runtime-backed score intent.
  - `load_project_from_file(...)` already stops active transport playback before loading, restores project data, clears ephemeral UI selection/tool state, syncs transport truth to the reopened score, refreshes runtime activation from app inventory, and emits refreshed state.
  - Existing tests already prove single-route reopen readiness and explicit degraded runtime behavior when inventory cannot resolve the saved plugin.
- `src-tauri/src/lib.rs`
  - `refresh_playback_activation_runtime(...)` resolves expected slot targets from score intent plus machine-local inventory, syncs slot runtime sessions, and rebuilds playback activation truth from live slot targets.
- `src-tauri/src/playback/runtime_activation.rs`
  - `expected_slot_targets_from_score_and_inventory(...)` and `from_live_slot_targets(...)` already model the portable-intent plus machine-local-inventory boundary this story needs.
- `src-tauri/src/score/playback.rs`
  - The persisted playback model already keeps rack slots, slot targets, route targets, route addresses, and optional portable plugin state in the score payload.

### This Story Owns

- Verification that the existing save, load, runtime-refresh, and preparation seams collectively satisfy the valid-plugin reopen contract for Epic 5.
- Any minimal backend corrections needed to make reopened playback preparation trustworthy in the current transport era.
- Regression coverage that proves save, close, reopen, and playback preparation preserve portable intent and trustworthy runtime restoration.

### This Story Does Not Own

- Persisting active transport playback state, current tick as a resumable play contract, queued transport work, or in-flight engine state across app close and reopen.
- Broader missing-plugin or unavailable-route recovery UX; Story 5.5 owns graceful recovery behavior beyond the narrow integrity guardrails needed here.
- New score-context UI whose main purpose is to literalize the acceptance criteria.
- Any new user-facing panel, badge, banner, modal, warning treatment, or status copy added only to "show" that reopen worked.
- HALion-style multi-timbral, multi-slot, or generalized multi-route expansion beyond the already-supported rack-first model.

### Current Relevant Seams

- `src-tauri/src/files.rs`
  - Owns `.soloc` save/load orchestration and already contains the strongest reopen regression seam.
- `src-tauri/src/lib.rs`
  - Owns backend refresh of playback activation runtime from score plus inventory truth.
- `src-tauri/src/playback/runtime_activation.rs`
  - Owns mapping from persisted rack and route intent to live ready or non-ready runtime truth.
- `src-tauri/src/playback/preparation.rs`
  - Owns the supported playback preparation snapshot and should be the seam that proves reopened state remains playable without extra coordination.
- `src-tauri/src/playback/transport.rs`
  - Owns transport reset and tempo sync behavior after reopen; do not turn reopen into a second transport-state persistence model.
- `src-tauri/src/score/playback.rs`
  - Defines the persisted rack-first score playback structures this story must preserve.

### Implementation Guidance

- Start with the seam audit. If the current code already satisfies an acceptance criterion, keep it and strengthen proof instead of rewriting it.
- Treat valid reopen as: portable score playback intent is restored from the file, machine-local inventory re-resolves what is currently available, slot-owned runtime sessions are rebuilt from that intent, and supported playback preparation can run truthfully afterward.
- Keep the persistence boundary strict. `.soloc` may store portable route and slot intent plus portable plugin state blobs, but it must not store machine-local plugin discovery truth, cached availability, live runtime sessions, or transport-owned queue state.
- Keep transport reopen behavior intentionally bounded. Reopen should reset transport truth to the reopened score state, not recreate a previously playing session.
- If coverage is the only real gap, prefer adding seam-local tests over adding runtime features or UX changes.
- If a missing gap points to degraded unavailable routes, cover only the integrity boundary needed here and leave richer user-facing recovery semantics to Story 5.5.
- Default expected UX delta for Story 5.4 is none. Only consider a user-facing change if verification proves an existing product-facing reopen failure that cannot be resolved inside the current backend and score-context surfaces.
- If such a user-facing gap is discovered, treat it as follow-up scope that requires explicit approval rather than silently expanding this story.

### Risks And Regression Traps

- Reopening may look complete because rack and route JSON round-trips, while preparation or runtime activation after load still misses a transport-era edge. Validate the whole backend chain, not just serialization.
- It would be easy to overreach into missing-plugin recovery UX, but this story should stay focused on valid-plugin restore plus portable-intent integrity.
- Persisting machine-local plugin paths, cache truth, or live runtime host objects into project files would violate the architecture even if it appears to simplify reopen.
- Persisting play or queue state across project reopen would blur the transport-owned reset contract established in Stories 5.0 through 5.3.
- Adding new frontend state to "help" reopen would create a second owner for playback truth and make the current architecture weaker, not stronger.

### Testing Requirements

- Extend Rust tests near `src-tauri/src/files.rs` for:
  - valid-plugin save, load, and reopen restoration of rack slot targets, instrument routes, and portable plugin state;
  - unavailable-inventory reopen degradation that preserves persisted intent without silently rewriting it;
  - any multi-instrument or multi-slot scenario needed to prove the current rack-first model survives reopen without route drift.
- Extend preparation or transport-adjacent Rust tests only as needed to prove reopened score state feeds `prepare_supported_playback(...)` correctly after runtime refresh.
- Keep tests seam-local. Do not create a new cross-cutting reopen harness if `files.rs`, preparation, or transport tests can prove the contract.
- Recommended validation commands:
  - `cargo test files --manifest-path src-tauri/Cargo.toml`
  - `cargo test prepare_supported_playback --manifest-path src-tauri/Cargo.toml`
  - `cargo test transport_ --manifest-path src-tauri/Cargo.toml`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `npm run lint`
  - `npm run check-types`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

### Documentation Impact

- Update docs in the same change if Story 5.4 clarifies what playback state does and does not survive project reopen.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `TESTING.md`
  - `.github/copilot-instructions.md`
- If implementation remains proof-only with no behavior change, keep doc updates minimal and focused on validation expectations rather than rewriting product semantics.

### Architecture Compliance

- Rust remains the sole authoritative owner of persisted playback truth, runtime activation truth, playback preparation truth, and transport truth.
- Reopen restoration must continue to resolve saved project intent against current machine-local inventory rather than persisting resolved runtime availability in project files.
- The supported product path remains rack-first: racks own slot lifecycle, slots own target intent and runtime session identity, and instruments own routes to existing slots.
- Reopen should preserve the backend-owned transport reset contract. No second resume-on-reopen model should be introduced.
- User-facing recovery chrome must remain narrower than backend truth. Do not expand warning UI just to prove restore behavior.
- Zero new UI is the preferred outcome for this story.

### Library And Framework Requirements

- Keep the existing Tauri command pattern with `#[tauri::command(rename_all = "snake_case")]` handlers and Rust-owned managed state.
- Reuse the existing runtime-activation and playback-preparation seams instead of introducing a dedicated reopen service or frontend helper.
- No new persistence library, state-management library, or transport-control surface should be introduced for this story.

### File Structure Notes

- Persistence and reopen work belongs primarily under `src-tauri/src/files.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/playback/`, and `src-tauri/src/score/playback.rs`.
- Tests should stay close to the owning save/load, preparation, and transport seams.
- Frontend changes should be avoided unless a real product-facing gap emerges during verification.
- A pure backend and test-only completion is the expected happy path.

### Git Intelligence Summary

- Recent Epic 5 work kept transport trust, reset semantics, and runtime coherence in backend seams. Story 5.4 should keep the same center of gravity.
- The current branch history suggests story preparation happened before the latest transport hardening landed, so this story should explicitly build on the now-completed 5.3 reset contract rather than repeating older reopen assumptions.

### Latest Technical Information

- Current repo versions remain the baseline: Tauri 2.10.1, Rust 2021, React 19, and TypeScript 5.9.
- The current supported transport path depends on `prepare_supported_playback(...)` snapshots and backend-owned queueing, which reinforces that reopen restoration should rebuild truthful preparation state rather than persist live transport state.
- Existing `files.rs` tests already show that portable plugin state blobs and rack-first routing structures round-trip correctly at the serialization seam; the remaining focus is confidence in transport-era restore proof, not a new persistence model.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Story 5.4 acceptance criteria and Epic 5 scope.
- `_bmad-output/planning-artifacts/prd.md`
  - PRD requirements around project reopen integrity, portable persistence, and trusted playback.
- `_bmad-output/planning-artifacts/architecture.md`
  - Rust-owned playback truth, portable `.soloc` boundaries, rack-first ownership, and transport reset rules.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Recovery and readiness UX should stay score-contextual, compact, and non-diagnostic.
- `_bmad-output/implementation-artifacts/5-3-define-bounded-editing-during-playback.md`
  - Transport reset and backend-owned playback truth contract that reopen must preserve.
- `/memories/repo/slot-owned-live-runtime-sessions-story-1-4.md`
  - Slot-owned runtime session and portable-intent reopen lessons.
- `/memories/repo/files-load-test-helper-runtime-refresh.md`
  - Files-test helper warning: restore tests must recompute `PlaybackActivationRuntime` the same way the real load seam does.
- `/memories/repo/playback-rack-first-model.md`
  - Rack-first ownership reminders relevant to reopen restoration.
- `/memories/repo/playback-readiness-ui-semantics.md`
  - UI should stay narrower than backend route truth during restore and recovery.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-04: Story created after reviewing Epic 5 planning, PRD and architecture persistence rules, UX recovery constraints, prior Story 5.3 transport-reset guardrails, and the current backend save/load/runtime-activation seams.
- 2026-04-04: Seam review indicated Story 5.4 is verification-first: `files.rs`, runtime activation, and persisted rack-first playback structures already implement most valid-plugin reopen behavior.
- 2026-04-04: Story guidance was therefore narrowed to minimal backend deltas plus stronger regression proof, especially around reopen-to-preparation trust.
- 2026-04-04: Audit confirmed `.soloc` save data stays limited to the project DTO and portable slot target state, while project load already stops transport, resets ephemeral UI state, re-syncs transport truth, and refreshes runtime readiness from machine-local inventory.
- 2026-04-04: Discovered and fixed one transport-era gap: project load could retain stale `pending_playback_refresh`, so `load_project_from_file(...)` and the matching files-test helper now clear deferred playback-refresh state before transport sync and runtime rebuild.
- 2026-04-04: Added reopen regressions in `src-tauri/src/files.rs` and `src-tauri/src/playback/preparation.rs`, then validated with `cargo test files`, `cargo test prepare_supported_playback`, `cargo test`, `npm run lint`, `npm run check-types`, `npm run test:js`, and `cargo clippy --all-targets --all-features -- -D warnings`.

### Completion Notes List

- Audit confirmed the existing save/load/runtime-activation seams already satisfy the rack-first portable-intent reopen model without any frontend-owned reopen orchestration.
- Fixed `src-tauri/src/files.rs` so project load clears stale deferred playback-refresh state before transport truth and runtime readiness are rebuilt from the reopened score and current machine-local inventory.
- Added reopen regressions proving valid-plugin reopen restores slot targets, routes, and portable plugin state into supported playback preparation, and that unavailable inventory keeps persisted intent intact while runtime truth degrades honestly.
- Updated `docs/architecture.md`, `docs/development-guide.md`, `docs/data-models.md`, and `TESTING.md` to document reopen as a restore-and-reset boundary rather than a resume-on-reopen path.
- Validation completed successfully with `cargo test files --manifest-path src-tauri/Cargo.toml`, `cargo test prepare_supported_playback --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml`, `npm run lint`, `npm run check-types`, `npm run test:js`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`.

### File List

- `TESTING.md`
- `_bmad-output/implementation-artifacts/5-4-restore-playback-state-on-project-reopen.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src-tauri/src/files.rs`
- `src-tauri/src/playback/preparation.rs`

### Review Findings

- [x] [Review][Patch] Reopen regression tests bypass the production runtime-refresh seam [src-tauri/src/files.rs:520]
- [x] [Review][Patch] Reopen coverage does not assert the hard-reset transport contract it now documents [src-tauri/src/files.rs:2074]
- [x] [Review][Patch] In-flight slot-session matching can reuse stale plugin state across reopen [src-tauri/src/playback/plugin_editor_poc.rs:277]

## Change Log

- 2026-04-04: Audited the reopen save/load seams, cleared stale deferred playback-refresh state during project load, added reopen-to-preparation regressions, and documented the restore-vs-reset contract for playback state on reopen.
