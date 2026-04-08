# Story 1.1: Remove SCVI From the Active Playback Runtime

Status: done

## Story

As a composer,
I want Compose to use only the intended VST-first playback runtime,
so that playback behavior is no longer split across legacy and current execution paths.

## Acceptance Criteria

1. Given the app contains SCVI-specific runtime playback entry points or orchestration branches, when playback is initialized or requested through the supported product playback paths, then those paths execute through the VST-first runtime only and no SCVI-owned runtime branch remains reachable from supported playback entry points.
2. Given SCVI-specific runtime modules or orchestration branches are still wired into executable playback behavior, when this story is complete, then those runtime connections are removed or disconnected from active product behavior and the remaining playback ownership is unambiguously VST-first in code structure.
3. Given this story removes the active SCVI runtime path, when the updated codebase is built, then the app still builds successfully and the story does not introduce migration, compatibility-preservation, or legacy fallback scaffolding.

## Tasks / Subtasks

- [x] Task 1: Disconnect SCVI from supported runtime playback entry points (AC: 1, 2)
  - [x] Remove the sampler-backed SCVI dependency from the supported transport start path in `src-tauri/src/playback/transport.rs`.
  - [x] Ensure supported playback requests no longer clone or consume SCVI sampler cache state as runtime truth.
  - [x] Replace any remaining supported SCVI runtime execution path with the intended VST-first seam or with a bounded compile-safe non-SCVI outcome if the VST path is not yet complete.
- [x] Task 2: Remove active SCVI runtime wiring from backend state and command seams without broad cleanup spillover (AC: 1, 2, 3)
  - [x] Update backend setup in `src-tauri/src/lib.rs` so supported playback runtime ownership no longer depends on `SamplerContext` as an active product runtime.
  - [x] Update Tauri command registration and automation/backend command seams so supported playback commands no longer route through SCVI runtime behavior.
  - [x] Keep build-only or tool-only SCVI code out of active runtime paths without deleting Story 1.3 tooling scope prematurely.
- [x] Task 3: Keep frontend and backend seams compile-safe while preserving Story 1.1 scope boundaries (AC: 2, 3)
  - [x] Update `src/services/actions.ts` and any directly coupled callers only as needed so supported runtime commands remain valid after backend changes.
  - [x] Do not perform user-facing label or workflow cleanup here beyond what is strictly necessary to keep the app coherent; Story 1.2 owns UI language and workflow cleanup.
  - [x] Do not add SCVI fallback logic, migration scaffolding, or compatibility shims.
- [x] Task 4: Add focused validation close to the owning runtime seam (AC: 3)
  - [x] Add or update Rust tests around transport/runtime command behavior so supported playback paths cannot silently continue using SCVI-owned runtime state.
  - [x] Run the relevant build and test checks needed to prove the runtime cleanup keeps the project buildable.
- [x] Task 5: Capture documentation impact for developer-facing runtime guidance (AC: 2, 3)
  - [x] Update developer-facing docs only if runtime ownership or backend seam descriptions become inaccurate after the code change.
  - [x] Leave broad SCVI documentation retirement and user-facing guidance cleanup to Story 1.4.

## Dev Notes

### Story Intent

- This story is runtime cleanup, not broad product cleanup.
- The required outcome is that supported playback entry points no longer execute SCVI runtime behavior.
- If the full VST runtime path is not ready yet, use a bounded non-SCVI outcome that preserves buildability and makes the runtime seam unambiguous.
- Do not keep SCVI alive as a hidden fallback just to preserve temporary playback behavior.

### Scope Boundaries

- In scope:
  - Active runtime playback entry points.
  - Backend state ownership for supported playback runtime behavior.
  - Minimal frontend command alignment required to keep the app coherent and buildable.
- Out of scope:
  - User-facing SCVI labels, controls, settings, or workflow copy. Story 1.2 owns that.
  - SCVI build scripts, inspection commands, and repo tooling cleanup. Story 1.3 owns that.
  - Broad documentation cleanup. Story 1.4 owns that.
  - Repo-wide SCVI verification sweep. Story 1.5 owns that.
  - Persistence migration or compatibility-preservation work. The specs explicitly forbid adding it here.

### Architecture Compliance

- Rust remains the sole source of truth for playback preparation state, transport state, and plugin/runtime truth.
- Playback runtime logic belongs in `src-tauri/src/playback/`.
- Tauri command boundaries belong in backend command seams with matching frontend callers in `src/services/actions.ts`.
- Frontend code must not invent a second playback truth while this runtime cleanup is happening.
- The playback runtime remains split between a Rust control plane and a realtime-safe playback plane; do not add UI-driven runtime ownership or lock-heavy work to the audio path.
- Keep regression coverage close to the owning playback seam rather than creating a new cross-cutting harness.

### Current Active Runtime Seam

- `src-tauri/src/lib.rs`
  - App setup currently manages `SamplerContext::new()` as active application state.
- `src-tauri/src/playback/transport.rs`
  - `transport_play(...)` currently accepts `sampler_state` and clones sampler cache into `PlaybackSnapshot` before starting transport playback.
  - `render_note(...)` currently renders from sampler cache data and builds streamer requests from cached SCVI sample metadata.
- `src-tauri/src/playback/sampler.rs`
  - `scan_plugins()` scans `/Library/Audio/Plug-Ins/SCVI` for `.scvi` files.
  - `load_instrument()` opens `.scvi` containers through `ScviContainer::open()` and builds runtime cache structures.
  - `sampler_audition(...)` previews notes by streaming cached SCVI sample data.
- `src-tauri/src/playback/streamer.rs`
  - Current disk streaming implementation is explicitly documented as reading `.scvi` container content.
- `src/services/actions.ts`
  - Frontend still exposes sampler-oriented command calls such as `rescanPlugins`, `syncWithInstruments`, and `audition`.

### Implementation Guidance

- Preferred outcome: move supported playback entry points onto the intended VST-first seam already emerging on the `feat/vst` branch.
- Acceptable interim outcome for this story: a bounded non-SCVI runtime path that compiles cleanly and makes it impossible for supported playback requests to execute SCVI-owned runtime behavior.
- Do not delete every SCVI-related file just because it exists. Story 1.1 is about disconnecting active runtime behavior first.
- It is acceptable to leave SCVI tooling modules present but inactive in supported runtime behavior until Story 1.3 removes tooling residue.
- Avoid widening the change into UI redesign, plugin inventory UX, or persistence semantics.

### File Structure Requirements

- Backend runtime changes should stay centered in:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/playback/transport.rs`
  - `src-tauri/src/playback/sampler.rs`
  - `src-tauri/src/playback/mod.rs`
  - Any directly related backend command-registration or automation files that keep those commands wired
- Frontend fallout should stay minimal and localized to:
  - `src/services/actions.ts`
  - Only the directly affected callers that must compile after backend command changes

### Testing Requirements

- Minimum validation target:
  - Rust tests covering the updated transport/runtime command behavior.
  - Build validation proving the app still compiles after SCVI runtime disconnection.
- Prefer existing project checks:
  - `npm run check-types`
  - `npm run test:rust` or targeted Rust tests if a narrower command is sufficient
  - `cargo clippy` if backend edits introduce new lint exposure
- Do not create a new repo-wide playback test framework for this cleanup.

### Library / Framework Requirements

- Preserve the existing brownfield Tauri + React + TypeScript + Vite foundation.
- Current repo versions relevant to this seam:
  - Tauri frontend API `2.10.1`
  - Tauri Rust crate `2.10.3`
  - `vst3 = 0.3.0`
  - `cpal = 0.17.3`
- Story 1.1 should extend the current runtime seam instead of re-platforming or introducing parallel architecture.

### Documentation Impact

- If the story changes developer-facing runtime truth, update only the narrow developer docs that become inaccurate, such as `docs/project-context.md` or `docs/development-guide.md`.
- Do not perform the broad SCVI reference cleanup here; Story 1.4 owns that full documentation pass.

### Git Intelligence Summary

- Recent branch history indicates the repo is already moving toward VST work:
  - `chore: spec vst work`
  - `feat: vst poc`
- Align Story 1.1 with that direction by removing SCVI from active runtime ownership without reintroducing a second transitional architecture.

### Project Structure Notes

- Existing project context still describes `src-tauri/src/playback/` as containing audio, mixer, sampler, transport, SCVI format, and streamer seams. This story should narrow active runtime ownership without forcing unrelated playback subdomains to move.
- SCVI remains a known concept in the repo today, but Story 1.1 should treat it as legacy runtime behavior to disconnect, not as a fallback to preserve.

### References

- `_bmad-output/planning-artifacts/epics.md` — Epic 1, Story 1.1 acceptance criteria
- `_bmad-output/planning-artifacts/prd.md` — product direction: SCVI replaced by VST3, no migration/backward-compatibility requirement
- `_bmad-output/planning-artifacts/architecture.md` — playback runtime ownership, Rust source-of-truth rules, file-placement rules
- `docs/project-context.md` — current repo structure, playback/audio gotchas, validation rules
- `src-tauri/src/lib.rs` — current backend setup includes `SamplerContext::new()`
- `src-tauri/src/playback/transport.rs` — current transport start path clones sampler cache and renders from sampler-backed data
- `src-tauri/src/playback/sampler.rs` — current SCVI scan/load/audition runtime seam
- `src-tauri/src/playback/streamer.rs` — current streamer still documents `.scvi` disk streaming runtime
- `src/services/actions.ts` — current frontend sampler command surface
- `package.json` and `src-tauri/Cargo.toml` — current scripts and dependency versions relevant to the seam

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Auto-selected from sprint status as the first backlog story.
- No previous story file exists for Epic 1, so there are no prior implementation learnings to import.
- Replaced the supported transport runtime with an explicit transport-only execution mode so `transport_play` no longer snapshots SCVI sampler cache state or dispatches SCVI streaming.
- Added `transport_play_with_states(...)` seam coverage so the supported transport start path is tested directly without depending on Tauri `State` wrappers.
- Validated with `npm run lint`, `npm run check-types`, and `npm run test:rust`; `cargo clippy --lib -- -D warnings` still fails on pre-existing unrelated issues in `src/automation/registry.rs` and `src/instructions/parse/get_horizontal_spacing.rs`.

### Completion Notes List

- Story context created with explicit runtime scope boundaries to avoid spilling into Stories 1.2 through 1.5.
- Story assumes supported playback must stop executing SCVI runtime behavior even if the VST runtime seam is still partial.
- Supported `transport_play` now advances only transport timing/tempo state from score data and cannot reach SCVI cache-backed rendering.
- Removed the obsolete sampler cache clone helper and transport-only `PlaybackTick` helpers that became dead code after the runtime disconnect.
- Frontend transport actions remained compile-safe without `src/services/actions.ts` changes.
- Updated `docs/architecture.md` to classify the SCVI streamer path as legacy sampler/audition infrastructure instead of the supported transport runtime.
- Added transport start-path seam tests covering the supported timing-only command path and the past-flow-end no-start case.
- Updated `docs/project-context.md` so agent guidance matches the timing-only transport path and legacy sampler streaming ownership.

### File List

- `_bmad-output/implementation-artifacts/1-1-remove-scvi-from-the-active-playback-runtime.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/project-context.md`
- `src-tauri/src/lib.rs`
- `src-tauri/src/playback/sampler.rs`
- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/playback/tuplet_projection.rs`

### Change Log

- 2026-03-20: Disconnected supported transport playback from SCVI sampler runtime state, added transport-only seam tests, and updated narrow developer runtime guidance.
- 2026-03-20: Added direct `transport_play` seam coverage, reran lint/typecheck/Rust tests, and aligned `docs/project-context.md` with the timing-only transport model.
