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
  - docs/source-tree-analysis.md
  - _bmad-output/planning-artifacts/research/technical-testing-and-tooling-research-2026-03-04.md
workflowType: "prd"
workflow: "edit"
documentCounts:
  productBriefs: 0
  research: 1
  brainstorming: 0
  projectDocs: 8
classification:
  projectType: desktop_app
  domain: general
  complexity: medium
  projectContext: brownfield
lastEdited: "2026-03-11"
editHistory:
  - date: "2026-03-11"
    changes: "Normalized FR/NFR identifiers to remove numbering gaps and added editorial traceability note."
---

# Product Requirements Document - compose

**Author:** Aled
**Date:** 2026-03-10

## Executive Summary

Compose is a native desktop music composition application focused on performant, accurate score writing with sensible default notation behavior. Its core goal is to let users enter musical ideas and see correctly written, visually strong score output appear with minimal manual correction. Rather than forcing users to explicitly manage written note values and engraving cleanup as a primary workflow, Compose aims to encode notation intelligence directly into the authoring experience so the score reads correctly as it is created.

Compose is a serious brownfield system with an existing Rust-owned state model, native Skia/Metal rendering pipeline, and rule-driven notation logic. The next stage of product maturity is to move beyond the current intentionally simplified single-flow presentation model toward a more professional score view. Multi-flow rendering is the first step in that transition, enabling broader score continuity and preparing the architecture for future page-based presentation.

### What Makes This Special

What makes Compose distinctive is the combination of musical rule-awareness and native performance. The product is not trying to be a generic editor where users enter data and then repair notation afterward. Its value comes from producing musically sensible notation by default, reducing the amount of cleanup and explicit engraving intervention required from the user.

The core insight is that users should be able to watch well-formatted music emerge as they input notes, rather than treating notation quality as a separate editing phase. That makes the experience feel both faster and more musically coherent. Compose is differentiated by aiming for accurate written output, beautiful score presentation, and responsive native rendering within a system complex enough to test serious product and engineering ideas.

## Project Classification

- Project Type: Desktop application
- Domain: General creative software / music composition tooling
- Complexity: Medium
- Project Context: Brownfield existing system

## Success Criteria

### User Success

Users can compose faster because they spend less time correcting notation after note entry. Multi-flow rendering makes score structure easier to read at a glance, reduces the need to switch context between flows, and makes the score feel more like a professional document rather than a set of isolated rendered segments.

For this feature specifically, success means users can move across multiple flows naturally, keep their orientation, and continue editing without friction. Horizontal scrolling should remain effortless, and interaction affordances such as hit targets and popovers must feel continuous across flows rather than scoped awkwardly to a single rendered unit.

### Business Success

For this project, business success is best interpreted as product maturity and project leverage rather than commercial metrics. This feature is successful if it materially advances Compose from a simplified internal rendering model toward a more professional score presentation model. It should establish a foundation for future page layout, frame-based music flow, and pagination work without requiring a later architectural reset.

This work is also successful if it proves the application's architecture at a more demanding level. A positive outcome is that the renderer, instruction model, and interaction model can scale to multi-flow presentation while preserving correctness, responsiveness, and maintainability.

### Technical Success

Native rendering must remain performant and coherent as multiple flows are displayed in a single composed presentation. Editing one flow must not force unrelated flows to recompute written notation or spacing. The instruction/render split should continue to isolate recalculation so performance scales sensibly as scores grow.

Interaction behavior must remain correct across flows. Hit targets and popovers must continue to resolve accurately, and active-flow semantics for editing and playback must remain logical and well-scoped. Existing single-flow workflows must not suffer major regressions.

The most important technical constraint is that this feature must not change or destabilize written-notation logic. In particular, the existing written-duration and notation-derivation behavioral contract must remain intact. Multi-flow rendering is a presentation and orchestration change, not a notation rewrite.

### Measurable Outcomes

- Users can view and interact with multiple flows in one continuous rendered sequence without needing the current flow-switching pattern for basic reading and editing.
- Horizontal scrolling remains smooth and continuous across the combined flow presentation.
- Hit targets function correctly across all rendered flows.
- Popovers resolve correctly across all rendered flows.
- Editing one flow does not trigger unnecessary notation or spacing recomputation for other flows.
- Existing written-notation behavior remains unchanged, with regression coverage protecting the current parser logic.
- Rendering remains responsive enough that the feature feels native and immediate, with growth targets focused on completion in only a handful of milliseconds even for larger scores.

## Product Scope

### MVP - Minimum Viable Product

Render multiple flows horizontally in sequence with a small gap between them while preserving accurate notation and existing interaction quality. Horizontal scrolling should continue to work naturally across the combined presentation. Hit targets and popovers must function across all rendered flows.

The architecture must preserve isolated recalculation so editing one flow does not force unrelated flows through notation and spacing recomputation. The feature must not alter written-notation logic.

Playback may remain scoped to individual flows at MVP, provided the UI remains coherent.

### Growth Features (Post-MVP)

Improve performance further so rendering completes within only a handful of milliseconds even for larger scores. Extend playback behavior beyond single-flow scoping so playback can move across multiple flows coherently. If needed, add separate flow selection controls in sequence-oriented contexts where active-flow behavior still matters.

### Vision (Future)

Evolve multi-flow rendering into a full music-document layout pipeline that supports frame-based rendering across pages. Music should flow between music frames in a way analogous to sophisticated word processing, but for notation. Pagination should become straightforward within the rendering model rather than bolted on afterward.

System spacing should eventually scale to fit readable music into systems based on calculated density, producing page layouts that balance musical clarity, visual quality, and available space.

## User Journeys

### Journey 1: Primary Composer - Continuous Composition Flow

A skilled composer opens Compose to work on a piece with multiple flows or sections and expects to think musically rather than administratively. They enter notes with the assumption that the software understands notation rules well enough to present the music correctly without repeated cleanup work. As the score grows, they want to move through the material as one coherent musical document, not as a set of disconnected rendering islands.

The journey succeeds when multiple flows render in sequence with accurate notation, the score remains easy to follow visually, and the composer can scroll horizontally through the music without needing to stop and switch context. The emotional payoff is that the score feels alive and professionally structured while they work, so their attention stays on composition rather than repair.

This journey implies capabilities for continuous multi-flow presentation, accurate notation preservation, smooth navigation, and interaction continuity across flow boundaries.

### Journey 2: Primary Composer - Performance and Interaction Confidence

The same composer works on a larger score and edits one flow while other flows are already visible in the same continuous rendering. They expect the system to feel stable and immediate. They do not want visible stutter, unnecessary recomputation, or a sense that unrelated parts of the score are being disturbed by local edits.

The critical moment is when the composer edits one flow and the product responds cleanly while preserving the rest of the visible score. The user does not necessarily know how the architecture works, but they feel the result as confidence: the software remains responsive, scrolling remains natural, hit targets still resolve correctly, and popovers continue to work across all visible flows.

This journey implies capabilities for isolated recalculation, stable multi-flow rendering performance, and correct interaction behavior across the combined score surface.

### Journey 3: Engraving-Sensitive Expert User - Readability and Professional Trust

An expert user with strong notation and engraving expectations reviews the score not just as raw musical content, but as a document that should read professionally. They are sensitive to whether the score structure feels coherent and whether presentation choices support fast reading. They do not want the software to flatten everything into generic output or sacrifice formatting quality for convenience.

The journey succeeds when the score feels like a serious music document, even before full page layout exists. Rendering multiple flows in sequence is valuable here because it begins to establish continuity, hierarchy, and document-like professionalism. The user sees that the software is moving toward a real engraving pipeline rather than remaining a set of isolated technical demos.

This journey implies capabilities for visually coherent multi-flow presentation and a roadmap that clearly supports later page layout, frame-based flow, and professional engraving outcomes.

### Journey 4: Maintainer / Debugging User - Safe Evolution of the Rendering Pipeline

The maintainer opens Compose and its debugger tools while developing complex rendering changes. They need to improve the renderer and layout model without breaking the notation logic that has already taken significant effort to build. They rely on internal tools, targeted tests, fixtures, and performance visibility to validate that architectural changes have not damaged output quality or responsiveness.

The journey succeeds when the maintainer can evolve rendering orchestration, measure performance, and inspect behavior while trusting that the written-notation layer remains protected. The emotional payoff is confidence that the product can become more sophisticated without destabilizing the hard-won musical correctness already present in the system.

This journey implies capabilities for regression safety around written-notation logic, performance monitoring, debugger support, and a clean enough architecture to let rendering composition evolve independently of notation parsing.

### Journey 5: Future Technical / Integration User - External Control and Automation

In a later phase, an advanced technical user or external system wants to interact with Compose programmatically, potentially through an API or MCP-driven workflow. Their interest is not direct notation editing through the UI alone, but controlling or interrogating the application as part of a broader toolchain.

This journey is not part of MVP, but it informs long-term architectural direction. It succeeds when the product's internal boundaries are coherent enough that external interaction can be added without reworking the core rendering and score model from scratch.

This journey implies future capabilities around API surface design, state-safe automation, and system boundaries that remain comprehensible as the product grows.

### Journey Requirements Summary

These journeys reveal a clear set of capability requirements:

- Multi-flow rendering must support continuous score reading for advanced composers.
- Interaction behavior must remain seamless across flow boundaries.
- Local edits must not cause unnecessary recomputation or visible instability elsewhere.
- The rendering pipeline must evolve toward document-quality score presentation without changing notation logic.
- Regression safety for written-notation behavior is a first-class product requirement, not just an implementation detail.
- Internal debugging and performance visibility are important enabling tools for safe product evolution.
- Future external integration is a valid long-term direction, but not part of MVP scope.

## Domain-Specific Requirements

### Notation Standards and Musical Correctness

Compose operates in a domain where trust is built on notation correctness, not just feature breadth. The governing notation reference for expected behavior is Elaine Gould's Behind Bars. Product decisions that affect written notation, spacing logic, rest handling, grouping, or score readability should be evaluated against established engraving practice rather than convenience alone.

This creates an important product requirement: notation behavior cannot be reduced to a simplistic one-size-fits-all algorithm where the musical context demands exceptions or more nuanced rules. The system should continue to prefer musically sensible output by default, even when that requires specialized handling in particular rhythmic or metric contexts.

### Product-Specific Technical Constraints

The current feature work must preserve existing written-duration logic and the trust it represents. Multi-flow rendering is a rendering and interaction orchestration change, not a rewrite of notation semantics. In particular, the existing written-duration behavioral contract must remain unchanged unless a separate, explicitly justified notation initiative is undertaken.

The architecture should continue moving toward a future page-layout and music-frame model. That means solutions should avoid coupling flow rendering too tightly to single-flow assumptions in ways that would block later pagination, frame assignment, or density-aware system layout.

### Interoperability and Format Constraints

The existing .soloc format is acceptable for the current phase of the product and does not need to be displaced as part of this initiative. MusicXML is a future interoperability opportunity rather than a current requirement, so this PRD should not assume import/export scope beyond preserving reasonable architectural openness for later work.

### Domain Risks and Mitigations

The most important domain risk is breaking user trust in notation quality. If multi-flow rendering introduces layout or interaction changes that make notation feel less logical, less readable, or less professionally presented, the feature would harm the product even if it is technically functional.

Another major risk is architectural coupling that solves today's presentation problem in a way that blocks tomorrow's page-layout work. The mitigation is to treat this feature as a progression toward document-level score presentation, with clear separation between notation derivation, flow composition, interaction mapping, and future frame-based layout concerns.

Finally, there is a presentation risk: if the resulting score feels less polished or less coherent than the current single-flow view, expert users may interpret the feature as a regression in professionalism. Validation should therefore include both interaction correctness and visual-readability review against the product's engraving goals.

## Innovation & Novel Patterns

### Detected Innovation Areas

The main innovation opportunity is not inventing a new music-notation category. It is creating a more rigorous interaction model for a notoriously complex domain. Compose can be differentiated by making advanced notation workflows feel meticulously logical, concise, and predictable for expert users rather than powerful-but-muddled.

A meaningful innovation signal is the attempt to express complex musical ideas through a UI that stays simple without becoming simplistic. That means the product should avoid hiding domain depth, but it should also avoid scattering related rules across inconsistent panels, modes, or mental models. The product can be novel in how clearly it maps high-level musical intent to the underlying data model, written score, and playback behaviors.

A second innovation area is the product philosophy inherited and refined from Dorico's most important conceptual move: treating music as musical events, with notation and playback derived from that model. The opportunity for Compose is not merely copying that architecture, but making its consequences clearer and more usable. If users can reliably understand where a concept lives, why a setting belongs where it does, and how a musical action affects visible notation versus playback, that would be a genuine UX innovation in this space.

### Market Context & Competitive Landscape

Existing professional notation tools already prove that advanced score software can be built around deeper semantic models rather than raw graphical editing alone. Dorico is the clearest reference point here. Its model is powerful, but its UX and information architecture can feel muddled, especially around settings, engraving rules, and score setup.

That creates a realistic differentiation path. Compose does not need to claim a brand-new conceptual foundation. Instead, it can compete on clarity of expression: making the semantic model easier to navigate, making configuration more discoverable, and making advanced behavior feel inevitable rather than hidden. The innovation is a better translation layer between expert musical thinking and software structure.

### Validation Approach

This innovation should be validated through expert-UX questions rather than market-novelty claims. The key question is whether advanced users can more quickly predict where to go, what to change, and why the software behaves as it does.

Validation signals would include:

- expert users can locate notation, engraving, setup, and playback controls with less hesitation
- users can explain the product's mental model in a way that matches the implementation
- complex editing tasks require fewer corrective detours
- the distinction between musical data, written notation, and playback behavior feels coherent rather than arbitrary
- advanced users perceive the UI as simpler because it is better organized, not because capability was removed

### Risk Mitigation

The main risk is claiming UX innovation while only restyling a familiar system. If the underlying information architecture remains inconsistent, the product would inherit the same confusion it aims to fix. The mitigation is to define explicit placement rules for settings and behaviors: what belongs to musical structure, what belongs to notation output, what belongs to playback, and what belongs to document/layout concerns.

Another risk is oversimplifying for the sake of clarity and thereby frustrating expert users. The mitigation is to optimize for expert logic, not novice minimalism. The UI should be concise, but it must remain domain-faithful and capable.

A final risk is architectural drift. If the event model, notation derivation, and UI surface are not aligned, the interface will eventually become muddled regardless of intent. The mitigation is to preserve strong semantic boundaries in both the data model and the user-facing organization.

## Desktop Application Specific Requirements

### Project-Type Overview

Compose is a native desktop composition application with macOS as the only current target platform. This is a deliberate product and engineering choice rather than an accidental limitation: simplify around one platform first while avoiding unnecessary architectural decisions that would permanently block future backend portability.

The application is expected to feel like a serious native desktop tool rather than a web-style shell. That means desktop-native interaction quality, predictable local file handling, and responsive rendering/performance are core expectations rather than implementation details.

### Technical Architecture Considerations

The product should be designed and validated as a macOS-first application. Native rendering, local file workflows, and audio device integration are first-class concerns in both architecture and testing. Although only macOS is in scope now, the renderer and surrounding architecture should avoid needless platform lock-in where reasonable, especially where backend abstraction already exists.

Because the application is offline-first, core composition, rendering, editing, playback, and file opening/saving must work without any network dependency. Network-based services such as cloud storage integration may be considered later, but they must remain optional extensions rather than foundational assumptions.

Desktop integration should favor native platform behavior wherever it improves clarity and trust. Native file pickers, file open/save conventions, and desktop file association support are important parts of making the product feel coherent and professional on macOS.

### Platform Support

macOS is the only active target platform for this phase of the product. The team should optimize for a strong single-platform experience instead of diluting effort across multiple operating systems prematurely.

At the same time, implementation decisions should avoid unnecessary hard-coding that would make future portability substantially harder than it needs to be. The guiding principle is macOS-first execution without careless architectural dead ends.

### System Integration

Native file handling is a priority. Compose should support standard macOS file open/save workflows and should aim for file association support so score files can be opened directly in the application from the desktop environment.

Native dialogs and platform-appropriate file interaction patterns should be preferred over generic abstractions where possible. The product should feel consistent with user expectations for desktop document software.

Audio device integration is a critical requirement because playback is a core part of the product experience. The playback system should integrate cleanly with desktop audio devices and preserve predictable, low-friction device behavior. MIDI input from external devices is a valid future direction, particularly for recording workflows, but it is not required for MVP.

### Update Strategy

In-application auto-update behavior is not required for the current phase and should be treated as a later enhancement rather than MVP scope. This work should not depend on update infrastructure being present.

However, the product already has an important forward-compatibility concept through migrations that allow newer versions to open older files safely. That migration capability is strategically important and should continue to be treated as part of the desktop product's long-term maintainability and user trust model.

### Offline Capabilities

Compose should be treated as fully offline-first. Users must be able to create, edit, render, save, reopen, and play back scores without internet connectivity.

Potential future integrations with services such as Dropbox or Google Drive may be considered later, but they should remain low-priority extensions. A future account-based sync model for scores, settings, and template files is also a legitimate long-term product direction, especially for multi-device continuity and environment portability. However, these capabilities must remain optional extensions. The product's core value proposition must not depend on network access, account systems, or cloud synchronization.

### Implementation Considerations

The desktop implementation should optimize for native performance, local reliability, and document-style workflows. Feature prioritization should favor areas that strengthen the macOS-native experience, especially file handling, playback-device stability, and the overall feeling of a professional offline composition environment.

This also implies disciplined scope control: avoid premature cross-platform abstraction, avoid unnecessary network assumptions, and focus engineering effort on the platform integrations that materially improve the experience for the current target user.

Where practical, score files, settings, and templates should be structured so they can later participate in optional sync and portability workflows without requiring the MVP architecture to adopt account or cloud dependencies upfront.

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Architecture-first MVP with direct user value

This feature slice should be treated primarily as an architectural proof that also delivers meaningful user-facing value. The goal is to validate that Compose can move beyond a single-flow rendering model without compromising notation correctness, interaction quality, or future page-layout ambitions.

The MVP is successful if multiple flows render in one continuous score surface and users can interact with them naturally through selection and popovers. That is enough to prove the core architectural direction while also making the score meaningfully more useful and document-like.

**Resource Requirements:** Focused implementation led by strong architectural ownership

This work requires careful ownership of the rendering and interaction architecture more than sheer feature volume. The essential capability is deep familiarity with the current instruction/render pipeline, hit-testing model, and state boundaries, plus enough product judgment to avoid short-term shortcuts that would undermine later layout work.

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**

- Primary composer continuous composition flow
- Primary composer performance and interaction confidence
- Engraving-sensitive expert user, at least to the extent of establishing document-like multi-flow continuity
- Maintainer/debugging user through safe architectural evolution and regression protection

**Must-Have Capabilities:**

- Render multiple flows horizontally in one continuous score view
- Preserve accurate notation and existing written-duration behavior
- Support interaction across flows, including selection and popovers
- Preserve smooth horizontal scrolling across the combined rendered output
- Maintain isolated recalculation so editing one flow does not force unnecessary recomputation elsewhere
- Keep the implementation aligned with future page-layout and frame-based rendering goals

**Explicitly Out of Scope for MVP:**

- Cross-flow playback behavior
- Account systems or sync workflows
- Cloud-dependent collaboration features
- Full page-layout or pagination implementation
- Broader platform expansion beyond macOS-first execution

### Post-MVP Features

**Phase 2 (Post-MVP):**

- Performance refinements for larger scores
- Broader interaction polish across multi-flow contexts
- Playback evolution beyond single-flow scope
- Early architectural extensions that make page-layout work easier
- Optional MIDI input exploration for recording workflows

**Phase 3 (Expansion):**

- Full page-layout and music-frame model
- Pagination and density-aware system layout
- Optional account-based sync for scores, settings, and template files
- Optional cloud storage integration
- Broader device and ecosystem portability where justified

### Risk Mitigation Strategy

**Technical Risks:** The primary technical risk is damaging the architectural path toward page layout by solving multi-flow rendering too locally or too narrowly. Mitigation requires explicit separation between notation derivation, flow composition, interaction mapping, and future layout concerns. Playback-related scope should remain cuttable so the architecture can stay clean.

**Market Risks:** The main product risk is shipping something that is technically impressive but not meaningfully usable. The MVP addresses this by requiring real interaction across flows, not just passive rendering. Validation should focus on whether the feature makes the score feel more coherent and usable for expert composition work.

**Resource Risks:** If resources tighten, playback-related work and secondary polish should be cut first. The irreducible core is multi-flow rendering plus correct interaction behavior. The feature should only ship when it proves the architectural direction rather than introducing a temporary convenience layer that will need to be replaced.

## Functional Requirements

Requirement identifiers in the Functional Requirements and Non-Functional Requirements sections were normalized on 2026-03-11 to remove editorial numbering gaps and improve downstream traceability without changing requirement intent.

### Score Presentation and Flow Continuity

- FR1: Composers can view multiple flows within a single continuous score presentation rendered by the product's native score renderer.
- FR2: Composers can move through visible flows without using the current single-flow switching pattern for basic score reading.
- FR3: The product can present flows in sequence separated by a fixed gap of at least 20 pixels, sufficient for flow boundaries to be identifiable at standard zoom level.
- FR4: The product can render multiple flows simultaneously without overlapping staves, clipped content, or illegible note spacing.
- FR5: The product can render adjacent flows with consistent staff sizing, clef presentation, and spacing style so they read as a unified score rather than visually independent panels.

### Interaction Across Flows

- FR6: Composers can select score content within any visible flow in the continuous presentation.
- FR7: Composers can trigger and use popovers within any visible flow in the continuous presentation.
- FR8: The product can resolve hit targets correctly across all rendered flows.
- FR9: Composers can interact with score elements across flow boundaries with each interaction resolving to exactly one well-defined target.
- FR10: The product can maintain a clearly indicated active flow for editing purposes while multiple flows are visible simultaneously.
- FR11: The product can support editing interactions within the currently relevant flow while other flows remain visible.

### Notation Integrity and Musical Correctness

- FR12: The product can preserve existing written-notation behavior when multi-flow rendering is enabled.
- FR13: The product can preserve existing written-duration behavior for musical content rendered in multi-flow contexts.
- FR14: The product can render multi-flow score output without requiring users to rewrite musical input to accommodate the presentation model.
- FR15: The product can preserve score output conforming to the notation behaviors documented in the project's notation reference (Elaine Gould, _Behind Bars_) when extending presentation beyond a single flow.

### Navigation and Reading Experience

- FR16: Composers can scroll through the combined multi-flow score presentation as one continuous reading surface.
- FR17: The product can indicate the user's active flow and position through visible affordances while multiple flows are displayed, without requiring additional navigation steps to identify context.
- FR18: The product can display flow boundaries and flow sequence clearly enough that a user can identify individual flows and their relative order without scrolling or additional navigation.
- FR19: The product can support reading across multiple flows without requiring users to invoke a flow-switching interaction for basic score reading and navigation.

### Update Isolation and Rendering Behavior

- FR20: The product can update one flow without requiring unrelated flows to be re-derived unnecessarily.
- FR21: The product can preserve isolated recalculation boundaries between flows during editing.
- FR22: The product can render all non-edited flows without visible update artifacts, stutter, or layout shifts while a local edit is processed in another flow.
- FR23: The product can preserve stable interaction behavior while one visible flow is updated.
- FR24: The product can support multi-flow presentation without coupling all visible flows into a single mandatory recalculation unit.

### Desktop Document Workflow

- FR25: macOS users can open score documents through standard desktop file workflows.
- FR26: macOS users can save and reopen score documents through native desktop document flows.
- FR27: The product can participate in desktop file association workflows for supported score documents.
- FR28: The product can use native-style file interaction patterns appropriate to a desktop document application.
- FR29: The product can support core composition and score interaction workflows without requiring network connectivity.

### Playback and Audio Boundaries

- FR30: Composers can continue using existing playback behavior while multi-flow rendering is introduced.
- FR31: The product can support desktop audio-device-based playback as part of the broader product context without making cross-flow playback a requirement for this MVP.
- FR32: Composers can keep a single active flow for sequencer and piano-roll workflows while multiple flows are visible in score presentation.

### Future-Ready Product Boundaries

- FR33: The product can support future portability of score files, settings, and templates without requiring account-based sync in this MVP.
- FR34: The product can remain usable as a fully offline-first composition environment even as future optional sync capabilities are considered.
- FR35: Maintainers can evolve score presentation architecture without treating notation logic as part of the same capability change.
- FR36: Maintainers can access render-timing output covering multi-flow rendering scenarios to validate correctness and performance during development, extending existing single-flow timing instrumentation where necessary.

## Non-Functional Requirements

### Performance

- NFR1: In release builds on representative target hardware, unchanged-score render requests should complete the render stage in sub-millisecond time where no score changes require re-derivation.
- NFR2: In release builds on representative target hardware, interaction updates and local edits in a score of up to three flows with 60 bars each should complete quickly enough to feel immediate in normal use, with parse-and-render work where recalculation is required completing within one display frame at 60 fps (≤16ms).
- NFR3: The multi-flow architecture changes must not regress single-flow render performance. A single-flow score of 3 instruments and 30 bars, currently rendering in under 1ms, must continue to render in under 1ms after multi-flow support is introduced.
- NFR4: The system must preserve isolated recalculation so that editing one flow does not trigger unnecessary notation or spacing recomputation for unrelated visible flows.

### Reliability and Correctness

- NFR5: Multi-flow rendering must not introduce regressions in notation output.
- NFR6: Any change to multi-flow presentation must preserve the existing written-duration behavior and broader notation semantics already relied on by the product.
- NFR7: Hit targets and popovers must resolve to the correct flow and score element throughout the continuous multi-flow presentation.
- NFR8: Score presentation, active-flow semantics, and interaction mapping must not expose contradictory states to the user — specifically: the visually indicated active flow must always match the flow targeted by editing interactions, and hit targets must always resolve to the flow they are visually within.
- NFR9: The system must fail in ways that preserve score integrity; presentation changes must not corrupt score data, notation state, or document open/save behavior.

### Score-Size Scalability

- NFR10: Multi-flow rendering must remain within one display frame (≤16ms at 60fps) as score size scales from the single-flow baseline to the three-flow, 60-bar reference case in NFR2, with render time growing proportionally to the number of flows and bars requiring recalculation rather than as a fixed cost applied unconditionally.

### Desktop Integration Quality

- NFR11: Native desktop file workflows must behave consistently with macOS Human Interface Guidelines for document-based applications, including native file open/save dialogs, file association, and document state management.
- NFR12: Existing single-flow audio playback must continue to function correctly after multi-flow rendering is introduced — this feature is not required to enable cross-flow playback, but must not break it.
- NFR13: Offline use must remain first-class; the feature must not introduce any network dependency into core composition, rendering, interaction, or playback workflows.

### Accessibility

- NFR14: The product should maintain an accessibility baseline consistent with Apple's macOS Accessibility Programming Guide for core workflows, ensuring keyboard navigation and VoiceOver compatibility are not deliberately degraded by this feature.
- NFR15: Accessibility improvements for this feature must not come at the expense of core notation correctness, interaction clarity, or architectural soundness for the target expert user base.

## Architecture Constraints

These are implementation principles rather than measurable quality attributes. They provide context for architectural decision-making and should be treated as design guidance during technical planning.

- AC1: Performance optimization should remain evidence-driven; implementation decisions should avoid premature complexity while preserving headroom for larger scores.
- AC2: The architecture must support further performance work for larger scores without requiring the multi-flow model to be redesigned from scratch.
- AC3: The implementation should scale by minimizing unnecessary work rather than by coupling more of the score into shared recalculation paths.
- AC4: Notation derivation must remain logically distinct from presentation composition — changes to how flows are rendered must not require changes to how notation is derived.
- AC5: Score presentation behavior and playback behavior must remain logically separate — changes to multi-flow rendering must not require changes to the playback pipeline.
- AC6: Score presentation scope and active-flow sequencing scope must remain logically distinct — the flow active for editing or sequencing need not be the only flow visible in the score presentation.
- AC7: The multi-flow rendering model must be designed to extend toward future page-layout and music-frame workflows without requiring a redesign of the core composition approach.
- AC8: Clear boundaries must be maintained between musical data, notation output, playback behavior, and document/layout concerns so that each can evolve independently.
