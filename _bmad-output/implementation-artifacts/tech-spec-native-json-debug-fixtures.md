---
title: "Native JSON Debug Fixtures"
slug: "native-json-debug-fixtures"
created: "2026-03-06"
status: "ready-for-dev"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  [
    "rust 2021",
    "tauri 2.10",
    "typescript",
    "react",
    "vite",
    "vitest",
    "@testing-library/react",
    "serde_json",
    "flate2",
    "semver",
  ]
files_to_modify:
  - "src/services/actions.ts"
  - "src-tauri/src/debug_fixtures.rs"
  - "src-tauri/src/files.rs"
  - "src-tauri/src/lib.rs"
  - "src/windows/debugger/index.test.tsx"
  - "src/windows/debugger/components/score-fixture-loader.test.tsx"
  - "src/windows/debugger/score-fixtures/index.ts"
  - "src/windows/debugger/score-fixtures/types.ts"
  - "src/windows/debugger/components/score-fixture-loader.tsx"
  - "src/windows/debugger/test-cases/README.md"
  - "src/windows/debugger/test-cases/*.json"
code_patterns:
  - "Single source of truth: native saved-project shape { ui, score, next_id }"
  - "Debugger fixture discovery via frontend glob import"
  - "Tauri file loading and migration pipeline in files.rs"
  - "Keep fixture labels inferred from filename only"
  - "Prefer debugger-specific plain JSON loading without changing normal app save behavior"
  - "Debugger UI currently assumes fixture metadata fields name, description, cases, and payload"
  - "Tauri commands are registered centrally in lib.rs and invoked from services/actions.ts"
test_patterns:
  - "Vitest coverage for debugger fixture discovery and loader UI"
  - "Rust unit tests for plain JSON fixture load path and migration handling"
  - "Behavior-preserving refactor with existing load_project_from_file semantics"
  - "Frontend tests currently mock actions.debug.loadScoreFixture and assert success messaging"
  - "No existing unit tests live in files.rs or debug_fixtures.rs today"
---

# Tech-Spec: Native JSON Debug Fixtures

**Created:** 2026-03-06

## Overview

### Problem Statement

The debugger fixture system currently defines and loads fixtures through a custom declarative score DSL, then translates that DSL into the real app score graph. This introduces a second score authoring model alongside the app’s actual persisted project model, increases fixture-specific backend code, and creates drift risk. The fixture README also documents the old fixture shape, so it no longer matches the desired source-of-truth format. These fixtures are not a product feature for end users; they exist solely as agent-generated regression/test inputs.

### Solution

Replace debugger fixture payload files with plain, uncompressed JSON files that already use the app’s native saved-project structure `{ ui, score, next_id }`. Load these files for debugger use through a debugger-only plain-JSON path that still applies the existing migration pipeline. Treat fixture generation as an agent-only workflow for creating test files quickly; users do not create fixtures through the product UI. Update the fixture README with concise, accurate instructions for generating these native JSON fixture files in that context.

### Scope

**In Scope:**

- Replace current debugger fixture payload files with plain `.json` files containing the native saved-project shape
- Support debugger loading of plain JSON project fixtures
- Reuse the same migration pipeline used for regular project loading
- Reduce or remove fixture-specific score-construction logic from the debugger backend
- Infer debugger fixture labels from file names only
- Make it explicit in docs/spec language that fixture creation is solely for agents generating regression inputs, not for end users
- Update fixture authoring documentation with concise native JSON generation instructions

**Out of Scope:**

- Changing the normal app save format for user `.soloc` files
- Building a user-facing fixture creation workflow in the main app
- Supporting manual end-user fixture authoring
- Adding sidecar manifests for `name`, `description`, or `cases`
- Introducing a build-time conversion pipeline
- Changing the canonical runtime score model
- Adding a second metadata layer for debugger fixtures

## Context for Development

### Codebase Patterns

- Native project persistence already uses `{ ui, score, next_id }` serialized in Rust and saved through `save_project_to_file()` using gzip compression
- Regular project loading in `load_project_from_file()` currently assumes gzip input, then applies migrations before hydrating app state and syncing the sampler
- Debugger fixture discovery happens in the frontend via `import.meta.glob()` over `src/windows/debugger/test-cases/*.json`
- The current debugger fixture contract is FE-defined and metadata-heavy: `id`, `name`, `description`, `payload`, and `cases`
- `ScoreFixtureLoader` selects fixtures by `id`, renders `name`/`description`, and invokes `actions.debug.loadScoreFixture(selectedFixture.payload)`
- `actions.debug.loadScoreFixture()` is a thin Tauri command wrapper; the backend `debug_load_score_fixture()` expands the custom fixture DSL into the real score graph
- Tauri commands are wired centrally in `src-tauri/src/lib.rs`, so any new debugger load command or signature change must be registered there and mirrored in `actions.ts`
- Project rules explicitly state that `.soloc` is the native save format and FE↔BE state changes should go through Tauri commands
- Rust/TS serialized shapes must stay in sync manually

### Files to Reference

| File                                                            | Purpose                                                         |
| --------------------------------------------------------------- | --------------------------------------------------------------- |
| `src/services/actions.ts`                                       | Frontend Tauri command wrapper for debugger fixture loading     |
| `src-tauri/src/files.rs`                                        | Native save/load pipeline and migration entry point             |
| `src-tauri/src/lib.rs`                                          | Tauri command registration and app context definitions          |
| `src-tauri/src/debug_fixtures.rs`                               | Existing fixture-specific DSL and score-construction path       |
| `src/windows/debugger/score-fixtures/index.ts`                  | Fixture discovery via frontend glob imports                     |
| `src/windows/debugger/score-fixtures/types.ts`                  | Current custom fixture type definitions                         |
| `src/windows/debugger/components/score-fixture-loader.tsx`      | Debugger UI for selecting and loading fixtures                  |
| `src/windows/debugger/components/score-fixture-loader.test.tsx` | Loader UI tests tied to current metadata and load action        |
| `src/windows/debugger/index.test.tsx`                           | Debugger integration test for fixture loading button flow       |
| `src/windows/debugger/test-cases/README.md`                     | Fixture authoring guidance to update                            |
| `src/windows/debugger/test-cases/*.json`                        | Current checked-in fixture source files using the custom schema |
| `docs/project-context.md`                                       | Project conventions and persistence rules                       |

### Technical Decisions

- Repo fixture source-of-truth will be plain `.json` files using the native project structure, not the custom fixture DSL
- Fixture labels in the debugger will be inferred from file names only; no extra metadata manifest will be introduced
- Plain JSON fixture support is debugger-only for now; normal app save behavior remains unchanged
- Fixture generation is explicitly an agent-only workflow for fast test creation and maintenance, not part of the core end-user product surface
- Users never create fixtures through the product UI; the debugger only consumes agent-generated artifacts checked into the repo
- Fixture loads must still pass through the existing migration path so checked-in fixtures are treated like first-class project files
- README guidance must explain how to generate fixture files directly from the established score structure, not a reduced custom schema
- The README should open by explicitly stating its audience, for example: "This README is for agents generating debugger fixture files."
- The likely backend seam is to extract shared project-data parsing/hydration from `load_project_from_file()` so debugger JSON fixtures can reuse migration + state application without copying logic
- Frontend fixture discovery will likely need to shift from typed default-imported metadata objects to file-path-based inventory, because filename becomes the only label source
- Existing fixture UI around `description` and `cases` is not compatible with the agreed target state and will need to be simplified or removed
- Existing fixture JSON files in `src/windows/debugger/test-cases` will need to be regenerated or replaced with native project JSON examples during implementation

## Implementation Plan

### Tasks

- [ ] Task 1: Extract shared project parsing and hydration helpers from the native file loader
  - File: `src-tauri/src/files.rs`
  - Action: Split `load_project_from_file()` into reusable internal helpers for reading raw bytes, decoding project JSON, applying migrations, hydrating `{ ui, score, next_id }`, resetting ephemeral UI state, and syncing sampler state.
  - Notes: Keep current `.soloc` behavior unchanged. The extracted seam should support reuse without duplicating migration or hydration logic.

- [ ] Task 2: Add debugger-only support for plain native-project JSON fixture loading
  - File: `src-tauri/src/files.rs`
  - Action: Add a debugger-targeted load path that accepts plain `.json` fixture files, parses UTF-8 JSON directly, and then routes through the same migration and hydration pipeline used by normal project loading.
  - Notes: Normal user-facing file loading should continue to assume `.soloc`; the new plain JSON support is only for debugger fixtures.

- [ ] Task 3: Replace fixture-specific backend score construction with file-path-based fixture loading
  - File: `src-tauri/src/debug_fixtures.rs`
  - Action: Remove or dramatically narrow the custom fixture DSL types and `debug_load_score_fixture()` implementation so debugger fixture loading operates on native project files by path instead of constructing score state from `payload` data.
  - Notes: Prefer a thin command that delegates to the shared loader seam. Any remaining fixture-specific code should be limited to debugger orchestration, not score-model translation.

- [ ] Task 4: Rewire debugger command registration and frontend action signatures to load fixtures by file path
  - Files: `src-tauri/src/lib.rs`, `src/services/actions.ts`
  - Action: Register the new debugger fixture load command shape in Rust and update the frontend action wrapper so the debugger passes a file path string instead of a custom payload object.
  - Notes: Keep command naming and call patterns consistent with other Tauri command wrappers.

- [ ] Task 5: Simplify frontend fixture discovery to use file-path inventory and filename-derived labels
  - Files: `src/windows/debugger/score-fixtures/index.ts`, `src/windows/debugger/score-fixtures/types.ts`
  - Action: Replace metadata-object imports with file-path-based discovery, derive visible labels from file names, and remove types that only exist for the custom fixture DSL.
  - Notes: The new fixture inventory should not require `id`, `name`, `description`, `payload`, or `cases` fields in each fixture file.

- [ ] Task 6: Update the debugger fixture loader UI to match the new minimal fixture model
  - File: `src/windows/debugger/components/score-fixture-loader.tsx`
  - Action: Update the loader component to select fixtures by discovered file path, display filename-derived labels, and remove UI elements that depend on deleted metadata such as descriptions and included cases.
  - Notes: Preserve the basic select + load interaction and status messaging.

- [ ] Task 7: Replace or regenerate checked-in fixture files as native project JSON
  - File: `src/windows/debugger/test-cases/*.json`
  - Action: Convert existing debugger fixtures from the custom schema to the native `{ ui, score, next_id }` project structure, keeping filenames stable where practical.
  - Notes: Fixtures should remain intentional, readable, and migration-compatible. Remove obsolete metadata fields rather than embedding them into the saved project model.

- [ ] Task 8: Rewrite fixture authoring instructions for native JSON generation
  - File: `src/windows/debugger/test-cases/README.md`
  - Action: Replace the old fixture-schema documentation with concise instructions aimed at agents, clearly explaining that fixture files are plain native project JSON, how agents generate them quickly from real score state, how filenames drive debugger labels, and what validation expectations remain.
  - Notes: Keep the README short, practical, aligned with the actual loader behavior, and explicit that it is an agent-facing generation guide rather than a user-facing feature guide or manual authoring workflow. The opening should explicitly name the audience, e.g. "This README is for agents generating debugger fixture files."

- [ ] Task 9: Update automated tests for the new fixture-loading model
  - Files: `src/windows/debugger/components/score-fixture-loader.test.tsx`, `src/windows/debugger/index.test.tsx`, `src-tauri/src/files.rs`, `src-tauri/src/debug_fixtures.rs`
  - Action: Adjust frontend tests to assert path-based loading and simplified UI, and add Rust tests covering plain JSON fixture parsing, migration handling, and behavior preservation for existing `.soloc` loading.
  - Notes: Add focused unit tests near the changed Rust code even though these files currently have no tests.

### Acceptance Criteria

- [ ] AC 1: Given a checked-in debugger fixture JSON file containing native `{ ui, score, next_id }` project data, when it is selected in the debugger, then the app loads it successfully without using the custom fixture DSL.
- [ ] AC 2: Given a debugger fixture JSON file saved from an older schema version, when the debugger loads it, then the existing migration pipeline upgrades it before state hydration.
- [ ] AC 3: Given a normal user `.soloc` file, when it is opened through the regular file-open flow, then existing gzip-based load behavior remains unchanged.
- [ ] AC 4: Given a fixture file in `src/windows/debugger/test-cases`, when it appears in the debugger UI, then its label is derived from the filename and no `name`, `description`, or `cases` metadata is required inside the file.
- [ ] AC 5: Given the updated debugger fixture loader UI, when a user selects and loads a fixture, then the simplified picker still provides clear success or failure status messaging.
- [ ] AC 6: Given the updated fixture README, when an agent follows it, then it has clear instructions to generate a valid native JSON fixture file without referencing the deleted custom schema.
- [ ] AC 6a: Given the finalized docs/spec, when a reader reviews the feature intent, then it is clear that fixture generation is solely an agent testing workflow rather than core end-user app functionality.
- [ ] AC 7: Given the backend refactor is complete, when reviewing the implementation, then fixture-specific score-construction logic is removed or reduced to thin debugger orchestration only.
- [ ] AC 8: Given the changed loader paths, when automated tests run, then frontend tests cover the new path-based fixture selection and Rust tests cover plain JSON fixture parsing plus migration handling.

## Additional Context

### Dependencies

- Existing Rust migration functions in `MIGRATIONS`
- Existing Tauri state hydration and sampler sync flow
- Existing debugger fixture discovery and selection UI
- Current native save structure emitted by `save_project_to_file()`
- `import.meta.glob()` support in the debugger frontend bundle
- Existing fixture filenames in `src/windows/debugger/test-cases` if stable naming is to be preserved

### Testing Strategy

- Add Rust unit tests in or near `src-tauri/src/files.rs` for: plain JSON fixture decoding, migration application, hydration of `{ ui, score, next_id }`, and preservation of existing gzip `.soloc` loading behavior
- Add Rust coverage around the debugger fixture command path in `src-tauri/src/debug_fixtures.rs`, ensuring it delegates to shared loading logic rather than rebuilding score state
- Update `src/windows/debugger/components/score-fixture-loader.test.tsx` to assert filename-derived options, path-based command invocation, and retained status messaging after load success/failure
- Update `src/windows/debugger/index.test.tsx` to keep end-to-end debugger fixture loading coverage aligned with the new action signature
- Manually verify that representative checked-in fixture JSON files load in the running debugger and replace the current unsaved score state as expected
- Manually verify that normal File → Open for `.soloc` still works exactly as before

### Notes

- Primary architecture goal: eliminate the duplicate score authoring model used only for debugger fixtures
- Compression is treated as an on-disk transport concern, not as the definition of the fixture source format
- Documentation must be concise and practical, not a restatement of internal implementation details
- The current `ScoreFixtureLoader` UI will necessarily lose some richness because filename-only labels replace fixture-specific descriptions and indexed cases
- If plain JSON fixture support later proves broadly useful, the shared parsing seam created here can be promoted beyond the debugger without redoing the migration/hydration work
- Fixture generation guidance should strongly encourage deriving files from real saved project state, then normalizing them for repo readability rather than hand-authoring large nested score objects from scratch
- The intended author is an agent producing regression fixtures quickly on demand; the app only needs enough debugger support to load those artifacts reliably
- Fixture documentation should explicitly target agents and give clear generation instructions, while only secondarily keeping checked-in JSON readable for humans reviewing repo changes
- A good README outcome is one where an agent can read the first paragraph and immediately understand: these files are agent-generated, they use native `{ ui, score, next_id }` JSON, and the debugger only loads them
