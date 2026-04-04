# Story 5.0: Make Transport Drive Audible Routed Score Playback

Status: ready-for-dev

## Story

As a composer,
I want the existing score transport to drive actual audible routed playback,
so that transport becomes the real composition playback path before later trust and realism features extend it.

## Acceptance Criteria

1. Given the active flow contains score-derived notes and at least one supported direct-plugin route is transport-ready, when I start playback from the supported transport surface, then Compose emits audible score-derived note output through the routed VST-backed session rather than advancing timing-only transport state, and the result comes from the backend-owned transport-plus-engine path rather than an audition-only or plugin-proof seam.
2. Given the current MVP remains single-target-first and some score instruments may still be unresolved, when transport playback starts from the current tick, then the one supported ready route can sound through the product path, and explicit `Unrouted` and `NeedsAttention` routes remain recoverable backend truth rather than blocking the ready route or triggering hidden fallback playback.
3. Given Epic 6 will later shape playback realism through expression maps, when this story introduces audible transport playback, then it reuses the existing backend preparation, runtime-activation, and continuous-engine seams, and it stops short of introducing expression-map logic, multi-target transport scope, or a second frontend-owned transport model.

## Tasks / Subtasks

- [ ] Task 1: Make `transport_play` drive audible score playback through the existing backend-owned engine path (AC: 1)
  - [ ] Extend `src-tauri/src/playback/transport.rs` so transport start submits score-derived scheduled note events into the continuous engine instead of staying timing-only.
  - [ ] Keep `src-tauri/src/playback/engine.rs` as the realtime playback-plane seam and `src-tauri/src/playback/audio.rs` as the owner of OS stream lifetime and callback wiring.
  - [ ] Ensure score-derived note scheduling starts from the current transport tick on the authoritative active flow and uses the existing backend projection logic rather than inventing a second transport-note model.
  - [ ] Preserve score-first top-bar transport entry points and avoid introducing a new playback workspace or plugin-debug transport surface.

- [ ] Task 2: Reuse the existing trust-gated playback preparation path without widening MVP scope (AC: 1, 2, 3)
  - [ ] Reuse `prepare_supported_playback(...)` in `src-tauri/src/playback/preparation.rs` as the canonical transport-start handoff instead of creating a transport-local, audition-only, or frontend-owned preparation model.
  - [ ] Preserve the current single-target-first MVP boundary: transport start may sound at most one supported direct-plugin target.
  - [ ] Keep `src-tauri/src/playback/runtime_activation.rs` as the owner of transport-readiness truth, including explicit blocker semantics for unsupported or non-ready routes.
  - [ ] Keep `Unrouted` and `NeedsAttention` routes explicit and recoverable while allowing the one supported ready route to play.

- [ ] Task 3: Keep the frontend transport controls thin and backend-driven (AC: 1, 3)
  - [ ] Reuse `src/windows/main/components/transport.tsx`, `src/services/actions.ts`, and `src/windows/main/hooks/use-keyboard-shortcuts.ts` as the existing transport triggers.
  - [ ] Keep the frontend as a trigger and poll layer over backend transport state via `transport_get_state`; do not add frontend-owned playback truth, event-heavy transport updates, or a second readiness cache.
  - [ ] Preserve the current score-first workflow and top-bar boundaries while transport gains audible behavior.
  - [ ] If minimal UI copy or affordance changes are needed, keep them compact and aligned with existing playback-control grammar.

- [ ] Task 4: Add regression coverage for audible transport start on the supported path (AC: 1, 2, 3)
  - [ ] Extend backend tests in `src-tauri/src/playback/transport.rs` and the owning engine seam to prove transport start now drives scheduled note events into the continuous engine for the supported route.
  - [ ] Add seam-local tests that prove blocked or non-ready starts remain explicit no-ops and do not mutate engine ownership or route truth.
  - [ ] Extend frontend tests in `src/windows/main/components/transport.test.tsx`, `src/services/actions.test.ts`, and `src/windows/main/hooks/use-keyboard-shortcuts.test.ts` only as needed to prove the UI still triggers the same backend commands.
  - [ ] Keep assertions deterministic when flattening map-derived playback state in Rust tests; sort derived vectors before asserting when the source is an `FxHashMap`.

- [ ] Task 5: Update documentation and guardrails for audible transport ownership before later Epic 5 and Epic 6 work (AC: 3)
  - [ ] Update developer-facing docs if this story changes the transport contract from timing-only to real score-driven audible playback.
  - [ ] Update agent guidance if this story sharpens the ownership line between transport, preparation, and the continuous engine.
  - [ ] Document explicitly that expression maps will extend this transport-owned playback path later rather than becoming the first source of real transport audio.

## Dev Notes

### Story Intent

- This story closes the planning gap between Epic 4's first audible audition path and Epic 5's stated ownership of real transport-driven composition playback.
- The goal is not richer realism yet. The goal is to make the existing transport controls drive actual score-derived sound through the already-approved VST-backed product path.
- This story should establish the audible transport baseline that later Epic 5 trust stories and Epic 6 expression-map work can extend without replacing.

### Why This Story Comes Before Expression Maps

- The PRD already treats stable transport-driven playback as the core playback foundation and describes expression maps as a later extension over that foundation.
- Expression maps should shape which articulation, keyswitch, or route-owned behavior transport emits for score content; they should not become the first story that makes transport itself produce audible score playback.
- Putting audible transport playback first keeps Epic 6 incremental: expression maps can extend note selection, articulation choice, and realism on top of a real transport path instead of forcing Epic 6 to solve the transport baseline implicitly.

### Epic And Story Context

- Epic 4 proved a first audible path through bounded score-context auditioning and kept the workflow compact and score-first.
- Story 4.1 established `prepare_supported_playback(...)` as the canonical backend transport and audition preparation seam.
- Story 4.3 established `src-tauri/src/playback/engine.rs` as the continuous realtime playback-plane seam that later transport work is supposed to feed.
- The current repo still documents supported `transport_play` as timing-only, so this story is the missing bridge between the Epic 4 audible foundation and the remaining Epic 5 trust work.

### This Story Owns

- Making the supported top-bar transport start path produce audible score-derived playback through the routed VST-backed session.
- Reusing the existing backend-owned preparation, runtime-activation, and continuous-engine seams rather than creating a second playback path.
- Preserving the current single-target-first MVP boundary while transport start becomes audible.
- Establishing a transport-owned audible baseline that later pause/resume, seek, bounded editing, reopen, recovery, and expression-map stories can extend.

### This Story Does Not Own

- Full pause, resume, seek, or edit-during-playback trust semantics. Those remain later Epic 5 stories.
- Multi-target, shared-slot, multi-part, or multi-timbral transport playback expansion.
- Expression-map authoring, linkage, or realism logic. Epic 6 owns that layer.
- A new DAW-like transport surface, host-debug workspace, or plugin-manager-first playback workflow.
- Broader persistence or migration redesign beyond what a directly required persisted-data change would force.

### Previous Story Intelligence

- Story 4.2 explicitly bounded first audible playback to auditioning and kept full transport behavior out of scope.
- Story 4.3 exists specifically so later transport work can submit events into a continuous engine instead of inventing a second audio-production owner.
- Story 5.1, already created under the prior plan, hardens pause and resume trust on top of a supported audible playback path. This inserted story provides that missing path without renumbering the review-stage 5.1 artifact.

### Current Relevant Seams

- `src-tauri/src/playback/transport.rs`
  - Owns `transport_play`, `transport_pause`, `transport_seek`, `transport_get_state`, transport shared atomics, and the transport worker thread.
- `src-tauri/src/playback/engine.rs`
  - Owns render-active slot tracking, queued live render events, and the continuous playback-plane behavior that transport should feed rather than replace.
- `src-tauri/src/playback/preparation.rs`
  - Owns the canonical transport and audition preparation handoff via `prepare_supported_playback(...)`, including blockers and supported-target resolution.
- `src-tauri/src/playback/runtime_activation.rs`
  - Owns the single-instrument trust gate and resolved route truth that transport start must respect.
- `src-tauri/src/playback/audio.rs`
  - Owns OS stream lifetime and callback wiring; transport must not take over stream ownership.
- `src/windows/main/components/transport.tsx`
  - Existing top-bar score-first transport surface.
- `src/services/actions.ts`
  - Thin frontend command bridge for transport commands.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Existing keyboard transport entry point that should remain aligned with backend transport truth.

### Implementation Guidance

- Keep transport as the producer of score-derived scheduled note events and state transitions, while the engine remains the consumer that processes the live slot-owned session on the audio callback.
- Reuse the current active-flow rule through `ui.flow_id`; do not widen transport playback to inactive flows.
- Reuse existing projected score-event collection and timing logic where possible rather than inventing a second note-scheduling representation just for audible transport start.
- Keep the MVP bounded to one supported direct-plugin root route and one live audible target.
- Treat blocked starts as explicit backend outcomes. If preparation cannot produce a supported target, transport should remain a no-op without hidden fallback audio.
- Keep frontend transport surfaces thin; do not solve backend transport-audio ownership gaps in React.

### Risks And Regression Traps

- Reintroducing an audition-only or plugin-proof seam as the transport audio owner would split product playback truth immediately.
- Letting transport take over stream lifetime or canonical live-session identity would violate the current engine and audio ownership boundaries.
- Broadening the story into pause/resume/seek trust, expression-map realism, or multi-target transport behavior would weaken verification and blur epic sequencing.
- Treating unresolved routes as hard blockers for every start would violate the current recoverable-route model.
- Depending on `FxHashMap` iteration order in transport assertions will create unstable regressions; derived vectors should be sorted before asserting.

### Testing Requirements

- Extend Rust tests near the owning seams in `src-tauri/src/playback/transport.rs` and `src-tauri/src/playback/engine.rs` for:
  - transport start driving scheduled note events into the continuous engine for the supported target;
  - score-derived transport start from the current tick on the active flow;
  - blocked starts leaving transport and engine ownership unchanged;
  - mixed ready versus non-ready route handling that preserves the supported ready path.
- Extend seam-local preparation coverage in `src-tauri/src/playback/preparation.rs` if additional blocker or mixed-readiness assertions are needed.
- Extend frontend tests only as needed in:
  - `src/windows/main/components/transport.test.tsx`
  - `src/services/actions.test.ts`
  - `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
    to prove the UI still triggers the same backend transport commands.
- Recommended validation commands for the implementation change:
  - `cargo test transport_play_with_states --manifest-path src-tauri/Cargo.toml`
  - `cargo test prepare_supported_playback --manifest-path src-tauri/Cargo.toml`
  - `npm run test:js -- src/windows/main/components/transport.test.tsx src/services/actions.test.ts src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
  - `npm run lint`
  - `npm run check-types`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Documentation Impact

- Update docs in the same implementation change if this story changes the supported transport contract from timing-only to real score-driven audible playback.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if transport-versus-engine ownership rules become sharper

### Architecture Compliance

- Rust remains the sole authoritative owner of route truth, playback preparation, transport state, transport lifecycle, and playback-engine truth.
- The continuous engine remains the realtime playback-plane seam; transport may submit events and manage scheduling, but it must not become the owner of stream lifetime or live plugin-session identity.
- Frontend transport surfaces stay as thin trigger and display layers over backend state. No second transport model may be introduced in React.
- The supported product path remains VST-owned. Do not reintroduce sampler-era or SCVI-era fallback playback, hidden readiness logic, or compatibility-residue ownership.
- Expression maps must remain later extensions over this transport-owned audible path, not the first story that makes transport produce audible score playback.

### Library And Framework Requirements

- Keep the existing Tauri 2 command bridge pattern: frontend `invoke(...)` calls into Rust `#[tauri::command(rename_all = "snake_case")]` handlers with managed state injected through `tauri::State`.
- Current repo guidance still supports the `Mutex`-wrapped managed-state command pattern so long as guards are not held across async await points.
- No new frontend state library, event bus, transport scheduler dependency, or alternate audio host stack should be introduced for this story.

### File Structure Notes

- Backend transport and playback work belongs under `src-tauri/src/playback/`.
- Frontend transport interaction stays under `src/windows/main/` and `src/services/`.
- Do not create new playback-specific windows or top-level folders for this story.

### Git Intelligence Summary

- Epic 4 closed with the continuous engine correction and compact score-first audible path in place.
- Story 5.1 has already been implemented as trust hardening on top of a supported audible path, which exposes this missing prerequisite directly.
- The inserted Story 5.0 is therefore a planning correction that makes Epic 5 match FR23 and FR27 without forcing a renumber of the review-stage Story 5.1 artifact.

### Latest Technical Information

- Current repo versions remain the baseline for this story: Tauri 2.10.1, Rust 2021, and the existing backend command-plus-managed-state architecture.
- The approved architecture continues to treat transport and audible playback as backend-owned seams split between a control plane and a realtime playback plane, with future reusable playback assets such as expression maps added later on top of that foundation.
- No external framework change justifies introducing a new transport ownership model or making expression-map work the first owner of audible transport playback.

### References

- `_bmad-output/planning-artifacts/prd.md` - stable transport-driven playback goals and the explicit statement that expression maps should extend, not replace, the playback foundation.
- `_bmad-output/planning-artifacts/architecture.md` - transport, continuous playback-plane, and future expression-map asset boundaries.
- `_bmad-output/planning-artifacts/epics.md` - Epic 5 goals and the later stories that should build on this transport-owned audible baseline.
- `docs/architecture.md` - current timing-only transport statement that this story is expected to replace.
- `docs/project-context.md` - transport polling, active-flow ownership, and playback ownership rules.
- `/memories/repo/continuous-render-engine-ownership.md` - engine ownership boundary and render-target regression trap.
- `/memories/repo/epic4-playback-preparation-story-4-1.md` - preparation seam reuse guardrails.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-04: Identified a planning gap between PRD FR23 and FR27, which expect transport-driven playback, and the current Epic 5 story inventory, which begins with trust-hardening on top of an assumed already-audible transport path.
- 2026-04-04: Confirmed from the PRD and architecture that expression maps are intended to extend a real playback foundation later rather than become the first owner of audible transport playback.
- 2026-04-04: Confirmed from Epic 4 and Story 4.3 artifacts that `prepare_supported_playback(...)` and `src-tauri/src/playback/engine.rs` already form the intended backend seams for an explicit audible transport story.
- 2026-04-04: Inserted Story 5.0 as a deliberate prerequisite correction without renumbering the existing review-stage Story 5.1 artifact.

### Completion Notes List

- Added an explicit ready-for-dev Epic 5 story for making transport drive real routed score playback.
- Kept the new story deliberately bounded so later Epic 5 trust stories and Epic 6 expression-map stories can extend it cleanly.
- Updated Epic 5 planning so FR23 now lives with the epic that owns real transport playback rather than with Epic 4's audition-first audible foundation.

### File List

- `_bmad-output/implementation-artifacts/5-0-make-transport-drive-audible-routed-score-playback.md`
