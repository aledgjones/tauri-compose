# Story 5.3: Document Tuplet Scope, Editing Rules, and Persistence Impact

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want the shipped tuplet behavior documented clearly,
so that future contributors understand the supported scope and do not reintroduce ambiguity.

## Scope Note

- This story owns durable developer-facing documentation for the shipped tuplets MVP behavior.
- The goal is to document what the product actually supports now: triplet-first authoring, current editing rules, notation and playback boundaries, persistence behavior, and explicit non-goals.
- This story should not silently expand behavior. If the docs imply support beyond the shipped implementation, that is a defect in this story.
- Project guardrail and AI-agent instruction updates are closely related but belong primarily to Story 5.4. Story 5.3 should focus on durable product and developer documentation.

## Acceptance Criteria

1. **Given** foundational tuplet support is implemented
   **When** developer-facing documentation is updated
   **Then** it describes supported tuplet scope, popover-plus-sequencer entry flow, editing rules, notation boundaries, playback boundaries, and persistence impact
   **And** migration expectations are documented wherever persisted behavior changed

2. **Given** the MVP intentionally excludes some broader tuplet behaviors
   **When** the documentation is reviewed
   **Then** deferred areas such as broader ratios, advanced sequencer manipulation, or other post-MVP concerns are called out clearly
   **And** the docs do not imply support beyond the implemented scope

3. **Given** nested tuplets are explicitly out of scope for MVP
   **When** documentation is updated
   **Then** it states that tuplets are non-nestable in shipped MVP behavior
   **And** it does not present recursive child-content support as active user-facing capability

## Tasks / Subtasks

- [x] Audit the current tuplets documentation set against the shipped behavior (AC: 1, 2, 3)
  - [x] Review `docs/architecture.md`, `docs/data-models.md`, `docs/development-guide.md`, and `docs/project-context.md` for current tuplets guidance.
  - [x] Identify any gaps or contradictions around entry flow, editing rules, notation behavior, playback scope, persistence, or migration impact.
  - [x] Identify any wording that implies broader-than-MVP tuplets support.
- [x] Document the shipped tuplets authoring and editing scope explicitly (AC: 1, 2, 3)
  - [x] Describe the popover-plus-sequencer entry flow as the supported MVP authoring path.
  - [x] Document the ownership and editing rules that matter to contributors: tones live where they start, tuplet child timing is local, container geometry is separate from crossing-duration consumption, and empty local space is derived rather than persisted.
  - [x] Document the current delete, slice, selection, and non-selectable-empty-space expectations where they affect contributor understanding.
- [x] Document notation, playback, persistence, and migration boundaries clearly (AC: 1, 2)
  - [x] Explain that notation is derived from persisted tuplet containers rather than stored as printed-score output.
  - [x] Explain that playback consumes the shared tuplet model and remains scoped to the active flow for MVP.
  - [x] Explain the current persistence impact: tuplets are additive persisted data, legacy non-tuplet files remain loadable, and no tuplet-specific migration is currently required because pre-tuplet releases did not produce tuplet-bearing payloads.
- [x] Call out deferred and unsupported areas explicitly (AC: 2, 3)
  - [x] State that broader ratios, nested tuplets, richer sequencer-native tuplet manipulation, and other post-MVP concerns remain deferred.
  - [x] Ensure docs do not imply recursive child entries, generalized arbitrary tuplets editing UX, or cross-flow playback behavior.
- [x] Validate documentation impact and consistency before closing the story
  - [x] Check that the same core contract appears consistently across the durable docs set.
  - [x] Update adjacent docs if a tuplets rule is referenced there and would otherwise become stale.

## Dev Notes

### Story Foundation

- The tuplet feature now spans persistence, parser derivation, notation drawing, playback, and editing behavior. Without a coherent docs pass, later contributors will reintroduce ambiguity the implementation already resolved.
- This story should leave one clear maintainer narrative for what tuplets MVP means in this repo.
- The most important failure mode is not missing prose. It is documentation that accidentally overpromises beyond the shipped scope.

### Technical Requirements

- Keep the docs aligned with the actual backend-owned tuplet model and the current authoring flow.
- Document the key invariants contributors need to preserve:
  - tones remain single stored events owned by where they start
  - tuplet child timing is local to the container
  - container span and crossing duration are separate concerns
  - notation and playback are derived consumers of the shared model
  - MVP tuplets are explicitly non-nestable
- Document the migration stance precisely: migration review remains mandatory for persisted-data changes, but the current tuplet release state does not require a dedicated tuplet migration step for legacy non-tuplet files.

### Architecture Compliance

- The docs must reinforce that tuplets are one shared semantic model across persistence, parsing, rendering, and playback.
- The docs must preserve the current active-flow playback boundary and must not imply broader transport scope.
- The docs must make clear that printed notation is derived from backend-owned tuplet containers, not stored separately.

### Library / Framework Requirements

- No new library or framework is expected for this story.
- Work should stay in Markdown docs already owned by the repo.

### File Structure Requirements

- Primary durable docs likely to update:
  - `docs/architecture.md`
  - `docs/data-models.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
- Secondary touchpoints if discovery shows a real documentation gap:
  - `README.md`
  - `TESTING.md`
  - `docs/index.md`

### Testing Requirements

- Validate by cross-checking docs content against the shipped code and prior story artifacts.
- If documentation changes reference fixture or workflow behavior, ensure those references match the actual current commands and file locations.

### Previous Story Intelligence

- Story 1.6 and Story 1.7 already documented important persistence and file-workflow rules that this story should consolidate rather than restate inconsistently.
- Story 2.5 captured the explicit MVP delete and slice rules for tuplet members and containers.
- Stories 3.1 through 3.4 and 4.1 through 4.4 established the current notation and playback boundaries that documentation now needs to present clearly as the shipped baseline.

### Git Intelligence Summary

- Recent relevant commits:
  - `57591fe3` `feat(4.4): keep MVP playback scoped to active flow`
  - `3c60c495` `feat(4.3): preserve host-only playback behavior`
  - `9e867590` `feat(4.2): finalize tuplet playback ordering`
- The implementation phase has finished the major tuplets seams. Story 5.3 is the documentation synthesis step that prevents future contributors from re-litigating already made decisions.

### Documentation Impact

- Documentation impact is the core deliverable of this story.
- Make sure every durable contract changed by tuplets has an updated docs home: supported scope, editing rules, playback boundary, persistence impact, and migration stance.
- If a docs page cannot be kept accurate without changing a nearby project rule summary, update that summary in the same change.

### Project Structure Notes

- `docs/data-models.md` already contains the richest persistence and ownership detail.
- `docs/architecture.md` already carries the notation and persistence contracts.
- `docs/development-guide.md` and `docs/project-context.md` already carry contributor-facing guardrails and are likely where ambiguity will matter most.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 5 / Story 5.3 acceptance criteria
- Source: `_bmad-output/planning-artifacts/prd.md` — tuplets product scope, maintainer journey, and deferred post-MVP work
- Source: `_bmad-output/planning-artifacts/architecture.md` — tuplets structural model and current MVP boundaries
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — supported authoring flow and explicit non-goals
- Source: `docs/architecture.md`
- Source: `docs/data-models.md`
- Source: `docs/development-guide.md`
- Source: `docs/project-context.md`
- Source: `_bmad-output/implementation-artifacts/1-6-round-trip-tuplet-data-through-save-load-and-migration.md`
- Source: `_bmad-output/implementation-artifacts/2-5-apply-explicit-mvp-delete-and-slice-rules-for-tuplet-members-and-containers.md`
- Source: `_bmad-output/implementation-artifacts/4-4-keep-mvp-playback-scoped-to-the-active-flow.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Reconcile the durable docs set against the shipped tuplet behavior and eliminate any contradictory or over-broad statements.
- Present the supported tuplet MVP as one coherent maintainer contract covering authoring, editing, notation, playback, persistence, and migration.
- Make deferred areas explicit so contributors can tell what is intentionally unsupported versus accidentally undocumented.

### Debug Log

- Audited the existing tuplets narrative across `docs/architecture.md`, `docs/data-models.md`, `docs/development-guide.md`, and `docs/project-context.md` against the shipped MVP behavior and story history.
- The durable docs already covered most persistence and ownership details, but the cross-doc narrative still needed explicit scope language for the popover-first authoring path, active-flow playback boundary, and deferred broader-ratio / richer-editing areas.

### Completion Notes

- Updated `docs/architecture.md` to state the shipped popover-plus-sequencer authoring path, keep sequencer editing tone-focused, and pin MVP playback to the active flow selected through `ui.flow_id`.
- Updated `docs/data-models.md` to clarify the supported triplet-first parser suffixes, the intentionally narrow sequencer editing surface, and the active-flow playback boundary for tuplet scheduling.
- Updated `docs/development-guide.md` to align contributor-facing tuplets scope claims with the shipped MVP, including no-current-migration language for legacy non-tuplet files and explicit deferred areas.
- Validated the updated docs set for consistent phrasing around ownership, derived empty local space, non-selectable blank tuplet space, derived notation, playback scope, and migration stance.
- Validation passed with `npm run lint`, `npm run check-types`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### File List

- docs/architecture.md
- docs/data-models.md
- docs/development-guide.md
- \_bmad-output/implementation-artifacts/5-3-document-tuplet-scope-editing-rules-and-persistence-impact.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-03-18: Clarified the shipped tuplets MVP documentation contract across architecture, data model, and development guide docs, then validated the repo with lint, typecheck, cargo test, and clippy.
