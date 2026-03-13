# Story 3.2: Add Regression Protection for Multi-Flow Notation Safety

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want regression coverage that protects notation behavior while multi-flow rendering evolves,
so that presentation changes do not silently damage written durations or broader notation semantics.

This story is a safety-net increment for Epic 3. It should strengthen the existing parser-level and fixture-level regression seams that already characterize notation behavior, not introduce a second notation test framework or any frontend-owned interpretation of notation truth. The target is confidence that multi-flow presentation work remains a rendering-orchestration change rather than an accidental notation rewrite.

## Acceptance Criteria

1. Given the multi-flow score-view work in progress, when regression coverage is added or updated, then it explicitly protects existing written-duration behavior relevant to multi-flow rendering scenarios and confirms that presentation-layer changes do not rewrite notation semantics.
2. Given existing notation-sensitive fixtures or parser-level tests, when multi-flow-related regression coverage is introduced, then the coverage extends current test support patterns rather than creating a parallel test framework and the affected notation behaviors remain locally understandable to maintainers.
3. Given a single-flow scenario already covered by existing behavior, when the regression suite runs after multi-flow-safe changes, then current notation expectations remain intact and any new coverage does not weaken the existing baseline.
4. Given this regression-protection increment is validated, when future multi-flow stories are implemented, then maintainers have an explicit safety net for notation correctness and no persisted score-format changes are required by this story.

## Tasks / Subtasks

- [x] Audit the current notation-safety seams that multi-flow work depends on (AC: 1, 2, 3, 4)
  - [x] Confirm the existing parser-local written-duration harness in `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs` remains the primary reusable seam for scenario-driven notation assertions.
  - [x] Identify the smallest regression additions needed in `src-tauri/src/instructions/parse/get_written_durations.rs` to cover multi-flow-relevant written-duration behavior without restating the entire parser corpus.
  - [x] Identify where manual visual fixtures in `tests/fixtures/source/rhythms/` already cover notation-sensitive scenarios and where a new or expanded fixture would add meaningful confidence.
- [x] Extend parser-level notation regression coverage using existing local support patterns (AC: 1, 2, 3)
  - [x] Add or extend scenario coverage in `src-tauri/src/instructions/parse/get_written_durations.rs` for written-duration outcomes that must stay stable while multi-flow presentation changes land.
  - [x] Reuse the shared parser-local helpers under `src-tauri/src/instructions/parse/test_support/` rather than introducing a repo-wide notation testing abstraction.
  - [x] Include explicit assertions that single-flow baseline scenarios still produce the same notation outcomes after the new coverage is in place.
- [x] Add downstream notation-consumer confidence checks only where they materially protect multi-flow safety (AC: 1, 2, 3)
  - [x] Extend focused downstream tests in files such as `src-tauri/src/instructions/parse/get_beams.rs`, `src-tauri/src/instructions/parse/get_shunts.rs`, `src-tauri/src/instructions/parse/get_accidentals.rs`, `src-tauri/src/instructions/parse/get_dots.rs`, `src-tauri/src/instructions/parse/get_articulations.rs`, and `src-tauri/src/instructions/parse/get_stem_directions.rs` only if they are needed to prove notation shape stays stable.
  - [x] Keep these checks thin and local; do not turn this story into a broad new coverage initiative unrelated to multi-flow notation safety.
- [x] Strengthen manual visual-regression support for notation-sensitive score output where useful (AC: 1, 2, 4)
  - [x] Update or add rhythm fixtures under `tests/fixtures/source/rhythms/` only when a visual scenario protects notation output that parser-only tests cannot make sufficiently obvious to maintainers.
  - [x] Follow the existing fixture authoring rules: group by concern, keep one flow per test case, avoid unsupported complex time signatures, and rebuild with `npm run build:fixtures` after any fixture JSON change.
  - [x] Keep `.soloc` outputs uncommitted and rely on the existing manual visual workflow documented in `tests/fixtures/README.md`.
- [x] Document any preserved notation boundaries or test-entry expectations that future Epic 3 stories must respect (AC: 2, 4)
  - [x] Update documentation only if this story changes maintainer expectations about where notation-safety coverage belongs or how fixture-based validation should be exercised.
  - [x] Do not add migration work; if implementation suggests a persisted-score change, stop and reassess because that would violate the story boundary.
- [x] Validate the regression safety net end to end (AC: 1, 2, 3, 4)
  - [x] Run Rust notation-focused validation with the existing parser and downstream test suites.
  - [x] Run the standard repo verification expected for shipped story work: `npm run lint`, `npm run check-types`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
  - [x] Run `npm run build:fixtures` if any fixture JSON was added or changed.

## Dev Notes

### Story Foundation

- Epic 3 is about keeping multi-flow rendering musically correct while later stories narrow recalculation scope and improve performance.
- Story 3.2 is the notation-safety backstop for the rest of the epic. Stories 3.3 through 3.5 should be able to change render orchestration with confidence because notation behavior is explicitly characterized first.
- This story is not permission to redesign notation logic. Multi-flow support remains a presentation/composition concern layered over an existing Rust-owned notation engine.

### Developer Context Summary

- The repo already has a substantial written-duration regression investment in `src-tauri/src/instructions/parse/get_written_durations.rs`, including scenario tables for simple and compound meter plus parser-local shared helpers in `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`.
- The visual fixture suite already includes notation-sensitive rhythm corpora under `tests/fixtures/source/rhythms/`, including four-four, three-four, three-four-versus-six-eight comparison, and compound-meter fixture files.
- `tests/fixtures/README.md` explicitly treats these fixtures as manual visual regression rather than automated screenshot testing. This story should preserve that workflow instead of inventing a new visual test harness.
- The completed written-duration regression spec in `_bmad-output/implementation-artifacts/tech-spec-testability-regression-safety-written-duration-parsing.md` already established the local testing pattern: scenario-driven parser tests, thin downstream checks, and a separate findings document for suspicious behavior.
- Story 3.1 added observability for render and recalculation scope. Story 3.2 should build alongside that work by protecting notation semantics while later multi-flow optimization stories change invalidation behavior.

### Technical Requirements

- Protect the existing written-duration contract that multi-flow rendering depends on.
- Prove that multi-flow presentation changes do not alter notation semantics, rest handling, ties, grouping, or other written-output behavior already characterized by the parser suite.
- Extend current local test support patterns rather than introducing a repo-wide notation test framework.
- Keep regression coverage scannable and musically meaningful to maintainers; prefer scenario names and canonical assertion helpers over opaque coverage padding.
- Reuse existing manual fixture coverage only where it adds confidence that parser-level assertions cannot provide by themselves.
- Preserve current supported/unsupported boundaries for time signatures. Compound fixtures are valid coverage; complex signatures remain out of scope and should not be introduced as fixture targets.
- No persistence, migration, or `.soloc` semantic change is expected. The story should not touch `src-tauri/src/migrations/next.rs`.

### Architecture Compliance

- Rust remains the single source of truth for notation behavior, render preparation, and derived score output.
- Multi-flow work must remain a presentation/orchestration change over existing flow data rather than a rewrite of notation derivation.
- Test support should stay close to the parser modules it protects. The current shared parser-local test support location under `src-tauri/src/instructions/parse/test_support/` remains the right seam.
- Manual rhythm fixtures remain grouped under `tests/fixtures/source/rhythms/` and must be rebuilt via the existing fixture builder rather than through ad hoc conversion scripts.
- This story should improve regression confidence without coupling the frontend to notation truth or inventing frontend-owned score expectations.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep Rust test code aligned with existing parser patterns and performance-sensitive structures such as `FxHashMap` and current parse ordering.
- Use the existing fixture builder and save-path infrastructure for any fixture updates.
- External web research was not available in this environment; local repository docs and source are the authoritative baseline for this story.

### Project Structure Notes

- Primary parser seam:
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
- Shared parser-local test support:
  - `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`
  - `src-tauri/src/instructions/parse/test_support/mod.rs`
- Likely downstream notation-consumer seams if additional confidence is needed:
  - `src-tauri/src/instructions/parse/get_beams.rs`
  - `src-tauri/src/instructions/parse/get_shunts.rs`
  - `src-tauri/src/instructions/parse/get_accidentals.rs`
  - `src-tauri/src/instructions/parse/get_dots.rs`
  - `src-tauri/src/instructions/parse/get_articulations.rs`
  - `src-tauri/src/instructions/parse/get_stem_directions.rs`
- Manual visual-regression fixture seams:
  - `tests/fixtures/source/rhythms/written-durations-four-four.json`
  - `tests/fixtures/source/rhythms/written-durations-three-four.json`
  - `tests/fixtures/source/rhythms/written-durations-three-four-six-eight-comparison.json`
  - `tests/fixtures/source/rhythms/written-durations-compound.json`
  - `tests/fixtures/README.md`
- Out of scope unless implementation discovery proves otherwise:
  - new frontend score chrome
  - render-timing UI changes beyond what Story 3.1 already delivered
  - persisted-data migrations
  - a new cross-repo notation test framework

### File Structure Requirements

- Extend existing parser files and local test support before adding any new files.
- If a new fixture JSON is required, place it under `tests/fixtures/source/rhythms/` and keep one flow per case.
- Do not create new top-level test directories for this story.
- Keep `.soloc` fixture outputs untracked in `tests/fixtures/built/`.

### Testing Requirements

- Keep single-flow written-duration baseline scenarios green while adding multi-flow-relevant safety coverage.
- Extend `src-tauri/src/instructions/parse/get_written_durations.rs` using the existing scenario-driven assertion style.
- Reuse `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs` for shared parser fixture setup instead of creating duplicate helper stacks.
- Add downstream notation-consumer tests only when they materially prove notation shape remains stable for the affected scenarios.
- If fixture JSON changes, regenerate built fixtures with `npm run build:fixtures` and verify the affected `.soloc` files open correctly in the app's manual visual workflow.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
  - `npm run build:fixtures` when fixture sources change

### Previous Story Intelligence

- Story 3.1 established the maintainer-facing render-scope observability contract in the existing debugger/event pipeline rather than a parallel diagnostics surface. Follow that same pattern here: extend existing safety nets rather than building separate tooling.
- Story 3.1 also documented that Epic 3 starts with narrow enabling work before optimization. Story 3.2 should remain similarly constrained: protect notation semantics first, then let later stories optimize recalculation.
- Recent git history reinforces the local repo pattern of tightening backend-owned contracts and pairing them with targeted tests instead of creating duplicate state or framework layers.

### Git Intelligence Summary

- `61d9a08c chore(3.1): prep` shows Epic 3 work is currently being staged carefully; keep Story 3.2 narrow and explicit.
- `4d0dadb8 feat(2.5): preserve active flow across score and sequence` reinforces the repo pattern of preserving a single source of truth and verifying it with focused tests.
- `7ee83c7c feat(2.4): verify multi-flow popover anchors` shows recent multi-flow work already relies on regression coverage around precision-sensitive seams.
- `bca5d7e9 chore: epic 2 retro` confirms Epic 2 is complete, so Story 3.2 should act as enabling infrastructure for the current epic rather than backfilling earlier interaction work.

### Project Context Reference

- `docs/project-context.md` defines the native Rust render pipeline, the flow-preparation diagnostics contract, and the rule that shared parser test support for written durations lives under `src-tauri/src/instructions/parse/test_support/`.
- `_bmad-output/planning-artifacts/epics.md` defines Story 3.2 as regression protection for notation safety, with no persistence change.
- `_bmad-output/planning-artifacts/prd.md` says multi-flow rendering must preserve existing written-duration behavior and broader notation semantics; this is a presentation/orchestration change, not a notation rewrite.
- `_bmad-output/planning-artifacts/architecture.md` says regression tooling, fixture validation, and debugging support are architectural enablers for safe brownfield delivery.
- `_bmad-output/implementation-artifacts/tech-spec-testability-regression-safety-written-duration-parsing.md` already established the preferred parser-local coverage model that this story should extend.
- `tests/fixtures/README.md` defines the manual visual regression fixture workflow and existing rhythm fixture corpus.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.2 acceptance criteria)
- Source: `_bmad-output/planning-artifacts/prd.md` (notation trust, FR12-FR15, notation-risk constraints, performance/correctness boundaries)
- Source: `_bmad-output/planning-artifacts/architecture.md` (brownfield safety, regression tooling, fixture validation, Rust ownership)
- Source: `docs/project-context.md` (render pipeline, parser-local test-support rule, stack versions)
- Source: `src-tauri/src/instructions/parse/get_written_durations.rs` (existing notation regression corpus)
- Source: `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs` (shared parser-local test harness)
- Source: `_bmad-output/implementation-artifacts/tech-spec-testability-regression-safety-written-duration-parsing.md` (established written-duration regression strategy)
- Source: `tests/fixtures/README.md` (manual visual regression workflow and rhythm fixture inventory)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Existing parser-local written-duration harness: `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`
- Existing written-duration scenario corpus: `src-tauri/src/instructions/parse/get_written_durations.rs`
- Existing manual rhythm fixtures: `tests/fixtures/source/rhythms/`
- Audit confirmed the existing rhythm fixtures already cover the relevant manual visual comparison seams; no fixture JSON changes were needed for this story.
- Added parser regression coverage for equivalent 3/4 versus 6/8 material so the simple-meter baseline and compound-meter rewrite remain explicitly characterized.
- Added shared-surface composition coverage to prove a later visible flow keeps its rendered notation and hit targets intact after multi-flow offset composition.
- Validation run completed with `npm run lint`, `npm run check-types`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Completion Notes List

- Audited the existing notation-safety seams and confirmed the parser-local written-duration harness remains the primary reusable regression entry point.
- Added a focused parser regression in `src-tauri/src/instructions/parse/get_written_durations.rs` that compares equivalent 3/4 and 6/8 material so single-flow baseline spelling and compound-meter rewrites are both locked in.
- Added a focused shared-surface regression in `src-tauri/src/instructions/mod.rs` that verifies the second flow keeps its rendered notation and hit targets intact after multi-flow composition offsets are applied.
- Reviewed downstream parser consumers and existing manual rhythm fixtures; no additional fixture JSON, no documentation update, and no migration work were required.
- Validation passed with `npm run lint`, `npm run check-types`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### File List

- `_bmad-output/implementation-artifacts/3-2-add-regression-protection-for-multi-flow-notation-safety.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/instructions/parse/get_written_durations.rs`

### Change Log

- 2026-03-11: audited the existing notation-safety seams, added focused parser and shared-surface regression tests for multi-flow-safe notation behavior, and validated the repo without fixture or migration changes.
