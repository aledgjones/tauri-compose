---
stepsCompleted:
  - "step-01-validate-prerequisites"
  - "step-02-design-epics"
  - "step-03-create-stories"
  - "step-04-final-validation"
inputDocuments:
  - "/Users/aledjones/compose/_bmad-output/planning-artifacts/prd.md"
  - "/Users/aledjones/compose/_bmad-output/planning-artifacts/architecture.md"
  - "/Users/aledjones/compose/_bmad-output/planning-artifacts/ux-design-specification.md"
  - "/Users/aledjones/compose/_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md"
---

# compose - Epic Breakdown

This document provides the complete epic and story breakdown for compose, decomposing the requirements from the PRD, UX Design, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Composers can use Compose with a VST3-based playback environment on macOS.
FR2: Composers can access a discovered inventory of available VST3 plugins from within Compose.
FR3: Composers can use a cached plugin inventory so previously discovered plugins remain available without a full probe on every launch.
FR4: Composers can trigger a full plugin rescan when they need Compose to refresh the available VST3 inventory.
FR5: Compose can distinguish between discovered plugins that are currently usable and plugins that cannot currently be used.
FR6: Maintainers can validate plugin discovery and rescan behavior against representative installed plugin libraries.
FR7: Composers can assign a playable VST-backed sound source to score instruments.
FR8: Compose can attempt automatic playback assignment when a new score instrument is created.
FR9: Compose can leave an instrument unassigned when no suitable playback mapping is available.
FR10: Composers can identify which score instruments are assigned, unassigned, or no longer resolvable.
FR11: Composers can change the playback assignment for an instrument without restructuring the score.
FR12: Compose can support both single-part VST instrument workflows and multi-part or channel-addressed VST instrument workflows in MVP, including one plugin instance serving multiple score instruments where the plugin exposes that model.
FR13: Composers can load assigned VST3 instruments into the playback environment.
FR14: Compose can maintain usable plugin instance lifecycle behavior across playback sessions within the app.
FR15: Composers can open and use plugin-provided editor interfaces for supported VST3 instruments.
FR16: Compose can preserve playback usability even when some plugin editor interactions are unavailable or limited.
FR17: Maintainers can validate plugin-host behavior against a representative known-good plugin set.
FR18: Compose can translate written score instruments into playback-capable instrument assignments.
FR19: Composers can hear audible results from written score material without leaving the core score workflow.
FR20: Compose can support realistic playback through a defined expression-map model, with at least one active expression per instrument and a path to richer score-linked behavior.
FR21: Compose can deliver an initial expression-map capability in MVP that links expression behavior to score data where feasible, or proves the concept through a constrained workflow where full linkage would add disproportionate scope.
FR22: Compose can expand expression-map behavior later into a fuller score-linked system without replacing the underlying playback foundation.
FR23: Composers can start playback of their score through assigned VST instruments.
FR24: Composers can pause and resume playback without losing trust in playback state.
FR25: Composers can seek to different score positions and hear playback continue from the selected location.
FR26: Compose can keep playback state coherent across repeated play, pause, resume, and seek actions.
FR27: Compose can support transport-driven playback that composers can use as part of iterative writing and revision.
FR28: Compose can represent playback failure states in ways that allow users to recover without abandoning score work.
FR29: Composers can control basic per-instrument playback mix settings within Compose.
FR30: Compose can apply per-instrument playback control settings during score playback.
FR31: Compose can retain a basic mixer model that supports current playback needs while allowing later expansion.
FR32: Compose can persist playback-relevant project state for reopening later in the same product model.
FR33: Compose can restore instrument assignments when reopening a project if the required plugins are available.
FR34: Compose can degrade missing or unavailable plugin assignments to recoverable unassigned states when reopening a project.
FR35: Composers can continue working on a score even when some playback assignments cannot be restored.
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
NFR4: Cached plugin discovery, manual full rescan, assignment-to-first-audible-playback, and project reopen behavior must have explicit measurable performance targets.
NFR5: Rendering and other non-audio activity must not introduce audible glitches or playback instability.
NFR6: Playback state must remain coherent and recoverable without hangs, stuck notes, or corrupted session behavior.
NFR7: Project reopen must restore valid assignments correctly when plugins are available.
NFR8: Missing or unusable plugins must degrade to clear, recoverable unassigned states rather than breaking score access.
NFR9: The MVP must support a release-credible known-good subset of sensible, spec-aligned VST3 plugins on macOS.
NFR10: Plugin discovery and loading must operate within local desktop permissions and only access required local VST3 and project data paths.
NFR11: Playback-related project data must preserve integrity across save, close, and reopen flows.
NFR12: The system must integrate reliably with local VST3 installation paths, cached discovery, manual rescan, audio-device settings, score workflow assignment, project persistence, and native plugin editor hosting.
NFR13: Compatibility evaluation must be grounded in sensible adherence to the VST3 specification rather than blanket support for out-of-spec plugins.
NFR14: Validation must use explicit representative scenarios for transport, plugin compatibility, project reopen and recovery, cached discovery and rescan behavior, and realistic score playback conditions.

### Additional Requirements

- Preserve the existing brownfield Tauri + React + TypeScript + Vite foundation; implementation must extend the current app rather than reinitialize or replatform it.
- Treat Rust as the sole source of truth for plugin inventory, assignment state, playback preparation state, transport state, and persisted playback-domain state.
- Keep `.soloc` portable by storing project intent and assignment references only; machine-local plugin discovery and cache state must remain outside project files.
- Split playback architecture into a Rust control plane and a realtime-safe playback plane where the audio path consumes precomputed, lock-safe state only.
- Keep playback-critical communication backend-owned and command-oriented; avoid high-frequency event chatter on transport and playback-critical paths.
- Treat stable, jitter-free playback as a hard architectural and release gate for all implementation choices.
- Model missing plugin resolution as explicit, recoverable unassigned states rather than silent fallback or corrupted project state.
- Treat VST3 plugins as untrusted third-party code from a host-safety perspective and contain host failures without destabilizing playback or project data.
- Keep score-context playback relationships in contextual panels, and app-scoped playback assets or inventories in dedicated windows.
- Reuse backend seams for UI and MCP/automation flows rather than introducing parallel implementation paths.
- Keep validation and regression coverage close to the owning seam rather than creating broad duplicate harnesses.
- Formalize plugin inventory, identity, assignment, and recovery models before broader transport and workflow integration.
- Treat basic VST3 support as explicitly covering both single-part instrument plugins such as Spitfire-style setups and multi-part or channel-addressed plugins such as HALion Sonic, including the MVP case where one plugin instance serves multiple score instruments through explicit part or channel assignment, without implying full DAW-style routing or arbitrary shared-instance topology.
- Respect the project’s release and persistence discipline: any persisted-data change must assess migration impact and update pending migrations in `src-tauri/src/migrations/next.rs`.
- Treat the first implementation work as foundation-building on the existing app, not starter setup; Epic 1 Story 1 must not be project initialization.
- Treat native VST editor hosting as a narrow macOS-only seam first: one explicit editor-host path, one window label, one open command, and one close path before broader host abstraction.
- Keep native VST editor lifecycle backend-owned, with the frontend limited to a thin trigger through existing `actions.ts` and secondary-window patterns.
- Prefer direct AppKit subview attachment inside a dedicated Tauri-managed host window over HTML or React embedding for native plugin UI hosting.
- Keep the native editor host seam separate from transport, score-assignment lifecycle, preset management, automation, and broader production-host concerns until the proof is validated.
- Preserve the existing HTML plugin-settings workflow unless a deliberate incremental story chooses to replace or converge it; do not conflate metadata/audition UI with native editor hosting.
- Treat focus, resize, close, reopen, and teardown behavior of the native editor host as explicit acceptance concerns, not incidental implementation details.
- Require manual macOS runbook validation alongside unit/frontend coverage for the native editor host proof.
- Allow a fallback host-windowing route only if direct Tauri/AppKit parenting proves insufficient; record that as a tested architectural outcome rather than assuming it up front.
- Treat the remaining SCVI sampler/runtime code as temporary legacy scaffolding only. Epic 2 through Epic 6 must build clean VST-owned seams rather than extending `SamplerContext`, `soloc://state/sampler`, `actions.sampler.*`, or other SCVI-era contracts; the explicit end state is full removal of that remaining SCVI subsystem in Epic 7 Story 7.2 once the replacement VST path is complete.

### UX Design Requirements

UX-DR1: Instrument rows must expose a compact direct assignment control that shows the current playback configuration with minimal indirection.
UX-DR2: Instrument rows must expose a compact readiness or health signal so users can tell at a glance whether playback is ready, needs attention, or is unavailable.
UX-DR3: The expanded instrument panel must reflect the same assignment model as the row while adding metadata, richer status explanation, and deeper contextual controls.
UX-DR4: Compact score-context playback state must collapse into the three user-facing states Ready, Needs Attention, and Unavailable.
UX-DR5: Non-ready states must provide one primary contextual recovery action that opens the relevant plugin panel in the current score context.
UX-DR6: Plugin assignment must remain a score-first workflow that feels like choosing a usable sound for a musical part rather than configuring host topology.
UX-DR7: The common assignment and playback workflow must preserve a direct score-first mental model for MVP, while also allowing explicit slot or channel selection and clear cross-instrument linkage where a supported multi-part plugin serves multiple score instruments from one instance.
UX-DR8: Native plugin editors must be available as deliberate secondary handoff surfaces for plugin-specific control, not as the default answer to routine playback tasks.
UX-DR9: The system must provide explicit score-context entry points to open the native plugin editor from relevant playback surfaces.
UX-DR10: Expression maps must be represented as durable, reusable playback assets rather than transient properties of whichever score object is selected.
UX-DR11: The instrument panel must show which expression map is linked to the current instrument’s playback configuration and whether that linkage is missing or incomplete.
UX-DR12: Both instrument-driven and library-driven expression-map workflows must converge on one dedicated expression-map editor window.
UX-DR13: Full plugin discovery, inventory health, and organisation must live in a dedicated app-scoped Plugin Manager window.
UX-DR14: Contextual plugin pickers in instrument rows and panels must present a filtered, musician-friendly view of currently usable plugins for the current assignment task.
UX-DR15: Panels must act as score-context relational surfaces, while windows must own app-scoped inventories, reusable assets, and deep editors.
UX-DR16: Canonical app-scoped entry points for Plugin Manager and Expression Map Library must exist in the system app menu, with contextual launch paths also available from score workflows.
UX-DR17: Playback UI must preserve Compose’s existing custom design system, compact density, current blue-highlight language, and familiar panel/window grammar rather than introducing a parallel design language.
UX-DR18: Playback status must never rely on color alone and must combine text, iconography, and consistent placement for accessibility.

### FR Coverage Map

FR1: Epic 2 - Establish the VST3-based playback environment on macOS.
FR2: Epic 2 - Expose a discovered inventory of available VST3 plugins.
FR3: Epic 2 - Use a cached plugin inventory across launches.
FR4: Epic 2 - Support a manual full plugin rescan.
FR5: Epic 2 - Distinguish usable plugins from unavailable ones.
FR6: Epic 2 - Validate discovery and rescan against representative installed libraries.
FR7: Epic 3 - Assign a playable VST-backed sound source to score instruments.
FR8: Epic 3 - Attempt automatic assignment when a new score instrument is created.
FR9: Epic 3 - Leave instruments explicitly unassigned when no suitable mapping exists.
FR10: Epic 3 - Show whether instruments are assigned, unassigned, or unresolved.
FR11: Epic 3 - Reassign playback without restructuring the score.
FR12: Epic 3 - Support both single-part instrument plugins and multi-part or channel-addressed plugins in MVP, including one-instance multi-instrument assignment where the plugin exposes it.
FR13: Epic 3 - Load assigned VST3 instruments into the playback environment.
FR14: Epic 3 - Maintain usable per-instrument plugin lifecycle behavior across sessions.
FR15: Epic 2 - Open and use supported plugin-provided editor interfaces through the bounded native-host seam.
FR16: Epic 2 - Preserve playback usability when plugin editor interactions are limited or unavailable.
FR17: Epic 2 - Validate plugin-host behavior against a representative known-good plugin set.
FR18: Epic 4 - Translate written score instruments into playback-capable assignments during first audible playback.
FR19: Epic 4 - Hear written score material without leaving the core score workflow.
FR20: Epic 6 - Support a defined expression-map model with initial active-expression realism.
FR21: Epic 6 - Deliver an initial expression-map capability linked to score data where feasible or constrained where necessary.
FR22: Epic 6 - Preserve a path to richer score-linked expression behavior without replacing the playback foundation.
FR23: Epic 4 - Start score playback through assigned VST instruments.
FR24: Epic 5 - Pause and resume playback without losing trust in playback state.
FR25: Epic 5 - Seek to different score positions and continue playback coherently.
FR26: Epic 5 - Keep playback state coherent across repeated transport actions.
FR27: Epic 5 - Support transport-driven playback for iterative writing and revision.
FR28: Epic 5 - Represent playback failure states in recoverable ways.
FR29: Epic 4 - Control basic per-instrument playback mix settings.
FR30: Epic 4 - Apply per-instrument playback control settings during score playback.
FR31: Epic 4 - Retain a basic mixer model that supports current needs and later expansion.
FR32: Epic 5 - Persist playback-relevant project state for reopening.
FR33: Epic 5 - Restore instrument assignments on reopen when plugins are available.
FR34: Epic 5 - Degrade missing or unavailable plugin assignments to recoverable unassigned states on reopen.
FR35: Epic 5 - Continue score work even when some playback assignments cannot be restored.
FR36: Epic 1 - Retire SCVI from the active product and code path entirely.
FR37: Epic 5 - Evaluate playback against representative transport regression scenarios.
FR38: Epic 7 - Evaluate playback against representative real-world VST instrument libraries.
FR39: Epic 7 - Support a release-credible workflow for a known-good subset of sensible, spec-aligned VST3 plugins.
FR40: Epic 5 - Determine whether playback is trustworthy enough for real composition work.
FR41: Epic 5 - Support live score editing during playback as part of real composition workflow.

## Epic List

### Epic 1: Retire SCVI And Clear The Product For VST-First Playback

Compose removes SCVI-specific runtime, workflow, tooling, and guidance residue so users and maintainers can rely on one direct VST-first playback model without legacy ambiguity.
**FRs covered:** FR36

### Epic 2: Build a Usable VST Plugin Library

Users and maintainers can discover which VST3 plugins Compose can use, refresh that inventory, and exercise the bounded native editor-host seam on macOS.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR15, FR16, FR17

### Epic 3: Make Score Instruments Playable

Composers can assign playback to score instruments, including supported single-part and multi-part VST configurations, see when multiple score instruments share one plugin instance, see whether each part is ready or needs attention, and rely on stable plugin lifecycle behavior.
**FRs covered:** FR7, FR8, FR9, FR10, FR11, FR12, FR13, FR14

### Epic 4: Get From Written Score to First Audible Playback

Composers can hear written material through assigned VST instruments and shape basic playback levels inside Compose.
**FRs covered:** FR18, FR19, FR23, FR29, FR30, FR31

### Epic 5: Trust Playback During Real Composition

Composers can pause, resume, seek, edit during playback, reopen projects, and recover from missing plugins without losing trust in the score workflow.
**FRs covered:** FR24, FR25, FR26, FR27, FR28, FR32, FR33, FR34, FR35, FR37, FR40, FR41

### Epic 6: Make Playback Musically Expressive

Compose gains the first score-linked expression-map playback path so written articulations and dynamics produce inspectable, reusable realism behavior.
**FRs covered:** FR20, FR21, FR22

### Epic 7: Prove And Document The Supported MVP Playback Baseline

Compose validates the named plugin baseline, removes the last legacy SCVI subsystem once the VST replacement seams are complete, and documents the final MVP support posture the team can honestly ship.
**FRs covered:** FR36, FR38, FR39

## Epic 1: Retire SCVI And Clear The Product For VST-First Playback

Compose removes SCVI-specific runtime, workflow, tooling, and guidance residue so users and maintainers can rely on one direct VST-first playback model without legacy ambiguity.

### Story 1.1: Remove SCVI From the Active Playback Runtime

As a composer,
I want Compose to use only the intended VST-first playback runtime,
So that playback behavior is no longer split across legacy and current execution paths.

**FRs implemented:** FR36

**Acceptance Criteria:**

**Given** the app contains SCVI-specific runtime playback entry points or orchestration branches
**When** playback is initialized or requested through the supported product playback paths
**Then** those paths execute through the VST-first runtime only
**And** no SCVI-owned runtime branch remains reachable from supported playback entry points.

**Given** SCVI-specific runtime modules or orchestration branches are still wired into executable playback behavior
**When** this story is complete
**Then** those runtime connections are removed or disconnected from active product behavior
**And** the remaining playback ownership is unambiguously VST-first in code structure.

**Given** this story removes the active SCVI runtime path
**When** the updated codebase is built
**Then** the app still builds successfully
**And** the story does not introduce migration, compatibility-preservation, or legacy fallback scaffolding.

### Story 1.2: Remove SCVI From User-Facing Playback Workflow

As a composer,
I want the app to stop presenting SCVI-specific playback workflow elements,
So that Compose communicates one clear playback model without legacy confusion.

**FRs implemented:** FR36

**Acceptance Criteria:**

**Given** the app contains SCVI-specific playback labels, controls, entry points, settings, or workflow branches
**When** this story is complete
**Then** those user-facing SCVI-specific elements are removed or updated
**And** the UI no longer suggests SCVI is a supported playback path.

**Given** playback-related screens or flows previously assumed SCVI-specific concepts
**When** a user navigates those surfaces after implementation
**Then** they encounter only the remaining intended playback direction
**And** no broken or orphaned UI states remain.

**Given** a user inspects playback-related windows, panels, and settings surfaces
**When** this story is complete
**Then** SCVI does not appear as an active option, explanation, or fallback concept
**And** the remaining workflow language is internally consistent across those surfaces.

### Story 1.3: Remove SCVI Tooling and Build Workflow Residue

As a maintainer,
I want the repository to stop carrying SCVI-specific scripts and build-time support paths,
So that future playback work proceeds on one clear implementation path instead of splitting effort across live and dead tooling.

**FRs implemented:** FR36

**Acceptance Criteria:**

**Given** the repository contains SCVI-specific build, inspect, compression, or support scripts
**When** this story is complete
**Then** those SCVI-specific scripts are removed
**And** no active build or developer workflow depends on them.

**Given** package scripts or internal workflow entry points reference SCVI-specific tooling
**When** the cleanup is finished
**Then** those references are removed or updated
**And** the remaining workflow surface reflects the VST-first direction only.

**Given** a maintainer reviews the repository's active tooling after this story
**When** they inspect package scripts, developer entry points, and playback-related support scripts
**Then** they do not find SCVI-specific operational paths presented as current workflows
**And** the repo remains buildable with the remaining supported toolchain.

### Story 1.4: Remove SCVI References From Documentation and Guidance

As a maintainer,
I want the project documentation to stop describing SCVI as a current playback path,
So that contributors and reviewers see one unambiguous playback direction when working in the product.

**FRs implemented:** FR36

**Acceptance Criteria:**

**Given** project documentation or developer guidance still describes SCVI as an active playback path
**When** this story is complete
**Then** those references are removed or updated
**And** the docs describe the intended VST-first direction instead.

**Given** docs previously pointed contributors to SCVI-specific scripts or workflows
**When** documentation cleanup is complete
**Then** those instructions no longer appear as current guidance
**And** no broken references remain to retired SCVI tooling.

**Given** this epic is a clean retirement rather than a compatibility transition
**When** the documentation is updated
**Then** it does not introduce migration or coexistence guidance for SCVI
**And** it treats SCVI as retired from the current product direction.

### Story 1.5: Verify Clean SCVI Retirement Across Product and Repo Surfaces

As a maintainer,
I want to verify that SCVI has been cleanly retired from the active product and repository workflow,
So that future playback work can proceed on one unambiguous foundation.

**FRs implemented:** FR36

**Acceptance Criteria:**

**Given** SCVI runtime code, user-facing workflow residue, tooling, and docs have been cleaned up
**When** the agreed verification checks are run
**Then** repository search confirms no active SCVI-specific runtime, UI, tooling, or guidance path remains
**And** the remaining SCVI references, if any, are only intentionally historical or explanatory.

**Given** the cleanup has been completed
**When** the app build and relevant validation checks are run
**Then** the product remains buildable
**And** the checks do not rely on SCVI-specific behavior to pass.

**Given** maintainers inspect the repository for the current playback direction
**When** Epic 1 is complete
**Then** they can treat SCVI as retired from active product behavior and workflow
**And** future playback stories can proceed without dual-path assumptions, migration scaffolding, or legacy fallback requirements.

## Epic 2: Build a Usable VST Plugin Library

Users and maintainers can discover which VST3 plugins Compose can use, refresh that inventory, and exercise the bounded native editor-host seam on macOS.

### Story 2.1: Discover Installed VST3 Plugins Into a Local Inventory

As a composer,
I want Compose to discover installed VST3 plugins on macOS,
So that I can see which playback tools are available without configuring them manually.

**FRs implemented:** FR1, FR2, FR5

**Acceptance Criteria:**

**Given** Compose is launched on a supported macOS environment
**When** plugin discovery runs
**Then** the app identifies installed VST3 plugins from the supported local discovery paths
**And** stores the resulting inventory as machine-local application state rather than project data.

**Given** valid and invalid plugin candidates may both exist on disk
**When** discovery completes
**Then** Compose records enough information to distinguish discovered plugins from undiscoverable or unusable ones
**And** the discovery result is available to the app as inventory truth.

**Given** this is the first VST inventory story
**When** it is complete
**Then** no score-assignment workflow is required for the story to be valuable
**And** users or maintainers can confirm that Compose knows what plugins exist locally.

### Story 2.2: Reuse a Cached Plugin Inventory on Startup

As a composer,
I want Compose to reuse a cached plugin inventory on startup,
So that available plugins are shown quickly without forcing a full rescan every launch.

**FRs implemented:** FR3

**Acceptance Criteria:**

**Given** Compose has previously discovered a local VST3 inventory
**When** the app starts normally
**Then** it can load the cached plugin inventory for immediate use
**And** startup does not require a full reprobe to expose known plugins.

**Given** cached inventory data exists and the user has not requested a full rescan
**When** the app starts normally
**Then** Compose may perform a bounded file-lookup sanity check of cached bundle paths and related cached entry state
**And** it does not require a full discovery reprobe to keep ordinary startup visibility coherent.

**Given** some cached plugin entries may no longer be present or valid on disk
**When** the startup sanity check runs
**Then** Compose marks those entries consistently as unavailable, stale, or otherwise non-usable according to the current inventory model
**And** cached startup behavior remains consistent with the current local environment.

**Given** no cached inventory exists yet
**When** the app starts normally
**Then** Compose performs initial discovery so plugin surfaces are not left permanently empty
**And** the resulting inventory becomes the machine-local cache for later startups.

**Given** plugin inventory is machine-local state
**When** cache reuse is implemented
**Then** that cache remains outside `.soloc` project data
**And** project persistence is not used as a substitute for local plugin discovery state.

### Story 2.3: Expose Plugin Inventory and Availability in a Dedicated Plugin Manager

As a maintainer,
I want a dedicated Plugin Manager window that shows the discovered plugin library and availability state,
So that I can inspect what Compose can use and what needs attention.

**FRs implemented:** FR2, FR5

**Acceptance Criteria:**

**Given** Compose has a discovered or cached plugin inventory
**When** the user opens the Plugin Manager
**Then** they can see the plugin inventory in a dedicated app-scoped window
**And** that window is clearly separate from score-context assignment surfaces.

**Given** some plugins may be usable while others are unavailable or problematic
**When** the inventory is displayed
**Then** the Plugin Manager communicates plugin availability state clearly
**And** status is not conveyed by color alone.

**Given** the Plugin Manager is an app-scoped surface
**When** the user reaches it from canonical app entry points or contextual launch paths
**Then** the experience remains consistent with the project's panel-versus-window scope model
**And** it uses the existing Compose design system.

### Story 2.4: Support Manual Full Plugin Rescan

As a maintainer,
I want to trigger a full VST3 rescan on demand,
So that Compose can refresh its plugin inventory when the local environment changes.

**FRs implemented:** FR4, FR6

**Acceptance Criteria:**

**Given** the local plugin environment changes after cached inventory was created
**When** the user requests a full rescan
**Then** Compose performs a full refresh of the supported VST3 discovery paths
**And** updates the inventory state used by the app.

**Given** the inventory is shown in the Plugin Manager
**When** a full rescan completes
**Then** the updated discovery results are reflected in that inventory view
**And** the user can tell that the inventory has been refreshed.

**Given** a full rescan is a distinct workflow from cached startup reuse
**When** this story is complete
**Then** both behaviors coexist cleanly
**And** the app does not require a full rescan for ordinary startup visibility.

### Story 2.5: Open a Bounded Native Plugin Editor Host for Supported Plugins

As a maintainer,
I want to open a supported plugin's native editor through a dedicated host window,
So that Compose can prove the macOS editor-host seam without mixing it into the main UI model.

**FRs implemented:** FR15, FR16

**Acceptance Criteria:**

**Given** a known-good supported plugin is available in the inventory
**When** the user requests to open its native editor
**Then** Compose opens a dedicated host window for that plugin editor
**And** the editor is hosted through the bounded native window seam rather than HTML embedding.

**Given** the native editor seam is intentionally narrow
**When** this story is implemented
**Then** the host path remains backend-owned and explicit
**And** the frontend acts only as a thin trigger through the established command and window patterns.

**Given** native editor hosting is not guaranteed for every plugin
**When** the host path cannot provide a usable editor interaction
**Then** Compose fails gracefully without destabilizing the rest of the app
**And** playback usability outside that editor window is preserved.

### Story 2.6: Stabilize Native Editor Host Lifecycle Within a Session

As a maintainer,
I want the bounded editor-host seam to survive focus, resize, close, and reopen behavior,
So that we can treat it as a usable platform seam rather than a one-shot proof.

**FRs implemented:** FR16

**Acceptance Criteria:**

**Given** a supported plugin editor has been opened in the dedicated host window
**When** the user resizes the window, changes focus, closes it, and reopens it in the same session
**Then** the host window continues to attach the editor correctly through each lifecycle step
**And** the app is not left with an orphaned window, crashed host path, or lost editor ownership.

**Given** the native editor host has already been opened successfully in the current session
**When** the user closes the host window and later opens that same supported plugin editor again
**Then** Compose reuses the bounded host flow coherently for the new open request
**And** the reopened editor path does not require an app restart or manual cleanup to work.

**Given** Epic 2 is proving a bounded macOS editor-host seam rather than a full production host abstraction
**When** this story is complete
**Then** lifecycle behavior is explicit and reviewable for the supported session interactions
**And** later playback or assignment concerns remain outside this story's scope.

### Story 2.7: Validate Editor Host Failure Handling Against a Known-Good Plugin Subset

As a maintainer,
I want invalid-target handling and host validation checked against a representative known-good plugin subset,
So that Compose can claim a bounded credible editor-host path without overclaiming broad compatibility.

**FRs implemented:** FR16, FR17

**Acceptance Criteria:**

**Given** invalid, unavailable, or unsupported plugin targets can be requested
**When** the user triggers the editor host for one of those targets
**Then** Compose reports failure gracefully
**And** it does not leave behind an orphaned host window or unstable state.

**Given** Epic 2 must prove a usable plugin library rather than broad arbitrary compatibility
**When** plugin-host validation is run
**Then** it is framed around a representative known-good subset of sensible, spec-aligned plugins
**And** the result does not overclaim universal editor compatibility.

**Given** native editor hosting may vary across real-world plugins
**When** maintainers review the validation outcome for the known-good subset
**Then** they can identify which tested plugins support the bounded editor-host path credibly
**And** any limitations, exclusions, or caveats are explicit rather than implicit.

## Epic 3: Make Score Instruments Playable

Composers can assign playback to score instruments through both single-part and one-instance multi-instrument plugin workflows, see whether each part is ready or needs attention, and rely on stable plugin lifecycle behavior.

### Story 3.1: Represent Playback Assignment State on Score Instruments

As a composer,
I want score instruments to carry explicit playback assignment state,
So that Compose can tell me whether each instrument is ready, needs attention, or is unassigned.

**FRs implemented:** FR7, FR9, FR10, FR12

**Acceptance Criteria:**

**Given** a score contains instruments that may or may not have playback assigned
**When** Compose resolves playback state for those instruments
**Then** each instrument has explicit assignment state owned by the backend
**And** that state is available to score-context UI surfaces.

**Given** some instruments may not yet have a valid playback mapping
**When** their state is resolved
**Then** they are represented explicitly as unassigned or attention-needed
**And** Compose does not silently substitute hidden fallback behavior.

**Given** this is the first score-assignment story
**When** it is complete
**Then** the assignment model supports both direct single-part plugin assignments and explicit multi-part or channel-addressed assignments
**And** it can represent the MVP case where multiple score instruments intentionally share one plugin instance.

### Story 3.2: Show Assignment Readiness in Instrument Rows and Panels

As a composer,
I want to see playback readiness directly on score instruments,
So that I can tell at a glance which parts are playable and which need action.

**FRs implemented:** FR10

**Acceptance Criteria:**

**Given** score instruments have resolved playback assignment state
**When** they are displayed in relevant score-context surfaces
**Then** instrument rows show compact readiness or health state
**And** the expanded instrument panel shows richer contextual playback information.

**Given** playback state can be Ready, Needs Attention, or Unavailable
**When** those states are shown in the UI
**Then** they are legible without relying on color alone
**And** the presentation remains consistent with the existing Compose design system.

**Given** an instrument is not ready for playback
**When** a user inspects that instrument's score-context UI
**Then** they are given one primary contextual recovery path
**And** the product does not force them into an unrelated app-scoped workflow by default.

### Story 3.3: Manually Assign a Plugin to a Score Instrument

As a composer,
I want to assign an available single-part plugin to a score instrument from the score workflow,
So that I can prepare that instrument for playback quickly without leaving my composition context.

**FRs implemented:** FR7, FR11

**Acceptance Criteria:**

**Given** a score instrument is selected and a usable plugin inventory exists
**When** the user chooses a plugin from the contextual assignment control
**Then** that plugin is assigned to the selected score instrument
**And** the assignment is stored as backend-owned playback intent.

**Given** the score-context picker is used for assignment
**When** the user opens it
**Then** it presents a filtered, musician-friendly view of currently usable plugins
**And** it does not require the user to manage the full plugin inventory window just to make an assignment.

**Given** a manual assignment is completed successfully
**When** the instrument's playback state is refreshed
**Then** its readiness state reflects the new assignment
**And** the instrument becomes eligible for later playback stories.

### Story 3.4: Select A Part Or Channel When Assigning A Compatible Multi-Part Plugin

As a composer,
I want to choose an explicit part or channel when assigning a compatible multi-part plugin,
So that the playback target for my instrument stays direct and unambiguous.

**FRs implemented:** FR12

**Acceptance Criteria:**

**Given** a selected plugin exposes multiple supported parts, slots, or channel-addressed instrument targets
**When** the user assigns that plugin from the score workflow
**Then** Compose can capture the chosen part or channel as part of each instrument's playback intent
**And** the selected target is visible in the instrument's score-context playback state.

**Given** a requested part or channel is unavailable, invalid, or already occupied by a conflicting assignment rule
**When** the user attempts to complete the assignment
**Then** Compose prevents ambiguous assignment and surfaces a clear recoverable outcome
**And** it does not silently guess an alternative part, channel, or instance.

**Given** this MVP support must stay score-first rather than drifting into DAW routing design
**When** the target-selection workflow is implemented
**Then** it exposes only the explicit plugin part or channel choices needed for the supported workflow
**And** it does not expand into arbitrary bus routing or generic host-topology configuration.

### Story 3.5: Reuse A Compatible Multi-Part Plugin Instance Across Multiple Score Instruments

As a composer,
I want compatible score instruments to share one multi-part plugin instance through explicit target choices,
So that HALion-style playback setups work as a first-class MVP workflow instead of a later extension.

**FRs implemented:** FR7, FR12

**Acceptance Criteria:**

**Given** one score instrument is already using a compatible multi-part plugin instance
**When** the user assigns another score instrument to that same plugin through a different supported part or channel
**Then** Compose reuses the existing compatible plugin instance rather than silently creating a duplicate instance
**And** the resulting shared-instance relationship is explicit in backend-owned assignment state.

**Given** more than one score instrument can share the same compatible plugin instance
**When** those assignments are shown in score-context surfaces
**Then** the UI makes the shared-instance linkage legible
**And** users can still understand each instrument's own readiness and target part or channel.

**Given** the compatible plugin instance later becomes unavailable or invalid
**When** Compose refreshes assignment state
**Then** each affected score instrument moves into an explicit recoverable state
**And** the product preserves the shared-instance relationship as user intent rather than guessing a replacement topology.

### Story 3.6: Attempt Automatic Assignment for New Score Instruments

As a composer,
I want Compose to attempt a sensible playback assignment when I create a new score instrument,
So that I can get to playable setup faster without manual work every time.

**FRs implemented:** FR8, FR9

**Acceptance Criteria:**

**Given** a user creates a new score instrument
**When** Compose has sufficient local inventory or mapping information to make an assignment attempt
**Then** it attempts to assign a suitable playback source automatically
**And** the result is reflected in the instrument's assignment state.

**Given** some plugins may require an explicit slot or channel decision to be unambiguous
**When** automatic assignment cannot choose that target safely
**Then** Compose leaves the instrument in an explicit recoverable state instead of guessing hidden part, channel, or shared-instance routing
**And** the user can complete the assignment manually.

**Given** no suitable automatic assignment is available
**When** the new instrument is created
**Then** the instrument remains explicitly unassigned
**And** the user can see that further action is needed.

**Given** automatic assignment is a convenience rather than hidden magic
**When** it succeeds or fails
**Then** the resulting state is visible in the score workflow
**And** the user can still manually change the assignment afterward.

### Story 3.7: Reassign and Recover Instrument Playback Without Restructuring the Score

As a composer,
I want to change or repair an instrument's playback assignment directly,
So that I can keep working without rebuilding score structure when playback setup changes.

**FRs implemented:** FR11

**Acceptance Criteria:**

**Given** a score instrument already has an assignment or attention-needed state
**When** the user selects a different usable plugin from the contextual assignment workflow
**Then** the assignment is updated for that instrument
**And** the score structure itself does not need to change.

**Given** an instrument is unresolved or needs attention
**When** the user follows the primary contextual recovery action
**Then** they can reach the relevant reassignment path quickly
**And** the workflow remains anchored in the score context.

**Given** reassignment occurs
**When** the new assignment is resolved
**Then** readiness state updates accordingly
**And** prior unresolved state does not linger as stale UI.

### Story 3.8: Activate Assigned Plugins for Single-Part and Shared Multi-Part Playback

As a maintainer,
I want valid score-instrument assignments to activate usable plugin targets within the current app session,
So that assigned instruments are actually ready to participate in later playback work.

**FRs implemented:** FR12, FR13

**Acceptance Criteria:**

**Given** score instruments have valid playback assignments
**When** those assignments are activated for use within the app session
**Then** Compose loads or attaches the plugin targets needed for those assignments
**And** each assigned instrument resolves to a usable playback target owned by backend state.

**Given** the MVP must support both single-part and multi-part plugin workflows
**When** assignment activation is implemented for this epic
**Then** it works for both supported single-part assignments and explicit multi-part or channel-addressed shared-instance assignments
**And** it does not require broader DAW-style routing infrastructure beyond those supported MVP patterns.

**Given** multiple score instruments may intentionally share one compatible multi-part plugin instance
**When** those assignments are activated
**Then** Compose reuses that shared instance coherently for the participating instruments
**And** each instrument retains its own explicit target part or channel in assignment state.

### Story 3.9: Preserve Assignment Lifecycle and Recoverability Across the App Session

As a maintainer,
I want activated assignments to remain stable and recoverable across the app session,
So that playback setup behaves like ongoing usable state rather than a one-shot activation.

**FRs implemented:** FR12, FR14

**Acceptance Criteria:**

**Given** assigned plugin targets have been activated in the current app session
**When** the user continues working with the score and related playback setup
**Then** assignment state remains coherent from the user's perspective
**And** the assignment does not behave like a one-shot setup action that is lost after first use.

**Given** lifecycle behavior must remain valid for both supported single-part and shared multi-part assignments
**When** those assignments continue through ordinary in-app use
**Then** Compose preserves stable lifecycle behavior for both patterns
**And** it keeps the supported shared-instance relationship explicit rather than silently rewriting topology.

**Given** lifecycle issues or unavailable assignments can occur during the app session
**When** they affect an assigned instrument or shared plugin instance
**Then** the affected instruments transition into an explicit attention-needed or unavailable state
**And** Compose preserves recoverability instead of failing invisibly.

## Epic 4: Get From Written Score to First Audible Playback

Composers can hear written material through assigned VST instruments and shape basic playback levels inside Compose.

### Story 4.1: Translate Assigned Score Instruments Into Playback-Ready Material

As a composer,
I want Compose to turn assigned score instruments and written material into playback-ready data,
So that the system can prepare my score to be heard.

**FRs implemented:** FR18

**Acceptance Criteria:**

**Given** a score contains written material on instruments with valid playback assignments
**When** Compose prepares that score for playback
**Then** it translates those assigned score instruments into playback-ready material
**And** the preparation is owned by backend playback logic rather than frontend inference.

**Given** some score instruments may still be unresolved or unassigned
**When** playback preparation occurs
**Then** only valid playable material is prepared for audible use
**And** unresolved instruments remain explicitly non-playable rather than failing invisibly.

**Given** this is the first audible-playback epic
**When** this story is complete
**Then** the system can prepare assigned score content for playback
**And** it does not require advanced transport behavior such as seek or resume to be valuable.

### Story 4.2: Start Audible Playback of Written Score Material

As a composer,
I want to start playback and hear my written score through assigned VST instruments,
So that playback becomes part of my composition workflow.

**FRs implemented:** FR19, FR23

**Acceptance Criteria:**

**Given** playback-ready score material exists for assigned instruments
**When** the user starts playback
**Then** Compose produces audible playback through those assigned VST instruments
**And** the user hears written score material without leaving the core workflow.

**Given** some instruments may be assigned and others unresolved
**When** playback starts
**Then** the valid assigned instruments produce audible output
**And** unresolved instruments do not prevent the rest of the score from being heard where playback is otherwise valid.

**Given** the goal of this story is first audible playback
**When** it is complete
**Then** users can verify that written material is being rendered audibly through the assigned path
**And** later transport-trust concerns remain out of scope for this story.

### Story 4.3: Surface Basic Per-Instrument Mix Controls

As a composer,
I want basic per-instrument mix controls inside Compose,
So that I can quickly shape what I hear while evaluating playback.

**FRs implemented:** FR29, FR31

**Acceptance Criteria:**

**Given** score instruments participate in audible playback
**When** the user opens the relevant playback controls
**Then** basic per-instrument mix settings are available inside Compose
**And** the controls remain aligned with the product's existing control model rather than introducing DAW-style routing complexity.

**Given** multiple playable instruments exist
**When** mix controls are displayed
**Then** the user can understand which controls belong to which instrument
**And** the UI remains compact and legible.

**Given** this epic is scoped to basic mixer behavior
**When** the controls are implemented
**Then** they support current playback needs
**And** they do not attempt to solve advanced routing or broader mixer expansion.

### Story 4.4: Apply Basic Mix Settings During Playback

As a composer,
I want per-instrument mix settings to affect playback output,
So that the controls I adjust actually shape what I hear.

**FRs implemented:** FR30

**Acceptance Criteria:**

**Given** a user has adjusted basic per-instrument mix settings
**When** audible playback runs
**Then** those settings are applied to the corresponding instrument output
**And** the audible result reflects the user's adjustments.

**Given** multiple instruments are playing
**When** one instrument's mix settings are changed
**Then** the applied change affects that instrument's playback behavior
**And** it does not require unrelated reassignment or score restructuring.

**Given** this is still the first-audible-playback epic
**When** mix application is implemented
**Then** the result remains within a basic mixer model
**And** it does not imply completion of later transport-trust or realism work.

### Story 4.5: Keep the Basic Mixer Model Stable and Extendable

As a maintainer,
I want the initial mixer behavior to be implemented as a stable basic model,
So that current playback needs are met without blocking later expansion.

**FRs implemented:** FR31

**Acceptance Criteria:**

**Given** basic per-instrument mix controls and application behavior exist
**When** the mixer model is reviewed after implementation
**Then** it supports the current MVP playback need for basic per-instrument mix control during audible playback
**And** it does not depend on future advanced mixer capabilities such as broader routing or grouped mixer features to function.

**Given** future playback work may require richer mixer behavior later
**When** this story is complete
**Then** the initial mixer model remains intentionally basic with one clear per-instrument control path
**And** current functionality does not need to be replaced to add later mixer growth on top of it.

**Given** Epic 4 is about first audible playback rather than transport trust
**When** the mixer model is finalized for this epic
**Then** it remains scoped to first-playback usability
**And** it does not accumulate unrelated transport-state, reopen, or realism concerns.

## Epic 5: Trust Playback During Real Composition

Composers can pause, resume, seek, reopen projects, edit during playback, and recover from missing plugins without losing trust in the score workflow.

### Story 5.1: Maintain Coherent Pause and Resume Behavior

As a composer,
I want to pause and resume playback without losing trust in playback state,
So that I can audition and revise music without restarting the whole workflow.

**FRs implemented:** FR24, FR26

**Acceptance Criteria:**

**Given** score playback is running through assigned VST instruments
**When** the user pauses playback
**Then** Compose stops playback coherently
**And** it does not leave the session in a broken or obviously invalid state.

**Given** playback has been paused
**When** the user resumes playback
**Then** Compose resumes from the intended point using coherent playback state
**And** the user can continue auditioning without treating playback as reset-only.

**Given** pause and resume may occur repeatedly during composition
**When** this story is complete
**Then** repeated pause/resume cycles can run through the representative transport regression scenarios without hangs, stuck notes, or transport state corruption
**And** later seek and edit-during-playback concerns remain separable from this story.

### Story 5.2: Support Trustworthy Seek Behavior

As a composer,
I want to seek to another score position and continue playback from there,
So that I can quickly audition the part of the score I am actively revising.

**FRs implemented:** FR25, FR26

**Acceptance Criteria:**

**Given** playback-ready score material exists
**When** the user seeks to a different score position and starts or continues playback
**Then** playback begins from the selected location
**And** the result does not rely on stale pre-seek playback state, stuck notes, or an app restart to recover.

**Given** playback state may already be active before the seek
**When** the seek is processed
**Then** Compose updates playback state consistently for the new location
**And** it does not rely on stale pre-seek state.

**Given** seek is a trust-critical interaction
**When** this story is complete
**Then** the user can use seek as part of ordinary revision workflow
**And** the behavior does not depend on restarting the app or rebuilding the project state.

### Story 5.3: Keep Transport State Coherent Across Repeated Playback Actions

As a composer,
I want repeated play, pause, resume, and seek actions to remain coherent,
So that playback feels dependable during real composition sessions.

**FRs implemented:** FR26, FR27

**Acceptance Criteria:**

**Given** the user performs repeated transport actions during a session
**When** Compose updates transport state across those actions
**Then** the resulting playback state remains coherent
**And** the workflow does not degrade into hangs, obviously stale state, or unusable playback transitions.

**Given** transport behavior is being used iteratively rather than once
**When** this story is complete
**Then** the user can perform ordinary playback navigation repeatedly
**And** the app continues to accept further transport actions without hangs, stale state, or a mandatory restart.

**Given** this story is about transport coherence as a system property
**When** it is implemented
**Then** it builds on pause/resume and seek without expanding into reopen or realism scope
**And** it provides a stable base for later recovery and validation stories.

### Story 5.4: Allow Score Editing During Playback With a Clear Update Rule

As a composer,
I want to continue editing and updating the score during playback,
So that playback supports active composition instead of forcing me to stop every time I make a change.

**FRs implemented:** FR27, FR41

**Acceptance Criteria:**

**Given** playback is currently running
**When** the user edits supported score content during playback
**Then** the app allows the edit without collapsing into a blocked or invalid state
**And** playback follows the documented immediate-or-deferred update rule without crashes, blocked editing, or ambiguous playback state.

**Given** some score edits may be safe to reflect immediately while others may need a bounded defer rule
**When** edits occur during playback
**Then** Compose applies those edits according to an explicit, consistent rule
**And** the user is not left guessing whether playback reflects the updated score.

**Given** this is the first iteration of edit-during-playback support
**When** this story is complete
**Then** the feature is scoped to trustworthy behavior rather than universal instant propagation
**And** the limitations of what updates apply immediately versus later are clear and testable.

### Story 5.5: Recover Gracefully From Playback Failure States

As a composer,
I want playback failure states to be recoverable,
So that I can continue working instead of abandoning the score when something goes wrong.

**FRs implemented:** FR28

**Acceptance Criteria:**

**Given** playback encounters a failure or unusable state during composition workflow
**When** that failure is surfaced
**Then** Compose presents it as a recoverable state
**And** the user is not forced into a broken session with no clear next action.

**Given** an instrument or playback path becomes invalid during use
**When** the user inspects the relevant score-context surface
**Then** they can understand that attention is needed
**And** they are given one primary contextual recovery action leading to the relevant score-context panel or reassignment flow.

**Given** failures are part of a real composition workflow rather than just QA scenarios
**When** this story is complete
**Then** users can continue score work after failure conditions where recovery is possible
**And** the app does not treat ordinary failure handling as session-ending.

### Story 5.6: Restore Valid Playback Assignments On Reopen

As a composer,
I want valid playback assignments to restore when reopening a score,
So that I can resume work without rebuilding working playback setup every session.

**FRs implemented:** FR32, FR33

**Acceptance Criteria:**

**Given** a project with valid playback-relevant state is reopened on a machine where the required plugins are available
**When** Compose restores the project
**Then** playback assignments are restored coherently
**And** the user can continue from the reopened state without manual rebuild of setup.

**Given** project reopen is part of trust in ordinary composition workflow
**When** this story is complete
**Then** reopen behavior preserves continuity where possible
**And** valid restored assignments remain aligned with the same persisted project intent that was saved before closing.

### Story 5.7: Degrade Missing Playback Assignments Safely On Reopen

As a composer,
I want missing playback assignments to degrade safely when reopening a score,
So that I can keep working even when the current machine cannot restore every plugin.

**FRs implemented:** FR34, FR35

**Acceptance Criteria:**

**Given** a reopened project references plugins that are now missing or unavailable
**When** Compose resolves playback state on reopen
**Then** those assignments degrade to explicit recoverable unassigned or attention-needed states
**And** the score itself remains usable.

**Given** recovery from missing plugins is part of ordinary composition trust
**When** the unresolved state is shown after reopen
**Then** the user can tell which instrument needs attention
**And** the original assignment intent is preserved rather than silently remapped.

### Story 5.8: Validate Trusted Playback Against Representative Transport Scenarios

As a maintainer,
I want representative transport regression scenarios for trusted composition playback,
So that we can determine whether playback is actually dependable enough for real work.

**FRs implemented:** FR37, FR40

**Acceptance Criteria:**

**Given** pause, resume, seek, reopen, recovery, and edit-during-playback behavior have been implemented
**When** representative transport regression scenarios are defined and run
**Then** the scenario set explicitly covers those representative workflow behaviors
**And** each scenario records explicit pass/fail outcomes rather than relying on vague subjective confidence.

**Given** trusted playback is a release-critical product property
**When** Epic 5 is complete
**Then** maintainers can determine whether Compose supports real composition work rather than demo-only playback based on the recorded scenario results
**And** the validation surface makes that decision traceable to the representative transport scenarios.

**Given** Epic 5 is still distinct from Epic 7 release-hardening scope
**When** validation is implemented here
**Then** it focuses on representative transport and workflow trust scenarios
**And** it does not broaden into full compatibility or realism certification.

## Epic 6: Make Playback Musically Expressive

Compose gains the first score-linked expression-map playback path so written articulations and dynamics produce inspectable, reusable realism behavior.

### Story 6.1: Introduce Durable Expression Maps as Reusable Playback Assets

As a composer,
I want expression maps to exist as durable reusable playback assets,
So that realism behavior is defined in a stable model instead of being scattered across instrument or note state.

**FRs implemented:** FR20, FR21, FR22

**Acceptance Criteria:**

**Given** Compose needs an initial realism model that can grow later without replacement
**When** the first expression-map foundation is implemented
**Then** expression maps are stored as durable reusable playback assets
**And** the model supports at least named articulation-driven expression slots, one bounded dynamics-control path, and explicit switching actions for both keyswitch-note and controller-driven articulation changes.

**Given** expression maps are app-scoped assets rather than transient score selection state
**When** this foundation story is complete
**Then** their definitions are owned as reusable playback assets rather than one-off score-panel state
**And** later score workflows can reference those assets without redefining the model.

**Given** Epic 6 must stay incremental rather than opening full realism scope at once
**When** this first model is defined
**Then** it establishes one concrete expression-map shape for MVP
**And** it leaves room for richer later behavior without requiring a second foundation rewrite.

### Story 6.2: Provide One Dedicated Expression-Map Editor Surface

As a composer,
I want one dedicated editor surface for expression maps,
So that deeper realism configuration is managed in a single app-scoped workflow instead of scattered across score-context panels.

**FRs implemented:** FR21, FR22

**Acceptance Criteria:**

**Given** expression maps are app-scoped reusable playback assets
**When** a user or maintainer opens the expression-map editing workflow
**Then** Compose provides one dedicated expression-map editor surface
**And** that editor is the canonical place to inspect and change map definitions.

**Given** expression-map workflows may be launched from instrument context or library context
**When** the editor is opened from those entry points
**Then** both paths converge on the same dedicated editor surface
**And** the product does not create parallel expression-map editing experiences.

**Given** Epic 6 is adding a deep editor without replacing score-context panels
**When** this story is complete
**Then** score-context surfaces can link into the dedicated editor for deeper work
**And** they do not duplicate full expression-map editing responsibility locally.

### Story 6.3: Link Instrument Playback Configurations to Expression Maps and Surface Current State

As a composer,
I want each playable score instrument to show which expression map it uses and whether that linkage is healthy,
So that realism behavior is understandable from the score workflow instead of hidden in a detached setup tool.

**FRs implemented:** FR20, FR21

**Acceptance Criteria:**

**Given** an instrument has a playback assignment that can use realism behavior
**When** the user inspects the instrument panel
**Then** Compose shows which expression map is linked to that instrument
**And** whether that linkage is present, missing, or incomplete.

**Given** the first inspectable realism surface should live in score context
**When** the instrument panel is displayed
**Then** it exposes the current resolved expression state in a compact understandable form
**And** it provides a direct path into the dedicated expression-map editor.

**Given** expression maps are reusable assets and not one-off panel state
**When** this story is complete
**Then** the score-context panel explains linkage and current behavior
**And** deeper editing remains owned by the dedicated expression-map editor surface.

### Story 6.4: Resolve Written Articulations Into Concrete Playback Expression Changes

As a composer,
I want supported score articulations to change playback expression in a defined way,
So that written musical differences produce audible realism rather than identical playback.

**FRs implemented:** FR20, FR21

**Acceptance Criteria:**

**Given** a playable instrument has a linked expression map with supported articulation slots
**When** the score contains supported written articulations such as sustain, staccato, or accent
**Then** Compose resolves those markings into the mapped playback expression behavior
**And** the active articulation-related expression can be inspected from the score-context workflow.

**Given** supported plugins may expect different articulation-switch mechanisms
**When** an expression map is configured for a supported articulation
**Then** Compose can drive that articulation through either a keyswitch-note action or a controller-driven switching action
**And** the chosen switching path is explicit in the map rather than hidden as plugin-specific magic.

**Given** the MVP must prove one concrete score-linked realism path rather than vague future potential
**When** this story is complete
**Then** articulation handling is based on explicit supported score inputs and explicit mapped outcomes
**And** it does not rely on manual per-pass reconfiguration to demonstrate the behavior.

**Given** some articulations may fall outside the first supported slice
**When** unsupported articulation semantics are encountered
**Then** Compose degrades them predictably without corrupting playback state
**And** the user is not misled into thinking unsupported realism behavior is active.

### Story 6.5: Resolve Written Dynamics Into a Bounded Playback Dynamics Path

As a composer,
I want supported score dynamics to influence playback through one defined control path,
So that louder and softer writing affects the audible result in a credible and inspectable way.

**FRs implemented:** FR20, FR21

**Acceptance Criteria:**

**Given** a playable instrument has a linked expression map with the first supported dynamics path
**When** the score contains supported written dynamics within the defined MVP slice
**Then** Compose applies those dynamics through one bounded playback-control route
**And** the resolved dynamics behavior is inspectable alongside the linked expression map state.

**Given** Epic 6 should remain reviewable and implementation-ready
**When** dynamics-linked realism is implemented
**Then** it names one concrete supported score-to-playback path for MVP
**And** it does not broaden into arbitrary plugin-specific automation coverage.

**Given** the first realism slice already includes articulation-linked behavior
**When** dynamics are added
**Then** both articulation and dynamics work through the same underlying expression-map model
**And** later realism expansion can extend that shared model instead of replacing it.

## Epic 7: Prove And Document The Supported MVP Playback Baseline

Compose validates the named plugin baseline, removes the last legacy SCVI subsystem once the VST replacement seams are complete, and documents the final MVP support posture the team can honestly ship.

### Story 7.1: Validate Realism and Compatibility Against Named Plugin Baselines

As a maintainer,
I want realism and playback behavior validated against named real-world plugin libraries,
So that release confidence is grounded in explicit evidence rather than anecdotal auditioning.

**FRs implemented:** FR38, FR39

**Acceptance Criteria:**

**Given** Epic 6 introduces the first score-linked realism behavior
**When** representative real-world validation is defined
**Then** it names an explicit plugin baseline including BBC Spitfire libraries, HALion Sonic, and the user's existing library
**And** the validation is framed around sensible spec-aligned compatibility rather than blanket support claims.

**Given** realism and compatibility can vary across plugins
**When** the named baseline is exercised
**Then** maintainers can identify which plugins support the intended playback workflow based on recorded validation results
**And** which plugins expose limitations, caveats, or unsupported behavior, including any channel-oriented or multi-instrument constraints surfaced by HALion Sonic.

**Given** the MVP assignment model explicitly includes one-instance multi-instrument support for compatible multi-part plugins
**When** HALion Sonic or another multi-part plugin is validated
**Then** the matrix makes clear how multiple score instruments can share that instance through explicit part or channel assignment
**And** it distinguishes that supported MVP pattern from broader unsupported DAW-style multi-routing behavior.

**Given** basic VST3 support now explicitly includes both Spitfire-style single-part plugins and HALion-style multi-part plugins
**When** the named baseline is exercised
**Then** the validation proves both configurations are supported as first-class baseline cases
**And** any difference in supported slot, part, channel, or shared-instance behavior is documented explicitly rather than left implicit.

**Given** this story is a release-hardening proof rather than just a developer smoke test
**When** validation is complete
**Then** the evidence covers both ordinary playback behavior and the first articulation-plus-dynamics realism path
**And** it is explicit enough to guide an MVP support baseline and its documented limits.

### Story 7.2: Remove the Remaining SCVI Legacy System After VST Replacement Is Complete

As a maintainer,
I want the remaining SCVI legacy sampler/runtime system removed once the VST replacement seams are complete,
So that Compose ships on one playback architecture instead of carrying a dormant second system.

**FRs implemented:** FR36

**Acceptance Criteria:**

**Given** Epics 2 through 6 and Epic 7.1 have established and validated the VST inventory, assignment, playback, recovery, and realism seams needed for MVP
**When** final legacy cleanup is performed
**Then** the remaining SCVI-only backend modules, state contexts, commands, and frontend state surfaces are removed
**And** no supported product or developer workflow depends on `SamplerContext`, `.scvi` container handling, or sampler-specific event/action contracts.

**Given** some current internal behaviors may still be represented through legacy sampler-era surfaces
**When** this story is complete
**Then** each required behavior is either replaced by a VST-owned seam or intentionally removed
**And** the codebase no longer carries dormant SCVI fallback, audition, or cache infrastructure as an undeleted subsystem.

**Given** the project intent is full SCVI replacement by MVP completion rather than permanent coexistence
**When** repository/build/test verification is run after cleanup
**Then** the app builds and tests without SCVI modules present in active source
**And** maintainers can inspect the codebase and treat VST playback as the only remaining playback system apart from historical planning or release records.

### Story 7.3: Publish a Known-Good Compatibility Matrix and Manual Release Runbook

As a maintainer,
I want a concrete compatibility matrix and manual validation runbook for the MVP playback baseline,
So that release decisions are based on a documented known-good subset rather than unwritten team memory.

**FRs implemented:** FR39

**Acceptance Criteria:**

**Given** Epic 7 is the final release-readiness hardening epic
**When** its validation outputs are finalized
**Then** Compose has a maintained known-good compatibility matrix for the named MVP plugin subset
**And** that matrix records the supported playback workflow, realism expectations, supported switching types such as keyswitch-note or controller-driven articulation changes, and any slot, part, channel, or shared-instance behavior relevant to each baseline plugin or library, along with major caveats.

**Given** compatibility evidence alone is not enough to make the release path reproducible
**When** the final validation guidance is written
**Then** Compose also has a manual runbook describing how to verify the baseline playback and realism workflow on macOS
**And** that runbook names the representative scenarios and checks required before calling the MVP release-credible.

**Given** the product should avoid overstating compatibility
**When** the matrix and runbook are published
**Then** they define one known-good supported subset and its limits clearly
**And** they do not imply blanket support for arbitrary VST3 plugins beyond the validated baseline.
