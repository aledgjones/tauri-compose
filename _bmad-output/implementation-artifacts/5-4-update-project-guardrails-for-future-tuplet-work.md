# Story 5.4: Update Project Guardrails for Future Tuplet Work

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want AI-agent and developer instructions updated for the new tuplet architecture,
so that future work follows the same timing, persistence, and testing rules.

## Scope Note

- This story owns repo guardrails and contributor instructions, not end-user feature behavior.
- The goal is to make future tuplets work harder to do incorrectly by updating the repo's explicit rules where agents and developers actually look first.
- Guardrails added here must extend the existing project rules consistently. Do not invent a parallel process just for tuplets.
- The story should encode the tuplet timing invariant, shared semantic-model rule, migration discipline, test-placement expectations, and MVP non-nestable boundary clearly enough that later work starts from the right assumptions.

## Acceptance Criteria

1. **Given** foundational tuplet support establishes durable project rules
   **When** project instructions are updated
   **Then** they reflect the backend-owned timing model, migration discipline, and test-placement expectations introduced by this feature
   **And** future contributors are guided away from frontend-owned parallel state or ad hoc persistence fixes

2. **Given** the repo already has explicit migration and documentation rules
   **When** tuplet-specific guidance is added
   **Then** it extends those rules consistently rather than inventing a conflicting process
   **And** future tuplets work has a clear architectural baseline to build on

3. **Given** the MVP implementation simplifies tuplet-owned child content to one level
   **When** project guardrails are updated
   **Then** they state that nested `Tuplet` child entries are intentionally disallowed in MVP
   **And** any future reintroduction would require explicit post-MVP planning

## Tasks / Subtasks

- [ ] Audit the current guardrail surfaces that future contributors rely on (AC: 1, 2, 3)
  - [ ] Review `.github/copilot-instructions.md` for current tuplet rules, migration discipline, and documentation expectations.
  - [ ] Review `docs/project-context.md` and `docs/development-guide.md` for contributor-facing implementation guardrails.
  - [ ] Identify any tuplet architectural rule that is still only implicit in code or prior story artifacts.
- [ ] Update AI-agent and contributor instructions with the durable tuplet rules (AC: 1, 2, 3)
  - [ ] Reinforce the backend-owned shared semantic-model rule across persistence, parsing, rendering, playback, and editing.
  - [ ] Reinforce the timing invariant that container geometry is compressed while crossing duration remains literal.
  - [ ] Reinforce the rule that `ui.flow_id` remains the authoritative active-flow slot and that future work must not add frontend-owned parallel flow or tuplet truth.
  - [ ] Reinforce migration discipline and the current no-new-migration stance unless a released older payload actually requires transformation.
  - [ ] Reinforce test-placement expectations so seam-owned regressions stay close to the owning module.
  - [ ] Reinforce the MVP single-level child-content rule and explicitly defer nested tuplets to post-MVP planning.
- [ ] Align guardrails across all maintained instruction surfaces (AC: 1, 2)
  - [ ] Ensure the same tuplet rules appear consistently wherever future agents or developers are likely to read them.
  - [ ] Avoid contradictory phrasing between `.github/copilot-instructions.md`, `docs/project-context.md`, and `docs/development-guide.md`.
- [ ] Validate that the resulting guidance prevents the known tuplet failure modes
  - [ ] Check that the instructions explicitly push contributors away from frontend-owned state duplication, local subsystem-specific tuplet reinterpretation, and ad hoc persistence repair logic.
  - [ ] Check that the instructions explicitly preserve docs-update discipline for any future persisted-data or workflow changes.

## Dev Notes

### Story Foundation

- Tuplets are now implemented across multiple seams, which means future work can easily go wrong if the durable rules stay trapped in prior story artifacts.
- This story converts the tuplets lessons of Epics 1 through 4 into repo-level guardrails.
- The key outcome is leverage: future agents and humans should start from the correct tuplet assumptions without rereading the full epic history.

### Technical Requirements

- Preserve and surface the tuplet rules that future work is most likely to violate:
  - one shared semantic model across persistence, parsing, rendering, playback, and editing
  - backend-owned truth, not frontend-owned parallel state
  - explicit migration review for persisted-data changes
  - seam-owned testing close to the responsible module
  - active-flow playback scope remains governed by `ui.flow_id`
  - nested tuplets remain out of MVP scope
- Extend current repo rules rather than adding tuplet-only exceptions that conflict with existing migration, docs, or workflow guidance.

### Architecture Compliance

- The instructions must reinforce that later consumers adapt the shared tuplet model for their output, but do not re-derive ownership, boundary meaning, projection, or continuity through local inference.
- The instructions must preserve the canonical timing invariant and the active-flow scope rule.
- The instructions must keep migration behavior explicit and versioned rather than normalized into load-time repair code.

### Library / Framework Requirements

- No new library or framework is expected for this story.
- Work should remain in instruction and guardrail Markdown files already owned by the repo.

### File Structure Requirements

- Primary guardrail surfaces:
  - `.github/copilot-instructions.md`
  - `docs/project-context.md`
  - `docs/development-guide.md`
- Secondary touchpoints if the audit finds a real rules gap:
  - `docs/architecture.md`
  - `_bmad-output/planning-artifacts/architecture.md`

### Testing Requirements

- Validate guardrail consistency by cross-checking the updated instruction surfaces against the shipped tuplet behavior and current repo practices.
- If an instruction references a command or file location, confirm the command and path are current.

### Previous Story Intelligence

- Story 1.7 already captured versioning and offline workflow discipline for tuplet-bearing files.
- Story 3.2 established the local-test-support pattern that this story should reinforce at the repo rule level.
- Story 4.4 and the current docs already codify `ui.flow_id` as the authoritative active-flow slot, which must remain part of future tuplet guardrails.

### Git Intelligence Summary

- Recent relevant commits:
  - `049288a2` `chore: epic 4 retro`
  - `57591fe3` `feat(4.4): keep MVP playback scoped to active flow`
  - `3c60c495` `feat(4.3): preserve host-only playback behavior`
- The tuplet feature has reached the point where durable guidance matters more than local memory. Story 5.4 should move that guidance into the repo's stable instruction surfaces.

### Documentation Impact

- Documentation impact is required for this story because the deliverable is updated contributor guidance.
- Keep guardrails synchronized with any broader documentation updates made in Story 5.3 if both stories are implemented together.
- If project rules change meaningfully, update both the agent instructions and the contributor-facing docs in the same change.

### Project Structure Notes

- `.github/copilot-instructions.md` is already the repo's highest-signal AI-agent rules file and should remain authoritative for tuplet-specific implementation constraints.
- `docs/project-context.md` is optimized for AI agents and should carry concise, durable rules rather than historical narrative.
- `docs/development-guide.md` is the right place for richer contributor-facing implementation guardrails and command/process expectations.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 5 / Story 5.4 acceptance criteria
- Source: `_bmad-output/planning-artifacts/prd.md` — maintainer journey and exact tuplet product constraints
- Source: `_bmad-output/planning-artifacts/architecture.md` — shared tuplet model and persistence/playback boundaries
- Source: `docs/project-context.md`
- Source: `docs/development-guide.md`
- Source: `.github/copilot-instructions.md`
- Source: `_bmad-output/implementation-artifacts/1-7-keep-versioning-and-offline-desktop-behavior-intact-for-tuplet-files.md`
- Source: `_bmad-output/implementation-artifacts/4-4-keep-mvp-playback-scoped-to-the-active-flow.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Audit the current guardrail surfaces for tuplet rules that are missing, duplicated inconsistently, or still only implied by story history.
- Update the repo's instruction surfaces so future contributors start from the correct tuplets invariants, migration rules, and seam-owned testing expectations.
- Keep the result concise and authoritative enough that future work does not need to reconstruct tuplet architecture from past epics.
