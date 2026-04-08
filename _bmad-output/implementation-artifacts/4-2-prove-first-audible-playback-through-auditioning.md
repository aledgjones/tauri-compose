# Story 4.2: Prove First Audible Playback Through Auditioning

Status: done

## Story

As a composer,
I want to audition routed written material from the score workflow,
so that Compose can prove the first audible playback path before full transport behavior is introduced.

For this story, auditioning means playing one specific pitch for 500ms at a moderate velocity using a default legato articulation so the user can hear pitches in the instrument voice while interacting with the score.

## Acceptance Criteria

1. Given at least one score instrument has a valid routed playback target, when I trigger audition from score-context interaction, then Compose plays the targeted score pitch for 500ms through the routed VST-backed instrument voice at a moderate velocity with a default legato articulation, and the audible result comes from score-derived musical content rather than a separate plugin-only test path.
2. Given some instruments are routed and others are not, when auditioning begins for the currently targeted tone, then the selected tone's routed instrument can sound through the supported playback path, and unrouted instruments remain explicitly non-ready rather than blocking the workflow.
3. Given Epic 4 is proving playback basics rather than full transport trust, when this story is completed, then the result demonstrates a trustworthy first-audible path through auditioning, and it does not require play, pause, resume, or seek behavior that belongs to later transport stories.
4. Given the user is interacting with score tones, when a tone is selected through the renderer or sequencer, its pitch is changed through keyboard commands or sequencer vertical dragging, or its accidental is changed through the left-panel control or keyboard command, then Compose triggers the supported 500ms audition for the affected pitch through the same routed score-context playback path.

## Tasks / Subtasks

- [x] Task 1: Add a backend-owned audition command that reuses the canonical playback-preparation seam (AC: 1, 2, 3, 4)
  - [x] Start from `src-tauri/src/playback/preparation.rs` and `src-tauri/src/playback/transport.rs` instead of introducing a plugin-only or frontend-owned preparation path.
  - [x] Create the supported audition entry seam under `src-tauri/src/playback/` and expose it through a Tauri command registered in `src-tauri/src/lib.rs`.
  - [x] Reuse `prepare_supported_playback(...)` so auditioning consumes the same single-target preparation artifact, resolved-route truth, and audio sample rate that transport uses.
  - [x] Pass explicit note parameters for the audition contract: one pitch, 500ms duration, moderate velocity, and default legato articulation.
- [x] Task 2: Deliver audible VST-backed note playback through the existing slot-owned runtime-session architecture (AC: 1, 3, 4)
  - [x] Reuse slot-owned live sessions from `src-tauri/src/playback/plugin_editor_poc.rs` where the architecture already keeps the canonical running plugin instance.
  - [x] Feed score-derived musical content into the supported audible path rather than probing a plugin in isolation.
  - [x] Keep the first-audible implementation bounded to the current single-target-first model: one prepared direct-plugin target, one playable instrument path, no multi-target expansion.
  - [x] Use a default legato articulation for now instead of expanding into articulation mapping or broader expression logic.
- [x] Task 3: Connect the score-context audition workflow without creating a second playback model (AC: 1, 2, 4)
  - [x] Extend `src/services/actions.ts` with an audition action that invokes the new backend command using snake_case arguments.
  - [x] Implement the existing `auditionTone` hook path in `src/windows/main/hooks/use-keyboard-shortcuts.ts` so note-edit auditioning uses the supported backend path.
  - [x] Trigger audition when a tone is selected in renderer or sequencer, when its pitch changes through keyboard commands, when its pitch changes through sequencer vertical dragging, and when its accidental changes through the left-panel control or keyboard command.
  - [x] Respect the existing audition preference and current route-readiness semantics rather than adding frontend-derived fallback readiness.
- [x] Task 4: Preserve explicit non-ready states and keep the workflow score-first (AC: 1, 2, 3, 4)
  - [x] Keep unrouted and `NeedsAttention` instruments explicit and non-blocking when another routed supported target can audition.
  - [x] Do not broaden this story into full transport semantics, plugin-manager-first flows, or a generic plugin test harness.
  - [x] Keep the audible result anchored to score-context user actions and score-derived content, not a separate host-debugging workflow.
  - [x] Keep the targeting rule explicit: audition the affected tone's pitch in the currently addressed instrument voice, not an arbitrary first playable plugin.
- [x] Task 5: Add seam-local regression coverage and document the canonical first-audible path (AC: 1, 2, 3, 4)
  - [x] Add Rust tests near the new audition seam and extend existing preparation or transport tests where reuse is appropriate.
  - [x] Extend the existing keyboard-shortcut, renderer or sequencer selection, and action tests in `src/` so the supported audition workflow is covered from each approved trigger path.
  - [x] Update developer-facing documentation and agent guidance if the canonical backend-owned audition seam or required reuse rules become more explicit.

### Review Findings

- [x] [Review][Patch] Frontend audition gating ignores the single-target transport trust gate [src/windows/main/utils/audition.ts:7]
- [x] [Review][Patch] Audition command failures are unhandled across score-context triggers and lack failure-path regression coverage [src/windows/main/hooks/use-keyboard-shortcuts.ts:255]
- [x] [Review][Patch] Vertical drag can queue duplicate same-pitch auditions before `lastPitch` updates [src/windows/main/components/sequencer-track.tsx:661]
- [x] [Review][Patch] A selected tone can miss its first preview when readiness changes after selection [src/windows/main/hooks/use-keyboard-shortcuts.ts:249]

## Dev Notes

### Story Intent

- Story 4.2 is the first audible proof on top of Story 4.1's shared backend preparation seam.
- The goal is not general playback sophistication. The goal is to prove that routed score content can become audible through the real VST-backed product path while keeping the architecture honest.
- This story must keep the app score-first and rack-first. It should not teach the system to pass by running a separate plugin test path that bypasses score routing truth.
- Auditioning is intentionally narrow in this story: one specific pitch, 500ms long, moderate velocity, default legato articulation, triggered while the user is actively interacting with score tones.
- Follow-up triage on 2026-04-03 determined that the current bounded burst-render implementation is not yet a trustworthy working audible path in practice, so this story is reopened and blocked pending the continuous-render correction in Story 4.3.

### Epic And Story Context

- Epic 4 is now active because the Story 1.3 through 1.6 trust-restoration gate and maintainer proof are complete on the corrected slot-owned-session architecture.
- Story 4.1 established `prepare_supported_playback(...)` as the canonical backend seam for transport and upcoming audition work.
- Story 4.2 proves first audible playback through auditioning only. Full transport trust, repeated play or seek, and broader transport behavior stay in Epic 5.

### This Story Owns

- Creating the supported backend-owned audition command and command registration.
- Reusing the canonical playback-preparation seam and slot-owned runtime-session model for audible note output.
- Wiring the current score-context audition trigger paths so tone selection, pitch edits, and accidental edits exercise the real product path.
- Preserving explicit unrouted and attention-needed behavior while proving the routed audible path.

### This Story Does Not Own

- Full transport play, pause, resume, or seek trust behavior.
- Multi-instrument, multi-target, shared-slot, or multi-timbral audible playback expansion.
- A plugin-only proof path promoted into product ownership.
- Broad playback UX redesign, app-scoped playback windows, or a DAW-like control surface.
- Persistence or migration redesign beyond what is strictly required by an approved persisted-data change.

### Prior Art Research Summary

- `src-tauri/src/playback/preparation.rs` now owns `PreparedPlaybackTarget`, `PlaybackPreparation`, and `prepare_supported_playback(...)`. Story 4.2 must consume this seam rather than fork it.
- `src-tauri/src/playback/transport.rs` already calls `prepare_supported_playback(...)` before starting the timing-only transport path. This is the closest canonical command pattern for the new audition seam.
- `src-tauri/src/playback/plugin_editor_poc.rs` owns the slot-keyed runtime-session registry for live VST instances. That registry is the strongest existing basis for sounding notes through the already-running product-path plugin instance.
- `src-tauri/src/playback/plugin_playback_poc.rs` is bounded prior art for proving audible output and MIDI note delivery, but it is explicitly transport-adjacent proof support only. It must not become the canonical live playback state model.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts` already has an `auditionTone` stub and established shortcut gating via the audition preference. Story 4.2 should finish that path rather than inventing a different UI entry point.
- The user intent for this story is now explicit: audition is not phrase playback or transport-lite. It is a 500ms pitch preview in the instrument voice while interacting with score tones.

### Cross-Story Intelligence

- Story 1.4 moved canonical live VST ownership into slot-keyed runtime sessions and made editor windows transient views over those sessions.
- Story 1.6 extended slot-owned runtime identity to include restorable plugin state, so reopen or target refresh can rebuild the correct live instance.
- Story 3.3 kept `resolved_playback_route_by_instrument_id` backend-owned and preserved explicit `Unrouted` and `NeedsAttention` semantics.
- Story 4.1 blocked unsupported multiple-routed-instrument starts, preserved empty-slot `NeedsAttention`, and documented that only broken assigned plugin routes get warning chrome in Sequence.

### Architecture Compliance

- Rust remains the only owner of plugin inventory truth, rack state, slot state, route state, playback-preparation state, and transport-facing playback truth.
- The app must stay rack-first: racks own slots, slots own plugin assignment or unassignment, and instruments only own routes to existing slots.
- `.soloc` continues to persist portable score and playback intent only. No live runtime session or machine-local plugin inventory state should be persisted to make audition pass.
- The realtime path remains lock-sensitive. Any command or helper added here must capture what it needs without turning the audio side into a UI-thread-dependent path.
- The product path remains VST-owned. Do not reintroduce sampler-era or SCVI fallback ownership.

### Current Relevant Seams

- `src-tauri/src/playback/preparation.rs`
  - Canonical backend playback-preparation seam. Audition must consume this directly or through a thin shared helper.
- `src-tauri/src/playback/transport.rs`
  - Current supported command pattern for playback preparation and timing-only playback start.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Slot-owned live runtime-session registry and native-editor-host lifecycle. Reuse the live-session foundation instead of spinning up a plugin-only side path.
- `src-tauri/src/playback/plugin_playback_poc.rs`
  - Prior-art proof seam for offline audible detection and MIDI note delivery. Study it, but do not promote it into a second canonical runtime.
- `src-tauri/src/playback/audio.rs`
  - Source of active output-device and sample-rate truth.
- `src-tauri/src/lib.rs`
  - Tauri command registration point for the new audition command.
- `src/services/actions.ts`
  - Frontend mutation and command-invocation surface. Add the audition action here.
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
  - Existing note-edit audition trigger path waiting for the real implementation.
- `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
  - Existing tests already cover shortcut gating and should be extended to assert the supported audition invocation.

### Implementation Guidance

- Prefer a thin new backend seam for auditioning under `src-tauri/src/playback/` that accepts or derives a `PreparedPlaybackTarget` from `prepare_supported_playback(...)`. Keep preparation and audible triggering separate enough that transport and audition can share the same preparation contract without duplicating logic.
- Reuse the active audio sample rate from `AudioContext` exactly as Story 4.1 established. Do not negotiate or derive a second sample-rate path for audition.
- Use score-derived musical content. The targeted note or passage should come from the score-context action and current score meaning, not from a free-standing plugin ping.
- Use score-derived pitch targeting. The affected tone's resulting pitch should be what sounds, whether the trigger came from selection, pitch change, or accidental change.
- Keep the first implementation bounded. If broader multi-note, multi-route, or transport-style behavior becomes tempting, capture it as future scope and stop.
- Follow the current Tauri command pattern: command names registered once in `generate_handler!`, snake_case arguments, `Result<..., String>` or another serializable error type, and no module-scoped duplication.
- Frontend action wiring should stay inside `actions.ts` and the existing keyboard-shortcut hook. Do not add a React-owned cache of playback readiness, target identity, or sample-rate truth.
- Preserve the current score-context workflow. Approved entry paths are tone selection through renderer or sequencer, pitch edits through keyboard or sequencer vertical dragging, and accidental edits through the left-panel control or keyboard command, each gated by user preference and backend route readiness.

### Risks And Regression Traps

- Creating a second preparation path for audition would immediately undermine Story 4.1 and split transport versus audition truth.
- Using `plugin_playback_poc_probe` directly as the audible product path would bypass score ownership and create a plugin-test workflow instead of score-first playback.
- Rehydrating or tearing down slot sessions incorrectly during audition could regress editor-host stability or same-session route behavior.
- Holding `AppContext`, transport, or session locks too long while doing host work risks UI stalls and fragile audio behavior.
- Treating unrouted or empty-slot instruments as hard blockers would violate the explicit non-ready workflow and break AC2.
- Broadening the story into transport trust would blur the Epic 4 and Epic 5 boundary and make scope verification weak.

### Testing Requirements

- Add seam-local Rust tests near the new audition seam and extend current preparation or transport tests where shared behavior is being asserted.
- Minimum backend coverage should prove:
  - a valid routed single-target prepared path can produce a supported audition command result;
  - explicit `Unrouted` and `NeedsAttention` states remain honest and do not silently reroute;
  - multiple routed instruments remain blocked by the same single-target trust gate used by transport;
  - the audition seam consumes the active audio sample rate from backend state rather than frontend input;
  - the audition seam uses the bounded note contract of 500ms duration, moderate velocity, and default legato articulation.
- Minimum frontend coverage should prove:
  - tone selection through renderer or sequencer invokes the audition action when the preference is enabled and a routed playable instrument exists;
  - the existing note-edit shortcut path invokes the audition action after pitch or accidental changes when the preference is enabled and a routed playable instrument exists;
  - sequencer vertical pitch dragging invokes the same audition path;
  - pitch-edit operations still proceed when audition is disabled or the instrument is not currently playable;
  - command failures surface as expected without creating a second readiness model.
- Validation commands for the implementation change should include:
  - `npm run test:js -- src/windows/main/hooks/use-keyboard-shortcuts.test.ts src/services/actions.test.ts`
  - `npm run lint`
  - `npm run check-types`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`

### Documentation Impact

- Update docs in the same change if Story 4.2 establishes a clearly named canonical backend-owned audition seam or clarifies how first-audible playback reuses slot-owned sessions.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if new agent-facing guardrails are required

### Latest Technical Information

- Current repo versions remain the baseline for this story: Tauri 2.10.1, Rust 2021, and cpal 0.17.3.
- External verification still aligns with the current repo patterns:
  - CPAL 0.17.3 `StreamConfig` remains defined by `channels`, `sample_rate`, and `buffer_size`, which matches the repo's existing `AudioContext` stream-configuration approach.
  - Tauri 2 command guidance still supports backend commands defined in modules and registered once through `tauri::generate_handler!`, with serializable `Result` errors and snake_case argument support via `rename_all` when needed.
- No library upgrade, alternate host stack, or frontend-owned event protocol is required for Story 4.2.

### Project Structure Notes

- Backend implementation should remain concentrated under `src-tauri/src/playback/` and `src-tauri/src/lib.rs`, with frontend wiring limited to `src/services/` and the current main-window hook or related tests.
- Avoid introducing new windows, new product-path plugin proof modules, or broad UI surface area for this story.
- If implementation starts to require generalized transport orchestration or persisted-data redesign, re-scope the work. Those concerns belong outside this story's bounded audible-proof goal.

### References

- `_bmad-output/planning-artifacts/epics.md` - Epic 4 and Story 4.2 acceptance criteria.
- `_bmad-output/planning-artifacts/prd.md` - route-to-first-audible expectations, score-first playback goals, and realtime trust constraints.
- `_bmad-output/planning-artifacts/architecture.md` - authoritative playback ownership, rack-first model, and audible-playback boundaries.
- `_bmad-output/planning-artifacts/ux-design-specification.md` - score-first playback UX, direct routing expectations, and calm workflow boundaries.
- `docs/project-context.md` - repo-wide rules for playback ownership, slot-owned sessions, and explicit readiness semantics.
- `docs/architecture.md` - current audio-pipeline documentation and the canonical `prepare_supported_playback(...)` seam.
- `docs/development-guide.md` - developer guidance for command registration, validation commands, and playback prep notes.
- `/memories/repo/epic4-playback-preparation-story-4-1.md` - Story 4.1 reuse guardrails.
- `/memories/repo/rack-playback-proof-seam.md` - bounded proof-seam constraints.
- `/memories/repo/playback-rack-first-model.md` - rack-first ownership boundaries.
- `/memories/repo/slot-owned-live-runtime-sessions-story-1-4.md` - slot-owned runtime-session lifecycle constraints.
- `/memories/repo/same-session-playback-activation-story-3-8.md` - activation-runtime refresh boundaries and backend-owned route truth.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Create-story analysis covered Epic 4.2 acceptance criteria, Story 4.1 implementation learnings, slot-owned runtime-session prior art, keyboard-shortcut audition stubs, and current transport-preparation seams.
- Recent commit context reviewed:
  - `a98b6cc4 feat(4.1): playback prep`
  - `3bd76024 feat(3.4): slot consistency`
  - `ea6ea2fe feat(3.3): slot validity`
  - `3c724393 feat(3.2): slot sanity`

### Completion Notes List

- Added `src-tauri/src/playback/audition.rs` as the backend-owned audition seam, reusing `prepare_supported_playback(...)` and enforcing the bounded note contract of one pitch, 500ms duration, velocity 96, and legato articulation.

- Story 4.3's continuous render correction and the follow-up audible regression fix restored dependable routed playback on the shared engine path, so this story is no longer blocked.
- Extended the slot-owned macOS runtime-session path in `src-tauri/src/playback/plugin_editor_poc.rs` to render a one-note VST preview through the existing live slot session, then stream the rendered audio through the mixer.
- Wired score-context audition triggers through `src/services/actions.ts`, `src/windows/main/hooks/use-keyboard-shortcuts.ts`, `src/windows/main/components/sequencer-track.tsx`, and `src/windows/main/screens/write/components/left-panel.tsx` while preserving the existing audition preference and backend route-readiness gate.
- Added and updated frontend and backend regression coverage for actions, keyboard shortcuts, sequencer dragging, and left-panel accidental edits.
- Validation summary: `npm run test:js`, `npm run lint`, `npm run check-types`, `cargo test --manifest-path src-tauri/Cargo.toml playback::audition`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` passed.
- Investigated the remaining Rust failures and resolved them by aligning stale route-resolution tests in `src-tauri/src/score/instruments/mod.rs` and the performance binary's lightweight runtime-activation shim in `src-tauri/src/bin/performance.rs` with the canonical semantics: missing slot references normalize to `Unrouted`, while existing empty slots remain explicit `NeedsAttention`.
- Addressed code-review follow-ups by tightening frontend audition gating to the backend `single_instrument_transport_ready` signal, centralizing caught preview failures in the shared audition helper, and adding regression coverage for readiness flips and duplicate drag suppression.
- Reopened the story as blocked after post-review triage concluded that real audible playback is still not dependable through the current burst-render path, and that Story 4.3 must now correct the underlying engine shape before this story can be considered complete.

### File List

- `_bmad-output/implementation-artifacts/4-2-prove-first-audible-playback-through-auditioning.md`
- `docs/architecture.md`
- `docs/development-guide.md`
- `src-tauri/src/bin/performance.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/playback/audition.rs`
- `src-tauri/src/playback/mixer.rs`
- `src-tauri/src/playback/mod.rs`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src/services/actions.test.ts`
- `src/services/actions.ts`
- `src/windows/main/components/sequencer-track.test.tsx`
- `src/windows/main/components/sequencer-track.tsx`
- `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`
- `src/windows/main/screens/write/components/left-panel.test.tsx`
- `src/windows/main/screens/write/components/left-panel.tsx`
- `src/windows/main/utils/audition.test.ts`
- `src/windows/main/utils/audition.ts`

## Change Log

- 2026-03-27: Created the Story 4.2 implementation guide and moved Story 4.2 sprint tracking to ready-for-dev.
- 2026-03-28: Implemented the backend-owned first-audible audition path, wired score-context triggers, added regression coverage, and moved the story to review with an explicit note about unrelated full Rust suite failures in `src-tauri/src/score/instruments/mod.rs`.
- 2026-04-03: Investigated and fixed the remaining Rust test failures by updating stale route-resolution expectations and aligning the performance test shim with the canonical empty-slot versus missing-slot readiness semantics.
- 2026-04-03: Resolved code-review findings by aligning frontend audition gating with backend transport readiness, catching preview-command failures through the shared helper, and adding readiness-flip plus drag-dedup regression coverage.
- 2026-04-03: Reopened the story as blocked after triage determined that the shipped burst-render audition path is not yet a reliable working audible proof and must be corrected through Story 4.3's continuous render-engine work.
