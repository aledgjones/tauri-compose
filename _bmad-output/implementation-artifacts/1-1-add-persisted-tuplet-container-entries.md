# Story 1.1: Add Persisted Tuplet Container Entries

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want triplets represented as first-class score entries,
so that the score model can preserve exact tuplet structure instead of treating it as renderer-only decoration.

## Acceptance Criteria

1. Given the existing persisted instrument-track entry model, when foundational tuplet support is introduced, then the score model supports a persisted `Entry::Tuplet`-style container entry with its own identity, and the container stores host start tick, governed host duration, local printed duration, base note value, grouping metadata, and child entries where tones start inside the tuplet span.
2. Given a persisted tuplet container entry, when it is loaded into Rust-owned score state, then the backend treats it as authoritative score data, and no frontend-owned parallel tuplet state is required to reconstruct its rhythmic meaning.
3. Given future non-triplet tuplets are considered later, when this persisted shape is reviewed, then the MVP representation is not hard-coded to a dead-end triplet-only schema, and the model retains a viable extension path unless a more general shape is proven materially less safe.

## Tasks / Subtasks

- [x] Add the new persisted tuplet entry type to the score model and serialization layer (AC: 1, 2, 3)
  - [x] Add a new `tuplet.rs` entry module under `src-tauri/src/score/entries/` with a `Tuplet` struct that persists `id`, `tick`, `note_count`, `in_space_of`, `base_note_value`, `groupings`, and `entries`, deriving host and local tick spans from that ratio.
  - [x] Extend `Entry` in `src-tauri/src/score/entries/mod.rs` with `Entry::Tuplet(Tuplet)` and update `id()`, `tick()`, and `set_tick()` to treat tuplets as first-class entries.
  - [x] Preserve the existing dual-index storage contract in `Entries` and `Track` so tuplet containers participate in `by_tick` and `by_id` exactly like other entries.
- [x] Keep the model future-extensible instead of triplet-hard-coded (AC: 1, 3)
  - [x] Use existing generic duration and grouping types where possible (`Tick`, `Ticks`, `NoteDuration`, `Vec<u8>`) rather than introducing triplet-only fields.
  - [x] Keep child content in nested `Entries` so later tuplets can recurse through the same storage model instead of requiring a second representation.
  - [x] Document any intentionally deferred generalization if implementation must temporarily narrow behavior to supported triplets.
- [x] Wire persistence, load/save, and migration seams without ad hoc shortcuts (AC: 1, 2)
  - [x] Verify `.soloc` save/load continues to round-trip the new entry through the existing serde-based `ProjectFile` path in `src-tauri/src/files.rs` and `src-tauri/src/soloc.rs`.
  - [x] Add any unreleased persisted-data migration step needed for legacy compatibility in `src-tauri/src/migrations/next.rs`; do not hide migration behavior in unrelated load logic.
  - [x] Ensure `score.meta.app_version` remains aligned with the current app version during save/load and migration paths.
- [x] Add regression coverage close to the owning Rust modules (AC: 1, 2, 3)
  - [x] Add unit tests for `Entry::Tuplet` identity/tick behavior alongside the existing entry tests in `src-tauri/src/score/entries/mod.rs`.
  - [x] Add track-level tests in `src-tauri/src/score/tracks.rs` proving tuplets insert, shift, and remove correctly from both indices.
  - [x] Add persistence or migration-focused tests in the existing Rust test locations covering serde round-trip and legacy-file safety for the new variant.
- [x] Update developer documentation affected by the shipped persisted model (AC: 1, 2, 3)
  - [x] Update `docs/data-models.md` to document `Entry::Tuplet`, child `Entries`, and the ownership rule that start position determines where a tone is stored.
  - [x] Update any persistence or migration notes that become stale once the tuplets schema lands.

## Dev Notes

### Story Foundation

- This is the first story in Epic 1, so there is no previous-story implementation context to inherit.
- Epic 1 is the persistence and exact-timing foundation. This story is intentionally backend-first and should avoid frontend-only stopgaps.
- The architecture decision is explicit that tuplets are host-track container entries with child content, not renderer annotations and not a parallel timing model.

### Technical Requirements

- Keep Rust as the source of truth. The frontend must not invent or cache a second tuplet model just to make the new entry usable.
- Preserve the existing `Tick`/`Ticks` primitives for storage boundaries, but represent tuplet context explicitly with container metadata so exact projection can happen later at parser, render, and playback seams.
- Maintain the existing tagged-enum serialization pattern on `Entry` using `#[serde(tag = "type")]`.
- Preserve the existing dual-index storage pattern:
  - `Entries.by_tick` answers what starts at a host tick.
  - `Entries.by_id` answers what a specific entry is.
  - Tuplets must appear in both indices like every other first-class entry.
- Keep `tone_id` authoritative for tones. This story should not introduce a replacement tone identity scheme.
- Keep child content in `Entries` so later parser/render recursion can follow an already-familiar structure.
- Do not add host-level directives inside tuplet child content for MVP. The current architecture defers inner tempo, clef, key-signature, and time-signature support.

### Architecture Compliance

- The recommended persisted shape from the architecture document is:

```rust
pub struct Tuplet {
  pub id: UniqueId,
  pub tick: Tick,
  pub note_count: u8,
  pub in_space_of: u8,
  pub base_note_value: NoteDuration,
  pub groupings: Vec<u8>,
  pub entries: Entries,
}
```

- Ownership invariants to preserve:
  - One tone is always represented by exactly one stored event.
  - A tone is owned by where it starts.
  - Tones that start inside a tuplet belong in child `entries`.
  - Tones that start outside remain host-track tones even if they cross the tuplet span.
  - Crossing a tuplet boundary does not itself force a storage split.
- This story only establishes the persisted container and storage contract. It does not need to finish parser, renderer, UI, or playback projection behavior yet, but it must not block those later stories.

### Library / Framework Requirements

- Rust only for the authoritative model changes in this story.
- Continue using `FxHashMap`, never `std::collections::HashMap`.
- Continue using serde derive patterns already present in `src-tauri/src/score/entries/mod.rs`, `src-tauri/src/score/tracks.rs`, and related score structs.
- Use existing `NoteDuration` values from `src-tauri/src/score/components/duration.rs` for `base_note_value`, and derive host/local tick spans from the persisted ratio instead of storing duplicate span fields.
- No external library addition is expected for this story.
- External web research was not available in this workflow context; use the pinned project stack and architecture docs already committed in the repo.

### File Structure Requirements

- Primary code touchpoints are expected to include:
  - `src-tauri/src/score/entries/mod.rs`
  - `src-tauri/src/score/entries/tuplet.rs` (new)
  - `src-tauri/src/score/tracks.rs`
  - `src-tauri/src/files.rs`
  - `src-tauri/src/migrations/next.rs`
  - `docs/data-models.md`
- If other modules require exhaustive `match` updates on `Entry`, update them surgically and keep behavior unchanged for non-tuplet entries.
- Avoid broad parser, renderer, or UI churn in this story unless required to keep the new enum variant compiling.

### Testing Requirements

- Add or update Rust unit tests near the changed ownership points rather than introducing a detached test harness.
- Cover at minimum:
  - serde round-trip for the new `Entry::Tuplet` variant
  - `Entry::id()`, `Entry::tick()`, and `Entry::set_tick()` behavior for tuplets
  - track insertion, movement, and removal for tuplets in both `by_tick` and `by_id`
  - migration behavior if older persisted payloads need normalization or additive fields
  - non-tuplet compatibility so existing entry variants still serialize and load unchanged
- Validate with the repo’s standard Rust checks after implementation: `npm run test:rust` and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Documentation Impact

- This story changes persisted score structure, so docs are part of the implementation, not follow-up polish.
- Minimum required docs update is `docs/data-models.md`.
- If migration behavior changes materially, update any relevant migration or persistence guidance in the docs set during the same implementation.
- Do not run release scripts; unreleased migration work belongs only in `src-tauri/src/migrations/next.rs`.

### Project Structure Notes

- The repo currently documents `Entry` as having six variants. This story will change that assumption and should update the documentation in lockstep.
- The current save/load path serializes the full `ProjectFile` through serde-backed JSON packed into `.soloc`; the new entry should fit that established flow rather than introducing special-case container handling.
- `ui.flow_id` remains the authoritative active-flow slot, but this story should not introduce new UI-owned active-flow state because the work is storage-focused.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 / Story 1.1 / Requirements Inventory
- Source: `_bmad-output/planning-artifacts/prd.md` — Timing Model and Tuplet Data Structures; Data Model and Persistence Stakes; Persistence and Migration Requirements
- Source: `_bmad-output/planning-artifacts/architecture.md` — Executive Summary; Agreed Model Invariants; Recommended Persisted Data Model; Why This Model Fits Compose
- Source: `docs/project-context.md` — Technology Stack & Versions; Critical Implementation Rules; Development Workflow Rules
- Source: `docs/data-models.md` — Dual-Index Storage Pattern; Entry System (Tagged Enum); Duration & Tick System
- Source: `src-tauri/src/score/entries/mod.rs` — current `Entry` enum and `Entries` dual-index behavior
- Source: `src-tauri/src/score/tracks.rs` — track insertion, shift, and removal behavior
- Source: `src-tauri/src/score/components/duration.rs` — `Tick`, `Ticks`, and `NoteDuration`
- Source: `src-tauri/src/files.rs` — `ProjectFile`, app-version handling, migration flow, and `.soloc` load/save path
- Source: `src-tauri/src/soloc.rs` — container format and gzip-compressed project payload handling
- Source: `src-tauri/src/migrations/next.rs` — pending migration location for unreleased persisted-data changes

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Add a first-class persisted tuplet container entry to the Rust score model while preserving the existing tagged-enum and dual-index contracts.
- Keep the persisted schema future-extensible by reusing `Tick`, `Ticks`, `NoteDuration`, `Vec<u8>`, and nested `Entries` rather than triplet-only fields.
- Prove the serde `.soloc` round-trip and legacy non-tuplet compatibility in existing Rust test locations, then update the documented data model and TS mirror types.

### Debug Log References

- `npm run test:rust`
- `npm run test:js`
- `npm run check-types`
- `npm run lint`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Completion Notes List

- Added `Tuplet` as a persisted score container entry and exposed it through `Entry::Tuplet(Box<Tuplet>)` so the recursive `Entry -> Tuplet -> Entries -> Entry` model stays memory-safe while preserving the tagged serde shape.
- Simplified the persisted tuplet payload to ratio metadata (`note_count`, `in_space_of`, `base_note_value`) plus grouping and nested `Entries`, and added derived helper methods for host/local tick spans.
- Added Rust regression coverage for `Entry::Tuplet` identity/tick behavior, ratio-derived spans, track insertion/shift/removal through both indices, `.soloc` round-trip persistence, and legacy non-tuplet load safety.
- Updated the frontend score DTO mirror with `Tuplet` and shared `Entries` types so Rust-owned state can serialize without a parallel tuplet model.
- Updated `docs/project-context.md` so the repo guidance no longer claims `Entry` has only six variants.

### File List

- \_bmad-output/implementation-artifacts/1-1-add-persisted-tuplet-container-entries.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- docs/data-models.md
- docs/project-context.md
- src/services/types.ts
- src-tauri/src/score/entries/mod.rs
- src-tauri/src/score/entries/tuplet.rs
- src-tauri/src/score/tracks.rs
- src-tauri/src/files.rs

## Senior Developer Review (AI)

- Outcome: Approved after fixes.
- Fixed the stale repo guidance that still documented `Entry` as six variants.
- Simplified the persisted tuplet shape to ratio metadata.
- Updated regression coverage so the current persisted shape and legacy non-tuplet files remain covered.

## Change Log

- 2026-03-13: Added persisted `Entry::Tuplet` support in the Rust score model, covered round-trip and legacy-load behavior with tests, updated the frontend score DTO mirror, and documented the new ownership and persistence rules.
- 2026-03-13: Simplified `Tuplet` persistence to ratio metadata (`note_count`, `in_space_of`, `base_note_value`), fixed stale developer guidance, and approved the review.
