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
  - _bmad-output/planning-artifacts/archive/2026-03-multi-flow-score/prd.md
  - _bmad-output/planning-artifacts/archive/2026-03-multi-flow-score/prd-validation-report.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
  - _bmad-output/planning-artifacts/research/technical-testing-and-tooling-research-2026-03-04.md
  - docs/index.md
  - docs/architecture.md
  - docs/component-inventory.md
  - docs/data-models.md
  - docs/development-guide.md
  - docs/project-overview.md
  - docs/source-tree-analysis.md
  - docs/project-context.md
workflowType: "architecture"
project_name: "compose"
user_name: "Aled"
date: "2026-03-11"
lastStep: 8
status: "complete"
completedAt: "2026-03-11"
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
The current requirement set describes a focused but technically demanding enhancement to an existing desktop notation application. The core functional scope is continuous multi-flow score presentation, with users able to read, navigate, select, and edit across multiple visible flows without falling back to the current single-flow switching model for basic work. Architecturally, the functional requirements cluster into eight areas: score presentation and continuity, cross-flow interaction, notation integrity, navigation and reading, update isolation, desktop document workflows, playback boundaries, and future-ready extensibility.

The most important functional implication is that this is not a notation-logic rewrite. Multi-flow support is a presentation and orchestration capability layered onto an existing notation engine whose behavioral contract must remain intact. The architecture therefore needs clear separation between notation derivation, flow composition, interaction mapping, and any active-flow editing semantics. The desktop workflow requirements also confirm that document open/save behavior, native file interactions, and offline use remain first-class and cannot become secondary concerns while rendering evolves.

**Non-Functional Requirements:**
The non-functional requirements are dominated by performance and correctness. The architecture must preserve native-feeling responsiveness, avoid regressing current single-flow performance, and keep recalculation isolated so local edits do not force unnecessary work in unrelated flows. Correctness requirements are equally strict: multi-flow rendering must not introduce notation regressions, hit targets and popovers must remain accurate, and presentation changes must never compromise score integrity or persistence behavior.

The UX and supporting documentation add further non-functional direction. The product must remain macOS-first, offline-first, and accessible at a practical desktop baseline. Interaction precision matters more than visual novelty. The system also needs development-time visibility through timing instrumentation, targeted regression coverage, and debugging support so the rendering pipeline can evolve safely inside a brownfield architecture.

**Scale & Complexity:**
This is a medium-complexity brownfield project with concentrated technical depth rather than broad platform sprawl. The functional scope is narrower than a greenfield application, but the consequences of change are high because the work touches rendering composition, interaction resolution, performance isolation, and the contract between native score rendering and React-based controls.

- Primary domain: native desktop music composition and notation software
- Complexity level: medium
- Estimated architectural components: 8 major concern areas

The eight major concern areas are:

1. Score derivation and notation preservation
2. Multi-flow presentation composition
3. Hit-testing and interaction mapping
4. Active-flow and editing-state semantics
5. React shell and panel orchestration
6. Playback boundary management
7. Persistence and migration compatibility
8. Testing, profiling, and regression tooling

### Technical Constraints & Dependencies

The project is constrained by an existing brownfield architecture in which Rust owns application state, React acts as the UI shell, and the score surface is rendered natively rather than in HTML. This means architectural decisions must respect the existing backend/frontend boundary rather than blur it. State mutation continues to flow through backend commands, and score interaction remains dependent on accurate mapping between native-rendered elements and frontend-driven controls such as popovers and properties panels.

Notation behavior is an explicit protected boundary. Existing written-duration and broader notation semantics must remain unchanged unless a separate notation initiative is undertaken. The architecture must also remain compatible with the existing persistence and migration model, because `.soloc` behavior and versioned migrations are already treated as strategic product infrastructure. The product is macOS-first, offline-first, and document-oriented, so native file workflows, local reliability, and non-network-dependent operation remain baseline constraints rather than optional qualities.

A further dependency is the current UX shell and information architecture direction. The product already favors VS Code-like structure, collapsible panels, contextual editing, and score-first hierarchy. Architectural decisions should support that shell rather than introduce interaction models that fight it.

### Cross-Cutting Concerns Identified

The first major cross-cutting concern is notation trust. Any architectural change that weakens written output quality, visual coherence, or editing precision would damage the product's core value. This concern touches rendering, interaction, testing, and release safety.

The second is performance isolation. Multi-flow rendering must scale by minimizing unnecessary work, which means boundaries between derivation, layout, rendering, and local edit invalidation need to stay explicit. This concern affects score parsing, render orchestration, UI responsiveness, and profiling.

The third is interaction consistency across system boundaries. The product depends on a native-rendered score surface paired with React controls, so selection, hit targets, popovers, and active-flow behavior must remain logically unified even though implementation responsibilities are split across layers.

The fourth is future layout extensibility. Multi-flow rendering is intended as a step toward more document-like score layout, including frame-based and paginated presentation. Architectural choices made now should therefore avoid reinforcing single-flow assumptions that would later require redesign.

The fifth is delivery safety in a brownfield environment. Regression coverage, fixture-driven validation, timing instrumentation, migration awareness, and debugging support are all architectural enablers, not just team process details, because they determine whether this system can evolve without destabilizing hard-won behavior.

## Starter Template Evaluation

### Primary Technology Domain

Desktop application architecture based on a native Tauri shell, Rust-owned application state, and a React/TypeScript UI layer.

This classification is driven by the current product architecture and by the project requirement to evolve an existing macOS-first notation application rather than create a new greenfield product.

### Starter Options Considered

For this project, a conventional greenfield starter template is not the right architectural foundation because the product already has a working and opinionated runtime stack. The relevant options are therefore not "which new starter should we adopt?" but rather "should we preserve the current foundation or replace it with a different one?"

The practical foundation options considered are:

1. Preserve the existing Tauri + React + Rust architecture
2. Replatform to Electron-based desktop architecture
3. Replatform to a web-first stack and reintroduce native concerns later
4. Introduce a new frontend or full-stack boilerplate on top of the existing app

Options 2 through 4 are poor fits for the current project scope. They would introduce architectural churn without helping the actual feature goal, which is multi-flow score presentation inside an existing native notation system. They would also increase regression risk in rendering, interaction mapping, persistence, and performance.

### Selected Starter: Existing Brownfield Foundation

**Rationale for Selection:**
The correct architectural foundation is the existing project stack rather than a new external starter. Compose already has the critical decisions in place that matter for this feature: Rust owns state, the score surface is rendered natively, React serves as the UI shell, persistence and migrations already exist, and the application is optimized for offline-first macOS desktop use.

The current initiative is a brownfield architectural extension, not a greenfield application bootstrap. Replacing the foundation would distract from the actual problem and create unnecessary implementation risk. The architecture work should therefore treat the current repository as the starter baseline and document how to evolve it safely.

**Initialization Command:**

```bash
No new starter initialization command applies.
Use the existing repository and current application architecture as the implementation baseline.
```

**Architectural Decisions Provided by Starter:**

**Language & Runtime:**
The foundation already standardizes on Rust for core state, rendering, playback, and persistence logic, with TypeScript and React for the control surface and supporting UI.

**Styling Solution:**
The project already uses Tailwind CSS and an existing custom UI layer, which aligns with the current UX direction and should remain the basis for incremental UI evolution.

**Build Tooling:**
The foundation already uses Vite for the frontend and Tauri for native packaging, with Rust build optimization configured for performance-sensitive runtime behavior.

**Testing Framework:**
The project has an evolving testing story rather than a mature starter-provided one. Architectural planning should treat testing and profiling as first-class support capabilities to be strengthened alongside feature delivery.

**Code Organization:**
The current repository already establishes the key architectural partitions: Rust score/data/render/audio domains, React UI primitives and window-specific code, and a strict frontend/backend mutation boundary via invoke commands.

**Development Experience:**
The existing setup already provides the relevant workflow foundation for this project: hot-reload-oriented frontend development, typed frontend/backend contracts, native desktop execution, fixture-based validation, and migration-aware persistence work.

**Note:** For this project, implementation should begin from the current repository state. The first implementation stories should extend the existing architecture rather than initialize a new codebase.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**

- Preserve the existing Rust-owned score model as the single source of truth for multi-flow work
- Implement multi-flow presentation as a derived composition layer rather than a new persisted score structure
- Preserve the existing frontend-to-backend command boundary and typed state propagation model
- Define explicit invalidation and caching boundaries so local flow edits do not trigger unnecessary recalculation elsewhere
- Treat regression coverage, fixture validation, and render timing instrumentation as mandatory architectural support capabilities

**Important Decisions (Shape Architecture):**

- Keep React responsible for UI orchestration and ephemeral interaction state only
- Keep active-flow semantics distinct from visible-flow presentation
- Extend existing selection, hit-testing, and popover contracts rather than introducing a parallel interaction model
- Keep persistence changes minimal and migration-gated only when document semantics truly change

**Deferred Decisions (Post-MVP):**

- Cross-flow playback behavior
- External automation or API surfaces
- Cloud-connected services or sync
- Broader deployment and operational monitoring concerns beyond local development and test instrumentation

### Data Architecture

The data architecture remains centered on the current Rust-owned in-memory document model. Multi-flow support should not introduce a second source of truth or a parallel presentation datastore. Existing score, flow, stave, and track structures remain authoritative.

Multi-flow presentation should be implemented as a derived orchestration layer over existing flow data. The architecture should compose multiple flows into one visible score surface without duplicating notation state or denormalizing written output into a separate long-lived representation.

Caching should remain computation-oriented rather than persistence-oriented. The important caching boundary is render and derivation reuse, especially at the flow level or the smallest practical invalidation unit. Editing one flow should only invalidate the derivation and presentation artifacts that actually depend on that flow.

Persistence should remain on the existing `.soloc` and migration path. If the feature can be implemented without changing persisted score semantics, that is preferred. If persisted structure must change, migration work must be explicit and versioned through the established release flow.

### Authentication & Security

This feature does not require an application-level authentication architecture. There is no user-account, tenancy, or session-management requirement in scope for the current product direction.

Security for this feature is primarily local-system and data-integrity oriented. The architecture should preserve strict command boundaries between frontend and backend, maintain safe file loading and migration behavior, and avoid introducing presentation-layer changes that could corrupt score data or document state.

If future external control or automation surfaces are added, authentication and authorization should be introduced as a separate architectural concern rather than folded prematurely into this feature.

### API & Communication Patterns

The frontend-to-backend interaction model should remain command-oriented. All durable score mutations should continue to flow through explicit backend commands rather than frontend-owned state mutation.

State propagation should continue using the current event-based and polling-based mechanisms already established by the application architecture. This feature should not introduce a competing communication paradigm.

For this initiative, the most important API surface is internal rather than public. The architecture should define and preserve explicit contracts between notation derivation, multi-flow presentation composition, hit-testing, selection mapping, and UI-facing interaction state.

Error handling should remain boundary-oriented: invalid commands, unsupported state transitions, migration failures, and renderer inconsistencies should fail clearly without compromising document integrity.

### Frontend Architecture

The frontend remains a React orchestration shell rather than the owner of score truth. It should continue to manage view state, panel state, ephemeral interaction state, and command dispatch, while backend state remains authoritative for score content, selection payloads, and rendered interaction targets.

Multi-flow UI should preserve score-first boundaries. The renderer overlay, properties editing surfaces, popovers, and active-flow indicators should all coordinate around backend-derived score and selection state.

Active-flow semantics must remain logically distinct from visible-flow presentation. Multiple flows may be visible simultaneously while editing scope, playback scope, or other task-specific semantics remain attached to a single active flow.

The architecture should avoid introducing heavyweight new frontend state machinery unless concrete limitations appear in the current context-based model. Complexity should be added only where it resolves a real orchestration problem.

### Infrastructure & Deployment

The deployment architecture remains the current native desktop packaging model. No new hosting or cloud platform decision is required for this feature.

Operational architecture for this feature should focus on local development safety: render timing instrumentation, targeted regression testing, fixture coverage, and debugger support. These concerns are part of the implementation architecture because they determine whether the brownfield system can evolve safely.

Environment management, packaging, and platform targeting should remain aligned with the current macOS-first Tauri application structure.

### Decision Impact Analysis

**Implementation Sequence:**

1. Define the multi-flow composition boundary relative to existing flow and render structures
2. Define invalidation and caching rules for local versus cross-flow edits
3. Extend selection, hit-testing, and popover contracts for multi-flow visibility
4. Extend React shell orchestration for active-flow and panel behavior without moving score truth into the frontend
5. Add regression, fixture, and timing instrumentation to validate architectural safety

**Cross-Component Dependencies:**
These decisions tightly couple render orchestration, selection mapping, and frontend shell behavior. Data-architecture choices affect caching and invalidation. Interaction-contract choices affect renderer hit targets, frontend popovers, and properties editing. Persistence decisions affect migration requirements and release safety. Testing and instrumentation support every layer because they are required to verify that architectural boundaries remain intact during implementation.

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**Critical Conflict Points Identified:**
The highest-risk conflict areas for AI-agent implementation are:

1. Frontend/backend ownership boundaries
2. Multi-flow composition and invalidation boundaries
3. Selection, hit-testing, and popover contracts
4. Persistence and migration handling
5. File and symbol naming consistency
6. Test placement and regression coverage style
7. Error-handling and recovery behavior
8. Performance instrumentation and verification

These are the areas where different agents could make locally reasonable but globally incompatible choices if the patterns are not explicit.

### Naming Patterns

**Database Naming Conventions:**
This project does not introduce a traditional application database for the feature. Persisted score data remains part of the `.soloc` document model and supporting local metadata flows. If future persisted fields are added, serialized field names must follow the existing Rust and TypeScript cross-boundary convention of `snake_case`.

Examples:

- Good: `flow_id`, `time_signature_id`, `show_metronome`
- Avoid: `flowId`, `timeSignatureId`, `showMetronome`

**API Naming Conventions:**
Internal command and payload naming must remain aligned with the current Tauri command boundary. Frontend-to-backend parameters use `snake_case`, and backend commands remain explicit rather than generic mutation channels.

Examples:

- Good: `renderer_hit_test`, `transport_get_state`, payload fields such as `flow_id`
- Avoid: `rendererHitTest`, `getTransportState`, mixed camelCase payloads

**Code Naming Conventions:**
File naming should follow the repository’s established conventions:

- TypeScript and TSX files: `kebab-case`
- Rust files: `snake_case`
- React component exports: `PascalCase`
- Cross-boundary data fields: `snake_case`

Examples:

- Good: `use-selected-tempo.ts`, `time_signature.rs`, `Renderer`
- Avoid: `useSelectedTempo.ts`, `TimeSignature.rs`, `renderer_component.tsx`

### Structure Patterns

**Project Organization:**
Agents must preserve the existing separation of concerns:

- Rust score model changes go under the current score, instructions, renderer, playback, UI-state, or migration modules as appropriate
- React window and screen behavior stays under the current window/screen/component/hook layout
- Shared UI primitives belong in the existing UI layer, not inside feature-specific screens
- Shared parser test support belongs under `src-tauri/src/instructions/parse/test_support/` when it truly supports multiple parser scenarios

Tests should follow current project practice:

- TypeScript tests colocated with the source they verify
- Parser behavior tests kept close to the parser modules they protect
- Visual-regression fixture sources under `tests/fixtures/source/`

**File Structure Patterns:**
Agents should extend existing modules before creating new top-level organizational patterns. New files should appear only when the current module boundaries are no longer coherent.

Examples:

- Good: extending existing selection hooks, renderer modules, draw/parse modules, or write-screen properties components
- Avoid: creating parallel feature folders that duplicate existing domain structure

### Format Patterns

**API Response Formats:**
For this feature, the relevant API surface is the internal command boundary and event/state contract rather than a web JSON API. Commands should return focused, typed results and fail clearly at the boundary. Agents must not introduce ad hoc wrapper formats unless a boundary already expects them.

**Data Exchange Formats:**
Cross-boundary serialized structures must preserve:

- `snake_case` field naming
- existing enum/value synchronization rules between Rust and TypeScript
- explicit optional/null semantics rather than loosely inferred absence
- migration-aware evolution when persisted data changes

Examples:

- Good: add matching Rust and TypeScript fields together, preserve enum ordering/representation rules, add migration steps when older files would otherwise break
- Avoid: frontend-only field additions that silently diverge from backend shape, changing persisted semantics without migration review

### Communication Patterns

**Event System Patterns:**
Agents must preserve the current communication model:

- durable mutations go through backend commands
- pushed state continues through existing event channels
- transport behavior continues using the existing polling strategy where already established
- no parallel communication model should be introduced for convenience

Selection, hit-target, and popover coordination should use explicit typed payloads rather than inferred DOM state.

**State Management Patterns:**
Frontend state should remain limited to view orchestration and ephemeral interaction state. Backend state remains authoritative for score content, selection identity, and render-derived interaction targets.

Rules:

- Do not duplicate score truth in React state
- Do not introduce derived score caches in the frontend when the backend already owns the source model
- Keep active-flow semantics explicit and separate from visible-flow presentation state
- Prefer extending existing context and hook patterns before adding new global state layers

### Process Patterns

**Error Handling Patterns:**
Errors must fail at the correct boundary and preserve document integrity.

- Backend command failures should surface as clear operation failures
- Renderer inconsistencies should not silently mutate score state
- Migration and persistence failures should remain explicit
- User-facing errors should be concise and local where possible; internal debugging detail belongs in logs or developer tooling

**Loading State Patterns:**
Loading and busy-state handling should remain local and task-oriented.

- Use local loading indicators for command-driven UI work
- Avoid introducing broad global loading state for score interactions unless the whole application is genuinely blocked
- Keep layout stable while local panels, popovers, or controls update
- Do not fake optimistic score-state mutations on the frontend for backend-owned actions

### Enforcement Guidelines

**All AI Agents MUST:**

- Preserve Rust as the single source of truth for score state and durable document mutations
- Preserve the native-rendered score surface as the sole rendering path for score notation
- Preserve `snake_case` for all frontend/backend boundary fields and commands
- Preserve migration review for any persisted-data or `.soloc` behavior change
- Extend existing module boundaries before creating parallel architectural structures
- Add or update regression coverage when notation, selection, hit-testing, or render behavior changes
- Treat performance instrumentation as part of the feature contract when rendering behavior is affected

**Pattern Enforcement:**
Patterns should be verified through code review, type alignment between Rust and TypeScript, targeted regression tests, fixture coverage where relevant, and validation against documented project-context rules. Pattern violations should be corrected in the implementation before follow-on work builds on them.

### Pattern Examples

**Good Examples:**

- Adding a new persisted field to a Rust struct, updating the matching TypeScript type, and adding a pending migration if old files need transformation
- Extending existing renderer hit-target payloads rather than creating frontend-only coordinate approximations
- Keeping multi-flow presentation state derived from existing flow structures rather than creating duplicated presentation models
- Adding parser-level regression tests in the local parser area when notation behavior could be affected

**Anti-Patterns:**

- Storing score-authoritative multi-flow state in React for convenience
- Rendering score notation in HTML/React to avoid backend work
- Introducing camelCase fields across the Rust/TypeScript boundary
- Changing `.soloc` semantics without migration impact analysis
- Creating a second selection or popover model that competes with the renderer-owned interaction contract
- Adding new top-level organizational structures when existing domain modules already define the correct home

## Project Structure & Boundaries

### Complete Project Directory Structure

```text
compose/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── eslint.config.js
├── TESTING.md
├── docs/
│   ├── index.md
│   ├── architecture.md
│   ├── component-inventory.md
│   ├── data-models.md
│   ├── development-guide.md
│   ├── project-context.md
│   ├── project-overview.md
│   └── source-tree-analysis.md
├── src/
│   ├── services/
│   │   ├── actions.ts
│   │   ├── types.ts
│   │   ├── hit-test.ts
│   │   ├── popovers.ts
│   │   ├── sequencer-utils.ts
│   │   └── state/
│   ├── ui/
│   ├── components/
│   ├── windows/
│   │   ├── main/
│   │   │   ├── boot.tsx
│   │   │   ├── index.tsx
│   │   │   ├── components/
│   │   │   │   ├── renderer.tsx
│   │   │   │   ├── transport.tsx
│   │   │   │   ├── tools.tsx
│   │   │   │   ├── sequencer.tsx
│   │   │   │   ├── mixer.tsx
│   │   │   │   └── ...
│   │   │   ├── hooks/
│   │   │   ├── screens/
│   │   │   │   ├── setup/
│   │   │   │   ├── write/
│   │   │   │   │   ├── index.tsx
│   │   │   │   │   └── components/
│   │   │   │   └── sequence/
│   │   │   └── utils/
│   │   └── ...
│   ├── utils/
│   └── test-setup.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── files.rs
│   │   ├── soloc.rs
│   │   ├── migrations/
│   │   │   ├── mod.rs
│   │   │   └── next.rs
│   │   ├── score/
│   │   │   ├── mod.rs
│   │   │   ├── flows.rs
│   │   │   ├── tracks.rs
│   │   │   ├── staves.rs
│   │   │   ├── layouts.rs
│   │   │   ├── entries/
│   │   │   ├── instruments/
│   │   │   └── components/
│   │   ├── instructions/
│   │   │   ├── mod.rs
│   │   │   ├── parse/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── get_horizontal_spacing.rs
│   │   │   │   ├── get_vertical_spacing.rs
│   │   │   │   ├── get_written_durations.rs
│   │   │   │   ├── get_beams.rs
│   │   │   │   └── test_support/
│   │   │   └── draw/
│   │   │       ├── mod.rs
│   │   │       ├── draw_staves.rs
│   │   │       ├── draw_time_signature.rs
│   │   │       ├── draw_key_signature.rs
│   │   │       ├── draw_clef.rs
│   │   │       └── ...
│   │   ├── renderer/
│   │   │   ├── mod.rs
│   │   │   ├── hit_test.rs
│   │   │   ├── converter.rs
│   │   │   └── backends/
│   │   ├── playback/
│   │   ├── ui/
│   │   │   ├── mod.rs
│   │   │   ├── renderer.rs
│   │   │   ├── sequencer.rs
│   │   │   └── screens/
│   │   └── bin/
├── tests/
│   └── fixtures/
│       ├── source/
│       └── built/
└── _bmad-output/
    └── planning-artifacts/
        └── architecture.md
```

This feature should extend that structure through the following bounded areas rather than by creating new top-level subsystems:

- Multi-flow derivation and layout logic in `src-tauri/src/instructions/parse/`
- Multi-flow draw orchestration in `src-tauri/src/instructions/draw/` and `src-tauri/src/instructions/mod.rs`
- Render interaction and hit-target updates in `src-tauri/src/renderer/`
- Backend UI-state extensions in `src-tauri/src/ui/`
- Frontend renderer orchestration and write-screen behavior in `src/windows/main/`
- Shared frontend contracts in `src/services/`
- Regression and fixture support in `src-tauri/src/instructions/parse/test_support/`, parser-local tests, and `tests/fixtures/source/`

### Architectural Boundaries

**API Boundaries:**
There is no separate public web API boundary for this feature. The primary application boundary is the Tauri command interface between the React frontend and Rust backend. Durable mutations belong to backend commands, and typed state flows back through the existing event and polling mechanisms.

**Component Boundaries:**
The score surface remains a native-rendered backend responsibility. React components handle orchestration, panels, controls, local interaction state, and command dispatch. Score notation, hit-target truth, and selection identity must not be reimplemented in frontend components.

**Service Boundaries:**
Frontend shared services in `src/services/` define contracts, invoke calls, shared types, and interaction helpers. Backend service-like boundaries remain distributed across score, instruction-generation, renderer, playback, persistence, and UI-state modules. Multi-flow work should respect those domains rather than introduce a new cross-cutting god module.

**Data Boundaries:**
Authoritative musical and document state remains in Rust score and UI models. Multi-flow presentation data is derived from existing flow structures rather than stored as a second authoritative representation. Persisted-data boundaries remain governed by `.soloc` plus migration rules.

### Requirements to Structure Mapping

**Feature Mapping:**

- Score presentation and flow continuity
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/instructions/parse/`
  - `src-tauri/src/instructions/draw/`
  - `src-tauri/src/renderer/`
  - `src/windows/main/components/renderer.tsx`

- Interaction across flows
  - `src-tauri/src/renderer/hit_test.rs`
  - `src-tauri/src/ui/`
  - `src/services/hit-test.ts`
  - `src/services/popovers.ts`
  - `src/windows/main/screens/write/`
  - `src/windows/main/hooks/`

- Notation integrity and musical correctness
  - `src-tauri/src/instructions/parse/get_written_durations.rs`
  - `src-tauri/src/instructions/parse/get_beams.rs`
  - adjacent parser modules and their tests
  - `tests/fixtures/source/`

- Navigation and reading experience
  - `src-tauri/src/ui/renderer.rs`
  - `src/windows/main/components/renderer.tsx`
  - `src/windows/main/index.tsx`
  - write-screen components and hooks

- Update isolation and rendering behavior
  - `src-tauri/src/instructions/mod.rs`
  - parse/draw orchestration modules
  - renderer caching and invalidation code
  - render timing instrumentation points

- Desktop document workflow
  - `src-tauri/src/files.rs`
  - `src-tauri/src/soloc.rs`
  - `src-tauri/src/migrations/`

- Playback boundary preservation
  - `src-tauri/src/playback/`
  - `src/services/actions.ts`
  - transport-related frontend components

**Cross-Cutting Concerns:**

- Persistence and migration safety
  - `src-tauri/src/files.rs`
  - `src-tauri/src/soloc.rs`
  - `src-tauri/src/migrations/mod.rs`
  - `src-tauri/src/migrations/next.rs`

- Shared type alignment
  - `src/services/types.ts`
  - matching Rust structs and enums across `src-tauri/src/score/` and `src-tauri/src/ui/`

- Testing and regression support
  - colocated TS tests
  - parser-local Rust tests
  - `src-tauri/src/instructions/parse/test_support/`
  - `tests/fixtures/source/`

### Integration Points

**Internal Communication:**
Frontend controls communicate with backend state through `actions.ts` and the Tauri command layer. Backend state updates propagate through existing event channels, while transport state continues to use the established polling path. Selection, hit-testing, and popover positioning must remain coordinated through typed shared contracts.

**External Integrations:**
This feature does not require new third-party service integrations. The relevant external boundaries remain local file workflows, native rendering, and audio-device integration.

**Data Flow:**
User interaction begins in React controls or the renderer overlay, flows through typed frontend service calls into backend commands, updates authoritative Rust state, triggers recalculation only where needed, regenerates draw instructions and hit targets, and returns visible state through the existing application update channels.

### File Organization Patterns

**Configuration Files:**
Root-level configuration remains where it already exists: package/tooling config in the repository root, native configuration under `src-tauri/`, and workflow-specific documentation under `docs/` and `_bmad-output/`.

**Source Organization:**
Source remains split by responsibility:

- shared frontend logic in `src/services/`, `src/ui/`, and `src/components/`
- window-specific behavior in `src/windows/`
- authoritative backend domains in `src-tauri/src/score/`, `instructions/`, `renderer/`, `playback/`, `ui/`, `files.rs`, and `migrations/`

**Test Organization:**
Tests stay close to the behavior they verify:

- TypeScript tests colocated with source files
- Rust parser and renderer tests near the owning modules
- shared parser harness utilities in `src-tauri/src/instructions/parse/test_support/`
- manual visual fixtures under `tests/fixtures/source/`

**Asset Organization:**
Frontend assets remain under `src/assets/`, instrument content remains under `instruments/`, and test fixtures remain in the existing fixture directories.

### Development Workflow Integration

**Development Server Structure:**
The structure supports parallel frontend and native development through the existing Vite plus Tauri workflow. Frontend changes remain isolated to the React shell and supporting services, while backend changes remain in the Rust domains that own state, rendering, and persistence.

**Build Process Structure:**
The build process continues to compile the frontend bundle through Vite and the native application through Tauri and Cargo. The structure keeps frontend assets, backend domains, and fixture tooling separate enough for targeted validation.

**Deployment Structure:**
The existing structure supports the current macOS-first native packaging model. No reorganization is needed for deployment; the main concern is preserving clean module boundaries so the feature can ship without destabilizing the current desktop application packaging and migration flow.

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:**
The architecture is internally coherent. The core decisions reinforce one another: Rust remains the single source of truth, the score surface remains natively rendered, React remains the orchestration layer, and persistence remains governed through the existing `.soloc` plus migration model. These choices are well aligned with the brownfield nature of the product and with the multi-flow feature boundary defined in the PRD.

**Pattern Consistency:**
The implementation patterns support the architectural decisions effectively. Naming rules, cross-boundary serialization expectations, ownership boundaries, and test-placement rules all match the chosen stack and repository conventions. The document is especially strong where AI agents are most likely to drift: frontend/backend ownership, selection contracts, persistence changes, and module placement.

**Structure Alignment:**
The project structure section aligns with the architecture. It extends the existing repository shape rather than inventing a parallel feature hierarchy, and it maps the multi-flow work into the backend rendering, parser, renderer, UI-state, and frontend orchestration areas that actually own the relevant behavior.

### Requirements Coverage Validation ✅

**Epic/Feature Coverage:**
The architecture fully supports the current feature scope as expressed through the PRD requirement clusters. Score presentation, interaction continuity, notation preservation, update isolation, desktop document behavior, and playback boundaries all have clear architectural homes and boundaries.

**Functional Requirements Coverage:**
All major functional requirement categories are architecturally supported:

- multi-flow score presentation maps to parse, draw, renderer, and frontend renderer orchestration
- interaction across flows maps to hit-testing, selection, popovers, and UI state
- notation integrity maps to parser-local protections and regression coverage
- navigation and reading map to renderer/view orchestration and active-flow handling
- update isolation maps to invalidation and caching boundaries
- desktop workflow maps to files, `.soloc`, and migration modules
- playback preservation maps to keeping playback architecture separate from presentation changes

**Non-Functional Requirements Coverage:**
The architecture addresses the important non-functional requirements:

- performance through isolated recalculation, caching boundaries, and instrumentation expectations
- correctness through notation-preservation rules, explicit interaction contracts, and regression testing
- offline-first and macOS-native operation through the unchanged desktop-first foundation
- persistence safety through migration-aware change control
- maintainability through explicit module boundaries and consistency rules

### Implementation Readiness Validation ✅

**Decision Completeness:**
The critical architectural decisions are sufficiently documented for implementation. The document makes clear what must remain stable, what can evolve, and what is intentionally deferred.

**Structure Completeness:**
The project structure guidance is concrete enough for implementation. It identifies the relevant repository areas, maps requirements to them, and defines where cross-cutting concerns belong.

**Pattern Completeness:**
The implementation patterns cover the major conflict areas that would otherwise produce inconsistent AI-agent output. This is especially true for naming, ownership, persistence, communication, and testing boundaries.

### Gap Analysis Results

**Critical Gaps:**
No critical architectural gaps were identified.

**Important Gaps:**

- The exact invalidation contract for multi-flow recalculation is not yet specified at a lower-level design granularity
- Render-timing instrumentation expectations are defined directionally rather than at specific hook points
- Some PRD requirements remain partly qualitative, so implementation stories should sharpen acceptance criteria as work is broken down

**Nice-to-Have Gaps:**

- A later implementation spec could define concrete invalidation scenarios and example flow-update cases
- A supplemental testing matrix could make parser, renderer, fixture, and interaction coverage expectations even more explicit

### Validation Issues Addressed

The main architectural risk identified during validation was the possibility of feature work introducing parallel ownership models or duplicated score state in the frontend. This has already been addressed in the architectural decisions and implementation patterns by explicitly preserving Rust ownership, native rendering, and typed interaction contracts.

A second risk was persistence drift. This has also been addressed by making migration review and `.soloc` compatibility part of the architectural rules rather than treating them as optional implementation details.

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

- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** High

**Key Strengths:**

- Strong alignment with the existing brownfield architecture
- Clear ownership boundaries between Rust and React
- Explicit protection of notation behavior and persistence safety
- Good implementation guidance for multi-agent consistency
- Practical mapping from requirements to concrete repository areas

**Areas for Future Enhancement:**

- Lower-level invalidation design for multi-flow recalculation
- More explicit instrumentation points for render-performance verification
- Story-level acceptance criteria refinement for the PRD’s more qualitative requirements

### Implementation Handoff

**AI Agent Guidelines:**

- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently across all touched modules
- Respect Rust/React ownership boundaries and existing repository structure
- Treat persistence, migrations, notation correctness, and regression coverage as first-class architectural concerns

**First Implementation Priority:**
Define the multi-flow composition and invalidation boundary inside the existing Rust render pipeline before extending frontend orchestration.
