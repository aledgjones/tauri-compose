# Story 7.2: Validate Plugin Discovery And Host Behavior Against The Baseline

Status: done

## Story

As a maintainer,
I want discovery, loading, and bounded editor-host behavior validated against representative plugins,
so that the supported playback path is credible beyond isolated implementation slices.

## Acceptance Criteria

1. Given the representative plugin baseline is defined, when discovery, loading, and bounded editor-host scenarios are exercised, then Compose can show which validated plugins are usable through the supported path, and failures or unsupported behaviors are recorded without broadening the support claim.
2. Given plugin-host behavior is part of the release story, when representative known-good plugins are tested, then loading, basic usability, and bounded native editor-host behavior are evaluated as part of the support baseline, and unsupported plugins or interactions remain outside the MVP support claim unless explicitly validated.
3. Given inventory and host validation are backend-owned concerns with user-facing consequences, when validation evidence is collected, then it reuses the real product seams rather than an unrelated one-off path, and it is strong enough to support release review.
4. Given intermittent plugin UI hangs remain a credible support risk, when representative native editor-host scenarios are validated, then the baseline explicitly records whether those hangs are reproducible, bounded, or unresolved, and the MVP support verdict does not ignore host-lifecycle instability that would undermine release confidence.

## Tasks / Subtasks

- [x] Task 1: Validate startup discovery and manual refresh behavior against the Story 7.1 baseline (AC: 1, 3)
  - [x] Reuse the canonical scenario matrix in `docs/playback-support-baseline.md` instead of creating a second validation checklist.
  - [x] Confirm startup cache reuse remains bounded to `src-tauri/src/playback/plugin_inventory.rs`: cached baseline bundles appear immediately, blocked posture stays distinct, and startup does not widen into manual refresh behavior.
  - [x] Confirm the cheap startup root sweep adds newly seen uncached bundles without regressing cached entry truth.
  - [x] Confirm manual refresh or full reprobe preserves valid baseline entries, refreshes inventory truth, and does not collapse blocked host posture into generic bundle failure.
  - [x] Extend seam-local inventory tests only where current coverage does not already prove the required startup and refresh semantics.

- [x] Task 2: Validate bounded native editor-host behavior on known-good and blocked bundles (AC: 2, 3, 4)
  - [x] Execute the bounded native editor-host scenario set from `docs/playback-support-baseline.md` using concrete baseline bundles, starting with `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3`.
  - [x] Verify open, focus, supported resize, close, and same-session reopen behavior through `src-tauri/src/playback/plugin_editor_poc.rs` and backend-emitted `editor_host_validation` evidence.
  - [x] Verify blocked bundles such as `HALion Sonic.vst3` or `Kontakt 7.vst3` are rejected through the real bounded policy seam before host attachment or visible host-window creation.
  - [x] Record whether intermittent hangs are reproducible, bounded, or unresolved, with special attention to same-session reopen and focus or resize-driven regressions.
  - [x] Keep any code fixes seam-local to the owner seam; do not add new UI or a separate validation-only runtime path.

- [x] Task 3: Capture release-review evidence without inventing a second compatibility taxonomy (AC: 1, 2, 3, 4)
  - [x] Collect evidence from backend-owned seams only: inventory state, `editor_host_validation`, bounded host lifecycle phases, and the canonical baseline document.
  - [x] Keep the support vocabulary aligned with Story 7.1: `known-good`, `blocked/unsupported for the bounded seam`, `discovered but unvalidated`, and `future compatibility expansion`.
  - [x] Make the result explicit enough for Story 7.4 to reuse without re-running interpretation work.
  - [x] Reject any implementation that broadens support claims from one validated bundle to a plugin family without explicit proof.

- [x] Task 4: Keep documentation and test guidance aligned with the evidence actually produced (AC: 1, 3, 4)
  - [x] Update `docs/playback-support-baseline.md` with the validation outcome for each executed Story 7.2 scenario.
  - [x] Update `TESTING.md` only where command guidance or manual validation sequencing changes materially.
  - [x] Update `docs/development-guide.md` if the validated host lifecycle guidance or maintainer runbook changes.
  - [x] Update `docs/index.md` if Story 7.2 adds a new maintainer-facing document.

## Dev Notes

### Story Intent

- Story 7.2 is the first evidence-collection pass for Epic 7.
- Story 7.1 already defined the support boundary. Story 7.2 must validate that boundary through real discovery and bounded editor-host seams, not reword it.
- The output of this story should let maintainers answer two questions precisely: which concrete bundles are actually validated today, and which failures remain bounded or unresolved without overstating support.

### Story 7.1 Intelligence To Carry Forward

- `docs/playback-support-baseline.md` is the canonical support artifact for Epic 7. Reuse it; do not fork a second matrix into `TESTING.md`, `docs/development-guide.md`, or UI state.
- The concrete baseline bundles currently inside the Story 7.1 definition are:
  - `BBC Symphony Orchestra.vst3`
  - `Originals - Intimate Grand Piano.vst3`
  - `Reverb.vst3`
  - `ProEQ.vst3`
- The planning phrase “BBC Spitfire libraries” is intentionally concretized to the first two bundles above. Do not fall back to vague family-level language.
- The current blocked bounded-host list is backend-owned in `src-tauri/src/playback/plugin_host_compatibility.rs` and includes:
  - `HALion Sonic.vst3`
  - `HALion Sonic SE.vst3`
  - `Groove Agent SE.vst3`
  - `Kontakt 7.vst3`
  - `MT-PowerDrumKit.vst3`
- Preserve Story 7.1 vocabulary exactly: `known-good`, `blocked/unsupported for the bounded seam`, `discovered but unvalidated`, and `future compatibility expansion`.
- Treat repository code as truth over stale notes. A concrete drift trap is `MT-PowerDrumKit.vst3`: current repo code and baseline docs treat it as blocked for the bounded seam.

### This Story Owns

- Validating Story 7.1's discovery, manual refresh, and bounded editor-host scenario matrix through real owner seams.
- Capturing release-review evidence for representative known-good and blocked bundles.
- Narrow seam-local fixes needed to keep inventory truth, blocked posture, or host lifecycle evidence honest.
- Updating maintainer docs so the baseline records actual results rather than planned scenarios only.

### This Story Does Not Own

- Transport trust validation. That belongs to Story 7.3.
- Broadening the support claim beyond the explicit Story 7.1 baseline.
- Adding new product UI or developer-only proof surfaces to demonstrate acceptance criteria literally.
- Creating a frontend-owned support taxonomy or parallel host-validation model.

### Current Relevant Seams

- `docs/playback-support-baseline.md`
  - Canonical Story 7.2 scenario matrix and support vocabulary.
- `TESTING.md`
  - Existing command guide and manual validation guidance.
- `docs/development-guide.md`
  - Maintainer-facing native editor handoff behavior and startup inventory semantics.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Owns startup cache reuse, bounded startup sweep, manual discovery, persisted inventory truth, and `editor_host_validation` storage on inventory entries.
  - Relevant functions and tests already present:
    - `blocked_editor_host_validation(...)`
    - `inventory_editor_host_validation(...)`
    - `plugin_inventory_startup_reuses_cached_entries_when_no_new_bundles_exist()`
    - `plugin_inventory_startup_treats_empty_persisted_state_as_cache_present()`
    - `manual_full_rescan_replaces_bounded_startup_snapshot_and_persists_new_truth()`
    - `plugin_inventory_startup_discovers_new_uncached_bundles_during_bounded_sweep()`
    - `plugin_inventory_startup_marks_missing_cached_entries_unavailable()`
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Owns `bounded_editor_host_unsupported_reason(...)` and the bounded blocked-bundle list.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Owns real bounded native editor-host opening, lifecycle reservation, attach results, and `editor_host_validation` emission.
  - Relevant seams and tests already present:
    - `attached_editor_host_validation()`
    - `queue_editor_host_validation_emit(...)`
    - `ensure_editor_host_target_supported(...)`
    - `plugin_editor_poc_open(...)`
    - lifecycle reservation tests around `reserve_plugin_target(...)`
    - bounded unsupported-target tests around `ensure_editor_host_target_supported(...)`

### Repo Memory Guardrails

- Startup cache semantics remain bounded to `plugin_inventory.rs`: if persisted state exists, even an empty one, treat it as cache-present; only missing persisted state should trigger startup discovery. [Source: /memories/repo/plugin-inventory-startup-cache.md]
- The hardened editor-host lifecycle is `Idle -> Reserved -> Attached -> Closing`, with same-session reopen safety dependent on backend-owned reservation cleanup and post-storage focus or size sync. [Source: /memories/repo/plugin-editor-host-session-lifecycle.md]
- Same-session reopen remains the highest-risk host scenario. Focus-time resize feedback loops, stale reservations, or teardown races can still undermine release confidence if not explicitly validated.

### Implementation Guidance

- Start by validating the baseline through existing tests and existing maintainer flows before changing code. Extend current seam-local tests rather than inventing a broad new harness.
- Discovery validation guidance:
  - Keep startup cache reuse, empty-cache-present semantics, bounded startup sweep, and manual full reprobe distinct.
  - Preserve stable plugin identity when cached bundles become missing or invalid.
  - Preserve the difference between machine-local availability and bounded editor-host blocking.
- Host validation guidance:
  - Use concrete bundles from the baseline doc, not generic plugin-family placeholders.
  - For known-good bundles, validate open, focus, resize when supported, close, and same-session reopen.
  - For blocked bundles, validate rejection through the policy seam before attachment and preserve inventory visibility instead of rewriting the bundle as invalid.
  - If intermittent hangs reproduce, record exactly whether the issue is reproducible, bounded to a scenario, or unresolved. Do not hide lifecycle instability behind a pass verdict.
- Evidence guidance:
  - Evidence must come from real seams such as inventory state, `editor_host_validation`, and host lifecycle behavior.
  - Do not rely on anecdotal “it seemed fine” notes.
  - Do not create a validation-only fallback path in React, tests, or docs.

### Risks And Regression Traps

- Confusing `discovered but unvalidated` with `known-good` and accidentally widening the MVP claim.
- Letting stale compatibility notes override current backend truth, especially for `MT-PowerDrumKit.vst3`.
- Regressing bounded startup semantics by turning ordinary startup into full refresh behavior.
- Losing the distinction between blocked bounded-host posture and generic inventory invalidity.
- Treating `editor_host_validation.status = attached` as transport proof; it only proves bounded editor-host usability.
- Same-session reopen can still regress through stale reservation state, resize feedback loops, or close-teardown ordering.
- Adding UI or diagnostics whose main purpose is to prove acceptance criteria rather than reusing the product seams already named by Story 7.1.

### Testing Requirements

- Prefer existing seam-local Rust tests first:
  - `cd src-tauri && cargo test plugin_inventory --lib -- --nocapture`
  - `cd src-tauri && cargo test plugin_editor_poc --lib -- --nocapture`
  - `cd src-tauri && cargo test bounded_editor_host_unsupported_reason --lib`
- If host or inventory code changes, also run:
  - `npm run check-types`
  - `npm run lint`
  - `npm run test:rust`
- Manual validation should follow the canonical Story 7.2 scenario matrix in `docs/playback-support-baseline.md` and capture explicit evidence for:
  - startup cache reuse
  - bounded startup sweep for newly added bundles
  - manual refresh or full reprobe
  - open known-good editor
  - focus and responsiveness
  - resize when supported
  - close and same-session reopen
  - blocked bundle rejection

### Documentation Impact

- Update `docs/playback-support-baseline.md` with executed Story 7.2 results.
- Update `TESTING.md` if validation commands or maintainer sequencing change.
- Update `docs/development-guide.md` if validated lifecycle or inventory guidance changes.
- Update AI instructions only if Story 7.2 establishes a durable repo-wide validation rule rather than a one-off result.

### Architecture Compliance

- Backend ownership remains intact: inventory truth and host validation stay Rust-owned.
- The story must not move support truth into frontend state.
- `.soloc` portability rules remain unchanged; machine-local inventory and host validation remain outside project files.
- Keep the current MVP claim macOS-only, VST3-only, and bounded to the explicit baseline.

### Library And Framework Requirements

- Stay on the existing repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- Do not add dependencies for this story unless a narrow seam-local test or validation-format need is unavoidable.

### File Structure Notes

- Keep evidence and support-boundary updates in existing maintainer docs under `docs/`.
- Keep code changes inside the owning playback seams under `src-tauri/src/playback/`.
- Do not add a separate validation app, fixture corpus, or frontend-owned support matrix for Epic 7.

### Git Intelligence Summary

- The latest commit for Epic 7 was `feat(7.1): playback support docs.`, which means Story 7.2 should execute the baseline that Story 7.1 documented rather than redefining it.
- Recent shipped behavior before Epic 7 was expression-map playback in Story 6.4; Story 7.2 should not blur editor-host validation into transport or expression-map trust claims.

### Open Questions To Record During Implementation

- What exact evidence format is sufficient for release review: baseline-doc updates alone, or a second maintainer-facing result artifact linked from the baseline?
- How many same-session reopen cycles are required before a host behavior can be called stable for the bounded seam?
- For blocked bundle rejection, is the strongest pass condition “no host window created at all” or “no attachment attempt plus blocked validation state”? Prefer the stronger condition if the current seam already supports it.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 7 and Story 7.2 acceptance criteria.
- `_bmad-output/implementation-artifacts/7-1-define-the-supported-plugin-and-scenario-baseline.md`
  - Prior story guidance, vocabulary, and regression traps.
- `_bmad-output/planning-artifacts/prd.md`
  - Release-credibility framing, representative library expectations, and evidence-driven MVP posture.
- `_bmad-output/planning-artifacts/architecture.md`
  - Backend ownership, bounded macOS host seam, and release-validation architecture.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Requirement to avoid proof-surface UI and keep deep plugin editing as a secondary handoff.
- `docs/playback-support-baseline.md`
  - Canonical plugin subset and Story 7.2 scenario matrix.
- `TESTING.md`
  - Test commands and manual validation guidance.
- `docs/development-guide.md`
  - Startup inventory and bounded native editor handoff behavior.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Startup cache, bounded sweep, refresh, and inventory validation state.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Bounded blocked-bundle policy seam.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Bounded native editor-host lifecycle and `editor_host_validation` emission.
- `/memories/repo/plugin-inventory-startup-cache.md`
  - Startup cache-present semantics.
- `/memories/repo/plugin-editor-host-session-lifecycle.md`
  - Same-session reopen and lifecycle traps.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-07: Story prepared from Epic 7 planning, Story 7.1 baseline guidance, PRD release-credibility requirements, architecture ownership rules, maintainer docs, and the current playback owner seams.
- 2026-04-07: Story context explicitly anchors validation to `docs/playback-support-baseline.md`, `plugin_inventory.rs`, `plugin_host_compatibility.rs`, and `plugin_editor_poc.rs` to avoid a second support taxonomy.
- 2026-04-07: Story context captures the current same-session reopen and startup-cache traps from repo memory so implementation focuses on the real regression seams.
- 2026-04-07: Development started for Story 7.2; sprint tracking moved to `in-progress` before Task 1 validation.
- 2026-04-07: Automated discovery evidence captured with `cargo test plugin_inventory --lib -- --nocapture` (17 passed), confirming bounded startup cache reuse, bounded startup sweep, manual full reprobe, and blocked-posture separation.
- 2026-04-07: Automated host-seam evidence captured with `cargo test plugin_editor_poc --lib -- --nocapture` (45 passed) and `cargo test plugin_host_compatibility --lib -- --nocapture` (7 passed).
- 2026-04-07: Maintainer manual validation reported intermittent open-or-hang behavior for `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3`; blocked-bundle rejection was manually verified for `HALion Sonic.vst3`, `HALion Sonic SE.vst3`, `Groove Agent SE.vst3`, `Kontakt 7.vst3`, and `MT-PowerDrumKit.vst3`.
- 2026-04-07: Repo validation completed with `npm run check-types`, `npm run lint`, `npm run test:js` (100 files, 1281 tests passed), and `npm run test:rust` (1295 passed).
- 2026-04-07: User-directed story closure marked Story 7.2 done and created follow-up Story 7.2a to isolate bounded native editor-host stabilization for the BBC and Originals real-library control bundles.

### Completion Notes List

- Story context created for Story 7.2.
- Story 7.2 moved to `ready-for-dev` in sprint tracking.
- The story directs implementation toward baseline execution, seam-local validation, and evidence-driven documentation updates without broadening the current MVP support claim.
- Automated Story 7.2 discovery evidence passed without code changes; existing seam-local inventory coverage already proved the required startup and refresh semantics.
- The canonical baseline now records the real Story 7.2 verdict: blocked bundles remain cleanly rejected in the bounded seam, while `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3` remain unresolved because the native editor host intermittently hangs.
- `docs/development-guide.md` now carries the maintainer-facing caution that those two real-library control bundles are not `known-good` for release posture today.
- `TESTING.md` did not require an update because the command guidance and manual validation sequence did not materially change.
- `docs/index.md` did not require an update because Story 7.2 reused existing maintainer-facing documents instead of adding a new one.
- Created follow-up Story 7.2a so the unresolved bounded host instability can be fixed in a seam-local story before Epic 7 publishes the final support verdict.

## Change Log

- 2026-04-07: Captured Story 7.2 automated discovery and host-seam evidence, recorded the maintainer manual host verdict in the canonical baseline, and moved the story to review.
- 2026-04-07: Closed Story 7.2 as done at user direction and created Story 7.2a for bounded host stabilization follow-up.

### File List

- `docs/playback-support-baseline.md`
- `docs/development-guide.md`
- `_bmad-output/implementation-artifacts/7-2a-stabilize-bounded-native-editor-host-for-real-library-control-bundles.md`
- `_bmad-output/implementation-artifacts/7-2-validate-plugin-discovery-and-host-behavior-against-the-baseline.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
