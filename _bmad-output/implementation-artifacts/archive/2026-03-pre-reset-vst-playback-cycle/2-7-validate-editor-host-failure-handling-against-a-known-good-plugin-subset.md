# Story 2.7: Validate Editor Host Failure Handling Against a Known-Good Plugin Subset

Status: done

## Story

As a maintainer,
I want invalid-target handling and host validation checked against a representative known-good plugin subset,
so that Compose can claim a bounded credible editor-host path without overclaiming broad compatibility.

## Acceptance Criteria

1. Given invalid, unavailable, or unsupported plugin targets can be requested, when the user triggers the editor host for one of those targets, then Compose reports failure gracefully and it does not leave behind an orphaned host window or unstable state.
2. Given Epic 2 must prove a usable plugin library rather than broad arbitrary compatibility, when plugin-host validation is run, then it is framed around a representative known-good subset of sensible, spec-aligned plugins and the result does not overclaim universal editor compatibility.
3. Given native editor hosting may vary across real-world plugins, when maintainers review the validation outcome for the known-good subset, then they can identify which tested plugins support the bounded editor-host path credibly and any limitations, exclusions, or caveats are explicit rather than implicit.

## Tasks / Subtasks

- [x] Task 1: Harden invalid-target and failure-path behavior around the bounded host seam (AC: 1)
  - [x] Verify invalid, unavailable, or unsupported targets fail without leaving orphaned host windows or stale active-session state.
  - [x] Ensure failure outcomes remain explicit and reviewable rather than silent.
  - [x] Keep failures local to the host seam and preserve general app stability.
- [x] Task 2: Define and run a representative known-good plugin subset validation pass (AC: 2, 3)
  - [x] Establish a maintainer-facing known-good subset that is representative, sensible, and explicit about why those plugins were chosen.
  - [x] Run the bounded editor-host workflow against that subset using the supported validation harness or documented manual procedure.
  - [x] Capture which plugins behave credibly, which do not, and what caveats apply.
- [x] Task 3: Record bounded credibility without overclaiming compatibility (AC: 2, 3)
  - [x] Document limitations, exclusions, and caveats explicitly.
  - [x] Keep the result framed as bounded maintainer validation rather than a universal compatibility promise.
  - [x] Avoid prematurely turning this into the public release compatibility matrix; later release-story documentation owns that broader artifact.
- [x] Task 4: Add focused failure-handling coverage and validation guidance (AC: 1, 2, 3)
  - [x] Add backend tests where feasible for invalid-target and failure-path behavior.
  - [x] Extend maintainer validation guidance for the known-good subset and explicit unsupported/failure cases.
  - [x] Update docs that describe current editor-host support and validation expectations.

## Dev Notes

### Story Intent

- Story 2.7 is the credibility and failure-handling capstone for the bounded editor-host seam established in 2.5 and stabilized in 2.6.
- The goal is not universal compatibility. The goal is a defensible claim about a representative known-good subset plus explicit limits and graceful failure behavior.

### Scope Boundaries

- In scope:
  - invalid-target handling
  - orphan-prevention and stability on failure
  - representative known-good subset validation
  - explicit caveats and support boundaries
- Out of scope:
  - Broad user-facing compatibility promises or release matrix publication. Later Epic 7 documentation owns that.
  - Assignment, playback activation, transport integration, or native-editor feature expansion.

### Cross-Story Alignment

- Story 2.5 provides the bounded open seam.
- Story 2.6 stabilizes lifecycle within a session.
- Story 2.7 proves the seam is credible for a representative subset and explicit about where it is not.
- This story should leave later public compatibility publication to the dedicated later epic.

### Previous Story Intelligence

- Current repo inspection found only minimal automated editor-host test coverage, so this story should explicitly combine targeted backend checks with a disciplined maintainer validation pass.
- The existing development guide already contains a manual validation checklist for the host POC and can be refined rather than replaced.
- The repo memory and current code both emphasize that the current host shims are intentionally minimal, which reinforces the need for honest validation boundaries.

### Architecture Compliance

- Failure handling remains backend-owned and explicit.
- Native host validation stays bounded to the supported macOS VST3 seam.
- Validation results must not imply broader product compatibility than the evidence supports.
- Do not turn known-good subset validation into a second runtime ownership model or a frontend-owned compatibility registry.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_editor_poc.rs`
  - invalid-target resolution, host-open path, and cleanup behavior
- `src-tauri/src/playback/vst3.rs`
  - bundle validation and target resolution helpers
- `docs/development-guide.md`
  - existing manual host validation checklist
- `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`
  - bounded host POC context and expectations

### Implementation Guidance

- Be explicit about the subset:
  - why each tested plugin belongs in it
  - what “credible support” means in this bounded story
  - what caveats still apply
- Prefer maintainable evidence over broad prose claims.
- Test invalid targets deliberately:
  - missing bundle
  - unavailable target
  - unsupported or attach-failing plugin
- Preserve orphan-prevention as a first-class requirement. A failed host attempt that leaves stale state behind should count as a story failure.

### File Structure Requirements

- Backend validation work should stay close to:
  - `src-tauri/src/playback/plugin_editor_poc.rs`
  - `src-tauri/src/playback/vst3.rs`
- Documentation and validation artifacts should stay in existing maintainer-facing docs rather than inventing a separate product-facing matrix this early.

### Testing Requirements

- Add backend tests where feasible for:
  - invalid target rejection
  - failure-path cleanup
  - no orphaned host state after failed open attempts
- Extend maintainer validation docs/checklists for:
  - representative known-good subset passes
  - explicit unsupported/failure cases
  - caveat recording
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `cargo test --lib plugin_editor_poc`
  - `cargo clippy --lib -- -D warnings`

### Documentation Impact

- Update maintainer-facing docs and tech specs to record the validated known-good subset, failure-handling expectations, and limitations.
- Likely touch points:
  - `docs/development-guide.md`
  - `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`
  - `TESTING.md` if validation guidance belongs there after implementation

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 2.7 acceptance criteria
- `_bmad-output/planning-artifacts/architecture.md` — plugin lifecycle/editing ownership and explicit validation seams
- `_bmad-output/implementation-artifacts/2-5-open-a-bounded-native-plugin-editor-host-for-supported-plugins.md`
- `_bmad-output/implementation-artifacts/2-6-stabilize-native-editor-host-lifecycle-within-a-session.md`
- `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`
- `docs/development-guide.md`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/vst3.rs`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.7`.
- Repo inspection showed this story is the right place to formalize known-good-subset validation and honest limitation reporting because earlier editor-host stories focus on opening and lifecycle.
- Cross-story planning intentionally left public compatibility publication for later release-focused work.
- Added an explicit pre-window unsupported-target gate for the currently unsupported Steinberg family so the maintainer action fails locally before reserving host lifecycle state.
- Recorded the bounded validation subset around `BBC Symphony Orchestra.vst3` and documented why Steinberg-family bundles remain excluded in the current proof host.

### Completion Notes List

- Story context turns failure handling and subset validation into explicit deliverables instead of implied follow-up.
- Story context preserves bounded credibility without overclaiming universal editor compatibility.
- Story context keeps validation evidence maintainer-facing at this stage.
- `plugin_editor_poc.rs` now rejects Steinberg-family editor-host targets before creating or reserving a host window, with a maintainer-facing error that explains the bounded exclusion and points back to the known-good subset.
- Added focused Rust coverage for unsupported-target detection and for proving the rejection path leaves host lifecycle state idle.
- Updated maintainer-facing docs and the host tech spec to record the current known-good control case (`BBC Symphony Orchestra.vst3`), the explicit Steinberg-family exclusion, and the expected local failure behavior.
- Validation passed with `cargo test --lib plugin_editor_poc`, `cargo clippy --lib -- -D warnings`, `npm run lint`, and `npm run check-types`.

### File List

- `_bmad-output/implementation-artifacts/2-7-validate-editor-host-failure-handling-against-a-known-good-plugin-subset.md`
- `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `TESTING.md`
- `docs/development-guide.md`
- `src-tauri/src/playback/plugin_editor_poc.rs`

## Change Log

- 2026-03-22: Added explicit unsupported-target handling for the currently unsupported Steinberg editor-host family, documented the bounded known-good subset around BBC Symphony Orchestra, and validated the host seam with focused Rust tests, clippy, lint, and typecheck.
