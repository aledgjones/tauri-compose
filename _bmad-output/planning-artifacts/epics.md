---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - _bmad-output/planning-artifacts/prd-foundational-tuplet-support.md
  - _bmad-output/planning-artifacts/architecture-foundational-tuplet-support.md
---

# compose - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for compose, decomposing the requirements from the PRD and Architecture requirements into implementable stories for foundational tuplet support.

## Requirements Inventory

### Functional Requirements

FR1: Composers can create triplet-authored passages whose performed timing is stored exactly rather than approximated through binary tick rounding.
FR2: The product can represent supported triplet groups as first-class score data rather than as renderer-only annotations.
FR3: The product can associate triplet groups with their member notes and rests while preserving each member's existing flow, track, and selection ownership.
FR4: The product can persist enough tuplet metadata to round-trip triplet grouping, display intent, and exact rhythmic meaning across save and reopen.
FR5: The product can support a persisted triplet representation that does not require a second schema reset when broader tuplet ratios are added later, unless architecture review proves that a triplet-specific schema is materially safer.
FR6: Composers can create supported triplet groups from a focused tuplet popover in the existing workflow without manual approximation of timing values.
FR7: Composers can select and edit individual triplet members while the product preserves valid triplet grouping where musically possible.
FR8: Composers can edit pitch, duration, and position of notes inside a triplet without the product silently converting the passage into approximate non-tuplet timing.
FR9: Composers can insert rests within supported triplet groups and preserve correct triplet duration accounting.
FR10: Composers can delete or slice triplet members and receive predictable group-preservation, repair, or dissolve behavior according to documented editing rules.
FR11: The product can expose triplet-aware popover creation and sequencer note-entry affordances that fit the existing primary note-entry workflow instead of introducing a disconnected rhythm-authoring mode.
FR12: The product can preserve the current authoritative ui.flow_id behavior while tuplets are edited in score and related workflow surfaces.
FR13: The product can derive written triplet notation from backend-owned rhythmic data without relying on frontend reconstruction.
FR14: The product can render triplet numbers and brackets where required for readability and can omit unnecessary bracket chrome where notation standards do not require it.
FR15: The product can beam supported triplet groups in ways that reinforce the tuplet grouping rather than contradict it.
FR16: The product can render supported triplet rests, ties, and chords with notation semantics consistent with the product's Gould-driven rules.
FR17: The product can handle triplets that begin or end adjacent to non-tuplet material without breaking surrounding spacing, hit targets, or selection behavior.
FR18: The product can keep triplet rendering legible within the existing native score surface, including number placement and collision avoidance sufficient for MVP correctness.
FR19: Composers can play back triplet-authored material and hear exact timing that matches the persisted rhythmic intent.
FR20: The product can resolve playback timing for tuplets from the same source-of-truth timing model used by notation and persistence.
FR21: Existing non-tuplet playback behavior remains correct after triplet support is introduced.
FR22: The product can preserve single-flow playback semantics for MVP even when tuplets exist in the active flow.
FR23: macOS users can save and reopen projects containing triplets without loss of exact timing, grouping semantics, or visible notation intent.
FR24: The product can migrate older projects safely if exact triplet support requires a persisted timing-lattice or score-schema change.
FR25: The product can distinguish between files that require structural upgrade and files that can remain semantically unchanged under the new model.
FR26: The product can keep .soloc header versioning, score.meta.app_version, and migration registration consistent with the repo's release-flow rules when tuplets alter persisted behavior.
FR27: Maintainers can implement any required pending migration in src-tauri/src/migrations/next.rs before release preparation, rather than embedding migration logic ad hoc in unrelated load paths.
FR28: Maintainers can validate supported triplet behavior through explicit parser, renderer, playback, persistence, and migration regression coverage.
FR29: Maintainers can validate that existing non-tuplet notation behavior remains unchanged unless explicitly revised by a separate notation initiative.
FR30: The product can document supported triplet scope, editing semantics, persistence impact, and migration expectations in developer-facing docs.
FR31: The product can update AI-agent and developer instructions when this work establishes durable new project rules for timing representation, persistence, or notation testing.
FR32: The product can introduce triplet support without moving score-owned rhythmic truth into frontend-only state.
FR33: The product can keep tuplets compatible with the current parser and renderer pipeline structure rather than requiring a parallel notation system.
FR34: The product can keep save/load, playback, and notation derivation behavior aligned under one exact timing model.
FR35: The product can preserve offline-first desktop behavior while tuplets are authored, rendered, saved, and reopened.

### NonFunctional Requirements

NFR1: Triplet support must not materially regress normal non-tuplet score editing or unchanged-score rendering performance.
NFR2: In representative triplet editing scenarios, local score updates should remain fast enough to feel immediate in normal desktop use, with performance characterization added where the timing-model change materially affects parser or playback work.
NFR3: Any broader timing-resolution change introduced for triplets must remain operationally practical across current project sizes and must not impose disproportionate overhead on non-tuplet projects.
NFR4: Supported triplet timing must be exact; the system must not depend on lossy rounding or hidden approximation.
NFR5: Non-tuplet projects must continue to load, save, and render correctly after tuplets ship.
NFR6: Tuplet save/load behavior must be deterministic and round-trip stable.
NFR7: Parser, rendering, playback, and persistence layers must not disagree about the rhythmic meaning of the same tuplet passage.
NFR8: Migration behavior must preserve score integrity and fail explicitly if an unsupported persisted state is encountered.
NFR9: The implementation must be structured so future non-triplet tuplets can extend the same timing and persisted-model foundation rather than bypass it.
NFR10: Regression coverage for tuplets must live close to the relevant parser, renderer, playback, and persistence modules rather than in a disconnected test harness that obscures ownership.
NFR11: Documentation of tuplets, persistence impact, and migration rules must be kept current with the shipped behavior.
NFR12: Core desktop document workflows for open, save, reopen, and recent-file behavior must remain intact for tuplet-authored files.
NFR13: Offline use must remain first-class; tuplets must introduce no network dependency into composition, playback, or file workflows.
NFR14: Tuplet authoring and editing affordances should remain consistent with the product's existing expert-focused UI model and should not require hidden mode knowledge to perform common triplet tasks.
NFR15: Hit targets, selection cues, and popover anchoring for triplet members and tuplet adornments must remain visually and behaviorally coherent in the native score surface.

### Additional Requirements

- Implement foundational tuplet support as a tuplet-container model that supports both child tone entries and continuous cross-boundary tones rather than a global timing-model rewrite.
- Add a new persisted Entry::Tuplet variant on instrument tracks with wrapper-owned rhythmic context: host tick, governed host span, local printed span, base note value, grouping metadata, and child entries where needed.
- Keep tuplets backend-owned in Rust and prohibit any frontend-owned parallel tuplet state model.
- Treat tuplets as host-track entries that govern a local metric window; new same-track note entry starting inside that window goes into the child frame through tuplet-aware entry rules, while continuous cross-boundary tones remain single persisted tones owned by where they start.
- Reconcile tuplet creation by adding the tuplet container, placing tones into host or child storage according to where they start, and deriving boundary splits only where notation or playback requires them.
- Treat boundary splitting as a derivation concern by default: crossing a tuplet edge does not itself require rewriting one persisted tone into multiple stored tones.
- Preserve written note values when structurally valid instead of proactively respelling them during reconciliation or derivation.
- Keep tuplet-local rests derived from absence of tones in the child frame rather than explicit persisted rest entries.
- Defer host-level directives inside MVP tuplet spans; tempo, clef, key signature, and time signature remain host-level concerns for this phase.
- Implement recursive local parse and flattening so parser, renderer, spacing, selection, and playback all project child content through the tuplet wrapper back into host musical time.
- Keep playback host-time based, projecting tuplet child onsets and durations into host time before scheduling so tempo remains a host-level concern.
- Extend rendering through the existing parse/draw pipeline rather than ad hoc overlay rendering, including tuplet number and bracket draw instructions, tuplet-local note projection, and tuplet hit targets.
- Use a widest-needed spacing strategy so tuplet-local layout can either expand host spacing or stretch to ambient host width, depending on context.
- Preserve tone_id as the authoritative tone identity and use tuplet-aware lookup context such as tuplet_path and local_tick only as selection and anchoring aids.
- Scope MVP deletion behavior so deleting a tuplet deletes its contained content; dissolve-back-to-host-time behavior is deferred.
- Add additive migration support in src-tauri/src/migrations/next.rs for tuplet serialization and child-entry support, with legacy non-tuplet files loading unchanged.
- Cover isolated eighth-note and quarter-note triplets, boundary splitting, hit testing, spacing expansion, playback projection, load/save round-trips, and legacy-file safety with explicit regression tests.
- Keep nested tuplets data-model-capable if practical, while still treating broader directive attachment, cross-barline support, and generalized child directives as post-MVP work.

### FR Coverage Map

FR1: Epic 1 - exact triplet timing stored in score data
FR2: Epic 1 - triplets as first-class score data
FR3: Epic 1 - member ownership preserved inside triplet groups
FR4: Epic 1 - persisted tuplet metadata round-trips
FR5: Epic 1 - future-aware persisted representation
FR6: Epic 1 - triplet creation from the write workflow
FR7: Epic 2 - editable individual triplet members
FR8: Epic 2 - edits do not silently collapse into approximate timing
FR9: Epic 2 - rests can be inserted within triplet groups
FR10: Epic 2 - predictable delete and slice behavior
FR11: Epic 1 - triplet-aware entry fits the existing workflow
FR12: Epic 2 - preserve authoritative ui.flow_id behavior during editing
FR13: Epic 3 - backend-owned notation derivation
FR14: Epic 3 - render correct tuplet numbers and brackets
FR15: Epic 3 - beam tuplets coherently
FR16: Epic 3 - render rests, ties, and chords correctly in tuplets
FR17: Epic 2 - handle adjacency to non-tuplet material without breaking interaction
FR18: Epic 3 - keep triplet rendering legible in the score surface
FR19: Epic 4 - exact playback of triplet-authored material
FR20: Epic 4 - playback resolves from the same source-of-truth timing model
FR21: Epic 4 - preserve existing non-tuplet playback behavior
FR22: Epic 4 - preserve single-flow playback semantics for MVP
FR23: Epic 1 - save and reopen triplet projects safely
FR24: Epic 1 - migrate older projects safely if schema changes
FR25: Epic 1 - distinguish structural-upgrade files from unchanged files
FR26: Epic 1 - keep .soloc versioning and app-version rules consistent
FR27: Epic 1 - implement pending migration in src-tauri/src/migrations/next.rs
FR28: Epic 5 - regression coverage across parser, renderer, playback, persistence, migration
FR29: Epic 5 - protect existing non-tuplet notation behavior
FR30: Epic 5 - document tuplet scope, editing semantics, and persistence impact
FR31: Epic 5 - update AI-agent and developer instructions
FR32: Epic 1 - preserve backend-owned truth and existing flow ownership model
FR33: Epic 3 - stay within the current parser and renderer pipeline shape
FR34: Epic 1 - align save/load, playback, and notation under one exact model
FR35: Epic 1 - preserve offline-first desktop behavior for tuplet workflows

### NFR Coverage Map

NFR1: Epic 2 and Epic 3 - preserve acceptable editing and rendering performance in tuplet-aware paths
NFR2: Epic 1 and Epic 2 - keep tuplet entry and edit updates immediate in normal desktop use
NFR3: Epic 1 and Epic 4 - keep the exact timing approach operationally practical across storage and playback projection
NFR4: Epic 1, Epic 2, and Epic 4 - preserve exact timing across authoring, editing, and playback
NFR5: Epic 3, Epic 4, and Epic 5 - protect existing non-tuplet render and playback behavior
NFR6: Epic 1 - guarantee deterministic tuplet save/load round-trips
NFR7: Epic 1, Epic 3, Epic 4, and Epic 5 - keep parser, rendering, playback, and persistence aligned
NFR8: Epic 1 - keep migration behavior explicit and safe
NFR9: Epic 1 - preserve a future-extensible tuplet foundation
NFR10: Epic 5 - keep regression coverage near owning modules
NFR11: Epic 5 - keep documentation and project instructions current with shipped behavior
NFR12: Epic 1 - preserve desktop open, save, and reopen workflows
NFR13: Epic 1 - preserve offline-first workflows
NFR14: Epic 1 and Epic 2 - preserve keyboard-operable, focus-stable, expert-oriented editing affordances
NFR15: Epic 1, Epic 2, and Epic 3 - preserve coherent hit targets, selection cues, and visual anchoring

## Epic List

### Epic 1: Author and Preserve Exact Triplets

Composers can create triplet passages in the normal write workflow and trust that the underlying score data, save/load behavior, and migration path preserve exact rhythmic meaning.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR11, FR23, FR24, FR25, FR26, FR27, FR32, FR34, FR35
**NFRs covered:** NFR2, NFR3, NFR4, NFR6, NFR7, NFR8, NFR9, NFR12, NFR13, NFR14, NFR15

### Epic 2: Edit Tuplets Without Breaking Musical Intent

Composers can select, modify, rest-fill, slice, and delete tuplet content predictably while the product preserves valid tuplet semantics and clear interaction ownership.
**FRs covered:** FR7, FR8, FR9, FR10, FR12, FR17

### Epic 3: Read Tuplets as Real Notation

Composers can see tuplets rendered as correct musical notation derived from backend-owned rhythmic data.
**FRs covered:** FR13, FR14, FR15, FR16, FR18, FR33

### Epic 4: Hear Tuplets Played Back Correctly

Composers can play tuplet-authored material and hear timing that matches the same exact backend-owned rhythmic intent.
**FRs covered:** FR19, FR20, FR21, FR22

### Epic 5: Ship Tuplet Support Safely in a Brownfield Codebase

Maintainers can harden and document foundational tuplet support with regression protection and architectural guardrails that keep existing notation behavior stable and leave room for future tuplets.
**FRs covered:** FR28, FR29, FR30, FR31

## Epic 1: Author and Preserve Exact Triplets

Composers can create triplet passages in the normal write workflow and trust that the underlying score data, save/load behavior, and migration path preserve exact rhythmic meaning.

### Story 1.1: Add Persisted Tuplet Container Entries

As a composer,
I want triplets represented as first-class score entries,
So that the score model can preserve exact tuplet structure instead of treating it as renderer-only decoration.

**FRs implemented:** FR2, FR4, FR5, FR32
**NFRs implemented:** NFR3, NFR4, NFR9

**Acceptance Criteria:**

**Given** the existing persisted instrument-track entry model
**When** foundational tuplet support is introduced
**Then** the score model supports a persisted Entry::Tuplet-style container entry with its own identity
**And** the container stores host start tick, governed host duration, local printed duration, base note value, grouping metadata, and child entries where tones start inside the tuplet span

**Given** a persisted tuplet container entry
**When** it is loaded into Rust-owned score state
**Then** the backend treats it as authoritative score data
**And** no frontend-owned parallel tuplet state is required to reconstruct its rhythmic meaning

**Given** future non-triplet tuplets are considered later
**When** this persisted shape is reviewed
**Then** the MVP representation is not hard-coded to a dead-end triplet-only schema
**And** the model retains a viable extension path unless a more general shape is proven materially less safe

### Story 1.2: Create Tuplet Containers Through a Tuplet Popover

As a composer,
I want to create a triplet from a focused popover in the existing workflow,
So that I can declare the tuplet structure before entering notes.

**FRs implemented:** FR1, FR3, FR6, FR11
**NFRs implemented:** NFR4, NFR14

**Implementation slices:**

- Trigger, placement, and backend container-creation handshake
- Ratio and base-note input behavior, including invalid-input handling
- Focus entry and focus-return behavior back to the sequencer context

**Acceptance Criteria:**

**Given** a supported note-entry context on an instrument track
**When** a composer opens the tuplet popover and confirms a supported triplet shape
**Then** the product creates a tuplet container for the targeted host-track span
**And** the container is inserted as backend-owned score data rather than a temporary UI annotation

**Given** the composer invokes tuplet creation from the primary note-entry workflow
**When** the MVP popover opens
**Then** the hotkey trigger is `;`
**And** the popover opens at the current selection tick

**Given** the MVP tuplet popover is shown
**When** the composer configures a supported triplet
**Then** the ratio is shown explicitly as `3:2`
**And** the main input is a single text-entry field for the ratio with a small inline base-note selector that defaults to eighth

**Given** the composer enters an invalid popover value
**When** the popover is submitted
**Then** the invalid value is ignored using the same pattern as other current popovers
**And** the product does not introduce a special tuplet-only invalid-input flow for MVP

**Given** the popover is opened from keyboard or score-attached interaction
**When** it becomes active
**Then** focus lands on the ratio input first
**And** closing the popover returns focus to the originating sequencer context or the newly created tuplet region

### Story 1.3: Reveal and Select the Created Tuplet Region in the Sequencer

As a composer,
I want a newly created triplet to appear immediately as a clear sequencer region,
So that the handoff from popover creation into note entry feels continuous.

**FRs implemented:** FR6, FR11
**NFRs implemented:** NFR14, NFR15

**Acceptance Criteria:**

**Given** triplet creation is confirmed from the popover
**When** the creation flow completes
**Then** the created tuplet span is immediately revealed as one visibly bounded region in the sequencer on the active track
**And** the interaction hands off directly into sequencer note entry rather than forcing the user to rediscover the new region manually

**Given** a tuplet has just been created
**When** the creation flow completes
**Then** the newly created tuplet container is selected immediately
**And** the selection behavior matches other existing creation patterns for score entries

**Given** the created tuplet span is revealed in the sequencer
**When** the user reads the new region
**Then** it includes a horizontal line spanning the consumed host ticks together with a compact `3:2` tuplet annotation
**And** the region is visually distinct from ordinary host-track space before any notes are entered

**Given** the tuplet annotation is shown in the sequencer
**When** the user works within the tuplet region
**Then** the `3:2` annotation remains centered over the region and sticky at the top of the sequencer view
**And** it does not use a solid background that obscures tones underneath it

**Given** the created tuplet span is visible in the sequencer
**When** the user inspects its internal timing structure
**Then** the vertical tick indicators inside the region appear slightly compressed to show tuplet warping
**And** the local triplet slots read as narrower than ordinary host-track spacing without destabilizing the broader sequencer layout

**Given** the created tuplet container has no entered notes yet
**When** it is first shown in the sequencer
**Then** the grouped tuplet region remains visible and targetable as empty content rather than collapsing away
**And** the empty local slots remain legible as available tuplet positions

### Story 1.4: Enter Tuplet Notes in the Sequencer

As a composer,
I want to add notes for a created triplet directly in the sequencer,
So that tuplets fit the same primary note-entry workflow I already use.

**FRs implemented:** FR1, FR3, FR11, FR12
**NFRs implemented:** NFR2, NFR4, NFR14

**Acceptance Criteria:**

**Given** a tuplet container has been created on the active track
**When** the composer enters notes in the sequencer within that tuplet span
**Then** those notes are written into the tuplet child event stream through the active tuplet multiplier rather than plain host-time entry rules
**And** their stored timing remains exact while the written value stays musically intelligible inside the tuplet frame

**Given** a sequencer view showing a track with a tuplet container
**When** the composer targets note entry inside the tuplet span
**Then** the sequencer exposes a tuplet-aware entry region that maps input to valid child positions
**And** the user does not need to calculate approximate binary timing values manually

**Given** a composer prepares a note inside the tuplet span
**When** the entry is committed
**Then** the system checks the active tuplet multiplier in the same way other contextual directives affect interpretation
**And** the resulting note is stored with exact backend timing rather than approximate binary rounding

**Given** tuplet notes are entered through the sequencer
**When** the resulting score state is inspected
**Then** flow, track, and tone ownership remain unambiguous
**And** the current authoritative ui.flow_id behavior remains intact

**Given** entered tuplet notes coexist with empty local space in the same region
**When** the sequencer refreshes after note entry
**Then** entered notes remain legible as members of the tuplet container
**And** any remaining empty local slots stay targetable for later entry

### Story 1.5: Create Tuplets Over Existing Material Without Losing Ownership

As a composer,
I want creating a tuplet over existing notes to preserve valid ownership and written intent,
So that the new tuplet structure does not scramble already-authored material.

**FRs implemented:** FR3
**NFRs implemented:** NFR4, NFR7

**Acceptance Criteria:**

**Given** ordinary same-track material exists in the consumed span
**When** tuplet creation is confirmed through the popover
**Then** the product records the tuplet span as a bounded rhythmic container over that host window
**And** any tone whose start lies inside that window is represented in the tuplet child entries while crossing tones remain single persisted tones owned by their start position

**Given** existing notes are structurally valid across or inside the tuplet frame
**When** reconciliation and tuplet-aware derivation evaluate them
**Then** their written values are preserved rather than proactively respelled
**And** only boundary-driven normalization is applied

**Given** notes cross into or sit inside a tuplet-governed span
**When** the updated score state is inspected
**Then** tones that start inside the tuplet live in child entries while a crossing tone remains one persisted tone owned by where it starts, with derived rests from absence of tones in local tuplet slots
**And** tone ownership and lookup remain compatible with existing flow, track, and selection expectations

**Given** a created tuplet contains empty local time after reconciliation or partial sequencer entry
**When** the score state is inspected after creation
**Then** the empty span remains represented by absence of tones rather than newly persisted rest entries
**And** the tuplet model stays consistent with the current event-stream mental model

### Story 1.6: Round-Trip Tuplet Data Through Save, Load, and Migration

As a desktop document user,
I want triplet-authored scores to save and reopen safely,
So that exact rhythmic meaning survives normal file workflows and older files remain safe.

**FRs implemented:** FR4, FR23, FR24, FR25, FR27, FR34
**NFRs implemented:** NFR6, NFR7, NFR8

**Implementation slices:**

- Tuplet serialization and deterministic save/load round-trip behavior
- Pending migration registration and upgraded load-path behavior
- Legacy-file pass-through and structural-upgrade differentiation

**Acceptance Criteria:**

**Given** a score containing supported tuplets entered through the popover and sequencer workflow
**When** the project is saved and reopened
**Then** the tuplet container, child entries where present, grouping semantics, continuous cross-boundary tones, and exact rhythmic intent round-trip without loss
**And** visible notation intent remains recoverable from the persisted data

**Given** foundational tuplet support changes persisted score shape or load/save semantics
**When** implementation lands
**Then** pending migration support is added in src-tauri/src/migrations/next.rs
**And** migration handling is not embedded ad hoc in unrelated load paths

**Given** a legacy non-tuplet file
**When** it is opened under the new model
**Then** it loads correctly without requiring a semantic tuplet upgrade
**And** files that truly need structural upgrade are distinguishable from files that can remain unchanged

### Story 1.7: Keep Versioning and Offline Desktop Behavior Intact for Tuplet Files

As a maintainer,
I want tuplet-enabled files to follow existing versioning and desktop workflow rules,
So that the feature ships without destabilizing Compose's document and offline guarantees.

**FRs implemented:** FR26, FR34, FR35
**NFRs implemented:** NFR12, NFR13

**Acceptance Criteria:**

**Given** tuplets alter persisted behavior
**When** tuplet-authored files are produced
**Then** .soloc header versioning, score.meta.app_version, and migration registration remain consistent with repo release-flow rules
**And** the implementation does not bypass the project's documented migration discipline

**Given** tuplet support is available
**When** a composer uses standard desktop open, save, and reopen workflows offline
**Then** the full tuplet authoring path of popover creation plus sequencer note entry works without network dependency
**And** offline-first desktop behavior remains unchanged for the supported MVP workflow

**Given** this epic is complete
**When** later editing and rendering epics build on it
**Then** they rely on one exact backend-owned timing model shared across save/load, notation derivation, and playback
**And** no separate frontend truth layer is introduced

## Epic 2: Edit Tuplets Without Breaking Musical Intent

Composers can select, modify, rest-fill, slice, and delete tuplet content predictably while the product preserves valid tuplet semantics and clear interaction ownership.

**NFRs covered:** NFR1, NFR2, NFR4, NFR14, NFR15

### Story 2.1: Target Tuplet Containers and Members Reliably

As a composer,
I want tuplet containers and tuplet notes to resolve to the correct target,
So that I can edit tuplets without ambiguity about what I selected.

**FRs implemented:** FR7, FR12, FR17
**NFRs implemented:** NFR14, NFR15

**Acceptance Criteria:**

**Given** a track containing a tuplet container and one or more tuplet notes
**When** the composer targets a tuplet note in the sequencer
**Then** the product selects the correct note inside the tuplet region
**And** selection does not fall through to surrounding host-track material

**Given** a visible tuplet container affordance
**When** the composer targets the container rather than a note inside the span
**Then** the product can distinguish container selection from member selection
**And** the resulting selection state is unambiguous

**Given** a tuplet region contains both notes and empty slots
**When** the composer hovers or selects within that region
**Then** the sequencer visually distinguishes container selection, note selection, and empty-slot targeting
**And** the user can tell whether they are acting on the tuplet boundary or on content inside it

**Given** the tuplet container itself is selected
**When** the sequencer updates its visual state
**Then** the bracket or region-level container treatment is highlighted in blue like other selected entries
**And** inner notes are not implicitly selected as part of container selection

**Given** a tuplet note is selected
**When** the sequencer updates its visual state
**Then** that note is highlighted in blue like any other selected note
**And** the container remains visually distinct from the note-selection state

**Given** an empty tuplet slot is targeted as rest content
**When** the selection state is shown
**Then** it uses the product's existing rest-selection behavior
**And** the product does not introduce a separate custom empty-slot selection style for MVP

**Given** container, note, and empty-slot states are shown together
**When** the sequencer communicates the current target
**Then** those states remain distinguishable through boundary, placement, or shape cues in addition to color
**And** the user can identify the active target without relying on blue highlight alone

**Given** tuplet selection changes on the active track
**When** the edit state updates
**Then** the authoritative ui.flow_id remains correct
**And** no separate frontend-owned active-flow state is introduced for tuplets

### Story 2.2: Edit Pitch Inside a Tuplet Without Changing Timing

As a composer,
I want to change pitches of notes inside a tuplet,
So that I can refine musical content without disturbing the tuplet rhythm.

**FRs implemented:** FR7, FR8, FR17
**NFRs implemented:** NFR4

**Acceptance Criteria:**

**Given** a valid tuplet with one or more notes
**When** the composer changes the pitch of a selected tuplet note
**Then** the note pitch is updated
**And** its local timing and tuplet membership remain unchanged

**Given** adjacent non-tuplet material surrounds the tuplet
**When** a pitch edit is applied inside the tuplet
**Then** no surrounding host-track ownership changes occur
**And** boundary behavior remains stable

### Story 2.3: Add and Remove Notes Within Tuplet Slots

As a composer,
I want to add notes into empty tuplet space and remove notes from occupied tuplet space,
So that I can shape real triplet patterns inside one rhythmic container.

**FRs implemented:** FR7, FR9, FR10
**NFRs implemented:** NFR4, NFR15

**Acceptance Criteria:**

**Given** a valid tuplet container with empty local space
**When** the composer enters a note into an available tuplet position
**Then** the note is stored through tuplet-aware exact timing on the authoritative track model
**And** the tuplet remains structurally valid

**Given** a tuplet note is removed by the composer
**When** the removal is applied
**Then** the sounding content is removed from that local span
**And** the remaining empty span is represented by absence of tones rather than an explicit persisted rest entry

**Given** a tuplet note has been removed from the sequencer
**When** the region refreshes
**Then** the vacated local slot remains visibly targetable inside the tuplet region
**And** the tuplet does not visually collapse into ordinary host-track space

**Given** a partially filled tuplet
**When** the score state is re-evaluated after note removal
**Then** the container remains intact
**And** the empty region is available for later note entry

### Story 2.4: Reposition or Resize Tuplet Notes Within Supported Bounds

As a composer,
I want to move or resize tuplet notes within the tuplet frame,
So that I can refine rhythmic shape without breaking exact tuplet semantics.

**FRs implemented:** FR8, FR10, FR17
**NFRs implemented:** NFR1, NFR2, NFR4, NFR15

**Acceptance Criteria:**

**Given** a tuplet note is edited for position or duration inside the tuplet
**When** the requested edit remains within the tuplet's supported local frame rules
**Then** the change is applied using exact local timing
**And** the result remains a valid tuplet state

**Given** a tone duration is dragged across a tuplet boundary in a supported MVP case
**When** the edit is committed
**Then** the product preserves one continuous persisted tone and derives the boundary split at render or playback time
**And** the user-facing gesture remains one continuous duration edit rather than a manual delete-and-recreate workflow

**Given** a tone's start position is dragged across the tuplet boundary
**When** the edit is committed
**Then** ownership moves to the context where the tone now starts
**And** the tone still remains exactly one stored event

**Given** an edit would create unsupported overlap or otherwise violate MVP rules
**When** the composer attempts that edit
**Then** the product rejects the unsupported outcome explicitly
**And** it does not silently convert the passage into approximate non-tuplet timing

**Given** the user attempts an unsupported tuplet edit in the sequencer
**When** the edit is rejected
**Then** the product follows the existing product rejection behavior without introducing tuplet-specific error messaging
**And** the tuplet region remains visually stable enough for the user to recover without losing orientation

**Given** a tuplet sits next to non-tuplet material
**When** a supported move or resize occurs near the boundary
**Then** ownership across the boundary remains explicit
**And** hit targeting and selection remain stable

**Given** a tone already exists before a tuplet is created and the composer later extends it across the tuplet area
**When** the resize completes
**Then** the resulting host and tuplet segments preserve one coherent musical intent across the boundary
**And** the tuplet's visible inner beats and boundary remain legible during and after the edit

### Story 2.5: Apply Explicit MVP Delete and Slice Rules for Tuplet Members and Containers

As a composer,
I want delete and slice behavior for tuplets to follow clear rules,
So that correction workflows are predictable.

**FRs implemented:** FR10
**NFRs implemented:** NFR4

**Acceptance Criteria:**

**Given** the composer deletes a tuplet member
**When** the action is applied
**Then** only that note is removed
**And** the tuplet container remains in place with the vacated span becoming empty local tuplet space

**Given** the composer deletes the tuplet container
**When** the action is confirmed
**Then** the tuplet and all notes authored inside that tuplet span are removed together
**And** MVP does not attempt dissolve-back-to-host-time behavior

**Given** the composer slices a tone that is fully inside a tuplet or crosses a tuplet boundary
**When** the slice action is applied in a supported MVP case
**Then** the result remains governed by the one-stored-event-per-tone ownership rule for the resulting tones after the edit
**And** any tuplet-boundary fragment needed for notation or playback remains a derived result rather than an extra persisted split introduced solely by the boundary

**Given** a delete action would otherwise require unsupported repair behavior
**When** the action is processed
**Then** the product applies the defined MVP rule above rather than inventing a new structural interpretation
**And** no overlapping host and child ownership remains afterward

## Epic 3: Read Tuplets as Real Notation

Composers can see tuplets rendered as correct musical notation derived from backend-owned rhythmic data.

**NFRs covered:** NFR1, NFR5, NFR7, NFR15

### Story 3.1: Parse Tuplet-Spanned Content Into Backend-Owned Notation

As a composer,
I want tuplet content to derive notation from the backend model,
So that the score reads from real rhythmic structure instead of frontend reconstruction.

**FRs implemented:** FR13, FR33
**NFRs implemented:** NFR5, NFR7

**Acceptance Criteria:**

**Given** a score containing supported tuplets
**When** notation derivation runs
**Then** the parser evaluates authoritative track content through the tuplet's local frame
**And** the resulting notation is produced from backend-owned rhythmic data

**Given** empty spans exist inside a tuplet-governed local frame
**When** notation is derived
**Then** rests are inferred from absence of projected sounding content in the governed local tuplet slots, including host-owned crossing tones and tuplet-child tones
**And** no frontend reconstruction layer is required to invent missing rhythmic structure

**Given** tuplets are introduced into the parse path
**When** existing non-tuplet passages are derived
**Then** current non-tuplet notation behavior remains unchanged
**And** the implementation stays within the current parser and renderer pipeline shape

### Story 3.2: Render Tuplet Brackets and Numbers Correctly

As an engraving-sensitive composer,
I want tuplets to display the right bracket and number treatment,
So that the score reads like real notation rather than placeholder annotation.

**FRs implemented:** FR14, FR18
**NFRs implemented:** NFR7, NFR15

**Acceptance Criteria:**

**Given** a derived tuplet notation result that requires visible tuplet marking
**When** the score is rendered
**Then** the tuplet number is drawn in the correct relationship to its grouped content
**And** bracket visibility follows the supported MVP readability rules

**Given** a supported case where bracket chrome is not required for clarity
**When** the score is rendered
**Then** the number remains legible
**And** unnecessary bracket rendering is omitted according to the product's supported default notation rules

**Given** tuplets are rendered beside surrounding notation
**When** the score surface refreshes
**Then** tuplet markings remain visually attached to the correct content
**And** they do not resolve as unrelated overlay annotations

### Story 3.3: Render Tuplet Notes, Rests, Ties, and Beams Coherently

As an engraving-sensitive composer,
I want tuplet note grouping to read correctly,
So that beams, rests, and ties reinforce the tuplet instead of contradicting it.

**FRs implemented:** FR15, FR16
**NFRs implemented:** NFR7

**Acceptance Criteria:**

**Given** supported tuplet note groupings
**When** notation is rendered
**Then** beam decisions reinforce the tuplet grouping
**And** the rendered result does not contradict the tuplet's rhythmic structure

**Given** tuplets contain rests or partially filled spans
**When** the score is rendered
**Then** rests appear in positions consistent with the derived tuplet structure
**And** the result remains legible within MVP engraving scope

**Given** a created tuplet container has no entered notes yet
**When** the score is rendered
**Then** the governed local span reads as empty rest-bearing tuplet content
**And** the bracket and number treatment remain coherent with the derived tuplet rests

**Given** a tie enters, exits, or passes through supported tuplet content
**When** the score is rendered
**Then** tie presentation remains musically coherent
**And** the rendering does not imply approximate binary timing

**Given** a tone crosses into or out of a tuplet boundary
**When** notation is rendered
**Then** the boundary split is represented with the same musical logic Compose uses around barlines and ties
**And** the tuplet remains readable as one coherent rhythmic unit

### Story 3.4: Lay Out Tuplets Legibly Within the Existing Score Surface

As a composer,
I want tuplets to fit cleanly into the score layout,
So that numbers, brackets, and notes stay readable in normal work.

**FRs implemented:** FR18
**NFRs implemented:** NFR1, NFR15

**Acceptance Criteria:**

**Given** a score containing supported tuplets
**When** spacing is calculated
**Then** tuplet-local layout can claim enough width for readable note spacing, brackets, and numbers
**And** surrounding score spacing expands when required by the derived notation

**Given** the ambient score context already provides sufficient width
**When** the tuplet is laid out
**Then** local note positions stretch coherently within the available span
**And** the notation remains aligned with the surrounding score surface

**Given** tuplets begin or end next to non-tuplet material
**When** layout and hit targets are refreshed
**Then** number placement, collision avoidance, and targeting remain legible and stable
**And** the adjacent material remains distinguishable

## Epic 4: Hear Tuplets Played Back Correctly

Composers can play tuplet-authored material and hear timing that matches the same exact backend-owned rhythmic intent.

**NFRs covered:** NFR3, NFR4, NFR5, NFR7

### Story 4.1: Project Tuplet Timing Into Host Musical Time

As a composer,
I want tuplet playback timing to come from the same exact model as notation and persistence,
So that playback reflects the real rhythmic structure I entered.

**FRs implemented:** FR19, FR20
**NFRs implemented:** NFR3, NFR4, NFR7

**Acceptance Criteria:**

**Given** a score containing supported tuplets
**When** playback scheduling is prepared
**Then** tuplet child onsets and durations are projected into host musical time from the tuplet container's local frame
**And** the projection preserves exact timing rather than binary-tick approximation

**Given** a tuplet contains multiple notes at different local positions
**When** their playback timing is resolved
**Then** each onset and duration is computed from the same backend-owned timing model used by save/load and notation derivation
**And** no separate playback-only timing interpretation is introduced

**Given** a tone extends into or out of a tuplet across a supported boundary
**When** playback timing is resolved
**Then** each owned segment is scheduled in the correct context with preserved continuity across the boundary
**And** the result remains rhythmically exact relative to surrounding non-tuplet material

### Story 4.2: Schedule Tuplet Playback Correctly Beside Non-Tuplet Material

As a composer,
I want tuplets to play in the right relationship to surrounding notes,
So that mixed passages sound rhythmically correct.

**FRs implemented:** FR19, FR20, FR21
**NFRs implemented:** NFR4, NFR7

**Acceptance Criteria:**

**Given** tuplet material appears before, after, or between ordinary non-tuplet notes
**When** transport ordering is resolved
**Then** tuplet events are scheduled in the correct host-time order relative to surrounding material
**And** adjacent non-tuplet playback remains rhythmically correct

**Given** a passage mixes tuplets and non-tuplet material on the same supported playback path
**When** playback runs
**Then** the tuplet timing does not drift relative to neighboring events
**And** the result matches the persisted rhythmic intent

### Story 4.3: Preserve Existing Non-Tuplet Playback Behavior

As a composer,
I want existing playback to keep working after tuplets are added,
So that the new timing support does not regress older scores.

**FRs implemented:** FR21
**NFRs implemented:** NFR5

**Acceptance Criteria:**

**Given** a score with no tuplets
**When** playback runs after foundational tuplet support is introduced
**Then** existing non-tuplet playback behavior remains correct
**And** the added tuplet timing path does not alter legacy rhythmic output

**Given** tuplets are added to the playback implementation
**When** the transport handles existing non-tuplet scenarios
**Then** it continues to use the correct current behavior for those passages
**And** the feature does not force unrelated playback changes

### Story 4.4: Keep MVP Playback Scoped to the Active Flow

As a composer,
I want tuplets to play back correctly in the active flow without expanding MVP playback scope,
So that the feature ships with the right boundaries.

**FRs implemented:** FR22
**NFRs implemented:** NFR5

**Acceptance Criteria:**

**Given** MVP playback remains single-flow oriented
**When** tuplets exist in the active flow
**Then** playback works correctly for that flow
**And** the feature does not require broader cross-flow playback changes

**Given** tuplets are implemented for active-flow playback
**When** the playback story is completed
**Then** the system honors the current single-flow playback boundary
**And** later playback expansion remains a separate concern

## Epic 5: Ship Tuplet Support Safely in a Brownfield Codebase

Maintainers can harden and document foundational tuplet support with regression protection and architectural guardrails that keep existing notation behavior stable and leave room for future tuplets.

**NFRs covered:** NFR5, NFR10, NFR11

### Story 5.1: Add Targeted Regression Coverage for Each Tuplet Seam

As a maintainer,
I want targeted regression coverage across parser, rendering, playback, persistence, and migration seams,
So that tuplet support is protected where it can actually fail.

**FRs implemented:** FR28
**NFRs implemented:** NFR10

**Acceptance Criteria:**

**Given** the implemented tuplet feature set across earlier epics
**When** regression coverage is added
**Then** the suite includes explicit tests for parser, rendering, playback, persistence, and migration behavior relevant to supported tuplets
**And** the coverage lives close to the owning modules rather than in a disconnected repo-wide framework

**Given** supported triplet cases and representative boundary cases
**When** the regression corpus is run
**Then** it exercises isolated triplets, partially filled tuplets, boundary splitting, projection-driven playback timing, and load/save round-trips
**And** failures identify the relevant ownership seam clearly enough to debug

### Story 5.2: Protect Existing Non-Tuplet Behavior From Tuplet Regressions

As a maintainer,
I want non-tuplet notation and playback behavior protected while tuplets are added,
So that the new feature does not silently degrade existing musical output.

**FRs implemented:** FR29
**NFRs implemented:** NFR5

**Acceptance Criteria:**

**Given** existing non-tuplet notation behavior relied on by the product
**When** tuplets are introduced into parser, renderer, and playback paths
**Then** regression checks cover representative non-tuplet scenarios
**And** unchanged behavior remains explicitly protected rather than assumed

**Given** future tuplet work expands beyond MVP
**When** new changes are made later
**Then** the regression baseline established here still distinguishes intended tuplet changes from accidental regressions in older notation behavior
**And** existing non-tuplet correctness remains a protected product contract

### Story 5.3: Document Tuplet Scope, Editing Rules, and Persistence Impact

As a maintainer,
I want the shipped tuplet behavior documented clearly,
So that future contributors understand the supported scope and do not reintroduce ambiguity.

**FRs implemented:** FR30
**NFRs implemented:** NFR11

**Acceptance Criteria:**

**Given** foundational tuplet support is implemented
**When** developer-facing documentation is updated
**Then** it describes supported tuplet scope, popover-plus-sequencer entry flow, editing rules, notation boundaries, playback boundaries, and persistence impact
**And** migration expectations are documented wherever persisted behavior changed

**Given** the MVP intentionally excludes some broader tuplet behaviors
**When** the documentation is reviewed
**Then** deferred areas such as broader ratios, advanced sequencer manipulation, or other post-MVP concerns are called out clearly
**And** the docs do not imply support beyond the implemented scope

### Story 5.4: Update Project Guardrails for Future Tuplet Work

As a maintainer,
I want AI-agent and developer instructions updated for the new tuplet architecture,
So that future work follows the same timing, persistence, and testing rules.

**FRs implemented:** FR31
**NFRs implemented:** NFR11

**Acceptance Criteria:**

**Given** foundational tuplet support establishes durable project rules
**When** project instructions are updated
**Then** they reflect the backend-owned timing model, migration discipline, and test-placement expectations introduced by this feature
**And** future contributors are guided away from frontend-owned parallel state or ad hoc persistence fixes

**Given** the repo already has explicit migration and documentation rules
**When** tuplet-specific guidance is added
**Then** it extends those rules consistently rather than inventing a conflicting process
**And** future tuplets work has a clear architectural baseline to build on
