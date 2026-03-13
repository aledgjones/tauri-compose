# Story 3.5: Verify Non-Regression and Performance Scaling for Multi-Flow Rendering

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want evidence that multi-flow rendering preserves current single-flow performance expectations and scales predictably,
so that the feature can move forward without hidden regressions in responsiveness.

Epic 3 has already established the core guardrails this story depends on: Story 3.1 made render scope observable, Story 3.2 locked in notation safety, Story 3.3 introduced truthful flow-scoped recalculation, and Story 3.4 is intended to stabilize unaffected visible flows during local updates. Story 3.5 is the validation increment that turns those changes into concrete evidence. The goal is not to invent a parallel benchmark subsystem or frontend-owned metrics model; it is to use the existing Rust-owned timing and render-scope seams to prove whether single-flow behavior remains acceptable and whether multi-flow work scales with the affected flows instead of silently broadening into whole-surface cost.

## Acceptance Criteria

1. Given the prior observability and isolation stories are implemented, when representative single-flow and multi-flow scenarios are exercised, then maintainers can compare current single-flow behavior against the updated multi-flow path and any regression is visible through established timing output or test evidence.
2. Given a representative multi-flow scenario, when render and update timings are reviewed, then the results show whether work scales with the affected flows rather than as unconditional whole-score recalculation and the outcome is documented clearly enough to guide follow-on optimization work if needed.
3. Given the single-flow baseline scenario, when validation is performed after the multi-flow changes, then current single-flow behavior remains acceptable for the existing product standard and the project gains explicit evidence rather than assumptions about performance preservation.
4. Given this validation increment is completed, when Epic 3 is reviewed as a whole, then multi-flow rendering has observability, notation safety, recalculation isolation, and basic performance characterization and no persisted score or migration change is required by this validation story.

## Tasks / Subtasks

- [x] Define the representative validation scenarios and acceptance evidence before changing instrumentation (AC: 1, 2, 3, 4)
  - [x] Capture a single-flow baseline scenario aligned with the current product standard, including the existing three-instrument, 30-bar expectation called out in the planning artifacts and project requirements.
  - [x] Capture at least one representative multi-flow render scenario and one representative local-edit scenario that exercise the unified shared-surface path with two or three visible flows and make recalculation scope observable.
  - [x] Decide whether existing fixture scores already cover these scenarios or whether focused representative fixtures are needed under `tests/fixtures/source/`; if fixtures change, rebuild them with `npm run build:fixtures` and keep fixture metadata aligned with the current app version.
- [x] Reuse and, only if necessary, extend the existing Rust-owned diagnostics path to gather comparable evidence (AC: 1, 2, 3)
  - [x] Audit `src-tauri/src/lib.rs`, `src-tauri/src/instructions/render_plan.rs`, `src-tauri/src/instructions/mod.rs`, and `src/windows/debugger/components/performance-timings.tsx` to confirm the current `render-timing-*`, `render-timing-flow-preparation-details`, and `render-timing-surface-details` contracts expose enough data to compare baseline, multi-flow, and local-edit passes.
  - [x] If the current payloads are insufficient, extend them incrementally in Rust-owned `snake_case` structures rather than introducing console-only logging, ad hoc scripts, or frontend-derived benchmark state.
  - [x] Preserve the existing debugger timing rows and severity behavior so maintainers can compare performance evidence before and after the validation work without learning a second observability surface.
- [x] Add automated non-regression and scaling guardrails around the validation scenarios (AC: 1, 2, 3, 4)
  - [x] Add Rust tests around the render-plan and shared-surface composition seams that prove single-flow baseline passes still report the expected narrow scope and that representative multi-flow local edits report only the affected recalculated flow set when reuse is available.
  - [x] Add or extend Rust tests around shared-surface composition and reuse so unaffected visible flows remain composed in `score.flows.order` and broad fallback only occurs when the backend can prove reuse is not safe.
  - [x] Extend debugger-facing Vitest coverage only if the diagnostics payload shape or maintainer-facing summary changes materially.
- [x] Document the evidence and its limits in a way that guides follow-on work honestly (AC: 2, 3, 4)
  - [x] Record the measured single-flow baseline and representative multi-flow results in the story completion notes or adjacent maintainer-facing docs, including whether each measurement came from the current supported validation environment versus a release-like run.
  - [x] Call out any regressions, broad-fallback cases, or scaling cliffs explicitly instead of masking them behind averaged totals.
  - [x] Update `docs/project-context.md`, `docs/architecture.md`, and `TESTING.md` if this story changes the stable diagnostics contract, introduces a repeatable performance-validation workflow, or adds fixture-based validation expectations maintainers must follow.
- [x] Validate the performance-characterization path end to end (AC: 1, 2, 3, 4)
  - [x] Run `npm run lint`.
  - [x] Run `npm run check-types`.
  - [x] Run `npm run test:js`.
  - [x] Run `npm run test:rust`.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

## Dev Notes

### Story Foundation

- Epic 3 is sequencing performance work deliberately: Story 3.1 added observability, Story 3.2 added notation guardrails, Story 3.3 narrowed recalculation scope, and Story 3.4 is the visual-stability follow-up that should keep unaffected visible flows from churning during local updates.
- Story 3.5 is not another optimization story. Its purpose is to verify the results of the earlier work and make the current performance characteristics explicit enough that maintainers can decide whether Epic 3 is ready to close or needs additional optimization follow-up.
- This remains a presentation and validation increment over existing flow data and shared-surface rendering. It is not permission to rewrite notation logic, change persisted score semantics, or add a parallel benchmark subsystem.

### Developer Context Summary

- `src-tauri/src/lib.rs` already measures flow-preparation time before `generate_render_instructions(...)`, emits `render-timing-flow-preparation`, emits `render-timing-flow-preparation-details`, and clears those events null-safely when no render plan can be built.
- `src-tauri/src/instructions/render_plan.rs` already exposes prepared, composed, and recalculated flow ids/counts/scopes through `FlowRenderPreparationDiagnostics`, including targeted single-flow recalculation requests and hidden-flow no-op cases.
- `src-tauri/src/instructions/mod.rs` already owns the shared-surface composition and reuse decision through `resolve_render_plan_recalculation(...)`; that seam is the current truth source for whether a pass stayed narrow or broadened back to multiple flows.
- `src/windows/debugger/components/performance-timings.tsx` already renders maintainer-facing pass summaries such as `Targeted update` and `Shared update`, flow cards, per-phase CPU timings, and shared-surface page-box details. Story 3.5 should keep performance evidence flowing through that existing debugger surface rather than inventing another UI.
- `src/windows/debugger/components/performance-timings.test.tsx` already covers narrow, broad, three-page, and cleared-state timing scenarios, so it is the correct frontend regression seam if diagnostics or summaries need to change.

### Technical Requirements

- Use the existing Epic 3 diagnostics family as the primary evidence path. Prefer extending `render-timing-flow-preparation-details` or adjacent Rust-owned payloads over adding standalone logging or custom measurement scripts.
- Keep performance evidence truthful. If a representative local edit broadens back to multiple visible flows because reuse is unavailable or a shared dependency requires it, surface that outcome explicitly rather than claiming isolated scaling.
- Preserve and periodically reevaluate the single-flow baseline contract while validating multi-flow behavior. The current release-like `npm run perf:render` single-flow result is the active reference point, and a representative total median at or above 2ms should trigger a regression warning and follow-up review.
- Distinguish between no-op/shared-surface rerender cost, full visible-flow recalculation cost, and affected-flow-only recalculation cost where the current diagnostics can do so without ambiguity.
- Preserve written-duration behavior, notation semantics, hit-target correctness, selection continuity, active-flow semantics, and current document behavior while gathering performance evidence.
- No persistence, migration, or `.soloc` semantic change is expected. Do not touch `src-tauri/src/migrations/next.rs` unless implementation discovery proves older files would break, which would contradict the planned story boundary.

### Architecture Compliance

- Rust remains the single source of truth for score state, render timing payloads, recalculation scope, and shared-surface composition decisions.
- The native score surface remains the only notation render path; React continues to act as the overlay and maintainer-facing debugger shell.
- Performance characterization must build on the current render-plan and composition seams rather than introducing a second benchmark-only execution model with different invalidation rules.
- `ui.flow_id` remains the single authoritative active-flow slot. Do not create a frontend-owned notion of the measured flow or flow scope just to simplify debugger presentation.
- If this story exposes a stable maintainer workflow for validating render performance, document it through the existing docs set rather than burying it in story notes alone.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep frontend/backend payload fields in `snake_case` and preserve typed alignment between Rust diagnostics structs and TypeScript debugger consumers.
- Preserve current performance-oriented backend choices such as `FxHashMap`, the native Rust/Skia/Metal render path, renderer-side segment reuse, and the current command/event model.
- No dependency upgrade is expected for this story.
- External web research was not available in this environment; repository docs, source, and pinned package versions are the authoritative baseline.

### Project Structure Notes

- Primary performance-validation seams:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/instructions/render_plan.rs`
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/renderer/mod.rs`
- Existing maintainer-facing debugger seams:
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
- Likely score-update seams for representative affected-flow measurements:
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/score/entries/tempo.rs`
  - `src-tauri/src/score/entries/barline.rs`
  - `src-tauri/src/score/entries/time_signature.rs`
  - `src-tauri/src/score/entries/key_signature.rs`
  - `src-tauri/src/score/entries/clef.rs`
- Representative fixture and testing seams if scenario coverage needs to be codified:
  - `tests/fixtures/source/`
  - `tests/fixtures/README.md`
  - `TESTING.md`

### File Structure Requirements

- Extend the existing diagnostics, render-plan, and debugger modules before creating any new top-level performance-validation structure.
- If helper code is needed for representative scenario setup, keep it close to the owning Rust render tests or existing fixture tooling instead of creating a repo-wide benchmark framework.
- Keep debugger changes limited to the existing `performance-timings` component/test seam if the maintainer-facing presentation changes.
- If representative fixture JSON changes are introduced, keep them grouped by concern under `tests/fixtures/source/`, rebuild built fixtures with `npm run build:fixtures`, and do not commit generated `.soloc` files.

### Testing Requirements

- Add Rust tests that compare representative single-flow and multi-flow scenarios using the existing render-plan and shared-surface seams, with assertions that make recalculation scope and fallback behavior explicit.
- Keep single-flow baseline assertions in place so Epic 3 does not trade away the existing product standard while validating multi-flow behavior, and refresh the recorded baseline when material rendering features land.
- Extend debugger UI tests only if the displayed diagnostics contract or maintainer-facing summaries change.
- If fixture-based representative scenarios are added or updated, rebuild them with `npm run build:fixtures` before running validation.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 3.1 established the debugger-facing evidence path and explicitly positioned it as the foundation for later invalidation and performance work. Story 3.5 should consume that contract rather than rebuild it.
- Story 3.2 made notation safety explicit, so this story can characterize performance without reopening notation semantics or weakening parser-local regression coverage.
- Story 3.3 established truthful flow-scoped recalculation, including hidden-flow no-op behavior, reuse-based fallback, and Rust-owned recalculation requests. Story 3.5 should measure and document how those branches behave in representative scenarios.
- Story 3.4 is the immediate predecessor and frames visual stability as distinct from performance characterization. Story 3.5 should measure the resulting behavior, not absorb Story 3.4's implementation scope.

### Git Intelligence Summary

- `09e9b6d5 feat(3.3): isolate recalculation to affected flow` confirms the current Epic 3 baseline already has a dedicated recalculation-request model and truthful scope reporting that this story should validate rather than replace.
- `700e7dda test(3.2): add multi-flow notation safety regressions` reinforces the repo pattern of pairing render-path changes with guardrail coverage before relying on performance evidence.
- `b091f8ff feat(2.1): flow rendering debug info` shows the established maintainer habit of surfacing backend render behavior through debugger diagnostics instead of ad hoc logging.
- `61d9a08c chore(3.1): prep` reflects the deliberate sequencing of Epic 3 from observability into targeted isolation and then validation.

### Project Context Reference

- `docs/project-context.md` defines the shared-surface render pipeline, the Rust-owned diagnostics contract, and the rule that the debugger can expose render scope details without creating a frontend-owned source of truth.
- `docs/architecture.md` states that draw instructions are cached on `RendererContext`, recalculation enters a flow-preparation seam before parse/draw work, and timing payloads remain Rust-owned even when the debugger presents them.
- `_bmad-output/planning-artifacts/architecture.md` requires performance instrumentation, targeted regression testing, and debugger support to remain part of the implementation architecture rather than optional team process.
- `_bmad-output/planning-artifacts/epics.md` defines Story 3.5 as the performance-characterization increment that should confirm whether multi-flow work scales with affected flows while preserving the single-flow baseline.
- `_bmad-output/implementation-artifacts/3-1-add-multi-flow-render-and-recalculation-visibility.md`, `_bmad-output/implementation-artifacts/3-3-isolate-recalculation-to-the-affected-flow.md`, and `_bmad-output/implementation-artifacts/3-4-keep-non-edited-flows-visually-stable-during-local-updates.md` provide the immediate contracts this story must measure and preserve.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.5 acceptance criteria)
- Source: `_bmad-output/planning-artifacts/architecture.md` (instrumentation, regression, and debugger support as architectural requirements)
- Source: `docs/project-context.md` (render pipeline, diagnostics ownership, stack versions)
- Source: `docs/architecture.md` (renderer cache, flow-preparation seam, Rust-owned diagnostics)
- Source: `_bmad-output/implementation-artifacts/3-1-add-multi-flow-render-and-recalculation-visibility.md` (existing observability contract)
- Source: `_bmad-output/implementation-artifacts/3-3-isolate-recalculation-to-the-affected-flow.md` (flow-scoped recalculation and reuse baseline)
- Source: `_bmad-output/implementation-artifacts/3-4-keep-non-edited-flows-visually-stable-during-local-updates.md` (visual-stability predecessor and Story 3.5 boundary)
- Source: `src-tauri/src/lib.rs` (render entry point and timing emit seam)
- Source: `src-tauri/src/instructions/render_plan.rs` (prepared/composed/recalculated flow reporting)
- Source: `src-tauri/src/instructions/mod.rs` (shared-surface composition and fallback decision)
- Source: `src/windows/debugger/components/performance-timings.tsx` (maintainer-facing timing UI)
- Source: `src/windows/debugger/components/performance-timings.test.tsx` (existing debugger regression coverage)
- Source: `package.json` (validation and fixture-build commands)
- Source: `tests/fixtures/README.md` (fixture authoring and rebuild rules)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Completion Notes List

- Story 3.5 was created from the first backlog item in `sprint-status.yaml`, which follows the existing Epic 3 sequence after Story 3.4 was contexted.
- The story is framed as a validation increment over the current Rust-owned observability, recalculation, and shared-surface reuse seams, not as a separate optimization or benchmark-framework effort.
- The story context was grounded in Epic 3 planning, the planning architecture artifact, the repo-level project context, the current Rust render-plan and debugger seams, Story 3.3 implementation notes, Story 3.4 follow-on constraints, and recent git history.
- The repository does not contain `_bmad/core/tasks/validate-workflow.xml`, so the BMAD validator task could not be invoked and the create-story checklist was applied manually while preparing this artifact.
- External web research was not available in this environment; repository documentation and source were used as the authoritative baseline.
- No persisted-data migration is expected for this story.
- The diagnostics audit confirmed that the existing `render-timing-flow-preparation-details` and `render-timing-surface-details` payloads already distinguish prepared, composed, and recalculated scopes well enough to compare the baseline, shared multi-flow render, and local-edit reuse paths. No payload expansion or debugger UI change was required.
- No fixture JSON changes were needed. The validation scenarios are now documented in `TESTING.md`, and the Rust test seam synthesizes representative scenarios locally without introducing a repo-wide benchmark framework.
- Added Rust guardrails in `src-tauri/src/instructions/render_plan.rs` and `src-tauri/src/instructions/mod.rs` to prove that multi-flow local edits can remain single-flow recalculations when reuse is available and that broad fallback expands truthfully to all visible flows only when reuse is unavailable. These automated checks cover scope/reuse truthfulness; the timing threshold remains a manual characterization step.
- Measured warm-pass debug characterization evidence during development from the temporary characterization harness before the final standalone workflow was isolated into `src-tauri/src/bin/performance.rs`:
  - `baseline_single_flow` (3 instruments, 30 bars): `prepared_us=4`, `segment_us=8640`, `compose_us=76`, `composed_flows=1`, `recalculated_flows=1`.
  - `shared_two_flow`: `prepared_us=6`, `segment_us=17924`, `compose_us=163`, `composed_flows=2`, `recalculated_flows=2`.
  - `local_edit_three_flow`: `prepared_us=5`, `segment_us=8911`, `compose_us=235`, `composed_flows=3`, `recalculated_flows=1`.
- Refreshed current release-like characterization evidence from the standalone `performance` binary workflow on 2026-03-12 via `npm run perf:render -- --json`:
  - `baseline_single_flow` (3 instruments, 30 bars): `prepared_us median=0`, `segment_us median=1189`, `compose_us median=13`, `total_us median=1201`, `composed_flows=1`, `recalculated_flows=1`.
  - `shared_two_flow`: `prepared_us median=0`, `segment_us median=1887`, `compose_us median=26`, `total_us median=1916`, `composed_flows=2`, `recalculated_flows=2`.
  - `local_edit_three_flow`: `prepared_us median=0`, `segment_us median=1028`, `compose_us median=37`, `total_us median=1071`, `composed_flows=3`, `recalculated_flows=1`.
- The refreshed release-like baseline establishes the current representative reference point at `1201us` total median for the single-flow scenario. That remains below the newly adopted `2ms` regression-warning threshold, while the three-flow local-edit pass stays close to the single-flow cost and preserves a one-flow recalculation set. The shared two-flow full render remains predictably broader without evidence of hidden whole-surface fallback during the representative local-edit scenario.
- The standalone `performance` binary validates the backend render-plan and shared-surface composition seams directly, but it does not exercise the full Tauri event-emission and debugger subscription path end to end. Maintainers should continue using the debugger timing panel when validating the shipped observability contract.
- `npm run test:js` passed after the change set. The suite still emits pre-existing stderr noise from `plugins.test.tsx`, `write/index.test.tsx`, and `sequencer-track.test.tsx`, but those messages are warnings/logged errors rather than test failures.

### Review Fixes Applied

- Reconciled the story record and planning artifacts with the refreshed current-state baseline and the adopted `2ms` regression-warning threshold.
- Clarified that the added automated guardrails cover recalculation scope and reuse behavior, while timing validation remains a manual characterization workflow.
- Corrected the maintainer documentation so `npm run perf:render` is described as a backend characterization tool rather than a full debugger/event-path replacement, and documented the need to reevaluate the baseline as features land.

### File List

- `_bmad-output/implementation-artifacts/3-5-verify-non-regression-and-performance-scaling-for-multi-flow-rendering.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `TESTING.md`
- `docs/architecture.md`
- `docs/project-context.md`
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/src/bin/performance.rs`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/instructions/render_plan.rs`
- `src-tauri/src/renderer/mod.rs`

## Senior Developer Review (AI)

### Reviewer

Aled

### Date

2026-03-12

### Outcome

Approved

### Summary

- Updated the planning and maintainer docs to use the refreshed current-state single-flow measurement as the active baseline and to treat `2ms` total median as the current regression-warning trigger.
- Kept the automated guardrails scoped to flow-scope and reuse truthfulness while documenting that timing characterization remains manual and must be reevaluated as performance-sensitive features land.
- Reconciled the Story 3.5 record, File List, and sprint status with the approved threshold decision and refreshed performance samples.

### Change Log

- 2026-03-12: Added representative render-plan and shared-surface reuse guardrails, documented the multi-flow performance-validation workflow, and recorded debug plus release-like characterization samples for the 3-instrument/30-bar baseline and representative multi-flow scenarios.
- 2026-03-12: Finalized the manual characterization workflow as a standalone release binary in `src-tauri/src/bin/performance.rs` so performance validation stays isolated from both the shipped app API surface and the normal Rust test flow.
- 2026-03-12: Adopted the refreshed current-state release-like single-flow sample as the active baseline, set `2ms` total median as the regression-warning threshold, and aligned the story and docs with the new characterization policy.
