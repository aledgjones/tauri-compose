# Sprint Change Proposal

**Project:** compose  
**Date:** 2026-03-22  
**Change Trigger:** Story 3.2 implementation and review exposed that the current direct instrument-assignment model is the wrong primary product object for playback setup. Stakeholder feedback redirected the product toward a backend-owned rack and routing model.

## Section 1: Issue Summary

### Triggering Story

- Story 3.2: Show Assignment Readiness in Instrument Rows and Panels

### Core Problem

This is a combined failed-approach and strategic-pivot correction.

The current planning set assumes the primary playback action is assigning a plugin directly to a score instrument and then surfacing richer health detail around that direct relationship. Story 3.2 implementation proved that this model produces the wrong UX object and therefore the wrong backend shape.

The corrected product direction is:

- Rust owns the authoritative playback model.
- The Sequence bottom panel shows the current rack and its plugin instance slots.
- Score instruments do not primarily select plugins; they route to backend-owned rack slots and any required part or channel target.
- Instrument expansion is a routing surface, including route-owned expression-map linkage.
- Plugin Manager remains the app-scoped inventory browser, not the current score's rack.

### Supporting Evidence

Stakeholder feedback gathered during Story 3.2 review was consistent and specific:

- The readiness badge in the instrument item is too much.
- The explanatory information should be compacted into a lighter contextual surface rather than expanded panel-heavy status prose.
- The bottom panel is currently being used for the wrong thing.
- The dropdown itself is acceptable, but it is selecting the wrong thing.
- The rack concept is the more standard and coherent model.
- The rack model must be backend-led in Rust, with React acting as a thin display layer only.
- The bottom panel should display the current rack and plugin instance slots.
- Instrument expansion should show routing information, including the route-owned expression map.

This feedback conflicts directly with the current Epic 3 planning language, the existing Story 3.2 implementation, and the UX specification's direct-assignment framing.

### Playback Domain Model

Compose's corrected playback model is built around a score-context rack rather than direct instrument-to-plugin assignment.

- `Rack`: the score-context playback container for the current project.
- `Slot`: a rack position that hosts exactly one plugin instance.
- `Target`: a selectable playable destination exposed by the instance hosted in a slot.
- `Address`: the engine-specific payload used to reach a target. For MVP, the only supported address variant is a strict `VstAddress` rooted in actual VST3 host concepts.
- `Route`: the backend-owned connection from a score instrument to a specific target in a slot.

This model separates stable product concepts from engine-specific playback details. Compose owns the higher-level rack, slot, target, and route abstractions. The underlying playback engine defines valid address semantics and reachable destination behavior. For MVP, `VstAddress` should only encode verified VST3-facing routing coordinates such as unit, bus, and channel associations where they truly exist, and should not carry opaque plugin-specific payloads or Compose-invented routing concepts.

### Model Principles

- Projects persist only enough rack and route intent to reconstruct complete playback state on reopen.
- Machine-local inventory and probe state remain outside project data.
- Plugin Manager is the app-scoped inventory browser, not the current score's rack.
- The Sequence bottom panel is the primary surface for the current rack.
- Instrument surfaces are primarily route-focused.
- Rack slots should be labeled in MVP as `{slot index}. {Plugin Name}` so users can distinguish multiple instances of the same plugin at a glance.
- Target names should stay plugin-native by default, with only light Compose-side disambiguation when the plugin exposes ambiguous duplicate labels.
- MVP should keep the user-facing playback model route-based even when some route mixer capabilities are conditionally available under the VST3 host boundary.
- Route-level gain, pan, and meter behavior should be treated as conditional capabilities that appear when the hosted plugin exposes real separable routing or equivalent plugin-defined signals; instance-wide mixer behavior remains slot-level.
- Plugin-instance UI and route-owned expression-map editing are opened through route-derived actions in separate windows.
- The product-facing target model should remain stable even if future playback engines introduce new address variants.
- Compose owns the higher-level playback model, but valid address semantics must remain grounded in the actual VST3 host boundary for MVP. `VstAddress` should represent VST concepts rather than reinterpret them through a broader Compose-specific payload shape.

## Section 2: Impact Analysis

### Epic Impact

#### Epic 3

Epic 3 cannot be completed as originally planned.

Its current shape is centered on direct instrument-to-plugin assignment. The new direction requires Epic 3 to be redefined around the explicit backend-owned playback topology:

- current score rack state
- slot-hosted plugin instances
- selectable targets on those hosted instances
- routes from score instruments into those targets

This is not a small acceptance-criteria adjustment. It changes the core domain object for the epic.

Recommended Epic 3 rewrite:

- Old epic: Make Score Instruments Playable
- New epic: Build the Score Rack and Instrument Routing Model

Recommended Epic 3 story set:

- 3.1: Define backend-owned rack, slot, target, address, and route state for score playback
- 3.2: Show the current rack in the Sequence bottom panel and route summaries on score instruments
- 3.3: Load or create a rack slot from the available plugin inventory
- 3.4: Create or change a route from a score instrument to a slot target
- 3.5: Reuse a compatible slot across multiple score instruments through distinct routes
- 3.6: Attempt automatic routing for new score instruments when safe
- 3.7: Repair and reroute instrument playback without restructuring the score
- 3.8: Activate rack slots and routed targets for playback
- 3.9: Preserve rack-slot and route lifecycle across the app session

Recommended sequencing principle:

- backend model definition precedes UI expression
- routing precedes convenience features built on routing
- slot activation and lifecycle stabilization happen after the model and route behavior are explicit

Status impact if approved:

- Story 3.1 should no longer remain done as written; it needs to become the backend foundation story for rack, slot, target, address, and route state.
- Story 3.2 should not remain in review; its implementation reflects the superseded UX model.
- Story 3.3 should be rewritten before development continues because slot creation now precedes route creation.

#### Epic 2

Epic 2 remains viable with targeted wording updates.

Required adjustments:

- Clarify that Plugin Manager owns app-scoped inventory, not the score's current rack.
- Clarify that native editor access remains a deliberate handoff from inventory or rack context, not proof that instruments assign directly to plugins.

No epic split or reorder is required for Epic 2.

#### Epic 4

Epic 4 remains viable, but its story language must shift from assigned instruments to routed instruments and active rack slots.

Required adjustments:

- Story 4.1 should prepare playback from routed instruments and active rack targets.
- Story 4.2 should start audible playback through rack-backed routing rather than direct assignment.
- Stories 4.3 to 4.5 should be rewritten around a simplified MVP mixer model: the user experiences routing as route-based, while slot-level instance mixing remains valid underneath and route-level gain, pan, or metering only appear when the VST3 host boundary exposes real separable route capabilities.

#### Epic 5

Epic 5 remains viable, but reopen, recovery, and trust stories must restore rack state and routing state rather than only instrument assignments.

Required adjustments:

- Story 5.5 recovery language should route users back into rack or routing repair flows.
- Story 5.6 should restore valid rack slots and instrument routes on reopen.
- Story 5.7 should degrade missing rack targets and preserve intended routing rather than only preserving direct assignment intent.

#### Epic 6

Epic 6 remains viable but needs explicit routing-aware wording.

Required adjustments:

- Story 6.3 currently links expression maps to instrument playback configurations. The corrected model should make the expression map route-owned and present it from instrument context through the routed playback configuration.
- Stories 6.4 and 6.5 should remain compatible with one shared rack/routing model rather than implying direct instrument-plugin ownership.

#### Epic 7

Epic 7 remains viable with validation wording updates.

Required adjustments:

- Validation must explicitly prove the rack workflow, including shared slot use, part or channel targeting, and recovery behavior.
- Compatibility documentation should distinguish inventory support from rack-slot workflow support.

### Artifact Conflicts

#### PRD Conflicts

The PRD remains directionally valid, but several requirements must be rewritten because they encode direct assignment as the primary model.

Affected areas:

- Executive summary language that emphasizes a simpler score-first assignment model
- FR7 through FR14, which currently frame the user action as assigning plugins directly to instruments
- FR32 through FR35, which currently describe restoring assignments on reopen rather than rack state plus routing state
- UX-facing language around directness that should now describe direct routing to a current rack object rather than direct plugin ownership by the instrument

Recommended PRD changes:

- Replace direct instrument-assignment wording with rack-backed routing wording.
- Keep MVP scope intact.
- Explicitly define the current rack as a score-context object distinct from Plugin Manager inventory.

MVP impact:

- MVP is still achievable.
- This is a model correction, not an MVP reduction.
- The MVP becomes: score-first routing into a backend-owned current rack, with compact instrument context and explicit shared-slot support.

#### Architecture Conflicts

The architecture document already supports Rust-owned truth, but its playback-domain language still assumes direct assignment as the main relationship.

Sections requiring updates:

- Requirements Overview: replace direct-assignment framing with rack-slot and route-state framing
- Data Architecture: introduce backend-owned rack, slot, target, address, and route concepts as first-class persisted or runtime models where appropriate
- API and Communication Patterns: add explicit rack-management and routing APIs
- Frontend Architecture: clarify that Sequence bottom panel owns current-rack display and instrument surfaces show routing summaries rather than plugin-centric status panels
- Requirements to Structure Mapping: remap Epic 3 capability language from assignment-first to rack-and-routing-first

Architecture-specific consequences:

- Current Story 3.1 domain modeling is insufficient if it only models instrument assignment state.
- Shared-instance behavior should become explicit rack-slot reuse, not an implied optimization hidden behind assignment resolution.
- Expression maps should be described as route-owned behavior on the routed playback configuration, not treated as an unrelated side channel.
- `Target` should remain the stable product-level destination concept, while `Address` carries the engine-specific payload used to reach that destination. For MVP, the only supported address variant is a minimal `VstAddress` composed only of real VST3 routing coordinates the host can verify.

#### UX Specification Conflicts

The UX specification has the strongest direct conflict.

Most affected requirements:

- UX-DR1: currently requires a direct assignment control on the row
- UX-DR2: currently requires a row-level readiness signal
- UX-DR3: currently expects the expanded instrument panel to mirror the same assignment model with richer contextual information
- UX-DR5: currently routes recovery into the relevant plugin panel in current score context
- UX-DR6: explicitly says plugin assignment should remain score-first rather than host-topology-oriented
- UX-DR7: explicitly frames the common workflow as direct assignment near the instrument, even when multi-part behavior exists

Recommended UX rewrite:

- Instrument rows should show compact routing state, not plugin-readiness ornamentation.
- Deeper status explanation should move to a compact contextual affordance such as a popover or lightweight disclosure rather than a status-heavy bottom-panel view.
- The Sequence bottom panel should become the current rack surface.
- Instrument expansion should focus on routing details, target part or channel, and the route-owned expression map.
- Instrument surfaces should provide route-derived icon actions that open the hosted plugin-instance UI and route-owned expression-map editor in separate windows.
- Plugin Manager should remain the app-scoped inventory library.

#### Secondary Artifact Impact

Artifacts that will require follow-up updates if this proposal is approved:

- Story file 3.2 and likely Story file 3.1
- \_bmad-output/implementation-artifacts/sprint-status.yaml
- docs/architecture.md
- docs/development-guide.md
- docs/data-models.md
- docs/component-inventory.md
- current Story 3.2 implementation files and tests

Implementation impact already visible in code:

- the current Sequence bottom-panel playback-status panel is aligned to the superseded model
- the current instrument-row readiness badge is aligned to the superseded model

Testing impact:

- backend tests will need to move closer to rack-slot and routing-state seams
- frontend tests should validate rack display, routing summaries, and recovery handoffs instead of the current selected-instrument status panel behavior

## Section 3: Recommended Approach

### Option Evaluation

**Option 1: Direct Adjustment**  
Viability: Partially viable  
Effort: High  
Risk: Medium

Updating only the remaining Epic 3 stories while leaving Story 3.1 and the Story 3.2 implementation mostly intact would preserve short-term momentum, but it would force the rack model to grow on top of a direct-assignment foundation that the stakeholder has already rejected.

**Option 2: Potential Rollback**  
Viability: Partially viable  
Effort: Medium  
Risk: Low to Medium

Rolling back the current Story 3.2 UI implementation would remove the most visible product mismatch, but rollback alone does not solve the planning defect. The domain model, story sequence, and acceptance criteria would still point at the wrong target.

**Option 3: PRD MVP Review**  
Viability: Not viable as the primary path  
Effort: High  
Risk: Medium

The issue does not show that the MVP is too large. It shows that the MVP's playback setup object model is wrong. Reducing scope would avoid the hard correction but would leave the product concept weaker.

### Selected Approach

**Selected approach: Hybrid of Option 1 and Option 2**

Recommended path:

- perform a full planning correction across PRD, epics, architecture, and UX artifacts
- reopen or replace the affected Epic 3 stories before more implementation continues
- supersede the current Story 3.2 implementation rather than building new work on top of it

Rationale:

- This keeps the MVP intact.
- It fixes the root domain model instead of polishing the wrong UX.
- It respects the Rust-owned architecture rule.
- It gives shared-slot and routing behavior an explicit backend model rather than leaving it as an edge case of direct assignment.
- It avoids compounding review debt by treating the current Story 3.2 implementation as final when the stakeholder has already rejected the underlying model.

### Scope Classification

**Major within the current epic plan, but not a project reset**

The correction changes one epic fundamentally and creates wording changes in later epics and planning artifacts. It does not require a new architecture stack, a new MVP, or a new release strategy.

## Section 4: Detailed Change Proposals

### A. PRD Changes

#### Proposal A1: Replace Direct Assignment Language With Rack Routing Language

Update the PRD requirement and summary language so that score instruments route into a backend-owned current rack instead of owning plugins directly.

Key targets:

- Executive summary references to direct assignment simplicity
- FR7 to FR14
- FR32 to FR35 where reopen and recovery restore rack and route state

#### Proposal A2: Define the Current Rack as a Score-Context Object

Add explicit wording that:

- Plugin Manager is the app-scoped inventory browser
- the current rack is the score-context playback object
- score instruments route to rack slots and optional part or channel targets

#### Proposal A3: Keep MVP Scope But Narrow Topology Claims

Clarify that MVP supports:

- rack slot loading
- instrument-to-slot routing
- explicit part or channel targeting for compatible plugins
- shared use of compatible rack slots across score instruments

Clarify that MVP still does not imply arbitrary DAW-style routing.

### B. Epic and Story Changes

#### Proposal B1: Redefine Epic 3 Around Rack and Routing

Replace the current Epic 3 title and story sequence with the rack-and-routing-first structure described in Section 2.

The rewritten epic should explicitly treat rack, slot, target, address, and route state as the score-context playback topology, rather than treating plugin assignment as the primary user model.

#### Proposal B2: Reopen Story 3.1

Story 3.1 currently describes playback assignment state on score instruments. That should be rewritten as the backend foundation story for rack, slot, target, address, and route state, then moved out of done status.

#### Proposal B3: Supersede Story 3.2

Story 3.2 should be rewritten to cover:

- bottom-panel current-rack display
- compact instrument route summaries
- lightweight contextual status detail
- routing-focused expansion content
- route-derived icon actions for opening the hosted plugin-instance UI and route-owned expression-map editor in separate windows

Its current implementation should not be treated as the accepted outcome.

#### Proposal B4: Rewrite Story 3.3 Before Continuing Development

Manual assignment should no longer mean choosing a plugin directly for an instrument. It should mean preparing or selecting a rack slot, then routing the instrument to a target in that slot.

#### Proposal B5: Cascade the Terminology Into Epics 4 to 7

Update later stories so they consistently refer to:

- routed instruments
- active rack slots
- slot targets and their engine-specific addresses where relevant
- restored rack and route state
- route-owned expression-map linkage on the routed playback configuration

### C. Architecture and UX Changes

#### Proposal C1: Add Explicit Rack Data Model Guidance to Architecture

Document rack, slot, target, address, and route concepts and their ownership boundaries.

Make explicit that `target` is the stable product abstraction and `address` is the engine-specific routing payload, with only a strict, VST-rooted `VstAddress` supported in MVP.

#### Proposal C2: Rewrite UX Spec Around Rack Visibility and Routing Context

Replace direct-assignment-first UI language with:

- rack visibility in the Sequence bottom panel
- compact routing state in rows
- routing detail in expansion or popover surfaces
- route-owned expression maps as routing-adjacent context
- route-derived icon actions that open plugin-instance UI and route-owned expression-map editing in separate windows

#### Proposal C3: Update Brownfield Docs Impact Notes

Once approved, update implementation-facing docs so contributors do not keep building toward the superseded selected-instrument status panel model.

## Section 5: MVP Impact and High-Level Action Plan

### MVP Impact

MVP remains viable and should not be reduced.

What changes is the path to MVP:

- from direct instrument assignment with secondary shared-instance handling
- to backend-owned rack slots with explicit instrument routing and shared-slot behavior

### High-Level Action Plan

1. Update PRD, architecture, UX specification, and epics to the rack-and-routing model.
2. Rewrite Epic 3 stories and acceptance criteria before more Epic 3 implementation proceeds.
3. Update sprint status so Story 3.1 is reopened and Story 3.2 is no longer treated as an accepted review candidate.
4. Remove or supersede the current Story 3.2 UI implementation.
5. Implement the backend rack, slot, target, address, and route model before returning to Sequence UI polish.
6. Rebuild Sequence surfaces around the current rack, route summaries, and route-derived secondary window actions.
7. Re-anchor later stories and tests to rack-slot and routing seams.

### Dependencies and Sequencing

- Planning corrections must happen before further Epic 3 implementation.
- Backend rack, slot, target, address, and route state must be established before revised Sequence UI work.
- Reopen and expression-map work should consume the rack model rather than define parallel playback relationships; expression maps remain owned by routes within that model.

## Section 6: Agent Handoff Plan

### Product Manager / Architect

- Update PRD language and architecture decisions.
- Confirm MVP wording and risk posture.
- Approve the rack and routing domain model.

### Product Owner / Scrum Master

- Rewrite Epic 3 story sequence.
- Update sprint-status.yaml after approval.
- Ensure Story 3.1 and Story 3.2 statuses reflect the approved correction.

### Development Team

- Remove or supersede the current Story 3.2 UI implementation.
- Implement backend rack-slot and routing seams.
- Rebuild Sequence surfaces on top of the corrected backend model.

## Section 7: Approval Request

Recommended approval condition:

- approve the rack-and-routing pivot as the new Epic 3 foundation
- approve reopening Story 3.1 and superseding the current Story 3.2 implementation
- approve downstream wording updates across PRD, architecture, UX, epics, and docs

If approved, the next action is to update the planning artifacts and sprint tracking before any more Epic 3 feature implementation proceeds.
