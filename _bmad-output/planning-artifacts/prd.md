---
stepsCompleted:
  - "step-01-init.md"
  - "step-02-discovery.md"
  - "step-02b-vision.md"
  - "step-02c-executive-summary.md"
  - "step-03-success.md"
  - "step-04-journeys.md"
  - "step-05-domain.md"
  - "step-06-innovation.md"
  - "step-07-project-type.md"
  - "step-08-scoping.md"
  - "step-09-functional.md"
  - "step-10-nonfunctional.md"
  - "step-11-polish.md"
  - "step-12-complete.md"
inputDocuments:
  - "docs/project-context.md"
  - "docs/architecture.md"
  - "docs/development-guide.md"
  - "docs/project-overview.md"
  - "_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md"
documentCounts:
  briefCount: 0
  researchCount: 0
  brainstormingCount: 0
  projectDocsCount: 5
classification:
  projectType: "desktop_app"
  domain: "general"
  complexity: "high"
  projectContext: "brownfield"
workflowType: "prd"
---

# Product Requirements Document - compose

**Author:** Aled
**Date:** 2026-03-20

## Executive Summary

Compose is a brownfield macOS desktop notation application evolving from an experimental SCVI-based playback path into a VST3-based audio engine. The goal of this initiative is to replace SCVI entirely and make realistic playback a native part of the score-writing workflow rather than a separate technical setup task. The product outcome is immediate, transport-stable, realistic audio feedback from written scores, with VST3 support treated as a first-class platform capability rather than a peripheral integration.

The primary users are score authors who want more realistic playback and faster creative feedback without leaving their notation environment. The deeper problem is not only weak playback realism, but the loss of compositional momentum when playback setup becomes engineering work. This initiative therefore combines three linked concerns into one product effort: robust VST3 support on macOS, a realtime-safe transport-driven playback engine, and a simpler score-first assignment model that brings instruments into the workflow with less friction.

The MVP target is stable transport-driven playback through VST3 with coordinated play, pause, seek, and timing behavior. Discovery, loading, assignment, single-tone production, and editor hosting are important precursor milestones, but they exist in service of proving dependable transport playback. The engine must meet a high technical bar: no locks on the audio thread, strong isolation between rendering and audio execution, and jitter-free behavior under normal composition use. Because the product is unreleased, no backward compatibility or migration path from SCVI is required; SCVI should be removed rather than carried forward.

### What Makes This Special

Compose’s differentiator is not merely that it supports VST3 plugins. It is that it aims to make industry-standard playback feel native inside a score-first composition tool with a simpler, more opinionated UX than workflows that force the user to think like an audio engineer. Users should not have to jump through hoops to bring VST instruments into their workflow just to hear a realistic mockup of their music.

The core product promise is immediate, realistic audio representation of written scores with minimal playback setup friction. That positions Compose against the common failure mode in notation playback tools: believable audio often exists only after cumbersome configuration, fragmented tooling, or a split notation-plus-DAW workflow. Compose should instead make assigning instruments, hearing results, and iterating on orchestration feel direct and coherent.

This initiative is also timely. SCVI proved useful as an experiment, but VST3 already solves the broader ecosystem and compatibility problem and is the industry standard. The product should therefore stop investing in a proprietary playback path and instead build a dependable VST3-hosted engine that can later expand toward richer capabilities such as expression maps and more flexible routing without losing its opinionated core UX.

## Project Classification

- Project Type: Desktop app
- Domain: General software, specifically creative audio and notation playback tooling
- Complexity: High
- Project Context: Brownfield

## Success Criteria

### User Success

Users can trust playback as part of composition rather than treating it as a fragile preview mode. The product succeeds when three user outcomes all hold together: users can get from an empty score to meaningful playback quickly, users can change the music and hear the result with little friction, and users can rely on playback strongly enough to compose against it.

The defining user-success moment is the ability to seek around a score and get stable, realistic playback without glitches. That is the clearest proof that the playback engine is not only producing sound, but behaving like a dependable musical tool. Users should be able to revise notation, re-audition, and navigate through the score without losing momentum or feeling they are entering a separate technical setup workflow.

User-facing failure includes transport behavior that feels unreliable, even if the sound quality is high, and any workflow where realistic playback is available only through awkward or overly technical setup. Simpler UX is therefore part of the product outcome, not just a design preference.

### Business Success

This initiative is successful for the product if it moves playback from a foundational risk area to a credible release capability. In the near term, success means future playback features become incremental extensions instead of requiring more foundational rewrites. The work is worthwhile if it replaces SCVI with a clean-slate VST3-based foundation that no longer blocks the broader product direction.

Over the longer horizon, success means Compose has a credible release story as a serious composition tool. The playback engine should no longer feel experimental or provisional. Instead, it should support a believable claim that Compose can provide realistic, score-first playback on industry-standard technology and evolve further from that base.

### Technical Success

Technical success has three equally mandatory gates. First, the architecture must be safe on the realtime path: no locks on the audio thread, no rendering interference that causes audible glitches, and zero audio dropouts in defined transport regression scenarios. Second, transport behavior must be correct and trustworthy: play, pause, resume, and seek must behave consistently, preserve timing integrity, and avoid hangs, stuck notes, or unstable state transitions. Third, the VST3 host layer must be credible: discovery, loading, retained assignment, playback, and editor integration must work in a best-effort but spec-aligned way for sensible, compliant plugins.

No single one of these gates can compensate for failure in the others. High realism without trustworthy transport is a failure. Technically correct transport with awkward or partial plugin integration is a failure. Broad plugin support with unsafe realtime behavior is a failure.

The compatibility posture for MVP should be best-effort, grounded in sensible adherence to the VST3 specification rather than defensive accommodation of out-of-spec plugins. The host should be evaluated against representative real-world instrument plugins, including the user's existing library and specifically the BBC Spitfire libraries, while avoiding a blanket promise that every arbitrary plugin will work regardless of spec compliance.

### Measurable Outcomes

The MVP should be evaluated against defined representative transport regression scenarios and representative instrument/plugin scenarios rather than anecdotal manual testing alone. Zero audio dropouts in those defined transport scenarios is non-negotiable. The measurable bar should include repeated play, pause, resume, and seek operations across representative scores and plugin assignments without glitches, hangs, drift, or stuck playback state.

Acceptance must be judged against explicit targets for:

- representative score and instrument scenarios for transport regression testing
- representative seek and playback-navigation scenarios
- acceptable startup behavior with cached plugin discovery
- acceptable manual full rescan behavior
- assignment-to-first-audible-playback expectations
- project reopen behavior with valid and missing plugins

Exact numbers can be calibrated during architecture and validation planning, but MVP sign-off must use explicit targets rather than vague "feels fast" or "seems stable" claims.

## Product Scope

### MVP - Minimum Viable Product

- Full replacement of SCVI with a macOS-only VST3 playback path
- Clean-slate implementation direction with SCVI removed by MVP completion
- Cache-first VST3 startup with bounded cached-entry validation, initial discovery when the cache is empty, plus manual full rescan
- Instrument assignment integrated into the main score workflow
- Explicit support for both single-part plugins and compatible multi-part or channel-addressed plugin workflows, including one plugin instance serving multiple score instruments where the plugin exposes that model
- Auto-loading attempt on instrument creation, with unassigned fallback when no mapping exists
- Stable plugin loading and retained per-project assignment state
- Single-tone playback as an enabling milestone
- Basic mixer behavior using the existing UI surface and control model
- Stable transport-driven playback with dependable play, pause, resume, and seek behavior
- Bounded edit-during-playback behavior with explicit update rules for when score changes apply immediately versus at a safe defer point
- Realtime-safe audio architecture with no locks on the audio thread
- Best-effort VST3 support aligned with the specification for sensible plugin integration
- Practical compatibility validation against the user's real VST instrument set, especially BBC Spitfire libraries
- Foundational expression-map support by the end of MVP, designed to grow into a full system and linked to score data where that can be achieved without disproportionate scope
- VST instruments playing back reliably by the end of MVP

### Growth Features (Post-MVP)

- Broader plugin compatibility beyond the validated MVP baseline
- More sophisticated instance-sharing and routing flexibility
- Improved default mappings and assignment ergonomics
- Richer mixer capabilities beyond the current basic scope
- Stronger diagnostics and recovery behavior around missing or problematic plugins
- Deeper automation and articulation-aware playback behavior built on the foundational expression-map model

### Vision (Future)

- A full expression-map system with score-linked behavior and more automatic score-to-playback interpretation
- More flexible routing without turning Compose into an open-ended DAW environment
- Expanded playback intelligence while preserving a simpler score-first UX
- Higher playback sophistication as an extension of the same foundation rather than another architecture reset

## User Journeys

### Journey 1: Composer Gets From Empty Score To Trustworthy Playback

A composer starts a new score because they want immediate feedback on an idea, not a long playback setup session. They add instruments, expect Compose to make sensible auto-loading attempts, and accept that some instruments may remain unassigned when no valid default mapping exists. Their first expectation is not deep control. It is hearing something believable quickly enough that playback becomes part of the act of writing.

As they assign or confirm instruments, the product discovers available VST3 options through its cached registry and keeps the workflow inside the main score environment instead of forcing them into a technical side path. They enter a few bars of music, press play, and expect audible results that are realistic enough to judge pacing, texture, and orchestration choices. The critical delivery moment is not just hearing sound; it is hearing a believable mockup without having to think about plugin-hosting mechanics.

The journey succeeds when the composer feels that playback is now part of the same creative loop as notation. The journey fails if the path from empty score to first useful playback is slow, confusing, or dependent on technical knowledge that most score authors do not want to exercise.

Capabilities revealed by this journey:

- cache-first VST3 discovery, startup availability, and empty-cache fallback discovery
- instrument auto-loading attempt with unassigned fallback
- assignment UX inside the main composition flow
- fast path from score creation to first believable playback
- realistic enough default playback to support composition decisions

### Journey 2: Composer Revises Music And Trusts Seek-Based Playback

A composer is actively working inside an existing score and is making iterative musical changes. They alter orchestration, revise notes, move around the score, and repeatedly re-audition the result. Their need is not just playback realism; it is trust. They need to believe that what they hear after a seek, pause, or restart reflects the written score without transport weirdness or brittle state.

They seek to a point in the score, press play, pause, edit a passage, and resume from another location. This happens repeatedly during real composition work. The emotional bar here is higher than in the first journey: the user is no longer evaluating whether Compose can produce sound, but whether Compose is dependable enough to compose against.

That trust now includes bounded editing during playback. Users do not need every possible edit to propagate instantly, but they do need a clear and consistent rule for what updates immediately, what updates at the next safe playback boundary, and what requires a restart of the current pass. The workflow should stay musically legible rather than surprising.

The climax of this journey is stable, realistic playback after arbitrary navigation. If seek, play, and pause all behave correctly without glitches, stuck notes, or inconsistent state, the composer starts to trust the playback engine as part of their craft. If transport behavior feels unreliable, the product fails even if individual instrument sounds are impressive.

Capabilities revealed by this journey:

- deterministic play, pause, resume, and seek behavior
- bounded live-edit behavior with explicit immediate-versus-deferred update rules
- stable transport state transitions
- zero-dropout playback in representative transport scenarios
- reliable project state after iterative edits
- playback that remains composition-trustworthy rather than demo-only

### Journey 3: Composer Works Around Missing Or Incomplete Plugin Mapping Without Losing Momentum

A composer opens a score or creates an instrument and discovers that a preferred mapping is not available. A plugin may be missing, a default mapping may not exist, or a previously used instrument may now be unavailable. The user should not be punished with a broken project or an unrecoverable playback state.

Instead, Compose leaves the instrument unassigned, makes that state clear, and allows the composer to continue working. They can choose a replacement plugin, continue with partial playback, or defer assignment until later. The product should degrade gracefully rather than forcing immediate technical repair. The emotional goal is resilience: the user stays in control and keeps moving.

The journey succeeds when missing playback assets are handled as recoverable workflow conditions rather than session-breaking failures. It fails if the user feels trapped in repair work, cannot understand why playback is missing, or cannot recover without specialist knowledge.

Capabilities revealed by this journey:

- clear unassigned-instrument state
- recoverable plugin reassignment flow
- project reopen behavior that tolerates missing plugins
- graceful degradation rather than broken-session playback
- persistence model that separates score continuity from plugin availability

### Journey 4: Maintainer Validates The Playback Engine As Release-Credible Infrastructure

A maintainer or advanced internal user is not just composing; they are validating whether the playback system is good enough to ship. They rescan plugins, verify the cached registry, check that representative instrument libraries load, confirm playback on known-good plugins such as BBC Spitfire libraries, and run representative transport regression scenarios.

Their concern is not musical delight first. It is whether the system is architecturally trustworthy: no locks on the audio thread, no rendering interference causing glitches, no regressions in play/pause/seek behavior, and no evidence that future playback features will require another foundational rewrite. They are effectively asking whether the product now has a credible playback platform instead of an experiment.

The journey succeeds when the maintainer can validate sensible VST3 integration, stable transport behavior, and a clean enough architecture that future features such as expression maps feel incremental. It fails if playback appears to work only in narrow happy paths, if plugin support feels ad hoc, or if regressions suggest the architecture will collapse under future scope.

Capabilities revealed by this journey:

- manual full rescan
- representative compatibility validation against real plugin sets
- defined transport regression scenarios
- clear architecture constraints around realtime safety
- observability and validation criteria sufficient for release confidence

### Journey Requirements Summary

These journeys reveal six core capability groups:

- playback onboarding through cached plugin discovery, auto-loading attempts, and main-flow instrument assignment
- bounded live-edit trust through explicit update rules for score changes during active playback
- transport trust through stable play, pause, resume, and seek behavior with zero-dropout representative regressions
- resilience and recovery through unassigned fallback states, missing-plugin recovery, and stable reopen behavior
- VST3 platform credibility through best-effort spec-aligned host behavior, validation against representative real-world libraries, and usable editor/lifecycle support
- maintainable foundation through lock-free audio-thread execution, rendering isolation from audio, and an architecture that allows future playback features to remain incremental

This journey set is strong enough to drive the later functional requirements, because it covers:

- primary user success path
- primary user trust and edge-case path
- recovery and degraded-state path
- maintainer / operations validation path

## Desktop App Specific Requirements

### Project-Type Overview

Compose is a macOS-first native desktop application with a Rust-owned backend and a React/Tauri UI shell. This initiative keeps that overall product type intact while replacing the playback subsystem with a VST3-based audio engine. The app remains a local, offline-capable score-writing tool rather than a cloud-dependent playback service or browser-hosted experience.

The desktop nature of the product is not incidental. It is central to the solution because the product must integrate with local audio devices, locally installed VST3 plugins, native plugin editor windows, and realtime playback constraints that are not appropriately framed as web-first requirements.

### Technical Architecture Considerations

The desktop architecture must preserve local control over plugin discovery, plugin loading, audio-device routing, and native transport execution. Playback and plugin hosting are backend-owned concerns, with the frontend acting as a control and visualization layer rather than the source of runtime truth. This is especially important because the audio engine must satisfy strict realtime requirements and cannot depend on UI-thread behavior for correctness.

The same separation should apply to expression maps. The architecture should treat expression-map behavior as a first-class, score-aware domain model so later work can deliver a full score-linked system without reworking the playback foundation. If MVP scope requires a narrower proving path, the initial authoring or control UX may remain constrained, but the underlying model should not be designed as a manual-only dead end.

Because this is macOS-only in MVP, the implementation should prefer a clean platform-specific design over premature cross-platform abstraction. The PRD should explicitly allow macOS-native decisions where they simplify or harden the host/runtime path, while keeping later expansion possible if the product matures beyond the initial release target.

The architecture must also preserve the app's offline-first desktop workflow. Users should be able to discover plugins, assign instruments, open projects, and work with playback locally without requiring network services. Plugin inventory, project assignment state, and playback behavior should remain local desktop concerns.

### Platform Support

The MVP targets macOS only. Windows and Linux support are out of scope for this PRD. The supported plugin format is VST3 only. AU support is intentionally excluded even though the product is macOS-first, because the immediate goal is reducing architectural scope and building a credible playback foundation on one plugin standard.

This means platform-specific validation should focus on:

- macOS-native audio-device behavior
- macOS VST3 discovery paths and cache behavior
- macOS-native editor hosting and lifecycle
- representative validation against the user's real plugin library on macOS

### System Integration Requirements

The playback engine must integrate with:

- local VST3 installation paths and cached plugin discovery
- the existing audio device selection and settings flow
- the main score workflow for instrument creation and assignment
- project persistence for plugin assignment and playback state
- native transport and mixer behavior within the app

System integration should remain opinionated. The app should not initially expose arbitrary DAW-style routing or effects-chain complexity. The purpose of system integration is to make realistic playback easier to access from a score-first workflow, not to turn Compose into a general-purpose production host.

### Offline Capabilities

The app must remain usable as an offline local desktop tool. Plugin discovery, cached registry use, project open/save, playback preparation, and score-to-playback assignment must not depend on remote services. Missing plugins should degrade to unassigned states rather than breaking the project model or blocking score access.

Offline-first behavior also means the app must own enough local state to reopen projects consistently, recover from missing plugins sensibly, and allow the user to continue composing even when some playback assets are unavailable.

### Implementation Considerations

The product-type-specific implementation burden is concentrated in native desktop seams:

- native plugin discovery and caching
- native plugin editor hosting
- audio-device and transport coordination
- local persistence and reopen behavior
- safe separation of UI work from realtime playback execution

For this project type, the key architectural risk is not browser compatibility or responsive design. It is whether a desktop-native plugin/audio stack can be integrated in a way that feels simple to users while remaining safe and stable at runtime. That desktop-specific requirement should stay explicit throughout the PRD.

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Platform replacement with direct user value.
This MVP is not a market-test stub and not a narrow technology proof. It is the minimum complete replacement of the playback foundation that allows Compose to become a credible score-first composition tool. The MVP must solve the core user problem, which is not merely producing audio, but making realistic playback trustworthy and simple enough to support active composition.

**Resource Requirements:**
This work requires strong Rust systems/audio implementation, native desktop integration on macOS, VST3 host/runtime work, and enough product/UI attention to keep assignment and playback flows simple. The critical skill mix is:

- realtime/audio engine engineering
- native plugin-host integration
- Rust desktop/backend architecture
- score/workflow UX refinement
- regression validation with representative plugin libraries and transport scenarios

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**

- composer gets from empty score to believable playback
- composer changes music and trusts what they hear after play/pause/seek
- composer can recover from missing/unmapped plugin situations without breaking score work
- maintainer can validate the engine against real plugin libraries and transport regressions

**Must-Have Capabilities:**

- SCVI replacement path established and SCVI removed by MVP completion
- macOS-only VST3 discovery with cache-first startup, bounded cached-entry validation, and empty-cache fallback discovery
- manual full rescan
- main-flow instrument assignment
- explicit support for compatible single-part and multi-part or channel-addressed plugin workflows, including one-instance multi-instrument assignment where the plugin exposes it
- instrument auto-loading attempt with unassigned fallback
- stable plugin loading and retained assignment state
- single-tone production as an enabling milestone
- stable transport-driven playback with dependable play, pause, resume, and seek
- explicit edit-during-playback rules for supported score changes, including when updates apply immediately and when they defer to a safe playback boundary
- zero-dropout transport regression performance in defined scenarios
- no locks on the audio thread
- rendering isolated from causing audio glitches
- best-effort spec-aligned VST3 support for sensible plugins
- validation against the user's real plugin set, especially BBC Spitfire libraries
- basic mixer support through the existing UI/control surface
- foundational expression-map capability by the end of MVP, with score-linked behavior where it fits MVP scope and proof-of-concept behavior where full linkage would add disproportionate scope
- sufficient VST editor hosting to make the workflow practically usable, while not allowing host perfection work to displace transport and playback goals

### Post-MVP Features

**Phase 2 (Post-MVP):**

- broader VST3 compatibility coverage beyond validated MVP baseline
- improved default mappings and assignment ergonomics
- more flexible instance-sharing and routing behavior
- stronger diagnostics and recovery behavior for problematic plugins
- richer mixer capabilities beyond the basic control model
- more robust editor-host coverage across a wider plugin set

**Phase 3 (Expansion):**

- more advanced expression-map workflows
- more automatic articulation/expression switching
- deeper playback intelligence tied to score intent
- broader routing flexibility without becoming DAW-like
- richer automation capabilities that build naturally from expression/articulation work

### Risk Mitigation Strategy

**Technical Risks:**
The highest risks are realtime safety, transport correctness, and practical VST3 host compatibility. Mitigation should center on:

- keeping the audio thread lock-free
- defining explicit transport regression scenarios early
- validating against representative real plugin libraries from the start
- treating discovery, assignment, single-note playback, transport playback, and expression switching as dependency-ordered milestones rather than parallel wish lists

**Market Risks:**
The biggest market risk is building a technically impressive playback stack that still feels too awkward to use. The MVP mitigates this by making simpler score-first UX a success condition rather than a secondary polish task.

**Resource Risks:**
The main resource risk is allowing compatibility breadth or host-surface completeness to expand faster than the foundation can support. The mitigation is to keep the MVP promise focused on:

- trusted playback
- best-effort spec-aligned integration
- known-good representative plugin coverage
- a foundational expression-map model that stays compatible with a fuller score-linked system before deeper automation breadth

## Functional Requirements

### Playback Environment & Plugin Discovery

- FR1: Composers can use Compose with a VST3-based playback environment on macOS.
- FR2: Composers can access a discovered inventory of available VST3 plugins from within Compose.
- FR3: Composers can use a cached plugin inventory so previously discovered plugins remain available without a full probe on every launch, while empty-cache startup still performs initial discovery.
- FR4: Composers can trigger a full plugin rescan when they need Compose to refresh the available VST3 inventory.
- FR5: Compose can distinguish between discovered plugins that are currently usable and plugins that cannot currently be used.
- FR6: Maintainers can validate plugin discovery and rescan behavior against representative installed plugin libraries.

### Instrument Assignment & Playback Setup

- FR7: Composers can assign a playable VST-backed sound source to score instruments.
- FR8: Compose can attempt automatic playback assignment when a new score instrument is created.
- FR9: Compose can leave an instrument unassigned when no suitable playback mapping is available.
- FR10: Composers can identify which score instruments are assigned, unassigned, or no longer resolvable.
- FR11: Composers can change the playback assignment for an instrument without restructuring the score.
- FR12: Compose can support both single-part VST instrument workflows and compatible multi-part or channel-addressed VST instrument workflows in MVP, including one plugin instance serving multiple score instruments where the plugin exposes that model.

### Plugin Lifecycle & Editing

- FR13: Composers can load assigned VST3 instruments into the playback environment.
- FR14: Compose can maintain usable plugin instance lifecycle behavior across playback sessions within the app.
- FR15: Composers can open and use plugin-provided editor interfaces for supported VST3 instruments.
- FR16: Compose can preserve playback usability even when some plugin editor interactions are unavailable or limited.
- FR17: Maintainers can validate plugin-host behavior against a representative known-good plugin set.

### Score-To-Playback Mapping

- FR18: Compose can translate written score instruments into playback-capable instrument assignments.
- FR19: Composers can hear audible results from written score material without leaving the core score workflow.
- FR20: Compose can support realistic playback through a defined expression-map model, with at least one active expression per instrument and a path to richer score-linked behavior.
- FR21: Compose can deliver an initial expression-map capability in MVP that links expression behavior to score data where feasible, or proves the concept through a constrained workflow where full linkage would add disproportionate scope.
- FR22: Compose can expand expression-map behavior later into a fuller score-linked system without replacing the underlying playback foundation.

### Transport Playback

- FR23: Composers can start playback of their score through assigned VST instruments.
- FR24: Composers can pause and resume playback without losing trust in playback state.
- FR25: Composers can seek to different score positions and hear playback continue from the selected location.
- FR26: Compose can keep playback state coherent across repeated play, pause, resume, and seek actions.
- FR27: Compose can support transport-driven playback that composers can use as part of iterative writing and revision.
- FR28: Compose can represent playback failure states in ways that allow users to recover without abandoning score work.

### Mixer & Playback Control

- FR29: Composers can control basic per-instrument playback mix settings within Compose.
- FR30: Compose can apply per-instrument playback control settings during score playback.
- FR31: Compose can retain a basic mixer model that supports current playback needs while allowing later expansion.

### Project Persistence & Recovery

- FR32: Compose can persist playback-relevant project state for reopening later in the same product model.
- FR33: Compose can restore instrument assignments when reopening a project if the required plugins are available.
- FR34: Compose can degrade missing or unavailable plugin assignments to recoverable unassigned states when reopening a project.
- FR35: Composers can continue working on a score even when some playback assignments cannot be restored.
- FR36: Compose can remove SCVI from the project's active playback path by MVP completion.

### Validation & Release Readiness

- FR37: Maintainers can evaluate Compose playback against representative transport regression scenarios.
- FR38: Maintainers can evaluate Compose playback against representative real-world VST instrument libraries, including the user's existing library.
- FR39: Compose can support a release-credible playback workflow for a known-good subset of sensible, spec-aligned VST3 plugins.
- FR40: Maintainers can determine whether playback is trustworthy enough to support real composition work rather than demonstration-only playback.

### Editing During Playback

- FR41: Composers can continue editing and updating the score during playback without the workflow collapsing into a blocked or invalid state.

## Non-Functional Requirements

### Performance

Playback must satisfy realtime-safe execution constraints appropriate for a score-first composition product using VST3 instruments. The audio thread must not take locks, and the system must avoid audio dropouts in defined transport regression scenarios. This requirement is release-critical.

Transport playback must remain stable under representative composition scenarios that include repeated play, pause, resume, and seek operations across assigned VST instruments. Performance targets should be validated against explicit test scenarios rather than subjective assessment.

Plugin discovery and assignment workflows must be fast enough to preserve creative momentum. MVP acceptance must define explicit target thresholds for:

- cached startup discovery behavior
- manual full rescan duration
- assignment-to-first-audible-playback latency
- representative project reopen behavior

Rendering and other non-audio activity must not introduce audible playback instability. The system must isolate rendering and other UI-side work from causing glitches during transport playback.

### Reliability

Compose must provide dependable transport behavior that users can trust during active composition. Repeated play, pause, resume, and seek actions must preserve coherent playback state and avoid hangs, stuck notes, corrupted session behavior, or degraded recovery paths.

Project reopen behavior must remain resilient. When required plugins are available, playback assignments must restore correctly. When plugins are missing or unusable, the system must degrade to clear recoverable unassigned states rather than breaking the project or blocking score work.

The playback environment must remain stable enough to support a release-credible known-good subset of spec-aligned VST3 plugins on macOS. Reliability is judged not only by whether playback can start, but whether it remains trustworthy through realistic composition workflows.

### Security

The product must safely handle local plugin discovery, plugin loading, and project persistence without introducing unnecessary security exposure through this initiative. Security scope remains local-desktop focused because this work does not involve payments, multi-tenant services, or regulated personal data.

The system must preserve the integrity of persisted playback-related project data across save, close, and reopen flows, validated through regression coverage for playback-state persistence and recovery. Plugin discovery and loading must operate within normal application permissions and only access local VST3 locations and user-opened project data required for playback.

Security requirements for this initiative center on safe local execution boundaries and data integrity rather than enterprise compliance controls that are not relevant to the product scope.

### Integration

Compose must integrate reliably with:

- local VST3 installation paths on macOS
- cached plugin discovery and manual rescan flows
- the existing audio-device selection and playback settings surfaces
- the main score workflow for instrument creation and assignment
- project persistence and reopen behavior
- native plugin editor hosting where required for usable playback workflows

Integration quality should be judged against sensible adherence to the VST3 specification for compliant plugins rather than blanket accommodation of out-of-spec behavior. The system should be validated against the user's real plugin library, including BBC Spitfire libraries, as part of representative integration testing.

### Measurement & Validation

Non-functional requirements must be validated through explicit representative regression scenarios rather than vague statements of quality. At minimum, the PRD should require defined scenarios for:

- transport regression behavior
- plugin compatibility validation
- project reopen and recovery behavior
- cached discovery and full rescan behavior
- representative score playback under realistic composition conditions
