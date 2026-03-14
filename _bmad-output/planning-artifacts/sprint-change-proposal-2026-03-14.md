# Sprint Change Proposal: Explicitly Exclude Nested Tuplets from MVP

Date: 2026-03-14
Prepared by: John, Product Manager
Mode: Batch
Change trigger: During Story 1.3, the team confirmed the current sequencer UX works for foundational tuplets, but also confirmed that nested tuplets add planning, UX, editing, and validation complexity without clear MVP value.

## 1. Issue Summary

The current tuplet initiative was already intended to be triplet-first, but several planning artifacts still leave enough ambiguity that future story writing or implementation could treat nested tuplets as part of MVP-facing scope.

The key clarification is this:

- For MVP, tuplets are explicitly non-nestable as a product rule.
- UX, acceptance criteria, editing rules, parser coverage, rendering expectations, playback scope, and future story wording must not assume nested tuplet behavior is required in MVP.
- The revised plan should include a light internal refactor so tuplet-owned child content no longer permits nested `Tuplet` entries in MVP paths.
- This is a narrow simplification, not a general model rewrite.

This is a forward-only correction. Completed Stories 1.1 through 1.3 remain valid and should not be rolled back.

## 2. Impact Analysis

### Checklist Status

- [x] 1.1 Triggering story identified: Story 1.3, Render and Select the Created Tuplet Region in the Sequencer.
- [x] 1.2 Core problem defined: MVP scope ambiguity. The product intent is simpler than several planning artifacts currently imply.
- [x] 1.3 Supporting evidence gathered: PRD already places nested tuplets in post-MVP scope, but architecture and epic notes still preserve nested-capable MVP language.
- [x] 2.1 Current epic viability assessed: Epic 1 remains viable with no rollback.
- [x] 2.2 Epic-level changes identified: wording changes only; no epic replacement required.
- [x] 2.3 Future epics reviewed: Epics 2 through 5 need targeted wording updates so nested tuplets stop appearing as implied MVP behavior.
- [x] 2.4 Need for new or removed epics assessed: none.
- [x] 2.5 Epic order and priority assessed: no resequencing required.
- [x] 3.1 PRD conflicts reviewed.
- [x] 3.2 Architecture conflicts reviewed.
- [x] 3.3 UX conflicts reviewed.
- [x] 3.4 Other artifacts reviewed.
- [x] 4.1 Direct adjustment evaluated: viable.
- [ ] 4.2 Potential rollback evaluated: not viable.
- [x] 4.3 MVP review evaluated: viable.
- [x] 4.4 Recommended path selected.
- [x] 5.1 Issue summary prepared.
- [x] 5.2 Epic and artifact impacts documented.
- [x] 5.3 Recommended path documented.
- [x] 5.4 MVP action plan defined.
- [x] 5.5 Handoff plan defined.
- [x] 6.1 Review completed.
- [x] 6.2 Proposal consistency verified.
- [!] 6.3 Explicit user approval pending.
- [N/A] 6.4 Sprint-status structural change required: no new epic or story IDs, removals, or renumbering are recommended.
- [!] 6.5 Final handoff confirmation pending approval.

### Epic Impact

Epic 1 remains intact. Stories 1.1 to 1.3 stay done exactly as they are. Stories 1.4 to 1.7 need clarifying language so they do not implicitly enable nested tuplet creation or editing.

Epic 2 remains intact, but its editing stories must explicitly reject any edit path that would require nested tuplets.

Epic 3 remains intact, but parsing and rendering stories must limit MVP notation work to single-level tuplets.

Epic 4 remains intact, but playback stories must describe only single-level tuplet projection for MVP.

Epic 5 remains intact, but documentation and regression scope must explicitly exclude nested tuplets from MVP support and coverage targets.

### Artifact Conflicts

#### PRD

The PRD is mostly aligned already. It already lists nested tuplets in post-MVP scope, but it should state the MVP rule more explicitly so later stories cannot reinterpret future extensibility as present-scope behavior.

#### Architecture

The architecture document is the main source of ambiguity. It currently treats nested tuplets as part of recommended MVP child-content scope, future parser-only coverage, MVP boundary language, and selection-payload guidance. That language is too permissive for the clarified MVP decision.

#### UX Design Specification

The UX spec is mostly focused on single-level triplet interaction, but it should explicitly say MVP tuplets cannot be nested and that no nested-creation affordance, targeting rule, or editing expectation belongs in MVP stories.

#### Implementation Artifacts

Completed implementation artifacts for Stories 1.1 to 1.3 do not require rollback.

- Story 1.1 references nested child entries as future-extensible storage. That is acceptable as an internal representation note.
- Story 1.2 does not require nested behavior.
- Story 1.3 does not require nested behavior.

The smallest viable correction is to leave completed story artifacts intact, correct the active planning sources that will guide future backlog work, and simplify the active child-content model before deeper tuplet stories depend on recursive nesting.

### Technical Impact

No code rollback is recommended.

No persisted-data rollback is recommended.

No architecture reset is recommended.

The backend-owned tuplet model and exact-timing direction remain intact.

The technical-scoping adjustment is now this:

- Nested tuplet behavior must be treated as unsupported in MVP at creation, editing, parsing, rendering, playback, documentation, and regression-planning layers.
- The implementation model should be simplified so tuplet-owned child content cannot contain nested `Tuplet` entries.
- That change should remain light and local: remove nested `Tuplet` allowance without undoing the backend-owned tuplet container model, exact timing direction, or completed Stories 1.1 to 1.3.

## 3. Recommended Approach

Recommended path: Hybrid of Option 1 Direct Adjustment and Option 3 MVP Review, plus a light internal refactor.

This is an MVP scope reduction, not a replan and not a rollback.

### Why This Path

- It preserves momentum from completed Stories 1.1 to 1.3.
- It removes the biggest avoidable complexity multiplier before Story 1.4 and later work begin.
- It keeps the architecture direction and data ownership model intact.
- It reduces the need for repeated defensive "not at this level" checks in upcoming stories.
- It avoids wasting effort designing nested UX, nested edit semantics, nested validation rules, and nested regression matrices before the product has proven single-level tuplets.

### Rejected Option: Rollback

Rollback is not justified.

- Stories 1.1 to 1.3 still support the clarified MVP.
- The discovered issue is not that completed work is wrong.
- The issue is that upcoming planning language is too broad.

### Effort, Risk, Timeline

- Effort: Low to Medium.
- Risk: Low.
- Timeline impact: Neutral to slightly positive. The refactor adds a small amount of near-term work but should reduce downstream branching and validation cost.
- Scope classification: Moderate, because backlog wording and planning artifacts need coordinated updates before development continues.

## 4. Detailed Change Proposals

### 4.1 PRD Updates

#### PRD change 1: Make the non-nestable MVP rule explicit

Section: Product Scope, MVP / Out of Scope

OLD:

> Nested tuplets

NEW:

> Nested tuplets are explicitly out of scope for MVP.
> MVP tuplets are non-nestable as a product rule.
> Internal recursive representation may remain temporarily if it is implementation-convenient, but nested tuplets must not be treated as a required MVP behavior.

Rationale: The current out-of-scope bullet is directionally correct but too easy to reinterpret as “not prioritized yet” instead of “unsupported by product rule.”

#### PRD change 2: Add an MVP guardrail in authoring/editing semantics

Section: Editing Semantics

OLD:

> Tuplet editing rules must be explicit.

NEW:

> Tuplet editing rules must be explicit.
> For MVP, tuplets are non-nestable: no authoring, editing, repair, or acceptance criteria should require creating a tuplet inside another tuplet.
> If an interaction would require nested tuplet structure, MVP should reject or defer that path rather than invent nested behavior.

Rationale: This moves the product rule into the section most likely to drive Story 2 language.

### 4.2 Epics and Story Backlog Updates

#### Epic-level change 1: Replace the nested-capable MVP note

Section: Additional Requirements

OLD:

> Keep nested tuplets data-model-capable if practical, while still treating broader directive attachment, cross-barline support, and generalized child directives as post-MVP work.

NEW:

> Simplify the MVP implementation model so tuplet-owned child content does not contain nested `Tuplet` entries.
> Nested tuplets are not part of MVP behavior and must not drive story scope, UX requirements, acceptance criteria, or regression expectations.
> Broader directive attachment, cross-barline support, and generalized child directives remain post-MVP work.

Rationale: This removes the MVP-facing nested implication and the internal source of defensive branching.

#### Epic-level change 2: Add a light simplification step before deeper tuplet work

Recommendation:

- Do not create a new epic.
- Do not broad-replan the backlog.
- Fold the simplification into the next active tuplet implementation slice so later stories can rely on a single-level child-content invariant.

Suggested wording:

> Before deeper tuplet authoring and editing work continues, simplify the active tuplet child-content model so nested `Tuplet` entries are not permitted in MVP-owned child content.

#### Story wording updates needed before development continues

Story 1.4: Enter Tuplet Notes in the Sequencer

- Add an opening implementation slice that removes nested `Tuplet` allowance from tuplet-owned child content before deeper note-entry logic lands.
- Add acceptance language that note entry inside a tuplet supports only single-level MVP tuplets.
- Add rejection language for any attempt to create or require a nested tuplet inside an existing tuplet span.

Suggested addition:

> Given the composer is already working inside an MVP tuplet span, when an action would require creating another tuplet inside that governed local frame, then the product rejects or defers that action, and MVP does not create nested tuplets.

Story 1.5: Create Tuplets Over Existing Material Without Losing Ownership

- Clarify the story applies to host-track material, not to creating tuplets inside existing tuplets.
- Add a guard that tuplet reconciliation never emits nested tuplet child entries in MVP.

Suggested addition:

> Given the target span already falls inside an existing tuplet-governed region or would require nested tuplet structure, when tuplet creation is attempted, then MVP does not create the new tuplet and the existing structure remains unchanged.

Story 2.1: Target Tuplet Containers and Members Reliably

- Clarify targeting covers one container depth only for MVP.
- Remove any future story elaboration that assumes nested selection depth.

Story 2.3: Add and Remove Notes Within Tuplet Slots

- Clarify slot operations cover notes and empty space only, not nested-tuplet insertion.

Story 2.4: Reposition or Resize Tuplet Notes Within Supported Bounds

- Clarify supported edits exclude outcomes that would require nested tuplets.

Suggested addition:

> Given an edit would require nested tuplet structure to remain musically valid, when the composer attempts that edit, then MVP rejects or defers the edit rather than creating nested tuplets implicitly.

Story 2.5: Apply Explicit MVP Delete and Slice Rules for Tuplet Members and Containers

- Clarify delete and slice rules do not include nested repair behavior.

Story 3.1: Parse Tuplet-Spanned Content Into Backend-Owned Notation

- Explicitly scope parser work to single-level tuplets in MVP.
- Remove any future acceptance or notes that imply nested parser behavior is expected for MVP.
- Let parser work depend on the simplified non-nestable child-content rule rather than recursive nested-tuplet branching.

Story 3.3: Render Tuplet Notes, Rests, Ties, and Beams Coherently

- Clarify rendering expectations are for single-level tuplets only.

Story 4.1: Project Tuplet Timing Into Host Musical Time

- Clarify projection scope is one tuplet level for MVP.

Story 5.1: Add Targeted Regression Coverage for Each Tuplet Seam

- Remove nested tuplet coverage from the MVP regression target list.

Story 5.3: Document Tuplet Scope, Editing Rules, and Persistence Impact

- Add explicit requirement that docs state tuplets are non-nestable in MVP.

Story 5.4: Update Project Guardrails for Future Tuplet Work

- Add explicit guidance that MVP intentionally disallows nested `Tuplet` child entries and that future reintroduction would require explicit post-MVP planning.

### 4.3 Architecture Updates

#### Architecture change 1: Child Track Shape

Section: Child Track Shape

OLD:

> Recommended MVP child-content scope:
>
> - tones
> - nested tuplets
> - anything naturally attached to tones such as articulations through the tone itself

NEW:

> Recommended MVP child-content scope:
>
> - tones
> - anything naturally attached to tones such as articulations through the tone itself
>
> MVP structural rule:
>
> - tuplet-owned child content must not contain nested `Tuplet` entries.
> - Any helper or shared storage structure should be constrained so nested tuplets cannot be authored or persisted in MVP paths.

Rationale: This is the highest-value architecture correction.

#### Architecture change 2: Selection guidance

Section: Tone Selection / Tuplet Selection

OLD:

> `tuplet_path` expresses nested containment when needed

NEW:

> MVP selection behavior does not require nested tuplet targeting or nested user-facing containment semantics.
> Any remaining path-style lookup data should be treated as implementation residue, not as active MVP product surface.

Rationale: Preserve simple MVP targeting rules once nested child tuplets are removed from active structure.

#### Architecture change 3: Regression strategy

Section: Parser Regressions

OLD:

> - nested tuplets in parser-only coverage if the data model permits them

NEW:

> - no nested tuplet parser coverage is required for MVP
> - MVP regression scope assumes the simplified non-nestable child-content rule

Rationale: Removes unnecessary validation scope before MVP is proven.

#### Architecture change 4: MVP Boundaries

Section: MVP Boundaries

OLD:

> In scope for MVP:
>
> - nested tuplet-capable data model

NEW:

> In scope for MVP:
>
> - light structural simplification that removes nested `Tuplet` entries from MVP child content
>
> Out of scope for MVP:
>
> - nested tuplet behavior, creation, editing, parsing, rendering, playback, and validation expectations

Rationale: The architecture should stop blurring internal extensibility with MVP scope.

### 4.4 UX Specification Updates

#### UX change 1: Add explicit MVP boundary

Section: Tuplet UX Requirements, MVP Scope Boundaries

OLD:

> MVP requires clear sequencer presentation and note entry inside supported triplet regions.
> MVP does not require a fully generalized sequencer-native tuplet editing language for arbitrary future tuplets.

NEW:

> MVP requires clear sequencer presentation and note entry inside supported triplet regions.
> MVP tuplets are explicitly non-nestable.
> No nested-tuplet creation affordance, targeting model, or editing expectation belongs in MVP UX scope.
> MVP does not require a fully generalized sequencer-native tuplet editing language for arbitrary future tuplets.

Rationale: UX should not leave room for nested popovers, nested regions, or nested target states appearing in later stories.

#### UX change 2: Add note-entry guardrail

Section: Tuplet Note and Empty-Slot States

OLD:

> When the user prepares note entry inside a tuplet region, the sequencer should apply the active tuplet multiplier automatically rather than asking the user to translate written values into approximate host timing.

NEW:

> When the user prepares note entry inside a tuplet region, the sequencer should apply the active tuplet multiplier automatically rather than asking the user to translate written values into approximate host timing.
> If an action would require nested tuplets, MVP should reject or defer that action rather than exposing nested-tuplet authoring behavior.

Rationale: This is the UX seam most likely to drift in Story 1.4.

### 4.5 Implementation Model Simplification

#### Structural change proposal

OLD:

> Tuplet-owned child content currently uses a recursive shape that can also hold `Tuplet` entries.

NEW:

> For MVP, tuplet-owned child content should be simplified so it cannot contain `Tuplet` entries.
> The preferred implementation is the lightest safe change that removes nested tuplet allowance from authoring and persistence paths without undoing completed Stories 1.1 to 1.3.

Recommended implementation direction:

- Keep the backend-owned tuplet container model.
- Keep exact-timing and boundary-crossing direction intact.
- Narrow tuplet child content to non-tuplet musical entries for MVP, ideally tones-only if that is sufficient for the active stories.
- Replace open-ended recursive acceptance with one explicit invariant: no `Tuplet` may be stored inside tuplet-owned child content.

Rationale: This reduces downstream defensive logic while preserving the core architecture direction.

### 4.6 Implementation Artifact Assessment

#### Completed stories

No wording rollback is required for Stories 1.1 to 1.3, but the revised plan does include a follow-up internal simplification.

Assessment:

- Story 1.1 may continue to exist as completed work, but its nested-storage language should now be treated as superseded by the revised MVP plan.
- Story 1.2 is already aligned with single-level creation.
- Story 1.3 is already aligned with single-level sequencer presentation.

Optional cleanup only, not required for the smallest viable correction:

- Add a short note to Story 1.1 clarifying that the revised MVP plan removes nested `Tuplet` child-content support.

Recommendation: do not spend time editing completed artifacts unless the team wants stronger archival consistency; prioritize active planning docs and the light refactor instead.

## 5. Implementation Handoff

### Scope Classification

Moderate.

Reason: This does not require a fundamental replan, but it does require coordinated backlog and planning-artifact updates plus a small internal model simplification before or at the start of Story 1.4.

### Handoff Recipients

- Product Manager / Product Owner: apply the planning-artifact wording changes.
- Scrum Master: update upcoming story wording before the next story is opened for dev.
- Development team: apply the light child-content simplification and continue implementation against the narrowed MVP, with no rollback of Stories 1.1 to 1.3.
- Architect: align architecture wording so internal recursive representation remains optional rather than an MVP obligation.

### Success Criteria For Handoff

- Stories 1.1 to 1.3 remain done with no rollback.
- PRD, architecture, epics, and UX spec all explicitly state that MVP tuplets are non-nestable.
- The implementation model no longer permits nested `Tuplet` entries in tuplet-owned child content for MVP paths.
- Story 1.4 and later backlog items no longer imply nested behavior.
- Regression and documentation stories no longer include nested tuplets in MVP expectations.
- The active MVP model relies on a simple child-content invariant rather than recursive nested allowance.

### Sprint-Status Recommendation

Do not change current story completion statuses.

Do not add or remove stories.

Do not renumber epics or stories.

If the proposal is approved, `sprint-status.yaml` can remain structurally unchanged because this is a scope clarification, not a backlog topology change.

## Final Recommendation

Approve a narrow MVP scope correction plus a light structural simplification:

- Keep completed Stories 1.1 to 1.3 intact.
- Do not broaden MVP into nested tuplets.
- Correct the planning artifacts and upcoming story wording now, before Story 1.4 begins.
- Simplify the implementation model so tuplet-owned child content cannot contain nested `Tuplet` entries.
- Keep the refactor light and local rather than turning it into a broader model rewrite.
