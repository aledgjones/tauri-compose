---
stepsCompleted:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
  - 7
  - 8
inputDocuments:
  - "_bmad-output/planning-artifacts/prd.md"
  - "_bmad-output/planning-artifacts/ux-design-specification.md"
  - "docs/index.md"
  - "docs/project-overview.md"
  - "docs/component-inventory.md"
  - "docs/development-guide.md"
  - "docs/data-models.md"
  - "docs/architecture.md"
  - "docs/performance-baselines.md"
  - "docs/source-tree-analysis.md"
  - "docs/project-context.md"
workflowType: "architecture"
lastStep: 8
status: "complete"
completedAt: "2026-03-20"
project_name: "compose"
user_name: "Aled"
date: "2026-03-20"
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
The project defines 41 functional requirements centered on replacing the current SCVI playback path with a macOS-only VST3 playback architecture. The primary architectural driver is first-class VST3 instrument support inside Compose's score-first workflow. Architecturally, these requirements group into eight major capability areas: plugin discovery, rack and routing state, plugin lifecycle, score-to-playback mapping, transport playback, mixer control, persistence and recovery, and release validation.

The requirements imply several major architectural subsystems. First, the app needs a plugin discovery and inventory layer that can maintain a cached registry, support manual rescans, and distinguish usable versus unavailable plugins. Second, it needs a rack and routing model that links score instruments to playback-capable slot targets while preserving directness in the score workflow, keeping the current MVP single-target-first, and allowing missing or invalid mappings to degrade cleanly. Third, it needs a plugin runtime and lifecycle layer capable of loading VST3 instruments, supporting editor hosting where practical, and keeping plugin lifecycle behavior stable enough for repeated composition sessions.

The requirements also reveal a semantic translation layer between score data and playback behavior. This is most visible in the expression-map requirements, which indicate that playback cannot remain a purely plugin-local concern. The architecture will need a durable seam that maps score intent and score-linked semantics into playback instructions without collapsing those semantics into ad hoc frontend or plugin-specific logic.

Transport requirements are especially important, but as enabling architecture rather than an independent product goal. Stable play, pause, resume, seek, and bounded edit-during-playback behavior are necessary because VST3 support is irrelevant if the resulting playback is not composition-worthy. Architecturally, that pushes transport, scheduling, playback state coherence, explicit live-update rules, and recovery into the core of the solution.

Finally, the validation-oriented functional requirements mean the architecture must explicitly support representative compatibility testing, transport regression scenarios, and release-readiness evaluation. This is a platform-credibility effort, not only a user-facing feature addition.

**Non-Functional Requirements:**
The strongest non-functional requirements are realtime safety, playback stability, jitter-free performance, reliability, offline desktop usability, and measurable validation. Stable, jitter-free playback is a non-negotiable architectural and release gate. The audio thread must remain lock-free, rendering and UI work must not introduce audible instability, and representative transport scenarios must complete without dropouts or jitter. These are hard constraints that directly shape concurrency, ownership, buffering, scheduling, and runtime boundaries.

Reliability requirements are equally strong. Playback state must remain coherent across repeated transport operations, project reopen behavior must restore valid rack slots and routes correctly, and missing plugins must degrade to recoverable unrouted states rather than invalidate the project workflow. This implies the need for explicit state models around plugin availability, route health, and transport lifecycle.

Integration requirements are also significant because the system must coordinate local VST3 discovery paths, native editor hosting, score workflows, persistence, audio-device settings, and representative plugin validation. The architecture therefore needs strong seam definitions between plugin inventory, plugin runtime, score semantics, transport, and UI control surfaces.

Measurement and validation are first-class architectural concerns rather than post-hoc QA tasks. The PRD expects representative regression scenarios, explicit thresholds for startup, rescan, reopen, route-to-audible-playback behavior, and zero-jitter or zero-dropout transport behavior. The architecture should therefore expose stable seams for diagnostics and validation rather than relying on one-off manual reasoning.

**Scale & Complexity:**
This is a high-complexity brownfield desktop architecture effort. The project combines native plugin hosting, realtime audio execution, score-semantic modeling, multi-window UI coordination, persistence, and performance validation within an existing Rust-owned state architecture.

- Primary domain: native desktop notation and playback application
- Complexity level: high
- Estimated architectural components: 8 to 10 major decision areas

The project is not enterprise in an organizational sense, but it does have enterprise-like architectural risk because several cross-cutting constraints must hold at once: VST3 host credibility, realtime safety, jitter-free playback, transport trust, semantic correctness, persistence stability, and usable UX under degraded conditions.

### Technical Constraints & Dependencies

The existing product architecture already establishes important constraints that the playback solution should respect. Rust is the system owner for state and runtime behavior, while React remains a display and control layer. The score is rendered natively, and transport state already follows a constrained runtime model designed to avoid noisy event emissions during playback. These existing decisions strongly suggest that the new playback architecture should remain backend-owned rather than introducing frontend-owned orchestration logic.

The platform scope is explicitly macOS-only for MVP and VST3-only for the plugin format. That is a useful simplifying constraint and should be treated as an intentional architectural narrowing, not a temporary inconvenience to abstract away prematurely.

The product is offline-first and local-desktop oriented. Plugin discovery, cached inventory use, project loading, rack setup, routing, and playback must work without network dependencies. Persistence therefore remains local and must tolerate missing assets without breaking score continuity.

The codebase also brings its own structural constraints: a Rust-owned score domain model, a disciplined persistence and migration workflow, a multi-window Tauri shell, native renderer ownership, and an established distinction between score-context UI and app-scoped windows. These are not just implementation details; they should shape architecture decisions so the new playback work extends the existing system coherently.

### Cross-Cutting Concerns Identified

VST3 host credibility is the leading cross-cutting concern. Discovery, rack and route resolution, loading, editor hosting, persistence, transport integration, and validation all need to work together well enough that VST3 instruments feel like a first-class Compose capability rather than a bolted-on host experiment.

Realtime safety and jitter-free playback are the next most important cross-cutting concerns. Plugin hosting, transport scheduling, mixer/runtime coordination, UI control, and diagnostics all have to respect the constraint that the audio path remains lock-free and audibly stable under representative composition workloads.

Transport correctness is another foundational concern spanning score semantics, playback preparation, scheduler design, recovery handling, and validation. The product promise depends on trustworthy navigation and repeated audition, not merely on isolated sound generation.

Semantic translation between score intent and playback behavior is also cross-cutting. Instrument routing, route-owned expression maps, articulations, and future playback intelligence all require a clear ownership seam so notation meaning is not re-derived inconsistently across the runtime.

Persistence and degraded-state recovery affect both architecture and UX. The system must preserve project continuity even when playback assets change, which means rack state, route state, plugin identity, and recovery behavior need explicit durable models rather than informal conventions.

Finally, validation and observability are cross-cutting because the product's acceptance depends on representative VST3 compatibility checks, transport regressions, and performance thresholds. The architecture should make these evaluation paths explicit and maintainable.

## Starter Template Evaluation

### Primary Technology Domain

Desktop application, based on project requirements analysis.

This is specifically a macOS-first native desktop notation and playback product with a Rust-owned runtime, React/TypeScript UI shell, Vite frontend tooling, and Tauri desktop integration. Because the project is already a brownfield codebase with substantial architecture in place, the relevant starter decision is not which greenfield scaffold to generate, but which maintained foundation should be treated as canonical.

### Starter Options Considered

**1. Tauri official starter and manual Tauri + Vite setup**
The closest maintained baseline to the current architecture is Tauri's official project creation flow. Current official guidance supports `create-tauri-app` for generating a new Tauri project and also documents a manual setup path using Vite plus the Tauri CLI. Tauri's frontend guidance continues to recommend Vite for JavaScript and TypeScript SPA frameworks such as React.

This option aligns with the existing codebase because it preserves the desktop shell, the Rust runtime, the frontend/backend separation model, and the current Vite-based frontend setup. It is also the only evaluated option that matches the project's current architectural direction without introducing a platform reset.

**2. Electron Forge**
Electron Forge is a current, maintained desktop starter with `create-electron-app@latest` and first-party `vite` / `vite-typescript` templates. It provides a strong build and packaging pipeline, but it would move the application onto Electron's runtime model and away from the current Rust-owned system architecture.

For this project, Electron Forge is not a fit because the product already depends on Rust-native ownership for score state, rendering, performance-sensitive backend work, and the architectural direction needed for VST3 hosting and jitter-sensitive playback.

**3. Wails**
Wails remains a maintained desktop framework, but it is Go-based rather than Rust-based. It does not align with the current codebase, the current runtime ownership model, or the deeper Rust integration already present in rendering, persistence, and audio.

For this project, Wails is not an appropriate foundation candidate.

### Selected Starter: Existing Tauri + React + TypeScript + Vite Foundation

**Rationale for Selection:**
This project should preserve and extend its existing brownfield foundation rather than adopt a new starter. The current stack already matches the architectural direction revealed by the product requirements:

- Tauri keeps the app in a lightweight native desktop shell
- Rust remains the owner of application state, runtime behavior, rendering integration, and performance-critical logic
- React and TypeScript provide the UI shell while preserving the established backend-owned architecture
- Vite provides the recommended frontend build path for this class of Tauri application

This is the right foundation for the project's primary goal of first-class VST3 instrument support with non-negotiable stable, jitter-free playback. Re-platforming to a different desktop starter would add architectural churn without helping the core product goal.

**Reference Initialization Command:**

If this foundation needed to be recreated from scratch using the current official manual path, the reference baseline would be:

```bash
npm create vite@latest .
npm install -D @tauri-apps/cli@latest
npx tauri init
```

Alternatively, the current official Tauri scaffold path is:

```bash
sh <(curl https://create.tauri.app/sh)
```

with React + TypeScript selected during prompts.

**Architectural Decisions Provided by Foundation:**

**Language and Runtime:**

- Rust as the backend and runtime owner
- TypeScript for frontend application code
- React for the UI shell
- Tauri 2 as the desktop host

**Styling Solution:**

- Existing project-standard Tailwind CSS based styling
- Existing Compose design system and component primitives preserved rather than replaced

**Build Tooling:**

- Vite-based frontend build and development workflow
- Tauri CLI driven desktop development and packaging
- Rust/Cargo backend compilation and validation

**Testing Framework:**

- Existing repo-specific testing and validation setup should be preserved and extended in place rather than replaced by starter defaults

**Code Organization:**

- Rust-owned application state and backend commands
- React display/control layer
- native-rendered score surface plus HTML/React surrounding UI
- multi-window Tauri application structure

**Development Experience:**

- hot reload and Vite development server
- strict TypeScript configuration
- repo-specific linting, typing, and Rust validation rules
- a desktop-oriented local development workflow consistent with current project practice

**Brownfield Note:**
For this project, starter selection is a reference decision, not an initialization task. The architectural implication is to preserve the current Tauri + Rust + React + Vite foundation as canonical and evolve it deliberately for VST3 hosting, transport stability, and jitter-free playback, rather than replacing it with a new scaffold.

The project must not be reinitialized. Any future use of starter guidance is strictly for reference, comparison, or reconstructing equivalent conventions in isolated experiments, not for resetting the existing application.

**Note:** In this brownfield case, project initialization is not the first implementation story. The first implementation stories should build on the existing foundation rather than recreate it.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**

- Rust remains the sole authoritative owner of plugin inventory, rack state, slot state, route state, playback preparation state, and transport state.
- `.soloc` persists portable project intent and rack or route references, while machine-local plugin discovery/cache state remains outside project files.
- The realtime playback path consumes precomputed, lock-safe state only.
- Stable, jitter-free playback is a hard architectural and release gate.
- Internal app communication remains command-oriented and backend-owned, with playback-critical paths avoiding event chatter.
- The existing Tauri + Rust + React + TypeScript + Vite brownfield foundation is preserved rather than reinitialized or replaced.

**Important Decisions (Shape Architecture):**

- No product-level authentication or in-app authorization model is introduced for MVP.
- VST3 plugins are treated as untrusted third-party code from a host-safety perspective.
- The frontend remains a thin control/presentation layer over Rust-owned domain state.
- Score-context playback relationships live in contextual panels, while app-scoped playback assets and inventories live in dedicated windows.
- Canonical app-scoped entry points for Plugin Manager and Expression Map Library live in the system app menu, while contextual score surfaces can launch those same windows when needed.
- Diagnostics and validation are treated as first-class architectural seams.

**Deferred Decisions (Post-MVP):**

- Broader compatibility heuristics for problematic or out-of-spec plugins
- Richer routing and shared-instance sophistication
- Expanded diagnostics polish and possible user-facing tooling
- Future platform expansion beyond macOS MVP
- Deeper post-MVP expression-map sophistication and automation breadth

### Data Architecture

- Rust is the sole source of truth for plugin inventory, rack state, slot state, route state, playback preparation state, and transport state.
- `.soloc` stores portable project intent and rack or route references, not machine-local plugin discovery/cache state.
- Machine-local plugin inventory and discovery cache live in app-local storage outside project files.
- Playback runtime is split between a Rust control plane and a realtime-safe playback plane, with the audio path consuming precomputed, lock-safe state only.
- Missing plugin resolution degrades to explicit recoverable unrouted states rather than silent fallback or corrupted project state.
- Remaining SCVI sampler/runtime code is temporary legacy scaffolding only. Epic 2 through Epic 6 must build replacement VST-owned seams rather than extending `SamplerContext`, `soloc://state/sampler`, `actions.sampler.*`, or other sampler-era contracts; the planned end state is full removal of that subsystem once the VST path is complete.

**Rationale:**
This preserves the strongest existing architectural rule in the product, protects the jitter-sensitive path from UI and control-plane noise, and cleanly separates portable score intent from machine-local plugin availability.

### Authentication & Security

- No product-level user authentication is introduced in MVP.
- No in-app authorization model is introduced beyond local OS/application access.
- Security strategy focuses on local execution boundaries, defensive plugin-host handling, and persisted-data integrity.
- VST3 plugins are treated as untrusted third-party code from a host-safety perspective.
- Plugin discovery is limited to explicit local VST3 locations and validated loading paths.
- Plugin-related failures must degrade predictably without destabilizing audio or corrupting project state.
- No network-dependent auth or cloud identity infrastructure is introduced.

**Rationale:**
The real security surface in this product is local plugin execution, file integrity, and runtime stability, not SaaS-style user management.

### API & Communication Patterns

- Internal app communication is command-oriented between frontend and backend.
- Mutations remain explicit and backend-owned through typed Tauri command boundaries.
- Contracts remain aligned to Rust-owned models and existing `snake_case` conventions.
- State push is used where appropriate, but playback-critical paths avoid high-frequency event chatter.
- Transport runtime state uses polling or lock-safe snapshots rather than emitted-event streams.
- Plugin inventory, route status, transport preparation, and recovery state are exposed as Rust-owned domain APIs rather than frontend-owned orchestration logic.
- Error handling is standardized around explicit categories for recoverable user issues, missing plugins, host/runtime failures, and persistence/load validation failures.
- MCP/automation surfaces should reuse the same backend seams rather than introducing parallel logic paths.

**Rationale:**
This preserves runtime truth in Rust, reduces FE/BE drift, and protects playback stability from communication noise.

### Frontend Architecture

- The frontend remains a thin control and presentation layer over Rust-owned domain state.
- No second frontend-owned domain model is introduced for plugins, transport, or playback semantics.
- Local React state is limited to ephemeral UI concerns such as open/closed state, hover, local input drafts, and other transient presentation concerns.
- The native-rendered score surface is preserved, with React handling overlays, controls, panels, and windows.
- Score-context relationships remain in contextual panels, while app-scoped playback assets and inventories live in dedicated windows.
- Compact `Ready`, `Needs Attention`, and `Unavailable` states are presentation summaries over more precise Rust-owned routing and recovery state such as unrouted, missing plugin, incomplete linkage, or deferred live-update state.
- Progressive disclosure keeps routing and readiness primary, deeper plugin editor access secondary, and advanced semantic playback configuration behind deliberate entry points.
- The system app menu owns the canonical launch points for app-scoped playback resources, with score-context surfaces offering contextual handoff into those same windows.
- Frontend performance strategy minimizes rerenders, FE/BE chatter, and parallel derived state.
- Plugin editor hosting is treated as a backend/runtime-integrated capability exposed through frontend entry points, not a frontend-owned embedding architecture.

**Rationale:**
This matches the existing product architecture and keeps React from becoming the accidental owner of playback logic.

### Infrastructure & Deployment

- The primary deployment target is a signed macOS desktop application package produced through the existing Tauri release workflow.
- MVP remains local-desktop only, with no hosted backend or cloud runtime.
- CI/CD focuses on build validation, lint/type/test validation, Rust quality gates, and packaging integrity rather than service deployment orchestration.
- Release workflows remain human-controlled for versioning, migration promotion, packaging, and final release actions.
- Environment configuration remains minimal and explicit, with limited development overrides for plugin-path and experimental host settings.
- Diagnostics and logging focus on plugin discovery, route resolution, transport/playback behavior, host/editor lifecycle, and persistence/migration/load failures.
- Performance validation remains part of release-readiness, with representative playback and regression scenarios treated as ship gates.
- Scaling is product-capability scaling rather than cloud autoscaling.

**Rationale:**
This aligns deployment and operational practice with the actual product shape: a local desktop tool whose success depends on VST3 credibility, stable playback, and release integrity.

### Decision Impact Analysis

**Implementation Sequence:**

1. Preserve the existing Rust-owned/Tauri-based foundation and avoid reinitialization or replatforming.
2. Establish the plugin inventory, identity, rack-slot, route, and recovery model.
3. Define the control-plane versus realtime-playback-plane split.
4. Formalize command-oriented FE/BE seams and playback-safe communication rules.
5. Build frontend surfaces around Rust-owned playback state without introducing parallel domain logic.
6. Add diagnostics and validation seams that support compatibility testing, jitter-free playback verification, and release readiness.

**Cross-Component Dependencies:**

- Data architecture decisions directly constrain persistence, recovery UX, and playback preparation.
- API/communication decisions depend on Rust remaining the authoritative control plane.
- Frontend architecture depends on the command-oriented backend seam and explicit state ownership boundaries.
- Infrastructure and deployment decisions depend on diagnostics, validation seams, and migration discipline being architected in from the start.
- Security strategy depends on plugin-host boundaries and persisted-data validation rather than user identity infrastructure.

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:**
The highest-risk conflict areas are:

- state ownership boundaries between Rust, persisted project data, machine-local cache state, and React UI state
- naming consistency across Rust structs, TypeScript mirrors, Tauri commands, and automation-facing seams
- placement of playback, plugin-host, transport, and recovery logic
- consistent degraded-state and error-handling behavior for plugin/runtime failures
- consistent diagnostics and validation patterns for playback-critical features

### Naming Patterns

**Persistence and Shared Model Naming Conventions:**

- Any model that crosses the Rust ↔ TypeScript boundary must use `snake_case` field naming.
- Persisted `.soloc` payload fields must match Rust-owned field naming directly rather than introducing frontend-specific aliases.
- TypeScript mirrors of persisted or command-bound data must preserve the Rust field names exactly.
- Tauri commands must remain `snake_case` and align with backend-owned domain seams.

**Examples:**

- `flow_id`, `track_id`, `plugin_id`, `slot_id`
- `score_entries_tone_create`
- `transport_get_state`

**Code Naming Conventions:**

- Rust function and file naming follows existing Rust conventions already present in the project.
- TypeScript component exports remain PascalCase, but filenames follow the project's kebab-case rule.
- Do not invent alternative naming styles for playback-specific code.

**Examples:**

- file: `plugin-host-window.tsx`
- component: `PluginHostWindow`
- Rust file: `transport.rs`
- Rust function: `resolve_route_state`

**Domain Naming Rules:**

- Use `route` for the relationship between a score instrument and a playback-capable slot target.
- Use `rack` for the current score-context playback container and `slot` for one hosted plugin instance inside it.
- Use `target` for the stable product-facing destination and `address` for the engine-specific VST-rooted payload used to reach it.
- Use `inventory` for machine-local discovered plugin availability.
- Use `recovery` for reopen or re-resolution of rack slots and routes against current local inventory.
- Use `unrouted` for recoverable missing or unresolved playback state.
- Do not blur these terms together.

### Structure Patterns

**Project Organization Rules:**

- Rust owns runtime logic, persistence logic, transport logic, playback preparation, plugin discovery, rack or route resolution, and host lifecycle logic.
- React owns display, local interaction affordances, contextual panels, app windows, and transient UI state only.
- Native-rendered score behavior remains backend-owned; React overlays interact with it but do not recreate score-domain truth.

**Placement Rules:**

- Persisted project-domain logic belongs in `src-tauri/src/score/`, `src-tauri/src/files.rs`, or related Rust persistence seams.
- Playback runtime logic belongs in `src-tauri/src/playback/`.
- Tauri command boundaries belong in backend command seams and corresponding `actions.ts` frontend callers.
- Frontend playback UI belongs in the appropriate window or screen component tree, but must not own playback truth.

**Tests and Validation Placement:**

- Keep regression coverage close to the owning seam.
- Playback scheduling regressions belong near transport/playback logic.
- Persistence regressions belong near load/save or migration seams.
- UI-only display behavior belongs in frontend tests when such tests exist.
- Do not introduce broad duplicate harnesses when local seam coverage is sufficient.

### Format Patterns

**Persistence Format Rules:**

- `.soloc` stores portable project intent and rack or route references only.
- Machine-local discovery/cache data must not be serialized into project files.
- Do not persist frontend convenience state unless it is already part of the Rust-owned project model.

**Error Format Rules:**

- Use explicit backend-owned error categories for:
  - recoverable user-action issues
  - missing or unavailable plugins
  - plugin host/runtime failures
  - persistence or migration validation failures
- UI should render these categories differently, but must not invent its own parallel taxonomy.

**Diagnostic Format Rules:**

- Diagnostics for playback-critical behavior should be explicit, structured, and domain-specific.
- Prefer stable fields over ad hoc string-only logging when diagnostics are intended for maintainers or tooling.
- Use null-safe reset/cleared payloads instead of frontend-invented fallback state.

### Communication Patterns

**Frontend ↔ Backend Communication Rules:**

- All authoritative mutations go through backend-owned command seams.
- Use event/state push only where appropriate for application-state distribution.
- Do not use high-frequency emitted events for playback-critical runtime state.
- Use polling or lock-safe snapshots for transport/playback state where the architecture already requires that approach.

**Automation/MCP Rules:**

- MCP/automation actions must reuse the same backend-owned seams as the UI whenever practical.
- Do not create a second implementation path just for automation.
- Automation-facing actions may adapt parameter shape for tooling, but the domain logic must remain shared.

**State Ownership Rules:**

- Rust owns:
  - plugin inventory truth
  - rack-slot and route truth
  - transport truth
  - playback preparation truth
  - persisted project truth
- machine-local app storage owns:
  - local plugin discovery/cache state
  - local environment-specific inventory metadata
- React local state owns:
  - hover/open/selection-affordance UI state
  - transient text input drafts
  - window-local presentation state that is not domain truth

### Process Patterns

**Error Handling Patterns:**

- Missing plugins must degrade to explicit recoverable unrouted states.
- Host or editor failures must not silently mutate project intent.
- Runtime failures must be handled in a way that protects playback stability first, then surfaces recoverable UX second.
- Jitter, dropout risk, or realtime-safety regressions are release-blocking issues, not normal UI errors.

**Loading and Recovery Patterns:**

- Reopen flow should resolve project-persisted rack and route intent against current machine-local inventory.
- If resolution fails, preserve user intent and expose recoverable unresolved state.
- Never silently remap to a different plugin without an explicit product rule that says so.
- Never treat machine-local cache state as canonical project state.

**Performance-Sensitive Change Patterns:**

- Any feature touching playback, transport, plugin runtime, or renderer-adjacent timing must explicitly preserve the jitter-free playback constraint.
- Prefer precomputation, immutable snapshots, or lock-safe handoff to the audio path.
- Do not introduce UI-driven or command-chatter-driven behavior into the realtime path.

### Enforcement Guidelines

**All AI Agents MUST:**

- preserve Rust as the sole source of truth for playback, transport, plugin inventory truth, and persisted project-domain state
- keep `.soloc` portable by separating project intent from machine-local plugin discovery/cache state
- avoid introducing frontend-owned parallel playback or plugin domain models
- use existing naming conventions (`snake_case` across shared models and commands, kebab-case TS filenames, Rust conventions in backend code)
- keep validation and regression coverage near the owning seam
- treat stable, jitter-free playback as a non-negotiable constraint when making implementation choices

**Pattern Enforcement:**

- Verify new persisted fields against the Rust-owned data model and migration rules.
- Verify new frontend state against ownership boundaries before accepting it as domain state.
- Verify new playback or transport behavior against realtime-safety and communication rules.
- Treat any introduction of parallel truth between Rust and React as an architectural defect unless explicitly justified.

### Pattern Examples

**Good Examples:**

- Persisting `slot_id`, `target_id`, and `VstAddress`-backed route intent in project data while keeping the discovered local plugin inventory outside `.soloc`
- Using a backend command to update route state and pushing resolved state back to the UI
- Polling or snapshot-based transport state instead of emitting high-frequency playback events
- Keeping plugin editor open/focus actions frontend-triggered but backend-owned in lifecycle terms
- Reporting missing plugin reopen as an explicit unresolved route rather than silently clearing user intent

**Anti-Patterns:**

- storing machine-local plugin scan results inside project files
- introducing frontend-owned copies of rack or route truth or transport truth
- letting React derive or own playback semantics that should live in Rust
- adding event-heavy playback communication on the critical runtime path
- silently swapping unresolved routes to arbitrary fallback plugins
- inventing a separate automation-only domain seam that diverges from the UI/backend seam

## Project Structure & Boundaries

### Complete Project Directory Structure

```text
compose/
├── package.json
├── vite.config.ts
├── eslint.config.js
├── styles.css
├── README.md
├── TESTING.md
├── docs/
│   ├── architecture.md
│   ├── data-models.md
│   ├── development-guide.md
│   ├── component-inventory.md
│   ├── performance-baselines.md
│   ├── project-context.md
│   └── source-tree-analysis.md
├── src/
│   ├── components/                  # shared UI components
│   ├── ui/                          # reusable UI primitives
│   ├── services/
│   │   ├── actions.ts               # FE→BE command seams
│   │   ├── types.ts                 # Rust-aligned TS mirrors
│   │   ├── state/                   # FE state providers over Rust-owned state
│   │   └── utils/                   # frontend-safe helpers only
│   ├── windows/
│   │   ├── main/
│   │   │   ├── components/
│   │   │   ├── screens/
│   │   │   │   ├── setup/
│   │   │   │   ├── write/
│   │   │   │   └── sequence/
│   │   │   ├── hooks/
│   │   │   └── utils/
│   │   ├── plugin-settings/
│   │   ├── preferences/
│   │   ├── project-settings/
│   │   └── ... other existing windows
│   └── test-setup.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── lib.rs                   # app wiring and managed state
│       ├── main.rs
│       ├── files.rs                 # load/save/open/reopen seams
│       ├── migrations/              # persisted-data migrations
│       ├── score/                   # canonical score domain model
│       ├── ui/                      # Rust-owned UI state
│       ├── renderer/                # native score rendering
│       ├── instructions/            # parse/draw/render planning
│       ├── playback/
│       │   ├── audio.rs             # audio device I/O
│       │   ├── transport.rs         # transport state/scheduling
│       │   ├── sampler.rs           # current playback inventory/cache seam
│       │   ├── mixer.rs             # playback mixing
│       │   ├── streamer.rs          # audio streaming path
│       │   └── scvi.rs              # legacy/current sample-container seam
│       └── bin/                     # supporting CLI tooling
├── tests/
│   └── fixtures/                    # manual visual regression fixtures
└── _bmad-output/
  └── planning-artifacts/          # generated planning documents
```

### Architectural Boundaries

**Runtime and Domain Boundaries:**

- `src-tauri/src/score/` remains the canonical persisted score/project domain.
- `src-tauri/src/playback/` owns playback runtime behavior, transport coordination, inventory resolution, mixer behavior, and playback-safe state preparation.
- `src-tauri/src/files.rs` and `src-tauri/src/migrations/` own file-format load/save and migration boundaries.
- `src-tauri/src/ui/` may hold Rust-owned UI-facing state, but it must not become a second playback domain model.

**Frontend Boundaries:**

- `src/services/actions.ts` is the canonical frontend entry point for backend mutations.
- `src/services/types.ts` mirrors Rust-owned models and command-facing payloads; it does not define competing truth.
- `src/windows/main/screens/write/` and related contextual surfaces own score-context playback visibility and interaction.
- dedicated app-scoped playback inventory or reusable playback-asset management belongs in dedicated windows rather than selection-bound panels.

**Persistence Boundaries:**

- `.soloc` project data contains portable score and playback intent.
- machine-local plugin discovery/cache state must live outside `.soloc`.
- migration logic for any persisted-data change belongs in `src-tauri/src/migrations/`.

### Requirements to Structure Mapping

**FR Category: Plugin Discovery and Playback Environment**

- Primary backend location: `src-tauri/src/playback/`
- Supporting persistence/config locations: `src-tauri/src/files.rs`, machine-local app storage seams
- Frontend surfaces: plugin-management windows and routing entry points in existing UI

**FR Category: Instrument Routing and Playback Setup**

- Canonical domain ownership: `src-tauri/src/score/` for persisted rack and route intent
- Resolution/runtime ownership: `src-tauri/src/playback/`
- Frontend surfaces: instrument-related controls in `src/windows/main/` and existing settings windows

**FR Category: Plugin Lifecycle and Editing**

- Backend ownership: `src-tauri/src/playback/` plus app/window lifecycle wiring in `src-tauri/src/lib.rs`
- Frontend ownership: deliberate entry points only, not host truth or lifecycle truth

**FR Category: Score-To-Playback Mapping**

- Score semantics: `src-tauri/src/score/`
- Playback translation and future expression-map application seams: `src-tauri/src/playback/`
- Contextual UX: `src/windows/main/screens/write/` and related panels

**FR Category: Transport Playback**

- Backend ownership: `src-tauri/src/playback/transport.rs`, `mixer.rs`, `streamer.rs`, and adjacent playback modules
- Frontend ownership: transport controls and displays only

**FR Category: Mixer and Playback Control**

- Backend ownership: playback runtime/mixer seams
- Frontend ownership: mixer and transport control surfaces in `src/windows/main/components/`

**FR Category: Project Persistence and Recovery**

- Backend ownership: `src-tauri/src/files.rs`, `src-tauri/src/migrations/`, persisted score/playback identity in `src-tauri/src/score/`
- Frontend ownership: recovery-state display and rerouting UX only

**FR Category: Validation and Release Readiness**

- Diagnostics and runtime instrumentation: backend seams in `src-tauri/src/playback/`, `renderer/`, and related timing/validation paths
- Maintainer-facing docs and baselines: `docs/`, `TESTING.md`, `docs/performance-baselines.md`

### Integration Points

**Internal Communication:**

- React communicates with Rust through `src/services/actions.ts` and backend-owned Tauri commands.
- Rust pushes or exposes state to frontend consumers using the existing managed-state/event/polling architecture.
- Playback-critical runtime state avoids high-frequency event chatter and stays on polling/snapshot-safe paths.

**External Integrations:**

- VST3 plugin discovery and loading integrate through backend-owned local desktop seams only.
- Native plugin editor hosting integrates through backend/runtime window ownership with frontend-triggered entry points.
- MCP/automation integrations must reuse backend-owned command/domain seams rather than parallel implementations.

**Data Flow:**

1. User acts through React UI.
2. Frontend issues explicit backend command through `actions.ts`.
3. Rust mutates authoritative state.
4. Rust updates persisted domain state, runtime state, or local machine cache state in the appropriate boundary.
5. Frontend receives updated state or queries snapshots for display.
6. Playback runtime consumes precomputed, lock-safe state rather than UI-driven state.

### File Organization Patterns

**Configuration Files:**

- root config remains for frontend/build/tooling concerns
- Tauri/Rust config remains in `src-tauri/`
- project rules and architecture docs remain under `docs/` and planning artifacts

**Source Organization:**

- domain truth and runtime logic in `src-tauri/`
- UI display/control logic in `src/`
- avoid placing backend domain logic into frontend utility folders
- avoid placing UI-only concerns into playback/runtime modules

**Test Organization:**

- seam-local regression coverage close to the owning code
- fixture-based visual/manual regression assets under `tests/fixtures/`
- playback and persistence regressions near the backend seams they validate

**Asset Organization:**

- frontend static assets remain under existing frontend asset paths
- project or plugin-discovery local data is not stored as source-controlled static assets
- generated binary/runtime artifacts remain outside persisted project payloads unless intentionally part of the file format

### Development Workflow Integration

**Development Server Structure:**

- Vite/Tauri development remains the primary local workflow
- frontend and backend continue to be developed together through the existing desktop-dev loop

**Build Process Structure:**

- frontend build uses existing Vite/Tauri coordination
- backend build and validation continue through Cargo/Tauri
- release preparation remains aligned with the existing human-controlled version and migration flow

**Deployment Structure:**

- packaging remains macOS desktop oriented
- structure supports local persistence, local plugin discovery, and desktop-native playback/runtime behavior
- no cloud deployment structure is introduced for MVP

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:**
The architectural decisions are internally consistent. The preserved Tauri + Rust + React + TypeScript + Vite foundation aligns with the product's brownfield reality and with the primary architectural goal of first-class VST3 instrument support. The runtime/data architecture, communication patterns, frontend boundaries, and project structure all reinforce the same ownership model: Rust owns domain and runtime truth, while React remains a thin display/control layer.

The non-negotiable jitter-free playback requirement is also coherent across the architecture. It is reflected in the decision to keep the realtime path lock-safe, to avoid high-frequency event chatter, to separate portable project intent from machine-local plugin inventory/cache state, and to keep playback/runtime logic in backend-owned seams.

**Pattern Consistency:**
The implementation patterns support the architectural decisions well. Naming rules align with the existing Rust/TypeScript boundary model. Structure rules reinforce the runtime and persistence boundaries already chosen. Communication rules protect playback-critical paths from frontend-driven noise. Error-handling and degraded-state patterns support the recovery model required by the PRD.

**Structure Alignment:**
The project structure section is aligned with the architecture because it maps the existing repository to the required capability areas rather than inventing a hypothetical scaffold. The boundaries between score domain, playback runtime, persistence, renderer, frontend UI, and diagnostics are clear enough to guide implementation consistently.

### Requirements Coverage Validation ✅

**Feature Coverage:**
All major functional requirement clusters are architecturally supported:

- plugin discovery and playback environment
- instrument routing and playback setup
- plugin lifecycle and editing
- score-to-playback mapping
- transport playback
- mixer and playback control
- project persistence and recovery
- validation and release readiness

**Functional Requirements Coverage:**
The architecture provides explicit ownership and placement for all FR categories. Plugin inventory, routing, transport, recovery, and validation are all given concrete homes in the existing repository structure. The architecture also preserves the intended product framing: VST3 support is primary, and surrounding architecture exists to make that support stable and composition-worthy.

**Non-Functional Requirements Coverage:**
Performance, realtime safety, stability, offline-first behavior, persistence integrity, and release validation are all addressed. Stable, jitter-free playback is explicitly treated as a release gate. Security is appropriately scoped to local execution boundaries, plugin-host safety, and persisted-data integrity rather than irrelevant SaaS patterns.

### Implementation Readiness Validation ✅

**Decision Completeness:**
Critical architectural decisions are documented clearly enough to guide implementation. Versions and current baseline technologies have been validated where relevant. The most important architectural decisions are explicit: ownership boundaries, persistence boundaries, communication patterns, frontend/runtime separation, and deployment posture.

**Structure Completeness:**
The project structure is complete enough for a brownfield implementation guide. It identifies where each major responsibility belongs and maps requirement groups to the existing repository. This should be sufficient to prevent architectural drift during implementation.

**Pattern Completeness:**
The implementation patterns address the most likely conflict points between agents:

- naming and field-shape drift
- parallel frontend-owned playback truth
- misplaced runtime logic
- inconsistent degraded-state handling
- inconsistent diagnostics and validation approaches

### Gap Analysis Results

**Critical Gaps:**
No critical architectural gaps were identified that would block implementation.

**Important Gaps:**

- A more explicit plugin identity and re-resolution scheme may still need to be formalized during implementation planning.
- A formal error taxonomy for plugin/runtime/persistence failures would improve implementation consistency.
- A more explicit validation matrix for representative plugin compatibility and jitter-free playback scenarios would improve release-readiness discipline.

**Nice-to-Have Gaps:**

- More concrete examples of recovery UX states
- More detailed diagnostics visibility guidance
- A future expression-map seam diagram once that scope becomes implementation-near

### Validation Issues Addressed

The main validation risk was whether the architecture might accidentally drift into generic web-app patterns or greenfield-project assumptions. That risk has been addressed by:

- preserving the existing brownfield Tauri/Rust/React/Vite foundation
- explicitly prohibiting project reinitialization
- centering VST3 support as the primary architectural driver
- treating stable, jitter-free playback as non-negotiable
- mapping the architecture directly onto the existing repository structure

### Architecture Completeness Checklist

**✅ Requirements Analysis**

- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**✅ Architectural Decisions**

- [x] Critical decisions documented
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [x] Performance considerations addressed

**✅ Implementation Patterns**

- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented

**✅ Project Structure**

- [x] Concrete repository structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements-to-structure mapping completed

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** High

**Key Strengths:**

- Strong alignment with the existing product and repository
- Clear Rust-owned runtime and persistence model
- Explicit protection against parallel frontend truth
- VST3-primary framing preserved throughout
- Stable, jitter-free playback treated as an architectural gate rather than a follow-up concern
- Concrete brownfield structure guidance rather than abstract recommendations

**Areas for Future Enhancement:**

- Formal plugin identity/re-resolution scheme
- More explicit error taxonomy
- More explicit compatibility and playback validation matrix
- Future expression-map architectural elaboration when implementation scope requires it

### Implementation Handoff

**AI Agent Guidelines:**

- Follow the Rust-owned domain and runtime model exactly.
- Do not introduce frontend-owned parallel truth for playback, transport, or plugin state.
- Keep `.soloc` portable by separating project intent from machine-local discovery/cache state.
- Treat stable, jitter-free playback as a hard implementation constraint.
- Reuse existing backend seams for UI and automation work rather than creating parallel logic paths.

**First Implementation Priority:**
Build on the existing brownfield foundation by formalizing the plugin inventory, rack-slot, route, identity, recovery, and playback preparation seams inside the current Rust/Tauri architecture rather than reinitializing or replatforming the project.
