---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
---

# compose - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for compose, decomposing the requirements from the PRD, UX Design, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Composers can view multiple flows within a single continuous score presentation rendered by the product's native score renderer.
FR2: Composers can move through visible flows without using the current single-flow switching pattern for basic score reading.
FR3: The product can present flows in sequence separated by a fixed gap of at least 20 pixels, sufficient for flow boundaries to be identifiable at standard zoom level.
FR4: The product can render multiple flows simultaneously without overlapping staves, clipped content, or illegible note spacing.
FR5: The product can render adjacent flows with consistent staff sizing, clef presentation, and spacing style so they read as a unified score rather than visually independent panels.
FR6: Composers can select score content within any visible flow in the continuous presentation.
FR7: Composers can trigger and use popovers within any visible flow in the continuous presentation.
FR8: The product can resolve hit targets correctly across all rendered flows.
FR9: Composers can interact with score elements across flow boundaries with each interaction resolving to exactly one well-defined target.
FR10: The product can maintain a clearly indicated active flow for editing purposes while multiple flows are visible simultaneously.
FR11: The product can support editing interactions within the currently relevant flow while other flows remain visible.
FR12: The product can preserve existing written-notation behavior when multi-flow rendering is enabled.
FR13: The product can preserve existing written-duration behavior for musical content rendered in multi-flow contexts.
FR14: The product can render multi-flow score output without requiring users to rewrite musical input to accommodate the presentation model.
FR15: The product can preserve score output conforming to the notation behaviors documented in the project's notation reference (Elaine Gould, Behind Bars) when extending presentation beyond a single flow.
FR17: Composers can scroll through the combined multi-flow score presentation as one continuous reading surface.
FR18: The product can indicate the user's active flow and position through visible affordances while multiple flows are displayed, without requiring additional navigation steps to identify context.
FR19: The product can display flow boundaries and flow sequence clearly enough that a user can identify individual flows and their relative order without scrolling or additional navigation.
FR20: The product can support reading across multiple flows without requiring users to invoke a flow-switching interaction for basic score reading and navigation.
FR21: The product can update one flow without requiring unrelated flows to be re-derived unnecessarily.
FR22: The product can preserve isolated recalculation boundaries between flows during editing.
FR23: The product can render all non-edited flows without visible update artifacts, stutter, or layout shifts while a local edit is processed in another flow.
FR24: The product can preserve stable interaction behavior while one visible flow is updated.
FR25: The product can support multi-flow presentation without coupling all visible flows into a single mandatory recalculation unit.
FR26: macOS users can open score documents through standard desktop file workflows.
FR27: macOS users can save and reopen score documents through native desktop document flows.
FR28: The product can participate in desktop file association workflows for supported score documents.
FR29: The product can use native-style file interaction patterns appropriate to a desktop document application.
FR30: The product can support core composition and score interaction workflows without requiring network connectivity.
FR31: Composers can continue using existing playback behavior while multi-flow rendering is introduced.
FR33: The product can support desktop audio-device-based playback as part of the broader product context without making cross-flow playback a requirement for this MVP.
FR34: Composers can keep a single active flow for sequencer and piano-roll workflows while multiple flows are visible in score presentation.
FR38: The product can support future portability of score files, settings, and templates without requiring account-based sync in this MVP.
FR39: The product can remain usable as a fully offline-first composition environment even as future optional sync capabilities are considered.
FR40: Maintainers can evolve score presentation architecture without treating notation logic as part of the same capability change.
FR41: Maintainers can access render-timing output covering multi-flow rendering scenarios to validate correctness and performance during development, extending existing single-flow timing instrumentation where necessary.

### NonFunctional Requirements

NFR1: In release builds on representative target hardware, unchanged-score render requests should complete the render stage in sub-millisecond time where no score changes require re-derivation.
NFR2: In release builds on representative target hardware, interaction updates and local edits in a score of up to three flows with 60 bars each should complete quickly enough to feel immediate in normal use, with parse-and-render work where recalculation is required completing within one display frame at 60 fps (<=16ms).
NFR3: The multi-flow architecture changes must not regress single-flow render performance. A single-flow score of 3 instruments and 30 bars, currently rendering in under 1ms, must continue to render in under 1ms after multi-flow support is introduced.
NFR4: The system must preserve isolated recalculation so that editing one flow does not trigger unnecessary notation or spacing recomputation for unrelated visible flows.
NFR6: Multi-flow rendering must not introduce regressions in notation output.
NFR7: Any change to multi-flow presentation must preserve the existing written-duration behavior and broader notation semantics already relied on by the product.
NFR8: Hit targets and popovers must resolve to the correct flow and score element throughout the continuous multi-flow presentation.
NFR9: Score presentation, active-flow semantics, and interaction mapping must not expose contradictory states to the user; the visually indicated active flow must always match the flow targeted by editing interactions, and hit targets must always resolve to the flow they are visually within.
NFR10: The system must fail in ways that preserve score integrity; presentation changes must not corrupt score data, notation state, or document open/save behavior.
NFR11: Multi-flow rendering must remain within one display frame (<=16ms at 60fps) as score size scales from the single-flow baseline to the three-flow, 60-bar reference case in NFR2, with render time growing proportionally to the number of flows and bars requiring recalculation rather than as a fixed cost applied unconditionally.
NFR14: Native desktop file workflows must behave consistently with macOS Human Interface Guidelines for document-based applications, including native file open/save dialogs, file association, and document state management.
NFR15: Existing single-flow audio playback must continue to function correctly after multi-flow rendering is introduced; this feature is not required to enable cross-flow playback, but must not break it.
NFR16: Offline use must remain first-class; the feature must not introduce any network dependency into core composition, rendering, interaction, or playback workflows.
NFR17: The product should maintain an accessibility baseline consistent with Apple's macOS Accessibility Programming Guide for core workflows, ensuring keyboard navigation and VoiceOver compatibility are not deliberately degraded by this feature.
NFR18: Accessibility improvements for this feature must not come at the expense of core notation correctness, interaction clarity, or architectural soundness for the target expert user base.

### Additional Requirements

- Starter template constraint: implementation must extend the existing brownfield Tauri + React + Rust foundation rather than introduce a new starter or replatform; this should drive Epic 1 Story 1.
- Rust remains the single source of truth for score state, durable document mutations, and native score rendering; React remains the orchestration shell.
- Multi-flow support must be implemented as a derived presentation/composition layer over existing flow data rather than a new persisted score structure.
- Notation derivation must remain logically distinct from presentation composition; this work cannot become a notation-logic rewrite.
- Frontend/backend mutation boundaries and snake_case cross-boundary contracts must be preserved.
- Persistence changes are optional and should be avoided where possible; any required .soloc semantic change must be migration-gated through src-tauri/src/migrations/next.rs.
- Render orchestration must define explicit invalidation and caching boundaries so local flow edits only recalculate affected work.
- Existing selection, hit-testing, and popover contracts should be extended rather than replaced with a parallel interaction model.
- Render timing instrumentation must be extended to cover multi-flow scenarios as part of the feature contract.
- Regression protection is mandatory around written durations, notation behavior, hit-testing, and interaction continuity.
- The UX shell remains VS Code-inspired: workflow tabs left, playback centered, tools right, with the score as the dominant work surface.
- The product remains macOS-first, desktop-first, and offline-first; no network dependency or cloud assumption can enter MVP.
- The current note-entry model remains sequencer-first for primary input; score interaction remains focused on editing and directive popovers unless explicitly changed later.
- Popovers and contextual editing should remain local, orientation-preserving interactions instead of large modal flows.
- Responsive behavior should preserve the desktop shell through panel collapse/expansion rather than alternate navigation models.
- Accessibility expectations include keyboard navigation, visible focus, strong dark-theme contrast, and no deliberate regression in VoiceOver support for standard UI.
- Future page-layout and music-frame evolution must remain feasible; multi-flow design choices cannot reinforce single-flow assumptions that block pagination work.

### FR Coverage Map

FR1: Epic 1 - continuous multi-flow score presentation
FR2: Epic 1 - reading without flow switching
FR3: Epic 1 - visible flow separation
FR4: Epic 1 - clean simultaneous multi-flow rendering
FR5: Epic 1 - visually unified adjacent flows
FR6: Epic 2 - selection in any visible flow
FR7: Epic 2 - popovers in any visible flow
FR8: Epic 2 - correct hit target resolution
FR9: Epic 2 - unambiguous interaction across flow boundaries
FR10: Epic 2 - clearly indicated active flow
FR11: Epic 2 - editing in the relevant flow while other flows remain visible
FR12: Epic 3 - preserve existing written notation behavior
FR13: Epic 3 - preserve written-duration behavior
FR14: Epic 3 - no user rewrite of musical input for presentation
FR15: Epic 3 - preserve notation quality against the project notation standard
FR17: Epic 1 - continuous scrolling across flows
FR18: Epic 1 - visible active-flow and position affordances
FR19: Epic 1 - clear boundaries and ordering
FR20: Epic 1 - reading and navigation without extra flow-switching
FR21: Epic 3 - update one flow without unnecessary re-derivation elsewhere
FR22: Epic 3 - isolated recalculation boundaries
FR23: Epic 3 - no artifacts or layout shifts during local edits
FR24: Epic 2 - stable interaction while one flow updates
FR25: Epic 3 - no mandatory full multi-flow recalculation unit
FR26: Epic 4 - standard desktop open workflow remains intact
FR27: Epic 4 - native save and reopen workflow remains intact
FR28: Epic 4 - file association behavior remains supported
FR29: Epic 4 - native-style file interactions remain intact
FR30: Epic 4 - offline-first use remains intact
FR31: Epic 4 - existing playback behavior continues to work
FR33: Epic 4 - audio-device playback context remains supported
FR34: Epic 2 - active-flow continuity for sequencer and piano-roll workflows
FR38: Epic 4 - future portability remains possible without sync in MVP
FR39: Epic 4 - offline-first product boundary remains intact
FR40: Epic 3 - maintainers can evolve presentation without changing notation logic
FR41: Epic 3 - maintainers have render-timing visibility for multi-flow scenarios

## Epic List

### Epic 1: Read the Score Across Multiple Flows

Composers can view and navigate multiple flows as one continuous score surface, so larger pieces feel like a coherent document instead of isolated rendering islands.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR17, FR18, FR19, FR20

### Epic 2: Interact Confidently Within Multi-Flow Score View

Composers can select, edit, and use contextual score interactions across visible flows while keeping a clear active-flow model.
**FRs covered:** FR6, FR7, FR8, FR9, FR10, FR11, FR24, FR34

### Epic 3: Trust Multi-Flow Rendering to Stay Fast and Musically Correct

Composers and maintainers can rely on multi-flow rendering to preserve notation behavior, isolate recalculation, and remain responsive as the score grows through small, careful implementation increments.
**FRs covered:** FR12, FR13, FR14, FR15, FR21, FR22, FR23, FR25, FR40, FR41

### Epic 4: Ship Multi-Flow Without Regressing Core Desktop Workflow

Composers can adopt multi-flow score view without losing expected document, playback, offline, and future-ready product behavior.
**FRs covered:** FR26, FR27, FR28, FR29, FR30, FR31, FR33, FR38, FR39

## Epic 1: Read the Score Across Multiple Flows

Composers can view and navigate multiple flows as one continuous score surface, so larger pieces feel like a coherent document instead of isolated rendering islands.

### Story 1.1: Introduce Multi-Flow Score Composition Without Changing Visible Behavior

As a maintainer,
I want score rendering to compose from flow-aware presentation inputs,
So that multi-flow score view can be introduced through small, controlled changes without destabilizing the existing score output.

**Acceptance Criteria:**

**Given** the existing score view render pipeline
**When** rendering runs for a score with one or more flows
**Then** the renderer builds its presentation from flow-aware composition inputs
**And** the visible result remains unchanged for current single-flow use

**Given** a score containing multiple flows
**When** the updated render pipeline runs
**Then** it can enumerate and prepare more than one flow for future score composition work
**And** it does not yet need to display additional flows in the score view

**Given** the updated flow-aware composition path
**When** a score renders
**Then** existing written-duration and notation-derivation behavior remains unchanged
**And** no persisted score-format or migration change is required

**Given** a developer validating this refactor step
**When** the score render completes
**Then** the implementation exposes enough internal structure or timing visibility to confirm the new composition path is being exercised
**And** existing single-flow behavior remains the observable result

### Story 1.2: Render Two Flows in One Score Surface with Fixed Placement

As a composer,
I want the score view to show two flows in the same rendered surface,
So that multi-flow reading begins as a concrete visible capability without taking on final layout behavior too early.

**Acceptance Criteria:**

**Given** a score with at least two flows
**When** the score view renders after the flow-aware composition work is in place
**Then** the first two flows appear in the same score surface
**And** each flow is rendered in sequence using a simple deterministic horizontal placement rule

**Given** the first two flows are rendered in the same score surface
**When** the score is displayed
**Then** the second flow is visually distinct from the first
**And** the placement is sufficient to avoid overlap between the two rendered flows

**Given** a score with only one flow
**When** the score view renders through the updated path
**Then** the score output remains correct and usable
**And** the introduction of the second-flow layout path does not break single-flow rendering

**Given** two flows are rendered in the same score surface
**When** this first visible multi-flow slice is validated
**Then** written-duration and notation-derivation behavior remains unchanged
**And** spacing refinement, continuous scrolling, and orientation cues can remain for later stories

### Story 1.3: Scroll Across the Combined Multi-Flow Score Width

As a composer,
I want horizontal scrolling to move across the combined rendered width of visible flows,
So that the first multi-flow score surface can be read as one continuous workspace instead of separate fixed panels.

**Acceptance Criteria:**

**Given** a score where two flows are rendered in the same score surface
**When** the combined rendered width exceeds the current visible viewport
**Then** horizontal scrolling moves across the full combined score width
**And** both flows remain reachable through the existing score-view navigation pattern

**Given** a score with only one flow
**When** the score view renders through the updated scrolling path
**Then** horizontal scrolling remains correct for the existing single-flow case
**And** the change does not introduce regressions in current score navigation

**Given** a score with two rendered flows
**When** a user scrolls from the first flow toward the second
**Then** the movement feels continuous across the shared score surface
**And** the implementation does not yet require final boundary styling or active-flow indicators

**Given** the combined multi-flow score width is scrollable
**When** this increment is validated
**Then** rendering remains stable while scrolling
**And** written-notation behavior and persisted score semantics remain unchanged

### Story 1.4: Make Adjacent Flows Read as One Coherent Score

As a composer,
I want adjacent rendered flows to feel visually consistent and intentionally separated,
So that the multi-flow score view reads like one document instead of stitched-together rendering fragments.

**Acceptance Criteria:**

**Given** two flows are rendered in the same score surface
**When** the score view displays them side by side
**Then** staff sizing, clef presentation, and baseline spacing style remain visually consistent across both flows
**And** the flow boundary uses a clear fixed separation that distinguishes one flow from the next

**Given** two adjacent flows in the score surface
**When** a user reads from the end of the first flow to the start of the second
**Then** the transition is visually understandable without overlap or ambiguous boundary placement
**And** the score remains legible at normal working zoom levels

**Given** a score with one flow or with multi-flow rendering not yet extended beyond two visible flows
**When** the score view renders
**Then** existing score presentation remains correct
**And** this refinement does not require interaction, popover, or active-flow logic changes

**Given** this visual-coherence increment is implemented
**When** the output is validated
**Then** notation-derivation and written-duration behavior remain unchanged
**And** further orientation cues can remain for the next story

### Story 1.5: Add Flow Orientation Cues to Multi-Flow Score View

As a composer,
I want the score view to show clear flow boundaries and current flow context,
So that I can stay oriented while reading across multiple visible flows.

**Acceptance Criteria:**

**Given** two flows are rendered in the same score surface
**When** the score view is displayed
**Then** each flow boundary is visually identifiable
**And** the user can distinguish the relative order of the visible flows without extra navigation

**Given** multiple flows are visible in the score view
**When** the current flow context is shown
**Then** the UI provides a visible indication of the active or current flow position
**And** the cue does not require full editing interaction changes to be implemented in this story

**Given** a score with one visible flow
**When** the score view renders
**Then** the orientation cue behavior does not degrade the current single-flow experience
**And** score readability remains intact

**Given** this orientation increment is validated
**When** the score view is reviewed end to end for Epic 1
**Then** composers can read across multiple visible flows with clearer spatial understanding
**And** interaction-specific behavior can remain for Epic 2

## Epic 2: Interact Confidently Within Multi-Flow Score View

Composers can select, edit, and use contextual score interactions across visible flows while keeping a clear active-flow model.

### Story 2.1: Resolve Hit Targets Across Two Visible Flows

As a composer,
I want score interactions to resolve the correct target in either visible flow,
So that multi-flow score view can support precise interaction without ambiguity.

**Acceptance Criteria:**

**Given** two flows are rendered in the same score surface
**When** a user points to an interactive score element in either visible flow
**Then** hit-testing resolves the correct score target for that flow
**And** the result identifies a single unambiguous target

**Given** two visible flows with distinct rendered positions
**When** hit-testing runs near the boundary between flows
**Then** the resolved target belongs to the visually corresponding flow
**And** adjacent flow content does not steal the interaction

**Given** a score with only one visible flow
**When** hit-testing runs through the updated multi-flow-aware path
**Then** existing single-flow target resolution remains correct
**And** current score interactions remain usable

**Given** the multi-flow hit-target path is implemented
**When** this increment is validated
**Then** it does not yet require full selection, popover, or active-flow behavior to be complete
**And** written-notation behavior remains unchanged

### Story 2.2: Select Score Content in Either Visible Flow

As a composer,
I want to select score content in either visible flow,
So that the multi-flow score surface is usable for direct score work rather than passive reading only.

**Acceptance Criteria:**

**Given** two flows are visible in the same score surface
**When** a user selects a score element in either flow
**Then** the correct element becomes selected
**And** the selection state identifies the correct flow context

**Given** a new selection is made in one visible flow
**When** an earlier selection exists in the other visible flow
**Then** the selection updates predictably to the new target
**And** the UI does not present ambiguous multi-flow selection state unless explicitly supported

**Given** a score with one visible flow
**When** selection runs through the updated path
**Then** existing single-flow selection behavior remains correct
**And** current score-editing entry points remain usable

**Given** selection works across both visible flows
**When** this increment is validated
**Then** popovers and active-flow editing semantics may still remain incomplete
**And** written-notation behavior remains unchanged

### Story 2.3: Preserve Stable Selection While One Visible Flow Updates

As a composer,
I want score interaction state to remain stable while one visible flow updates,
So that local edits do not make multi-flow score interaction feel fragile or unpredictable.

**Acceptance Criteria:**

**Given** two visible flows and an active selection in one of them
**When** a local update occurs in that selected flow
**Then** the selection remains stable if the target still exists
**And** unrelated visible flow interaction state does not become invalidated unnecessarily

**Given** two visible flows and an active selection in one flow
**When** the other visible flow updates
**Then** the existing selection remains coherent
**And** the user does not see contradictory selection or target state across flows

**Given** a local update removes or invalidates the selected target
**When** the update completes
**Then** selection recovery follows a defined predictable rule
**And** the UI does not leave stale selection state behind

**Given** this interaction-stability increment is validated
**When** multi-flow updates are exercised
**Then** full popover behavior can remain for a later story
**And** the implementation preserves existing notation behavior and recalculation boundaries

### Story 2.4: Open Contextual Popovers Within the Multi-Flow Score Surface

As a composer,
I want contextual score popovers to work in either visible flow,
So that multi-flow score view supports the same local directive and editing workflows as single-flow score view.

**Acceptance Criteria:**

**Given** two visible flows in the score surface
**When** a user invokes a contextual popover for a supported score interaction in either flow
**Then** the correct popover opens for the selected target
**And** the popover is anchored to the correct visual flow context

**Given** a target near the boundary between visible flows
**When** a contextual popover is opened
**Then** the popover resolves against the correct underlying target
**And** it does not appear associated with the wrong flow

**Given** a score with one visible flow
**When** contextual popovers run through the updated path
**Then** existing single-flow popover behavior remains correct
**And** current directive workflows remain usable

**Given** this popover increment is validated
**When** popovers open across the visible multi-flow surface
**Then** active-flow semantics may still be refined separately
**And** notation-derivation and persisted score behavior remain unchanged

### Story 2.5: Make Active-Flow Editing Context Explicit in Multi-Flow Score View

As a composer,
I want the score view to show which visible flow is currently active for editing,
So that multi-flow interaction remains precise when more than one flow is on screen.

**Acceptance Criteria:**

**Given** multiple flows are visible in the score view
**When** the user selects or edits content in one flow
**Then** the active-flow context updates to the correct flow
**And** the UI exposes that state clearly enough to avoid ambiguity

**Given** a visible active-flow indication is present
**When** the user changes interaction focus to another visible flow
**Then** the active-flow state updates predictably
**And** editing interactions target the same flow that the UI indicates as active

**Given** the score view shows multiple visible flows
**When** no active-flow change is intended
**Then** passive reading or scrolling does not create contradictory active-flow state
**And** the UI does not imply an edit target different from the actual one

**Given** this active-flow increment is validated
**When** score interactions are exercised across visible flows
**Then** the active-flow semantics match editing behavior in score view
**And** broader sequencer and piano-roll alignment can remain for the next story

### Story 2.6: Preserve Single Active-Flow Behavior Across Score and Sequencer Contexts

As a composer,
I want score view multi-flow visibility to coexist with a single active flow for sequencer and piano-roll work,
So that the broader editing model stays coherent while the score view becomes more document-like.

**Acceptance Criteria:**

**Given** multiple flows are visible in score view
**When** one flow is active for sequencer or piano-roll workflows
**Then** the score view can still display multiple flows
**And** the shared active-flow concept remains coherent across surfaces

**Given** the active flow changes from score interaction or supporting workflow controls
**When** sequencer or piano-roll context is shown
**Then** the single active flow remains clearly defined
**And** multi-flow score visibility does not imply multi-flow sequencer activity

**Given** a user is working in a single-flow score scenario
**When** score, sequencer, and piano-roll behaviors run through the updated model
**Then** existing single-flow expectations remain correct
**And** no regression is introduced in established workflows

**Given** this cross-context coherence increment is validated
**When** score and sequencer workflows are exercised together
**Then** multi-flow score view remains compatible with the existing single-active-flow editing model
**And** notation behavior and persisted score semantics remain unchanged

## Epic 3: Trust Multi-Flow Rendering to Stay Fast and Musically Correct

Composers and maintainers can rely on multi-flow rendering to preserve notation behavior, isolate recalculation, and remain responsive as the score grows through small, careful implementation increments.

### Story 3.1: Add Multi-Flow Render and Recalculation Visibility

As a maintainer,
I want render timing and diagnostic output to distinguish multi-flow work from single-flow work,
So that multi-flow rendering can be evolved through small changes with evidence rather than guesswork.

**Acceptance Criteria:**

**Given** the score view render pipeline supports multi-flow composition inputs
**When** a score render completes
**Then** timing output distinguishes single-flow and multi-flow rendering work
**And** the output exposes enough detail to understand which part of the render path was exercised

**Given** a score with multiple flows
**When** a local update triggers recalculation
**Then** the diagnostic output shows whether the work was scoped to one flow or multiple flows
**And** the instrumentation is sufficient to support later invalidation improvements

**Given** a score with one flow
**When** timing output is collected through the updated path
**Then** existing single-flow timing visibility remains available
**And** the instrumentation change does not alter score output behavior

**Given** this observability increment is implemented
**When** it is validated
**Then** no written-notation, written-duration, or persisted-score behavior changes are required
**And** later optimization stories can rely on this visibility rather than introducing ad hoc measurement

### Story 3.2: Add Regression Protection for Multi-Flow Notation Safety

As a maintainer,
I want regression coverage that protects notation behavior while multi-flow rendering evolves,
So that presentation changes do not silently damage written durations or broader notation semantics.

**Acceptance Criteria:**

**Given** the multi-flow score-view work in progress
**When** regression coverage is added or updated
**Then** it explicitly protects existing written-duration behavior relevant to multi-flow rendering scenarios
**And** it confirms that presentation-layer changes do not rewrite notation semantics

**Given** existing notation-sensitive fixtures or parser-level tests
**When** multi-flow-related regression coverage is introduced
**Then** the coverage extends current test support patterns rather than creating a parallel test framework
**And** the affected notation behaviors remain locally understandable to maintainers

**Given** a single-flow scenario already covered by existing behavior
**When** the regression suite runs after multi-flow-safe changes
**Then** current notation expectations remain intact
**And** any new coverage does not weaken the existing baseline

**Given** this regression-protection increment is validated
**When** future multi-flow stories are implemented
**Then** maintainers have an explicit safety net for notation correctness
**And** no persisted score-format changes are required by this story

### Story 3.3: Isolate Recalculation to the Affected Flow

As a composer,
I want local score edits to avoid unnecessary recalculation in other visible flows,
So that multi-flow score view stays responsive and predictable as the rendered document grows.

**Acceptance Criteria:**

**Given** multiple flows are visible in score view
**When** a local edit affects only one flow
**Then** notation derivation and related recalculation are scoped to the affected flow where possible
**And** unrelated visible flows do not re-derive unnecessarily

**Given** instrumentation is available for multi-flow recalculation
**When** a local edit is exercised in one visible flow
**Then** the diagnostic output shows the recalculation boundary clearly
**And** maintainers can verify that unrelated flows were not included without cause

**Given** a single-flow score
**When** the updated recalculation path runs
**Then** current behavior remains correct
**And** this change does not introduce regressions in existing render or notation behavior

**Given** this isolation increment is validated
**When** future multi-flow interactions build on it
**Then** they inherit a narrower recalculation scope
**And** no persisted score semantic change is required

### Story 3.4: Keep Non-Edited Flows Visually Stable During Local Updates

As a composer,
I want visible flows that I am not editing to remain visually stable,
So that local score changes do not make the broader multi-flow document feel jittery or fragile.

**Acceptance Criteria:**

**Given** multiple flows are visible in score view
**When** a local update occurs in one visible flow
**Then** non-edited visible flows remain visually stable
**And** the user does not see unnecessary layout shifts or redraw artifacts in unaffected flows

**Given** a local update changes the edited flow visibly
**When** the render completes
**Then** the visible update is limited to the relevant flow and any directly affected shared presentation areas
**And** unaffected flows do not appear to flicker or move without cause

**Given** a score with one visible flow
**When** the same update path runs
**Then** current single-flow rendering behavior remains intact
**And** no regression is introduced by multi-flow stabilization logic

**Given** this visual-stability increment is validated
**When** users perform local edits in multi-flow score view
**Then** the rendered document feels more dependable
**And** notation semantics remain unchanged

### Story 3.5: Verify Non-Regression and Performance Scaling for Multi-Flow Rendering

As a maintainer,
I want evidence that multi-flow rendering preserves current single-flow performance expectations and scales predictably,
So that the feature can move forward without hidden regressions in responsiveness.

**Acceptance Criteria:**

**Given** the prior observability and isolation stories are implemented
**When** representative single-flow and multi-flow scenarios are exercised
**Then** maintainers can compare current single-flow behavior against the updated multi-flow path
**And** any regression is visible through established timing output or test evidence

**Given** a representative multi-flow scenario
**When** render and update timings are reviewed
**Then** the results show whether work scales with the affected flows rather than as unconditional whole-score recalculation
**And** the outcome is documented clearly enough to guide follow-on optimization work if needed

**Given** the single-flow baseline scenario
**When** validation is performed after the multi-flow changes
**Then** current single-flow behavior remains acceptable for the existing product standard
**And** the project gains explicit evidence rather than assumptions about performance preservation

**Given** this validation increment is completed
**When** Epic 3 is reviewed as a whole
**Then** multi-flow rendering has observability, notation safety, recalculation isolation, and basic performance characterization
**And** no persisted score or migration change is required by this validation story

## Epic 4: Ship Multi-Flow Without Regressing Core Desktop Workflow

Composers can adopt multi-flow score view without losing expected document, playback, offline, and future-ready product behavior.

### Story 4.1: Preserve Document Open, Save, and Reopen with Multi-Flow Score View

As a composer,
I want scores to open, save, and reopen correctly while multi-flow score view is introduced,
So that the new score presentation does not put normal document work at risk.

**Acceptance Criteria:**

**Given** a score that can be rendered in multi-flow score view
**When** the user opens the document through the existing desktop workflow
**Then** the document loads correctly
**And** the score can render through the updated multi-flow-capable path without corrupting document state

**Given** a score viewed in the updated score view
**When** the user saves and later reopens the document
**Then** the document reopens correctly
**And** the score remains usable without requiring a new persisted multi-flow document format

**Given** a single-flow score
**When** the same open, save, and reopen workflow is exercised
**Then** existing document behavior remains correct
**And** no regression is introduced by the multi-flow presentation work

**Given** this document-safety increment is validated
**When** maintainers review the implementation
**Then** multi-flow rendering remains a presentation-layer capability
**And** no migration is required for this story

### Story 4.2: Preserve Native Desktop Document Behavior Around Multi-Flow Score View

As a macOS user,
I want multi-flow score view to coexist with normal desktop document behavior,
So that the application still feels like a reliable native document tool.

**Acceptance Criteria:**

**Given** the application supports multi-flow score rendering
**When** a user opens or saves documents through the existing native-style file workflows
**Then** those workflows remain consistent with current application behavior
**And** multi-flow score view does not force a different document interaction model

**Given** supported score documents are opened through desktop document workflows
**When** the app handles them through its existing file entry points
**Then** file association expectations remain intact
**And** the multi-flow feature does not break the normal document-opening path

**Given** a single-flow document workflow
**When** it is exercised after the multi-flow changes
**Then** current native desktop behavior remains correct
**And** no regression is introduced in established file handling patterns

**Given** this desktop-workflow increment is validated
**When** the broader feature is reviewed
**Then** the score-view enhancement remains compatible with the app's macOS-first document model
**And** no network dependency or platform expansion is introduced

### Story 4.3: Preserve Existing Playback Behavior with Multi-Flow Score Visibility

As a composer,
I want existing playback behavior to keep working while multiple flows are visible in score view,
So that multi-flow reading does not break the current listening workflow.

**Acceptance Criteria:**

**Given** multiple flows are visible in the score view
**When** a user uses the existing playback workflow
**Then** playback continues to behave according to the current product model
**And** multi-flow score visibility does not imply new cross-flow playback behavior in this story

**Given** the app uses a single active flow for current playback-related workflows
**When** the score view shows multiple visible flows
**Then** playback remains scoped according to the existing active-flow model
**And** the UI does not imply playback behavior that the system does not support

**Given** a single-flow score and the existing playback workflow
**When** playback is exercised after the multi-flow changes
**Then** current playback behavior remains correct
**And** no regression is introduced by score-view presentation changes

**Given** this playback-preservation increment is validated
**When** multi-flow score view and playback are exercised together
**Then** the feature remains compatible with current audio-device-based playback expectations
**And** cross-flow playback can remain future work

### Story 4.4: Preserve Offline-First and Future Portability Boundaries

As a maintainer,
I want multi-flow score view to avoid introducing network assumptions or unnecessary persistence coupling,
So that the feature stays aligned with the project's offline-first and future-portable architecture.

**Acceptance Criteria:**

**Given** multi-flow score view is implemented as part of the desktop application
**When** the feature is reviewed for architecture impact
**Then** it operates without any network dependency
**And** core composition, rendering, interaction, and playback workflows remain offline-capable

**Given** the feature introduces new score-view behavior
**When** maintainers review persistence impact
**Then** multi-flow rendering remains derived from existing score data rather than requiring account, sync, or cloud assumptions
**And** future score portability remains possible without introducing new MVP sync behavior

**Given** current persisted score documents
**When** they are used with the updated score view
**Then** they remain usable without a forced change to document semantics
**And** any future persistence work remains separable from this presentation feature

**Given** this boundary-preservation increment is validated
**When** Epic 4 is reviewed as a whole
**Then** multi-flow score view remains compatible with the product's offline-first desktop direction
**And** no unnecessary migration or cloud-related scope has been introduced
