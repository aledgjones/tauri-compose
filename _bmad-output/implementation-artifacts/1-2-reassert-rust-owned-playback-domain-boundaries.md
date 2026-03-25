# Story 1.2: Reassert Rust-Owned Playback Domain Boundaries

Status: done

## Story

As a maintainer,
I want the playback domain to have one authoritative backend-owned model,
so that downstream work does not reintroduce parallel truth or unclear ownership.

## Acceptance Criteria

1. Given playback inventory, rack, slot, route, transport, and recovery data cross backend and frontend boundaries, when the Epic 1 foundation reset is complete, then Rust remains the sole source of truth for active playback-domain state, and frontend state is limited to transient presentation concerns.
2. Given project files and machine-local plugin state have different responsibilities, when playback-related state is saved or restored, then `.soloc` stores portable project intent and references only, and machine-local discovery, cache, and validation state remain outside project files.
3. Given backend-owned playback models are exposed to the frontend, when commands and shared payloads are updated, then naming and ownership stay aligned with the architecture rules, and no new sampler-era terminology or parallel adapter model is introduced for the supported path.

## Tasks / Subtasks

- [x] Audit and tighten backend-owned playback truth boundaries across active seams (AC: 1, 3)
  - [x] Trace the authoritative playback-domain model from `ScorePlayback` and `PlaybackActivationRuntime` through `AppContext`, Tauri commands, automation registry, and frontend mirrors to confirm every active mutation still originates in Rust.
  - [x] Remove or refactor any active code path that recreates rack, slot, route, readiness, or transport truth in React state, window-local helpers, or adapter layers rather than consuming Rust-owned payloads.
  - [x] Keep `PlaybackActivationRuntime::from_score_and_inventory(...)` and the backend refresh points as the only readiness-resolution path for the supported single-target workflow.
- [x] Reassert persistence and portability boundaries for playback-domain data (AC: 2)
  - [x] Verify `.soloc` save and load seams persist only portable score intent, rack or route references, and other project-owned playback data, without writing machine-local plugin inventory, validation results, cache state, or host-environment metadata.
  - [x] Confirm reopen or restore logic resolves persisted intent against current machine-local inventory rather than serializing resolved availability into project files.
  - [x] Assess migration impact; if persisted semantics or payload shape change, update `src-tauri/src/migrations/next.rs` using top-level `next_id` allocation rules where new playback slot ids are introduced, otherwise record that no migration change was required.
- [x] Align command, payload, and terminology boundaries with the architecture vocabulary (AC: 1, 3)
  - [x] Normalize active backend and frontend contracts around `inventory`, `rack`, `slot`, `route`, `target`, `address`, and `recovery`, and remove sampler-era or assignment-era wording that suggests parallel ownership for the supported path.
  - [x] Ensure FE→BE mutations continue to flow only through the existing Rust-owned commands in `src-tauri/src/score/instruments/mod.rs` and related seams, with `snake_case` payload naming preserved end to end.
  - [x] Confirm automation or MCP-facing surfaces reuse the same backend seams instead of introducing playback-domain shortcuts or duplicate contract models.
- [x] Add seam-local regression coverage for ownership and portability rules (AC: 1, 2, 3)
  - [x] Extend Rust tests around `runtime_activation`, playback save or load, and score playback mutation seams to prove readiness is derived from persisted intent plus machine-local inventory, not frontend state or serialized cache truth.
  - [x] Add or update tests that verify unavailable plugins degrade to recoverable route states without mutating persisted user intent.
  - [x] Update relevant TypeScript tests for state providers, actions, or shared type mirrors so they assert display-layer consumption of Rust-owned payloads rather than frontend-owned playback derivation.
- [x] Update documentation and agent guidance to preserve the boundary reset (AC: 2, 3)
  - [x] Update `docs/architecture.md`, `docs/project-context.md`, `docs/development-guide.md`, and `TESTING.md` wherever they need stronger guidance on Rust ownership, `.soloc` portability, machine-local inventory separation, or naming rules.
  - [x] Update AI-agent instructions if story 1.2 changes required implementation rules or terminology discipline for future playback work.

## Dev Notes

### Story Intent

Story 1.1 removed active sampler and SCVI ownership from the product path. Story 1.2 is the architectural lock-in pass that makes the reset durable. The implementation should not add features; it should make ownership boundaries explicit, testable, and hard to accidentally violate in later playback stories.

### Story Foundation

- Epic 1 is a trust-restoration epic, not a feature-expansion epic. This story exists to prevent the codebase from sliding back into frontend-owned playback truth, serialized machine-local state, or renamed legacy adapters.
- The approved course correction explicitly preserves the rack-first product model while treating prior playback progress as partial runway rather than a trustworthy foundation.
- Downstream stories, especially trust-baseline and first-audible-playback work, should be able to assume one backend-owned playback model with no ambiguity about project-owned versus machine-local state.

### Previous Story Intelligence

- Story 1.1 established `PlaybackActivationRuntime::from_score_and_inventory(...)` as the authoritative readiness seam and removed sampler-backed reconciliation from active product behavior.
- `SamplerContext`, `soloc://state/sampler`, and `actions.sampler.*` were removed from active frontend ownership. Do not recreate them under new names or through local caching.
- Story 1.1 intentionally left `.soloc` semantics unchanged. Story 1.2 must reassess migration impact only if payload ownership or persistence meaning changes.
- The highest repeat-risk from story 1.1 is leaving hidden compatibility terminology or secondary derivation paths in place after legacy runtime removal.

### Technical Requirements

- Rust remains the sole owner of plugin inventory truth, rack state, slot state, route state, playback-preparation state, transport state, and persisted playback-domain state.
- React state must stay limited to transient presentation concerns such as hover, open or closed state, draft input, and window-local affordances.
- `.soloc` may store portable score intent and playback references only. Machine-local discovery, validation, cache, and environment-specific metadata must remain outside project files.
- Reopen or restore behavior must resolve persisted playback intent against the current machine-local inventory rather than persisting resolved runtime availability.
- Shared FE↔BE payloads remain `snake_case` and must align with Rust-owned naming.
- Do not add adapter models, compatibility shims, or terminology that preserve sampler-era ownership concepts for the supported VST path.
- If a migration is required, author it in `src-tauri/src/migrations/next.rs` and allocate any new playback slot ids from top-level `next_id`.

### Architecture Compliance

- Preserve the brownfield Tauri + Rust + React + TypeScript + Vite foundation; this is not a replatforming slice.
- Keep playback-domain mutations in backend seams under `src-tauri/src/playback/`, `src-tauri/src/score/`, `src-tauri/src/files.rs`, and `src-tauri/src/lib.rs`.
- Keep score-context versus app-scoped UI boundaries intact. This story is about ownership and payload discipline, not adding new proof-oriented UI.
- Treat React-side parallel truth and storing machine-local playback facts inside `.soloc` as architectural defects.
- Automation and MCP surfaces must call the same Rust-owned commands as the product UI.

### Library / Framework Requirements

- Continue using typed Tauri commands with `snake_case` payloads for FE→BE mutation boundaries.
- Keep TypeScript mirrors in `src/services/types.ts` aligned with Rust structs and enums; do not introduce frontend-only topology types for active playback-domain truth.
- Keep `PlaybackActivationRuntime` as a backend-owned non-persisted runtime seam under `AppContext` rather than promoting runtime readiness into persisted score data or React contexts.

### File Structure Requirements

- Persisted playback models live in `src-tauri/src/score/playback.rs` and related score seams.
- Runtime readiness and activation logic live in `src-tauri/src/playback/runtime_activation.rs` and transport-facing playback seams.
- Save or load ownership checks live in `src-tauri/src/files.rs` and migration logic in `src-tauri/src/migrations/next.rs` when required.
- Command registration and shared general-state payload wiring live in `src-tauri/src/lib.rs` and `src-tauri/src/automation/registry.rs`.
- Frontend display-layer consumption remains in `src/services/state/`, `src/services/actions.ts`, and `src/services/types.ts`.
- Do not move playback-domain truth into React contexts, selector helpers, or window components.

### Testing Requirements

- Prefer seam-local Rust tests around `runtime_activation`, playback persistence, migration behavior, and score playback mutation seams.
- Cover save or load and reopen scenarios that prove project files preserve user intent while runtime readiness is recomputed from machine-local inventory.
- Cover degraded availability transitions so routes move to explicit recoverable states without rewriting persisted route intent.
- Update TypeScript tests only where needed to prove frontend playback state remains a consumer of Rust-owned snapshots and commands.
- Run at minimum the validation set required by touched code, including `npm run lint`, `npm run check-types`, and focused Rust tests; include `cargo clippy` if Rust code changes meaningfully.

### Implementation Risks And Guardrails

- The biggest failure mode is leaving duplicate playback truth in frontend state or convenience helpers after sampler removal made the duplication less visible.
- A second failure mode is allowing resolved machine-local plugin availability or validation state to leak into `.soloc`, which would corrupt portability and recovery semantics.
- A third failure mode is preserving assignment-first or sampler-era terminology in new contracts, which will invite parallel ownership models in later stories.
- A fourth failure mode is creating automation-only or MCP-only playback pathways that bypass the same backend commands used by the product UI.
- Do not broaden this story into new routing UX or audible-playback behavior. Its value is in boundary enforcement, not feature expansion.

### Relevant Code Seams

- `src-tauri/src/score/playback.rs`: canonical persisted rack, slot, and route models.
- `src-tauri/src/playback/runtime_activation.rs`: authoritative runtime readiness derived from score intent plus machine-local inventory.
- `src-tauri/src/lib.rs`: `AppContext`, emitted general state, and command registration.
- `src-tauri/src/files.rs`: save, load, and playback runtime reconstruction during project open.
- `src-tauri/src/score/instruments/mod.rs`: backend mutation commands for slot and route ownership.
- `src-tauri/src/automation/registry.rs`: automation or MCP-facing reuse of the same backend playback seams.
- `src/services/state/context.tsx` and `src/services/state/provider.tsx`: display-layer state providers that must not grow playback truth.
- `src/services/actions.ts` and `src/services/types.ts`: command and payload mirrors that must stay Rust-aligned.

### Documentation Impact

Minimum expected documentation review and update targets:

- `docs/architecture.md`
- `docs/project-context.md`
- `docs/development-guide.md`
- `TESTING.md`
- AI-agent instruction files if terminology or implementation rules change

### Git Intelligence Summary

- Recent commit direction is aligned with this story: `feat(1.1): sampler cleanup`, `feat: course correction`, `feat(3.8): vst management.`, `chore: epic 3 redirection`, and `chore: block 3.7` all reinforce that current work is about trust restoration, boundary cleanup, and preserving the rack-first model rather than adding another transitional playback path.

### Project Structure Notes

- The repo’s active playback model is already rack-first: racks own slot lifecycle, slots own plugin assignment, and instruments own routes to existing slots.
- `PlaybackRackSlot.target` remains optional, so empty slots are valid persisted state and must not be “repaired” by frontend assumptions.
- `resolved_playback_route_by_instrument_id` should continue to derive from backend runtime activation truth, not frontend recomputation.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.2: Reassert Rust-Owned Playback Domain Boundaries]
- [Source: _bmad-output/planning-artifacts/architecture.md#Core Architectural Decisions]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- [Source: _bmad-output/planning-artifacts/architecture.md#Frontend Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming Patterns]
- [Source: _bmad-output/planning-artifacts/architecture.md#Structure Patterns]
- [Source: _bmad-output/planning-artifacts/architecture.md#Format Patterns]
- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-03-24.md#4.1 Epic changes]
- [Source: _bmad-output/planning-artifacts/prd.md#Product Scope]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Core User Experience]
- [Source: docs/project-context.md#Critical Implementation Rules]
- [Source: /memories/repo/playback-rack-first-model.md]
- [Source: /memories/repo/playback-route-migration-next-id.md]
- [Source: /memories/repo/same-session-playback-activation-story-3-8.md]
- [Source: /memories/repo/scvi-retirement-verification.md]

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story creation workflow analysis across epics, architecture, PRD, UX, repo memory, prior story, and recent git history.
- Minimal boundary-hardening pass: delegated the score-side test-only resolved-route helper back to `PlaybackActivationRuntime`, added a round-trip persistence degradation regression in `files.rs`, and reran the Rust lib suite plus repo lint/typecheck.
- Follow-up audit pass verified the remaining Story 1.2 subtasks against existing architecture docs, automation registry wiring, frontend action contracts, provider behavior, and existing TypeScript playback-consumption tests before closing the story.
- Review follow-up pass moved the files test restore helper onto the production load seam by recomputing playback activation runtime during restore, updated the `performance` binary playback stub to match backend route-resolution semantics used by imported score tests, and reran targeted plus full Rust validation.

### Completion Notes List

- Comprehensive implementation context created for story 1.2 with explicit ownership, persistence, naming, testing, and documentation guardrails.
- Sprint status advanced from `backlog` to `ready-for-dev` for story 1.2.
- Minimal implementation pass confirmed the product-path playback ownership split was already sound and hardened the remaining test seam so route readiness now resolves through `PlaybackActivationRuntime` instead of a duplicate score-side implementation.
- Added a save/load regression proving persisted route intent survives `.soloc` round-trip while runtime availability degrades against the current machine-local inventory.
- Reviewed migration impact for this pass; no persisted semantic change was introduced, so `src-tauri/src/migrations/next.rs` did not require updates.
- Verified that the remaining architecture subtasks were already satisfied by the active codebase: Rust-owned commands and emitted payloads remain the only supported playback mutation path, automation reuses those same backend seams, and no active sampler-era terminology remains on the supported path outside migration-only compatibility handling.
- Verified that existing TypeScript tests already assert display-layer consumption of backend-owned playback state and `snake_case` command contracts; executed `src/services/actions.test.ts` and `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` successfully rather than adding duplicate test coverage.
- Reviewed agent-guidance impact; no additional AI instruction update was required because the active repo instructions already encode the rack-first, Rust-owned playback boundary and migration rules this story depends on.
- Fixed the code-review finding by making `restore_loaded_project_for_test(...)` refresh `PlaybackActivationRuntime` the same way `load_project_from_file(...)` does, so the new persistence regression now proves the load seam instead of a manual recomputation step.
- Updated the `src-tauri/src/bin/performance.rs` playback stub so imported `score::instruments` tests continue to exercise backend route-resolution semantics under `cargo test`.
- Validation after the fix: `cargo test` passed in `src-tauri/`; `npm run lint` and `npm run check-types` passed at repo root; `cargo clippy --tests -D warnings` remains blocked by pre-existing warnings in `src/playback/audio.rs`, `src/playback/mixer.rs`, and `src/playback/plugin_inventory.rs` outside this story's changes.

### File List

- \_bmad-output/implementation-artifacts/1-2-reassert-rust-owned-playback-domain-boundaries.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- src-tauri/src/bin/performance.rs
- src-tauri/src/score/instruments/mod.rs
- src-tauri/src/files.rs
- TESTING.md

## Change Log

- 2026-03-25: Applied a minimal Story 1.2 hardening pass by routing test-only playback resolution back through `PlaybackActivationRuntime`, adding a `.soloc` round-trip degradation regression, and tightening playback ownership test guidance.
- 2026-03-25: Completed the remaining Story 1.2 audit/verification pass, confirmed the existing command, automation, frontend-consumption, and documentation boundaries already satisfy the story, and advanced the story to review.
- 2026-03-25: Addressed the review finding by aligning the files test restore helper with the production load seam, updating the performance test stub to preserve backend route-resolution semantics under full-suite test runs, and closing the story as done after `cargo test`, `npm run lint`, and `npm run check-types` passed.
