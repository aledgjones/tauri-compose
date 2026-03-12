# Sprint Change Proposal

**Project:** compose  
**Date:** 2026-03-11  
**Change Trigger:** Implementation-readiness assessment found traceability-risk requirement numbering gaps, weak story completion signals in epics, and missing story-level traceability for UX accessibility and responsive-shell expectations.

## Section 1: Issue Summary

The implementation-readiness review found that the planning set is close to executable but still has three planning defects that should be corrected before sprint execution begins.

First, the PRD and epics both contain FR and NFR numbering gaps. The requirement set is substantively complete, but the identifier gaps create avoidable traceability risk when stories, tests, and review comments refer to requirement numbers.

Second, several stories in the epics document rely on deferment-oriented acceptance criteria such as "can remain for later stories" or "does not yet require" instead of defining a clearly testable done state. The most notable case is Story 1.1, which reads more like an enabling technical milestone than a user-meaningful sprint story.

Third, the UX specification already defines accessibility, focus-management, and responsive panel-orchestration expectations, but those expectations are only weakly reflected in story acceptance criteria. That leaves implementation teams to rediscover UX obligations instead of inheriting them explicitly from the planning set.

## Section 2: Impact Analysis

### Epic Impact

- Epic 1 is affected most directly. Story 1.1 needs to be rewritten so it has a stronger independent completion signal, and Stories 1.2, 1.3, and 1.5 should carry clearer positive acceptance outcomes.
- Epic 2 needs targeted acceptance-criteria strengthening in Stories 2.1, 2.3, 2.4, and 2.5 to cover positive completion signals plus accessibility and focus-management traceability.
- Epic 4 needs targeted strengthening in Stories 4.2 and 4.3 so responsive-shell and playback-scope UX expectations are explicitly represented.
- No new epic is required.
- No epic resequencing is required.

### Story Impact

Affected stories:

- Story 1.1: rewrite story framing and acceptance criteria
- Story 1.2: strengthen done-state criteria
- Story 1.3: strengthen done-state criteria
- Story 1.5: add responsive and non-color orientation traceability
- Story 2.1: strengthen done-state criteria
- Story 2.3: strengthen done-state criteria
- Story 2.4: add focus-management accessibility traceability
- Story 2.5: add non-color active-flow accessibility traceability and stronger completion wording
- Story 4.2: add responsive shell-preservation traceability
- Story 4.3: strengthen playback done-state wording

### Artifact Conflicts

**PRD:**

- Needs identifier cleanup for FR and NFR numbering.
- No requirement-content change is necessary; this is a traceability correction rather than a scope change.

**Epics:**

- Must be updated to match the corrected PRD requirement identifiers.
- Must strengthen identified weak stories.
- Must absorb UX accessibility and responsive expectations into relevant story acceptance criteria.

**Architecture:**

- No architecture-document change is required for this correction cycle.
- Existing architecture already supports the UX and planning direction in question.

**UI/UX Specification:**

- No UX-document content change is required.
- Existing UX guidance is sufficient; the gap is downstream traceability into stories, not missing UX source material.

### Technical Impact

- No code rollback is required.
- No persistence or migration impact is introduced by these planning corrections.
- No change to MVP scope is required.
- Implementation should become safer because story-level acceptance criteria will more directly encode the existing UX and traceability obligations.

## Section 3: Recommended Approach

### Option Evaluation

**Option 1: Direct Adjustment**  
Viability: Viable  
Effort: Medium  
Risk: Low

This option updates the PRD identifiers, aligns the epics document to the corrected numbering, and strengthens targeted story acceptance criteria without changing product scope or implementation ordering.

**Option 2: Potential Rollback**  
Viability: Not viable  
Effort: High  
Risk: Medium

There is no implementation rollback to perform because the issue was discovered during planning/readiness review, not after feature code landed.

**Option 3: PRD MVP Review**  
Viability: Not viable as primary path  
Effort: High  
Risk: Medium

The readiness findings do not indicate that the MVP itself is wrong. They indicate that the planning artifacts need sharpening. Reducing or redefining the MVP would be disproportionate.

### Selected Approach

**Selected approach: Option 1 - Direct Adjustment**

This is the best path because the problem is planning quality, not product direction. The MVP remains sound, epic ordering remains sound, and architecture/UX intent are already aligned. The correct response is to repair identifier hygiene, tighten story quality, and make existing UX expectations explicit where implementation teams need them.

### Scope Classification

**Moderate**

The change affects multiple planning artifacts and should be coordinated before sprint execution, but it does not require a fundamental re-plan or architectural reset.

## Section 4: Detailed Change Proposals

### A. PRD Changes

#### Proposal A1: Renumber Functional Requirements Sequentially

**Artifact:** PRD  
**Section:** Functional Requirements

**OLD:**

- FR1 ... FR15
- FR17 ... FR31
- FR33 ... FR34
- FR38 ... FR41

**NEW:**

- Renumber the existing 35 functional requirements sequentially as FR1 through FR36 with no gaps.

**Identifier mapping:**

| Old  | New  |
| ---- | ---- |
| FR17 | FR16 |
| FR18 | FR17 |
| FR19 | FR18 |
| FR20 | FR19 |
| FR21 | FR20 |
| FR22 | FR21 |
| FR23 | FR22 |
| FR24 | FR23 |
| FR25 | FR24 |
| FR26 | FR25 |
| FR27 | FR26 |
| FR28 | FR27 |
| FR29 | FR28 |
| FR30 | FR29 |
| FR31 | FR30 |
| FR33 | FR31 |
| FR34 | FR32 |
| FR38 | FR33 |
| FR39 | FR34 |
| FR40 | FR35 |
| FR41 | FR36 |

**Rationale:** This removes identifier ambiguity without changing requirement intent.

#### Proposal A2: Renumber Non-Functional Requirements Sequentially

**Artifact:** PRD  
**Section:** Non-Functional Requirements

**OLD:**

- NFR1 ... NFR4
- NFR6 ... NFR11
- NFR14 ... NFR18

**NEW:**

- Renumber the existing 15 non-functional requirements sequentially as NFR1 through NFR15 with no gaps.

**Identifier mapping:**

| Old   | New   |
| ----- | ----- |
| NFR6  | NFR5  |
| NFR7  | NFR6  |
| NFR8  | NFR7  |
| NFR9  | NFR8  |
| NFR10 | NFR9  |
| NFR11 | NFR10 |
| NFR14 | NFR11 |
| NFR15 | NFR12 |
| NFR16 | NFR13 |
| NFR17 | NFR14 |
| NFR18 | NFR15 |

**Rationale:** This restores consistent requirement tracing for implementation and testing.

#### Proposal A3: Add Editorial Traceability Note

**Artifact:** PRD  
**Section:** Functional Requirements and Non-Functional Requirements lead-in

**OLD:**

- No editorial traceability note

**NEW:**

- Add a brief note stating that requirement identifiers were normalized on 2026-03-11 to remove editorial numbering gaps and improve downstream traceability, without changing requirement intent.

**Rationale:** This makes the identifier cleanup explicit to later readers and reviewers.

### B. Epic and Story Changes

#### Proposal B1: Rewrite Story 1.1 to Strengthen Independent Value

**Artifact:** Epics  
**Story:** 1.1

**OLD TITLE:**

- Introduce Multi-Flow Score Composition Without Changing Visible Behavior

**NEW TITLE:**

- Establish the Flow-Aware Composition Boundary for Multi-Flow Score Rendering

**OLD STORY:**

```text
As a maintainer,
I want score rendering to compose from flow-aware presentation inputs,
So that multi-flow score view can be introduced through small, controlled changes without destabilizing the existing score output.
```

**NEW STORY:**

```text
As a maintainer,
I want the score renderer to expose a validated flow-aware composition boundary,
So that the first visible multi-flow slice can build on an explicit, testable rendering path instead of hidden refactor work.
```

**OLD FINAL ACCEPTANCE SIGNAL:**

- Internal structure or timing visibility confirms the new path is exercised

**NEW ACCEPTANCE SIGNALS:**

- The render path can enumerate flow-aware composition inputs for one or more flows.
- Single-flow rendering still produces identical visible output.
- Developers can verify through timing or diagnostic output that the flow-aware composition path ran.
- The story establishes a completed boundary that Story 1.2 can consume directly.

**Rationale:** This preserves the enabling work but makes its completion criteria more concrete and release-relevant.

#### Proposal B2: Strengthen Story 1.2 Done State

**Artifact:** Epics  
**Story:** 1.2

**OLD:**

```text
Then written-duration and notation-derivation behavior remains unchanged
And spacing refinement, continuous scrolling, and orientation cues can remain for later stories
```

**NEW:**

```text
Then written-duration and notation-derivation behavior remains unchanged
And the rendered score shows a fixed visible gap between the two flows that is sufficient to distinguish the boundary at normal working zoom
```

**Rationale:** Replaces deferment wording with an observable outcome tied to the first visible slice.

#### Proposal B3: Strengthen Story 1.3 Done State

**Artifact:** Epics  
**Story:** 1.3

**OLD:**

```text
Then the movement feels continuous across the shared score surface
And the implementation does not yet require final boundary styling or active-flow indicators
```

**NEW:**

```text
Then the movement feels continuous across the shared score surface
And the horizontal scroll range reaches the full rendered extent of both visible flows without truncating the second flow
```

**Rationale:** Preserves scope boundaries while giving the story a stronger positive pass condition.

#### Proposal B4: Strengthen Story 2.1 Done State

**Artifact:** Epics  
**Story:** 2.1

**OLD:**

```text
Then it does not yet require full selection, popover, or active-flow behavior to be complete
And written-notation behavior remains unchanged
```

**NEW:**

```text
Then repeated hit-tests at the same visible location resolve to the same flow-correct target
And written-notation behavior remains unchanged
```

**Rationale:** Removes the deferment-only language and replaces it with a testable interaction-stability outcome.

#### Proposal B5: Strengthen Story 2.3 Done State

**Artifact:** Epics  
**Story:** 2.3

**OLD:**

```text
Then full popover behavior can remain for a later story
And the implementation preserves existing notation behavior and recalculation boundaries
```

**NEW:**

```text
Then selection recovery follows the same defined rule across repeated update scenarios
And the implementation preserves existing notation behavior and recalculation boundaries
```

**Rationale:** Keeps the story focused on selection stability rather than what is deferred.

#### Proposal B6: Strengthen Story 2.5 Done State

**Artifact:** Epics  
**Story:** 2.5

**OLD:**

```text
Then the active-flow semantics match editing behavior in score view
And broader sequencer and piano-roll alignment can remain for the next story
```

**NEW:**

```text
Then the active-flow semantics match editing behavior in score view
And the active-flow indication remains visible and unambiguous as focus moves between visible flows
```

**Rationale:** Adds a concrete completion signal without consuming Story 2.6 scope.

#### Proposal B7: Strengthen Story 4.3 Done State

**Artifact:** Epics  
**Story:** 4.3

**OLD:**

```text
Then the feature remains compatible with current audio-device-based playback expectations
And cross-flow playback can remain future work
```

**NEW:**

```text
Then the feature remains compatible with current audio-device-based playback expectations
And the playback UI continues to communicate the single active playback scope clearly while multiple flows remain visible in score view
```

**Rationale:** Replaces a deferred-scope statement with a concrete UX-compatible playback outcome.

### C. UX Traceability Additions to Stories

#### Proposal C1: Add Orientation Accessibility and Responsive Traceability to Story 1.5

**Artifact:** Epics  
**Story:** 1.5

**NEW ACCEPTANCE CRITERIA TO ADD:**

```text
Given** multiple flows are visible at compact desktop width with secondary panels collapsed
When** the score view remains the primary work surface
Then** flow boundaries and current-flow cues remain visible within the reduced workspace
And** the shell preserves the same score-first layout structure without switching to an alternate navigation model

Given** visible flow-orientation cues are present
When** a user relies on contrast or shape rather than color alone
Then** the current-flow cue remains distinguishable without depending solely on accent color
And** score readability remains intact
```

**Rationale:** Pulls responsive-shell and non-color orientation expectations directly from the UX spec into implementation-ready acceptance criteria.

#### Proposal C2: Add Focus-Management Traceability to Story 2.4

**Artifact:** Epics  
**Story:** 2.4

**NEW ACCEPTANCE CRITERIA TO ADD:**

```text
Given** a contextual popover is opened from the multi-flow score surface
When** the user navigates by keyboard
Then** focus moves predictably into the popover and back to the originating score context when the popover closes
And** the visible score context remains clear while the popover is open
```

**Rationale:** Directly traces UX accessibility and orientation-preserving overlay expectations into a relevant implementation story.

#### Proposal C3: Add Non-Color Active-Flow Accessibility Traceability to Story 2.5

**Artifact:** Epics  
**Story:** 2.5

**NEW ACCEPTANCE CRITERIA TO ADD:**

```text
Given** the score view indicates the active editing flow
When** the user navigates by keyboard or relies on high-contrast focus cues
Then** the active flow remains distinguishable through visible state cues that do not depend on color alone
And** the indicated active flow continues to match the actual edit target
```

**Rationale:** Connects UX accessibility guidance to the story that owns active-flow communication.

#### Proposal C4: Add Responsive Shell Traceability to Story 4.2

**Artifact:** Epics  
**Story:** 4.2

**NEW ACCEPTANCE CRITERIA TO ADD:**

```text
Given** the main window is resized into the compact desktop layout tier
When** secondary panels collapse or expand around the score view
Then** document, playback, and tool workflows remain reachable through the same top-level shell structure
And** multi-flow score view remains the primary visible work surface without introducing a different navigation model
```

**Rationale:** Pulls the UX responsive-shell requirement into the desktop-workflow preservation story where it belongs.

### D. Architecture and UX Document Changes

#### Proposal D1: No Architecture Document Change

**Artifact:** Architecture  
**Change:** None required

**Rationale:** The architecture document already captures the ownership boundaries, UX shell alignment, and implementation patterns required by this correction.

#### Proposal D2: No UX Document Content Change

**Artifact:** UX Design Specification  
**Change:** None required

**Rationale:** The UX source guidance is already sufficient. The issue is missing downstream traceability into epics, not missing UX intent.

## Section 5: Implementation Handoff

### MVP Impact

MVP scope is unchanged.

### High-Level Action Plan

1. Update the PRD to normalize FR and NFR numbering.
2. Update the epics document to match the corrected identifiers.
3. Rewrite Story 1.1 and strengthen the identified deferment-heavy stories.
4. Add the missing UX traceability acceptance criteria to relevant stories.
5. Re-run implementation readiness or perform a focused validation pass against PRD and epics.

### Handoff Scope

**Moderate: backlog reorganization needed**

### Handoff Recipients and Responsibilities

- Product Manager / planning owner: apply the PRD identifier normalization and confirm requirement traceability remains intact.
- Scrum Master / story owner: update the epics document, revise story wording, and confirm story order remains implementation-safe.
- Architect / reviewer: confirm that no architecture or scope drift was introduced and that UX traceability changes remain consistent with the existing architecture.

### Success Criteria for Implementation

- PRD requirement identifiers are gap-free and internally consistent.
- Epics use the same corrected identifiers as the PRD.
- Story 1.1 has an independently testable completion signal.
- The identified weak stories have positive, testable done states.
- Accessibility, focus-management, and responsive-shell expectations from the UX spec are explicitly represented in relevant story acceptance criteria.
- A follow-up readiness or validation pass no longer flags these planning issues.

## Approval and Handoff Record

**User Approval:** Approved on 2026-03-11  
**Scope Classification:** Moderate  
**Implementation Route:** Product Owner / Scrum Master coordination, then refreshed planning validation

### Workflow State Notes

- No existing `sprint-status.yaml` or `sprint-status.yml` artifact was found in the workspace at approval time.
- Because no sprint-status artifact exists yet, no status file was updated as part of this workflow.
- The approved change proposal remains the authoritative handoff artifact for the planning corrections described above.

### Handoff Instructions

1. Apply the approved PRD identifier normalization.
2. Apply the approved epic/story wording and acceptance-criteria changes.
3. Run a focused validation or implementation-readiness pass against the updated PRD and epics.
4. Once the planning artifacts are clean, proceed into sprint-planning or story-preparation workflows from the corrected planning baseline.

### Handoff Recipients

- Product Manager / planning owner: update PRD requirement identifiers and preserve traceability.
- Scrum Master / story owner: update the epic stories and acceptance criteria based on this proposal.
- Architect / reviewer: confirm the corrections preserve architectural intent and UX alignment.
