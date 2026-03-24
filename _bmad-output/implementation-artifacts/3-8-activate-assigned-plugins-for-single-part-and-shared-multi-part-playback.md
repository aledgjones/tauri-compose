# Story 3.8: Activate Single-Target Rack Slots For Playback And Preserve Same-Session Runtime Coherence

Status: ready-for-dev

## Story

As a maintainer,
I want valid single-target rack slots and routed instruments to activate usable plugin destinations within the current app session,
so that the current routed score can participate in later playback work without relying on one-shot setup behavior.

## Acceptance Criteria

1. Given score instruments have valid single-target playback routes, when those routes are activated for use within the current app session, then Compose loads or attaches the slot targets needed for those routes and each routed instrument resolves to a usable backend-owned playback destination.
2. Given activated single-target routes remain in use during ordinary in-app work, when the user continues editing, loading scores, or revisiting routed playback surfaces within the same app session, then the activated runtime state remains coherent from the user's perspective and Compose does not treat activation as a one-shot setup action that is silently lost after first use.
3. Given lifecycle issues or unavailable routes can occur during the app session, when they affect an activated routed instrument or slot, then the affected instruments transition into an explicit attention-needed or unavailable state and Compose preserves recoverability instead of failing invisibly.

## Tasks / Subtasks

- [ ] Task 1: Define one backend-owned single-target activation seam that consumes existing rack and route intent without extending the legacy sampler-era product contract (AC: 1)
  - [ ] Reuse the persisted rack-slot and route model from Stories 3.1 through 3.4 rather than inventing a parallel activation topology.
  - [ ] Keep activation bounded to currently supported single-target routes; do not absorb shared-slot or multi-target runtime behavior from deferred Story 3.5.
  - [ ] Preserve the rule that React remains a presentation layer and must not infer activation truth or runtime ownership.
- [ ] Task 2: Refresh activation coherently through the ordinary same-session seams that already change score playback state (AC: 1, 2)
  - [ ] Ensure slot assignment, slot unassignment, route assignment, route clearing, and project load all drive the same backend-owned activation refresh path.
  - [ ] Keep same-session continuity backend-owned so the routed score does not feel like a one-shot setup action after file load or further editing.
  - [ ] Do not expand this story into audible transport behavior, first-playback rendering, or broader transport trust semantics.
- [ ] Task 3: Surface explicit degraded runtime state through existing readiness truth instead of silent fallback (AC: 2, 3)
  - [ ] Drive non-ready outcomes from backend-owned activation or availability truth rather than frontend heuristics.
  - [ ] Preserve clear recoverability when an activated slot becomes unavailable or cannot remain usable during the current app session.
  - [ ] Keep score-context recovery anchored in the existing rack and route workflow rather than inventing a plugin-manager-first runtime-repair flow.
- [ ] Task 4: Add focused validation and documentation updates for the bounded activation path (AC: 1, 2, 3)
  - [ ] Add backend tests for single-target activation refresh on route and slot mutations plus project load.
  - [ ] Add frontend or seam-local tests only where UI-facing readiness behavior changes as a result of the new backend-owned activation truth.
  - [ ] Update docs so Epic 3 activation is described as same-session runtime preparation for currently routed single-target slots, while reopen recovery remains owned by Epic 5.

## Dev Notes

### Story Intent

- Story 3.8 is the bridge from persisted rack or route intent into backend-owned runtime usability inside the current app session.
- This story is intentionally narrower than the original Epic 3 wording. It should prove the single-target activation path first.
- Same-session runtime coherence belongs with this activation seam. Cross-session reopen and degraded reopen recovery belong in Epic 5.

### User Decisions Captured

- Shared-slot and multi-target routing are deferred beyond the current MVP path, so Story 3.8 must not claim shared-slot or multi-target runtime activation yet.
- Story 3.7 is also deferred, so Story 3.8 should not absorb richer reroute or recovery UX beyond making non-ready runtime state explicit.
- Current observed plugin persistence across file loads is useful evidence, but it does not by itself satisfy the intended VST-owned runtime activation contract for this story.

### Bounded Slice Template

#### This Story Owns

- Backend-owned same-session activation of valid single-target routed slots.
- Activation refresh through ordinary score, slot, route, and file-load seams in the current app session.
- Explicit non-ready runtime outcomes when activation cannot remain usable.

#### This Story Does Not Own

- Shared-slot or multi-target activation. That remains deferred beyond the current MVP path.
- Automatic routing for new instruments. That remains deferred beyond the current MVP path.
- Repair and reroute UX beyond existing score-context recovery surfaces. That remains deferred beyond the current MVP path.
- Audible playback, transport scheduling, or first-heard score material. Epic 4 owns that.
- Cross-session reopen restoration or degraded reopen handling. Epic 5 owns those.

#### Authoritative State Lives In

- Rust-owned persisted rack and route intent in `score.playback.rack` and `Instrument.playback_route`.
- Rust-owned readiness and route summary emission in `resolved_playback_route_by_instrument_id`.
- The future backend-owned runtime activation seam that should sit beside the current playback model rather than widening `SamplerContext` into the supported product runtime.

#### Future Stories Intentionally Deferred

- Verified multi-target shared-slot activation.
- First audible playback and transport-driven playback behavior.
- Reopen restoration and degraded reopen recovery across app sessions.
- Route-owned expression behavior and later realism work.

#### Verification Required For Done

- Tests prove valid single-target routes activate usable runtime state within the app session.
- Tests prove ordinary same-session changes and project load refresh activation coherently instead of silently losing state.
- Tests prove non-ready runtime outcomes stay explicit and recoverable.

#### Open Questions Requiring Human Clarification

- None. The current planning decision is already to make this the next active Epic 3 implementation slice.

### Scope Boundaries

- In scope:
  - single-target runtime activation for currently routed slots
  - same-session activation refresh on score and file-load changes
  - backend-owned non-ready runtime state when activation cannot remain valid
  - focused seam-local tests and docs
- Out of scope:
  - shared-slot or multi-target activation
  - transport playback behavior
  - broader recovery UX redesign
  - reopen restoration across app restarts or machine changes

### Cross-Story Alignment

- Story 3.1 established the rack, slot, target, address, and route model this story must consume rather than replace.
- Story 3.3 prepares slots with assigned plugins but stops short of runtime activation.
- Story 3.4 persists explicit route intent but stops short of runtime participation.
- Story 3.5 must later justify any shared-slot or multi-target activation this story does not currently own.
- Epic 5 owns reopen restoration and degraded reopen handling once the app session ends.
- Epic 4 should consume this activation seam for first audible playback instead of re-deriving runtime preparation independently.

### Previous Story Intelligence

- Existing Epic 3 implementation artifacts repeatedly deferred runtime activation to Story 3.8.
- The current backend calls `sync_sampler_with_score(...)` from slot, route, and file-load seams, which provides observable same-session plugin-cache persistence but remains a legacy sampler-era path rather than the target VST-owned runtime contract.
- The current route-choice model is still single-target for prepared slots, which makes single-target activation the only bounded runtime slice that is safe to define here.

### Architecture Compliance

- Rust remains the sole owner of playback topology, activation truth, and degraded runtime state.
- React must not infer runtime activation or silently reinterpret route usability from scattered fields.
- Do not widen `SamplerContext`, `soloc://state/sampler`, or sampler-era actions into the long-term VST runtime architecture; if a compatibility bridge is required, keep it explicitly temporary and backend-owned.
- If implementation changes persisted playback semantics rather than runtime-only activation behavior, assess migration impact and update `src-tauri/src/migrations/next.rs`.

### Current Relevant Seams

- `src-tauri/src/score/instruments/mod.rs`
  - Current score mutations for slot and route assignment already trigger `sync_sampler_with_score(...)`.
  - Current readiness truth is emitted from `resolved_playback_route_by_instrument_id(...)`.
- `src-tauri/src/files.rs`
  - Project load stops transport, restores score state, and then drives `sync_sampler_with_score(...)` after load.
- `src-tauri/src/playback/sampler.rs`
  - Current same-session plugin-id cache behavior exists here, but this seam is legacy scaffolding rather than the architecture to extend indefinitely.
- `src-tauri/src/playback/transport.rs`
  - Current supported transport path is still timing-only and should later consume a proper activation seam rather than owning route activation itself.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Current compact readiness presentation is the frontend seam most likely to reflect backend-owned activation outcomes.

### Legacy Or Superseded Seams To Avoid

- Do not treat the SCVI sampler cache as the long-term playback-activation model.
- Do not reintroduce direct instrument-to-plugin ownership.
- Do not let transport, plugin editor hosting, or Plugin Manager become the hidden owner of runtime activation truth.

### Implementation Guidance

- Prefer one bounded backend refresh path over multiple partial activation hooks.
- Keep the first reopen of this story single-target only, even if enums already mention richer target modes.
- Treat current same-session persistence observations as evidence of where behavior already exists, not as proof that the right product seam is in place.
- Preserve explicit recoverability and avoid silent fallback runtime behavior.

### Persistence And Migration Requirements

- No migration should be required if the story only introduces runtime-only activation behavior.
- If persisted rack, route, or readiness semantics change, add or update the pending migration in `src-tauri/src/migrations/next.rs`.
- Machine-local runtime activation state must remain outside `.soloc` persistence.

### File Structure Requirements

- Backend work should stay centered in:
  - `src-tauri/src/score/instruments/mod.rs`
  - `src-tauri/src/files.rs`
  - `src-tauri/src/playback/transport.rs`
  - any new VST-owned playback activation seam under `src-tauri/src/playback/`
- Frontend-facing updates, if needed, should stay centered in:
  - `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - existing Sequence rack or instrument tests that consume backend-owned readiness truth

### Testing Requirements

- Backend tests should cover:
  - activation refresh after slot assignment and unassignment
  - activation refresh after route assignment and clearing
  - activation refresh after project load
  - explicit degraded state when a previously valid route cannot remain usable
- Frontend tests should only be added where backend-owned readiness presentation changes materially.
- Validate with the project-standard commands relevant to touched areas:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings` when Rust changes
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib` when Rust changes

### Library / Framework Requirements

- Stay on the existing React, TypeScript, Rust, Tauri, and Vite stack.
- Reuse existing backend general-state emission rather than inventing a frontend-owned runtime state model.
- Prefer seam-local tests over a broad new end-to-end playback harness at this stage.

### Documentation Impact

- Update `docs/architecture.md` to describe Story 3.8 as same-session runtime activation for routed single-target slots and to keep reopen restoration in Epic 5.
- Update `docs/development-guide.md` if implementation changes the recommended developer seam for runtime activation refresh.
- Update `docs/data-models.md` if runtime activation introduces a new backend-owned non-persisted activation state concept that developers need to distinguish from persisted route intent.

### Project Structure Notes

- Preserve the sprint-status key `3-8-activate-assigned-plugins-for-single-part-and-shared-multi-part-playback` for tracker continuity even though the story title has been narrowed.

## Dev Agent Record

### Implementation Plan

- Story file created during sprint correction and then reopened once Epic 3 was narrowed to the single-target playback runway.
- This is now the next active Epic 3 slice before Epic 4 playback hookup.

### Debug Log

- Created the dedicated Story 3.8 implementation artifact after narrowing the epic-level wording.
- Kept same-session runtime coherence with Story 3.8 and pushed reopen responsibilities into Epic 5.

### Completion Notes

- Comprehensive story context is in place so implementation can proceed on a bounded single-target runtime slice without resurrecting the old broader scope.

## File List

- \_bmad-output/implementation-artifacts/3-8-activate-assigned-plugins-for-single-part-and-shared-multi-part-playback.md

## Change Log

- 2026-03-24: Created the narrowed Story 3.8 implementation artifact and kept tracker-key continuity with the pre-correction slug.
- 2026-03-24: Reopened Story 3.8 as ready-for-dev after narrowing Epic 3 to the single-target playback runway.
