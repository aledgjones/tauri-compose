---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-02b-vision
  - step-02c-executive-summary
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
  - step-12-complete
inputDocuments:
  - docs/index.md
  - docs/project-context.md
  - docs/architecture.md
  - docs/project-overview.md
  - docs/data-models.md
  - docs/component-inventory.md
  - docs/development-guide.md
  - _bmad-output/implementation-artifacts/archive/2026-03-multi-flow-score/notation-edge-cases-written-durations.md
  - _bmad-output/implementation-artifacts/archive/2026-03-multi-flow-score/tech-spec-compound-meter-written-duration-and-beaming-support.md
workflowType: "prd"
workflow: "create"
documentCounts:
  productBriefs: 0
  research: 0
  brainstorming: 0
  projectDocs: 9
classification:
  projectType: desktop_app
  domain: music_notation
  complexity: high
  projectContext: brownfield
lastEdited: "2026-03-12"
editHistory:
  - date: "2026-03-12"
    changes: "Created PRD for foundational tuplet support with triplet timing, rendering, editing, persistence, and migration requirements."
---

# Product Requirements Document - Compose Foundational Tuplet Support

**Author:** Aled
**Date:** 2026-03-12

## Executive Summary

Compose currently provides exact integer timing for standard note values and increasingly capable rule-driven notation, but it does not yet support tuplets. That gap is now structural rather than cosmetic. The current score model stores tone positions and durations on a fixed lattice of 32 ticks per quarter note, which is sufficient for binary subdivisions but cannot represent exact triplet timing without rounding. As a result, triplet authoring, rendering, playback, persistence, and editing cannot be added safely as a thin renderer feature. They require a deliberate brownfield notation initiative.

This PRD defines a foundational tuplet project centered on **triplets first** while forcing the architecture to stop pretending tuplets are a purely visual concern. The goal is to let composers create, edit, render, save, reopen, and play back triplet notation accurately in Compose while establishing durable data structures and persistence rules that do not need to be replaced when broader tuplet families are added later.

The product goal is not “draw a 3 above some notes.” The product goal is to make tuplets feel like real musical structure inside Compose's Rust-owned model: exact rhythmic timing, correct notation semantics, predictable editing behavior, reliable save/load round-tripping, and migration safety for an existing user base with `.soloc` files that were created before tuplet support existed.

### What Makes This Special

The important opportunity here is that Compose can add tuplets without collapsing into the common trap of separating playback timing, visible notation, and editing behavior into contradictory systems. Because the app already aims to derive notation from musical data rather than treating the score as hand-drawn graphics, foundational tuplet support can strengthen that philosophy if it is implemented cleanly.

This initiative is also a discipline test for the brownfield architecture. Tuplets touch the timing grid, parser logic, beaming, spacing, hit targets, selection semantics, playback timing, persistence, and migrations. If Compose can absorb that change while preserving existing notation trust and desktop document reliability, it materially raises confidence in the product's long-term notation roadmap.

## Project Classification

- Project Type: Desktop application
- Domain: Music notation and composition tooling
- Complexity: High
- Project Context: Brownfield existing system

## Success Criteria

### User Success

Composers can enter triplets in normal writing workflows without fighting the current binary timing model. They can see correct notation immediately, including appropriate tuplet numbers and brackets, edit individual members without unpredictable breakage, and trust that playback matches what they wrote.

Success is not merely that triplets can exist. Success is that triplets behave like ordinary first-class notation in Compose: selectable, editable, visually clear, and stable after save/reopen.

### Business Success

For this project, business success means product maturity and notation credibility. Tuplets are a threshold feature for serious score writing. Shipping a correct triplet foundation closes an obvious capability gap, improves trust with notation-sensitive users, and creates leverage for later rhythmic features such as broader tuplet ratios, nested tuplets, richer beaming logic, and interoperability work.

This project is also successful if it prevents a future rewrite. A good outcome is that Compose ends this initiative with an exact timing and persistence model that can extend beyond triplets without invalidating saved work or forcing a second data-model reset.

### Technical Success

The system must support exact triplet timing without lossy rounding and without introducing contradictory state between score data, notation derivation, playback timing, and persisted `.soloc` documents. Existing non-tuplet scores must continue to load and behave correctly.

Technical success also means preserving current architectural principles: Rust remains the source of truth, the renderer and parser consume backend-owned data, and migration handling remains explicit and versioned rather than hidden in ad hoc repair code.

### Measurable Outcomes

- Composers can create and edit triplet groups in the write workflow and see correct notation without manual workaround patterns.
- Triplet-authored scores round-trip through save and reopen without timing drift, missing metadata, or broken notation grouping.
- Playback of triplet-authored material is rhythmically exact relative to the product's timing model, with no rounding artifacts introduced by UI or persistence layers.
- Existing non-tuplet projects open without migration regressions or visible notation changes unrelated to this feature.
- Parser, renderer, playback, persistence, and migration regression coverage exists for the supported triplet corpus.
- The solution establishes data-model headroom for future non-triplet tuplets without requiring immediate delivery of those later ratios.

## Product Scope

### MVP - Minimum Viable Product

Deliver **foundational triplet support** across the Rust-owned score model, parser, renderer, editing flows, playback timing, persistence, and migrations. MVP must include exact timing representation, persisted triplet grouping semantics, triplet notation rendering, popover-based tuplet creation, sequencer-based note entry inside tuplets, correct round-trip save/load behavior, and safe migration planning for any persisted-data change required to make triplets exact.

The MVP is intentionally triplet-first, but the underlying persisted model must not hard-code a dead-end “triplets only” shape if a more general tuplet representation is practical without major extra complexity.

### Growth Features (Post-MVP)

Extend the same foundation to non-triplet tuplets such as quintuplets and septuplets, add nested tuplets where musically justified, support richer display overrides for bracket and number presentation, and broaden advanced tuplet authoring and manipulation workflows in the sequencer or other editing surfaces beyond the MVP triplet-entry path.

Growth work can also expand notation intelligence around tuplet-rest conventions, beam exceptions, cross-bar tuplets, and import/export semantics such as MusicXML.

### Vision (Future)

Compose should eventually treat tuplets as an ordinary part of musical structure rather than a special case. Users should be able to think in musical rhythm, with the product reliably encoding timing, notation, and playback across simple, compound, and irregular subdivisions. This triplet initiative is the first durable step toward that goal.

## User Journeys

### Journey 1: Primary Composer - Entering a Triplet Naturally

A composer is writing in a simple meter and needs an eighth-note triplet in the middle of otherwise ordinary notation. They expect to create the tuplet through a focused popover and then enter the notes through the sequencer, not by calculating approximations or constructing tied fake rhythms. The journey succeeds when the composer can invoke triplet creation, enter the notes in the primary note-entry surface, and immediately see a coherent tuplet number and grouping that reads correctly in context.

This journey implies exact triplet timing, a popover affordance for tuplet creation, sequencer-native note entry inside the created tuplet span, and notation rendering that feels native to the existing score surface.

### Journey 2: Primary Composer - Editing an Existing Triplet Without Breaking It

The composer later adjusts pitch, duration, or position inside a triplet group. They may delete one member, replace it with a rest, shorten one note, or extend another up to a valid group boundary. They expect Compose to preserve the triplet's musical meaning when possible, surface invalid states clearly when not possible, and never silently turn the passage into approximate binary timing.

This journey implies group-aware editing rules, exact selection and hit targets for tuplet members and tuplet adornments, and predictable repair or dissolve rules when an edit invalidates a triplet group.

### Journey 3: Engraving-Sensitive User - Trusting the Displayed Notation

An expert user reviews the score and expects the triplet to look like real notation rather than placeholder annotation. They care about whether the number is shown when needed, whether brackets appear in the right contexts, whether beams and rests reflect the tuplet grouping, and whether ties and spacing remain readable.

This journey succeeds when triplets are legible and consistent with the product's Gould-driven notation standards, without introducing awkward spacing or contradictory grouping cues.

### Journey 4: Desktop Document User - Saving and Reopening Tuplet Scores Safely

The user saves a project containing triplets, closes the app, and reopens the file later or on a newer build. They expect the tuplet passage to remain exact, visually identical in intent, and musically equivalent in playback. They should not need repair prompts for the normal migration path.

This journey implies persisted tuplet-aware data, exact round-trip serialization, and versioned migrations if the existing timing or score schema changes.

### Journey 5: Maintainer - Evolving Rhythm Support Without Breaking Old Files

A maintainer implements triplet support in a codebase that already contains parser scenario tests, renderer logic, playback timing, and a concrete `.soloc` migration policy. They need a clear requirements contract for where timing truth lives, how old files are upgraded, and what regression corpus protects existing simple-meter behavior.

This journey succeeds when tuplets can be implemented incrementally with explicit migration rules, scenario tests, and documentation updates rather than hidden assumptions.

## Domain-Specific Requirements

### Notation Standards and Musical Correctness

Tuplet behavior must be evaluated against established engraving practice, with Elaine Gould's _Behind Bars_ remaining the product's notation authority. The feature should not reduce tuplets to “arbitrary note groups with a label.” Tuplets affect grouping, beaming, rests, spacing, tie readability, and when bracket display is required for clarity.

Triplet support should therefore be treated as notation semantics, not mere display. The product must preserve the distinction between performed duration, written grouping, and visual adornment while keeping those layers coherent.

### Brownfield Timing Constraints

The current timing system uses integer `Tick` and `Ticks` values with 32 ticks per quarter note. That model cannot encode exact quarter-note or eighth-note triplets without rounding. This is the central brownfield fact for the initiative.

The project must therefore introduce an exact triplet-capable timing approach. The requirement is outcome-focused rather than prescriptive: Compose may adopt a higher-resolution lattice, a rational-duration representation, or another exact backend-owned model, but it must eliminate triplet rounding and keep persisted score timing exact.

### Data Model and Persistence Stakes

Current tone entries persist `tick` and `duration` only, with no tuplet grouping object in the score model. Foundational triplet support must define how a tuplet span is represented in persisted score data and how that representation relates to child tone entries, rests, parser derivation, rendering, and playback.

The product direction is that tuplet spans behave like bounded containers that may own child tone entries rather than acting as a second overlapping note layer. Host ticks keep their ordinary global meaning before, inside, and after the tuplet span; tuplets must not imply tracks of different effective lengths. In practical terms, the persisted model must let track-to-notation split and tie boundary-crossing material the same way it already does around barlines, while keeping the underlying musical duration semantics literal.

The intended rule is straightforward even if the mechanics are subtle: each tone is persisted as exactly one event, that event is owned by where the tone starts, tuplets persist container context plus child content for tones that start inside them, and the draw/playback path introduces any boundary-aware split needed to read that music correctly.

Implementation summary:

- Start position determines storage ownership.
- Host-track tones may extend through a tuplet without becoming multiple stored tones.
- Tuplet-owned tones may extend out of the tuplet without becoming multiple stored tones.
- Derived notation and playback may still split those tones at the tuplet boundary to read correctly.
- Literal tick counts keep their written-duration meaning on both sides of the boundary.

This project must explicitly assess whether the new persisted structure should be triplet-specific or a more general tuplet container. The default should be to avoid a throwaway triplet-only schema if a modestly more general tuplet representation can be introduced without destabilizing the codebase.

### Editing Semantics

Tuplet editing rules must be explicit. A triplet group should behave as one rhythmic container with individually editable members. When a user changes notes inside the group, the system must either preserve a valid tuplet state or resolve the invalidity in a predictable, documented way. Silent drift into approximate non-tuplet timing is unacceptable.

Tuplet entry and editing must also be context-aware. When the user prepares a note inside a tuplet span, the system must apply the active tuplet multiplier the same way other contextual musical directives, such as tempo, affect downstream interpretation. Users should choose written values musically; Compose should handle the tuplet-aware mapping to exact backend timing.

MVP must explicitly support tones that extend into or out of tuplet boundaries. That means boundary crossing is not a post-MVP refinement: the product must preserve one persisted tone event per tone, derive the visual or playback split at the tuplet edge, preserve the intended written values when structurally valid, and render or play the resulting continuity coherently.

Written values remain literal under that rule: an eighth worth of ticks remains an eighth worth of notation even when the remainder is being drawn inside a tuplet.

This also applies to direct sequencer manipulation. If a user drags a tone duration across a tuplet area, the gesture should remain continuous from the user's perspective while the product derives any needed host and tuplet fragments internally at draw or playback time. Ownership should change only if the tone's start position changes sides.

Editing semantics must also cover rests, ties into and out of tuplets, slice behavior, selection behavior, and whether deletion removes one member, replaces it with rest space, or dissolves the group based on context.

### Persistence and Migration Requirements

This initiative must treat migration analysis as first-class product scope. If triplet support changes persisted score timing, score schema, `.soloc` payload semantics, or load/save behavior, a pending migration must be added in `src-tauri/src/migrations/next.rs` during implementation. Old projects must continue to load correctly.

The migration plan must explicitly state whether existing documents are merely passed through unchanged, structurally upgraded to a new timing lattice, or normalized to a new tuplet-aware persisted shape on load.

### Domain Risks and Mitigations

The highest-risk failure mode is creating “visual triplets” with inexact timing. That would break trust across playback, editing, and persistence even if the score looks plausible. The mitigation is to require exact backend timing as a non-negotiable product requirement.

Another risk is introducing a persisted shape that only works for triplets and must be replaced for future tuplets. The mitigation is to require future-aware schema review during architecture work, even though MVP delivery remains triplet-focused.

A third risk is destabilizing existing notation behavior. The mitigation is scenario-based regression protection for non-tuplet notation and explicit migration tests for persisted data.

## Innovation & Novel Patterns

### Detected Innovation Areas

The innovation opportunity is not that tuplets exist. It is that Compose can present tuplets as a coherent semantic feature across notation, playback, and editing instead of leaving users to reconcile conflicting models themselves.

Compose can also differentiate itself by making tuplet editing understandable. In many notation tools, tuplets are possible but structurally opaque. A well-designed Compose implementation can make the tuplet container legible in both UI behavior and data model consequences.

### Market and Product Positioning

Tuplets are expected capability in serious notation software. Compose is not trying to make tuplets novel; it is trying to make them exact, trustworthy, and architecturally coherent in a native desktop environment. The value is therefore product credibility and future leverage, not novelty for its own sake.

### Validation Approach

Validation should focus on whether expert users can trust the feature. The core questions are whether triplets round-trip exactly, whether editing feels predictable, whether playback timing matches the written result, and whether the feature clearly raises the product's notation maturity.

## Desktop Application Specific Requirements

### Project-Type Overview

Compose is a macOS-first native desktop application with offline-first document workflows. Tuplet support must integrate into that context: local files, native rendering, native playback, and deterministic save/reopen behavior remain core expectations.

### Technical Architecture Considerations

The implementation must preserve the Rust-owned state model. Tuplet truth must live in backend-owned score and timing data, not in frontend-only reconstruction state. The renderer, UI, playback, and persistence layers must all consume the same tuplet-aware source of truth.

### System Integration

Tuplet-enabled files must continue to participate in standard open/save workflows. Any persisted-data upgrade required for tuplets must be invisible to ordinary users except where version compatibility genuinely requires messaging.

### Offline Capabilities

Triplet authoring, editing, rendering, playback, save, and reopen must all work with no network dependency.

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** exact triplet foundation before broad tuplet breadth

This project should deliver the smallest feature set that still solves tuplets correctly end to end. That means exact triplet timing, persisted tuplet semantics, popover-based tuplet creation, sequencer-based note entry in the primary workflow, correct notation derivation, playback alignment, and migration safety. It explicitly does not mean rushing to every ratio before the timing and persistence model are sound.

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**

- Primary composer entering a triplet naturally
- Primary composer editing an existing triplet without breaking it
- Engraving-sensitive user trusting the displayed notation
- Desktop document user saving and reopening tuplet scores safely
- Maintainer evolving rhythm support without breaking old files

**Must-Have Capabilities:**

- Exact backend timing representation for supported triplets
- Persisted triplet grouping semantics in score data and `.soloc` round-trips
- Triplet creation through a focused popover in the existing workflow
- Sequencer-based note entry and editing inside supported triplet groups
- Tuplet-aware parser, beaming, spacing, rests, ties, and rendering for supported triplet cases
- Playback timing that matches triplet-authored score data exactly
- Migration handling for any persisted-data or timing-lattice changes
- Documentation and regression coverage for notation, persistence, and migration behavior

**Explicitly Out of Scope for MVP:**

- Arbitrary non-triplet ratios such as quintuplets and septuplets
- Nested tuplets
- MusicXML tuplets import/export
- User-facing custom bracket/number engraving overrides beyond what is necessary for correct default notation
- A brand-new repo-wide notation test framework when parser-local support remains sufficient
- Reworking unrelated multi-voice or cross-flow architecture as part of this project

### Post-MVP Features

**Phase 2 (Post-MVP):**

- Additional tuplet ratios beyond triplets
- More advanced rest and beam exceptions within tuplets
- Advanced sequencer-native tuplet authoring and manipulation beyond the MVP popover-plus-entry path
- Display controls for ratio style, bracket visibility, and advanced engraving cases

**Phase 3 (Expansion):**

- Nested tuplets
- Import/export interoperability for tuplet semantics
- Richer tuplets in compound and more irregular metric contexts
- Tuplet-aware page-layout and print-polish refinements where needed

### Risk Mitigation Strategy

**Technical Risks:** The main technical risk is changing timing representation in a way that breaks save/load compatibility or creates mixed exact/inexact rhythm behavior. Mitigation requires explicit migration planning, exact timing requirements, and end-to-end regression coverage.

**Product Risks:** The main product risk is shipping triplets that technically exist but feel fragile or confusing to edit. Mitigation requires group-aware editing rules and a constrained MVP scope.

**Resource Risks:** If scope pressure appears, later tuplet ratios and advanced display overrides should be cut first. Exact triplet timing, persistence, migration, and reliable editing are the irreducible core.

## Functional Requirements

Requirement identifiers in the Functional Requirements and Non-Functional Requirements sections were normalized on 2026-03-12 to support downstream traceability.

### Timing Model and Tuplet Data Structures

- FR1: Composers can create triplet-authored passages whose performed timing is stored exactly rather than approximated through binary tick rounding.
- FR2: The product can represent supported triplet groups as first-class score data rather than as renderer-only annotations.
- FR3: The product can associate triplet groups with their member notes and rests while preserving each member's existing flow, track, and selection ownership.
- FR4: The product can persist enough tuplet metadata to round-trip triplet grouping, display intent, and exact rhythmic meaning across save and reopen.
- FR5: The product can support a persisted triplet representation that does not require a second schema reset when broader tuplet ratios are added later, unless architecture review proves that a triplet-specific schema is materially safer.

### Authoring and Editing Behavior

- FR6: Composers can create supported triplet groups from a focused tuplet popover in the existing workflow without manual approximation of timing values.
- FR7: Composers can select and edit individual triplet members while the product preserves valid triplet grouping where musically possible.
- FR8: Composers can edit pitch, duration, and position of notes inside a triplet without the product silently converting the passage into approximate non-tuplet timing.
- FR9: Composers can insert rests within supported triplet groups and preserve correct triplet duration accounting.
- FR10: Composers can delete or slice triplet members and receive predictable group-preservation, repair, or dissolve behavior according to documented editing rules.
- FR11: The product can expose triplet-aware popover creation and sequencer note-entry affordances that fit the existing primary note-entry workflow, including predictable keyboard focus behavior across popover open and return to the sequencer, instead of introducing a disconnected rhythm-authoring mode.
- FR12: The product can preserve the current authoritative `ui.flow_id` behavior while tuplets are edited in score and related workflow surfaces.

### Notation Semantics and Rendering

- FR13: The product can derive written triplet notation from backend-owned rhythmic data without relying on frontend reconstruction.
- FR14: The product can render triplet numbers and brackets where required for readability and can omit unnecessary bracket chrome where notation standards do not require it.
- FR15: The product can beam supported triplet groups in ways that reinforce the tuplet grouping rather than contradict it.
- FR16: The product can render supported triplet rests, ties, and chords with notation semantics consistent with the product's Gould-driven rules.
- FR17: The product can handle triplets that begin or end adjacent to non-tuplet material without breaking surrounding spacing, hit targets, or selection behavior.
- FR18: The product can keep triplet rendering legible within the existing native score surface, including number placement and collision avoidance sufficient for MVP correctness.

### Playback and Sequencing Integrity

- FR19: Composers can play back triplet-authored material and hear exact timing that matches the persisted rhythmic intent.
- FR20: The product can resolve playback timing for tuplets from the same source-of-truth timing model used by notation and persistence.
- FR21: Existing non-tuplet playback behavior remains correct after triplet support is introduced.
- FR22: The product can preserve single-flow playback semantics for MVP even when tuplets exist in the active flow.

### Persistence, Load/Save, and Migration

- FR23: macOS users can save and reopen projects containing triplets without loss of exact timing, grouping semantics, or visible notation intent.
- FR24: The product can migrate older projects safely if exact triplet support requires a persisted timing-lattice or score-schema change.
- FR25: The product can distinguish between files that require structural upgrade and files that can remain semantically unchanged under the new model.
- FR26: The product can keep `.soloc` header versioning, `score.meta.app_version`, and migration registration consistent with the repo's release-flow rules when tuplets alter persisted behavior.
- FR27: Maintainers can implement any required pending migration in `src-tauri/src/migrations/next.rs` before release preparation, rather than embedding migration logic ad hoc in unrelated load paths.

### Regression Safety and Documentation

- FR28: Maintainers can validate supported triplet behavior through explicit parser, renderer, playback, persistence, and migration regression coverage.
- FR29: Maintainers can validate that existing non-tuplet notation behavior remains unchanged unless explicitly revised by a separate notation initiative.
- FR30: The product can document supported triplet scope, editing semantics, persistence impact, and migration expectations in developer-facing docs.
- FR31: The product can update AI-agent and developer instructions when this work establishes durable new project rules for timing representation, persistence, or notation testing.

### Brownfield Architectural Boundaries

- FR32: The product can introduce triplet support without moving score-owned rhythmic truth into frontend-only state.
- FR33: The product can keep tuplets compatible with the current parser and renderer pipeline structure rather than requiring a parallel notation system.
- FR34: The product can keep save/load, playback, and notation derivation behavior aligned under one exact timing model.
- FR35: The product can preserve offline-first desktop behavior while tuplets are authored, rendered, saved, and reopened.

## Non-Functional Requirements

### Performance

- NFR1: Triplet support must not materially regress normal non-tuplet score editing or unchanged-score rendering performance.
- NFR2: In representative triplet editing scenarios, local score updates should remain fast enough to feel immediate in normal desktop use, with performance characterization added where the timing-model change materially affects parser or playback work.
- NFR3: Any broader timing-resolution change introduced for triplets must remain operationally practical across current project sizes and must not impose disproportionate overhead on non-tuplet projects.

### Reliability and Correctness

- NFR4: Supported triplet timing must be exact; the system must not depend on lossy rounding or hidden approximation.
- NFR5: Non-tuplet projects must continue to load, save, and render correctly after tuplets ship.
- NFR6: Tuplet save/load behavior must be deterministic and round-trip stable.
- NFR7: Parser, rendering, playback, and persistence layers must not disagree about the rhythmic meaning of the same tuplet passage.
- NFR8: Migration behavior must preserve score integrity and fail explicitly if an unsupported persisted state is encountered.

### Maintainability

- NFR9: The implementation must be structured so future non-triplet tuplets can extend the same timing and persisted-model foundation rather than bypass it.
- NFR10: Regression coverage for tuplets must live close to the relevant parser, renderer, playback, and persistence modules rather than in a disconnected test harness that obscures ownership.
- NFR11: Documentation of tuplets, persistence impact, and migration rules must be kept current with the shipped behavior.

### Desktop Integration Quality

- NFR12: Core desktop document workflows for open, save, reopen, and recent-file behavior must remain intact for tuplet-authored files.
- NFR13: Offline use must remain first-class; tuplets must introduce no network dependency into composition, playback, or file workflows.

### Accessibility and UX Clarity

- NFR14: Tuplet authoring and editing affordances should remain consistent with the product's existing expert-focused UI model and should not require hidden mode knowledge to perform common triplet tasks. Tuplet popover and sequencer flows must remain keyboard-operable, return focus predictably after popover use, and communicate key target states without relying only on color.
- NFR15: Hit targets, selection cues, and popover anchoring for triplet members and tuplet adornments must remain visually and behaviorally coherent in the native score surface.

## Architecture Constraints

These are implementation principles rather than measurable quality attributes. They provide context for architecture and implementation planning.

- AC1: Exact tuplet timing must be owned by Rust and shared across notation derivation, playback, and persistence.
- AC2: The implementation must not introduce a frontend-owned parallel tuplet state model.
- AC3: If persisted score shape changes, migration work must be implemented in `src-tauri/src/migrations/next.rs` before any release preparation activity.
- AC4: The project should prefer the smallest exact timing-model change that solves triplets correctly while preserving a viable path to broader tuplets.
- AC5: Parser-local written-duration support and scenario coverage should stay near `get_written_durations.rs` and existing parser test-support utilities unless a clearly justified broader framework becomes necessary.
- AC6: Tuplet rendering should extend the existing parse/draw pipeline rather than bypassing it with ad hoc overlay rendering.
- AC7: Save/load semantics, `.soloc` schema expectations, and migration behavior must be documented whenever tuplets change persisted behavior.
- AC8: Existing non-tuplet notation rules are protected product behavior and must not be casually rewritten as part of this initiative.
