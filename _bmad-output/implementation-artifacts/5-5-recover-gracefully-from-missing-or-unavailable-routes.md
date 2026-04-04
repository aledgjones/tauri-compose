# Story 5.5: Recover Gracefully From Missing Or Unavailable Routes

Status: done

## Story

As a composer,
I want missing plugins or unavailable routes to degrade into recoverable states,
so that I can continue working even when playback assets are unavailable.

## Acceptance Criteria

1. Given Story 5.0 established a supported audible transport playback path, and a previously saved or expected route cannot currently resolve, when Compose restores or evaluates playback state, then the route degrades to an explicit recoverable non-ready state, and the score remains usable.
2. Given some routes are recoverable and others are ready, when I continue working in the project, then ready routes keep working through the supported playback path, and unresolved routes do not block score access or force hidden substitutions.
3. Given recovery behavior is part of user trust, when missing-plugin and unavailable-route scenarios are surfaced, then Compose preserves user intent and offers a clear recovery path, and it avoids silent fallback or corrupted playback truth.

## Tasks / Subtasks

- [x] Task 1: Audit the current missing-route recovery contract across restore, runtime evaluation, and score-context UI before changing behavior (AC: 1, 2, 3)
  - [x] Confirm how `PlaybackActivationRuntime::resolved_playback_route_by_instrument_id(...)` currently distinguishes `Unrouted` from `NeedsAttention`, especially for missing slot references, empty slots, and unavailable inventory.
  - [x] Confirm project load and runtime refresh preserve portable rack and route intent while recomputing machine-local readiness from current inventory instead of rewriting score truth.
  - [x] Confirm the current score-context recovery path is the compact route picker and rack panel, not a new banner, modal, or app-scoped recovery workflow.

- [x] Task 2: Refine backend-owned degradation semantics only where the audit shows trust gaps (AC: 1, 2, 3)
  - [x] Prefer extending `src-tauri/src/playback/runtime_activation.rs`, `src-tauri/src/files.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/playback/preparation.rs`, or adjacent score playback seams over introducing a parallel recovery service.
  - [x] Preserve user intent in score data when a plugin is missing or unavailable: rack slots, slot targets, and instrument routes stay truthful, while runtime readiness degrades honestly.
  - [x] Keep ready routes playable through the supported transport path even when other routes are non-ready.
  - [x] Do not introduce hidden plugin substitutions, legacy fallback playback, frontend-owned route truth, or transport-owned compatibility residue.

- [x] Task 3: Sharpen the existing compact score-context recovery path without adding UI sprawl (AC: 2, 3)
  - [x] Reuse the existing Sequence instrument overview row and playback rack panel so broken assigned routes surface as one compact inline warning on the route control and a visible repair path through current slot and plugin surfaces.
  - [x] Keep explicit unrouted and empty-slot states visually neutral (`--` or `N. --`) rather than warning-heavy, while assigned-but-broken routes remain identifiable as needing repair.
  - [x] Ensure recovery stays musician-facing: rerouting or reassigning through the current picker and rack panel remains possible after degradation, without forcing plugin-manager-first repair or adding separate “open rack” controls.

- [x] Task 4: Extend regression proof for mixed ready and non-ready route scenarios (AC: 1, 2, 3)
  - [x] Add or extend Rust tests near `src-tauri/src/playback/runtime_activation.rs`, `src-tauri/src/files.rs`, and `src-tauri/src/playback/preparation.rs` to prove missing-plugin or unavailable-inventory cases preserve persisted intent while degrading runtime truth honestly.
  - [x] Add or extend transport or preparation tests to prove supported playback keeps ready routes active while non-ready routes remain non-blocking and never trigger hidden substitution.
  - [x] Add or extend frontend tests near `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` and `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` to prove the compact recovery semantics remain stable.

- [x] Task 5: Document the recovery contract and score-context UX boundaries if implementation clarifies them (AC: 3)
  - [x] Update developer-facing docs when the missing-route recovery contract, route-state semantics, or score-context repair path becomes clearer during implementation.
  - [x] Record explicitly that 5.5 preserves user intent and supports repair, but does not resume missing routes through fallback playback or broaden into general plugin-management redesign.

## Dev Notes

### Story Intent

- Story 5.4 proved the valid-plugin reopen path and intentionally stopped short of full missing-plugin recovery UX.
- Story 5.5 owns the next layer: make degraded routes trustworthy and recoverable without breaking score access, transport trust, or rack-first ownership.
- The likely happy path is seam refinement plus stronger proof, not a new workflow. The repo already contains route degradation states, compact score-context route warnings, and rack-panel repair affordances.

### Previous Story Intelligence

- Story 5.4 confirmed that `.soloc` files already preserve portable rack and route intent while runtime readiness is recomputed from machine-local inventory on load.
- Story 5.4 also documented that missing inventory currently degrades runtime truth to `NeedsAttention` without rewriting saved intent. Story 5.5 should build on that seam instead of introducing a second recovery model.
- Story 5.3 kept transport resets and active-playback state backend-owned. Story 5.5 must preserve that discipline so degraded routes do not reintroduce transport fallback logic or frontend-owned readiness probes.
- Recent implementation history kept Epic 5 work concentrated in backend playback seams plus compact Sequence surfaces. Follow the same pattern here.

### Existing Seams Already Doing Important Work

- `src-tauri/src/playback/runtime_activation.rs`
  - `resolved_playback_route_by_instrument_id(...)` already projects `Unrouted`, `Ready`, and `NeedsAttention` from score truth plus live runtime state.
  - Existing tests already prove that missing slot references normalize to `Unrouted`, unavailable inventory degrades previously ready routes to `NeedsAttention`, and stale live runtime no longer reports false readiness.
- `src-tauri/src/playback/preparation.rs`
  - `prepare_supported_playback(...)` already keeps no-route scenarios non-blocking and preserves ready targets when other instruments are unrouted.
  - This is the seam that should prove mixed ready and degraded routes remain transport-safe.
- `src-tauri/src/files.rs`
  - Existing load and round-trip tests already prove persisted playback intent survives while runtime truth degrades honestly against unavailable inventory.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - The compact route picker already reuses backend-owned route truth, keeps unrouted instruments visually neutral, and keeps assigned-route recovery inline instead of spawning a separate repair control.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Warning chrome is intentionally narrow: broken assigned routes get one compact warning, while unrouted and empty-slot states stay neutral.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - The score-context rack panel already preserves the current recovery path for empty slots, unavailable selected plugins, and plugin reassignment without pushing users into an app-scoped repair workflow.

### This Story Owns

- Tightening the backend-owned semantics for missing-plugin and unavailable-route degradation when current score intent cannot resolve to a ready runtime route.
- Ensuring mixed ready and degraded routes keep the supported playback path trustworthy and non-blocking for the ready subset.
- Making the existing score-context route picker and rack panel sufficient, clear recovery surfaces for degraded assigned routes.
- Regression proof that persisted intent remains intact while runtime truth and transport preparation degrade honestly.

### This Story Does Not Own

- Changing the rack-first ownership model: racks still own slot lifecycle, slots still own plugin assignment or unassignment, and instruments still own routes to existing slots.
- Persisting machine-local inventory, availability, cache truth, or live runtime host state into `.soloc` files.
- Adding a new modal, banner, inspector, dedicated recovery pane, or app-scoped plugin-management workflow just to literalize recovery acceptance criteria.
- Reintroducing sampler-era or SCVI-era fallback playback, hidden plugin substitution, or any automatic rerouting that mutates score truth behind the user’s back.
- Broad plugin compatibility expansion, plugin-manager redesign, or future Epic 7 validation work.

### Current Relevant Seams

- `src-tauri/src/playback/runtime_activation.rs`
  - Canonical backend seam for resolved route truth and degradation semantics.
- `src-tauri/src/playback/preparation.rs`
  - Canonical seam for proving transport preparation remains honest and non-blocking when routes are mixed.
- `src-tauri/src/files.rs`
  - Owns reopen and restore proof around persisted intent versus runtime degradation.
- `src-tauri/src/lib.rs`
  - Owns backend state publication to the frontend and should stay the only path for refreshed route truth.
- `src-tauri/src/score/playback.rs`
  - Owns the persisted route-status model and enum contract shared with TypeScript.
- `src/services/types.ts`
  - Must stay aligned with Rust enum and payload changes if route-status shape or fields change.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Canonical compact score-context repair surface.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Owns the intentionally narrow warning semantics for score-context playback recovery.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Owns slot-level reassignment and unavailable-plugin display within the current score context.

### Implementation Guidance

- Start with the seam audit. Much of Story 5.5 may already exist in partial form across runtime activation, preparation, and the current Sequence route controls.
- Preserve a strict distinction between:
  - explicit `Unrouted` states, which remain neutral and selectable;
  - explicit broken assigned-route states, which should remain recoverable and warning-worthy;
  - machine-local plugin availability, which must not rewrite the score’s saved intent.
- Prefer refining the backend route-status projection before changing frontend presentation. The UI should continue to be a narrow summary of backend truth, not the place where recovery logic is invented.
- Keep the repair path inside current score-context surfaces. The route picker and rack panel should remain enough for MVP recovery unless the audit proves a concrete product-facing failure that cannot be solved there.
- Preserve supported playback for ready routes. A degraded route must not block score access, transport start for supported routes, or same-project work on unaffected instruments.
- If a route references a missing slot, keep the existing normalization to `Unrouted` unless a better backend-owned distinction is required and justified. Do not broaden warning chrome just to expose every backend nuance.
- If a route still points to a slot whose plugin is unavailable, preserve the slot target and route intent, degrade runtime readiness honestly, and keep the current repair path obvious.
- Keep MCP, automation, and frontend actions reusing backend seams rather than adding a recovery-only code path.

### Risks And Regression Traps

- It is easy to over-correct by silently clearing routes or mutating saved slot targets when inventory is unavailable. That would destroy the preserved-intent contract Story 5.4 just established.
- It is equally easy to leave stale runtime activation in place and show false readiness after availability changes. Any backend refinement must continue clearing stale live-route assumptions.
- Recovery UX can sprawl quickly. The existing project rules require compact score-context controls, not AC-driven banners and duplicate buttons.
- Mixing ready and degraded routes is the main trust trap: unsupported routes must stay non-ready without blocking the ready subset or forcing hidden substitution.
- Route-state semantics cross Rust and TypeScript boundaries. Any enum or payload change must stay numerically and structurally aligned.

### Testing Requirements

- Extend Rust tests near `src-tauri/src/playback/runtime_activation.rs` for:
  - unavailable-inventory degradation of a previously assigned route;
  - missing slot reference behavior versus assigned-but-broken route behavior;
  - stale runtime refresh scenarios that must not report false readiness.
- Extend Rust tests near `src-tauri/src/playback/preparation.rs` and, if necessary, `src-tauri/src/playback/transport.rs` for:
  - mixed ready and degraded routes where supported playback remains available for the ready subset;
  - explicit proof that degraded routes do not trigger hidden fallback targets or block the supported transport path incorrectly.
- Extend save and reopen regressions near `src-tauri/src/files.rs` only as needed to prove persisted intent remains intact while runtime truth degrades honestly on load.
- Extend frontend tests near `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` for:
  - compact warning behavior on assigned-but-broken routes;
  - neutral treatment of unrouted and empty-slot states;
  - continued rerouting ability after degradation.
- Extend frontend tests near `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` for:
  - unavailable assigned plugin display;
  - reassignment and repair through current slot controls;
  - no new recovery-only UI chrome.
- Recommended validation commands:
  - `cargo test runtime_activation --manifest-path src-tauri/Cargo.toml`
  - `cargo test prepare_supported_playback --manifest-path src-tauri/Cargo.toml`
  - `cargo test files --manifest-path src-tauri/Cargo.toml`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `npm run test:js`
  - `npm run lint`
  - `npm run check-types`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

### Documentation Impact

- Update docs if implementation clarifies the user-facing meaning of `Unrouted` versus `NeedsAttention`, or clarifies how repair is expected to happen through current score-context surfaces.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `TESTING.md`
- If route-state semantics or persisted recovery expectations change, ensure the documentation stays explicit that project files keep intent while runtime truth is machine-local and recoverable.

### Architecture Compliance

- Rust remains the sole owner of route truth, runtime readiness, and persisted playback intent.
- `.soloc` files must continue to store portable rack and route intent only, never machine-local availability truth or live runtime ownership.
- The rack-first model remains authoritative: racks own slots, slots own targets, instruments own routes.
- User-facing warning UI must stay narrower than backend route truth. No badge for plain unrouted or empty-slot states; compact warning only for broken assigned routes that need repair.
- Supported transport playback remains backend-owned and snapshot-based. Story 5.5 must not introduce per-tick readiness probing, frontend transport ownership, or fallback playback.

### Library And Framework Requirements

- Stay on the repo baseline: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- Reuse existing backend seams and the current `actions` pattern; no new state-management or persistence library belongs here.
- If shared playback-route payloads change, update the Rust and TypeScript mirrors together.

### File Structure Notes

- Backend recovery logic belongs under `src-tauri/src/playback/`, `src-tauri/src/files.rs`, and adjacent score playback seams.
- Frontend recovery presentation belongs in existing Sequence score-context components, not in a new window or top-level warning system.
- Tests should remain seam-local and component-local. Do not invent a cross-cutting recovery harness if existing runtime, preparation, files, and Sequence tests can prove the contract.

### Git Intelligence Summary

- Recent commit `07334e34` (`feat(5.4): reopen`) strengthened `src-tauri/src/files.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/playback/preparation.rs`, and adjacent docs. Story 5.5 should extend that same reopen and runtime contract rather than bypassing it.
- Recent commit `2abd4919` (`feat(5.3): audio refresh`) kept transport and runtime coherence in backend playback seams and updated docs in the same pass. That is the correct pattern for 5.5 as well.
- Recent commit `dcd661b1` (`feat(5.2a): clock fixes`) shows Epic 5 transport trust work staying concentrated in `transport.rs`, `audio.rs`, and `preparation.rs`. Recovery work must preserve those backend-owned timing boundaries.

### Latest Technical Information

- No external version-driven change is required for this story beyond the repo baselines already in use. The critical technical constraint is internal: `prepare_supported_playback(...)` and backend-owned route resolution remain the canonical transport snapshot, so recovery semantics must flow through them rather than around them.
- Current repo types still expose only three frontend-facing route states: `Unrouted`, `Ready`, and `NeedsAttention`. Any attempt to broaden surface semantics should be justified carefully and kept aligned across Rust and TypeScript.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 5 scope and Story 5.5 acceptance criteria.
- `_bmad-output/planning-artifacts/prd.md`
  - FR34, FR35, and the resilience and recovery requirements for reopen and missing-plugin workflows.
- `_bmad-output/planning-artifacts/architecture.md`
  - Rust-owned playback truth, rack-first ownership, portable persistence boundaries, and compact score-context UX rules.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Recovery should preserve confidence, stay score-first, and avoid host-complexity leakage or UI sprawl.
- `_bmad-output/implementation-artifacts/5-4-restore-playback-state-on-project-reopen.md`
  - Immediate previous-story boundaries and reopen proof that 5.5 must extend without undoing.
- `/memories/repo/playback-rack-first-model.md`
  - Rack-first ownership reminders and empty-slot semantics.
- `/memories/repo/playback-readiness-ui-semantics.md`
  - Narrow warning semantics for score-context playback recovery.
- `src-tauri/src/playback/runtime_activation.rs`
  - Current route degradation logic and existing regression coverage.
- `src-tauri/src/playback/preparation.rs`
  - Mixed ready and unrouted preparation behavior.
- `src-tauri/src/files.rs`
  - Persisted-intent versus runtime-degradation reopen regressions.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Compact route picker and repair path.
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts`
  - Current compact warning presentation rules.
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
  - Slot-level plugin reassignment and unavailable-plugin display.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-04: Story prepared after reviewing Epic 5 planning, PRD resilience requirements, architecture ownership rules, UX recovery constraints, Story 5.4 handoff notes, current route-resolution and preparation seams, existing Sequence recovery components, and recent Epic 5 commits.
- 2026-04-04: Analysis confirmed that most required recovery primitives already exist, so Story 5.5 should bias toward seam refinement and regression proof rather than workflow invention.
- 2026-04-04: Audited runtime activation, playback preparation, file restore, and Sequence score-context seams; confirmed the shipped contract already preserves intent, keeps ready routes playable, and keeps recovery inside the existing picker and rack panel.
- 2026-04-04: Added mixed ready/degraded restore proof in `src-tauri/src/files.rs` and repair-path regressions in the existing Sequence route picker and rack-panel test suites; no production seam changes were required.
- 2026-04-04: Validation passed with `npm run lint`, `npm run check-types`, `npm run test:js`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`.

### Completion Notes List

- Story context created and sprint status updated to ready-for-dev.
- Audit result: existing backend and score-context seams already satisfied the intended 5.5 recovery contract, so implementation stayed focused on regression proof instead of introducing new recovery logic.
- Added a restore regression proving persisted mixed ready and degraded routes preserve score intent while leaving the ready subset playable through the supported preparation path.
- Added Sequence regressions proving unavailable assigned routes remain repairable through the current route picker and current rack slot controls without new recovery UI.
- Revalidated current documentation guidance and confirmed no doc text changes were required because the repo already states the 5.5 contract explicitly.
- Code review found no blocking issues in the 5.5 delta; story advanced from review to done.

### File List

- `_bmad-output/implementation-artifacts/5-5-recover-gracefully-from-missing-or-unavailable-routes.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/files.rs`
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
- `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`

## Change Log

- 2026-04-04: Added regression coverage for mixed ready/degraded route restore behavior and current-surface repair flows; story advanced to review with full lint, JS test, Rust test, and clippy validation passing.
- 2026-04-04: Code review found no blocking issues; story advanced from review to done.
