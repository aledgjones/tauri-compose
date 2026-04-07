# Story 7.2a: Stabilize Bounded Native Editor Host For Real-Library Control Bundles

Status: ready-for-dev

## Story

As a maintainer,
I want the bounded native editor host to behave predictably for the real-library control bundles,
so that Epic 7 can make a release verdict from stable host evidence instead of intermittent hangs.

## Acceptance Criteria

1. Given `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3` are currently blocked from `known-good` status by intermittent host instability, when the bounded native editor-host flow is exercised through the real product seam, then open, focus, supported resize, close, and same-session reopen behave predictably enough to support a concrete host verdict for each bundle.
2. Given same-session reopen, focus-time callbacks, and plugin-driven resize behavior are the highest-risk lifecycle seams, when the owner seam is hardened, then the fix stays inside `src-tauri/src/playback/plugin_editor_poc.rs` and adjacent owner seams, is covered by seam-local regression tests, and does not introduce a validation-only runtime path or new UI.
3. Given Epic 7 support language must remain evidence-driven, when the instability is resolved or bounded, then `docs/playback-support-baseline.md` and any directly linked maintainer guidance are updated to reflect the new verdict without broadening support claims beyond the concrete bundles actually revalidated.
4. Given intermittent hangs are a release risk, when Story 7.2a completes, then the result explicitly states whether the BBC and Originals control bundles are now `known-good`, remain `discovered but unvalidated`, or still carry a bounded unresolved issue, and the evidence is strong enough for Story 7.4 to reuse directly.

## Tasks / Subtasks

- [ ] Task 1: Reproduce and isolate the unstable bounded host lifecycle on the real-library control bundles (AC: 1, 2, 4)
  - [ ] Use the existing bounded native editor-host scenario matrix in `docs/playback-support-baseline.md` as the reproduction checklist; do not create a parallel host-validation matrix.
  - [ ] Capture which concrete lifecycle phase or scenario remains unstable for `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3`, with special attention to same-session reopen, focus changes, and plugin-driven resize behavior.
  - [ ] Keep evidence anchored to backend-owned lifecycle phases, `editor_host_validation`, and maintainer-observed product behavior rather than anecdotal summaries.

- [ ] Task 2: Harden the owner seam without widening product scope (AC: 1, 2)
  - [ ] Make any required fixes seam-local to `src-tauri/src/playback/plugin_editor_poc.rs` and directly adjacent owner seams only.
  - [ ] Preserve backend-owned reservation and teardown semantics for `Idle -> Reserved -> Attached -> Closing` and any detached-session reopen path already owned by the seam.
  - [ ] Keep blocked-bundle policy behavior unchanged for `HALion Sonic.vst3`, `HALion Sonic SE.vst3`, `Groove Agent SE.vst3`, `Kontakt 7.vst3`, and `MT-PowerDrumKit.vst3`.
  - [ ] Do not add new UI, fallback host paths, or frontend-owned support state to prove the fix.

- [ ] Task 3: Add seam-local regressions and rerun the bounded host validation set (AC: 1, 2, 4)
  - [ ] Extend `plugin_editor_poc.rs` tests only where current coverage does not already prove the repaired lifecycle behavior.
  - [ ] Re-run the owner seam validations for policy gating, reservation cleanup, attach or reattach ordering, and any new lifecycle guardrails added by the fix.
  - [ ] Execute maintainer manual validation against `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3` with repeated same-session open or close or reopen coverage strong enough to support a concrete verdict for each bundle.

- [ ] Task 4: Publish the updated host verdict without over-claiming support (AC: 3, 4)
  - [ ] Update `docs/playback-support-baseline.md` with the Story 7.2a outcome for the BBC and Originals control bundles.
  - [ ] Update `docs/development-guide.md` only if maintainer-facing host lifecycle guidance materially changes.
  - [ ] Update `TESTING.md` only if the recommended host-validation command set or manual sequencing materially changes.
  - [ ] Keep the resulting support vocabulary aligned with Epic 7: `known-good`, `blocked/unsupported for the bounded seam`, `discovered but unvalidated`, and `future compatibility expansion`.

## Dev Notes

### Story Intent

- Story 7.2a is a focused stability follow-up created from Story 7.2's unresolved host verdict.
- It exists to isolate bounded host lifecycle repair from the broader Epic 7 transport and final-release-verdict stories.
- The output should let maintainers say, with evidence, whether the two real-library control bundles are now stable enough for a bounded host `known-good` claim.

### Why This Story Exists

- Story 7.2 completed the discovery and host validation pass.
- Automated seam-local tests passed, and blocked-bundle policy behavior remains clean.
- Manual validation still found intermittent open-or-hang behavior for:
  - `BBC Symphony Orchestra.vst3`
  - `Originals - Intimate Grand Piano.vst3`
- Epic 7 cannot honestly round those bundles up to `known-good` until the instability is either fixed or explicitly bounded with stronger evidence.

### This Story Owns

- Reproducing and stabilizing the bounded native editor-host lifecycle for the two real-library control bundles above.
- Seam-local owner fixes in `plugin_editor_poc.rs` and directly adjacent host-lifecycle seams.
- Regression coverage and maintainer evidence needed to turn the current unresolved host verdict into a concrete release-facing outcome.
- Updating the canonical baseline docs once the verdict changes.

### This Story Does Not Own

- Transport validation. That remains Story 7.3.
- Publishing the final MVP support verdict across Epic 7. That remains Story 7.4.
- Broadening support claims to plugin families, broader Spitfire coverage, or universal VST3 host compatibility.
- New UI surfaces, diagnostics-only windows, or a second validation-only host path.

### Current Relevant Seams

- `docs/playback-support-baseline.md`
  - Canonical Epic 7 support matrix and the Story 7.2 unresolved host verdict that this story must update.
- `docs/development-guide.md`
  - Maintainer-facing current warning that BBC and Originals are not yet `known-good` for the bounded host seam.
- `TESTING.md`
  - Command guidance and bounded host manual validation sequence.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Owner seam for bounded native editor-host lifecycle, reservation state, attach, close, teardown, and same-session reopen.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Blocked-bundle policy seam that must remain unchanged by this story.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Backend-owned storage of `editor_host_validation` evidence surfaced to maintainer workflows.

### Repo Memory Guardrails

- Preserve the explicit backend lifecycle record: `Idle -> Reserved -> Attached -> Closing`, with detached-session reopen behavior remaining backend-owned rather than frontend-owned.
- Focus and initial layout sync must happen only after the live host session is stored; pre-storage sync leaves reopen and focus behavior unreliable.
- Reopen stalls are strongly associated with focus-time sync reentry, plugin-driven resize callbacks, teardown ordering, and stale reservation cleanup.
- BBC specifically has prior evidence around plugin-driven resize feedback loops and post-attach settling behavior on same-session reopen.

### Implementation Guidance

- Start with reproduction and instrumentation through the real owner seam. Do not guess at a fix before narrowing the unstable lifecycle phase.
- Likely high-risk areas to inspect first in `plugin_editor_poc.rs`:
  - focus-time sync behavior after attach or reattach
  - plugin-driven `resizeView` handling and main-thread window resizing
  - same-session close or destroy ordering and reservation cleanup
  - detached-session reattach ordering, including mutex or state handoff timing
  - any hidden dependency on legacy singleton active-session state instead of slot-runtime-backed state
- Keep fixes backend-owned and seam-local. If additional logging is needed, keep it bounded to the owner seam and consistent with the existing `SOLOC_PLUGIN_EDITOR_POC_VERBOSE=1` posture.
- Maintain the distinction between bundle validity, blocked policy posture, and real-library host stability. This story fixes or bounds the host lifecycle for two concrete bundles; it does not reclassify other discovered bundles.
- If one bundle becomes stable and the other remains unstable, record the verdict separately. Do not collapse them into one family-level claim.

### Risks And Regression Traps

- Accidentally widening the bounded host fix into a new generic host abstraction or fallback path.
- Reintroducing same-session reopen deadlocks through focus-time resize sync or attach-time lock ordering.
- Breaking blocked-bundle rejection while chasing the known-good control-bundle instability.
- Treating a single clean open as sufficient evidence when the problem is intermittent and same-session reopen-driven.
- Updating docs to say `known-good` before repeated manual validation really supports that verdict.

### Testing Requirements

- Prefer seam-local Rust tests first:
  - `cd src-tauri && cargo test plugin_editor_poc --lib -- --nocapture`
  - `cd src-tauri && cargo test plugin_host_compatibility --lib -- --nocapture`
  - `cd src-tauri && cargo test plugin_inventory --lib -- --nocapture`
- If host lifecycle code changes, also run:
  - `npm run check-types`
  - `npm run lint`
  - `npm run test:rust`
- Manual validation must use the canonical Story 7.2 host scenario set in `docs/playback-support-baseline.md` and explicitly cover:
  - open known-good editor
  - focus and responsiveness
  - supported resize, if the plugin exposes it
  - close and same-session reopen
  - repeated reopen cycles strong enough to support a concrete verdict for each bundle
- Keep new regressions local to `plugin_editor_poc.rs` unless another owner seam is directly responsible for the fault.

### Documentation Impact

- Update `docs/playback-support-baseline.md` with the Story 7.2a verdict for BBC and Originals.
- Update `docs/development-guide.md` only if the maintainer-facing runbook or lifecycle guidance changes materially.
- Update `TESTING.md` only if the bounded host validation workflow changes materially.
- Update AI instructions only if Story 7.2a establishes a durable repo-wide validation rule rather than a one-off host result.

### Architecture Compliance

- Backend ownership remains intact: host lifecycle truth, policy gating, and validation evidence remain Rust-owned.
- The story must not move support truth into frontend state.
- `.soloc` portability rules remain unchanged; machine-local host evidence stays outside project files.
- Keep the current MVP claim macOS-only, VST3-only, and bounded to the explicit Epic 7 baseline.

### Library And Framework Requirements

- Stay on the existing repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- Do not add dependencies for this story unless a narrow seam-local test need is unavoidable.

### File Structure Notes

- Keep code changes inside the owning playback seams under `src-tauri/src/playback/`.
- Keep evidence updates in the existing maintainer docs under `docs/`.
- Do not add a separate validation app, fixture corpus, or frontend-owned support matrix for this follow-up.

### Git Intelligence Summary

- Story 7.1 defined the baseline and Story 7.2 executed it.
- Story 7.2a exists because the automated owner-seam coverage passed while manual host validation still found instability on the two real-library control bundles.
- The follow-up should resolve that discrepancy by hardening the real owner seam, not by softening the support language.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 7, Story 7.2, Story 7.3, and Story 7.4 release-validation framing.
- `_bmad-output/implementation-artifacts/7-1-define-the-supported-plugin-and-scenario-baseline.md`
  - Original canonical baseline definition and scenario owners.
- `_bmad-output/implementation-artifacts/7-2-validate-plugin-discovery-and-host-behavior-against-the-baseline.md`
  - Completed validation pass and the unresolved host verdict that this follow-up must address.
- `docs/playback-support-baseline.md`
  - Canonical Epic 7 support matrix and current unresolved host results.
- `docs/development-guide.md`
  - Maintainer-facing current bounded host warning for BBC and Originals.
- `TESTING.md`
  - Command guidance and manual bounded host validation sequence.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Bounded native editor-host lifecycle owner seam.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Blocked-bundle policy seam.
- `src-tauri/src/playback/plugin_inventory.rs`
  - `editor_host_validation` evidence owner for inventory entries.
- `/memories/repo/plugin-editor-host-session-lifecycle.md`
  - Same-session reopen, resize, and teardown risks already observed in this repo.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-07: Story 7.2a created after Story 7.2 recorded unresolved intermittent open-or-hang behavior for `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3`.
- 2026-04-07: Story 7.2a intentionally isolates bounded native editor-host stabilization from Story 7.3 transport validation and Story 7.4 support-verdict publication.
- 2026-04-07: Story 7.2a carries forward the known plugin-editor lifecycle traps already documented in repo memory so implementation starts from the real failure seams rather than a fresh host rewrite.

### Completion Notes List

- Follow-up story created from Story 7.2's unresolved bounded host verdict.
- Story is ready-for-dev and scoped narrowly to seam-local host lifecycle stabilization and evidence updates.

### File List

- `_bmad-output/implementation-artifacts/7-2a-stabilize-bounded-native-editor-host-for-real-library-control-bundles.md`
- `_bmad-output/implementation-artifacts/7-2-validate-plugin-discovery-and-host-behavior-against-the-baseline.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
