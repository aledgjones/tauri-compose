# Story 1.3: Establish A Trusted Single-Instrument Reset Baseline

Status: done

## Story

As a maintainer,
I want explicit trust-restoration checks for the reset playback path,
so that later audible-playback work starts from a verified single-instrument VST-only baseline instead of assumed progress.

For this story, "single-instrument" means a single VST instance that hosts one playable instrument at a time, even if that instrument exposes multiple articulations, expressions, or mic mixes. HALion-style multi-slot or multi-part VST instances remain out of scope for this baseline.

## Acceptance Criteria

1. Given the legacy path has been removed from active behavior and backend ownership has been clarified, when the reset baseline is evaluated, then the project has explicit checks for single-instrument VST-path activation and runtime coherence, and those checks are strong enough to block downstream epic work if the foundation is still untrusted.
2. Given the reset baseline is now the gate before first-audible-playback work, when trust-restoration criteria are documented, then they define what must be true before Epic 4 style audible-playback stories can begin, and they do not overclaim unsupported multi-instrument or richer runtime capability.
3. Given this reset affects developer expectations and planning artifacts, when the baseline is finalized, then the relevant documentation and planning guidance reflect the VST-only foundation and deletion rules, and the historical Epic 3 runway is not described as a production-trustworthy playback base.

## Tasks / Subtasks

- [x] Define the explicit automated trust gate for the single-instrument VST-only baseline (AC: 1, 2)
  - [x] Reuse the existing Rust-owned playback seams to define a single authoritative trust gate around one routed instrument, one VST-backed target, and one backend-owned activation path.
  - [x] Add or extend automated checks that prove runtime activation coherence for the supported single-instrument baseline without relying on sampler-era or SCVI-era product-path ownership.
  - [x] Ensure the gate stays transport-readiness-only for this story: it may validate activation, state coherence, and transport-facing readiness, but it must not claim first audible playback.
- [x] Investigate and harden the native editor-host path as part of the trusted single-instrument baseline (AC: 1, 2)
  - [x] Treat plugin-editor stability as part of the baseline, not a cosmetic follow-up: resizing must update the owning host window and the embedded rendered layer coherently, not only the rendered layer.
  - [x] Root-cause the same-session second-open hang from the rack path as a lifecycle or teardown defect, and fix it at the ownership boundary rather than adding retry logic or fake delays.
  - [x] Prove that editor open requests target the intended live plugin instance for a routed single instrument, even when multiple instances of the same plugin bundle exist in the app session.
  - [x] Validate that instance identity, lifecycle, and editor routing remain correct through close, reopen, save, reopen-project, and ordinary app lifetime transitions for supported single-instrument plugins.
  - [x] Keep this work explicitly scoped to single-instrument VSTs; multi-instrument VSTs are not part of the trust claim for this story.
- [x] Prove trust survives a real single-instrument `.soloc` round trip (AC: 1, 2)
  - [x] Add or extend load/save validation that covers one routed single-instrument project being saved and reopened, with portable project intent preserved and runtime readiness re-resolved from current machine-local inventory.
  - [x] Verify degraded reopen behavior stays explicit and recoverable when the route cannot be resolved on the current machine, rather than silently rewriting user intent.
  - [x] Record any migration impact. If persisted semantics change, update `src-tauri/src/migrations/next.rs`; otherwise document why no migration update was required.
- [x] Create the maintainer-facing manual trust-restoration procedure (AC: 1, 2)
  - [x] Document the minimum manual proof flow for one routed instrument: open the app, confirm route and readiness coherence, inspect maintainer-facing diagnostics or logs, and run a save/reopen pass.
  - [x] Make the manual proof an explicit part of the trust gate instead of a nice-to-have note.
  - [x] State clearly that downstream Epic 4 work remains blocked until both automated checks pass and the manual proof flow is completed.
- [x] Rebaseline documentation and planning guidance around the trusted baseline (AC: 2, 3)
  - [x] Update architecture and testing guidance to describe the single-instrument VST-only trust baseline, the no-legacy-ownership rule, and the go or no-go criteria for resuming audible playback work.
  - [x] Update project-context or agent guardrails if this story strengthens implementation rules future playback stories must follow.
  - [x] Update planning artifacts so Epic 4 style audible-playback work is described as blocked on this trust gate and the prior Epic 3 runway is explicitly not treated as a production-trustworthy base.
- [x] Keep the scope bounded to trust restoration rather than feature expansion (AC: 1, 2, 3)
  - [x] Do not introduce new proof-oriented UX whose main purpose is to satisfy the story literally if existing diagnostics, logs, or score-context surfaces can carry the evidence.
  - [x] Do not add any new user-facing app UI for this task unless the need is demonstrated during implementation and explicitly reviewed first.
  - [x] Do not broaden the story into multi-instrument, multi-target, or first-audible-playback behavior.
  - [x] Do not reintroduce sampler-era or SCVI-era fallback ownership, naming, or hidden runtime influence while establishing the baseline.
  - [x] Do not fake plugin-host success conditions. If supported plugins require a real active audio path, processing state, sample-rate agreement, or device configuration, use the existing backend audio setup and app settings path rather than test-only illusions.

## Dev Notes

### Story Intent

Story 1.3 is the explicit go or no-go gate for whether the reset playback foundation is trustworthy enough for downstream audible-playback work. This is not a feature-expansion slice. Its deliverables are: automated trust checks, written gate criteria, documentation and planning updates, and a maintainer-run manual proof procedure.

This story now explicitly includes the single-instrument native editor-host path as part of that trust question. If opening the editor for a routed instrument can mis-size the host window, hang on same-session reopen, lose instance identity, or rely on fake runtime conditions, then the baseline is not trusted.

### Story Foundation

- Epic 1 is a trust-restoration sequence inserted by the approved course correction before first audible playback resumes.
- The story must remain a standalone maintainer story, but it should behave like a hard release gate rather than a soft note in developer guidance.
- The user expectation for this story is explicit: it should validate one routed score instrument through a single-instrument VST instance only, include a real `.soloc` reopen pass, stay transport-readiness-only, and require both automated checks and manual proof before downstream work is unblocked.

### Previous Story Intelligence

- Story 1.1 removed active legacy playback-path ownership and narrowed the supported product path to VST-owned behavior.
- Story 1.2 reasserted Rust-owned playback-domain boundaries, portable `.soloc` intent, and removal of sampler-era or assignment-era contract drift.
- Story 1.2 also identified the main failure modes this story must guard against: hidden duplicate truth, resolved machine-local runtime data leaking into project files, and ambiguous terminology that invites a second ownership model.
- Existing backend seams already center on `PlaybackActivationRuntime` as the runtime readiness bridge. Story 1.3 should verify and gate that seam rather than inventing a parallel trust model.
- Earlier editor-host work already exposed likely failure seams: host-session reservation ordering, close-time teardown ordering, resize callback reentry, and reopen races while teardown is still in flight.
- Existing repo history also shows the editor-host proof seam was intentionally bounded and allowlisted. This story should preserve that honesty, keep scope to supported single-instrument plugins, and avoid expanding compatibility claims just because a workaround appears to function.
- Wider-host evidence from MuseScore, Carla, Ardour, JUCE, and similar projects points to the same conclusion: editor resize defects, second-open hangs, and richer-plugin failures are usually symptoms of one incomplete lifecycle model rather than isolated UI bugs.

### Technical Requirements

- The trust baseline is intentionally limited to a single-instrument VST instance and a single direct target. Do not claim multi-instrument or multi-timbral VST-instance trust, richer runtime capability, or first-audible-playback readiness.
- Automated validation must prove one authoritative VST-only activation path with no active sampler-era or SCVI-era product-path interference.
- Automated and manual validation must include the native editor-host path for supported single-instrument VST instances because editor reopen hangs and mis-sized windows are trust failures, not polish issues.
- Resizing must keep the top-level host window and the embedded plugin view in sync. Resizing only the rendered sublayer is a baseline failure.
- Same-session reopen from the rack path must not hang. Treat this as a likely lifecycle, teardown-order, callback-reentry, or main-thread synchronization defect and fix it at the root.
- Instance isolation is mandatory. Multiple instances of the same plugin bundle may exist; the system must preserve distinct identity for each live instance and open the editor for the exact targeted instance rather than a bundle-level or plugin-name approximation.
- If the editor path or plugin readiness requires real processing activity, sample-rate negotiation, or active audio-device setup, wire that through the existing CPAL-backed backend and app settings path instead of inventing a fake host mode purely to make the UI appear functional.
- Treat VST3 host trust as one ordered lifecycle: host context exposure, component/controller initialization, component-handler and connection-point wiring, bus activation, runtime configuration, view attach, resize flow, close, and reopen must form one coherent model rather than separate local fixes.
- Reopen validation must use a real `.soloc` round trip for one routed instrument and confirm that portable score intent persists while runtime readiness is re-resolved from current machine-local inventory.
- Manual proof must include route/readiness confirmation, maintainer-facing diagnostics or logs, and a save/reopen pass.
- Epic 4 style work remains blocked until both automated checks pass and the manual proof procedure is completed.
- If persisted semantics change, migration impact must be assessed and `src-tauri/src/migrations/next.rs` updated; otherwise the story should record why no migration update was needed.

### Architecture Compliance

- Rust remains the sole authoritative owner of playback-domain truth, including inventory, rack, slot, route, playback-preparation, transport, and persisted playback-domain state.
- The active playback path must be truly VST-owned. Legacy sampler or SCVI seams must not remain accepted product-path owners once equivalent VST seams exist.
- `.soloc` stores portable project intent and playback references only; machine-local discovery, validation, and cache state remain outside project files.
- Validation and trust-restoration criteria must be explicit architectural seams because seam-local tests alone are not enough to claim product trust.
- Score-context versus app-scoped boundaries remain intact. Use the minimum useful surface for evidence; do not add proof-only UI if existing diagnostics are enough.
- The host-lifecycle and instance-identity model must stay backend-owned. Frontend windows or rack rows may request an editor open, but they must not become the authority for which plugin instance is live, which session is closing, or whether cleanup completed.

### Library / Framework Requirements

- Keep FE→BE mutations flowing through the existing Rust-owned commands and `snake_case` payload contracts.
- Keep frontend state limited to consumption of Rust-owned snapshots and transient presentation concerns.
- Keep transport-facing validation aligned with the existing transport-safe architecture; do not introduce event-heavy or UI-owned runtime truth for the sake of the gate.

### File Structure Requirements

- Runtime trust checks should stay close to backend playback seams under `src-tauri/src/playback/`.
- Persistence and reopen validation should live near `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, and `src-tauri/src/migrations/`.
- Frontend evidence consumption should stay in existing state and action seams under `src/services/` and main-window score-context surfaces if any UI evidence is needed.
- Native editor host investigation should stay centered in `src-tauri/src/playback/plugin_editor_poc.rs` and adjacent backend-owned host-compatibility seams rather than spreading host truth into window-local logic.
- Documentation updates should target `docs/architecture.md`, `TESTING.md`, `docs/project-context.md`, and any relevant AI-agent instruction files.
- Planning updates should remain under `_bmad-output/planning-artifacts/`.

### Testing Requirements

- Add or extend automated checks that prove single-instrument VST activation coherence without claiming audible playback.
- Add or extend targeted validation for supported single-instrument editor-host behavior: first open, resize, close, reopen in the same session, and exact-instance reopen when multiple same-bundle instances exist.
- Include at least one validation path that proves editor opening is bound to a specific instance identifier rather than only resource or bundle identity.
- If supported plugins require active processing or audio-device state before the editor behaves correctly, validate that path through the real backend audio setup instead of a mocked editor-only route.
- Add or extend reopen regression coverage for a routed single-instrument `.soloc` round trip.
- Reuse local owning seams instead of building a parallel trust-only framework.
- Validate touched areas with the appropriate repo-standard commands, including at minimum `npm run lint`, `npm run check-types`, and focused Rust tests. Run `cargo clippy` when Rust changes meaningfully and pre-existing warnings do not block it.
- Treat failing automated checks, missing required documentation updates, missing manual proof, or unresolved ownership ambiguity as blockers for downstream Epic 4 work.

### Manual Verification Expectations

- Prepare one routed single-instrument score against the supported VST-only path.
- Open the app and confirm the route and readiness state resolve coherently through existing score-context or diagnostics surfaces.
- Open the native editor for that routed instrument, verify the containing host window resizes coherently with the plugin UI, close it, and reopen it in the same session without hangs or stale-session behavior.
- Where multiple instances of the same plugin bundle exist, verify the rack opens the intended specific instance rather than another matching bundle instance.
- Inspect maintainer-facing diagnostics or logs to confirm the runtime reflects the expected single-instrument activation state without legacy-path interference.
- Save the project, reopen it, and verify the routed intent is preserved while runtime readiness is re-resolved correctly on load.
- Record the go or no-go result so later audible-playback stories inherit an explicit trust verdict rather than tribal knowledge.

### Implementation Risks And Guardrails

- The largest risk is overclaiming trust from seam-local checks alone. This story must produce stronger evidence than local pass/fail signals.
- Another risk is letting the gate drift into first-audible-playback work. This story stops at trusted activation, persistence, and transport-readiness coherence.
- A third risk is adding debugging or proof UI whose real purpose is only to satisfy the acceptance criteria literally. Prefer existing diagnostics and the minimum useful score-context surface.
- A fourth risk is treating the older Epic 3 runway as a trustworthy product base. This story must explicitly correct that framing in planning and documentation.
- A fifth risk is silently preserving fallback ownership or runtime influence from legacy seams while claiming the baseline is VST-only.
- A sixth risk is continuing to iterate locally on the current editor-host seam even if the lifecycle model itself is wrong. Start with a holistic review of session ownership, instance identity, thread boundaries, teardown order, and resize callback flow before patching symptoms.
- A seventh risk is validating against multi-instrument or multi-timbral VST instances and drawing the wrong conclusion about the supported baseline. Keep the trust claim limited to single-instrument VST instances until a later story explicitly expands scope.
- An eighth risk is dismissing richer-plugin failures such as HALion as irrelevant because they are out of scope. They are out of scope for the acceptance claim, but still valid diagnostic evidence that host context support, bus activation, controller wiring, or teardown ordering may be incomplete.

### Relevant Code Seams

- `src-tauri/src/playback/runtime_activation.rs`
- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/plugin_host_compatibility.rs`
- `src-tauri/src/score/playback.rs`
- `src-tauri/src/files.rs`
- `src-tauri/src/soloc.rs`
- `src-tauri/src/migrations/next.rs`
- `src-tauri/src/lib.rs`
- `src/services/actions.ts`
- `src/services/types.ts`
- `src/services/state/`
- `src/windows/main/`

### Documentation Impact

Minimum expected documentation and planning review targets:

- `docs/architecture.md`
- `TESTING.md`
- `docs/project-context.md`
- relevant AI-agent instruction files if trust-gate rules are strengthened
- `_bmad-output/planning-artifacts/prd.md`
- `_bmad-output/planning-artifacts/epics.md`
- any linked sprint or readiness guidance that still implies the prior Epic 3 runway is a trustworthy playback base

### Git Intelligence Summary

- Recent commit direction remains aligned with reset hardening rather than feature expansion: `chore: update bmad`, `feat(1.2): cleanliness check`, `feat(1.1): sampler cleanup`, `feat: course correction`, and `feat(3.8): vst management.`
- The implementation pattern to preserve is incremental cleanup and stronger backend-owned seams, not a new exploratory playback path.

### Project Structure Notes

- The repo’s playback architecture is already rack-first: racks own slot lifecycle, slots own plugin assignment, and instruments only own routes to existing slots.
- `PlaybackRackSlot.target` remains optional, so empty slots are valid persisted state and should surface as explicit attention-needed conditions rather than hidden fallback readiness.
- Current runtime readiness already bridges through backend-owned activation state under `AppContext`; trust restoration should verify that model rather than bypass it.
- Active SCVI workflow ownership is already retired from product-path surfaces, so future playback stories can assume no active SCVI fallback workflow remains.
- Existing editor-host history points to main-thread teardown ordering, host resize callback timing, and lifecycle reservation timing as the most plausible causes of same-session reopen hangs and bad resize behavior.
- The validation posture should mirror real host practice from other apps: bind windows to instance ids, resize the owning host window rather than only the inner surface, and treat plugin activity requirements as real runtime requirements instead of faking them away.
- External host comparisons worth consulting during implementation are now captured in `/memories/repo/plugin-host-open-source-comparators.md`, covering MuseScore, Carla, Ardour, JUCE AudioPluginHost, and Tracktion Engine patterns that map directly onto this story's resize, lifecycle, and instance-identity risks.
- A broader lifecycle checklist for implementation review is captured in `/memories/repo/vst3-host-lifecycle-sanity-check.md`; use it to verify initialization, wiring, runtime activation, view ownership, and teardown ordering before concluding that a resize or reopen defect is isolated.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.3: Establish A Trusted Single-Instrument Reset Baseline]
- [Source: _bmad-output/planning-artifacts/architecture.md#Problem Framing And Architectural Consequences]
- [Source: _bmad-output/planning-artifacts/architecture.md#Important Decisions (Shape Architecture)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Requirements to Structure Mapping]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Handoff]
- [Source: _bmad-output/planning-artifacts/prd.md#Journey 4: Maintainer Validates The Playback Engine As Release-Credible Infrastructure]
- [Source: _bmad-output/planning-artifacts/prd.md#Functional Requirements]
- [Source: _bmad-output/planning-artifacts/prd.md#Non-Functional Requirements]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Playback State And Recovery]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Core User Experience]
- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-03-24.md#Proposal P2: Add a trust-restoration gate before first audible playback]
- [Source: _bmad-output/planning-artifacts/sprint-change-proposal-2026-03-24.md#Approval And Handoff]
- [Source: docs/project-context.md#Critical Implementation Rules]
- [Source: /memories/repo/playback-rack-first-model.md]
- [Source: /memories/repo/scvi-retirement-verification.md]
- [Source: /memories/repo/same-session-playback-activation-story-3-8.md]
- [Source: /memories/repo/plugin-editor-host-session-lifecycle.md]
- [Source: /memories/repo/plugin-editor-poc-live-vst3-host.md]
- [Source: /memories/repo/plugin-host-bounded-allowlist.md]
- [Source: /memories/repo/plugin-host-open-source-comparators.md]
- [Source: /memories/repo/vst3-host-lifecycle-sanity-check.md]
- [Source: musescore/MuseScore `src/framework/vst/widgets/vstviewdialog_qwidget.cpp` - top-level window resized from `IPlugFrame::resizeView`, instance-bound editor open]
- [Source: musescore/MuseScore `src/framework/vst/internal/vstactionscontroller.cpp` - instrument editor actions routed by concrete instance id]
- [Source: musescore/MuseScore `src/framework/vst/internal/vstplugininstance.cpp` - per-instance ids and main-thread view creation]
- [Source: musescore/MuseScore `src/framework/vst/internal/vstaudioclient.cpp` - explicit plugin activity enable/disable rather than fake editor-only readiness]
- [Source: falkTX/Carla `source/backend/plugin/CarlaPluginVST3.cpp` - resize-state guards, capability-driven host resize, concrete plugin ids, real processing setup]
- [Source: Ardour/ardour `libs/ardour/vst3_plugin.cc` and `gtk2_ardour/vst3_*_plugin_ui.cc` - one cached view per instance, explicit `removed()`/`setFrame(0)` close path, top-level window resize from host callbacks]
- [Source: juce-framework/JUCE `extras/AudioPluginHost/Source/UI/PluginWindow.h` and `modules/juce_audio_plugin_client/juce_audio_plugin_client_VST3.cpp` - editor child-bounds monitoring, host-window resize ownership, resize reentrancy guards]
- [Source: tracktion/tracktion_engine `modules/tracktion_engine/plugins/tracktion_PluginWindowState.*` - per-plugin window state ownership and explicit shutdown-time hide/delete behavior]

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story creation workflow analysis across epics, architecture, PRD, UX, sprint change proposal, repo memory, prior story, and recent git history.
- User expectation pass captured story-shaping decisions before drafting: standalone maintainer story, automated plus manual gate, real `.soloc` reopen proof, transport-readiness-only scope, and explicit doc and planning rebaseline.
- Implemented `PlaybackActivationRuntime::single_instrument_trust_gate` as the authoritative backend trust seam for one routed direct plugin route, emitted its transport-readiness result through general app state, and added focused Rust coverage for ready, degraded, unsupported-route, and multi-route cases.
- Validation for task 1: `cargo test runtime_activation -- --nocapture`, `npm run check-types`, `npm run lint`.
- Hardened the bounded native editor-host seam so rack-originated editor opens resolve by slot identity plus bundle path, preserving exact-instance routing when multiple same-bundle plugins exist and keeping the supported target scope limited to single-instrument VST flows.
- Extended `.soloc` reopen coverage to assert that portable route intent survives save and reload while the backend trust gate re-resolves current-machine runtime readiness, including explicit degraded reopen behavior when inventory resolution fails.
- Updated architecture, testing, project context, planning artifacts, and agent instructions so Story 1.3 is the explicit prerequisite gate before Epic 4 audible-playback work.
- Full validation pass: `npm run test:js` -> 97 files, 1164 tests passed; `npm run test:rust` -> 1184 tests passed; `cargo clippy --lib -- -D warnings` remains blocked by pre-existing dead-code warnings in `src-tauri/src/playback/audio.rs` and `src-tauri/src/playback/mixer.rs` unrelated to Story 1.3.
- Clarified active story and planning terminology so "single-instrument" explicitly means one playable instrument per VST instance, not HALion-style multi-slot or multi-part hosting.
- Reworked the native editor-host lifecycle around a detached live session so same-session reopen reattaches the existing slot-owned VST instance instead of rebuilding a fresh empty editor stack, and aligned resize handling with open-source host patterns by resizing the owning macOS host window on the main thread while suppressing resize feedback loops.
- Focused validation after the lifecycle repair: `cd src-tauri && cargo test --lib plugin_editor_poc -- --nocapture` -> 35 passed, 0 failed. Maintainer manual proof for BBCSO-style resize and same-session reopen remains required before the story can be treated as the final go verdict for Epic 4.
- Extended the detached-session lifecycle to support exact-instance switching across same-bundle slots: when slot A has only a preserved detached session and slot B later opens the same bundle, the backend now tears down slot A's detached session before reserving slot B, instead of rejecting the second instance behind stale detached ownership.
- Focused validation after the detached target-switch repair: `CARGO_TARGET_DIR=/tmp/compose-plugin-editor-poc-target cargo test --lib plugin_editor_poc -- --nocapture` -> 37 passed, 0 failed.
- Final story review clarified the persistence boundary that remains required for follow-up work: `.soloc` should keep only the portable intent needed to rehydrate slot-owned live plugin sessions, including slot-to-plugin relationships and restorable plugin configuration, while the live runtime session itself remains backend-owned and rebuilt on assignment or load.

### Completion Notes List

- Created Story 1.3 context with explicit trust-gate deliverables and bounded scope.
- Reflected user direction to ask and resolve expectations before drafting rather than assuming the gate shape.
- Scoped the story to trusted single-instrument VST activation and reopen coherence, not first audible playback.
- Embedded the required documentation and planning impact as first-class story work.
- Incorporated the newly surfaced host-stability blockers directly into the story: host-window resize ownership, same-session reopen deadlock risk, exact-instance editor targeting, real-runtime prerequisites, single-instrument-only support posture, and no-new-UI guardrails without explicit review.
- Completed task 1 by adding an explicit backend-owned transport-readiness trust gate for the single-instrument VST-only baseline and exposing its readiness state in the general app snapshot without adding new proof-only UI.
- Added runtime-activation tests proving the gate passes only for one ready direct route and blocks degraded inventory, unsupported route shapes, and multiple routed instruments.
- Completed task 2 by making rack editor-open requests slot-aware at the backend boundary so supported plugin editors reopen against the intended live instance instead of a bundle-level approximation.
- Completed task 3 by extending the existing load/save seam to prove trust survives a real single-instrument `.soloc` round trip, including explicit degraded reopen behavior when inventory cannot resolve the saved route.
- Recorded that no persisted payload semantics changed for this story, so no `src-tauri/src/migrations/next.rs` update was required.
- Completed task 4 by documenting the maintainer-facing manual trust-restoration procedure and making it an explicit prerequisite for Epic 4 work alongside the automated gate.
- Completed task 5 by rebaselining architecture, testing, project context, PRD, epics, and agent instructions around the single-instrument trust gate and the requirement to delete legacy playback ownership.
- Kept the implementation bounded to trust restoration: no new proof-only UI, no multi-instrument scope expansion, and no reintroduction of sampler-era or SCVI-era runtime ownership.
- Tightened the active documentation language so Story 1.3's scope now explicitly distinguishes single-instrument VST instances such as BBCSO from multi-timbral VST instances such as HALion.
- Rooted the editor-host repair in comparator research from MuseScore, Carla, Ardour, JUCE AudioPluginHost, and Tracktion: preserve instance-owned backend session lifetime across window close or reopen, make the top-level host window own resize, and guard plugin-driven resize callbacks against host-plugin feedback loops.
- Extended the exact-instance routing repair so a preserved detached session from one slot no longer blocks opening another slot that hosts the same plugin bundle after the first window has closed.
- Recorded the discovered follow-up seam more precisely: the missing architectural piece is not persisting live runtime state in `.soloc`, but using portable saved intent to rehydrate slot-owned live plugin sessions before playback or editor-open.
- Story 1.3 is closed as done because it completed the intended trust-restoration baseline and surfaced the deeper runtime-ownership gap honestly. Epic 4 unblock proof now belongs to the approved follow-up Stories 1.4 through 1.6 on the corrected slot-owned-session architecture.

### File List

- \_bmad-output/implementation-artifacts/1-3-establish-a-trusted-single-instrument-reset-baseline.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- src-tauri/src/playback/runtime_activation.rs
- src-tauri/src/lib.rs
- src/services/types.ts
- src/services/state/context.tsx
- src-tauri/src/playback/plugin_editor_poc.rs
- src/services/actions.ts
- src/services/actions.test.ts
- src/windows/main/screens/sequence/components/playback-rack-panel.tsx
- src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx
- src-tauri/src/files.rs
- docs/architecture.md
- TESTING.md
- docs/project-context.md
- \_bmad-output/planning-artifacts/epics.md
- \_bmad-output/planning-artifacts/architecture.md
- \_bmad-output/planning-artifacts/prd.md
- .github/copilot-instructions.md
