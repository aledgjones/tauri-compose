# Story 1.6: Round-Trip Tuplet Data Through Save, Load, and Migration

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a desktop document user,
I want triplet-authored scores to save and reopen safely,
so that exact rhythmic meaning survives normal file workflows and older files remain safe.

## Scope Note

- This dev-cycle slice focuses on deterministic tuplet save/load round-tripping through the existing `.soloc` path. It does not add a new migration for tuplets.
- The reason migration is explicitly out of scope here is that files created before tuplet support do not contain tuplet payloads, so there is no released legacy tuplet data that requires structural upgrade to preserve meaning.
- Legacy non-tuplet files must still load unchanged under the current version-validation and migration flow. This story must not weaken that safety contract.
- This story does not pull notation derivation, playback projection, or release-preparation work forward. It only guarantees the persisted tuplet data needed for those later seams to read the same backend-owned timing model.

## Acceptance Criteria

1. Given a score containing supported tuplets entered through the popover and sequencer workflow, when the project is saved and reopened, then the tuplet container, child entries where present, grouping semantics, continuous cross-boundary tones, and exact rhythmic intent round-trip without loss, and visible notation intent remains recoverable from the persisted data.
2. Given a score containing host-owned tones that cross a tuplet boundary and child-owned tones that start inside a tuplet, when save/load round-trip occurs, then each tone remains exactly one persisted event owned by where it starts, and the load path does not force storage splits, duration rewriting, or ownership drift solely because a tuplet boundary exists.
3. Given the project is saved repeatedly after tuplet authoring and reopened again, when the persisted payload is inspected across round-trips, then tuplet-related project data remains deterministic and structurally stable through the existing `ProjectFile` and `.soloc` container path.
4. Given a legacy non-tuplet file created before tuplet support, when it is opened under the current model, then it loads correctly without requiring a semantic tuplet upgrade, and files with no tuplet payload continue through the existing load and version-validation flow unchanged.
5. Given this story is implemented during the current dev cycle before any released tuplet-bearing files exist, when the persistence work is completed, then no new tuplet-specific migration step is added in `src-tauri/src/migrations/next.rs`, and the story documents that decision explicitly instead of hiding migration assumptions in unrelated load logic.

## Tasks / Subtasks

- [x] Harden tuplets in the existing save/load seam rather than creating a parallel persistence path (AC: 1, 2, 3)
  - [x] Verify `ProjectFile` serialization/deserialization in `src-tauri/src/files.rs` continues to round-trip `Entry::Tuplet`, child `TupletEntries`, grouping metadata, and host-owned crossing tones through the typed project payload.
  - [x] Keep the `.soloc` container path in `src-tauri/src/soloc.rs` as the single save/load transport for tuplet-bearing projects.
  - [x] Preserve the one-stored-event-per-tone rule during load so crossing tones are not split or normalized just because tuplets now exist in persisted score data.
- [x] Preserve deterministic persisted tuplet structure across repeated round-trips (AC: 1, 2, 3)
  - [x] Reuse or extend the existing round-trip helpers and tests near `src-tauri/src/files.rs` so save-load-save cycles prove stable serialized structure for representative tuplet cases.
  - [x] Cover a case with child-owned tuplet tones and a case with host-owned tones crossing a tuplet boundary.
  - [x] Keep `score.meta.app_version` and the `.soloc` header app version aligned with the existing file workflow rules.
- [x] Preserve legacy non-tuplet file behavior without inventing unnecessary migration work (AC: 4, 5)
  - [x] Confirm legacy files with no tuplets still pass through the current migration and version-validation flow unchanged.
  - [x] Do not add a tuplet-specific migration in `src-tauri/src/migrations/next.rs` for this story, because there is no released tuplet-bearing file format to transform yet.
  - [x] Document the no-migration rationale in developer-facing notes so later persistence work can revisit it deliberately if that assumption changes.
- [x] Add focused regression coverage close to the owning persistence modules (AC: 1, 2, 3, 4)
  - [x] Add or extend Rust tests in `src-tauri/src/files.rs` and nearby owning modules proving tuplet round-trips, legacy non-tuplet files still load, and crossing-tone ownership survives reopen unchanged.
  - [x] Prefer local persistence tests over disconnected harnesses; keep the assertions near the loader and container code that owns the behavior.
  - [x] Run the relevant checks for touched code paths: `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`, `npm run lint`, and `npm run check-types`.
- [x] Update documentation for the current persistence contract and no-migration decision (AC: 4, 5)
  - [x] Update the relevant developer-facing docs if implementation clarifies how tuplets round-trip through `.soloc` without additional migration work in the current dev cycle.
  - [x] Ensure the docs say legacy non-tuplet files remain unchanged and that future migration work is only required if persisted compatibility needs actually emerge.

## Dev Notes

### Story Foundation

- Story 1.1 introduced `Entry::Tuplet` as persisted score data and already established the `.soloc` save/load seam as the authoritative path for round-tripping that new entry type.
- Story 1.5 established the ownership rule that crossing tones remain single persisted events owned by where they start, while tones that start inside tuplets live in child `entries`.
- Story 1.6 is the persistence hardening step that proves the current data model survives normal desktop document workflows without adding a speculative migration that has no real legacy tuplet data to transform.
- Later notation and playback stories still depend on this story preserving one exact backend-owned timing model across load/save boundaries.

### Technical Requirements

- Work in the existing typed `ProjectFile` and `.soloc` container flow. Do not create a second tuplet-specific loader, serializer, or ad hoc repair path.
- Save/load must preserve the current ownership invariants exactly:
  - tones that start outside tuplets remain host-owned even if they extend into a tuplet
  - tones that start inside tuplets remain child-owned even if they extend out
  - every tone remains one stored event through persistence round-trips
- Persisted tuplet metadata must remain sufficient to recover grouping semantics and exact rhythmic intent later in parser, renderer, and playback stories.
- Repeated round-trips must be deterministic. The reopened project should serialize back to the same effective structure for representative tuplet cases.
- Keep the current app-version validation discipline in place. This story should not weaken container/header validation just to make tuplets load.
- Do not add `src-tauri/src/migrations/next.rs` work for tuplets in this story unless implementation reveals an actual incompatible older payload that cannot load correctly. The current planning assumption is that no such released tuplet payload exists.

### Architecture Compliance

- The architecture keeps tuplets as first-class persisted entries on instrument tracks with child `entries` for tones whose starts fall inside the governed span.
- Save/load must honor the non-negotiable ownership rules: one tone is one stored event, ownership is decided by start position, and boundary splits remain a derivation concern rather than a persistence rewrite.
- The loader should continue to apply migrations to JSON before typed deserialization when needed, but this story's expected outcome is that tuplets need no new migration because pre-tuplet files contain no tuplets.
- Rust remains the source of truth. Do not introduce any frontend-owned persistence reconstruction layer for tuplets.

### Library / Framework Requirements

- Backend persistence work stays in the current Rust/Tauri stack using `serde`, the existing `.soloc` container helpers, and the current semver-based migration registry.
- Follow the current command and state-management patterns in `src-tauri/src/lib.rs` and related file commands.
- No new external dependency is expected for this story.

### File Structure Requirements

- Primary backend persistence seams:
  - `src-tauri/src/files.rs`
  - `src-tauri/src/soloc.rs`
  - `src-tauri/src/score/entries/tuplet.rs`
  - `src-tauri/src/score/entries/mod.rs` if entry serde coverage or helper behavior needs adjustment
- Migration review touchpoints:
  - `src-tauri/src/migrations/mod.rs`
  - `src-tauri/src/migrations/next.rs` for explicit non-change verification, not expected implementation work
- Documentation touchpoints if behavior or rules need clarification:
  - `docs/data-models.md`
  - `docs/architecture.md`
  - `docs/project-context.md`
- Keep frontend work out of scope unless a persistence-facing type contract genuinely requires a small sync update.

### Testing Requirements

- Verify tuplet-bearing projects round-trip through the current save/load path without losing child entries, grouping metadata, or ownership semantics.
- Verify a host-owned tone that crosses a tuplet boundary remains one host-owned persisted tone after reopen.
- Verify a child-owned tone that starts inside a tuplet remains in child `entries` after reopen.
- Verify repeated round-trips stay deterministic for representative tuplet-bearing project data.
- Verify legacy non-tuplet projects still load correctly with the current version-validation and migration path unchanged.
- Run the relevant frontend and Rust validation commands after implementation.

### Previous Story Intelligence

- Story 1.1 already added the persisted tuplet entry shape and initial `.soloc` round-trip coverage. Story 1.6 should extend that seam instead of rebuilding it.
- Story 1.3A simplified child content to a single-level `TupletEntries` model, reducing persistence complexity for this story.
- Story 1.4 established sequencer note entry into tuplets, which means Story 1.6 must prove those authored child entries survive reopen intact.
- Story 1.5 established reconciliation rules for existing material and crossing tones. Story 1.6 must preserve those same persisted ownership decisions after save/load.

### Git Intelligence Summary

- Recent relevant commits:
  - `4c8a33bf feat(1.5): reconcile tuplets over existing material`
  - `37a3ee54 feat(1.4): add tuplet note entry in sequencer`
  - `1ede423a feat(1.3a): tuplet entries structures non-recursive`
  - `356b5203 feat: course correct`
  - `38c8bede fix: tweak tuple background colour`
- Recent work has progressed seam by seam. Story 1.6 should follow that same pattern by tightening the existing file workflow rather than broadening into release automation, notation, or playback concerns.

### Documentation Impact

- Document that tuplets round-trip through the existing `.soloc` and typed `ProjectFile` path with no separate persistence channel.
- Document the explicit no-migration rationale for this story: pre-tuplet files contain no tuplet payloads, so there is no released tuplet-bearing data that needs transformation in the current dev cycle.
- If implementation reveals a stronger persistence rule or edge case, update the relevant docs rather than leaving it implicit in tests only.

### Project Structure Notes

- The current save/load architecture already uses container-header app versioning, JSON migration on `serde_json::Value`, and a single typed deserialize before app-state mutation.
- Existing persistence tests in `src-tauri/src/files.rs` already include a tuplet round-trip case; prefer extending those tests with ownership-specific and legacy-file assertions rather than inventing a disconnected test harness.
- The project rule that persisted-data changes need migration review still applies. For this story, the review outcome is expected to be “no migration required right now,” not “migration discipline is suspended.”

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 / Story 1.6 and Story 1.7
- Source: `_bmad-output/planning-artifacts/prd.md` — persistence, load/save, and migration requirements
- Source: `_bmad-output/planning-artifacts/architecture.md` — migration impact analysis; persisted tuplet invariants; one-stored-event-per-tone rule
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — tuplet creation and sequencer entry flow that produces the persisted child content
- Source: `docs/project-context.md` — Rust/Tauri patterns and migration workflow rules
- Source: `docs/architecture.md` — current `.soloc` container and migration pipeline summary
- Source: `docs/data-models.md` — migration system rules and persistence expectations
- Source: `_bmad-output/implementation-artifacts/1-1-add-persisted-tuplet-container-entries.md`
- Source: `_bmad-output/implementation-artifacts/1-3a-simplify-tuplet-child-content-for-non-nestable-mvp.md`
- Source: `_bmad-output/implementation-artifacts/1-4-enter-tuplet-notes-in-the-sequencer.md`
- Source: `_bmad-output/implementation-artifacts/1-5-create-tuplets-over-existing-material-without-losing-ownership.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Inspect the existing `.soloc` save/load path in `src-tauri/src/files.rs` and `src-tauri/src/soloc.rs` to confirm tuplet-bearing project data stays within the current typed `ProjectFile` pipeline.
- Extend persistence-focused Rust coverage to prove deterministic round-trips for tuplets, especially for child-owned tuplets and host-owned crossing tones.
- Preserve legacy non-tuplet loading behavior and leave `src-tauri/src/migrations/next.rs` unchanged unless implementation uncovers a real incompatible older payload.

### Debug Log References

- Capture any added or updated persistence round-trip tests in `src-tauri/src/files.rs` and related owning modules.
- Record validation runs for `cargo test`, `cargo clippy`, `npm run lint`, and `npm run check-types` after implementation.
- Added tuplets persistence regression coverage in `src-tauri/src/files.rs` for child-owned tones, host-owned crossing tones, deterministic save-load-save structure, and legacy non-tuplet load-path continuity.
- Validation complete: `cargo test --manifest-path src-tauri/Cargo.toml -q`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`, `npm run lint`, and `npm run check-types` all passed on 2026-03-15.
- Review fixes applied on 2026-03-15: the deterministic round-trip assertion now compares decompressed persisted payload bytes in addition to typed JSON shape, the legacy non-tuplet round-trip test now loads a committed pre-tuplet fixture payload instead of a synthetic current-schema object, and the reopen path now shares a decode/apply helper that is covered by a state-reset and transport-sync regression test.

### Completion Notes List

- Story prepared with migration explicitly out of scope for the current dev-cycle slice because there are no released pre-tuplet files containing tuplet payloads.
- Save/load round-trip behavior, deterministic persistence, and legacy non-tuplet pass-through remain the controlling outcomes for implementation.
- Verified the existing `ProjectFile` plus `.soloc` seam remains the only tuplet persistence path; no parallel loader or repair flow was introduced.
- Added ownership-focused persistence assertions proving host-owned crossing tones stay on the track, child-owned tones stay in `Tuplet.entries`, and repeated round-trips keep the serialized structure deterministic.
- Documented the current no-migration rationale in developer-facing docs and left `src-tauri/src/migrations/next.rs` unchanged because no released tuplet-bearing payload needs transformation.
- Follow-up code review fixes tightened AC coverage by proving deterministic persisted JSON bytes across save-load-save cycles, exercising a real older fixture payload through the shared `.soloc` decode path, and verifying reopen resets ephemeral UI state while syncing transport state for tuplet-bearing projects.

### File List

- `_bmad-output/implementation-artifacts/1-6-round-trip-tuplet-data-through-save-load-and-migration.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/data-models.md`
- `src-tauri/src/files.rs`

## Senior Developer Review (AI)

### Reviewer

- Aled via GPT-5.4 on 2026-03-15

### Findings Resolved

- Replaced `serde_json::Value`-only determinism checks with decompressed persisted payload byte equality plus typed equality so AC 3 is validated against the actual serialized project payload.
- Replaced the synthetic legacy non-tuplet round-trip case with a committed `1.23.3` fixture payload, while still forcing the shipped tempo-text migration path to run.
- Extracted shared `.soloc` decode and state-application helpers from `load_project_from_file` and added regression coverage proving reopen resets ephemeral UI state, preserves tuplets, updates `current_file_path`, and syncs transport tempo/tick.

### Validation

- `cargo test --manifest-path src-tauri/Cargo.toml files::tests::`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`
- `npm run lint`
- `npm run check-types`

## Change Log

- 2026-03-15: Added tuplet persistence regression coverage for ownership and deterministic round-trips, documented the single-path `.soloc` contract, and recorded the explicit no-migration decision for current legacy files.
- 2026-03-15: Addressed code review findings by sharing the real `.soloc` decode path with tests, validating deterministic persisted JSON bytes, and exercising reopen state application against tuplet-bearing data.
