# Story 8.1: Add Durable Playing-Technique Score Entries

Status: done

## Story

As a composer,
I want to add playing-technique markings to the score,
so that Compose stores durable technique intent as first-class score data rather than as hidden playback state.

## Acceptance Criteria

1. Given playing techniques are user-authored score markings, when Compose introduces the first playing-technique entry type, then `PlayingTechnique` entries are persisted as first-class score data on stave master tracks at specific ticks, and they are modeled as musical markings rather than plugin-specific playback instructions.
2. Given this story is a bounded proof of concept, when the first playing-technique entry type is shipped, then the supported technique vocabulary is limited to `pizz` and `arco`, and those first markings already use the same exclusive-group semantic structure intended for richer future technique sets rather than a one-off binary shortcut.
3. Given later playback realism will depend on these markings, when the first playing-technique entry type is defined, then the data model remains extensible to multiple simultaneous technique markings at the same tick, and it does not assume that technique meaning is limited to one note-local property at a time or that every future technique family will be a bespoke special case.

## Tasks / Subtasks

- [x] Task 1: Add one durable playing-technique entry family on stave master tracks (AC: 1, 2, 3)
  - [x] Add a new score-entry family such as `Entry::PlayingTechnique(PlayingTechnique)` under `src-tauri/src/score/entries/`; do not create top-level entry variants such as `Entry::Pizz` or `Entry::Arco`.
  - [x] Keep the persisted payload light and score-owned: stable `id`, `tick`, `hidden`, and one inner technique enum such as `PlayingTechniqueKind` limited to `Pizz` and `Arco` for this story.
  - [x] Treat meaning as derived from the inner technique variant; do not persist exclusivity groups, cancellation flags, playback routing state, or plugin-specific behavior on the entry itself.
  - [x] Allow multiple `PlayingTechnique` entries at the same tick as separate entries when their technique kinds differ, and replace an exact duplicate of the same technique at the same tick on the same stave instead of accumulating identical duplicates.

- [x] Task 2: Add a thin backend command path without widening into score UI or playback behavior (AC: 1, 2, 3)
  - [x] Add narrow domain commands near the owning entry seam for create, remove-by-id, and update behavior using the existing score-entry command family style.
  - [x] Keep command ownership on stave master tracks like clefs, not on tone tracks and not on flow master tracks.
  - [x] If an update command is included, keep it narrow to persisted entry fields needed for the data-first seam, such as `hidden`; changing the technique kind remains a delete-and-create authoring flow because individual technique types are distinct score markings rather than interchangeable edits.
  - [x] Register the thin automation or MCP path if needed for end-to-end invocation, but do not add normal write-screen authoring UI, renderer hit targets, selection wiring, properties panels, or playback rule matching in this story.

- [x] Task 3: Preserve `.soloc` persistence and document the additive score-model change (AC: 1, 2, 3)
  - [x] Extend typed load/save and round-trip coverage so `PlayingTechnique` entries serialize and deserialize deterministically through the existing `.soloc` project path.
  - [x] Perform migration review and record the explicit no-migration rationale if the final shape remains an additive tagged-entry variant that older released files can still load without transformation.
  - [x] Update developer-facing data-model and architecture docs to reflect the new entry family, stave-master ownership, and the rule that technique meaning is derived from the inner variant rather than stored as explicit cancellation state.

### Review Findings

- [x] [Review][Patch] Story wording no longer implies that playing-technique kind must be mutable through the update seam [_bmad-output/implementation-artifacts/8-1-add-durable-playing-technique-score-entries.md:28]
- [x] [Review][Patch] Playing-technique insertion now follows the repo-style replace-and-insert pattern while still replacing exact duplicate kind-at-tick entries [src-tauri/src/score/entries/playing_technique.rs:39]
- [x] [Review][Patch] `project-context.md` now consistently tells agents to handle all 8 `Entry` variants [docs/project-context.md:203]

## Dev Notes

### Story Intent

- Story 8.1 is the durable score-model foundation for Epic 8.
- The core job is to add a first-class score entry family for playing techniques, not to ship the visible notation workflow or the audible playback proof yet.
- The highest-risk failure mode is modeling `pizz` and `arco` as separate top-level entry categories or as hidden playback state instead of one durable musical-marking family.

### User Clarifications

- The model should use one `PlayingTechnique` entry family with shared config and an inner technique variant.
- Meaning is derived from the variant, not from separately persisted exclusivity or cancellation metadata.
- This matches the intended bounded mental model observed in Dorico for the initial proof of concept.
- Exact duplicate insertion of the same technique at the same tick on the same stave should replace the existing identical entry.

### This Story Owns

- One new first-class stave-master score entry family for playing techniques.
- A bounded initial technique vocabulary: `pizz` and `arco` only.
- A thin backend creation and mutation seam sufficient to create, remove, and inspect persisted entries end to end.
- Persistence, round-trip validation, and documentation updates for the additive score-model change.

### This Story Does Not Own

- Visible score rendering, hit targets, or shared selection behavior for playing techniques.
- Bottom-panel properties surfaces or any normal write-screen editing affordance.
- Playback preparation, technique-resolution logic, expression-map condition matching, or transport behavior.
- A broader technique taxonomy, technique-specific UI configuration, or plugin-specific semantics.

### Current Relevant Seams

- `src-tauri/src/score/entries/mod.rs`
  - Owner of the `Entry` tagged enum and the common `id()`, `tick()`, and `set_tick()` behavior that a new entry family must join.
- `src-tauri/src/score/entries/clef.rs`
  - Closest existing pattern for a stave-master-owned directive-style entry with `hidden` state and narrow create, clear, and update commands.
- `src-tauri/src/score/components/articulation.rs`
  - Reference mental model for deriving exclusivity semantics from a bounded enum instead of persisting cancellation state directly.
- `src-tauri/src/score/tracks.rs`
  - Generic dual-index track storage that already supports multiple entries at one tick and should remain the owner of same-tick entry ordering and removal behavior.
- `src-tauri/src/automation/registry.rs`
  - Existing action-definition and dispatch seam for thin end-to-end score-entry invocation.
- `src-tauri/src/lib.rs`
  - Tauri command wiring; keep command definitions near the owning score-entry domain and only wire them here.
- `src-tauri/src/files.rs`
  - Typed project-file load/save and `.soloc` round-trip test seam.
- `src-tauri/src/migrations/next.rs`
  - Pending migration review seam; preserve the empty scaffold unless implementation proves an older released payload requires transformation.
- `docs/data-models.md`
  - Canonical persisted-data description for entry families, `.soloc` behavior, and migration rules.
- `docs/architecture.md`
  - Canonical ownership and score-model architecture rules, including flow-master versus stave-master directive ownership.
- `docs/project-context.md`
  - Repo-wide agent guidance that currently describes the `Entry` family count and persistence rules.

### Architecture Compliance

- Keep backend-owned score truth canonical. React must not become the owner of durable playing-technique state.
- Treat playing techniques as musical markings on stave master tracks, not as tone-local articulations, hidden playback regions, or plugin-owned instructions.
- Use one new entry family plus one inner technique enum. Do not create a new top-level entry variant for each technique value.
- Keep the persisted payload minimal and additive. If older released files continue to load unchanged because they simply omit the new tagged variant, do not add compatibility residue or a placeholder migration step.
- Derive exclusivity and cancellation semantics in a later backend semantic-resolution seam. Transport and playback should consume derived meaning rather than inventing persistent technique logic on the fly.

### Implementation Guidance

- Prefer a new file such as `src-tauri/src/score/entries/playing_technique.rs` that mirrors the existing entry-family organization under `src-tauri/src/score/entries/`.
- Name the inner value type clearly, such as `PlayingTechniqueKind`, and keep its initial vocabulary bounded to `Pizz` and `Arco`.
- Persist `hidden` now so the entry family is structurally aligned with other directive-style entries, even though the properties-panel workflow lands later.
- Same-tick extensibility is required now. The model must allow two separate `PlayingTechnique` entries at one tick when they represent different future technique meanings.
- Duplicate handling must be narrow and deterministic: exact duplicate kind at the same tick on the same stave replaces the existing identical entry; different kinds at the same tick remain separate entries.
- If implementation later needs technique-specific config, extend the inner technique enum payload rather than splitting the top-level `Entry` enum further.
- Keep `.soloc` handling on the normal typed load/save path. Do not add ad hoc repair logic or a parallel persistence seam for this entry family.

### Risks And Regression Traps

- Modeling `pizz` and `arco` as separate top-level `Entry` variants, which would duplicate behavior across every `Entry` match arm and make later expansion brittle.
- Treating playing techniques as tone-local data like articulations. These markings persist across later notes and therefore belong on stave master tracks.
- Persisting exclusivity-group ids, cancellation booleans, or playback-state residue instead of deriving that meaning from the technique variant later.
- Removing all entries at a tick when inserting one technique, which would break the explicit same-tick extensibility requirement.
- Widening 8.1 into selection, draw, properties, or playback work that belongs to Stories 8.2 through 8.6.
- Adding a migration step to `src-tauri/src/migrations/next.rs` without a real released-data compatibility need, especially right after the recent migration cleanup pass.

### Testing Requirements

- Add Rust unit coverage for the new entry family under the owning score-entry seam, including field storage, same-tick coexistence for different techniques, and exact-duplicate replacement behavior.
- Extend `src-tauri/src/score/entries/mod.rs` tests so the new `Entry::PlayingTechnique` variant participates in `id()`, `tick()`, and `set_tick()` coverage.
- Extend `src-tauri/src/score/tracks.rs` tests or equivalent owner-seam coverage to prove track insertion, shifting, and removal work correctly with `PlayingTechnique` entries.
- Add `.soloc` round-trip coverage in `src-tauri/src/files.rs` proving that a project containing `PlayingTechnique` entries reloads with the same persisted structure.
- Add a regression check proving that older non-playing-technique project data still loads unchanged and does not invent technique entries.
- If automation or MCP registration is added, cover the new command seam at the owning Rust layer rather than relying only on manual testing.
- No committed visual-regression fixtures are needed for 8.1 because rendering is explicitly deferred.

### Documentation Impact

- Update `docs/data-models.md` to add the new `PlayingTechnique` entry family, its persisted fields, and the additive no-migration rationale if that remains true.
- Update `docs/architecture.md` to record that playing techniques are stave-master-owned musical markings and that their semantic meaning is derived later from the technique variant.
- Update `docs/project-context.md` if the documented `Entry` family inventory or persistence guidance needs to reflect the new entry family.
- Update AI-agent instructions only if implementation establishes a durable repo-wide rule beyond the feature-specific data shape captured here.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- No new dependencies should be introduced for Story 8.1.

### File Structure Notes

- Keep the new durable score-model code under `src-tauri/src/score/entries/` and adjacent owner seams.
- Keep thin command registration in the existing Rust command or automation wiring paths; do not create a second feature-specific command framework.
- Keep story tracking under `_bmad-output/implementation-artifacts/` and update sprint status in the same pass.

### Git Intelligence Summary

- Recent repo history includes `chore: cleanup migrations` and the release reseed commit, which reinforces that `src-tauri/src/migrations/next.rs` should remain an empty scaffold unless this story uncovers a real compatibility transform.
- The current repo trend continues to prefer narrow backend-owned domain commands and additive persisted-data changes over compatibility residue or frontend-owned shadow state.

### Latest Technical Information

- No external library research is needed for Story 8.1.
- The decisive current information is repo truth as of 2026-04-08 plus the clarified product decision that all initial playing-technique entries share one config shape and derive meaning from the inner technique variant.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 8 and Story 8.1 acceptance criteria.
- `docs/project-context.md`
  - Entry-family inventory, persistence rules, and backend-owned state guidance.
- `docs/data-models.md`
  - Dual-index entry storage, `.soloc` persistence rules, and additive schema examples.
- `docs/architecture.md`
  - Flow-master versus stave-master directive ownership and backend-owned score truth.
- `src-tauri/src/score/entries/mod.rs`
  - Entry tagged enum owner seam.
- `src-tauri/src/score/entries/clef.rs`
  - Existing stave-master directive pattern.
- `src-tauri/src/score/components/articulation.rs`
  - Derived exclusivity-group reference model.
- `src-tauri/src/score/tracks.rs`
  - Generic same-tick entry storage behavior.
- `src-tauri/src/automation/registry.rs`
  - Thin action registration and dispatch seam.
- `src-tauri/src/files.rs`
  - Typed project load/save and `.soloc` round-trip seam.
- `src-tauri/src/migrations/next.rs`
  - Pending migration review seam.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-08: Story prepared from Epic 8 planning, repo architecture and data-model docs, score-entry owner seams, `.soloc` persistence tests, and recent migration guidance.
- 2026-04-08: Product clarification recorded that initial playing-technique entries share one config shape and derive meaning from the inner technique variant rather than separate top-level entry categories.
- 2026-04-08: Story scope held intentionally data-first: backend model, thin command seam, persistence, tests, and docs only. Rendering, selection, properties, semantic resolution, and playback remain deferred to later Epic 8 stories.
- 2026-04-08: Implemented `Entry::PlayingTechnique` with bounded `PlayingTechniqueKind::{Pizz, Arco}`, same-tick duplicate replacement rules, and stave-master-scoped create/remove/update commands plus automation registry actions.
- 2026-04-08: Extended `.soloc` round-trip coverage and legacy-load regression coverage for the additive tagged-entry variant while keeping `src-tauri/src/migrations/next.rs` unchanged because no released payload requires transformation.
- 2026-04-08: Validation completed with `npm run check-types`, `npm run lint`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path /Users/aledjones/compose/src-tauri/Cargo.toml --lib --tests -- -D warnings`.
- 2026-04-08: Refined the thin update seam so playing-technique kind remains create-only; changing `Pizz` to `Arco` now requires delete-plus-create, and the stave-master lookup in this file was aligned with the existing inline clef-command pattern instead of keeping a one-off generic helper.
- 2026-04-08: Refined the playing-technique seam to follow the existing create/update/clear pattern more closely: track-owned `get_playing_technique_at_tick(...)`, `get_playing_techniques_at_tick(...)`, `insert_playing_technique(...)`, `clear_playing_techniques_at_tick(...)`, and `update_playing_technique(...)` now carry the score-entry ownership while the Tauri commands stay thin.

### Completion Notes List

- Story context created for Story 8.1.
- Epic 8 moved to `in-progress` because its first story now has a ready-for-dev artifact.
- Story 8.1 moved to `ready-for-dev` in sprint tracking.
- The story explicitly chooses one `PlayingTechnique` entry family plus an inner technique enum over separate top-level `Entry` variants for `Pizz` and `Arco`.
- The story also records the additive no-migration review stance: preserve the empty `next.rs` scaffold unless implementation proves that an older released payload truly requires transformation.
- Added `src-tauri/src/score/entries/playing_technique.rs` with the durable `PlayingTechnique` entry model, bounded `PlayingTechniqueKind` enum, deterministic same-tick duplicate replacement, and narrow stave-master-scoped create/remove/update commands.
- Wired the new backend seam through Tauri command registration, automation registry actions, and automation track-event inspection without widening into score UI, renderer hit targets, or playback behavior.
- Updated the mirrored TypeScript score model and action wrappers so frontend state remains aligned with the new backend-owned `Entry::PlayingTechnique` variant.
- Added Rust unit coverage for the new entry family, generic track behavior, `.soloc` round trips, and legacy non-playing-technique payload loading.
- Updated developer docs to record the new entry family, stave-master ownership, and the additive no-migration rationale.
- Tightened the update contract so it only mutates `hidden`; technique-kind changes are now intentionally modeled as delete-and-create operations.

### File List

- `_bmad-output/implementation-artifacts/8-1-add-durable-playing-technique-score-entries.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src-tauri/src/score/entries/playing_technique.rs`
- `src-tauri/src/score/entries/mod.rs`
- `src-tauri/src/score/tracks.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/automation/registry.rs`
- `src-tauri/src/files.rs`
- `src/services/types.ts`
- `src/services/actions.ts`
- `docs/data-models.md`
- `docs/architecture.md`
- `docs/project-context.md`

### Change Log

- 2026-04-08: Added durable stave-master playing-technique entries with bounded backend commands, automation wiring, persistence coverage, and documentation updates.
