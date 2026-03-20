# Story 5.2: Protect Existing Non-Tuplet Behavior From Tuplet Regressions

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want non-tuplet notation and playback behavior protected while tuplets are added,
so that the new feature does not silently degrade existing musical output.

## Scope Note

- This story is about preserving existing product contracts for non-tuplet scores. It is not a tuplet feature expansion story.
- The goal is to encode representative non-tuplet baselines close to the seams that tuplets touched, so later tuplet work cannot accidentally degrade older notation, playback, or load behavior.
- Existing simple and compound non-tuplet cases remain in scope. Unsupported complex signatures and post-MVP tuplets growth work remain out of scope.
- The story should leave a maintainer-visible baseline that distinguishes intended tuplet behavior from accidental regressions in older rhythm handling.

## Acceptance Criteria

1. **Given** existing non-tuplet notation behavior relied on by the product
   **When** tuplets are introduced into parser, renderer, and playback paths
   **Then** regression checks cover representative non-tuplet scenarios
   **And** unchanged behavior remains explicitly protected rather than assumed

2. **Given** future tuplet work expands beyond MVP
   **When** new changes are made later
   **Then** the regression baseline established here still distinguishes intended tuplet changes from accidental regressions in older notation behavior
   **And** existing non-tuplet correctness remains a protected product contract

## Tasks / Subtasks

- [x] Define the non-tuplet regression baseline across the seams tuplets already changed (AC: 1, 2)
  - [x] Identify representative parser scenarios that must remain unchanged for non-tuplet notation, including ordinary simple-meter and supported compound-meter patterns.
  - [x] Identify representative renderer behavior that must remain unchanged when no tuplets are present.
  - [x] Identify representative playback behavior that must remain unchanged for host-only material.
  - [x] Identify representative file-workflow behavior that must remain unchanged for legacy and current non-tuplet projects.
- [x] Add or extend parser regressions for non-tuplet behavior in local ownership seams (AC: 1, 2)
  - [x] Reuse the current scenario tables in `src-tauri/src/instructions/parse/get_written_durations.rs` and parser-local helpers rather than inventing a new notation harness.
  - [x] Keep compound-meter coverage aligned with existing supported behavior and avoid introducing unsupported complex signatures in fixtures or tests.
  - [x] Prefer musically named scenarios that make it obvious which older behavior failed.
- [x] Add or extend rendering and hit-testing regressions proving non-tuplet behavior still reads the same without tuplets present (AC: 1)
  - [x] Protect the existing draw and selection seams against accidental tuplets-specific assumptions leaking into host-only rendering paths.
  - [x] Confirm non-tuplet scores do not require tuplet placeholders, selection payloads, or special-case layout behavior to render correctly.
- [x] Add or extend playback regressions for host-only scheduling and flow behavior (AC: 1, 2)
  - [x] Reuse Story 4.3 and 4.4 transport coverage patterns to prove host-only playback remains stable when tuplets-related scheduling logic exists in the codebase.
  - [x] Keep non-tuplet playback proof in the existing transport tests rather than in a new transport harness.
- [x] Add or extend persistence regressions proving non-tuplet files remain unchanged in the existing load path (AC: 1)
  - [x] Preserve the explicit legacy non-tuplet load path coverage in `src-tauri/src/files.rs`.
  - [x] Confirm no tuplet-specific migration or load-time repair logic alters non-tuplet payloads.
- [x] Validate the protected baseline with standard repo checks
  - [x] Run focused Rust tests for touched parser, draw, playback, and file-workflow seams.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml`.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
  - [x] Run `npm run lint` and `npm run check-types` if any frontend-facing seams or fixtures are touched.
  - [x] Run `npm run build:fixtures` if fixture JSON changes.

## Dev Notes

### Story Foundation

- The PRD explicitly says tuplets must not degrade existing notation trust. Story 5.2 turns that requirement into a durable regression baseline.
- Earlier stories proved tuplets work. This story proves that ordinary non-tuplet scores still behave exactly as maintainers and users already expect.
- The most important output is not broad coverage percentage. It is a set of representative, readable regressions that fail when future tuplets work accidentally changes non-tuplet behavior.

### Technical Requirements

- Protect representative host-only behavior across parser, draw, playback, and file-loading seams.
- Reuse existing scenario-driven tests and fixtures where possible instead of building a parallel baseline framework.
- Preserve the current supported meter boundaries:
  - simple meters remain in scope
  - supported compound meters remain in scope
  - unsupported complex signatures remain out of scope
- Keep the baseline explicit enough that future tuplet work can tell the difference between intended tuplet changes and accidental regressions in older behavior.

### Architecture Compliance

- Tuplets were introduced as additive shared-model behavior. Non-tuplet scores must continue to move through the same parser, renderer, playback, and persistence seams without hidden reinterpretation.
- Older host-only notation must not become dependent on tuplet metadata, tuplet-owned timelines, or special-case selection structures.
- Persistence and migration behavior for non-tuplet projects must remain explicit and version-driven, not opportunistically rewritten during load.

### Library / Framework Requirements

- Continue using Rust `#[cfg(test)]` modules close to the owning code.
- Reuse the committed fixture approach under `tests/fixtures/source/` only if parser-local assertions cannot prove the non-tuplet baseline directly.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary parser seam:
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
- Parser-local support:
  - `src-tauri/src/instructions/parse/test_support/get_written_durations_test_harness.rs`
- Likely draw and hit-test seams:
  - `src-tauri/src/instructions/draw/`
  - `src-tauri/src/renderer/hit_test.rs`
- Playback seam:
  - `src-tauri/src/playback/transport.rs`
- Persistence seam:
  - `src-tauri/src/files.rs`
- Fixture touchpoints if needed:
  - `tests/fixtures/source/rhythms/`
  - `tests/fixtures/README.md`

### Testing Requirements

- Preserve existing non-tuplet parser behavior with scenario-driven tests for ordinary notation.
- Preserve host-only playback behavior and active-flow behavior established in Epic 4.
- Preserve legacy non-tuplet file loading and current non-tuplet save/load behavior.
- If fixtures are updated, keep them grouped by concern and do not commit built `.soloc` outputs.

### Previous Story Intelligence

- Story 3.2 established a strong precedent for local parser regressions and meter-aware written-duration baselines.
- Story 4.3 already proved host-only playback behavior after tuplet scheduling logic landed, which gives this story a direct playback starting point.
- Story 1.6 already proved legacy non-tuplet files still load through the current typed file path, so this story should extend that seam rather than rebuilding it.

### Git Intelligence Summary

- Recent relevant commits:
  - `57591fe3` `feat(4.4): keep MVP playback scoped to active flow`
  - `3c60c495` `feat(4.3): preserve host-only playback behavior`
  - `9e867590` `feat(4.2): finalize tuplet playback ordering`
- The codebase has already been hardened incrementally. Story 5.2 should keep that pattern by turning existing knowledge about non-tuplet behavior into explicit regression proof.

### Documentation Impact

- Documentation updates are only required if the story clarifies a durable testing or supported-behavior baseline that maintainers need to follow.
- If the supported non-tuplet baseline is clarified materially, update `docs/development-guide.md`, `docs/project-context.md`, or `tests/fixtures/README.md` in the same change.

### Project Structure Notes

- The repo already contains substantial written-duration scenario coverage. Start there before reaching for fixtures or broader render-level tests.
- Existing fixture rules require `npm run build:fixtures` after source JSON changes and prohibit committing built `.soloc` files.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 5 / Story 5.2 acceptance criteria
- Source: `_bmad-output/planning-artifacts/prd.md` — success metric that existing non-tuplet projects keep working without unrelated visible changes
- Source: `_bmad-output/planning-artifacts/architecture.md` — additive tuplets model and migration discipline
- Source: `docs/project-context.md` — exact shared-model, playback, and migration guardrails
- Source: `docs/development-guide.md` — tuplets guardrails and timing invariant phrasing
- Source: `_bmad-output/implementation-artifacts/1-6-round-trip-tuplet-data-through-save-load-and-migration.md`
- Source: `_bmad-output/implementation-artifacts/3-2-add-regression-protection-for-multi-flow-notation-safety.md`
- Source: `_bmad-output/implementation-artifacts/4-3-preserve-existing-non-tuplet-playback-behavior.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Turn existing known-good non-tuplet behavior into explicit regression proof at each seam tuplets touched.
- Prefer extending local scenario tables, transport tests, and file-workflow regressions over building new generalized coverage infrastructure.
- Keep the baseline readable enough that later tuplet work can tell what broke and where.

### Debug Log

- Audited the non-tuplet baseline across parser, draw, playback, and file seams touched by the tuplet work; existing compound/simple parser corpora and host-only transport/file tests were already substantial, but explicit current-story baselines were still missing.
- Added only seam-local regressions in existing Rust test modules; no fixtures, migrations, or frontend paths were changed.

### Completion Notes

- Added a host-only 6/8 dotted-quarter parser regression to keep supported compound-meter spelling explicit in the non-tuplet baseline.
- Extended renderer-side proof with a tuplet-aware rest selection payload test and cross-context tie rendering regression so host-only draw paths remain distinct from tuplet-specific ownership handling.
- Added host-only playback regression proof that the selected active flow stays integer-timed even when sibling flows contain tuplets.
- Added a current non-tuplet round-trip regression proving the load/save path does not invent tuplet entries; existing legacy non-tuplet load coverage remained in place.
- Validated with focused Rust tests, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.
- No docs or fixture updates were required because supported non-tuplet behavior did not change; the work only made that baseline more explicit in seam-local tests.

### File List

- src-tauri/src/files.rs
- src-tauri/src/instructions/draw/draw_rest.rs
- src-tauri/src/instructions/draw/draw_ties.rs
- src-tauri/src/instructions/parse/get_written_durations.rs
- src-tauri/src/playback/transport.rs
- \_bmad-output/implementation-artifacts/5-2-protect-existing-non-tuplet-behavior-from-tuplet-regressions.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-03-18: Added parser, rendering, playback, and file-workflow regressions that preserve supported non-tuplet behavior alongside the tuplet MVP baseline; validated full Rust suite and clippy.
