# Sprint Change Proposal

Date: 2026-03-24
Project: compose
Change scope: Major
Workflow: Correct Course
Mode: Batch
Recommended path: Hybrid of Potential Rollback + PRD MVP Review
Approval status: Approved by Aled on 2026-03-24

## 1. Issue Summary

### Triggering stories and decisions

- Triggering story cluster: Epic 3 playback-routing and activation work, especially Stories 3.4, 3.5, 3.8, the HALion investigation tech spec, and the Epic 3 retrospective outcome.
- Triggering decision: Epic 4 is blocked by project-lead decision.

### Core problem statement

This project has reached a point where story-complete progress no longer represents trustworthy product progress. The rack-first pivot was a major improvement, but the underlying branch remains burdened by sampler-era and SCVI-era residue, uncertain VST runtime behavior, and UI churn driven by literal acceptance-criterion proving instead of the simplest useful product direction.

This is a combined issue type:

- Technical limitation discovered during implementation
- Failed approach requiring different solution
- Misunderstanding or over-literalization of original requirements
- Strategic pivot based on newly learned constraints

### Evidence

- HALion Sonic, the only available multi-timbral validation path for richer Epic 3 routing, remained a major blocker and never became a trustworthy capability proof.
- The single-instrument VST path is still not trusted enough to ship.
- Current architecture and development docs still describe active sampler or SCVI-adjacent seams as bounded compatibility scaffolding inside the runtime path rather than as fully removed behavior.
- The project lead explicitly stated that SCVI is removed in product intent but that the codebase still accommodates or works through legacy seams in practice.
- Epic 3 required repeated simplification because UI work sometimes implemented extra controls and explanatory surfaces whose main purpose was to demonstrate acceptance criteria rather than improve the product.
- The Epic 3 retrospective formally concluded that Epic 4 is blocked and that the foundation is not trusted.

## 2. Impact Analysis

### Epic impact

#### Current epic impact

- Epic 3 can remain recorded as story-complete in the historical tracker, but it should no longer be treated as a sufficient foundation for Epic 4.
- Epic 3's actual deliverable should be reframed as: a rack-first model discovery and partial technical runway, not a trustworthy playback-ready base.

#### Future epic impact

- Epic 4 cannot proceed as currently planned.
- Epic 5, Epic 6, and Epic 7 all depend on assumptions that the active VST seam is cleaner and more trustworthy than current evidence supports.
- The existing sequencing of epics assumes continuity from partial runtime activation into audible playback, transport trust, and later realism work. That assumption is now invalid.

#### New epic-level need

- A new explicit stabilization or reset epic is needed before Epic 4.
- That epic must focus on VST-only foundation cleanup, seam deletion, trust restoration, and simplification of score-context playback UX.

### Story impact

- Stories 3.5, 3.6, and 3.7 should remain deferred or blocked.
- Story 3.8 should not be used as evidence that the runtime foundation is production-credible.
- No new Epic 4 stories should be created until the foundation-reset decision is complete.

### PRD conflicts

The current PRD still conflicts with newly learned reality in several ways:

- It treats the MVP as a clean progression from VST support into trustworthy playback, but the actual foundation still carries legacy seam interference.
- It under-specifies removal behavior for sampler or SCVI-era product-path residue.
- It does not clearly distinguish between story-complete technical runway and release-trustworthy playback foundation.
- It does not explicitly state that literal UI scaffolding to satisfy ACs is a failure mode to avoid.

### Architecture conflicts

The current architecture artifacts conflict with the desired direction because they still normalize bounded legacy sampler scaffolding inside the active path instead of treating its removal as a mandatory cleanup target.

Specific conflicts:

- `SamplerContext` remains documented as bounded scaffolding, but current project direction says SCVI is removed and should stop influencing the active product path.
- Transport and activation language still reads as an evolutionary path from compatibility seams rather than a decisive VST-only seam.
- The docs do not strongly enough distinguish between tolerated temporary residue and forbidden product-path coupling.

### UX conflicts

The UX spec points toward the right score-first, direct-routing model, but implementation interpretation drifted.

Specific conflicts:

- The UX spec favors direct routing, compact score-context interaction, and calm progressive disclosure.
- Some implemented UI slices instead used multiple controls and explanatory surfaces to prove ACs individually.
- Future specs need stronger anti-bloat language so the simplest useful workflow wins over literal criterion demonstration.

### Secondary artifact impact

- `docs/project-context.md` and agent guidance need stronger deletion rules around legacy playback seams.
- `docs/development-guide.md` should stop treating sampler or SCVI residue as a tolerated normal part of the current playback path.
- Story-prep and tech-spec work should add explicit removal and non-preservation rules when prior seams are obsolete.

## 3. Path Forward Evaluation

### Option 1: Direct Adjustment

Status: Not viable

- Can the issue be addressed by modifying existing stories? Not credibly.
- Can new stories be added within the current epic structure? Not without compounding the same weak foundation.
- Would this maintain scope and momentum? It would preserve momentum superficially, but with high risk of building further on untrusted seams.

Effort: Medium
Risk: High

Why not viable:

- This path encourages more edge-tweaking on top of a foundation the project lead no longer trusts.
- It does not force a clean decision on seam deletion, branch salvage, or PRD rebaseline.

### Option 2: Potential Rollback

Status: Viable

- Would reverting recently completed work simplify the problem? Possibly, in part.
- The highest value of rollback is not to erase the rack-first idea, but to remove branch residue and code that preserves obsolete playback-path behavior.
- Full rollback of Epic 3 is not recommended because the rack-first pivot is correct.
- Targeted rollback or hard pruning of obsolete seams is strongly justified.

Effort: High
Risk: Medium

### Option 3: PRD MVP Review

Status: Viable and required

- Is the original PRD MVP still achievable? Possibly, but not under the current assumptions.
- Does scope need adjustment? Yes. A stabilization or reset phase must precede first audible playback.
- Do core goals need modification? The goals remain directionally correct, but the sequencing and explicit removal rules need revision.

Effort: Medium
Risk: Medium

### Recommended path

Selected approach: Hybrid of Option 2 and Option 3

#### Recommendation

1. Keep the rack-first product model.
2. Block Epic 4.
3. Add a new pre-Epic-4 stabilization or reset epic.
4. Rebaseline the PRD, epics, architecture, and UX language to reflect what is now known.
5. Decide explicitly whether the current branch should be salvaged through hard pruning or substantially restarted from a cleaner point.

#### Rationale

- Direct continuation is too risky.
- Full conceptual reset is unnecessary because the rack-first pivot is valuable and should be preserved.
- Partial rollback plus planning reset gives the team a chance to preserve the right ideas while deleting the wrong seams.
- This approach best balances technical reality, team morale, future maintainability, and product clarity.

## 4. Detailed Change Proposals

### 4.1 Epic changes

#### Proposal E1: Insert a new stabilization epic before Epic 4

Artifact: `_bmad-output/planning-artifacts/epics.md`

OLD:

- `Epic 4: Get From Written Score to First Audible Playback`

NEW:

- `Epic 4: Rebuild The Playback Foundation Around A True VST-Only Path`
- `Epic 5: Get From Written Score to First Audible Playback`
- Renumber later epics accordingly

Rationale:

- First audible playback should not be the next move.
- A new epic is needed to remove or hard-retire sampler or SCVI-era runtime influence, simplify the score-context playback UX around the rack-first model, and establish a trustworthy single-instrument VST base.

#### Proposal E2: Reframe current Epic 3 as discovery and runway, not trusted foundation

Artifact: `_bmad-output/planning-artifacts/epics.md`

OLD:

- `Build The Single-Target Score Rack And Routing Runway`

NEW:

- Keep title if historical continuity matters, but add explicit planning note that Epic 3 produced a corrected product model and partial runway, not a production-trustworthy playback base.

Rationale:

- Historical truth should remain intact, but later planning should not overclaim what Epic 3 delivered.

### 4.2 PRD changes

#### Proposal P1: Add a non-negotiable seam-deletion requirement

Artifact: `_bmad-output/planning-artifacts/prd.md`

OLD:

- Existing language focuses on replacing SCVI and building VST3 playback, but still allows interpretation that bounded legacy scaffolding can remain in the active path.

NEW:

- Add requirement: `Once a VST-owned seam replaces a sampler or SCVI-era product path, the superseded path must be removed from active product behavior rather than preserved as parallel runtime or compatibility ownership.`

Rationale:

- The current issue is not just missing features. It is parallel functionality and tolerated obsolete ownership.

#### Proposal P2: Add a trust-restoration gate before first audible playback

Artifact: `_bmad-output/planning-artifacts/prd.md`

OLD:

- MVP sequence implies progression from route setup into audible playback.

NEW:

- Add planning gate: `Before first audible playback stories begin, the team must establish a trustworthy single-instrument VST-only foundation with no active sampler or SCVI-era product-path interference and with score-context playback UX simplified to the minimum useful surface.`

Rationale:

- This reflects the project lead's blocker decision.

#### Proposal P3: Add product-UX anti-bloat guidance

Artifact: `_bmad-output/planning-artifacts/prd.md`

OLD:

- UX requirements describe directness and score-first behavior but do not explicitly reject AC-demonstration UI.

NEW:

- Add requirement: `Story implementation must prefer the simplest useful user-facing control surface for the product model. UI added solely to make individual acceptance criteria separately visible is considered scope failure unless it creates clear user value.`

Rationale:

- This addresses repeated overbuilt playback UI churn.

### 4.3 Architecture changes

#### Proposal A1: Tighten legacy seam language from tolerated scaffolding to removal target

Artifact: `_bmad-output/planning-artifacts/architecture.md`

OLD:

- Legacy sampler and SCVI-era seams are described as bounded compatibility scaffolding.

NEW:

- Replace with explicit rule: `Sampler or SCVI-era seams must not remain in the active playback product path once equivalent VST-owned seams exist. Temporary bridging may exist only during a bounded migration slice and must be explicitly removed before downstream playback epics proceed.`

Rationale:

- Current architecture prose is too permissive for the present blocker.

#### Proposal A2: Add branch-reset decision point

Artifact: `_bmad-output/planning-artifacts/architecture.md`

NEW:

- Add planning note: `When a playback foundation pivot materially changes ownership or product model, the team must explicitly choose between hard pruning and fresh branch restart rather than relying on iterative edge-preserving edits.`

Rationale:

- This reflects the current concern that agents only tweak around the edges.

### 4.4 UX changes

#### Proposal U1: Strengthen compact-control rule for rack-first score-context playback UX

Artifact: `_bmad-output/planning-artifacts/ux-design-specification.md`

OLD:

- UX spec emphasizes directness and compact score-context interaction.

NEW:

- Add explicit note: `For score-context playback workflows, compact direct controls are preferred over explanatory or multi-control proof surfaces. If one dropdown captures the useful product interaction, the design should not expand into extra buttons, helper text, or state callouts unless they solve a real user problem.`

Rationale:

- This is the exact failure mode called out by the project lead.

#### Proposal U2: Clarify that the rack-first model is now the canonical product direction

Artifact: `_bmad-output/planning-artifacts/ux-design-specification.md`

OLD:

- The UX spec references direct routing and the old SCVI mental-model simplicity.

NEW:

- Add explicit statement: `The rack-first model is now the canonical playback mental model for Compose. Future stories must align to it directly rather than preserving older assignment-first or legacy-sampler interaction assumptions.`

Rationale:

- This reduces future agent ambiguity.

### 4.5 Agent and supporting-doc changes

#### Proposal G1: Strengthen agent rules around deletion

Artifact: `docs/project-context.md` and `.github/copilot-instructions.md`

OLD:

- Existing guidance warns against extending legacy seams, but still tolerates compatibility scaffolding language.

NEW:

- Add explicit agent rule: `When the active story or approved planning direction says a legacy playback path is removed, agents must delete or disconnect obsolete product-path behavior rather than preserve it through compatibility residue, unless the story explicitly requires a temporary bridge.`

Rationale:

- This is needed to change agent behavior, not just planning language.

## 5. High-Level Action Plan

### Sequence

1. Approve this sprint change proposal.
2. Update epics and sprint tracking to block the current Epic 4 path and introduce the new stabilization epic.
3. Update PRD, architecture, and UX artifacts with the change proposals above.
4. Decide explicitly whether the branch will be salvaged by hard pruning or restarted from a cleaner point.
5. Only after that decision, create new stories for the stabilization epic.

### Proposed stabilization-epic themes

- Remove sampler or SCVI-era active-path residue from the VST runtime path
- Re-validate and narrow the single-instrument VST path until it is genuinely trusted
- Simplify rack-first score-context playback UX to the minimum useful product surface
- Reclassify unproven multi-instrument support as future capability, not near-ready scope

## 6. MVP Impact

### MVP status

The MVP remains directionally achievable, but not through the current next-step plan.

### MVP change

- First audible playback is no longer the immediate next milestone.
- A foundation reset or stabilization milestone becomes part of MVP delivery.

### What is deferred

- Multi-instrument or HALion-backed richer routing remains deferred.
- Any downstream transport-trust, realism, reopen, or support-baseline work must wait until the VST-only foundation is trusted.

## 7. Handoff Plan

### Scope classification

Major

### Required handoffs

#### Product Manager / Product Owner

- Rebaseline PRD goals and epic sequencing
- Decide whether MVP wording needs reduction or clearer staging

#### Scrum Master

- Manage backlog and sprint reorganization
- Create the new stabilization or reset epic and stop Epic 4 story creation

#### Architect

- Tighten architecture rules around seam deletion and branch-reset decisions
- Define the clean VST-only foundation target state

#### Development lead / team

- Assess salvage-versus-restart options for the current branch
- Identify which code can be preserved and which must be deleted outright

#### QA

- Define trust-restoration criteria for the single-instrument VST path
- Refuse to treat seam-local checks alone as sufficient when product trust remains low

## 8. Checklist Summary

- 1.1 Trigger identified: [x] Done
- 1.2 Core problem defined: [x] Done
- 1.3 Evidence gathered: [x] Done
- 2.1 Current epic impact assessed: [x] Done
- 2.2 Epic-level changes identified: [x] Done
- 2.3 Remaining epics reviewed: [x] Done
- 2.4 New epic need evaluated: [x] Done
- 2.5 Epic order and priority reviewed: [x] Done
- 3.1 PRD conflicts checked: [x] Done
- 3.2 Architecture conflicts checked: [x] Done
- 3.3 UX conflicts checked: [x] Done
- 3.4 Secondary artifacts checked: [x] Done
- 4.1 Direct adjustment evaluated: [x] Done
- 4.2 Potential rollback evaluated: [x] Done
- 4.3 PRD MVP review evaluated: [x] Done
- 4.4 Recommended path selected: [x] Done
- 5.1 Issue summary created: [x] Done
- 5.2 Impact and artifact changes documented: [x] Done
- 5.3 Recommended path and rationale documented: [x] Done
- 5.4 MVP impact and action plan defined: [x] Done
- 5.5 Handoff plan defined: [x] Done
- 6.1 Final review pending user approval: [!] Action-needed
- 6.2 Proposal accuracy review pending user approval: [!] Action-needed
- 6.3 User approval pending: [!] Action-needed
- 6.4 sprint-status update pending approval: [!] Action-needed
- 6.5 Final next-step confirmation pending approval: [!] Action-needed

## 9. Approval Request

This proposal recommends a major course correction:

- block the current Epic 4 path
- insert a new stabilization or reset epic before first audible playback
- rebaseline PRD, architecture, UX, and agent guidance
- explicitly decide between branch salvage and substantial restart

No planning artifacts besides this proposal have been changed yet.

## 10. Approval And Handoff

- User decision: Approved
- Scope classification: Major
- Immediate follow-through approved: update sprint tracking to insert the stabilization epic before first-audible playback and route the deeper planning rebaseline to PM or Architect plus SM coordination
- Routed to:
  - Product Owner / Product Manager for PRD and epic rebaseline
  - Solution Architect for foundation-reset target state and seam-deletion rules
  - Scrum Master for backlog and sprint reorganization
  - QA for trust-restoration criteria before playback work resumes
