---
stepsCompleted:
  - "step-01-validate-prerequisites"
  - "step-02-design-epics"
  - "step-03-create-stories"
inputDocuments:
  - "/Users/aledjones/compose/_bmad-output/planning-artifacts/prd.md"
  - "/Users/aledjones/compose/_bmad-output/planning-artifacts/architecture.md"
  - "/Users/aledjones/compose/_bmad-output/planning-artifacts/ux-design-specification.md"
  - "/Users/aledjones/compose/_bmad-output/planning-artifacts/sprint-change-proposal-2026-03-24.md"
---

# compose - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown scaffold for compose, decomposing the requirements from the PRD, UX Design, Architecture, and the approved 2026-03-24 sprint course correction into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Composers can use Compose with a VST3-based playback environment on macOS.
FR2: Composers can access a discovered inventory of available VST3 plugins from within Compose.
FR3: Composers can use a cached plugin inventory so previously discovered plugins remain available without a full probe on every launch, while empty-cache startup still performs initial discovery.
FR4: Composers can trigger a full plugin rescan when they need Compose to refresh the available VST3 inventory.
FR5: Compose can distinguish between discovered plugins that are currently usable and plugins that cannot currently be used.
FR6: Maintainers can validate plugin discovery and rescan behavior against representative installed plugin libraries.
FR7: Composers can route a playable VST-backed slot target to score instruments.
FR8: Compose can keep new score instruments on the explicit manual-routing path until the user assigns playback.
FR9: Compose can leave an instrument unrouted when no suitable playback mapping is available.
FR10: Composers can identify which score instruments are routed, unrouted, or no longer resolvable.
FR11: Composers can change the playback route for an instrument without restructuring the score.
FR12: Compose can support single-target VST instrument workflows in the current MVP while preserving a clean path for later multi-target or channel-addressed expansion.
FR13: Composers can load routed VST3 rack slots into the playback environment.
FR14: Compose can maintain usable plugin instance lifecycle behavior across playback sessions within the app.
FR15: Composers can open and use plugin-provided editor interfaces for supported VST3 instruments.
FR16: Compose can preserve playback usability even when some plugin editor interactions are unavailable or limited.
FR17: Maintainers can validate plugin-host behavior against a representative known-good plugin set.
FR18: Compose can translate written score instruments into playback-capable routes.
FR19: Composers can hear audible results from written score material without leaving the core score workflow.
FR20: Compose can support realistic playback through a defined expression-map model, with at least one active expression per instrument and a path to richer score-linked behavior.
FR21: Compose can deliver an initial expression-map capability in MVP that links expression behavior to score data where feasible, or proves the concept through a constrained workflow where full linkage would add disproportionate scope.
FR22: Compose can expand expression-map behavior later into a fuller score-linked system without replacing the underlying playback foundation.
FR23: Composers can start playback of their score through routed VST instruments.
FR24: Composers can pause and resume playback without losing trust in playback state.
FR25: Composers can seek to different score positions and hear playback continue from the selected location.
FR26: Compose can keep playback state coherent across repeated play, pause, resume, and seek actions.
FR27: Compose can support transport-driven playback that composers can use as part of iterative writing and revision.
FR28: Compose can represent playback failure states in ways that allow users to recover without abandoning score work.
FR29: Composers can control basic route-first playback mix settings within Compose.
FR30: Compose can apply basic playback control settings during score playback according to real VST3-exposed route capability.
FR31: Compose can retain a basic mixer model that keeps the user experience route-based while allowing slot-level semantics where the VST3 boundary requires them.
FR32: Compose can persist playback-relevant project state for reopening later in the same product model.
FR33: Compose can restore rack slots and instrument routes when reopening a project if the required plugins are available.
FR34: Compose can degrade missing or unavailable plugin routes to recoverable unrouted states when reopening a project.
FR35: Composers can continue working on a score even when some playback routes cannot be restored.
FR36: Compose can remove SCVI from the project's active playback path by MVP completion.
FR37: Maintainers can evaluate Compose playback against representative transport regression scenarios.
FR38: Maintainers can evaluate Compose playback against representative real-world VST instrument libraries, including the user's existing library.
FR39: Compose can support a release-credible playback workflow for a known-good subset of sensible, spec-aligned VST3 plugins.
FR40: Maintainers can determine whether playback is trustworthy enough to support real composition work rather than demonstration-only playback.
FR41: Composers can continue editing and updating the score during playback without the workflow collapsing into a blocked or invalid state.

### NonFunctional Requirements

NFR1: The audio thread must remain lock-free during playback.
NFR2: Representative transport regression scenarios must complete with zero audio dropouts.
NFR3: Playback must remain stable through repeated play, pause, resume, and seek operations in representative composition scenarios.
NFR4: Cached plugin discovery, manual full rescan, route-to-first-audible-playback, and project reopen behavior must have explicit measurable performance targets.
NFR5: Rendering and other non-audio activity must not introduce audible glitches or playback instability.
NFR6: Playback state must remain coherent and recoverable without hangs, stuck notes, or corrupted session behavior.
NFR7: Project reopen must restore valid rack slots and routes correctly when plugins are available.
NFR8: Missing or unusable plugins must degrade to clear, recoverable unrouted states rather than breaking score access.
NFR9: The MVP must support a release-credible known-good subset of sensible, spec-aligned VST3 plugins on macOS.
NFR10: Plugin discovery and loading must operate within local desktop permissions and only access required local VST3 and project data paths.
NFR11: Playback-related project data must preserve integrity across save, close, and reopen flows.
NFR12: The system must integrate reliably with local VST3 installation paths, cached discovery, manual rescan, audio-device settings, score workflow routing, project persistence, and native plugin editor hosting.
NFR13: Compatibility evaluation must be grounded in sensible adherence to the VST3 specification rather than blanket support for out-of-spec plugins.
NFR14: Validation must use explicit representative scenarios for transport, plugin compatibility, project reopen and recovery, cached discovery and rescan behavior, and realistic score playback conditions.

### Additional Requirements

- Preserve the existing brownfield Tauri + Rust + React + TypeScript + Vite foundation; implementation must extend the current app rather than reinitialize or replatform it.
- Treat Rust as the sole source of truth for plugin inventory, rack state, slot state, route state, playback preparation state, transport state, and persisted playback-domain state.
- Keep `.soloc` portable by storing project intent and rack or route references only; machine-local plugin discovery and cache state must remain outside project files.
- Split playback architecture into a Rust control plane and a realtime-safe playback plane where the audio path consumes precomputed, lock-safe state only.
- Keep playback-critical communication backend-owned and command-oriented; avoid high-frequency event chatter on transport and playback-critical paths.
- Treat stable, jitter-free playback as a hard architectural and release gate for all implementation choices.
- Model missing plugin resolution as explicit, recoverable unrouted states rather than silent fallback or corrupted project state.
- Treat VST3 plugins as untrusted third-party code from a host-safety perspective and contain host failures without destabilizing playback or project data.
- Keep score-context playback relationships in contextual panels, and app-scoped playback assets or inventories in dedicated windows.
- Reuse backend seams for UI and MCP or automation flows rather than introducing parallel implementation paths.
- Keep validation and regression coverage close to the owning seam rather than creating broad duplicate harnesses.
- Formalize plugin inventory, identity, rack, slot, target, address, route, and recovery models before broader transport and workflow integration.
- Treat basic VST3 support in the current MVP as explicitly single-target-first. Preserve the rack-first route model so later multi-target or channel-addressed expansion remains possible, but do not require shared-slot workflows before first-audible playback is working.
- Respect the project’s release and persistence discipline: any persisted-data change must assess migration impact and update pending migrations in `src-tauri/src/migrations/next.rs`.
- Treat the first implementation work as foundation-building on the existing app, not starter setup; Epic 1 Story 1 must not be project initialization.
- Treat native VST editor hosting as a narrow macOS-only seam first: one explicit editor-host path, one window label, one open command, and one close path before broader host abstraction.
- Keep native VST editor lifecycle backend-owned, with the frontend limited to a thin trigger through existing action and secondary-window patterns.
- Prefer direct AppKit subview attachment inside a dedicated Tauri-managed host window over HTML or React embedding for native plugin UI hosting.
- Keep the native editor host seam separate from transport, score-routing lifecycle, preset management, automation, and broader production-host concerns until the proof is validated.
- Preserve the existing HTML plugin-settings workflow unless a deliberate incremental story chooses to replace or converge it; do not conflate metadata or audition UI with native editor hosting.
- Treat focus, resize, close, reopen, and teardown behavior of the native editor host as explicit acceptance concerns, not incidental implementation details.
- Require manual macOS runbook validation alongside unit and frontend coverage for the native editor host proof.
- Allow a fallback host-windowing route only if direct Tauri and AppKit parenting proves insufficient; record that as a tested architectural outcome rather than assuming it up front.
- Treat the remaining SCVI sampler and runtime code as temporary legacy scaffolding only. Downstream stories must build clean VST-owned seams rather than extending sampler-era contracts.
- Once a VST-owned seam replaces a sampler or SCVI-era product path, the superseded path must be removed from active product behavior rather than preserved as parallel runtime or compatibility ownership.
- Before first-audible-playback stories begin, the plan must establish a trustworthy single-instrument-VST-instance foundation with no active sampler or SCVI-era product-path interference.
- The approved 2026-03-24 course correction inserts a stabilization or reset epic before first audible playback; the prior direct progression from Epic 3 into audible playback is no longer authoritative sequencing.
- Epic 3 should be treated as rack-first model discovery and partial runway, not as a production-trustworthy playback foundation.
- Story implementation must prefer the simplest useful user-facing control surface for the product model; UI added solely to make individual acceptance criteria separately visible is considered scope failure unless it creates clear user value.
- The planning reset must explicitly preserve the rack-first product model while reclassifying unproven richer multi-instrument or multi-timbral VST-instance support as future capability rather than near-ready scope.

### UX Design Requirements

UX-DR1: Instrument rows must expose a compact direct routing control that shows the current playback route with minimal indirection.
UX-DR2: Instrument rows must expose a compact readiness or health signal so users can tell at a glance whether playback is ready, needs attention, or is unavailable.
UX-DR3: The expanded instrument panel must reflect the same routing model as the row while adding metadata, richer status explanation, and deeper contextual controls.
UX-DR4: Compact score-context playback state must collapse into the three user-facing states Ready, Needs Attention, and Unavailable.
UX-DR5: Non-ready states must provide one primary contextual recovery action that opens the relevant rack or route panel in the current score context.
UX-DR6: Playback routing must remain a score-first workflow that feels like choosing a usable sound path for a musical part rather than configuring host topology.
UX-DR7: The common routing and playback workflow must preserve a direct single-target, score-first mental model for the current MVP, with any future multi-target expansion added later without changing that core interaction.
UX-DR8: Native plugin editors must be available as deliberate secondary handoff surfaces for plugin-specific control, not as the default answer to routine playback tasks.
UX-DR9: The system must provide explicit score-context entry points to open the native plugin editor from relevant playback surfaces.
UX-DR10: Expression maps must be represented as durable, reusable playback assets rather than transient properties of whichever score object is selected.
UX-DR11: The instrument panel must show which route-owned expression map is linked to the current instrument’s playback route and whether that linkage is missing or incomplete.
UX-DR12: Both instrument-driven and library-driven expression-map workflows must converge on one dedicated expression-map editor window.
UX-DR13: Full plugin discovery, inventory health, and organisation must live in a dedicated app-scoped Plugin Manager window.
UX-DR14: Contextual plugin and target pickers in instrument rows and panels must present a filtered, musician-friendly view of currently usable plugins and targets for the current routing task.
UX-DR15: Panels must act as score-context relational surfaces, while windows must own app-scoped inventories, reusable assets, and deep editors.
UX-DR16: Canonical app-scoped entry points for Plugin Manager and Expression Map Library must exist in the system app menu, with contextual launch paths also available from score workflows.
UX-DR17: Playback UI must preserve Compose’s existing custom design system, compact density, current blue-highlight language, and familiar panel and window grammar rather than introducing a parallel design language.
UX-DR18: Playback status must never rely on color alone and must combine text, iconography, and consistent placement for accessibility.
UX-DR19: For score-context playback workflows, compact direct controls are preferred over explanatory or multi-control proof surfaces.
UX-DR20: The rack-first model is the canonical playback mental model for Compose; future stories must align to it directly rather than preserving assignment-first or legacy-sampler interaction assumptions.

### FR Coverage Map

FR1: Epic 1 - establish one VST-only playback environment on macOS.
FR2: Epic 2 - expose a discovered inventory of available VST3 plugins from within Compose.
FR3: Epic 2 - preserve cached plugin inventory across launches while still supporting empty-cache discovery.
FR4: Epic 2 - support manual full plugin rescans.
FR5: Epic 2 - distinguish usable plugins from unavailable ones.
FR6: Epic 7 - validate discovery and rescan behavior against representative installed libraries.
FR7: Epic 3 - route a playable VST-backed slot target to score instruments.
FR8: Epic 3 - keep new score instruments on the explicit manual-routing path until playback is assigned.
FR9: Epic 3 - preserve explicit unrouted state when no suitable mapping exists.
FR10: Epic 3 - identify routed, unrouted, and unresolved instruments.
FR11: Epic 3 - change playback route without restructuring the score.
FR12: Epic 3 - keep the MVP single-target while preserving future extensibility.
FR13: Epic 3 - load routed VST3 rack slots into the playback environment.
FR14: Epic 3 - maintain usable plugin instance lifecycle behavior across app sessions.
FR15: Epic 2 - open and use supported plugin editor interfaces.
FR16: Epic 2 - preserve playback usability when plugin editor interactions are limited or unavailable.
FR17: Epic 7 - validate plugin-host behavior against a representative known-good plugin set.
FR18: Epic 4 - translate written score instruments into playback-capable routes for audible playback.
FR19: Epic 4 - hear written score material without leaving the core score workflow.
FR20: Epic 6 - support a defined expression-map model with initial realistic playback behavior.
FR21: Epic 6 - deliver an initial MVP expression-map capability linked to score data where feasible.
FR22: Epic 6 - preserve a path to richer score-linked expression behavior later.
FR23: Epic 4 - start score playback through routed VST instruments.
FR24: Epic 5 - pause and resume playback without losing trust in playback state.
FR25: Epic 5 - seek to different score positions and continue playback coherently.
FR26: Epic 5 - keep playback state coherent across repeated transport actions.
FR27: Epic 5 - support transport-driven playback for iterative writing and revision.
FR28: Epic 5 - represent playback failure states in recoverable ways.
FR29: Epic 4 - control basic route-first playback mix settings within Compose.
FR30: Epic 4 - apply playback control settings during score playback according to real route capability.
FR31: Epic 4 - retain a route-first mixer model while allowing necessary slot-level semantics.
FR32: Epic 5 - persist playback-relevant project state for reopening.
FR33: Epic 5 - restore rack slots and instrument routes on reopen when plugins are available.
FR34: Epic 5 - degrade missing or unavailable plugin routes to recoverable unrouted states.
FR35: Epic 5 - continue score work even when some playback routes cannot be restored.
FR36: Epic 1 - remove SCVI from the active playback path by MVP completion.
FR37: Epic 7 - evaluate playback against representative transport regression scenarios.
FR38: Epic 7 - evaluate playback against representative real-world VST instrument libraries.
FR39: Epic 7 - establish a release-credible workflow for a known-good subset of sensible, spec-aligned VST3 plugins.
FR40: Epic 7 - determine whether playback is trustworthy enough to support real composition work.
FR41: Epic 5 - support editing during playback without collapsing the workflow.

## Epic List

### Epic 1: Trust One VST-Only Playback Foundation

Composers and maintainers can rely on a single VST-owned playback path, with SCVI removed from active behavior and the reset foundation treated as trustworthy enough to build on.
**FRs covered:** FR1, FR36

### Epic 2: Find And Prepare Usable Plugins

Composers can discover, rescan, inspect, and open supported VST plugins from inside Compose so playback setup starts in the app instead of outside it.
**FRs covered:** FR2, FR3, FR4, FR5, FR15, FR16

### Epic 3: Route Score Instruments With Rack-First Clarity

Composers can assign instruments to rack-first playback targets, understand readiness or unrouted state, and keep usable route and plugin session behavior without restructuring the score.
**FRs covered:** FR7, FR8, FR9, FR10, FR11, FR12, FR13, FR14

### Epic 4: Hear The Score Through Routed Instruments

Composers can move from written notation to first audible VST-backed playback, with basic route-first mix control inside the core score workflow.
**FRs covered:** FR18, FR19, FR23, FR29, FR30, FR31

### Epic 5: Trust Playback During Real Composition

Composers can pause, resume, seek, edit during playback, reopen projects, and recover from missing routes without losing trust in the score workflow.
**FRs covered:** FR24, FR25, FR26, FR27, FR28, FR32, FR33, FR34, FR35, FR41

### Epic 6: Shape Playback With Score-Linked Expression Maps

Composers can use an initial durable expression-map model to make playback more realistic now, while preserving a clean path to richer score-linked behavior later.
**FRs covered:** FR20, FR21, FR22

### Epic 7: Prove A Release-Credible Support Baseline

Maintainers can validate discovery, plugin-host behavior, transport trust, and representative plugin-library coverage so the supported VST subset is explicit and shippable.
**FRs covered:** FR6, FR17, FR37, FR38, FR39, FR40

## Epic 1: Trust One VST-Only Playback Foundation

Composers and maintainers can rely on a single VST-owned playback path, with SCVI removed from active behavior and the reset foundation treated as trustworthy enough to build on.

### Story 1.1: Remove Legacy Playback Path Ownership

As a composer,
I want Compose to use only the supported VST-owned playback path,
So that playback behavior is predictable and no hidden sampler or SCVI path changes what I hear.

**Acceptance Criteria:**

**Given** the app contains sampler-era or SCVI-era playback entry points, state surfaces, or routing hooks in the active product path
**When** the VST playback path is invoked through the supported UI and backend actions
**Then** the active playback flow uses only VST-owned runtime seams
**And** superseded sampler or SCVI product-path behavior is deleted or disconnected rather than preserved as fallback ownership

**Given** existing score and playback actions call backend playback seams
**When** those actions are executed after the reset
**Then** they resolve through one authoritative VST-owned backend path
**And** no frontend-facing action or state surface continues to present legacy playback ownership as active behavior

**Given** legacy removal changes persisted or shared playback-domain structures
**When** the implementation changes load or save semantics or persisted data shape
**Then** migration impact is assessed and pending migration steps are updated as required
**And** portable project intent remains intact

### Story 1.2: Reassert Rust-Owned Playback Domain Boundaries

As a maintainer,
I want the playback domain to have one authoritative backend-owned model,
So that downstream work does not reintroduce parallel truth or unclear ownership.

**Acceptance Criteria:**

**Given** playback inventory, rack, slot, route, transport, and recovery data cross backend and frontend boundaries
**When** the Epic 1 foundation reset is complete
**Then** Rust remains the sole source of truth for active playback-domain state
**And** frontend state is limited to transient presentation concerns

**Given** project files and machine-local plugin state have different responsibilities
**When** playback-related state is saved or restored
**Then** `.soloc` stores portable project intent and references only
**And** machine-local discovery, cache, and validation state remain outside project files

**Given** backend-owned playback models are exposed to the frontend
**When** commands and shared payloads are updated
**Then** naming and ownership stay aligned with the architecture rules
**And** no new sampler-era terminology or parallel adapter model is introduced for the supported path

### Story 1.3: Establish A Trusted Single-Instrument Reset Baseline

As a maintainer,
I want explicit trust-restoration checks for the reset playback path,
So that later audible-playback work starts from a verified single-instrument VST-only baseline instead of assumed progress.

For this story, "single-instrument" means a VST instance that hosts one playable instrument at a time, even if that instrument exposes multiple articulations or expressions. HALion-style multi-slot or multi-part instances remain outside this baseline.

**Acceptance Criteria:**

**Given** the legacy path has been removed from active behavior and backend ownership has been clarified
**When** the reset baseline is evaluated
**Then** the project has explicit checks for single-instrument-VST-instance activation and runtime coherence
**And** those checks are strong enough to block downstream epic work if the foundation is still untrusted

**Given** the reset baseline is now the gate before first-audible-playback work
**When** trust-restoration criteria are documented
**Then** they define what must be true before Epic 4 style audible-playback stories can begin
**And** they do not overclaim unsupported multi-instrument, multi-timbral, or richer runtime capability

**Given** this reset affects developer expectations and planning artifacts
**When** the baseline is finalized
**Then** the relevant documentation and planning guidance reflect the VST-only foundation and deletion rules
**And** the historical Epic 3 runway is not described as a production-trustworthy playback base

**Planning Note:** Story 1.3 established the explicit trust-restoration baseline, and follow-up Stories 1.4 through 1.6 plus the required maintainer manual proof completed that gate on the corrected slot-owned-session architecture. Epic 4 is now the next execution epic. Epic 3 then exercised that stabilized foundation through compact routing, explicit recovery states, and same-session stability work rather than serving as a separate blocker.

### Story 1.4: Make Rack Slots Own Live Plugin Sessions

As a maintainer,
I want each routed playback slot to own one real initialized VST instance at runtime,
So that playback and plugin state no longer depend on editor-window lifetime while project files remain portable.

**Acceptance Criteria:**

**Given** a slot is assigned to an available plugin
**When** the backend prepares or restores that slot
**Then** Rust creates or restores a slot-owned runtime session keyed by `slot_id`
**And** that session is the authoritative live plugin ownership seam for the slot

**Given** a slot-owned session exists
**When** playback or score-context workflows query readiness
**Then** they reflect that real runtime ownership instead of editor-created approximation
**And** they do not treat editor-open as the act that creates canonical runtime state

**Given** a slot is unassigned, removed, or the project closes
**When** ownership ends
**Then** the slot-owned session is torn down at that ownership boundary rather than at editor close
**And** the teardown leaves persisted user intent consistent

**Given** a project is saved
**When** playback state is persisted
**Then** `.soloc` stores only the portable plugin intent and the minimum data needed to rehydrate the slot-owned session later
**And** it does not persist live host-session objects or machine-local runtime cache state

### Story 1.5: Make Native Plugin Editors Transient Views On Slot-Owned Sessions

As a composer,
I want the native plugin editor to open against the running instance already owned by a slot,
So that opening or closing the editor does not create, destroy, or swap the real playback instance.

**Acceptance Criteria:**

**Given** a slot-owned runtime session already exists
**When** I open the editor
**Then** Compose attaches the editor to that existing slot-owned session rather than creating a second canonical instance
**And** the attach flow preserves the slot's current plugin state

**Given** the editor is closed
**When** the slot still exists
**Then** the editor detaches without destroying the slot-owned runtime session
**And** the slot remains playback-ready according to backend-owned truth

**Given** multiple slots host the same plugin bundle
**When** I open or reopen editors across them
**Then** each editor targets the exact slot-owned instance rather than a bundle-level approximation
**And** same-bundle slots remain distinct backend-owned editor targets

### Story 1.6: Initialize And Restore Slot-Owned Sessions On Assignment And Reopen

As a composer,
I want assigned slots to initialize their plugin instances during assignment, preparation, or project reopen from portable saved intent, including restorable plugin configuration,
So that the plugin is already the intended running instrument before I ever open its editor.

**Acceptance Criteria:**

**Given** I assign a plugin to a slot
**When** the backend accepts the assignment
**Then** the slot enters an explicit initialization or recovery path that can produce a real running slot-owned session without requiring editor-open
**And** the resulting state is surfaced through backend-owned readiness semantics

**Given** a project is reopened and the plugin is available
**When** slot and route intent are restored
**Then** the slot-owned session is restored or reinitialized before editor-open and without rewriting project intent
**And** the restored session reflects the portable saved intent recorded in `.soloc`, including the previously selected restorable instrument or patch state needed for the intended running instrument

**Given** the plugin cannot be initialized or restored
**When** the backend surfaces state
**Then** Compose reports an explicit recoverable non-ready condition rather than silently pretending the slot is initialized
**And** the score remains usable

**Given** `.soloc` is loaded
**When** slot and plugin configuration are rehydrated
**Then** the backend uses the portable saved intent to rebuild the slot-owned runtime session, including slot-to-plugin relationships and any restorable plugin configuration required to recover the intended instrument selection
**And** the file is not required to contain live runtime state

## Epic 2: Find And Prepare Usable Plugins

Composers can discover, rescan, inspect, and open supported VST plugins from inside Compose so playback setup starts in the app instead of outside it.

### Story 2.1: Build The App-Scoped Plugin Inventory Window

As a composer,
I want an app-scoped Plugin Manager that shows discovered VST plugins and their current health,
So that I can understand what Compose can use before I try routing playback.

**Acceptance Criteria:**

**Given** Compose has a machine-local plugin inventory source
**When** I open the Plugin Manager from its canonical app-scoped entry point
**Then** I can see the discovered VST plugin inventory in a dedicated window
**And** each entry shows enough status to distinguish usable plugins from unavailable or attention-needed ones

**Given** plugin inventory state is machine-local rather than project-owned
**When** the Plugin Manager is opened
**Then** it reflects backend-owned current inventory truth
**And** it does not imply that plugin availability is stored inside the project file

**Given** the product uses score-context panels and app-scoped windows for different responsibilities
**When** I inspect plugin inventory and scan health
**Then** that work happens in the Plugin Manager window rather than a score-context panel
**And** the window aligns with the existing Compose design system and scope model

### Story 2.2: Support Cached Discovery And Manual Rescan

As a composer,
I want Compose to load known plugins quickly and let me rescan when needed,
So that startup stays fast without preventing me from refreshing my installed inventory.

**Acceptance Criteria:**

**Given** Compose has a previously built plugin inventory cache
**When** the app starts
**Then** previously discovered plugins are available from the cached inventory without a full probe on every launch
**And** empty-cache startup still performs the required initial discovery path

**Given** I install, remove, or troubleshoot local plugins
**When** I trigger a manual full rescan from the supported inventory workflow
**Then** Compose refreshes the plugin inventory from the local VST3 locations
**And** the resulting inventory state is reflected in the Plugin Manager

**Given** discovery behavior has explicit performance and validation expectations
**When** cached discovery and manual rescan are implemented
**Then** the work includes measurable behavior or checks for startup and rescan expectations
**And** discovery remains limited to required local desktop paths and permissions

### Story 2.3: Expose Musician-Friendly Plugin Selection Inputs

As a composer,
I want score workflows to show only usable plugin choices in a clear musical form,
So that I can choose playback options without dealing with raw host-inventory complexity.

**Acceptance Criteria:**

**Given** Compose has a backend-owned plugin inventory with health status
**When** a score-context routing surface needs plugin choices
**Then** it receives a filtered view of currently usable plugins and targets
**And** unavailable or invalid inventory entries are not presented as ordinary ready-to-route choices

**Given** inventory management is app-scoped but routing is score-context
**When** I need more information about missing or unavailable options
**Then** the score-context surface can direct me to the Plugin Manager
**And** it does not duplicate the full inventory-management workflow locally

**Given** the rack-first model is the canonical playback mental model
**When** plugin options are presented in contextual pickers
**Then** they are framed as usable musical playback choices rather than host-topology internals
**And** the UI stays compact rather than expanding into proof-oriented control sprawl

### Story 2.4: Open Supported Native Plugin Editors Through A Bounded Host Seam

As a composer,
I want to open a supported plugin’s native editor from Compose,
So that I can adjust plugin-specific controls without Compose taking over that responsibility.

**Acceptance Criteria:**

**Given** a supported VST plugin is available through the bounded native editor-host seam
**When** I invoke the supported open-editor action from Compose
**Then** the plugin’s native editor opens through the backend-owned host path
**And** Compose treats it as a secondary handoff surface rather than a primary playback workflow

**Given** native editor support may be limited for some plugins or scenarios
**When** the editor cannot be opened or a supported interaction is unavailable
**Then** Compose preserves overall playback usability
**And** the product reports the limitation without implying that core routing or inventory state is lost

**Given** the native editor host seam has explicit lifecycle expectations
**When** the editor is opened, focused, resized, closed, or reopened
**Then** the supported lifecycle behavior follows the bounded host design
**And** it does not expand scope into transport, routing, preset-management, or broader production-host ownership

## Epic 3: Route Score Instruments With Rack-First Clarity

Composers can assign instruments to rack-first playback targets, understand readiness or unrouted state, and keep usable route and plugin session behavior without restructuring the score.

### Story 3.1: Show Compact Rack-First Route State On Instruments

As a composer,
I want each score instrument to show its current playback route and readiness in a compact form,
So that I can tell at a glance whether it is ready, needs attention, or is unrouted.

**Acceptance Criteria:**

**Given** a score contains instruments with different playback situations
**When** I view the score-context instrument surfaces
**Then** each instrument shows a compact route summary and a compact readiness state
**And** the user-facing readiness model collapses into Ready, Needs Attention, or Unavailable

**Given** a route is missing, unresolved, or explicitly absent
**When** the instrument state is displayed
**Then** the UI distinguishes that condition from a ready route
**And** it does so without relying on color alone

**Given** the rack-first model is the canonical interaction
**When** instrument playback state is presented
**Then** the UI frames the relationship as instrument-to-playback-route
**And** it avoids extra explanatory or proof-oriented controls that do not add user value

### Story 3.2: Assign And Change Single-Target Playback Routes

As a composer,
I want to assign or change a single-target playback route for an instrument,
So that I can connect musical parts to usable VST-backed playback without restructuring the score.

**Acceptance Criteria:**

**Given** an instrument has no playback route or an existing route
**When** I use the supported score-context routing control
**Then** I can assign or change the instrument’s single-target playback route
**And** the route is expressed through the rack-first model rather than legacy assignment or sampler language

**Given** only some plugins or targets are currently usable
**When** I select a playback route
**Then** the control presents only the valid contextual choices supplied by the backend
**And** invalid or unavailable options are not presented as ordinary ready choices

**Given** the MVP is single-target-first
**When** a route is assigned in this epic
**Then** the supported workflow remains bounded to one target per instrument
**And** the underlying model does not block later multi-target expansion

### Story 3.3: Preserve Explicit Unrouted And Recovery States

As a composer,
I want unrouted and unresolved instruments to remain explicit and recoverable,
So that missing playback setup does not break my score or force hidden fallback behavior.

**Acceptance Criteria:**

**Given** an instrument has no suitable playback mapping or a previously valid route cannot currently resolve
**When** Compose evaluates its playback state
**Then** the instrument remains explicitly unrouted or attention-needed instead of silently falling back
**And** the score remains usable

**Given** a non-ready instrument is shown in score-context surfaces
**When** I inspect its state
**Then** the UI provides one primary contextual recovery path into the relevant rack or route workflow
**And** that recovery path stays inside the score-context model unless deeper app-scoped inventory work is needed

**Given** playback readiness is a backend-owned domain state
**When** unresolved or missing-route states are surfaced
**Then** they preserve user intent and route truth
**And** they do not invent frontend-only fallback state

### Story 3.4: Keep Route And Plugin Session Behavior Stable In-App

As a composer,
I want routed instruments and their plugin sessions to remain stable while I work,
So that changing routes or reopening relevant playback surfaces does not make the workflow feel fragile.

**Acceptance Criteria:**

**Given** an instrument has a valid routed playback target
**When** I continue using the score workflow in the same app session
**Then** the route remains associated with the intended rack and slot state
**And** the plugin session behavior remains usable for that routed instrument

**Given** I change a route for an existing instrument
**When** the new route is applied
**Then** the old and new route state transitions remain coherent
**And** the score does not need restructuring to accommodate the change

**Given** route and plugin session state are backend-owned
**When** score-context routing workflows are used repeatedly in the same session
**Then** the frontend continues reflecting backend truth
**And** no parallel route-state ownership is introduced in the UI

**Planning Note:** Story 3.4 assumes Stories 1.4 through 1.6 have already established slot-owned runtime-session ownership and transient editor attachment. It does not replace that Epic 1 foundation work.

## Epic 4: Hear The Score Through Routed Instruments

Readiness Note: The Story 1.3 through 1.6 trust-restoration gate and the required maintainer manual proof are complete on the corrected slot-owned-session architecture. Epic 4 is now ready to begin.

Composers can move from written notation to first audible VST-backed playback, with basic route-first mix control inside the core score workflow.

### Story 4.1: Translate Routed Score Instruments Into Playback Preparation

As a composer,
I want routed score instruments to prepare a playback-capable path from notation into the VST runtime,
So that written music can become audible without leaving the score workflow.

**Acceptance Criteria:**

**Given** a score instrument has a valid routed playback target
**When** Compose prepares the score for playback or audition
**Then** the written instrument is translated into the backend playback path required for the routed VST target
**And** that preparation remains backend-owned rather than recreated in frontend state

**Given** an instrument is unrouted or unresolved
**When** playback preparation runs
**Then** Compose preserves the explicit non-ready state for that instrument
**And** it does not invent hidden fallback playback behavior

**Given** the current MVP is single-target-first
**When** playback preparation resolves routed instruments
**Then** the supported preparation path stays bounded to the current single-target model
**And** the model remains compatible with later expansion

### Story 4.2: Prove First Audible Playback Through Auditioning

As a composer,
I want to audition routed written material from the score workflow,
So that Compose can prove the first audible playback path before full transport behavior is introduced.

For this story, auditioning means playing one specific pitch for 1 second at a moderate velocity using a default legato articulation so the user can hear pitches in the instrument voice while interacting with the score.

**Acceptance Criteria:**

**Given** at least one score instrument has a valid routed playback target
**When** I invoke the supported audition workflow from the score context
**Then** Compose plays the targeted score pitch for 1 second through the routed VST-backed instrument voice at a moderate velocity with a default legato articulation
**And** the audible result comes from score-derived musical content rather than a separate plugin-only test path

**Given** some instruments are routed and others are not
**When** auditioning begins for the currently targeted tone
**Then** the selected tone's routed instrument can sound through the supported playback path
**And** unrouted instruments remain explicitly non-ready rather than blocking the workflow

**Given** Epic 4 is proving playback basics rather than full transport trust
**When** this story is completed
**Then** the result demonstrates a trustworthy first-audible path through auditioning
**And** it does not require play, pause, resume, or seek behavior that belongs to later transport stories

**Given** the user is interacting with score tones
**When** a tone is selected through the renderer or sequencer, its pitch is changed through keyboard commands or sequencer vertical dragging, or its accidental is changed through the left-panel control or keyboard command
**Then** Compose triggers the supported one-second audition for the affected pitch through the same routed score-context playback path

### Story 4.3: Establish A Continuous Render Path For Slot-Owned Sessions

As a composer,
I want mounted routed plugin sessions to stay connected to a continuous playback render path,
So that auditioning, plugin-UI interaction, and later transport work all use the same trustworthy audio foundation.

**Acceptance Criteria:**

**Given** a routed slot-owned plugin session exists and is render-active
**When** Compose's audio engine is running
**Then** the session is processed through a continuous backend-owned render path rather than only through burst audition rendering
**And** silence-safe buffers continue to flow even when no notes are currently scheduled

**Given** the user opens a native plugin editor for a mounted routed session
**When** the plugin exposes its own playable UI such as a built-in keyboard
**Then** that interaction can sound through the same continuous render path
**And** the editor window remains a transient view over the already-running slot-owned session rather than a second playback owner

**Given** transport trust work is still upcoming
**When** this render path is introduced
**Then** transport remains responsible for scheduling and transport-state transitions rather than audio-stream ownership
**And** the OS output stream, mixer lifetime, and live plugin processing stay outside transport

**Given** Compose must stay realtime-safe and rack-first
**When** the correction story is implemented
**Then** the audio path consumes precomputed lock-safe state and backend-owned event queues
**And** the implementation does not reintroduce sampler-style fallbacks, frontend-owned playback truth, or plugin-only side seams

**Planning Note:** This is a correction story inserted after Story 4.2 so the concrete implementation matches the already-approved architecture requirement for a realtime-safe playback plane before mix-control and transport-trust expansion continue.

### Story 4.4: Apply Basic Route-First Mix Controls During Audition And Playback

As a composer,
I want basic route-first mix controls to affect audible output,
So that I can make bounded playback adjustments within Compose while keeping the user model tied to routes.

**Acceptance Criteria:**

**Given** a routed instrument participates in auditioning or supported score playback
**When** I adjust a supported basic mix control in Compose
**Then** the control affects audible output according to the real route capability exposed by the active VST path
**And** the user-facing control remains route-first in meaning

**Given** some control semantics may still cross slot boundaries internally
**When** mix behavior is implemented
**Then** the product preserves the route-first user model
**And** any required slot-level mechanics remain behind the scenes

**Given** a requested mix control is not truly supported by the active route capability
**When** the playback surface is shown
**Then** Compose does not pretend the control is available
**And** the bounded MVP mixer surface remains aligned with actual supported behavior

### Story 4.5: Keep First Audible Playback Compact And Score-First

As a composer,
I want the first audible playback workflow to stay compact and tied to score-context controls,
So that hearing the score feels like part of composition instead of a detour into playback tooling.

**Acceptance Criteria:**

**Given** first audible playback is being proved through score-context auditioning and related controls
**When** the UI for this epic is implemented
**Then** the useful workflow is available through compact score-context controls
**And** the design avoids extra explanatory surfaces or duplicate controls that exist mainly to prove acceptance criteria

**Given** deeper inventory and plugin editing already belong to app-scoped windows
**When** first audible playback is used
**Then** the core audible workflow stays in the score context
**And** it escalates outward only when a real user need requires it

**Given** Compose uses a custom compact design language
**When** first audible playback surfaces are added or refined
**Then** they preserve the existing panel and window grammar
**And** readiness and control states remain legible without relying on color alone

## Epic 5: Trust Playback During Real Composition

Composers can pause, resume, seek, edit during playback, reopen projects, and recover from missing routes without losing trust in the score workflow.

### Story 5.1: Start, Pause, And Resume Trusted Score Playback

As a composer,
I want playback to start, pause, and resume coherently,
So that I can re-audition my score during composition without losing trust in playback state.

**Acceptance Criteria:**

**Given** the score has a supported audible playback path from the earlier epic
**When** I start, pause, and resume playback from the supported workflow
**Then** playback state transitions remain coherent
**And** the app avoids hangs, stuck notes, or corrupted session behavior during those actions

**Given** repeated transport actions are common during composition
**When** I perform multiple start, pause, and resume cycles
**Then** the supported playback path remains stable
**And** the behavior is suitable for representative regression validation rather than one-off demos

**Given** some score instruments may still be non-ready
**When** playback is started or resumed
**Then** ready routes continue through the supported playback path
**And** missing or unresolved routes remain recoverable without collapsing the session

### Story 5.2: Seek To Score Positions Without Losing Playback Trust

As a composer,
I want to seek to another score position and continue playback coherently,
So that I can navigate the piece freely while composing.

**Acceptance Criteria:**

**Given** the score is using the supported playback path
**When** I seek to another supported position and continue playback
**Then** playback resumes from the intended location through the routed VST path
**And** the transport state remains coherent across repeated seek operations

**Given** seek behavior is a trust-critical transition
**When** representative seek scenarios are exercised
**Then** the system avoids dropouts, hangs, and stale playback state
**And** the resulting behavior is strong enough to support composition rather than audition-only proof

**Given** the product remains score-first
**When** seek is used in the main workflow
**Then** the user experience stays tied to score navigation rather than host internals
**And** transport state remains backend-owned

### Story 5.3: Define Bounded Editing During Playback

As a composer,
I want clear rules for what score edits apply during playback and when,
So that editing while listening feels predictable rather than fragile.

**Acceptance Criteria:**

**Given** I make supported score edits during active playback
**When** the system evaluates those edits
**Then** it applies them immediately or defers them to a safe boundary according to explicit documented rules
**And** the workflow remains valid rather than collapsing into a blocked state

**Given** some edits are not safe to apply immediately
**When** they occur during playback
**Then** Compose preserves playback coherence and user intent
**And** it communicates the bounded update behavior without forcing a debugging mindset

**Given** edit-during-playback is part of real composition work
**When** representative scenarios are exercised
**Then** the supported behaviors are consistent and testable
**And** unsupported live-edit behavior is bounded rather than left ambiguous

### Story 5.4: Restore Playback State On Project Reopen

As a composer,
I want valid playback routes and rack state to restore when I reopen a project,
So that playback remains part of the normal project lifecycle.

**Acceptance Criteria:**

**Given** a saved project contains valid playback intent and the required plugins are available
**When** I reopen the project
**Then** Compose restores the relevant rack slots and instrument routes
**And** playback-relevant state remains coherent with the backend-owned model

**Given** project files must remain portable
**When** reopen restoration occurs
**Then** restoration resolves saved project intent against machine-local plugin inventory
**And** project files do not contain machine-local cache or discovery truth

**Given** reopened playback state is part of trusted composition workflow
**When** reopen scenarios are validated
**Then** state integrity across save, close, and reopen is preserved
**And** restoration behavior is suitable for regression validation

### Story 5.5: Recover Gracefully From Missing Or Unavailable Routes

As a composer,
I want missing plugins or unavailable routes to degrade into recoverable states,
So that I can continue working even when playback assets are unavailable.

**Acceptance Criteria:**

**Given** a previously saved or expected route cannot currently resolve
**When** Compose restores or evaluates playback state
**Then** the route degrades to an explicit recoverable non-ready state
**And** the score remains usable

**Given** some routes are recoverable and others are ready
**When** I continue working in the project
**Then** ready routes keep working through the supported playback path
**And** unresolved routes do not block score access or force hidden substitutions

**Given** recovery behavior is part of user trust
**When** missing-plugin and unavailable-route scenarios are surfaced
**Then** Compose preserves user intent and offers a clear recovery path
**And** it avoids silent fallback or corrupted playback truth

## Epic 6: Shape Playback With Score-Linked Expression Maps

Composers can use an initial durable expression-map model to make playback more realistic now, while preserving a clean path to richer score-linked behavior later.

### Story 6.1: Model Expression Maps As Durable Playback Assets

As a composer,
I want expression maps to exist as durable reusable playback assets,
So that playback realism is based on a coherent model rather than transient per-selection settings.

**Acceptance Criteria:**

**Given** expression behavior is part of playback realism
**When** Compose introduces the initial expression-map capability
**Then** expression maps are modeled as durable reusable assets rather than transient UI-only state
**And** the model preserves a path to richer score-linked behavior later

**Given** score-context panels and app-scoped windows have different responsibilities
**When** expression-map architecture is surfaced in the product
**Then** score-context surfaces show route linkage and current status
**And** the expression-map asset itself is owned outside transient selection context

**Given** the MVP may need a bounded first implementation
**When** the initial model is defined
**Then** it can support a constrained first-use path
**And** it does not lock the product into a manual-only dead end

### Story 6.2: Show Expression-Map Linkage In Score Context

As a composer,
I want to see whether my current routed instrument has an expression map and whether that linkage is healthy,
So that I can understand how playback realism is being driven for the current score context.

**Acceptance Criteria:**

**Given** an instrument is associated with a playback route
**When** I inspect the relevant score-context panel
**Then** I can see which expression map is linked to that route, if any
**And** I can tell whether the linkage is ready, missing, or incomplete

**Given** expression maps are reusable assets rather than selection-owned data
**When** linkage is displayed in score context
**Then** the panel shows the relationship to the current route
**And** it does not behave like the panel owns the asset definition itself

**Given** a linkage needs deeper work
**When** I use the score-context surface
**Then** Compose provides a contextual path into the dedicated expression-map editor
**And** that escalation stays aligned with the panel-versus-window scope model

### Story 6.3: Create One Dedicated Expression-Map Editor Window

As a composer,
I want one dedicated editor for expression maps,
So that both instrument-driven fixes and library-driven authoring use the same durable workflow.

**Acceptance Criteria:**

**Given** I enter expression-map work from a score-context panel or from an app-scoped library path
**When** the editor is opened
**Then** both entry paths converge on the same dedicated expression-map editor window
**And** the editor owns the expression-map asset itself

**Given** expression-map work is a reusable playback-asset workflow
**When** I use the dedicated editor
**Then** the workflow is framed in musical playback terms rather than low-level host internals
**And** the interface remains aligned with Compose’s existing design language

**Given** expression-map editing is app-scoped
**When** the editor window is used
**Then** it behaves as a durable asset-management surface rather than a transient properties panel
**And** it remains distinct from plugin-owned parameter editing

### Story 6.4: Drive Initial Realistic Playback Through A Linked Expression Map

As a composer,
I want an initial linked expression-map behavior to affect playback,
So that routed score playback sounds more musically meaningful than a raw default trigger path.

**Acceptance Criteria:**

**Given** a routed instrument has an expression map configured through the supported MVP path
**When** I audition or play supported written material
**Then** the active expression-map behavior influences playback through the routed VST path
**And** at least one active expression per instrument is supported through the defined model

**Given** full score-linked expression sophistication may exceed MVP scope
**When** the initial realistic playback path is implemented
**Then** the supported behavior can be constrained to a bounded first-use case
**And** it still proves the model can connect score meaning to playback behavior

**Given** future realism work must build on this foundation rather than replace it
**When** the initial expression-map path is delivered
**Then** the underlying model remains extensible to richer score-linked expression behavior later
**And** the implementation does not hard-code a one-off path that bypasses the durable asset model

## Epic 7: Prove A Release-Credible Support Baseline

Maintainers can validate discovery, plugin-host behavior, transport trust, and representative plugin-library coverage so the supported VST subset is explicit and shippable.

### Story 7.1: Define The Supported Plugin And Scenario Baseline

As a maintainer,
I want an explicit known-good plugin and scenario baseline,
So that Compose has a concrete definition of what supported and shippable means for MVP playback.

**Acceptance Criteria:**

**Given** MVP support is intentionally best-effort and bounded
**When** the support baseline is defined
**Then** it identifies the representative plugin set, scenario set, and compatibility expectations used for release decisions
**And** it does not claim blanket support for out-of-spec or unvalidated plugins

**Given** the product must be validated against representative real-world libraries
**When** the baseline is documented
**Then** it includes the user's real plugin library coverage expectations, including the relevant BBC Spitfire libraries
**And** it distinguishes known-good support from future compatibility expansion

**Given** release claims need explicit evidence
**When** the baseline is approved
**Then** the scenarios and supported subset are specific enough to drive repeatable validation
**And** they can be used to reject unsupported overclaims

### Story 7.2: Validate Plugin Discovery And Host Behavior Against The Baseline

As a maintainer,
I want discovery, loading, and bounded editor-host behavior validated against representative plugins,
So that the supported playback path is credible beyond isolated implementation slices.

**Acceptance Criteria:**

**Given** the representative plugin baseline is defined
**When** discovery, loading, and bounded editor-host scenarios are exercised
**Then** Compose can show which validated plugins are usable through the supported path
**And** failures or unsupported behaviors are recorded without broadening the support claim

**Given** plugin-host behavior is part of the release story
**When** representative known-good plugins are tested
**Then** loading, basic usability, and bounded native editor-host behavior are evaluated as part of the support baseline
**And** unsupported plugins or interactions remain outside the MVP support claim unless explicitly validated

**Given** inventory and host validation are backend-owned concerns with user-facing consequences
**When** validation evidence is collected
**Then** it reuses the real product seams rather than an unrelated one-off path
**And** it is strong enough to support release review

### Story 7.3: Validate Transport Trust Against Representative Composition Scenarios

As a maintainer,
I want transport trust validated through representative score scenarios,
So that release decisions are based on composition-grade behavior rather than isolated happy paths.

**Acceptance Criteria:**

**Given** the supported playback path now includes real transport behavior
**When** representative play, pause, resume, seek, and edit-during-playback scenarios are exercised
**Then** the resulting validation shows whether playback remains coherent, dropout-free, and trustworthy
**And** the checks are strong enough to catch regressions that matter to composition work

**Given** transport trust is release-gating
**When** the validation set is run
**Then** the results distinguish between demo-capable behavior and composition-trustworthy behavior
**And** the product is not considered release-credible if the representative scenarios fail

**Given** transport validation must be explicit rather than anecdotal
**When** the scenarios are documented and executed
**Then** the scenario set is repeatable and specific
**And** it can be reused as an ongoing regression baseline

### Story 7.4: Publish The MVP Support Verdict And Boundaries

As a maintainer,
I want a clear support verdict and boundary statement for MVP playback,
So that the team knows what can be shipped, what is known-good, and what remains outside scope.

**Acceptance Criteria:**

**Given** the representative plugin, host, and transport validations have been completed
**When** the MVP support verdict is produced
**Then** it states whether Compose is trustworthy enough to support real composition work for the validated subset
**And** it names the actual scope of that supported subset

**Given** some behaviors or plugins may remain outside the validated baseline
**When** the support verdict is documented
**Then** unsupported or unproven areas are called out explicitly
**And** they are not described in a way that implies broader support than the evidence justifies

**Given** the release baseline affects future planning as well as release confidence
**When** the verdict is published
**Then** it feeds back into planning, QA, and product guidance
**And** later compatibility expansion can build from the validated boundary rather than re-litigating MVP scope
