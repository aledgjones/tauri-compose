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
  - "_bmad-output/planning-artifacts/epics.md"
  - "_bmad-output/planning-artifacts/ux-design-specification.md"
  - "_bmad-output/planning-artifacts/sprint-change-proposal-2026-03-24.md"
  - "docs/index.md"
  - "docs/project-context.md"
  - "_bmad-output/implementation-artifacts/epic-3-retro-2026-03-24.md"
workflowType: "architecture"
lastStep: 8
status: "complete"
completedAt: "2026-03-24"
project_name: "compose"
user_name: "Aled"
date: "2026-03-24"
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
The loaded planning set defines 41 functional requirements, 14 explicit non-functional requirements, and 18 UX design requirements for a macOS-first VST3 playback initiative inside an existing desktop notation product. Architecturally, the work breaks into several capability groups: plugin discovery and inventory, rack-slot-route state, plugin lifecycle and editor hosting, score-to-playback translation, transport behavior, mixer control, persistence and recovery, expression-map realism, and release validation.

The core functional demand is not merely "play sound through plugins." The real product requirement is a trustworthy score-first playback workflow that stays inside Compose and lets users compose against playback rather than treat it as a fragile preview mode. This means the architecture must support fast routing, audible playback, transport trust, recovery from missing plugins, and eventual score-linked realism without turning the product into a DAW-shaped control surface.

The project context also includes a newly approved course correction. The Epic 3 retrospective and sprint change proposal establish that story-complete progress is currently ahead of product trust. Epic 4 is blocked, the current first-audible-playback sequence is no longer acceptable as the next step, and the architecture must now assume a required stabilization phase before audible-playback work resumes. That stabilization phase exists to remove sampler-era or SCVI-era active-path residue, restore trust in the single-instrument VST path, simplify rack-first score-context UX, and rebaseline downstream planning.

This introduces an important architectural fact: the current epic breakdown is historically useful but not authoritative for sequencing. The approved change proposal supersedes the old progression by inserting a new foundation-reset epic before first audible playback. The architecture therefore has to treat planning drift itself as an input constraint and explicitly define the target state the reset epic is working toward.

**Non-Functional Requirements:**
The strongest non-functional requirements are lock-free audio-thread behavior, zero-dropout representative transport regressions, coherent repeated play/pause/resume/seek behavior, bounded edit-during-playback behavior, and resilient reopen or recovery semantics. These are hard architectural constraints, not implementation polish. They directly constrain ownership, concurrency, scheduler design, communication patterns, and state modeling.

Reliability and recoverability are equally strong. The system must preserve score continuity when plugins are missing, degrade safely to explicit recoverable unrouted states, and restore valid routes when plugins are available. Because `.soloc` files must remain portable and machine-local plugin discovery must remain outside project files, persistence boundaries are central to the architecture.

A further non-functional requirement emerges from the course-correction material: deletion discipline. Once VST-owned seams replace sampler-era or SCVI-era behavior, those obsolete product paths must not remain as tolerated parallel ownership. The architecture has to treat legacy seam preservation as a defect, not as acceptable scaffolding, except where a bounded temporary bridge is explicitly approved.

UX also creates non-functional pressure. Score-context playback surfaces must remain compact and direct, while app-scoped inventories and deep editors belong in dedicated windows. This is not just a design preference; it is an architectural boundary that determines which layer owns route state, plugin inventory, expression-map assets, and native editor-host interactions.

**Scale & Complexity:**
This is a high-complexity brownfield desktop architecture effort.

- Primary domain: native desktop notation and playback tooling
- Complexity level: high
- Estimated architectural components: 9 to 11 major decision areas

The complexity comes from having to satisfy several hard constraints at once: native VST3 hosting on macOS, realtime-safe playback, transport trust, score-semantic translation, local persistence and recovery, compact score-first UX, and explicit deletion of obsolete playback seams during a live foundation reset.

### Technical Constraints & Dependencies

The existing product foundation is fixed: Tauri 2 desktop shell, Rust-owned state and runtime logic, React and TypeScript UI, and native score rendering via Skia and Metal. The architecture must extend this brownfield shape rather than reinitialize or replatform it.

The approved reset direction tightens several constraints. The active playback path must become truly VST-owned, not VST-shaped on top of sampler-era or SCVI-era compatibility residue. `SamplerContext`, `soloc://state/sampler`, and related sampler-era action seams are legacy seams and must not remain accepted owners for the supported product path once VST replacements exist.

Platform scope remains intentionally narrow: macOS only, VST3 only, offline-first, local desktop persistence, local plugin discovery, and local editor hosting. This is useful simplification and should not be abstracted away prematurely.

Another key dependency is planning integrity. The PRD and UX documents are still usable inputs, but they now sit alongside a retrospective and approved change proposal that say the current foundation is not trustworthy enough for direct continuation. The architecture therefore depends on making the reset target explicit enough that future stories do not drift back into evolutionary compatibility language or acceptance-criteria-driven UI bloat.

### Cross-Cutting Concerns Identified

The leading cross-cutting concern is authoritative playback ownership. Rust must remain the sole owner of plugin inventory truth, rack and slot state, route state, playback-preparation state, transport state, and persisted playback-domain truth. React can present and trigger, but it cannot become a second topology or playback owner.

The next concern is deletion of obsolete seams. The architecture must prevent sampler-era or SCVI-era product-path behavior from lingering as quiet fallback or parallel ownership once VST-owned seams exist. This is now a first-class architectural rule.

Realtime safety and transport trust are the next major concerns. The playback system must protect the audio path from locks, event chatter, and UI-driven state noise while supporting coherent play, pause, resume, seek, and bounded editing behavior. Any architecture that achieves sound generation but not dependable navigation still fails the product.

A fourth concern is score-context versus app-scoped UX ownership. Routing, readiness, and recovery belong in score-context surfaces. Plugin inventory, expression-map libraries, and deep editing belong in app-scoped windows. The architecture must preserve that split so the product stays direct instead of sprawling.

A fifth concern is persistence and recovery. `.soloc` must store portable project intent, not machine-local inventory state. Reopen behavior must resolve saved intent against current local inventory and degrade to explicit recoverable states when restoration fails.

Finally, validation is cross-cutting. Representative plugin compatibility, transport-trust scenarios, and trust-restoration gates must be architected as explicit seams. The product is blocked precisely because seam-local checks alone were not enough to establish trust, so the architecture has to support stronger evidence than "this slice passed its local tests."

## Starter Template Evaluation

### Primary Technology Domain

Desktop application within an existing brownfield codebase.

Compose already runs on the intended foundation: Tauri 2 desktop shell, Rust-owned backend and runtime logic, React and TypeScript UI, Vite frontend build, and native score rendering via Skia and Metal. The project is not selecting a new starter template. It is resetting architecture around a cleaner VST-only playback foundation inside the current repository.

### Starter Evaluation Outcome

Starter-template selection is not an active architecture decision for this workflow.

The relevant architectural decision is to preserve the current Tauri + Rust + React + TypeScript + Vite foundation and explicitly avoid reinitialization, replatforming, or starter-driven churn during the playback reset.

### Canonical Foundation

The current canonical base remains:

- Tauri 2 for desktop shell and windowing
- Rust for backend ownership, runtime logic, persistence, and performance-sensitive playback behavior
- React 19 and TypeScript 5.9 for the frontend display and control layer
- Vite 7 for frontend development and build workflow
- Tailwind-based existing Compose design system for UI styling

### Reference-Only Initialization Equivalents

If this architecture ever needed to be described in scaffold terms, the closest official equivalent would still be the Tauri React + TypeScript path, either through `create-tauri-app` or through manual Vite + Tauri CLI setup.

Those commands are reference-only and must not be treated as implementation work for this project.

### Architectural Implication

The reset effort must focus on foundation cleanup, ownership clarification, and deletion of obsolete playback seams inside the existing codebase.

Project initialization is not an implementation story.
Starter choice is not an open decision.
The architecture should treat the current foundation as fixed and spend decision-making effort on playback ownership, persistence boundaries, runtime safety, and seam deletion instead.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**

- Rust remains the sole authoritative owner of plugin inventory truth, rack state, slot state, route state, playback-preparation state, transport state, and persisted playback-domain state.
- The active playback product path must be truly VST-owned. Sampler-era and SCVI-era seams must not remain active product-path owners once equivalent VST seams exist.
- `.soloc` persists portable project intent and playback references only; machine-local plugin discovery, validation, and cache state remain outside project files.
- The playback runtime is split into a Rust control plane and a realtime-safe playback plane, with the audio path consuming precomputed, lock-safe state only.
- Stable, trustable transport behavior is a hard architectural gate, not a later polish concern.
- The score-context versus app-scoped UX boundary is architectural, not cosmetic: score-context surfaces own routing, readiness, and recovery relationships, while app-scoped windows own plugin inventory, reusable playback assets, and deep editors.
- Reinitialization or replatforming is out of scope. The existing Tauri + Rust + React + TypeScript + Vite foundation remains canonical.

**Important Decisions (Shape Architecture):**

- No product-level authentication or cloud identity model is introduced for MVP.
- VST3 plugins are treated as untrusted third-party code from a host-safety perspective.
- Native plugin editor hosting remains a bounded backend-owned seam, secondary to Compose-owned playback UX.
- The current reset phase is intentionally single-instrument and single-target trust-first; richer multi-target or HALion-style workflows are explicitly deferred until the base path is trustworthy.
- Validation and trust-restoration criteria must be explicit architectural seams, because seam-local tests alone are not enough to claim product trust.

**Deferred Decisions (Post-MVP or Post-Reset):**

- Broader plugin compatibility heuristics for problematic or partially compliant plugins
- Richer multi-target routing and shared-slot sophistication
- Expanded diagnostics polish and maintainability tooling
- Broader expression-map sophistication beyond the first durable model
- Any future platform expansion beyond macOS MVP

### Data Architecture

- Rust owns the canonical domain model for:
  - plugin inventory truth
  - rack, slot, target, address, and route state
  - playback-preparation state
  - transport state
  - persisted playback-domain truth
- `.soloc` stores portable score and playback intent, not machine-local discovery or cache state.
- Machine-local plugin inventory, plugin validation results, editor-host validation results, and related environment-specific metadata live in app-local storage outside project files.
- Missing or invalid plugin restoration degrades to explicit recoverable unrouted or attention-needed states instead of silent fallback.
- Persisted-data changes continue to follow the existing migration discipline in `src-tauri/src/migrations/next.rs` until human release preparation promotes them.

**Rationale:**
This preserves file portability, prevents environment-specific corruption of project data, and keeps playback truth in one backend-owned domain instead of splitting it between project files, local cache, and frontend-derived state.

### Authentication & Security

- No product-level authentication is introduced.
- No app-internal authorization layer is introduced beyond local desktop access.
- Security focus is local-desktop integrity, host-safety boundaries, and persisted-data integrity.
- VST3 plugins are treated as untrusted code:
  - host failures must degrade predictably
  - plugin/editor failures must not silently mutate project truth
  - plugin-related instability must not become accepted transport behavior
- Discovery remains limited to expected local VST3 locations and user-opened project data.

**Rationale:**
The real security problem in this product is not identity. It is safe local execution, host containment, and reliable persistence under third-party plugin behavior.

### API & Communication Patterns

- Internal FE/BE mutations remain command-oriented through typed Tauri command boundaries.
- Shared FE/BE payloads keep snake_case field names aligned with Rust-owned models.
- React does not own a second playback topology model, route model, or transport model.
- Playback-critical runtime state avoids event-heavy chatter; snapshot, polling, or other lock-safe state-consumption paths remain preferred where required by the transport model.
- MCP and automation surfaces must reuse the same backend-owned seams rather than creating parallel logic.
- Error handling is standardized around backend-owned categories such as:
  - recoverable routing issues
  - missing or unavailable plugins
  - plugin host/editor failures
  - persistence and migration validation failures
  - transport/runtime trust failures

**Rationale:**
The system already suffers from ambiguity when ownership drifts. These communication rules keep truth concentrated in Rust and reduce the risk of frontend drift or automation-only shortcuts.

### Frontend Architecture

- React remains a thin display and control layer.
- Local frontend state is limited to transient presentation concerns such as open/closed state, hover, draft input, and window-local affordances.
- Score-context surfaces own:
  - route visibility
  - readiness summary
  - contextual recovery
  - contextual handoff into deeper windows
- App-scoped windows own:
  - plugin inventory management
  - reusable expression-map assets
  - deep editing workflows
- Native plugin UI is a secondary handoff surface, not the primary home of Compose-owned playback semantics.
- Compact user-facing readiness states remain presentation summaries over deeper backend truth, not replacements for it.
- UI added solely to prove acceptance criteria separately is an architectural anti-pattern unless it creates real product value.

**Rationale:**
The user’s main complaint was not lack of controls. It was too much literalized UI growing around the wrong seams. These decisions keep the product direct and reduce the chance of future AC-driven bloat.

### Infrastructure & Deployment

- The app remains a local macOS desktop product delivered through the existing Tauri packaging and release process.
- There is no hosted backend or cloud runtime in MVP.
- CI and validation focus on:
  - build integrity
  - lint and type checks
  - Rust quality gates
  - persistence and migration safety
  - playback/trust regression evidence
- Release actions remain human-controlled.
- Diagnostics and observability focus on:
  - plugin discovery and validation
  - route resolution
  - transport behavior
  - editor-host lifecycle
  - persistence and reopen recovery
- Trust-restoration verification for the reset phase must be explicit and product-facing, not only seam-local.

**Rationale:**
This keeps operations aligned with the actual product shape: a local desktop tool whose success depends on playback credibility, not service deployment complexity.

### Decision Impact Analysis

**Implementation Sequence:**

1. Preserve the existing desktop foundation and avoid replatforming or starter churn.
2. Remove or disconnect obsolete sampler-era and SCVI-era product-path ownership.
3. Formalize the VST-owned data model for inventory, rack, slot, route, and recovery state.
4. Reassert the control-plane versus realtime-playback-plane split.
5. Keep frontend surfaces aligned to score-context versus app-scoped ownership rules.
6. Define trust-restoration and validation seams before resuming first-audible-playback work.

**Cross-Component Dependencies:**

- Data architecture constrains persistence, reopen recovery, and route UX.
- Communication patterns depend on Rust remaining the sole playback-domain owner.
- Frontend architecture depends on the data model and score-context versus app-scoped split being explicit.
- Security and host-safety rules depend on bounded plugin/editor ownership and predictable failure handling.
- Validation architecture depends on all of the above being explicit enough to test as product behavior rather than only as local implementation slices.

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:**
The highest-risk conflict areas are:

- state ownership boundaries between Rust, persisted project data, machine-local app state, and React UI state
- naming consistency across Rust structs, TypeScript mirrors, Tauri commands, and automation-facing seams
- placement of playback, plugin-host, transport, persistence, and recovery logic
- degraded-state and failure-handling behavior for missing plugins, host issues, and transport-trust failures
- score-context versus app-scoped UI ownership
- validation and diagnostics patterns for trust-restoration work
- deletion discipline for obsolete sampler-era or SCVI-era seams

### Naming Patterns

**Shared Model Naming Conventions:**

- Any model crossing the Rust ↔ TypeScript boundary uses `snake_case` field names.
- Persisted `.soloc` payload fields must match Rust-owned field names directly.
- TypeScript mirrors of persisted or command-bound structures must preserve Rust names exactly.
- Tauri commands remain `snake_case`.

**Examples:**

- `flow_id`, `track_id`, `slot_id`, `route_id`, `plugin_id`
- `transport_get_state`
- `plugin_inventory_rescan_full`

**Code Naming Conventions:**

- Rust files and functions follow existing Rust conventions.
- TypeScript and TSX filenames remain kebab-case.
- React component exports remain PascalCase.
- Do not introduce a playback-specific naming dialect that differs from the rest of the repo.

**Examples:**

- file: `plugin-manager-window.tsx`
- component: `PluginManagerWindow`
- Rust file: `transport.rs`
- Rust function: `resolve_route_state`

**Domain Naming Rules:**

- Use `rack` for the score-context playback container.
- Use `slot` for one hosted plugin instance in that rack.
- Use `route` for the relationship from score instrument to playback destination.
- Use `target` for the product-facing playback destination.
- Use `address` only for engine-level or plugin-level addressing payloads.
- Use `inventory` for machine-local discovered plugin availability.
- Use `recovery` for reopen and re-resolution behavior against current local inventory.
- Use `unrouted`, `needs_attention`, and `unavailable` as presentation summaries over deeper backend truth.
- Do not reuse sampler-era or SCVI-era terms for VST-owned seams.

### Structure Patterns

**Project Organization:**

- Rust owns:
  - plugin inventory truth
  - rack/slot/route domain logic
  - playback preparation
  - transport behavior
  - persistence and migration logic
  - host/editor lifecycle
- React owns:
  - display
  - contextual controls
  - window surfaces
  - transient UI state only
- Native-rendered score behavior remains backend-owned.
- Do not place new playback-domain truth in React contexts or frontend utility folders.

**Placement Rules:**

- Persisted score/playback-domain logic belongs in backend score/files/migrations seams.
- Playback runtime logic belongs under `src-tauri/src/playback/`.
- FE→BE mutation entry points remain in `src/services/actions.ts`.
- Rust-aligned TypeScript mirrors remain in `src/services/types.ts`.
- Score-context playback UI belongs in score-owned surfaces.
- App-scoped inventory and deep-editor UI belongs in dedicated windows.
- Do not solve architectural uncertainty by adding parallel adapter layers in both frontend and backend.

**Tests and Validation Placement:**

- Keep regression coverage close to the owning seam.
- Persistence regressions live near load/save and migration seams.
- Playback regressions live near playback and transport seams.
- UI-only behavior lives in frontend tests when that test surface exists.
- Trust-restoration validation may span seams, but should still reuse local owning seams rather than invent a parallel framework.

### Format Patterns

**Persistence Format Rules:**

- `.soloc` stores portable project intent and playback references only.
- Machine-local discovery, validation, cache state, and environment-specific metadata must remain outside `.soloc`.
- Persisted playback failure or restoration state should preserve user intent, not overwrite it with guessed replacements.

**Error Format Rules:**

- Backend-owned error categories should distinguish:
  - recoverable routing issues
  - missing/unavailable plugins
  - host/editor failures
  - persistence or migration validation failures
  - transport/runtime trust failures
- UI may summarize those categories, but must not invent a second incompatible taxonomy.

**Diagnostic Format Rules:**

- Diagnostics intended for maintainers or tooling should use structured backend-owned payloads, not ad hoc string logging alone.
- Reset or cleared states should be represented explicitly with null-safe structured payloads rather than frontend fallback invention.
- Trust-restoration checks must record product-facing outcomes, not only local implementation pass/fail states.

### Communication Patterns

**Frontend ↔ Backend Communication Rules:**

- All authoritative mutations go through backend-owned command seams.
- Frontend must not derive or cache a second source of route, transport, or playback truth.
- Playback-critical runtime state avoids high-frequency event chatter.
- Use polling, snapshots, or other lock-safe state-consumption paths where the transport model requires them.

**Automation/MCP Rules:**

- Automation must reuse the same backend seams as the product UI whenever practical.
- Do not create an automation-only playback domain path.
- Tool-facing adapters may reshape parameters, but domain logic remains shared.

**State Ownership Rules:**

- Rust owns:
  - plugin inventory truth
  - rack/slot/route truth
  - playback-preparation truth
  - transport truth
  - persisted playback-domain truth
- machine-local app storage owns:
  - discovery cache
  - validation cache
  - environment-specific inventory metadata
- React local state owns:
  - transient presentation state
  - panel/window affordance state
  - draft inputs that are not domain truth

### Process Patterns

**Error Handling Patterns:**

- Missing plugins degrade to explicit recoverable unrouted or attention-needed states.
- Host/editor failure must not silently rewrite playback intent.
- Transport-trust failures are architectural defects, not ordinary UI warnings.
- When a VST-owned seam exists, legacy sampler-era or SCVI-era fallback must be removed or disconnected rather than quietly preserved.

**Loading and Recovery Patterns:**

- Reopen resolves persisted playback intent against current machine-local inventory.
- If resolution fails, preserve user intent and expose recoverable unresolved state.
- Never silently remap to a different plugin unless an explicit product rule authorizes it.
- Same-session activation behavior and cross-session reopen behavior remain separate concerns and should not be conflated.

**UI Scope Patterns:**

- Score-context surfaces explain relationships, readiness, and next actions.
- App-scoped windows own reusable inventories, assets, and deep editing.
- Do not add extra controls purely to make acceptance criteria separately visible.
- If one compact control expresses the useful product action, prefer it over multi-control proof surfaces.

**Deletion Patterns:**

- When replacing sampler-era or SCVI-era behavior, agents must remove or disconnect obsolete product-path ownership.
- Temporary bridging is allowed only when explicitly approved and must be named as temporary.
- Legacy seams must not be preserved as hidden compatibility residue after the replacement seam is accepted.

### Enforcement Guidelines

**All AI Agents MUST:**

- preserve Rust as the sole playback-domain owner
- keep `.soloc` portable and machine-local discovery state separate
- avoid introducing frontend-owned parallel playback truth
- follow existing snake_case shared-model conventions
- preserve score-context versus app-scoped UI boundaries
- keep validation near the owning seam
- treat stable transport and trust-restoration as release-gating architecture concerns
- delete or disconnect obsolete legacy playback paths once a VST-owned replacement exists

**Pattern Enforcement:**

- Review persisted-field additions against migration rules and portable-data boundaries.
- Review new frontend state against ownership rules before accepting it as domain state.
- Review new playback and transport logic for event chatter, lock-safety violations, and fallback ambiguity.
- Treat any parallel truth between Rust and React as an architectural defect unless explicitly justified.

### Pattern Examples

**Good Examples:**

- Persisting route and slot references in `.soloc` while keeping plugin discovery cache outside the file
- Using one backend command to mutate route state and returning updated Rust-owned state to the UI
- Showing `Ready`, `Needs Attention`, or `Unavailable` in the UI while preserving richer backend reason codes
- Opening Plugin Manager for app-scoped inventory work while keeping rerouting entry points inside score-context surfaces
- Removing legacy sampler-owned runtime wiring once the equivalent VST-owned seam is established

**Anti-Patterns:**

- storing machine-local plugin discovery or host validation cache inside project files
- adding React-owned copies of rack, route, or transport truth
- using sampler-era names or contracts for new VST-owned seams
- silently falling back to legacy SCVI or sampler logic when a VST path fails
- adding explanatory UI panels or multiple overlapping controls just to prove acceptance criteria
- inventing automation-only domain seams that bypass product logic

## Project Structure & Boundaries

### Complete Project Directory Structure

```text
compose/
├── package.json
├── vite.config.ts
├── eslint.config.js
├── postcss.config.js
├── tsconfig.json
├── tsconfig.node.json
├── styles.css
├── README.md
├── TESTING.md
├── releases.md
├── docs/
│   ├── index.md
│   ├── architecture.md
│   ├── component-inventory.md
│   ├── data-models.md
│   ├── development-guide.md
│   ├── performance-baselines.md
│   ├── project-context.md
│   ├── project-overview.md
│   ├── project-scan-report.json
│   └── source-tree-analysis.md
├── src/
│   ├── __mocks__/
│   ├── assets/
│   ├── components/                 # shared app-level React components
│   ├── instruments/
│   ├── services/
│   │   ├── actions.ts              # FE→BE command seam
│   │   ├── types.ts                # Rust-aligned TS model mirrors
│   │   ├── state/                  # provider/context layer over Rust-owned state
│   │   ├── test-utils.tsx
│   │   └── *.test.ts(x)            # seam-local frontend tests
│   ├── ui/                         # low-level reusable UI primitives
│   ├── utils/
│   ├── windows/
│   │   ├── main/
│   │   │   ├── boot.tsx
│   │   │   ├── index.tsx
│   │   │   ├── components/
│   │   │   ├── hooks/
│   │   │   ├── screens/
│   │   │   │   ├── setup/
│   │   │   │   ├── write/
│   │   │   │   └── sequence/
│   │   │   └── utils/
│   │   ├── debugger/
│   │   ├── instrument-picker/
│   │   ├── layout-settings/
│   │   ├── player-settings/
│   │   ├── plugin-host-poc/
│   │   ├── plugin-manager/
│   │   ├── plugin-settings/
│   │   ├── preferences/
│   │   └── project-settings/
│   ├── test-setup.ts
│   └── vite-env.d.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs                  # app wiring and managed state
│       ├── files.rs                # load/save/open seams
│       ├── soloc.rs
│       ├── preferences.rs
│       ├── utils.rs
│       ├── automation/
│       ├── bin/
│       ├── instructions/
│       │   ├── draw/
│       │   ├── parse/
│       │   ├── render_plan.rs
│       │   └── mod.rs
│       ├── migrations/
│       │   ├── next.rs
│       │   └── versioned migrations
│       ├── playback/
│       │   ├── audio.rs
│       │   ├── mixer.rs
│       │   ├── plugin_editor_poc.rs
│       │   ├── plugin_host_compatibility.rs
│       │   ├── plugin_inventory.rs
│       │   ├── runtime_activation.rs
│       │   ├── sampler.rs
│       │   ├── scvi.rs
│       │   ├── streamer.rs
│       │   ├── transport.rs
│       │   ├── tuplet_projection.rs
│       │   ├── vst3.rs
│       │   └── mod.rs
│       ├── renderer/
│       │   ├── backend.rs
│       │   ├── backends/
│       │   ├── converter.rs
│       │   ├── fonts.rs
│       │   ├── hit_test.rs
│       │   ├── text_measurer.rs
│       │   └── mod.rs
│       ├── score/
│       │   ├── components/
│       │   ├── entries/
│       │   ├── instruments/
│       │   ├── config.rs
│       │   ├── engraves.rs
│       │   ├── flows.rs
│       │   ├── layouts.rs
│       │   ├── meta.rs
│       │   ├── playback.rs
│       │   ├── players.rs
│       │   ├── staves.rs
│       │   ├── tracks.rs
│       │   └── mod.rs
│       └── ui/
├── tests/                          # currently empty; no separate global test tree in active use
└── _bmad-output/
    ├── planning-artifacts/
    └── implementation-artifacts/
```

### Architectural Boundaries

**Runtime and Domain Boundaries:**

- `src-tauri/src/score/` is the canonical persisted score and playback-intent domain.
- `src-tauri/src/playback/` owns plugin inventory truth, runtime activation, playback preparation, transport behavior, mixer behavior, host integration, and recovery-facing playback logic.
- `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, and `src-tauri/src/migrations/` own load/save/file-format/migration boundaries.
- `src-tauri/src/renderer/` and `src-tauri/src/instructions/` own native score rendering and render planning.
- `src-tauri/src/ui/` may expose Rust-owned UI-facing state, but it must not become a second playback-domain owner.

**Frontend Boundaries:**

- `src/services/actions.ts` is the canonical FE→BE mutation entry point.
- `src/services/types.ts` mirrors Rust-owned models and must not define competing playback truth.
- `src/services/state/` provides React contexts over backend-owned state.
- `src/windows/main/screens/sequence/`, `write/`, and related main-window surfaces own score-context playback visibility and interaction.
- `src/windows/plugin-manager/` and future app-scoped playback windows own inventory and reusable asset management, not score-context routing truth.
- `src/windows/plugin-host-poc/` remains a bounded editor-host proof seam, not the future owner of general playback semantics.

**Legacy-Seam Boundary:**

- `src-tauri/src/playback/sampler.rs` and `src-tauri/src/playback/scvi.rs` are legacy seams and reset targets.
- The reset architecture does not extend them as active product-path owners.
- Any remaining dependency on them must be treated as temporary bridging and named explicitly.

### Requirements to Structure Mapping

**Foundation Reset / Stabilization Mapping:**

- Legacy seam deletion and VST-only ownership:
  - `src-tauri/src/playback/`
  - `src-tauri/src/files.rs`
  - `src/services/state/`
  - `src/services/actions.ts`
  - score-context playback windows under `src/windows/main/`
- Trust-restoration criteria and diagnostics:
  - `src-tauri/src/playback/`
  - `docs/performance-baselines.md`
  - `TESTING.md`
  - planning artifacts under `_bmad-output/`

**Plugin Discovery and Inventory:**

- Backend truth:
  - `src-tauri/src/playback/plugin_inventory.rs`
  - related persistence/config seams in backend
- Frontend surfaces:
  - `src/windows/plugin-manager/`
  - filtered contextual consumers in `src/windows/main/`

**Rack, Slot, Route, and Recovery:**

- Backend truth:
  - `src-tauri/src/score/playback.rs`
  - `src-tauri/src/playback/runtime_activation.rs`
  - `src-tauri/src/playback/vst3.rs`
  - `src-tauri/src/files.rs`
- Frontend score-context surfaces:
  - `src/windows/main/screens/sequence/`
  - `src/windows/main/screens/write/`

**Transport and Audible Playback:**

- Backend:
  - `src-tauri/src/playback/transport.rs`
  - `audio.rs`
  - `streamer.rs`
  - `mixer.rs`
- Frontend:
  - main-window transport and playback controls in `src/windows/main/`

**Expression Maps and Reusable Playback Assets:**

- Backend domain:
  - score/playback-related seams under `src-tauri/src/score/` and `src-tauri/src/playback/`
- Frontend:
  - app-scoped window surfaces, not selection-owned panels
- Current implication:
  - new reusable-asset editors should be added as dedicated windows alongside existing app-scoped windows

### Integration Points

**Internal Communication:**

- React issues mutations through `src/services/actions.ts`.
- Rust mutates authoritative state and emits or exposes updated state through the existing state providers and transport-safe channels.
- Playback-critical state must avoid high-frequency event chatter and use the transport-safe patterns already established.

**External Integrations:**

- VST3 discovery and loading integrate through backend-owned local desktop seams in `src-tauri/src/playback/`.
- Native editor-host behavior integrates through backend/runtime ownership plus bounded frontend triggers.
- Automation surfaces under `src-tauri/src/automation/` must reuse the same backend logic paths as the product.

**Data Flow:**

1. User acts in score-context or app-scoped React UI.
2. Frontend calls backend command through `actions.ts`.
3. Rust mutates canonical score, playback, or machine-local inventory state.
4. Rust persists only portable project intent to `.soloc`.
5. Rust exposes updated UI state back to the frontend.
6. The realtime playback plane consumes precomputed lock-safe state, not ad hoc UI-driven state.

### File Organization Patterns

**Configuration Files:**

- Root config files stay at the repo root for frontend/build/tooling concerns.
- Tauri and Rust build configuration stays under `src-tauri/`.
- Planning and architectural rules live in `docs/` and `_bmad-output/`.

**Source Organization:**

- Backend truth stays under `src-tauri/src/`.
- Frontend UI stays under `src/`.
- Shared UI primitives stay under `src/ui/`.
- Shared app-level components stay under `src/components/`.
- Multi-window code stays under `src/windows/`.
- Do not move domain logic into frontend convenience modules.

**Test Organization:**

- The repo currently favors co-located frontend tests near the owning files.
- The top-level `tests/` directory is not the active home of core product tests today.
- Backend seam validation should remain close to backend seams rather than depending on a new cross-cutting test harness by default.

**Asset Organization:**

- Frontend static assets remain under `src/assets/`.
- Machine-local runtime caches and plugin inventory metadata must remain outside source-controlled project assets.
- Persisted project payloads remain `.soloc`, with migrations handled in backend migration seams.

### Development Workflow Integration

**Development Server Structure:**

- Frontend development continues through Vite-backed multi-window entry points under `src/windows/`.
- Backend development continues through Tauri/Cargo with `src-tauri/src/lib.rs` as the central wiring point.

**Build Process Structure:**

- Vite builds the frontend entry points.
- Cargo/Tauri builds the desktop runtime and native integrations.
- Migration work stays inside `src-tauri/src/migrations/`.
- Release workflows remain human-controlled and should not be encoded as autonomous agent behavior.

**Deployment Structure:**

- The structure supports a local macOS desktop app with:
  - offline-first project use
  - local VST3 discovery
  - native plugin-host and renderer integration
  - portable project persistence
- No cloud deployment structure is required for MVP.

## Architecture Validation Results

### Coherence Validation

**Decision Compatibility:**
The architectural decisions are internally coherent. The preserved Tauri + Rust + React + TypeScript + Vite foundation aligns with the current brownfield codebase and with the approved reset direction. Ownership boundaries, persistence rules, communication rules, and UI-scope rules all reinforce the same model: Rust owns playback-domain truth, React presents and triggers, and obsolete sampler-era or SCVI-era ownership must be removed rather than tolerated.

Technology-version coherence is also sound. The current stack remains aligned with the validated project context: React 19.x, Vite 7.x, Tailwind 4.x, and Tauri 2.x all fit the architecture described here because this document is not proposing a framework pivot.

**Pattern Consistency:**
The implementation patterns support the architectural decisions well. Shared-model naming rules, command-boundary rules, portability rules for `.soloc`, score-context versus app-scoped UI rules, and deletion patterns all reinforce the main objective of the reset: eliminate ambiguity and prevent future agents from reintroducing parallel truth or legacy-residue ownership.

The strongest consistency success is that the architecture now treats seam deletion as a first-class rule rather than a vague cleanup preference.

**Structure Alignment:**
The project structure section aligns with the architecture because it maps decisions onto the real repository rather than a hypothetical future layout. Backend playback and persistence seams, frontend action/state layers, main score-context windows, and app-scoped windows all have concrete homes in the current repo. This should make the architecture implementation-guiding rather than aspirational.

### Requirements Coverage Validation

**Feature and Epic Coverage:**
The architecture supports the product capabilities described in the PRD and UX spec: plugin discovery, inventory management, rack/slot/route ownership, transport behavior, persistence and reopen recovery, bounded editor hosting, and future reusable playback assets.

However, one planning artifact mismatch remains visible: the approved 2026-03-24 change proposal inserts a stabilization/reset epic before first-audible playback, but the current epic breakdown artifact still shows the old sequencing. The architecture accounts for the approved reset direction, so this is a planning-sync issue rather than an architectural gap.

**Functional Requirements Coverage:**
The architecture provides explicit support for:

- plugin inventory and discovery
- score-context routing and readiness
- bounded plugin lifecycle and editor hosting
- score-to-playback preparation
- transport-safe playback architecture
- reopen and recovery behavior
- durable future playback-asset seams such as expression maps
- release-trust validation

The most important change is that the architecture now also covers an implicit but critical requirement discovered through course correction: once VST-owned seams exist, obsolete sampler-era or SCVI-era product-path ownership must be removed.

**Non-Functional Requirements Coverage:**
The architecture addresses the strongest NFRs:

- lock-safe realtime playback boundaries
- transport trust and coherence
- portable persisted data
- local machine-specific inventory separation
- predictable degraded states
- host-safety boundaries for VST3 plugins
- compact score-first UI boundaries
- trust-restoration and validation seams

### Implementation Readiness Validation

**Decision Completeness:**
Critical architectural decisions are now explicit enough to guide implementation. The document does not waste time re-deciding irrelevant greenfield concerns such as starter choice, cloud hosting, or auth models that do not apply to this product. Instead, it centers the real blocking decisions: ownership, persistence, runtime boundaries, UI scope boundaries, and deletion rules.

**Structure Completeness:**
The project structure section is sufficiently concrete for a brownfield implementation guide. It maps the current repository to backend playback seams, persistence seams, rendering seams, score-context surfaces, and app-scoped windows without inventing unnecessary reorganization.

**Pattern Completeness:**
The implementation patterns address the most likely future agent-conflict points:

- shared-model naming drift
- React-side parallel truth
- hidden legacy fallback
- misplaced playback logic
- acceptance-criteria-driven UI sprawl
- automation-only divergence
- ambiguous recovery behavior

### Gap Analysis Results

**Critical Gaps:**
No critical architectural gaps were found that would block implementation planning.

**Important Gaps:**

- The planning artifacts are not yet fully synchronized: the architecture now reflects the approved reset, but `_bmad-output/planning-artifacts/epics.md` still describes the old epic sequence.
- The PRD and UX artifacts remain directionally useful, but they should be rebaselined explicitly around the stabilization/reset epic so later story prep does not inherit outdated sequencing assumptions.
- The trust-restoration criteria are now architecturally named, but they still need explicit story-level operationalization before implementation resumes.

**Nice-to-Have Gaps:**

- A more formal plugin identity and re-resolution scheme could be documented later if reopen/recovery work needs finer precision.
- A more explicit backend error taxonomy could be added later if implementation complexity grows.
- A future expression-map asset/editor seam diagram may help once that scope becomes near-term again.

### Validation Issues Addressed

The main validation issue was whether the architecture was still silently inheriting stale planning assumptions from the pre-reset epic breakdown. That has been addressed by making the reset/stabilization phase an explicit architectural assumption, treating the current epic mismatch as an external planning-sync issue rather than allowing it to distort the architecture itself.

### Architecture Completeness Checklist

**Requirements Analysis**

- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**Architectural Decisions**

- [x] Critical decisions documented
- [x] Canonical foundation confirmed
- [x] Integration patterns defined
- [x] Performance and trust constraints addressed

**Implementation Patterns**

- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process and deletion patterns documented

**Project Structure**

- [x] Concrete directory structure defined
- [x] Component and seam boundaries established
- [x] Integration points mapped
- [x] Requirements-to-structure mapping completed

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION PLANNING

**Confidence Level:** High

**Key Strengths:**

- strong alignment with the actual brownfield repo
- explicit Rust-owned playback-domain model
- explicit deletion-first treatment of obsolete seams
- score-context versus app-scoped UX boundary clarified
- persistence portability and machine-local inventory separation made explicit
- architecture now reflects the approved reset direction rather than the obsolete forward-only sequence

**Areas for Future Enhancement:**

- synchronize PRD and epic planning artifacts to the reset architecture
- define trust-restoration criteria at story/spec level
- add more explicit reopen identity/re-resolution rules if needed during implementation planning
- expand reusable playback-asset architecture when expression-map work becomes active again

### Implementation Handoff

**AI Agent Guidelines:**

- preserve Rust as the only authoritative playback-domain owner
- do not preserve legacy sampler-era or SCVI-era product-path ownership once VST-owned seams exist
- keep `.soloc` portable and machine-local inventory state separate
- keep score-context and app-scoped UI responsibilities distinct
- treat transport trust and trust-restoration as hard gating concerns, not polish

**First Implementation Priority:**
Implement the stabilization/reset phase inside the existing codebase: delete or disconnect obsolete legacy playback-path ownership, formalize the VST-owned inventory/rack/route/recovery seams, and define the trust-restoration checks required before first-audible-playback stories resume.
