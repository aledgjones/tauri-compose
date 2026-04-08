# Story 7.1: Define The Supported Plugin And Scenario Baseline

Status: done

## Story

As a maintainer,
I want an explicit known-good plugin and scenario baseline,
so that Compose has a concrete definition of what supported and shippable means for MVP playback.

## Acceptance Criteria

1. Given MVP support is intentionally best-effort and bounded, when the support baseline is defined, then it identifies the representative plugin set, scenario set, and compatibility expectations used for release decisions, and it does not claim blanket support for out-of-spec or unvalidated plugins.
2. Given the product must be validated against representative real-world libraries, when the baseline is documented, then it includes the user's real plugin library coverage expectations, including the relevant BBC Spitfire libraries, and it distinguishes known-good support from future compatibility expansion.
3. Given release claims need explicit evidence, when the baseline is approved, then the scenarios and supported subset are specific enough to drive repeatable validation, and they can be used to reject unsupported overclaims.

## Tasks / Subtasks

- [x] Task 1: Establish one canonical maintainer-facing support-baseline artifact and vocabulary (AC: 1, 2, 3)
  - [x] Create or designate a single source-of-truth baseline document under `docs/` rather than scattering the supported subset across unrelated docs.
  - [x] Define the baseline vocabulary explicitly: `known-good`, `blocked/unsupported for the bounded seam`, `discovered but unvalidated`, and `future compatibility expansion`.
  - [x] Keep the baseline scoped to macOS, VST3, rack-first routed playback, and the current MVP transport path.

- [x] Task 2: Define the representative plugin subset and compatibility posture from current product truth (AC: 1, 2, 3)
  - [x] Enumerate the concrete known-good control bundles already evidenced in repo docs and code, not generic plugin families alone.
  - [x] Reconcile the planning-language expectation of “BBC Spitfire libraries” with a concrete bundle list or explicit coverage note that maintainers can actually validate.
  - [x] Record the current blocked or unsupported editor-host targets from the owning backend seam without broadening them into a blanket product-support statement.
  - [x] Make the boundary explicit: discovered bundles outside the validated subset remain outside the MVP support claim until later stories validate them.

- [x] Task 3: Define the representative scenario baseline for later validation stories (AC: 1, 3)
  - [x] Specify the discovery and rescan scenarios that Story 7.2 must run against cached inventory and manual refresh flows.
  - [x] Specify the bounded native editor-host scenarios that Story 7.2 must run against known-good and blocked bundles, including open, focus, resize, close, and same-session reopen behavior.
  - [x] Specify the transport-trust scenarios that Story 7.3 must run against routed playback, including play, pause, resume, seek, and bounded edit-during-playback behavior.
  - [x] For each scenario, capture the owner seam, expected evidence source, and pass/fail meaning so the baseline is repeatable instead of anecdotal.

- [x] Task 4: Align maintainer and developer docs around the canonical baseline without creating a second runtime truth (AC: 1, 2, 3)
  - [x] Update the existing maintainer-facing docs to reference the canonical baseline instead of duplicating drifting support language.
  - [x] Keep support-baseline language aligned with current backend-owned evidence such as `editor_host_validation`, `supported_targets`, and the bounded host policy seams.
  - [x] If implementation introduces any code-backed metadata or labels to keep the baseline honest, keep that change narrow to the owning seam and back it with seam-local tests.

## Dev Notes

### Story Intent

- Story 7.1 is the definition pass for Epic 7, not the evidence-collection pass.
- The implementation should give later stories a concrete release baseline they can execute against, not a vague compatibility narrative.
- The output should let maintainers say exactly what is supported, what is only discovered, what is blocked for the bounded seam, and which scenarios are release-gating.

### User Model Clarified

- The primary user here is a maintainer deciding whether Compose has a shippable MVP playback boundary.
- They need a concrete yes/no release baseline, not a broad marketing-style compatibility claim.
- “User's real plugin library” and “BBC Spitfire libraries” must become concrete enough to validate. Generic family names alone are not sufficient for repeatable execution.

### Cross-Epic Intelligence

- Stories 1.3 through 1.6 established the trust-restoration gate for one routed single-instrument VST instance, durable reopen behavior, and bounded editor-host lifecycle stability. Story 7.1 must treat that as completed baseline evidence, not relitigate the foundation.
- Story 6.4 extended the supported transport path with initial linked Expression Map behavior on the backend-owned preparation plus transport seam. Story 7.1 should include that current supported path in the baseline without over-claiming broader realism or compatibility.
- Epic 7 now exists because the product has slices of proof spread across discovery, host behavior, transport, and expression-map work. This story must consolidate those slices into one explicit support boundary.

### This Story Owns

- The canonical supported-subset definition for MVP playback.
- The representative plugin-library baseline that later validation stories must exercise.
- The representative scenario matrix for discovery, host behavior, and transport-trust validation.
- The maintainer-facing documentation structure that future Epic 7 stories can extend.

### This Story Does Not Own

- Running the actual discovery, host, or transport validation matrix end to end. That belongs to Stories 7.2 and 7.3.
- Broadening plugin compatibility claims beyond the currently evidenced subset.
- Adding new playback UI, new product-facing support indicators, or a frontend-owned support matrix just to satisfy the story.
- Reworking host policy or transport logic unless a narrow owner-seam change is required to keep documentation aligned with already-shipped truth.

### Current Relevant Seams

- `docs/development-guide.md`
  - Already contains the current bounded host subset, blocked bundles, and a maintainer manual validation checklist.
- `TESTING.md`
  - Already contains host validation guidance, transport/performance scenario guidance, and the single-instrument trust-gate framing.
- `docs/performance-baselines.md`
  - Already records the representative render-performance baseline and should remain separate from playback-support claims.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Current owner of the bounded native editor-host unsupported list. Baseline docs must follow this seam's live truth.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Distinguishes bundle availability from `editor_host_validation`; crucial for documenting “discovered but not supported” versus “blocked.”
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Owns phase-aware editor-host validation evidence such as `policy_gate`, `module_load`, `component_create`, `controller_initialize`, `view_create`, and `view_attach`.
- `src-tauri/src/playback/preparation.rs`
  - Owns `supported_targets`, which should anchor the definition of which routed playback targets count as currently supported.
- `src-tauri/src/playback/transport.rs`
  - Owns the actual play/pause/resume/seek scenario seams that Story 7.3 will validate.
- `src/windows/plugin-manager/` and `src/windows/main/components/plugins.tsx`
  - Existing UI consumers of backend-owned plugin inventory and bounded-host status; non-target unless baseline wording must be synced to already-exposed truth.

### Implementation Guidance

- Prefer a dedicated maintainer-facing baseline doc under `docs/` as the canonical artifact for Epic 7 rather than copying long support matrices into both `docs/development-guide.md` and `TESTING.md`.
- Link the surrounding docs back to that canonical artifact so later stories update one baseline instead of several drifting summaries.
- Keep the baseline structured around three axes:
  - concrete plugin subset
  - concrete scenario subset
  - compatibility-language boundary
- Plugin subset guidance:
  - Start from current repo truth. Today that includes concrete known-good examples in `docs/development-guide.md` and current blocked editor-host targets in `src-tauri/src/playback/plugin_host_compatibility.rs`.
  - Reconcile the planning phrase “BBC Spitfire libraries” with the actual bundles that appear in current evidence, such as `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3`.
  - Do not hide the difference between “bundle discovered and available” and “validated for the bounded host seam.”
- Scenario subset guidance:
  - Discovery/cache reuse and manual rescan.
  - Route readiness plus reopen recovery.
  - Native editor-host open, focus, resize, close, and same-session reopen.
  - Transport play, pause, resume, seek, and bounded edit-during-playback behavior.
- For each scenario, document:
  - the owner seam
  - the concrete setup or prerequisite
  - the evidence to capture
  - what counts as pass, fail, or unresolved risk
- Keep baseline language backend-truth-driven. Do not create a second support taxonomy in React or a marketing-facing support list that diverges from backend-owned evidence.
- Treat repo truth as authoritative over stale notes. One concrete trap here is MT-PowerDrumKit: current repository code still blocks it in the bounded host seam, so the baseline must document the current code path rather than older contradictory notes.

### Risks And Regression Traps

- Writing a vague prose summary that is too soft to drive later validation.
- Treating “spec-aligned best effort” as an excuse for broad unsupported claims.
- Leaving “BBC Spitfire libraries” at a family-name level instead of naming concrete validation targets.
- Mixing together three different states: discovered, validated, and blocked.
- Letting `docs/development-guide.md`, `TESTING.md`, and any new baseline artifact drift apart.
- Treating bounded native editor-host success as evidence of transport trust, or vice versa.
- Adding runtime/UI surfaces whose only purpose is to prove acceptance criteria literally instead of producing a reusable maintainer artifact.

### Testing Requirements

- If the implementation is documentation-only, validate that every stated plugin or scenario maps cleanly to an existing owner seam and current repo truth.
- If any code-backed support metadata or labels are added, keep tests seam-local to the owner:
  - `src-tauri/src/playback/plugin_host_compatibility.rs`
  - `src-tauri/src/playback/plugin_inventory.rs`
  - `src-tauri/src/playback/plugin_editor_poc.rs`
  - `src-tauri/src/playback/preparation.rs`
  - `src-tauri/src/playback/transport.rs`
- Recommended validation commands if code or tests change:
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_inventory --lib`
  - `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib`
  - `cargo test --manifest-path src-tauri/Cargo.toml transport_ --lib`
  - `npm run check-types`
  - `npm run lint`

### Documentation Impact

- Add or update one canonical Epic 7 support-baseline artifact under `docs/`.
- Update `docs/development-guide.md` to point at the canonical baseline and keep the current host-validation subset wording aligned.
- Update `TESTING.md` so the representative validation scenarios align with the same baseline artifact.
- Update `docs/index.md` if a new doc is added under `docs/`.
- If Story 7.1 establishes new required agent or maintainer validation rules rather than one-off prose, update project instructions such as `docs/project-context.md` or `.github/copilot-instructions.md` only where the new rule is durable and repo-wide.

### Architecture Compliance

- Backend ownership remains unchanged: this story defines a release baseline; it does not move support truth into the frontend.
- `.soloc` portability rules remain unchanged. Machine-local discovery, validation, and host evidence stay outside project files.
- The supported baseline must stay explicitly macOS-only and VST3-only for MVP.
- Best-effort support means spec-aligned and evidence-backed, not universal.

### Library And Framework Requirements

- Stay on the existing repo baselines documented in project context: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- Do not add dependencies for this story unless a narrowly justified documentation or validation-format need emerges.

### File Structure Notes

- Keep the canonical support baseline in `docs/`, not under runtime source directories.
- Reuse existing backend validation owners if any machine-readable support metadata becomes necessary.
- Do not add committed visual-fixture corpora or a new standalone validation app for Epic 7 baseline definition.

### Git Intelligence Summary

- The latest commit opened Epic 7 planning by updating `_bmad-output/planning-artifacts/epics.md` and sprint tracking. Story 7.1 should now convert that planning intent into the concrete maintainer artifact later stories will execute.
- The most recent shipped feature work is Story 6.4 expression-map playback. The support baseline should treat that transport-owned path as part of current MVP playback scope without widening the claim to broader compatibility or realism.

### Latest Technical Information

- No external dependency upgrade or new framework decision is required for Story 7.1.
- The important “latest” truth is current repo evidence: the bounded host subset, blocked targets, transport scenario owners, and current representative performance baseline already recorded in the repo.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 7 and Story 7.1 acceptance criteria.
- `_bmad-output/planning-artifacts/prd.md`
  - Best-effort spec-aligned compatibility posture, representative plugin-library expectations, and release-credibility framing.
- `_bmad-output/planning-artifacts/implementation-readiness-report-2026-03-20.md`
  - NFR29 through NFR35 for representative plugin, transport, reopen, and discovery scenario coverage.
- `docs/development-guide.md`
  - Current known-good bundles, blocked bundles, and maintainer manual validation checklist.
- `TESTING.md`
  - Existing host-validation subset, transport/performance scenario guidance, and trust-gate framing.
- `docs/performance-baselines.md`
  - Current representative render baseline; adjacent but not interchangeable with playback support claims.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Current bounded host unsupported list.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Availability versus `editor_host_validation` truth.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Phase-aware native editor-host validation evidence.
- `src-tauri/src/playback/preparation.rs`
  - `supported_targets` snapshot seam.
- `src-tauri/src/playback/transport.rs`
  - Supported transport scenario owner.
- `/memories/repo/expression-map-transport-proof-story-6-4.md`
  - Current transport-owned expression-map proof boundary.
- `/memories/repo/plugin-editor-host-session-lifecycle.md`
  - Host lifecycle stability constraints that matter to baseline scenario design.
- `/memories/repo/vst3-host-lifecycle-sanity-check.md`
  - Cross-host lifecycle sanity checks and the narrow current claim boundary.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-07: Story prepared from Epic 7 planning, PRD release-credibility requirements, implementation readiness NFRs, current maintainer docs, and the live backend seams that currently own plugin inventory truth, bounded host evidence, and transport validation behavior.
- 2026-04-07: Story context explicitly captures the current ambiguity between generic “BBC Spitfire library” planning language and the concrete bundle names presently evidenced in repository docs.
- 2026-04-07: Story context explicitly warns against stale compatibility notes overriding current repository truth, with MT-PowerDrumKit called out as a known drift trap.
- 2026-04-07: Added `docs/playback-support-baseline.md` as the canonical Epic 7 source of truth for bounded support vocabulary, representative bundle coverage, blocked-seam posture, and release-gating scenario matrices.
- 2026-04-07: Reduced `docs/development-guide.md` and `TESTING.md` to canonical-baseline references so support claims follow backend-owned evidence instead of drifting across multiple maintainer docs.
- 2026-04-07: Validated the documentation-only implementation against current owner seams and ran `npm run lint`, `npm run check-types`, `npm run test:js`, and `npm run test:rust` with passing results.

### Completion Notes List

- Story context created for Story 7.1.
- Epic 7 moved to `in-progress` because Story 7.1 is the first story in the epic.
- Story 7.1 moved to `ready-for-dev` in sprint tracking.
- The story directs implementation toward one canonical support-baseline artifact, concrete plugin and scenario subsets, and documentation alignment without inventing a second runtime truth.
- Created `docs/playback-support-baseline.md` as the single maintainer-facing support baseline for MVP playback, including explicit vocabulary, concrete bundle coverage, blocked-bundle posture, and repeatable Story 7.2 / 7.3 scenario matrices.
- Updated `docs/development-guide.md`, `TESTING.md`, and `docs/index.md` to point at the canonical baseline instead of carrying a second drifting support matrix.
- Kept the baseline aligned to backend-owned truth by anchoring blocked posture to `plugin_host_compatibility.rs`, discovery posture to `plugin_inventory.rs`, and transport support posture to `supported_targets` in `preparation.rs`.
- Full regression validation passed: `npm run lint`, `npm run check-types`, `npm run test:js` (100 files / 1281 tests), and `npm run test:rust` (1295 tests).

### File List

- `docs/playback-support-baseline.md`
- `docs/development-guide.md`
- `TESTING.md`
- `docs/index.md`
- `_bmad-output/implementation-artifacts/7-1-define-the-supported-plugin-and-scenario-baseline.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-04-07: Added the canonical MVP playback support baseline under `docs/`, aligned maintainer docs to reference it, and validated the repo with lint, typecheck, JS tests, and Rust tests.
