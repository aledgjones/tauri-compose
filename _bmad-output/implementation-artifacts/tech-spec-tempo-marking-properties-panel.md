---
title: "Tempo Marking Properties Panel"
slug: "tempo-marking-properties-panel"
created: "2026-03-08"
status: "ready-for-dev"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  - "TypeScript 5 + React 19 frontend for write-screen UI, hooks, and typed state access"
  - "Rust 2021 + Tauri 2 backend for sequencer selection state, hit-testing, and native score rendering"
  - "Skia/Metal renderer pipeline with space-based hit targets and draw instructions"
  - "Vitest + Testing Library for frontend component/hook/service tests"
  - "Inline Rust #[cfg(test)] unit tests beside renderer, draw, and score-entry modules"
files_to_modify:
  - "package.json"
  - "src/services/types.ts"
  - "src/services/actions.ts"
  - "src/services/actions.test.ts"
  - "src/services/hit-test.ts"
  - "src/services/hit-test.test.ts"
  - "src/services/utils.ts"
  - "src/services/utils.test.ts"
  - "src/services/popovers.ts"
  - "src/services/popovers.test.ts"
  - "src/windows/main/components/renderer.tsx"
  - "src/windows/main/screens/write/components/bottom-panel.tsx"
  - "src/windows/main/screens/write/components/bottom-panel.test.tsx"
  - "src/windows/main/hooks/use-keyboard-shortcuts.ts"
  - "src/windows/main/hooks/use-selected-tone-info.ts"
  - "src-tauri/src/ui/sequencer.rs"
  - "src-tauri/src/ui/mod.rs"
  - "src-tauri/src/instructions/draw/draw_tempo.rs"
  - "src-tauri/src/instructions/mod.rs"
  - "src-tauri/src/renderer/mod.rs"
  - "src-tauri/src/renderer/hit_test.rs"
  - "src-tauri/src/score/entries/tempo.rs"
  - "src-tauri/src/score/entries/mod.rs"
  - "src-tauri/src/playback/transport.rs"
  - "src-tauri/src/migrations/mod.rs"
  - "src-tauri/src/migrations/1.24.0.rs"
  - "src-tauri/src/lib.rs"
  - "src-tauri/Cargo.toml"
  - "src-tauri/tauri.conf.json"
  - "src/windows/main/screens/write/components/properties-panel.tsx"
  - "src/windows/main/screens/write/components/properties-panel.test.tsx"
  - "src/windows/main/screens/write/components/tempo-properties-section.tsx"
  - "src/windows/main/screens/write/components/tempo-properties-section.test.tsx"
  - "src/windows/main/hooks/use-selected-tempo.ts"
  - "src/windows/main/hooks/use-selected-tempo.test.ts"
  - "docs/architecture.md"
  - "docs/component-inventory.md"
  - "docs/data-models.md"
code_patterns:
  - "Sequencer selection is a shared FE/BE tagged union mirrored between src/services/types.ts and src-tauri/src/ui/sequencer.rs"
  - "Native score hit targets are emitted during draw passes and returned through renderer_hit_test, then the React renderer dispatches selection actions from the hit result"
  - "Selected score items are highlighted in Rust draw helpers by comparing the current optional sequencer selection against the item being drawn"
  - "Write-screen bottom-panel UI uses tab-gated rendering with lightweight toolbars/content blocks and Vitest snapshot-style interaction coverage"
  - "Tempo entry mutation currently uses create-or-replace-at-tick semantics, so immediate property editing either needs a dedicated update command or a safe recreate path that preserves tick identity semantics"
  - "Reusable UI primitives already exist for compact controls: IconButton, Switch, Checkbox, TextInput, and NumberInput"
  - "Hooks such as useSelectedToneInfo() isolate selection-to-domain lookups and are a good pattern for tempo-specific or generalized properties data hooks"
test_patterns:
  - "Frontend action wrappers are tested by asserting exact invoke payloads in src/services/actions.test.ts"
  - "Renderer bridge utilities such as hitTest() use direct Vitest service tests in src/services/hit-test.test.ts"
  - "Write-screen panel components use renderWithProviders() + createAppState() in focused component tests"
  - "Selection-derived hooks use renderHook-based tests beside the hook implementation"
  - "Rust draw helpers keep inline tests validating draw instructions and hit targets next to the implementation"
  - "Rust selection and renderer command changes should add focused inline tests where the matching logic lives rather than relying on broad integration coverage"
---

# Tech-Spec: Tempo Marking Properties Panel

**Created:** 2026-03-08

## Overview

### Problem Statement

The write screen already exposes a placeholder `Properties` bottom-panel tab, but tempo markings are not selectable through the existing sequencer selection mechanism and therefore cannot participate in score-selection workflows. Users need to click a tempo marking directly in the score, see clear selection highlighting, and edit its structured configuration from the write-screen properties panel without leaving the current score workflow.

### Solution

Extend the existing sequencer selection mechanism to include tempo entries, backed by renderer hitboxes and selected-state highlighting for tempo markings. Fill the write-screen `Properties` tab with the first real properties experience: a compact, horizontally scrollable, section-based panel that renders dynamic content by selected item type. In v1, only tempo selections receive editable controls. Text edits commit on blur or Enter to avoid churn while typing, while visibility toggles, beat type, BPM, and parenthesized state apply immediately; empty states cover no selection and unsupported selection types.

### Scope

**In Scope:**

- Write-screen only implementation of the `Properties` bottom panel
- Tempo hitboxes and hit-testing support in the native score renderer
- Tempo integration into the existing sequencer selection model/state
- Tempo selection highlight styling in score rendering
- Properties-panel foundation that swaps sections dynamically by selection type
- Compact, data-dense panel design suitable for future multi-section property editing
- Horizontal scrolling behavior for the properties content area
- Tempo-specific properties section with immediate-apply controls for:
  - text label
  - text visibility toggle
  - BPM numeric value
  - beat type segmented icon buttons
  - metronome mark visibility toggle
  - parenthesized toggle
- Empty state for no selection and for selected types with no implemented properties UI yet

**Out of Scope:**

- Auto-switching to the `Properties` tab when a tempo marking is selected
- Note, rest, or other non-tempo properties editing
- Engrave-screen implementation work
- Apply/save staging workflows or deferred commit UX
- Broader redesign of score-wide selection behavior beyond the minimum needed to support tempo selection

## Context for Development

### Codebase Patterns

- The write screen already includes a `Properties` bottom-panel tab placeholder in [src/windows/main/screens/write/components/bottom-panel.tsx](src/windows/main/screens/write/components/bottom-panel.tsx), and its current tests in [src/windows/main/screens/write/components/bottom-panel.test.tsx](src/windows/main/screens/write/components/bottom-panel.test.tsx) explicitly expect no toolbar/content for that tab today.
- Sequencer selection is a mirrored tagged union across [src/services/types.ts](src/services/types.ts) and [src-tauri/src/ui/sequencer.rs](src-tauri/src/ui/sequencer.rs); adding tempo selection cleanly means extending both surfaces together and registering any new Tauri selection command in [src-tauri/src/lib.rs](src-tauri/src/lib.rs).
- The React score surface in [src/windows/main/components/renderer.tsx](src/windows/main/components/renderer.tsx) performs a renderer hit test first, then dispatches explicit selection actions based on the returned hit-target type. Right now that flow is hard-coded for `Tone` and `Rest`, so tempo support needs a third branch there.
- Native hit targets are stored as `HitTarget { entry, x, y, width, height, popover_x, popover_y }` in [src-tauri/src/renderer/mod.rs](src-tauri/src/renderer/mod.rs) and resolved through [src-tauri/src/renderer/hit_test.rs](src-tauri/src/renderer/hit_test.rs). Tempo selection must therefore be emitted from the Rust draw pipeline, not approximated in React.
- Existing draw helpers show the selection/highlight pattern clearly: [src-tauri/src/instructions/draw/draw_notehead.rs](src-tauri/src/instructions/draw/draw_notehead.rs) and [src-tauri/src/instructions/draw/draw_rest.rs](src-tauri/src/instructions/draw/draw_rest.rs) both compare the current `Selection` and switch draw color to the project’s sky accent while also pushing a matching hit target.
- Tempo data already carries nearly all of the editable fields the panel needs in [src-tauri/src/score/entries/tempo.rs](src-tauri/src/score/entries/tempo.rs) and [src/services/types.ts](src/services/types.ts): `text`, `beat_type`, `dots`, `bpm`, `hidden`, `show_text`, `show_metronome`, and `parenthesized`. This slice should simplify that model by treating tempo text as a required string field and using `show_text` for display toggling rather than splitting “has text” across both `Option<String>` and `show_text`, but that is a persisted-data change and must be implemented with migration/backward-compat coverage.
- Tempo mutation currently exposes only create/clear operations in [src/services/actions.ts](src/services/actions.ts) and [src-tauri/src/score/entries/tempo.rs](src-tauri/src/score/entries/tempo.rs), using replace-at-tick semantics. Immediate properties editing will need an explicit update strategy rather than ad hoc local state.
- Existing selection-derived hooks such as [src/windows/main/hooks/use-selected-tone-info.ts](src/windows/main/hooks/use-selected-tone-info.ts) and tone-only keyboard behavior in [src/windows/main/hooks/use-keyboard-shortcuts.ts](src/windows/main/hooks/use-keyboard-shortcuts.ts) show that selection-specific logic is intentionally isolated in hooks/helpers. Tempo properties should follow that pattern instead of bloating the bottom-panel component.
- Compact control primitives already exist in [src/ui/input/text-input.tsx](src/ui/input/text-input.tsx), [src/ui/input/number-input.tsx](src/ui/input/number-input.tsx), [src/ui/switch.tsx](src/ui/switch.tsx), [src/ui/checkbox.tsx](src/ui/checkbox.tsx), and [src/ui/buttons/icon-button.tsx](src/ui/buttons/icon-button.tsx), which supports the requested dense properties layout without inventing a new control system.
- Persisted score schema changes already flow through the migration layer rooted at [src-tauri/src/migrations/mod.rs](src-tauri/src/migrations/mod.rs), so the nullable-to-required tempo-text change must include an explicit migration or backward-compatible load path instead of claiming “no migration required”.
- Project versioning is semver-synced across [package.json](package.json), [src-tauri/Cargo.toml](src-tauri/Cargo.toml), and [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json). Because this slice requires migration coverage for older `.soloc` files, implementation should start with a minor bump from `1.23.3` to `1.24.0` before the schema change lands.
- Project versioning is semver-synced across [package.json](package.json), [src-tauri/Cargo.toml](src-tauri/Cargo.toml), and [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json). Because this slice requires migration coverage for older `.soloc` files, implementation should start with the project’s scripted minor release flow via [package.json](package.json), using `npm run release:minor` before the schema change lands. Do not manually edit version numbers.
- Migration files should be version-targeted modules named for the version they migrate to, for example [src-tauri/src/migrations/1.24.0.rs](src-tauri/src/migrations/1.24.0.rs), then imported and registered from [src-tauri/src/migrations/mod.rs](src-tauri/src/migrations/mod.rs).

### Files to Reference

| File                                                                                                                                                        | Purpose                                                                                                             |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| [src/windows/main/screens/write/components/bottom-panel.tsx](src/windows/main/screens/write/components/bottom-panel.tsx)                                    | Existing write-screen bottom-panel shell where the `Properties` tab content must finally render                     |
| [src/windows/main/screens/write/components/bottom-panel.test.tsx](src/windows/main/screens/write/components/bottom-panel.test.tsx)                          | Current expectations for write bottom-panel tabs and a natural anchor for new properties-panel rendering tests      |
| [src/windows/main/components/renderer.tsx](src/windows/main/components/renderer.tsx)                                                                        | React overlay that turns renderer hit-test results into sequencer selection actions                                 |
| [src/services/types.ts](src/services/types.ts)                                                                                                              | Frontend `Tempo`, `Selection`, and `BottomPanelTab` typing surface that must stay aligned with Rust serialization   |
| [src/services/actions.ts](src/services/actions.ts)                                                                                                          | FE mutation wrappers, including existing tempo create/clear commands and sequencer selection actions                |
| [src/services/actions.test.ts](src/services/actions.test.ts)                                                                                                | Contract tests for invoke payloads; likely home for any new tempo update/select action assertions                   |
| [src/services/hit-test.ts](src/services/hit-test.ts)                                                                                                        | FE hit-test bridge used by the renderer click path                                                                  |
| [src/services/hit-test.test.ts](src/services/hit-test.test.ts)                                                                                              | Existing service-level hit-test verification pattern                                                                |
| [src/services/utils.ts](src/services/utils.ts)                                                                                                              | Tempo parser output shape that must change if `Tempo.text` becomes a required string                                |
| [src/services/utils.test.ts](src/services/utils.test.ts)                                                                                                    | Parser tests that currently encode metronome-only tempo as nullable text                                            |
| [src/services/popovers.ts](src/services/popovers.ts)                                                                                                        | Tempo popover dispatch path that forwards parsed tempo payloads into create/update actions                          |
| [src/services/popovers.test.ts](src/services/popovers.test.ts)                                                                                              | Existing tempo-popover contract coverage that must reflect the new text representation                              |
| [src/windows/main/hooks/use-selected-tone-info.ts](src/windows/main/hooks/use-selected-tone-info.ts)                                                        | Existing pattern for mapping sequencer selection to richer UI-ready data                                            |
| [src/windows/main/hooks/use-keyboard-shortcuts.ts](src/windows/main/hooks/use-keyboard-shortcuts.ts)                                                        | Tone-only keyboard behavior that must remain safe when non-tone selection types like tempo are active               |
| [src-tauri/src/ui/sequencer.rs](src-tauri/src/ui/sequencer.rs)                                                                                              | Backend definition of sequencer `Selection` and selection-set commands                                              |
| [src-tauri/src/ui/mod.rs](src-tauri/src/ui/mod.rs)                                                                                                          | Selection-coordinate lookup that depends on `Selection` equality matching hit targets                               |
| [src-tauri/src/renderer/mod.rs](src-tauri/src/renderer/mod.rs)                                                                                              | `HitTarget` structure used by native selection hit-testing                                                          |
| [src-tauri/src/renderer/hit_test.rs](src-tauri/src/renderer/hit_test.rs)                                                                                    | Native renderer hit-test command returning a sequencer selection                                                    |
| [src-tauri/src/instructions/draw/draw_notehead.rs](src-tauri/src/instructions/draw/draw_notehead.rs)                                                        | Best current example of selected-state coloring plus hit-target emission                                            |
| [src-tauri/src/instructions/draw/draw_rest.rs](src-tauri/src/instructions/draw/draw_rest.rs)                                                                | Best current example of non-tone sequencer selection/hitbox behavior                                                |
| [src-tauri/src/instructions/draw/draw_tempo.rs](src-tauri/src/instructions/draw/draw_tempo.rs)                                                              | Tempo draw helper that currently renders black-only tempo content with no selection hitbox support                  |
| [src-tauri/src/instructions/mod.rs](src-tauri/src/instructions/mod.rs)                                                                                      | Draw-pass orchestration where current selection is threaded into note/rest rendering and must be extended for tempo |
| [src-tauri/src/score/entries/tempo.rs](src-tauri/src/score/entries/tempo.rs)                                                                                | Tempo field model, metrics helpers, and current create/clear command surface                                        |
| [src-tauri/src/score/entries/mod.rs](src-tauri/src/score/entries/mod.rs)                                                                                    | Shared entry constructors/tests likely touched by any `Tempo` constructor/signature change                          |
| [src-tauri/src/playback/transport.rs](src-tauri/src/playback/transport.rs)                                                                                  | Transport tests/helpers construct `Tempo` values and must stay aligned with schema changes                          |
| [src-tauri/src/migrations/mod.rs](src-tauri/src/migrations/mod.rs)                                                                                          | Migration registry for persisted score-data changes                                                                 |
| [src-tauri/src/migrations/1.24.0.rs](src-tauri/src/migrations/1.24.0.rs)                                                                                    | Concrete migration module for the tempo schema change after the minor version bump                                  |
| [package.json](package.json)                                                                                                                                | Frontend app version that must be minor-bumped before schema-changing work begins                                   |
| [src-tauri/Cargo.toml](src-tauri/Cargo.toml)                                                                                                                | Rust package version that must stay in sync with the app version                                                    |
| [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)                                                                                                      | Tauri app version that must stay in sync with the app version                                                       |
| [\_bmad-output/implementation-artifacts/tech-spec-immediate-tempo-markings.md](_bmad-output/implementation-artifacts/tech-spec-immediate-tempo-markings.md) | Prior completed tempo spec documenting current storage and rendering behavior                                       |

### Technical Decisions

- This slice is write-screen only, but the properties-panel foundation should be reusable by future screens such as engrave.
- Selecting a tempo marking should only update selection/highlight state; it must not automatically switch the active bottom-panel tab.
- Tempo should integrate into the existing sequencer selection mechanism instead of creating a parallel directive-selection system.
- Tempo property edits should apply immediately on control change with no apply/save button.
- The properties panel should render a compact, data-dense layout with grouped sections and horizontal scrolling support.
- Dynamic section rendering by selection type is part of the foundation, even though only tempo sections are implemented in v1.
- When nothing is selected, or when a selected type has no implemented properties UI, the panel should show an explicit empty state.
- Tempo-section controls in v1 should cover text, text visibility, BPM, beat type, metronome visibility, and parenthesized display.
- Because the existing tempo command surface is create/clear-only, Step 3 should choose one explicit editing path and document it precisely. My current recommendation is a dedicated backend update command that mutates a tempo entry by `id`/selection context rather than repeatedly recreating the entry from the panel.
- The exact tempo selection shape should be `type: "Tempo"`, `flow_id: UniqueId`, `tempo_id: UniqueId`, and `tick: Tick`. This preserves the existing convention of carrying relevant context while ensuring stable identity via `tempo_id`.
- Tempo text storage should be simplified from `Option<String>` to a required `String` value. Metronome-only tempos can store an empty string while `show_text = false`; display behavior should be controlled by `show_text`, not duplicated between `Option<String>` and `show_text`.
- `show_text` and `show_metronome` should both be surfaced as explicit switches in the tempo properties section so users can control visible tempo parts independently.
- Beat-type controls should reuse the project’s existing interactive icon-button selection pattern so the panel stays visually consistent with the left-panel’s compact notation controls.
- Tone-only hooks and keyboard shortcuts should degrade safely to `null`/no-op when a tempo selection is active; the implementation should avoid broad selection-side regressions by preserving existing tone behavior and only branching where tempo support is required.
- Text-field edits should commit on blur or Enter rather than on every keystroke. All other tempo properties in this slice should apply immediately.
- Because `Tempo.text` changes persisted schema in this plan, the implementation must include a migration/backward-compatible load path and the spec/docs must explicitly say migration is required.
- Implementation should begin with a minor version bump from `1.23.3` to `1.24.0`, synced across [package.json](package.json), [src-tauri/Cargo.toml](src-tauri/Cargo.toml), and [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json), before the schema change lands.
- Implementation should begin by running `npm run release:minor`, which performs the required semver-synced minor bump and generates release notes before the schema change lands.
- The migration for this slice should live in a dedicated version-targeted module at [src-tauri/src/migrations/1.24.0.rs](src-tauri/src/migrations/1.24.0.rs) and be registered from [src-tauri/src/migrations/mod.rs](src-tauri/src/migrations/mod.rs).

## Implementation Plan

### Tasks

- [ ] Task 0: Bump the app version before schema-changing work begins
  - File: [package.json](package.json)
  - Action: Run `npm run release:minor` before schema-changing work begins.
  - Notes: This scripted flow must be used instead of manual version edits because it keeps version-synced files aligned and generates release notes. The resulting target version for this slice is expected to be `1.24.0`.

- [ ] Task 1: Extend sequencer selection to support tempo entries end-to-end
  - File: [src/services/types.ts](src/services/types.ts)
  - Action: Add a tempo-selection branch to the frontend `Selection` union with the exact shape `type: "Tempo"`, `flow_id`, `tempo_id`, and `tick`.
  - Notes: Keep the shape aligned exactly with Rust serialization and avoid breaking existing `Tone`/`Rest` consumers.
  - File: [src-tauri/src/ui/sequencer.rs](src-tauri/src/ui/sequencer.rs)
  - Action: Add the matching Rust `Selection::Tempo { flow_id, tempo_id, tick }` variant plus a new `ui_sequencer_selection_set_tempo` command that writes tempo selection into `state.ui.sequencer.selection` and emits a render refresh.
  - Notes: Preserve the current tone/rest selection commands and keep the selection union mirrored with the TypeScript type.
  - File: [src-tauri/src/lib.rs](src-tauri/src/lib.rs)
  - Action: Register/import the new tempo selection command.
  - Notes: Follow existing snake_case Tauri command conventions.

- [ ] Task 2: Add tempo hit targets and selected highlighting to native score rendering
  - File: [src-tauri/src/instructions/draw/draw_tempo.rs](src-tauri/src/instructions/draw/draw_tempo.rs)
  - Action: Thread the current optional sequencer selection into `draw_tempo()`, render selected tempo markings in the same sky accent used by other selections, and push a `HitTarget` covering the rendered tempo extent.
  - Notes: Reuse tempo metrics so hitboxes match visible content, and place popover coordinates appropriately above the top stave like other score directives.
  - File: [src-tauri/src/instructions/mod.rs](src-tauri/src/instructions/mod.rs)
  - Action: Pass selection state into the tempo draw pass and ensure tempo hit-target emission happens during normal score drawing.
  - Notes: Tempo markings should remain rendered once above the top stave; selection support must not duplicate draw output across staves.
  - File: [src-tauri/src/renderer/mod.rs](src-tauri/src/renderer/mod.rs)
  - Action: Reuse the existing `HitTarget` structure for tempo entries without widening it unless the investigation proves necessary.
  - Notes: The current structure already supports selection identity and popover coordinates.
  - File: [src-tauri/src/renderer/hit_test.rs](src-tauri/src/renderer/hit_test.rs)
  - Action: Confirm hit testing works unchanged with the new `Selection::Tempo` variant and add/adjust tests if variant matching needs coverage.
  - Notes: This should remain a generic rectangle hit test; the main change is the new returned selection shape.

- [ ] Task 3: Wire tempo hit-test results into the existing React renderer selection flow
  - File: [src/windows/main/components/renderer.tsx](src/windows/main/components/renderer.tsx)
  - Action: Extend the click-hit-test branch to dispatch the new tempo selection action when `renderer_hit_test` returns a tempo target.
  - Notes: Do not auto-switch the write bottom panel to `Properties`; only update selection/highlight state. Preserve the current clear-selection-on-empty-click behavior.
  - File: [src/services/actions.ts](src/services/actions.ts)
  - Action: Add `actions.ui.sequencer.selectTempo()` wrapping the new Tauri command.
  - Notes: Keep snake_case argument names and mirror the Rust command payload exactly.
  - File: [src/services/actions.test.ts](src/services/actions.test.ts)
  - Action: Add invoke-payload coverage for the new tempo selection action.
  - Notes: Match the existing tone/rest action test style.
  - File: [src/services/hit-test.test.ts](src/services/hit-test.test.ts)
  - Action: Broaden hit-test service expectations if needed so tempo-shaped selection payloads are covered.
  - Notes: The command surface is generic, so this may be a small assertion extension rather than a full new test block.

- [ ] Task 4: Add an explicit backend update path for immediate tempo property editing
  - File: [src-tauri/src/score/entries/tempo.rs](src-tauri/src/score/entries/tempo.rs)
  - Action: Simplify the tempo field model by changing `text` from `Option<String>` to a required `String`, add backward-compatible deserialization and/or a migration for older saved data, and add one or more Tauri update commands so the panel can mutate an existing tempo selection without recreate-at-tick behavior.
  - Notes: Preferred shape is a single `score_entries_tempo_update` command keyed by `flow_id` plus `tempo_id`, validating the entry exists on the flow master track and then mutating only the supplied editable fields. Metronome-only tempos should use an empty string with `show_text = false`.
  - File: [src-tauri/src/lib.rs](src-tauri/src/lib.rs)
  - Action: Register/import the tempo update command alongside the existing tempo create/clear commands.
  - Notes: The spec was previously missing this command-surface update.
  - File: [src-tauri/src/migrations/mod.rs](src-tauri/src/migrations/mod.rs)
  - Action: Import and register the dedicated migration module for this slice.
  - Notes: Migration files should be version-targeted modules named for the version they migrate to.
  - File: [src-tauri/src/migrations/1.24.0.rs](src-tauri/src/migrations/1.24.0.rs)
  - Action: Add the migration needed to normalize older tempo payloads that still store `text: null` into the new required-string representation.
  - Notes: This is mandatory if persisted project data would otherwise fail to load.
  - File: [src-tauri/src/score/entries/mod.rs](src-tauri/src/score/entries/mod.rs)
  - Action: Update any shared `Tempo::new(...)` call sites or entry tests impacted by the constructor/signature change.
  - Notes: Keep `Entry` behavior in sync with the new tempo schema.
  - File: [src-tauri/src/playback/transport.rs](src-tauri/src/playback/transport.rs)
  - Action: Update helper constructors/tests that instantiate `Tempo` so they use the required-string representation.
  - Notes: Playback tests already build tempos directly and will fail if omitted.
  - File: [src/services/types.ts](src/services/types.ts)
  - Action: Change frontend `Tempo.text` from `string | null` to `string` and align any tempo-selection typing changes.
  - Notes: Keep FE serialization aligned with Rust.
  - File: [src/services/actions.ts](src/services/actions.ts)
  - Action: Add a matching frontend update wrapper under `actions.score.tempo` and update the existing create wrapper to pass required-string tempo text.
  - Notes: Keep the create/clear commands intact for authoring and deletion workflows.
  - File: [src/services/actions.test.ts](src/services/actions.test.ts)
  - Action: Add invoke-payload tests for the tempo update command surface.
  - Notes: Lock down the immediate-edit contract explicitly.
  - File: [src/services/utils.ts](src/services/utils.ts)
  - Action: Update tempo parsing output so metronome-only tempos produce `text: ""` with `show_text = false` instead of `text: null`.
  - Notes: This is the canonical parser seam for the new representation.
  - File: [src/services/utils.test.ts](src/services/utils.test.ts)
  - Action: Update parser tests to match the required-string representation and add explicit coverage for metronome-only empty-string output.
  - Notes: These tests currently encode the nullable text shape.
  - File: [src/services/popovers.ts](src/services/popovers.ts)
  - Action: Keep tempo create/update dispatch aligned with the new parser output and required-string backend contract.
  - Notes: This is the authoring bridge from UI text entry into score mutations.
  - File: [src/services/popovers.test.ts](src/services/popovers.test.ts)
  - Action: Update popover tests so expected tempo payloads no longer use `null` text.
  - Notes: This closes the contract gap across parser, popover, action, and Rust layers.

- [ ] Task 5: Build a reusable write-screen properties-panel foundation with dynamic section rendering
  - File: [src/windows/main/screens/write/components/properties-panel.tsx](src/windows/main/screens/write/components/properties-panel.tsx)
  - Action: Create the write-screen properties content shell that reads the current sequencer selection, renders a compact horizontally scrollable panel surface, and routes to selection-type-specific sections.
  - Notes: The shell should show an empty state when there is no selection or when the selected type has no implemented properties UI.
  - File: [src/windows/main/hooks/use-selected-tempo.ts](src/windows/main/hooks/use-selected-tempo.ts)
  - Action: Add a hook that resolves a tempo selection into the corresponding `Tempo` entry plus any needed context such as `flow_id` and master-track location.
  - Notes: Follow the same lookup-isolation pattern used by `useSelectedToneInfo()`.
  - File: [src/windows/main/hooks/use-selected-tone-info.ts](src/windows/main/hooks/use-selected-tone-info.ts)
  - Action: Confirm non-tone selections continue returning `null` cleanly and tighten logic only if the new union variant requires it.
  - Notes: Avoid regressions in current tone info consumers.
  - File: [src/windows/main/screens/write/components/bottom-panel.tsx](src/windows/main/screens/write/components/bottom-panel.tsx)
  - Action: Render the new `PropertiesPanel` when the active write bottom-panel tab is `BottomPanelTab.Properties`.
  - Notes: Leave existing sequencer and mixer behavior untouched.

- [ ] Task 6: Implement the compact tempo properties section UI
  - File: [src/windows/main/screens/write/components/tempo-properties-section.tsx](src/windows/main/screens/write/components/tempo-properties-section.tsx)
  - Action: Build the tempo-specific properties section with compact grouped controls for text, text visibility, BPM, beat type, metronome visibility, and parenthesized display.
  - Notes: Use existing UI primitives (`TextInput`, `NumberInput`, `Switch`, `IconButton`) and keep the layout dense, horizontally scrollable, and ready for future multi-section property groups.
  - File: [src/windows/main/screens/write/components/properties-panel.tsx](src/windows/main/screens/write/components/properties-panel.tsx)
  - Action: Register the tempo section in the selection-type switch so tempo selections render their section and other types still fall back to the empty state.
  - Notes: This is the key future-proofing seam for later tone/note/rest properties work.
  - File: [src/services/types.ts](src/services/types.ts)
  - Action: Reuse the existing `Tempo` type fields directly in the section props and avoid introducing duplicate view-model types unless a clear transformation boundary is needed.
  - Notes: The current schema already matches the requested controls closely.

- [ ] Task 7: Connect tempo properties controls to immediate backend updates
  - File: [src/windows/main/screens/write/components/tempo-properties-section.tsx](src/windows/main/screens/write/components/tempo-properties-section.tsx)
  - Action: Commit text changes on blur or Enter, and apply all other control changes immediately so score rendering and selection state stay live.
  - Notes: This is an explicit UX rule for the slice, not an implementation fallback.
  - File: [src/windows/main/hooks/use-keyboard-shortcuts.ts](src/windows/main/hooks/use-keyboard-shortcuts.ts)
  - Action: Preserve existing tone-only shortcut behavior by bailing out cleanly when the active selection is not `Tone`.
  - Notes: Tempo selection must not accidentally trigger tone mutation shortcuts.

- [ ] Task 8: Add focused automated coverage for tempo selection and properties-panel behavior
  - File: [src/windows/main/screens/write/components/bottom-panel.test.tsx](src/windows/main/screens/write/components/bottom-panel.test.tsx)
  - Action: Add tests that verify the `Properties` tab renders the new panel shell and appropriate empty/tempo states.
  - Notes: Extend, do not replace, the current tab rendering tests.
  - File: [src/windows/main/screens/write/components/properties-panel.test.tsx](src/windows/main/screens/write/components/properties-panel.test.tsx)
  - Action: Add tests for no-selection empty state, unsupported-selection empty state, and tempo-section routing.
  - Notes: Keep the panel foundation testable independently from the larger bottom-panel shell.
  - File: [src/windows/main/screens/write/components/tempo-properties-section.test.tsx](src/windows/main/screens/write/components/tempo-properties-section.test.tsx)
  - Action: Add tests covering compact control rendering and immediate action dispatch for each tempo property.
  - Notes: Verify the beat-type segmented controls reflect current selection state.
  - File: [src/windows/main/hooks/use-selected-tempo.test.ts](src/windows/main/hooks/use-selected-tempo.test.ts)
  - Action: Add hook tests for resolving a valid tempo selection and returning `null` for missing/invalid references.
  - Notes: Mirror the existing hook-test style used elsewhere in the write screen.
  - File: [src/windows/main/components/renderer.tsx](src/windows/main/components/renderer.tsx)
  - Action: Add or update tests so a tempo-shaped hit-test result dispatches the tempo selection action instead of falling through to pan/clear behavior.
  - Notes: Keep tone/rest click behavior intact.
  - File: [src-tauri/src/instructions/draw/draw_tempo.rs](src-tauri/src/instructions/draw/draw_tempo.rs)
  - Action: Add inline Rust tests verifying tempo selection highlighting and hit-target emission dimensions/content.
  - Notes: Reuse the current draw-helper test style.
  - File: [src-tauri/src/ui/sequencer.rs](src-tauri/src/ui/sequencer.rs)
  - Action: Add Rust tests for setting and clearing tempo selection state if the module already supports that test style, or add the narrowest practical coverage nearby.
  - Notes: Selection serialization parity matters because hit targets and state matching depend on exact shape equality.

- [ ] Task 9: Update documentation for the now-functional write properties panel
  - File: [docs/architecture.md](docs/architecture.md)
  - Action: Update architecture notes to explain that write-screen score selection now includes tempo directives via native renderer hit targets and that the bottom-panel `Properties` tab hosts immediate tempo editing.
  - Notes: Document that this slice changes UI/interaction behavior but not persisted `.soloc` schema, so no migration is required.
  - File: [docs/component-inventory.md](docs/component-inventory.md)
  - Action: Add the new write-screen properties-panel components and their responsibilities.
  - Notes: This keeps the component map current for future engrave-screen reuse.
  - File: [docs/project-context.md](docs/project-context.md)
  - Action: Update only if implementation introduces a new standing project rule or agent-facing convention.
  - Notes: Current expectation is that no AI-agent instruction update is required unless the implementation reveals a durable new rule.

### Acceptance Criteria

- [ ] AC 1: Given the write screen is open and the user clicks a rendered tempo marking, when the renderer hit test resolves that marking, then the existing sequencer selection state becomes a tempo selection and the tempo marking is highlighted in the score.
- [ ] AC 2: Given a tempo marking is selected, when the user remains on the current bottom-panel tab, then the selection highlight updates without automatically switching the UI to the `Properties` tab.
- [ ] AC 3: Given a tempo marking is selected and the user opens the `Properties` tab, when the panel renders, then it shows a compact horizontally scrollable tempo properties section instead of the previous placeholder-empty content.
- [ ] AC 4: Given no score item is selected, when the user opens the `Properties` tab, then the panel shows an explicit empty state.
- [ ] AC 5: Given a non-tempo selection type that has no implemented properties UI yet, when the user opens the `Properties` tab, then the panel shows an explicit unsupported/empty state rather than broken or misleading controls.
- [ ] AC 6: Given a selected tempo with textual content, when the user edits the text field and commits with blur or Enter, then the tempo entry updates through the action/backend path without requiring an apply button and the score re-renders with the new text.
- [ ] AC 7: Given a selected tempo, when the user toggles text visibility, then the tempo updates immediately and the score hides or shows the descriptive text portion accordingly.
- [ ] AC 8: Given a selected tempo, when the user changes the BPM control, then the tempo entry updates immediately and subsequent rendering/playback state continues to reflect the edited BPM value.
- [ ] AC 9: Given a selected tempo, when the user changes the beat-type segmented control, then the selected beat type updates immediately and the metronome glyph in score rendering reflects the new note value.
- [ ] AC 10: Given a selected tempo, when the user toggles metronome visibility, then the tempo updates immediately and the score hides or shows the metronome portion accordingly.
- [ ] AC 11: Given a selected tempo with metronome display enabled, when the user toggles parenthesized display, then the tempo updates immediately and the rendered metronome text reflects the parentheses state.
- [ ] AC 12: Given a metronome-only tempo, when it is stored or edited, then the data model uses an empty string plus `show_text = false` rather than relying on `Option<String>` to represent missing text.
- [ ] AC 13: Given a tempo selection is active, when the user clicks empty renderer space, then the existing clear-selection behavior still clears the selection and the `Properties` tab returns to its empty state.
- [ ] AC 14: Given tone-specific keyboard shortcuts already exist, when a tempo selection is active, then those tone mutation shortcuts do not fire against tempo data.
- [ ] AC 15: Given a tempo marking is rendered, when its visible bounds are clicked near any of its displayed parts, then the hitbox is large enough to make selection practical and consistent with the drawn content.
- [ ] AC 16: Given the properties-panel foundation is introduced for tempo first, when future non-tempo selection types are added, then the panel architecture already supports dynamic per-selection section rendering without rewriting the shell.
- [ ] AC 17: Given this slice changes write-screen interaction behavior and adds new components, when the work is complete, then [docs/architecture.md](docs/architecture.md) and [docs/component-inventory.md](docs/component-inventory.md) are updated accordingly.
- [ ] AC 18: Given an older saved project still contains tempo payloads with `text: null`, when the project is loaded after this slice, then migration/backward-compatible load logic normalizes the data and the file loads successfully.
- [ ] AC 19: Given this slice changes persisted tempo schema, when the work is complete, then the spec and updated docs explicitly note that a migration/backward-compatible load path was required and implemented.
- [ ] AC 20: Given this slice introduces a persisted-data migration, when implementation begins, then `npm run release:minor` is used before the schema change lands so version-synced files are updated consistently and release notes are generated.
- [ ] AC 21: Given the project uses version-targeted migration modules, when this slice is implemented, then the tempo migration lives in [src-tauri/src/migrations/1.24.0.rs](src-tauri/src/migrations/1.24.0.rs) and is registered from [src-tauri/src/migrations/mod.rs](src-tauri/src/migrations/mod.rs).

### Documentation Impact

- Update [docs/architecture.md](docs/architecture.md) to document that write-screen native score selection now includes tempo markings via renderer hit targets and that tempo properties are edited from the bottom-panel `Properties` tab, with text commit on blur/Enter and other tempo controls applying immediately.
- Update [docs/component-inventory.md](docs/component-inventory.md) to add the new write-screen properties-panel shell, tempo section component, and any new selection hook introduced for resolving tempo data.
- Update [docs/data-models.md](docs/data-models.md) to reflect the tempo schema change from nullable text to required string plus `show_text` visibility semantics.
- Explicitly note in the spec/docs that this slice changes persisted tempo schema or load semantics and therefore required a migration/backward-compatible load path for older `.soloc` files.
- Update any release/versioning notes touched by the required pre-work minor bump if they would otherwise reference the old version, and treat the generated release notes from `npm run release:minor` as part of that scripted versioning flow rather than hand-editing version strings.
- No AI-agent instruction update is expected unless implementation introduces a new durable project rule; if that happens during development, update [docs/project-context.md](docs/project-context.md) as part of the same slice.

## Additional Context

### Dependencies

- No new external libraries are currently required; the work should use existing project primitives and Tauri command infrastructure.
- The feature depends on the already-implemented immediate tempo entry model and tempo rendering support in [src-tauri/src/score/entries/tempo.rs](src-tauri/src/score/entries/tempo.rs) and [src-tauri/src/instructions/draw/draw_tempo.rs](src-tauri/src/instructions/draw/draw_tempo.rs).
- The properties-panel foundation depends on the existing write-screen bottom-panel shell and sequencer selection state already present in the app state.
- If a dedicated tempo update command is introduced, frontend work depends on that backend command contract being defined first so UI actions can bind to a stable mutation surface.
- Because this plan changes persisted tempo schema, implementation also depends on the project migration layer in [src-tauri/src/migrations/mod.rs](src-tauri/src/migrations/mod.rs).
- Because this plan uses version-targeted migrations, implementation also depends on the scripted pre-work minor version bump landing first via `npm run release:minor` so the migration target version is correct and stable.

### Testing Strategy

- Add frontend action tests for any new tempo selection/update wrappers in [src/services/actions.test.ts](src/services/actions.test.ts).
- Add or extend service tests in [src/services/hit-test.test.ts](src/services/hit-test.test.ts) so generic hit-test payload handling remains correct with tempo selections.
- Add component tests for the write bottom-panel, properties-panel shell, and tempo properties section using `renderWithProviders()` and realistic app-state fixtures.
- Add hook tests for tempo-selection resolution and ensure current tone-only hooks continue returning `null`/no-op when selection is not a tone.
- Add renderer interaction coverage so tempo hit-test results dispatch tempo selection correctly and empty clicks still clear selection.
- Add inline Rust tests in the tempo draw/selection path to verify selected coloring and hit-target emission.
- Add migration/load compatibility tests proving older tempo payloads with `text: null` are normalized and still load successfully.
- Validate that `npm run release:minor` updated version-synced files consistently and produced the expected release-note output before schema work proceeds.
- Manual validation should cover: selecting tempos at different ticks, verifying highlight color, opening the `Properties` tab, editing each control, confirming immediate score updates, clearing selection by clicking empty space, and confirming tone keyboard shortcuts remain safe when tempo is selected.

### Notes

- Highest-risk area: choosing an immediate-update strategy for tempo text editing that feels live without spamming backend churn or causing awkward selection flicker. The spec prefers a dedicated update command to keep this manageable.
- Highest-risk renderer detail: tempo hitboxes must cover the full visible tempo content, including text-only, metronome-only, and parenthesized forms; otherwise the feature will feel unreliable even if technically implemented.
- This spec now treats `show_text` as a first-class user-facing control and recommends simplifying the stored tempo text field to a required string so visibility semantics live in one place.
- Highest-risk persistence detail: the required-string text change will break older saved data unless migration/backward-compatible load handling is implemented correctly and tested.
- Highest-risk release-management detail: if `npm run release:minor` is skipped or delayed until after schema work starts, old/new `.soloc` boundaries become less precise and the migration target version becomes muddier.
- This slice intentionally builds a reusable panel shell for future selection types such as tones, but it should not over-generalize so far that tempo delivery slows down.
- Out-of-scope future follow-up options include note/rest property sections, engrave-screen reuse of the properties-panel foundation, and broader score-directive editing beyond tempo.
