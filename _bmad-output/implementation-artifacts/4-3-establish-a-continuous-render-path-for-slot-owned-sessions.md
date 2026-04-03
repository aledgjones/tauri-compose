# Story 4.3: Establish A Continuous Render Path For Slot-Owned Sessions

Status: ready-for-dev

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

- [ ] Task 1: Introduce a backend-owned continuous render engine seam for live slot sessions (AC: 1, 3, 4)
  - [ ] Add a focused playback-engine seam under `src-tauri/src/playback/` that owns per-block plugin processing for render-active slot-owned sessions.
  - [ ] Keep OS output-stream and mixer lifetime in `src-tauri/src/playback/audio.rs`; do not move stream creation into transport.
  - [ ] Keep the audio path lock-safe by consuming precomputed slot render state, fixed-size block work, and backend-owned event queues rather than ad hoc UI-thread callbacks.
  - [ ] Bound initial scope to the current single-target-first architecture; do not expand into multi-target or multi-timbral routing.
- [ ] Task 2: Make live slot sessions render-active without turning the editor seam into the playback owner (AC: 1, 2, 4)
  - [ ] Reuse the slot-owned `PluginHostSession` lifecycle from `src-tauri/src/playback/plugin_editor_poc.rs` instead of creating a second runtime host object.
  - [ ] Define render-active ownership in backend terms such as editor-open, active audition, active transport playback, or safe tail-release handling.
  - [ ] Ensure mounted sessions can be processed continuously even when the current block resolves to silence.
  - [ ] Preserve the rule that opening the native editor attaches to an already-running session instead of creating the canonical playback instance.
- [ ] Task 3: Route audition and plugin-UI interaction through the new render path (AC: 1, 2, 3)
  - [ ] Refactor the current bounded audition path so it injects note events into the continuous engine rather than offline-rendering a fixed buffer as the long-term owner.
  - [ ] Prove that a mounted plugin's own playable UI can sound through the same session-owned path once the render loop is active.
  - [ ] Keep score-context audition behavior intact while moving its concrete audio production onto the shared engine.
  - [ ] Avoid promoting plugin-only proof seams into product ownership.
- [ ] Task 4: Keep transport scoped to scheduling and state transitions (AC: 3, 4)
  - [ ] Preserve `src-tauri/src/playback/transport.rs` as the scheduling and transport-state seam rather than the owner of output stream creation or live processor execution.
  - [ ] Have transport submit scheduled events or state changes into the engine instead of directly owning plugin processing lifecycle.
  - [ ] Keep pause, stop, and seek semantics compatible with the continuous render path without broadening this story into full transport-trust implementation.
  - [ ] Document any explicit engine-versus-transport ownership rules that become sharper during the refactor.
- [ ] Task 5: Add seam-local regression coverage and planning notes for later stories (AC: 1, 2, 3, 4)
  - [ ] Add backend tests that prove render-active slot sessions stay connected to the continuous engine and that missing-slot versus empty-slot readiness behavior remains unchanged.
  - [ ] Add focused tests for engine ownership boundaries so transport, audition, and editor-host seams cannot silently retake playback ownership later.
  - [ ] Update docs and story artifacts so later mix-control and transport-trust stories build on the corrected engine model rather than the burst-render proof path.

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

### Completion Notes List

- Inserted this story as the next Epic 4 story so the playback engine becomes concrete before mix-control and transport-trust work proceed.
- Preserved the earlier architectural decisions: persistent output stream stays outside transport, slot-owned sessions remain canonical, and transport stays scoped to scheduling and state transitions.

### File List

- `_bmad-output/implementation-artifacts/4-3-establish-a-continuous-render-path-for-slot-owned-sessions.md`

## Change Log

- 2026-04-03: Created the next Epic 4 correction story for a continuous slot-session render path and updated story sequencing so later playback work builds on the correct engine shape.
- 2026-04-03: Updated the story framing to explicitly own correction of the currently non-working Story 4.2 audible path rather than assuming the burst-render proof already held in practice.
