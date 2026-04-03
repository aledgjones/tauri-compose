# Story 4.3: Establish A Continuous Render Path For Slot-Owned Sessions

Status: done

## Story

As a composer,
I want mounted routed plugin sessions to stay connected to a continuous playback render path,
so that auditioning, plugin-UI interaction, and later transport work all use the same trustworthy audio foundation.

## Acceptance Criteria

1. Given a routed slot-owned plugin session exists and is render-active, when Compose's audio engine is running, then the session is processed through a continuous backend-owned render path rather than only through burst audition rendering, and silence-safe buffers continue to flow even when no notes are currently scheduled.
2. Given the user opens a native plugin editor for a mounted routed session, when the plugin exposes its own playable UI such as a built-in keyboard, then that interaction can sound through the same continuous render path, and the editor window remains a transient view over the already-running slot-owned session rather than a second playback owner.
3. Given transport trust work is still upcoming, when this render path is introduced, then transport remains responsible for scheduling and transport-state transitions rather than audio-stream ownership, and the OS output stream, mixer lifetime, and live plugin processing stay outside transport.
4. Given Compose must stay realtime-safe and rack-first, when the correction story is implemented, then the audio path consumes precomputed lock-safe state and backend-owned event queues, and the implementation does not reintroduce sampler-style fallbacks, frontend-owned playback truth, or plugin-only side seams.

## Tasks / Subtasks

- [x] Task 1: Introduce a backend-owned continuous render engine seam for live slot sessions (AC: 1, 3, 4)
  - [x] Add a focused playback-engine seam under `src-tauri/src/playback/` that owns per-block plugin processing for render-active slot-owned sessions.
  - [x] Keep OS output-stream and mixer lifetime in `src-tauri/src/playback/audio.rs`; do not move stream creation into transport.
  - [x] Keep the audio path lock-safe by consuming precomputed slot render state, fixed-size block work, and backend-owned event queues rather than ad hoc UI-thread callbacks.
  - [x] Bound initial scope to the current single-target-first architecture; do not expand into multi-target or multi-timbral routing.
- [x] Task 2: Make live slot sessions render-active without turning the editor seam into the playback owner (AC: 1, 2, 4)
  - [x] Reuse the slot-owned `PluginHostSession` lifecycle from `src-tauri/src/playback/plugin_editor_poc.rs` instead of creating a second runtime host object.
  - [x] Define render-active ownership in backend terms such as editor-open, active audition, active transport playback, or safe tail-release handling.
  - [x] Ensure mounted sessions can be processed continuously even when the current block resolves to silence.
  - [x] Preserve the rule that opening the native editor attaches to an already-running session instead of creating the canonical playback instance.
- [x] Task 3: Route audition and plugin-UI interaction through the new render path (AC: 1, 2, 3)
  - [x] Refactor the current bounded audition path so it injects note events into the continuous engine rather than offline-rendering a fixed buffer as the long-term owner.
  - [x] Prove that a mounted plugin's own playable UI can sound through the same session-owned path once the render loop is active.
  - [x] Keep score-context audition behavior intact while moving its concrete audio production onto the shared engine.
  - [x] Avoid promoting plugin-only proof seams into product ownership.
- [x] Task 4: Keep transport scoped to scheduling and state transitions (AC: 3, 4)
  - [x] Preserve `src-tauri/src/playback/transport.rs` as the scheduling and transport-state seam rather than the owner of output stream creation or live processor execution.
  - [x] Have transport submit scheduled events or state changes into the engine instead of directly owning plugin processing lifecycle.
  - [x] Keep pause, stop, and seek semantics compatible with the continuous render path without broadening this story into full transport-trust implementation.
  - [x] Document any explicit engine-versus-transport ownership rules that become sharper during the refactor.
- [x] Task 5: Add seam-local regression coverage and planning notes for later stories (AC: 1, 2, 3, 4)
  - [x] Add backend tests that prove render-active slot sessions stay connected to the continuous engine and that missing-slot versus empty-slot readiness behavior remains unchanged.
  - [x] Add focused tests for engine ownership boundaries so transport, audition, and editor-host seams cannot silently retake playback ownership later.
  - [x] Update docs and story artifacts so later mix-control and transport-trust stories build on the corrected engine model rather than the burst-render proof path.

### Review Findings

- [x] [Review][Patch] Audio callback still allocates per block instead of using precomputed fixed-size render work [src-tauri/src/playback/plugin_editor_poc.rs:1993]
- [x] [Review][Patch] Transport natural completion leaves the render target active [src-tauri/src/playback/transport.rs:421]
- [x] [Review][Patch] Due engine events are dropped when slot rendering returns `Busy` or transiently `MissingSession` [src-tauri/src/playback/engine.rs:87]
- [x] [Review][Patch] Story 4.3 changes Story 4.2's 1-second audition contract to 500ms instead of preserving the unblock behavior [src-tauri/src/playback/audition.rs:12] — accepted intentionally at 500ms and synced across Story 4.2 plus playback docs.
- [x] [Review][Patch] All discovered plugin output buses are summed into the main output without bus-role filtering [src-tauri/src/playback/plugin_editor_poc.rs:2055]

## Dev Notes

### Story Intent

- This is a correction story, not a restart. The repo already has the persistent output stream, slot-owned runtime sessions, and backend-owned readiness model, but the Story 4.2 burst-render audition path is not yet a dependable working audible proof in practice.
- The missing concrete piece is a continuous realtime-safe render engine between the persistent output stream and the slot-owned plugin sessions.
- The goal is to make the implemented path match the already-approved architecture rule that playback is split into a Rust control plane and a realtime-safe playback plane, and to restore a genuinely working score-context audible path on top of that engine.

### Why This Story Exists Now

- Story 4.2 established the intended score-context audition contract and the bounded burst-render seam, but maintainers cannot currently rely on that implementation as a trustworthy working audible path.
- The current burst-render approach is therefore not just incomplete architecture; it is also the active blocker preventing Story 4.2 from being accepted as truly done.
- Future transport, mix, and plugin-UI interactions should not each invent their own audio-production mechanism.
- Inserting this story now prevents Epic 5 transport-trust work from being built on top of an architecturally weaker burst-render model.

### This Story Owns

- Defining the concrete continuous render seam for render-active slot-owned sessions.
- Keeping stream ownership outside transport and preserving the slot-owned runtime-session model.
- Moving audition onto the shared engine path rather than leaving burst rendering as the long-term product owner, and restoring the currently non-working 4.2 audible path on that corrected foundation.
- Making plugin-UI interaction audibly meaningful through the same session-owned engine path.

### This Story Does Not Own

- Full transport play, pause, resume, or seek trust behavior.
- Final mixer-control UX or broader route-first mix semantics.
- Multi-target, multi-slot, or multi-timbral playback expansion.
- Expression-map realism or other later playback-shaping behavior.
- A DAW-style always-process-everything host model with no notion of render-active scope.

### Architecture Guardrails

- Rust remains the only owner of plugin inventory truth, rack state, slot state, route state, playback preparation, transport state, and playback-engine truth.
- The architecture must stay rack-first: racks own slot lifecycle, slots own plugin assignment or unassignment, and instruments only own routes to existing slots.
- The editor host seam remains a transient UI attachment over the slot-owned session. It must not become the owner of canonical playback runtime state.
- The audio-thread path must remain lock-safe and consume precomputed state rather than taking UI-thread locks or waiting on window-host callbacks.
- Keep legacy sampler or SCVI ownership out of the active product path.

### Current Relevant Seams

- `src-tauri/src/playback/audio.rs`
  - Owns the persistent OS output stream and current mixer handle. This should remain outside transport.
- `src-tauri/src/playback/mixer.rs`
  - Owns the backend mixer that the audio callback drains. The continuous engine should feed this path rather than replace it with a second output model.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Owns slot-keyed `PluginHostSession` lifecycle and native editor attachment. Reuse its slot-owned runtime sessions rather than creating a parallel live host path.
- `src-tauri/src/playback/transport.rs`
  - Should remain the scheduling and transport-state seam. It can submit events into the engine, but it should not take ownership of output stream creation or live processor execution.
- `src-tauri/src/playback/audition.rs`
  - Current bounded audible proof seam. Refactor its long-term audio production path onto the engine while preserving the score-context contract.
- `src-tauri/src/lib.rs`
  - Refreshes playback activation from expected slot targets and the live slot-session registry; engine ownership should stay backend-owned here rather than spilling into React.

### Implementation Guidance

- Introduce a small backend playback-engine seam rather than further expanding `transport.rs` or burying continuous render ownership inside `plugin_editor_poc.rs`.
- The engine should process render-active slot sessions continuously, not necessarily every slot in the rack forever. Keep CPU scope bounded and explicit.
- A render-active session should still accept blocks that resolve to silence. Continuous processing means the path stays alive even when no notes are currently scheduled.
- Transport should become a producer of scheduled note or state events into the engine, not the owner of the audio stream.
- Audition should become a producer of immediate note events into the same engine, not a separate long-term playback owner.
- Mounted plugin UI interactions should be able to sound through the same active session path once the engine is alive.

### Risks And Regression Traps

- Letting the native editor host seam become the hidden owner of playback would recreate split ownership under a new name.
- Driving live plugin processing from the UI thread or main-thread window lifecycle would violate the realtime-safe playback-plane rule.
- Leaving burst audition as a parallel long-term audio-production path would keep future transport and UI work architecturally fragmented.
- Processing every slot forever with no render-active boundary would burn CPU needlessly and blur ownership decisions.
- Moving stream creation or device lifetime into transport would regress the current separation that already exists in `audio.rs`.

### Testing Requirements

- Add backend tests that prove:
  - render-active slot sessions stay connected to a continuous backend-owned engine path;
  - existing empty-slot versus missing-slot readiness semantics remain explicit and unchanged;
  - transport can remain stopped while the continuous engine still keeps render-active sessions processable;
  - audition and mounted plugin-UI interactions can target the same engine-owned session path.
- Preserve or extend existing targeted validation for:
  - `npm run lint`
  - `npm run check-types`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`

### Documentation Impact

- Update docs in the same change if the playback-engine seam becomes the concrete implementation of the approved control-plane versus realtime-playback-plane split.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`

### References

- `_bmad-output/planning-artifacts/epics.md` - Epic 4 correction-story placement and the later transport stories this work unblocks.
- `_bmad-output/planning-artifacts/architecture.md` - the approved split between a Rust control plane and a realtime-safe playback plane.
- `docs/project-context.md` - current guardrails for slot-owned runtime sessions, transport-safe playback behavior, and backend-owned truth.
- `/memories/repo/playback-rack-first-model.md` - rack-first ownership and empty-slot readiness semantics.
- `/memories/repo/slot-owned-live-runtime-sessions-story-1-4.md` - slot-owned session lifecycle and playback readiness derivation.
- `/memories/repo/plugin-editor-host-session-lifecycle.md` - native-editor lifecycle constraints so the editor seam stays a transient view over the slot-owned session.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story created as a lightweight correction slice after Story 4.2 review exposed that the approved architecture already assumes a realtime-safe playback plane, but the current next-story plan did not yet make the continuous render-engine seam concrete.
- 2026-04-03: Added `src-tauri/src/playback/engine.rs` as the backend-owned continuous render seam and rewired `audio.rs` so the persistent output callback processes render-active slot sessions before mixing any remaining preview voices.
- 2026-04-03: Reworked `plugin_editor_poc.rs` so slot-owned `PluginHostSession`s expose per-block realtime processing and editor-open toggles engine render activity instead of creating a second playback owner.
- 2026-04-03: Replaced the burst audition owner path with engine note-queue submission in `audition.rs`, and kept `transport.rs` scoped to state transitions by publishing engine render-active state without taking over stream lifetime.
- 2026-04-03: Follow-up hotfix after manual audible check stayed silent: activated VST audio/event buses on the live slot-session path before initial and reconfigured realtime processing so audition and plugin-playable UI interactions use a fully activated runtime.
- 2026-04-03: Added env-gated runtime diagnostics under `SOLOC_PLAYBACK_ENGINE_DEBUG=1` and `SOLOC_PLUGIN_EDITOR_POC_VERBOSE=1` so silent audition and plugin-keyboard repros expose engine event flow, slot render outcomes, host processing setup, and observed output peaks.
- 2026-04-03: Follow-up live-host fix after diagnostics showed BBCSO received note events but still rendered silence: cached bus activation once per slot session, captured the plugin's real audio-output bus topology, and passed a full `AudioBusBuffers[]` array into realtime `process(...)` instead of assuming a single output bus.
- 2026-04-03: Tightened the debug stream after runtime repros proved too noisy to capture: live-host logging now emits on processing reconfiguration, event-bearing blocks, and audible output instead of every silent idle block.
- 2026-04-03: Fixed score-context audition for multi-route scores by decoupling audition from the transport single-instrument trust gate; audition now targets the explicitly requested ready instrument route directly while transport start remains gated separately.
- 2026-04-03: Tuned supported score-context audition duration from 1000ms to 500ms after manual validation showed the default note length felt too long in practice.
- 2026-04-03: Follow-up audition-release fix after manual repeated-note checks exposed stuck BBCSO voices: audition note-on/note-off pairs now carry explicit per-slot note IDs through the engine and live VST host so note-off targets the correct active voice instead of using `noteId = -1`.
- 2026-04-03: Follow-up sustained-sample audition fix after violins still discharged the previous note into the next preview: a fresh audition on an audition-only slot now clears queued/tail state and restarts that slot runtime before queuing the new note, so leftover plugin state cannot bleed across repeated previews.
- 2026-04-03: Follow-up lifecycle correction after repeated sustained-sample checks showed the prior fix was still too shallow: audition release rendering now keeps the slot alive for a real time-based tail window instead of only 8 callback blocks, and superseding audition-only previews now perform a deeper slot reset via component active-state cycling rather than just `setProcessing(0)`.
- 2026-04-03: Corrected the audition overlap model after manual quick-repeat checks showed that hard-resetting a superseded preview made sustained instruments sound choppy: quick auditions now stay on the same live slot session and append note events so overlapping preview voices can tail naturally, while the longer release-tail window remains in place.
- 2026-04-03: Follow-up regression fix after the review patch muted playback entirely: the live host now exposes all declared VST output buses to processing again and selects the strongest audible bus for stereo mixdown instead of assuming bus 0 is always the canonical audible path.

### Completion Notes List

- Inserted this story as the next Epic 4 story so the playback engine becomes concrete before mix-control and transport-trust work proceed.
- Preserved the earlier architectural decisions: persistent output stream stays outside transport, slot-owned sessions remain canonical, and transport stays scoped to scheduling and state transitions.
- Implemented a lock-safe continuous render path for live slot sessions, with engine-owned render-active reasons for editor-open, audition, transport handoff, and bounded tail release.
- Added seam-local engine tests for queued audition events, editor-driven render activity without transport ownership, transport target switching, and live-slot sync removal; existing audition readiness tests continue to cover missing-slot versus empty-slot semantics.
- Updated architecture and development docs so future playback stories build on the continuous engine model instead of the earlier burst-render proof path.
- Added a post-review-prep hotfix in `plugin_editor_poc.rs` to activate VST audio and event buses before realtime processing; this addresses the concrete symptom where editor-open and queued audition events stayed silent despite the continuous engine being active.
- Added gated backend diagnostics so the remaining silent-runtime failure can be narrowed with concrete engine and host evidence instead of further architectural guessing.
- Updated the live host to cache realtime bus activation and downmix from all declared plugin output buses, closing the mismatch where multi-output instruments were processed through a single-bus host buffer contract.
- Reduced playback debug noise so manual repro capture stays usable while preserving the lines needed to trace event delivery and audible output.
- Updated audition gating on both backend and frontend so a ready instrument route can audition even when multiple routed instruments are live in the score.
- Shortened the supported audition note length so manual note preview feels snappier while preserving the same queueing and release behavior.
- Strengthened audition note lifecycle handling so repeated previews release cleanly instead of leaking the prior voice into the next audition.
- Restarted audition-only slot runtimes when a new preview supersedes an older unowned audition, addressing sustained sample carryover that explicit note IDs alone did not fully eliminate.
- Corrected the audition lifecycle so realtime preview remains on the existing audio callback, but uses a longer bounded release tail and a deeper plugin-state reset when an older audition is explicitly superseded.
- Updated the audition behavior to match the product need for overlapping preview voices on one live slot instead of restarting the slot when notes are triggered quickly.
- Removed the temporary env-gated playback diagnostics from the engine and live host once the multi-output host fix and audition lifecycle fixes were validated, so normal development builds no longer carry the ad hoc debug controls.
- Fixed the post-review no-audio regression by restoring full output-bus exposure during VST processing while keeping mixdown bounded to one audible bus instead of summing every declared bus.

- Validations run:
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
  - `npm run check-types`
  - `npm run lint`
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::engine::tests::queued_audition_runs_through_note_off_then_tail_release`
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::engine::tests::editor_activity_keeps_slot_rendering_without_transport`
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::transport::tests::transport_play_with_states_starts_transport_only_playback_from_supported_command_seam`
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::engine::tests::queued_audition_runs_through_note_off_then_tail_release` (post-diagnostics patch)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` (post-diagnostics patch)
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc` (post-bus-topology fix)
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::engine::tests::queued_audition_runs_through_note_off_then_tail_release` (post-bus-topology fix)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` (post-bus-topology fix)
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc` (post-log-noise reduction)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` (post-log-noise reduction)
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::audition`
  - `npm run test:js -- src/windows/main/utils/audition.test.ts`
  - `npm run check-types`
  - `npm run lint`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::engine::tests::queued_audition_runs_through_note_off_then_tail_release` (post-note-id fix)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` (post-note-id fix)
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::engine::tests::queue` (post-audition-restart fix)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` (post-audition-restart fix)
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::engine::tests::queue` (post-release-tail + deep-reset fix)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` (post-release-tail + deep-reset fix)
  - `cargo test --manifest-path src-tauri/Cargo.toml playback::engine::tests::queue` (post-overlap correction)
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` (post-overlap correction)

### File List

- `_bmad-output/implementation-artifacts/4-3-establish-a-continuous-render-path-for-slot-owned-sessions.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `src-tauri/src/lib.rs`
- `src-tauri/src/playback/audio.rs`
- `src-tauri/src/playback/audition.rs`
- `src-tauri/src/playback/engine.rs`
- `src-tauri/src/playback/mixer.rs`
- `src-tauri/src/playback/mod.rs`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/transport.rs`

## Change Log

- 2026-04-03: Created the next Epic 4 correction story for a continuous slot-session render path and updated story sequencing so later playback work builds on the correct engine shape.
- 2026-04-03: Updated the story framing to explicitly own correction of the currently non-working Story 4.2 audible path rather than assuming the burst-render proof already held in practice.
- 2026-04-03: Implemented the continuous render engine, moved score-context audition onto engine note queues, kept transport scoped to engine state handoff, updated playback docs, and marked the story ready for review.
- 2026-04-03: Activated VST audio/event buses for live slot-session processing after manual silence feedback showed the continuous engine was running against an incompletely activated plugin runtime.
- 2026-04-03: Added engine and live-host debug logging to capture queued audition events, slot render outcomes, realtime processing setup, and output-peak observations during silent-runtime repros.
- 2026-04-03: Fixed the live host's output-bus contract for multi-output instruments by caching output-bus layouts and presenting the full bus array to VST realtime processing instead of a hard-coded single output bus.
- 2026-04-03: Reduced debug-log chatter in the live host so only reconfiguration, event-bearing processing, and audible-output lines remain during runtime repro capture.
- 2026-04-03: Removed the obsolete transport-ready audition gate so ready instrument routes can audition in multi-route sessions, matching the continuous live-render architecture.
- 2026-04-03: Reduced the default supported audition note duration to 500ms after manual runtime feedback.
- 2026-04-03: Added explicit note IDs to live audition note events so VST note-off reliably releases the same audition voice on plugins that do not behave cleanly with `noteId = -1`.
- 2026-04-03: Restarted audition-only slot runtimes before a superseding preview note so sustained plugin state from the previous audition cannot spill into the next one.
- 2026-04-03: Replaced the fixed 8-block audition tail with a real time-based release window and deepened the superseding audition reset path to cycle component active state, addressing sustained-sample carryover and clipped release behavior more directly.
- 2026-04-03: Removed the superseding audition hard-reset path after manual validation showed it broke the required overlapping-preview behavior; quick auditions now append onto the live slot session so prior tails are preserved instead of chopped.
- 2026-04-03: Removed the temporary `SOLOC_PLAYBACK_ENGINE_DEBUG` and `SOLOC_PLUGIN_EDITOR_POC_VERBOSE` diagnostics after the live-host bus fix and audition lifecycle fixes were verified, leaving only ordinary error reporting in the playback path.
- 2026-04-03: Fixed the no-audio regression from the review follow-up by dropping the brittle bus-0-only filter and selecting the strongest audible processed output bus for mixdown instead.
