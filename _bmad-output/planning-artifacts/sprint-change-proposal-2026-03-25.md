# Sprint Change Proposal

Date: 2026-03-25
Project: compose
Change scope: Moderate
Workflow: Correct Course
Mode: Batch
Recommended path: Hybrid of Direct Adjustment + Narrow Architecture Rebaseline
Approval status: Approved and applied

## 1. Issue Summary

### Triggering story and discovery

- Triggering story: Story 1.3 - Establish A Trusted Single-Instrument Reset Baseline.
- Triggering discovery: maintainer-guided investigation during the Story 1.3 editor-host hardening work.

### Core problem statement

Story 1.3 has produced valid trust-restoration work and exposed real lifecycle defects, but it also revealed a larger architectural issue that the current plan does not model explicitly enough: rack slots currently hold portable saved intent and readiness metadata, while the plugin editor path is still the seam that creates the only real live VST instance.

That means the current implementation can make slot and route state look coherent without the slot actually owning a persistent initialized plugin session. The resulting model is not stable enough to support Epic 4 audible-playback work, because the product expectation is stronger: a slot should own one real initialized plugin instance at runtime, `.soloc` should store only the minimum portable intent needed to rehydrate that instance on assignment or load, and the native editor should be a transient attach or detach surface on that running slot-owned instance rather than the act that creates canonical runtime state.

### Issue type

- Technical limitation discovered during implementation
- Misunderstanding of the current story's sufficiency as a trust gate
- Failed implementation shape requiring a more correct architectural seam

### Evidence

- Slot assignment currently mutates score state and refreshes playback-activation metadata, but does not instantiate a live plugin instance.
- `PlaybackActivationRuntime` resolves inventory-backed bundle paths and readiness truth, but not real hosted plugin instances.
- `plugin_editor_poc_open` and `build_plugin_host_session(...)` currently create the live VST component, controller, and view, meaning editor-open still acts as instance creation.
- Manual testing showed that plugin sample or instrument loading often occurs only when the UI is opened, which is inconsistent with the expected slot-owned runtime model.
- Same-bundle multi-slot editor issues were symptoms of this deeper ownership mismatch rather than isolated editor-window defects.
- The intended persistence boundary is narrower than runtime ownership: `.soloc` should preserve portable slot-to-plugin relationships and the minimum plugin configuration needed to restore loaded instruments or patches, while live host sessions remain runtime-only and are rebuilt on load.

## 2. Impact Analysis

### Checklist summary

- 1.1 Triggering story identified: [x] Done
- 1.2 Core problem precisely defined: [x] Done
- 1.3 Supporting evidence gathered: [x] Done
- 2.1 Current epic viability evaluated: [x] Done
- 2.2 Epic-level changes identified: [x] Done
- 2.3 Future epic impact reviewed: [x] Done
- 2.4 Need for new stories or gaps assessed: [x] Done
- 2.5 Sequencing and priority reviewed: [x] Done
- 3.1 PRD conflicts checked: [x] Done
- 3.2 Architecture conflicts checked: [x] Done
- 3.3 UX impact checked: [x] Done
- 3.4 Secondary artifact impact checked: [x] Done
- 4.1 Direct adjustment evaluated: [x] Done
- 4.2 Rollback evaluated: [x] Done
- 4.3 MVP review evaluated: [x] Done
- 4.4 Recommended path selected: [x] Done

### Epic impact

#### Epic 1 impact

- Epic 1 remains the correct home for this work because the issue is still a trust-restoration and architectural-foundation issue, not first audible playback itself.
- Story 1.3 should be treated as valid completed discovery and stabilization work once its normal review closure is complete.
- Epic 1 needs follow-up stories that turn the newly discovered architectural requirement into explicit implementation work.

#### Epic 4 impact

- Epic 4 remains blocked.
- Story 1.3 alone is not a sufficient gate for Epic 4, even if it is completed and records valid trust-restoration output.
- Epic 4 should be blocked until the slot-owned runtime-session follow-up stories are complete and the maintainer proof is rerun on that architecture.

#### Future epic impact

- Epic 5 depends on the same runtime ownership seam for reopen, recovery, and stable transport behavior.
- Epic 7 validation stories should treat slot-owned runtime sessions as part of the supported baseline, not an optional refinement.

### Story impact

- Story 1.3 should not be reopened as a failed story. It did useful work: clarified the trust gate, hardened several host-lifecycle seams, and surfaced the deeper ownership problem honestly.
- New follow-up stories are needed as a discovered link under Epic 1.
- Story 3.4 and Epic 4 planning should be updated to depend on these new Epic 1 follow-up stories rather than implying that slot-session stability has already been established.

### PRD conflicts

The PRD currently implies the right product behavior but does not state the runtime ownership seam and persistence boundary strongly enough.

Specific conflict:

- The product intent says rack slots are meaningful playback resources, but the plan does not explicitly require a slot to own a real initialized plugin session independently of editor-window lifetime while keeping `.soloc` limited to portable rehydration intent rather than live runtime state.

### Architecture conflicts

The architecture document already says "Use `slot` for one hosted plugin instance in that rack," but the implementation and planning language still allow a weaker interpretation where a slot is only a plugin reference plus readiness metadata.

Specific conflict:

- Native editor hosting is treated as bounded and backend-owned, but the architecture does not yet say clearly enough that editor windows must attach to existing slot-owned sessions rather than create canonical runtime instances.

### UX impact

- No major UX redesign is required.
- The UX implication is clarifying one rule: plugin editors are deep, transient handoff surfaces for a running slot instance, not the action that initializes playback reality.

### Secondary artifact impact

- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/1-3-establish-a-trusted-single-instrument-reset-baseline.md`
- `_bmad-output/planning-artifacts/epics.md`
- `_bmad-output/planning-artifacts/prd.md`
- `_bmad-output/planning-artifacts/architecture.md`
- `_bmad-output/planning-artifacts/ux-design-specification.md`
- `docs/architecture.md`
- `docs/project-context.md`

## 3. Path Forward Evaluation

### Option 1: Direct Adjustment

Status: Viable

- The product direction does not need a new reset epic or rollback.
- The issue can be addressed by adding explicit follow-up stories under Epic 1 and tightening the planning and architecture language.
- This preserves the valid work already done in Story 1.3 while acknowledging that the original gate was necessary but not sufficient.

Effort: Medium
Risk: Medium

### Option 2: Potential Rollback

Status: Not viable

- A rollback is not warranted because the newly discovered issue is architectural incompleteness, not evidence that Story 1.3's work was wrong.
- Reverting the Story 1.3 seam hardening would discard useful lifecycle and trust-gate improvements without solving the ownership problem.

Effort: High
Risk: High

### Option 3: PRD MVP Review

Status: Partially viable but narrower than a full MVP reset

- The MVP itself is still directionally correct.
- What needs changing is the explicit architectural gate before Epic 4, not the overall product goal.

Effort: Low to Medium
Risk: Low

### Recommended path

Selected approach: Hybrid of Option 1 and a narrow Option 3

#### Recommendation

1. Keep Story 1.3 as valid completed trust-restoration work once its normal review closure completes.
2. Add explicit discovered-link follow-up stories under Epic 1.
3. Tighten PRD, architecture, UX, persistence, and sprint-tracking language so slot-owned runtime sessions become a non-negotiable gate before Epic 4 while `.soloc` remains limited to portable rehydration intent.
4. Leave Epic 4 blocked until those follow-up stories are complete and the maintainer proof is rerun.

#### Rationale

- This is not a product pivot.
- It is a sharper understanding of what the trust gate actually has to prove.
- The cleanest move is to preserve Story 1.3's valid discoveries, avoid pretending it fully solved runtime ownership, and schedule explicit follow-up implementation stories rather than smuggling the work into Epic 4.

## 4. Detailed Change Proposals

### 4.1 Epic 1 story changes

#### Proposal E1: Reframe Story 1.3 as a valid baseline-discovery story, not the final runtime-ownership gate

Artifact: `_bmad-output/planning-artifacts/epics.md`

OLD:

- `Planning Note: Epic 4 audible-playback stories remain blocked until Story 1.3's automated single-instrument-VST-instance trust gate passes and the maintainer manual proof flow records a go verdict. Epic 3 remains discovery runway only, not a production-trustworthy playback baseline.`

NEW:

- `Planning Note: Story 1.3 establishes the explicit trust-restoration baseline and records whether the pre-Epic-4 path is conceptually stable enough to continue. Epic 4 audible-playback stories remain blocked until the follow-up slot-owned runtime-session stories are complete and the maintainer manual proof flow records a go verdict on that architecture. Epic 3 remains discovery runway only, not a production-trustworthy playback baseline.`

Rationale:

- This keeps Story 1.3 valid while removing the false implication that Story 1.3 alone proves the final ownership seam.

#### Proposal E2: Add Story 1.4 to establish slot-owned live plugin sessions

Artifact: `_bmad-output/planning-artifacts/epics.md`

NEW STORY:

`### Story 1.4: Make Rack Slots Own Live Plugin Sessions`

`As a maintainer,`
`I want each routed playback slot to own one real initialized VST instance at runtime,`
`So that playback and plugin state no longer depend on editor-window lifetime while project files remain portable.`

Acceptance Criteria summary:

- Given a slot is assigned to an available plugin, when the backend prepares or restores that slot, then Rust creates or restores a slot-owned runtime session keyed by `slot_id`.
- Given a slot-owned session exists, when playback or score-context workflows query readiness, then they reflect that real runtime ownership instead of editor-created approximation.
- Given a slot is unassigned, removed, or the project closes, when ownership ends, then the slot-owned session is torn down at that ownership boundary rather than at editor close.
- Given a project is saved, when playback state is persisted, then `.soloc` stores only the portable plugin intent and the minimum data needed to rehydrate the slot-owned session later rather than any live host-session object or machine-local runtime cache.

Rationale:

- This is the core discovered architectural gap.

#### Proposal E3: Add Story 1.5 to separate transient editor attachment from runtime session ownership

Artifact: `_bmad-output/planning-artifacts/epics.md`

NEW STORY:

`### Story 1.5: Make Native Plugin Editors Transient Views On Slot-Owned Sessions`

`As a composer,`
`I want the native plugin editor to open against the running instance already owned by a slot,`
`So that opening or closing the editor does not create, destroy, or swap the real playback instance.`

Acceptance Criteria summary:

- Given a slot-owned runtime session already exists, when I open the editor, then Compose attaches the editor to that existing slot-owned session rather than creating a second canonical instance.
- Given the editor is closed, when the slot still exists, then the editor detaches without destroying the slot-owned runtime session.
- Given multiple slots host the same plugin bundle, when I open or reopen editors across them, then each editor targets the exact slot-owned instance rather than a bundle-level approximation.

Rationale:

- This preserves plugin state and resolves the “UI creates the instance” mismatch.

#### Proposal E4: Add Story 1.6 to initialize and restore slot-owned sessions before UI-open

Artifact: `_bmad-output/planning-artifacts/epics.md`

NEW STORY:

`### Story 1.6: Initialize And Restore Slot-Owned Sessions On Assignment And Reopen`

`As a composer,`
`I want assigned slots to initialize their plugin instances during assignment, preparation, or project reopen from portable saved intent,`
`So that the plugin is already the intended running instrument before I ever open its editor.`

Acceptance Criteria summary:

- Given I assign a plugin to a slot, when the backend accepts the assignment, then the slot enters an explicit initialization or recovery path that can produce a real running slot-owned session without requiring editor-open.
- Given a project is reopened and the plugin is available, when slot and route intent are restored, then the slot-owned session is restored or reinitialized before editor-open and without rewriting project intent.
- Given the plugin cannot be initialized or restored, when the backend surfaces state, then Compose reports an explicit recoverable non-ready condition rather than silently pretending the slot is initialized.
- Given `.soloc` is loaded, when slot and plugin configuration are rehydrated, then the backend uses the portable saved intent to rebuild the slot-owned runtime session and does not require the file to contain live runtime state.

Rationale:

- This directly addresses the observed startup behavior where sounds only load when the UI opens.

### 4.2 Epic 4 gating change

#### Proposal E5: Tighten the Epic 4 blocked condition

Artifact: `_bmad-output/planning-artifacts/epics.md`

OLD:

- `Blocked until the Story 1.3 single-instrument-VST-instance trust gate is green and the required maintainer manual proof has completed successfully.`

NEW:

- `Blocked until Story 1.3 is closed as complete, Stories 1.4 through 1.6 establish slot-owned runtime-session ownership and transient editor attachment, and the required maintainer manual proof has completed successfully on that architecture.`

Rationale:

- This preserves the current block but makes the real dependency explicit.

### 4.3 PRD changes

#### Proposal P1: Explicitly define slot ownership as real initialized instance ownership

Artifact: `_bmad-output/planning-artifacts/prd.md`

OLD:

- MVP bullet: `Stable plugin loading and retained per-project rack and route state`

NEW:

- MVP bullet: `Stable slot-owned plugin instance loading and retained per-project rack and route state, where a rack slot owns one real initialized VST instance at runtime, `.soloc` stores only the portable rehydration intent needed to rebuild that state on load, and editor windows are transient surfaces on that running instance`

Rationale:

- This makes the product expectation explicit instead of leaving it implicit.

#### Proposal P2: Tighten Journey 4 gate language

Artifact: `_bmad-output/planning-artifacts/prd.md`

OLD:

- `Before Epic 4 audible-playback stories begin, maintainers must be able to prove one routed path through a single-instrument VST instance is transport-ready, editor-host lifecycle-stable, and durable across a .soloc save or reopen pass.`

NEW:

- `Before Epic 4 audible-playback stories begin, maintainers must be able to prove one routed path through a slot-owned single-instrument VST instance is transport-ready, editor-host lifecycle-stable, durable across a .soloc save or reopen pass, independent of whether the native editor window is currently open, and rehydrated from portable saved intent rather than persisted live runtime state.`

Rationale:

- This aligns the gate with the newly discovered runtime-ownership requirement.

### 4.4 Architecture changes

#### Proposal A1: Make slot-owned runtime session ownership explicit in the architecture

Artifact: `_bmad-output/planning-artifacts/architecture.md`

OLD:

- `Use slot for one hosted plugin instance in that rack.`

NEW:

- `Use slot for one hosted plugin instance in that rack.`
- `A slot is not merely an inventory reference or readiness token; it is the authoritative owner of one live initialized plugin session keyed by slot identity.`
- `Persisted project files store only the minimum portable intent required to rebuild that slot-owned session, including slot-to-plugin relationships and restorable plugin configuration where needed; they do not store live runtime host objects or machine-local runtime state.`
- `Native plugin editor windows attach to and detach from that slot-owned session; they do not create or destroy the canonical runtime instance.`

Rationale:

- The current wording is directionally right but too easy to interpret weakly.

#### Proposal A2: Clarify editor-host ownership boundaries

Artifact: `_bmad-output/planning-artifacts/architecture.md`

OLD:

- `Native plugin editor hosting remains a bounded backend-owned seam, secondary to Compose-owned playback UX.`

NEW:

- `Native plugin editor hosting remains a bounded backend-owned seam, secondary to Compose-owned playback UX.`
- `That seam owns view attachment, focus, resize, detach, and teardown ordering, but it must not become the creator of canonical slot runtime state.`

Rationale:

- This makes the editor-host boundary precise.

### 4.5 UX specification changes

#### Proposal U1: Clarify that editor surfaces are deep handoffs to already-running slot instances

Artifact: `_bmad-output/planning-artifacts/ux-design-specification.md`

OLD:

- UX language already treats native editors as secondary handoff surfaces.

NEW:

- Add clarification near the native-editor requirements: `Opening a native plugin editor is a deep handoff into an already-running slot-owned plugin instance. It is not the action that initializes the score's canonical playback instance.`

Rationale:

- No UX redesign is needed, but the runtime meaning should be explicit.

### 4.6 Sprint tracking changes

#### Proposal S1: Update sprint tracking to close Story 1.3 and add discovered-link follow-up stories

Artifact: `_bmad-output/implementation-artifacts/sprint-status.yaml`

OLD:

- `1-3-establish-a-trusted-single-instrument-reset-baseline: review`

NEW:

- `1-3-establish-a-trusted-single-instrument-reset-baseline: done`
- `1-4-make-rack-slots-own-live-plugin-sessions: backlog`
- `1-5-make-native-plugin-editors-transient-views-on-slot-owned-sessions: backlog`
- `1-6-initialize-and-restore-slot-owned-sessions-on-assignment-and-reopen: backlog`

Rationale:

- Story 1.3 has done valid work and exposed the next required seam.
- The new work should be explicit backlog, not hidden under Epic 4.

#### Proposal S2: Update the Story 1.3 implementation artifact to record the discovered link explicitly

Artifact: `_bmad-output/implementation-artifacts/1-3-establish-a-trusted-single-instrument-reset-baseline.md`

NEW NOTE:

- `Story 1.3 completed the trust-restoration baseline and revealed a deeper architectural follow-up: slot-owned live plugin sessions must exist independently of editor-window lifetime before Epic 4 can begin, while `.soloc` remains limited to portable rehydration intent rather than live runtime state. That work is intentionally split into follow-up Epic 1 stories rather than reopening Story 1.3 as a failed slice.`

Rationale:

- This preserves the integrity of the completed story while documenting the discovered gap.

## 5. Implementation Handoff

### Scope classification

Moderate

- No new reset epic is required.
- No rollback is required.
- Backlog reorganization and artifact updates are required.
- The implementation work is still architecture-significant and should not be treated as a casual bug-fix continuation.

### Handoff recipients and responsibilities

- Scrum Master / planning owner
  - Apply the approved epic and sprint-tracking edits.
  - Add Stories 1.4 through 1.6 to the active plan.

- Architect
  - Update architecture and PRD wording to make slot-owned session ownership explicit.
  - Confirm the implementation seam for per-slot runtime-session ownership, editor attach or detach, and restore behavior.

- Developer
  - Implement the follow-up Epic 1 stories.
  - Stop treating editor-open as the creator of canonical runtime state.

- QA / maintainer validation
  - Refresh manual proof after the follow-up stories land.
  - Confirm that assigned slots initialize or restore real plugin state before editor-open and that closing the editor preserves that runtime state.

### Success criteria

- Story 1.3 is recorded as complete without overstating what it proved.
- Stories 1.4 through 1.6 exist explicitly in plan and sprint tracking.
- Epic 4 remains blocked by the correct ownership gate.
- Planning and architecture artifacts state clearly that slots own live initialized plugin sessions and editors are transient attachments.

## 6. Final Review Status

Proposal accuracy review: complete

- The proposal preserves Story 1.3 as valid completed work.
- The proposal does not overclaim that Epic 4 can now begin.
- The proposal narrows the correction to the discovered architectural gap instead of reopening the broader 2026-03-24 reset.

Pending user decision:

- Approve this proposal as written
- Revise story names, scope, or sequencing
- Convert one or more proposed follow-up stories into a different epic if desired
