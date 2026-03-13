---
project: compose
date: 2026-03-12
change_trigger: implementation-readiness report for foundational tuplet support
mode: hybrid
---

# Sprint Change Proposal

**Project:** compose  
**Date:** 2026-03-12  
**Change Trigger:** Implementation-readiness review found Epic 1 dependency leakage into later epics, over-broad Story 1.2 scope, a technical-milestone-shaped Story 1.4, and incomplete feature-level tracing for key tuplet UX obligations.

## Section 1: Issue Summary

The current tuplet planning set has strong requirement coverage but is not fully implementation-ready. Epic 1 currently requires rendering and playback outcomes that belong to later epics, which weakens epic independence and creates delivery-order ambiguity. Story 1.2 mixes popover behavior, sequencer reveal, visual treatment, empty-state behavior, reconciliation, and later-epic derivation concerns into one oversized story. Story 1.4 is framed more like an internal milestone than a clean user-facing slice. In parallel, the architecture and stories do not yet make some UX obligations explicit enough, especially sticky sequencer annotation behavior, anchored inline unsupported-edit feedback, and keyboard/focus accessibility behavior.

## Section 2: Impact Analysis

### Epic Impact

- Epic 1 required restructuring so its stories can complete without depending on Epic 3 notation rendering or Epic 4 playback behavior.
- Epic 2 remained structurally sound, but its selection story benefited from an explicit non-color state communication requirement.
- Epic 3 absorbed score-rendering obligations that were previously embedded too early in Epic 1.
- Epic 4 retained playback ownership, with cross-boundary playback responsibility kept there rather than implied in authoring stories.
- Epic 5 did not require scope changes.

### Story Impact

- Story 1.2 was narrowed to popover creation and focus behavior only.
- New Story 1.3 now owns sequencer reveal, selection handoff, sticky annotation, compressed tick presentation, and empty-region visibility.
- Former Story 1.3 became Story 1.4 and now focuses on note entry only.
- Former Story 1.4 became Story 1.5 and was reframed around user-visible preservation of ownership and written intent during creation over existing material.
- Former Stories 1.5 and 1.6 were renumbered to 1.6 and 1.7.
- Story 3.3 now explicitly owns empty rest-bearing score rendering for empty tuplets.
- Story 2.1 now explicitly requires target states to remain distinguishable beyond color alone.

### Artifact Conflicts

- [epics.md](/Users/aledjones/compose/_bmad-output/planning-artifacts/epics.md) required direct restructuring.
- [architecture.md](/Users/aledjones/compose/_bmad-output/planning-artifacts/architecture.md) required explicit UX-contract additions for sequencer presentation, inline rejection feedback, and focus/accessibility behavior.
- [sprint-status.yaml](/Users/aledjones/compose/_bmad-output/implementation-artifacts/archive/2026-03-pre-new-epics-context/sprint-status.yaml) required realignment to the current tuplet epic/story set because it still tracked the completed multi-flow plan.
- No mandatory PRD text change was applied in this objective pass.

### Technical Impact

- No code rollback is required.
- No architecture reset is required.
- No MVP scope reduction is required.
- The main effect is cleaner implementation ordering, sharper story boundaries, and reduced interpretation risk for future story creation.

## Section 3: Recommended Approach

### Selected Approach

**Direct Adjustment with selective clarification**

This is the correct path because the readiness issues are planning-shape problems, not product-direction failures. The underlying tuplet strategy remains sound. The solution is to re-slice Epic 1, localize responsibilities to the right epics, and make already-agreed UX behavior explicit where implementation could otherwise drift.

### Effort, Risk, Timeline

- Effort: Medium
- Risk: Low
- Timeline impact: Small planning-only delay that reduces downstream implementation churn

## Section 4: Detailed Change Proposals

### Stories

**OLD:** Story 1.2 mixed popover flow, sequencer reveal, visual treatment, empty-state behavior, reconciliation, and boundary derivation.

**NEW:**

- Story 1.2 owns popover invocation, structure entry, validation, and focus return.
- Story 1.3 owns sequencer reveal, immediate selection, sticky `3:2` annotation behavior, compressed tick presentation, and visible empty-region targeting.
- Story 1.4 owns note entry inside the created region.
- Story 1.5 owns creation over existing material while preserving ownership and written intent.
- Score rendering of empty rest-bearing tuplets moved to Story 3.3.
- Playback consequences of crossing tones remain in Epic 4.

**Rationale:** This restores Epic 1 independence and gives each story one dominant user outcome.

### Architecture

**OLD:** UX-specific sequencer presentation, inline rejection feedback, and focus behavior were implied but not explicitly contracted.

**NEW:** The architecture now defines:

- sequencer reveal and presentation obligations for created tuplets
- sticky centered annotation and compressed internal tick behavior as explicit product contract
- unsupported-edit handling as existing product rejection behavior rather than new tuplet-specific messaging
- focus entry/return rules for the tuplet popover
- non-color distinction requirements for container, note, and empty-slot targeting

**Rationale:** This closes the main UX-to-architecture traceability gap without changing the core technical model.

### Product Decision Record

- Keep existing popover and rejection behavior for MVP rather than inventing tuplet-specific error messaging.
- Promote tuplet-specific keyboard/focus accessibility expectations into the PRD rather than leaving them only in architecture and story text.

### Sprint Tracking

**OLD:** Sprint status still reflected the completed multi-flow plan.

**NEW:** Sprint tracking now reflects the current tuplet epic and story inventory in backlog state.

**Rationale:** The planning system needs one coherent current source of sprint truth.

## Section 5: Implementation Handoff

**Scope Classification:** Moderate

**Route To:** Product Owner / Scrum Master coordination, then story creation for development

**Responsibilities:**

- Product/Planning owner: confirm whether accessibility and focus expectations should be elevated from architecture/story traceability into feature-level PRD requirements.
- Scrum/story preparation: create story files from the re-sliced Epic 1 inventory in the new order.
- Development: implement against the updated story boundaries without pulling Epic 3 or Epic 4 responsibilities into Epic 1.

**Success Criteria:**

- Epic 1 stories can be executed without depending on notation-rendering or playback completion.
- Key tuplet UX expectations are owned by explicit architecture/story language rather than inferred from the general UX spec.
- Sprint tracking and future story generation align with the current tuplet plan.
