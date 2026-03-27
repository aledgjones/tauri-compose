# Story 4.1: Translate Routed Score Instruments Into Playback Preparation

Status: done

## Story

As a composer,
I want routed score instruments to prepare a playback-capable path from notation into the VST runtime,
so that written music can become audible without leaving the score workflow.

## Acceptance Criteria

1. Given a score instrument has a valid routed playback target, when Compose prepares the score for playback or audition, then the written instrument is translated into the backend playback path required for the routed VST target, and that preparation remains backend-owned rather than recreated in frontend state.
2. Given an instrument is unrouted or unresolved, when playback preparation runs, then Compose preserves the explicit non-ready state for that instrument, and it does not invent hidden fallback playback behavior.
3. Given the current MVP is single-target-first, when playback preparation resolves routed instruments, then the supported preparation path stays bounded to the current single-target model, and the model remains compatible with later expansion.

## Tasks / Subtasks

- [x] Task 1: Build or extract the backend playback-preparation seam for routed score instruments (AC: 1, 2, 3)
  - [x] Start from the existing Rust-owned seams in `src-tauri/src/playback/runtime_activation.rs` and `src-tauri/src/playback/transport.rs` instead of introducing a new frontend or automation-only path.
  - [x] Translate score instrument route intent into preparation-ready backend data keyed by the current rack-first model: racks own slots, slots own targets, instruments only route to existing slots.
  - [x] Keep all new preparation truth in Rust under `src-tauri/src/playback/`; do not create a parallel React state model, derived cache, or fallback ownership path.
- [x] Task 2: Reuse the current slot-owned runtime-session and audio-context foundations without distorting the architecture (AC: 1, 3)
  - [x] Reuse `PlaybackActivationRuntime` as the source of route readiness and slot-target activation truth.
  - [x] Reuse the existing `AudioContext` sample-rate and mixer access pattern where it fits cleanly; if a shared preparation helper is architecturally cleaner than inlining more logic into transport, prefer the cleaner backend seam.
  - [x] Do not create a second sample-rate negotiation path, a second live-session registry, or instrument-owned plugin lifecycle state.
- [x] Task 3: Keep unresolved and unsupported states explicit while remaining single-target-first (AC: 2, 3)
  - [x] Preserve `Unrouted` and `NeedsAttention` semantics from `ResolvedPlaybackRoute` when preparation runs.
  - [x] Keep the current supported path bounded to one routed playable target at a time; do not broaden this story into multi-target, multi-instrument, or multi-timbral readiness.
  - [x] Do not reactivate sampler-era or SCVI-era fallback playback ownership to make the story pass.
- [x] Task 4: Integrate the preparation seam into the supported playback entry path and keep it reusable for the upcoming audition proof (AC: 1, 3)
  - [x] Wire the preparation output into the current transport-side preparation/snapshot path, or extract a shared Rust helper that transport uses now and auditioning can reuse in Story 4.2.
  - [x] Keep the `plugin_playback_poc_probe` seam as prior art and proof support only; do not turn it into a second canonical playback state model.
  - [x] Avoid UI expansion in this story. Any user-visible work should remain limited to existing readiness semantics already driven by backend truth.
- [x] Task 5: Add seam-local regression coverage and document the canonical preparation path (AC: 1, 2, 3)
  - [x] Add or extend Rust tests near the owning seams, primarily `src-tauri/src/playback/transport.rs` and `src-tauri/src/playback/runtime_activation.rs`.
  - [x] Cover the supported prepared path, explicit non-ready path, and bounded single-target behavior.
  - [x] Update developer-facing docs if this story establishes or clarifies the canonical playback-preparation seam or its reuse expectations.

## Dev Notes

### Story Intent

- This is the first Epic 4 backend bridge from score routing truth into audible-playback preparation.
- The story is not asking for broad transport trust, new UI, or a second playback runtime. It is asking for the canonical backend-owned preparation seam that later audible-playback stories will consume.
- The most important constraint is to extend the existing route/runtime/session foundations rather than inventing a new model.

### User Decisions Captured

- Prior art research is particularly important for this story.
- Some of the audio-context setup already exists, including sample-rate handling. Reuse that where it fits cleanly.
- Reuse should not force the wrong shape. If a cleaner backend seam is better than awkwardly threading old code forward, choose the cleaner seam.

### Epic And Story Context

- Epic 4 is now unblocked because the Story 1.3 through 1.6 trust-restoration gate and required maintainer proof are complete on the corrected slot-owned-session architecture.
- Story 4.1 is the first audible-playback foundation story, not the full proof of sound output.
- Story 4.2 owns proving first audible playback through auditioning, so 4.1 should create a reusable preparation seam rather than hard-coding a transport-only dead end.

### This Story Owns

- Translating routed score instruments into backend playback-preparation data.
- Reusing slot-owned live-session readiness and current audio sample-rate context in that preparation path.
- Keeping preparation backend-owned, explicit, and bounded to the current single-target-first MVP.
- Adding seam-local regression tests for the prepared path and the explicit non-ready path.

### This Story Does Not Own

- Full transport trust behavior across repeated play, pause, resume, or seek.
- New score-context or app-scoped playback UI.
- Multi-instrument, shared-slot, multi-target, or multi-timbral playback expansion.
- Reopen recovery redesign, expression-map semantics, or mixer control behavior.
- Any reintroduction of sampler-era or SCVI-era ownership as a hidden fallback.

### Prior Art Research Summary

- `src-tauri/src/playback/runtime_activation.rs` is already the backend-owned bridge from persisted rack/route intent to same-session runtime readiness. It exposes `PlaybackActivationRuntime`, `ActivatedSlotTarget`, `ActivatedInstrumentRoute`, `resolved_playback_route_by_instrument_id`, and the single-instrument trust gate.
- `src-tauri/src/playback/transport.rs` already creates a playback snapshot through `build_supported_playback_snapshot(...)`, but that snapshot is currently transport-only. This is the closest existing seam for 4.1.
- `src-tauri/src/score/playback.rs` defines the rack-first persisted model: rack slots own `PlaybackSlotTarget`; instruments own `InstrumentPlaybackRoute` references to slots.
- `src-tauri/src/playback/audio.rs` already owns the active device, persistent stream, mixer handle, and current sample rate. That should remain the source of sample-rate truth.
- `src-tauri/src/playback/plugin_playback_poc.rs` is useful prior art for bounded VST proof work and offline preparation behavior, but it is explicitly not the canonical live playback state model.
- Story 3.1 through 3.4 hardened the route model, explicit readiness states, and same-session route/session stability. Epic 4 should build on those seams instead of reopening the UI workflow.

### Cross-Story Intelligence

- Stories 1.4 through 1.6 already moved live VST ownership into slot-keyed runtime sessions and kept `.soloc` persistence limited to portable intent plus optional restorable plugin state.
- Story 3.3 made `resolved_playback_route_by_instrument_id` derive from activation runtime truth and preserved `Unrouted` and `NeedsAttention` as honest states.
- Story 3.4 was explicitly a stability gate over the existing route and session behavior. Do not destabilize `instrument-overview-row`, `playback-rack-panel`, or the current route workflow while implementing 4.1.

### Architecture Compliance

- Rust remains the sole owner of plugin inventory truth, rack state, slot state, route state, playback-preparation state, and transport state.
- The active product path must stay truly VST-owned. Sampler-era and SCVI-era seams are compatibility residue only and must not become accepted owners again.
- `.soloc` continues to store portable score and playback intent only, not live runtime objects or machine-local discovery state.
- The playback runtime remains split between a control plane and a realtime-safe playback plane. Any audio-thread consumption must use precomputed, lock-safe state only.
- Score-context and app-scoped UX boundaries remain unchanged. This story is backend preparation work, not a UI-story excuse.

### Current Relevant Seams

- `src-tauri/src/playback/runtime_activation.rs`
  - Current activation/runtime truth and readiness derivation.
- `src-tauri/src/playback/transport.rs`
  - Current transport-side playback snapshot and start seam.
- `src-tauri/src/playback/audio.rs`
  - Current audio device, mixer, and sample-rate owner.
- `src-tauri/src/score/playback.rs`
  - Canonical rack/slot/route persisted data model.
- `src/services/types.ts`
  - Frontend mirror of backend-owned route and audio state contracts. Touch only if backend payload shapes actually change.
- `src-tauri/src/playback/plugin_playback_poc.rs`
  - Bounded proof seam to study, not to promote into a second canonical model.

### Implementation Guidance

- Prefer extending `build_supported_playback_snapshot(...)` only if that still leaves a reusable backend preparation seam. If transport starts to absorb too much story-specific preparation knowledge, extract a shared helper under `src-tauri/src/playback/` and keep transport as a consumer.
- Use `PlaybackActivationRuntime` and the rack-first score model as the entrypoint for preparation. Do not prepare directly from frontend-emitted route summaries.
- Reuse the current `AudioContext` pattern for sample-rate and mixer access. A clean pattern already exists: lock briefly, capture what is needed, then release the lock before doing heavier preparation work.
- Preserve explicit unresolved states. If a route lacks a live activated slot target, preparation should report or respect that non-ready state rather than manufacturing a hidden substitute.
- Keep the supported path single-target-first. If broader multi-route support looks tempting, note it and stop. That is future scope.
- Treat `plugin_playback_poc_probe` as research and test prior art only. The story should not fork the product path into probe-prepared playback versus transport-prepared playback.

### Reuse Guidance From Existing Audio Context Setup

- The repo already owns active output-device selection and sample-rate changes in `src-tauri/src/playback/audio.rs`.
- `AudioContext::build_stream(...)` already builds the persistent output stream around `cpal::StreamConfig { channels, sample_rate, buffer_size }`.
- The current project direction does not require a new audio-device abstraction for 4.1. Reuse the existing audio state unless a cleaner backend preparation helper naturally consumes a subset of that state.

### Risks And Regression Traps

- Sample-rate mismatch between preparation-time assumptions and the current audio device can make the first audible path unstable.
- Holding `AppContext` or audio locks while doing heavier preparation work risks transport fragility and UI stalls.
- Letting transport, audition, and probe paths each invent their own preparation logic will create three competing playback models.
- Reopening old frontend route surfaces for this backend story is likely scope drift.
- Reintroducing sampler or SCVI fallback behavior would violate the approved rack-first, VST-owned model and create misleading readiness behavior.

### Testing Requirements

- Add or extend Rust unit tests at the owning seams rather than inventing a cross-cutting playback harness.
- Minimum coverage should prove:
  - a valid routed single-target instrument produces backend preparation output through the supported seam;
  - unrouted or unresolved instruments remain explicitly non-ready;
  - the bounded single-target-first gate is preserved;
  - transport-facing preparation does not silently depend on frontend-owned state.
- Run the repo-standard validation relevant to touched files:
  - `cargo test`
  - `cargo clippy`
  - `npm run lint`
  - `npm run check-types`

### Documentation Impact

- If implementation establishes or clarifies the canonical backend playback-preparation seam, update the relevant architecture/developer docs in the same change.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - any AI-agent instruction file that must reflect a new required seam or prohibited fallback

### Latest Technical Information

- Current repo-approved stack remains the source of truth: Tauri 2.10.1, Rust 2021, and cpal 0.17.3.
- External verification still matches the repo’s current patterns:
  - CPAL `StreamConfig` continues to describe stream setup through `channels`, `sample_rate`, and `buffer_size`, which aligns with the existing `AudioContext::build_stream(...)` approach.
  - Tauri 2 state management guidance still supports `std::sync::Mutex` for managed state when guards are not held across `await`, which matches the repo’s current `AppContext` and `AudioContext` ownership pattern.
- No library upgrade or alternate state-management pattern is required for this story.

### Project Structure Notes

- This story should stay concentrated in backend playback seams under `src-tauri/src/playback/` plus any mirrored type updates if backend payloads change.
- Avoid spreading 4.1 into new UI surfaces, new windows, or a top-level architecture rewrite.
- If implementation starts to require persistence or migration changes, stop and re-evaluate scope because 4.1 is a preparation story, not a persisted-data redesign.

### References

- `_bmad-output/planning-artifacts/epics.md` - Epic 4 and Story 4.1 acceptance criteria.
- `_bmad-output/planning-artifacts/prd.md` - Score-to-playback mapping, transport playback, and recovery requirements.
- `_bmad-output/planning-artifacts/architecture.md` - playback ownership, control-plane versus realtime-plane split, and Epic 4 readiness guidance.
- `_bmad-output/planning-artifacts/ux-design-specification.md` - score-first, low-indirection playback UX boundaries.
- `docs/project-context.md` - repo-wide rules for playback ownership, slot-owned runtime sessions, and score-context boundaries.
- `/memories/repo/playback-rack-first-model.md` - rack-first playback ownership constraints.
- `/memories/repo/slot-owned-live-runtime-sessions-story-1-4.md` - slot-owned live-session foundation and portable plugin-state restoration guidance.
- `/memories/repo/rack-playback-proof-seam.md` - bounded proof seam guidance for `plugin_playback_poc_probe`.
- `/memories/repo/same-session-playback-activation-story-3-8.md` - activation runtime refresh boundaries and backend-owned route truth.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Prior-art exploration focused on `runtime_activation`, `transport`, `audio`, `score/playback`, and `plugin_playback_poc`.
- Recent commit footprint review:
  - `feat(3.4): slot consistency`
  - `feat(3.3): slot validity`
  - `feat(3.2): slot sanity`
  - `feat(3.1): rack attention states`
- Implemented shared backend preparation in `src-tauri/src/playback/preparation.rs` and routed transport snapshotting through it.
- Validation commands: `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `npm run lint`, `npm run check-types`.

### Completion Notes List

- Created a 4.1 story artifact that anchors the dev agent to the actual Epic 4 backend seams rather than generic playback language.
- Captured the user’s explicit direction that prior art research matters most here and that audio sample-rate setup should be reused when it genuinely fits.
- Preserved the rack-first, slot-owned-session model as the non-negotiable architectural guardrail.
- Kept Story 4.1 bounded to backend playback preparation and reusable preparation seams, leaving audible proof, transport trust, and broader expansion to later stories.
- Added `prepare_supported_playback(...)` as the canonical backend preparation seam, carrying resolved route truth, single-target support gating, slot-owned restorable plugin state, and the active audio sample rate.
- Updated transport snapshot construction to consume the shared preparation artifact so playback scheduling no longer invents a parallel multi-route preparation model.
- Added seam-local Rust coverage for the ready path, explicit non-ready path, and bounded single-target behavior, and updated architecture/development docs to point future transport and audition work at the shared seam.
- Kept Sequence route controls minimal after review follow-up: unrouted instruments and empty assigned slots remain badge-free (`--` / `N. --`), while the warning icon stays reserved for broken assigned plugin routes.
- Documented the UI-semantics guardrail in developer docs and agent instructions: backend readiness stays truthful, but the user-facing route control only surfaces warning chrome for broken assigned plugin routes.

### File List

- `_bmad-output/implementation-artifacts/4-1-translate-routed-score-instruments-into-playback-preparation.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `.github/copilot-instructions.md`
- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `src-tauri/src/playback/mod.rs`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src-tauri/src/playback/preparation.rs`
- `src-tauri/src/playback/runtime_activation.rs`
- `src-tauri/src/playback/transport.rs`
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`

## Change Log

- 2026-03-26: Created the Story 4.1 implementation guide and moved Epic 4 / Story 4.1 sprint tracking into active ready-for-dev state.
- 2026-03-26: Implemented the shared backend playback-preparation seam, integrated transport snapshotting with single-target preparation gating, added seam-local regression coverage, updated playback docs, and moved Story 4.1 to review.
- 2026-03-27: Resolved code-review findings by blocking unsupported routed transport starts, preserving empty-slot routes as `NeedsAttention`, and keeping unrouted Sequence controls badge-free while recovery states keep the existing warning icon/title treatment.
- 2026-03-27: Narrowed Sequence readiness presentation further so empty assigned slots stay visually neutral (`N. --`) and only broken assigned plugin routes show the warning icon.
- 2026-03-27: Added explicit documentation and agent-rule guidance that backend playback truth stays richer than the warning UI, so unrouted and empty-slot states remain neutral while only broken assigned plugin routes surface warning chrome.

### Review Findings

- [x] [Review][Patch] Multiple routed instruments fall through to silent transport playback [src-tauri/src/playback/transport.rs:243]
- [x] [Review][Patch] Routed instruments pointing at empty slots still collapse to `Unrouted` instead of an explicit non-ready state [src-tauri/src/playback/runtime_activation.rs:166]
