---
title: "Directive Selection Cleanup for Tempo, Time Signature, Key Signature, and Clef"
slug: "directive-selection-cleanup-tempo-time-signature-key-signature-clef"
created: "2026-03-09"
status: "ready-for-dev"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  - "TypeScript 5.9 + React 19 frontend with hook-based state derivation and typed Tauri action wrappers"
  - "Tauri 2.10 + Rust 2021 backend commands with mirrored FE/BE serialized selection types"
  - "Native Skia/Metal score rendering with Rust-side draw instructions and hit-target based selection"
  - "Vitest 4 + Testing Library for frontend services, hooks, and component interaction coverage"
  - "Inline Rust #[cfg(test)] unit tests beside score-entry, renderer, and draw modules"
files_to_modify:
  - "src/services/types.ts"
  - "src/services/actions.ts"
  - "src/services/actions.test.ts"
  - "src/services/hit-test.ts"
  - "src/services/hit-test.test.ts"
  - "src/windows/main/components/renderer.tsx"
  - "src/windows/main/components/renderer.test.tsx"
  - "src/windows/main/hooks/use-keyboard-shortcuts.ts"
  - "src/windows/main/hooks/use-keyboard-shortcuts.test.ts"
  - "src/windows/main/screens/write/components/properties-panel.tsx"
  - "src/windows/main/screens/write/components/properties-panel.test.tsx"
  - "src/windows/main/screens/write/components/time-signature-properties-section.tsx"
  - "src/windows/main/screens/write/components/time-signature-properties-section.test.tsx"
  - "src/windows/main/screens/write/components/key-signature-properties-section.tsx"
  - "src/windows/main/hooks/use-selected-time-signature.ts"
  - "src/windows/main/hooks/use-selected-time-signature.test.ts"
  - "src/windows/main/hooks/use-selected-key-signature.ts"
  - "src/windows/main/hooks/use-selected-key-signature.test.ts"
  - "src/windows/main/hooks/use-selected-clef.ts"
  - "src/windows/main/hooks/use-selected-clef.test.ts"
  - "src-tauri/src/ui/sequencer.rs"
  - "src-tauri/src/ui/mod.rs"
  - "src-tauri/src/lib.rs"
  - "src-tauri/src/renderer/hit_test.rs"
  - "src-tauri/src/instructions/mod.rs"
  - "src-tauri/src/instructions/draw/draw_clef.rs"
  - "src-tauri/src/instructions/draw/draw_key_signature.rs"
  - "src-tauri/src/instructions/draw/draw_time_signature.rs"
  - "src-tauri/src/score/entries/clef.rs"
  - "src-tauri/src/score/entries/key_signature.rs"
  - "src-tauri/src/score/entries/time_signature.rs"
  - "docs/architecture.md"
  - "docs/component-inventory.md"
  - "docs/data-models.md"
code_patterns:
  - "Sequencer selection is a mirrored tagged union between src/services/types.ts and src-tauri/src/ui/sequencer.rs, with explicit Tauri commands per selection kind"
  - "Renderer click handling in src/windows/main/components/renderer.tsx first runs renderer_hit_test, then dispatches typed selection commands based on the returned hit-target payload"
  - "Native selection and popover anchoring depend on exact Selection equality against renderer HitTarget.entry in src-tauri/src/ui/mod.rs"
  - "Directive drawing currently happens in Rust draw helpers and orchestration in src-tauri/src/instructions/mod.rs; tempo already emits selection hit targets while clef, key signature, and time signature do not"
  - "Write-screen properties UI is selection-driven and uses focused hooks/components such as useSelectedTempo() and TempoPropertiesSection to keep bottom-panel composition light"
  - "Keyboard deletion/edit shortcuts are centralized in use-keyboard-shortcuts.ts and currently bail out unless the selection resolves to a Tone"
test_patterns:
  - "Frontend action wrappers assert exact invoke payloads in src/services/actions.test.ts"
  - "Renderer interaction coverage uses mocked hit-test responses in src/windows/main/components/renderer.test.tsx"
  - "Selection-derived hooks use renderHook with createWrapper/createAppState fixtures"
  - "Bottom-panel properties behavior uses focused Testing Library component tests beside each section component"
  - "Rust renderer and score-entry behavior is covered by inline unit tests next to hit-test, draw, and entry-command modules"
---

# Tech-Spec: Directive Selection Cleanup for Tempo, Time Signature, Key Signature, and Clef

**Created:** 2026-03-09

## Overview

### Problem Statement

Score directives do not yet participate consistently in the write-screen selection workflow. Tempo deletion hotkeys should match tone deletion, while time signatures, key signatures, and clefs still lack the same level of direct selection, delete-key support, and properties-panel integration. This leaves core notation items behaving inconsistently and blocks a unified score-editing workflow.

### Solution

Extend the existing sequencer-selection, renderer hit-target, delete-hotkey, and properties-panel patterns so tempos, time signatures, key signatures, and clefs all behave consistently in the write screen. Add selectable/deletable support for time signatures, key signatures, and clefs; make tempo deletion follow the current tone-delete flow; add a time-signature properties section with a guarded common-time / cut-time toggle; add a blank key-signature properties section; and preserve start-of-flow invariants by reverting tick-0 clef deletion to the instrument default clef while keeping the initial time signature protected at tick 0.

### Scope

**In Scope:**

- Delete-key support for selected tempo markings using the same interaction model as selected tones.
- Selectable and deletable time signatures in the write screen.
- Time-signature properties panel content, including a `C` / `cutC` toggle that maps to draw type and is disabled for invalid signatures.
- Selectable and deletable key signatures in the write screen.
- Blank key-signature properties content that matches the existing filler style used by unimplemented properties areas.
- Selectable and deletable clefs in the write screen.
- Tick-0 clef deletion behavior that restores the assigned instrument stave’s default clef instead of leaving the flow without an opening clef.
- Explicit handling of tick-0 time signatures so delete does not break flow initialization semantics.

**Out of Scope:**

- New editable key-signature controls beyond the blank properties section shell.
- Broader properties-panel redesign outside the selection types needed for this cleanup.
- Non-write-screen UX changes except where shared selection/rendering infrastructure must be extended.
- New creation workflows for directives; this slice is about selection, deletion, and properties integration.

## Context for Development

### Codebase Patterns

- Sequencer selection is intentionally mirrored across [src/services/types.ts](src/services/types.ts) and [src-tauri/src/ui/sequencer.rs](src-tauri/src/ui/sequencer.rs), with one explicit Tauri setter command per selectable score item. Today only `Tone`, `Rest`, and `Tempo` are wired end-to-end.
- The click path in [src/windows/main/components/renderer.tsx](src/windows/main/components/renderer.tsx) is hard-coded branch logic over hit-test result type. Adding time signatures, key signatures, and clefs means extending this branch table rather than inventing a generic dispatcher.
- Native popover positioning and selection persistence depend on exact `Selection` equality in [src-tauri/src/ui/mod.rs](src-tauri/src/ui/mod.rs), so each new selectable directive must emit a stable `HitTarget.entry` payload that matches the FE/BE selection union exactly.
- Tempo is the reference implementation for selectable directives: [src-tauri/src/instructions/draw/draw_tempo.rs](src-tauri/src/instructions/draw/draw_tempo.rs) already applies selected-state tinting, emits hit targets, and drives hook-based properties UI through [src/windows/main/hooks/use-selected-tempo.ts](src/windows/main/hooks/use-selected-tempo.ts) and [src/windows/main/screens/write/components/tempo-properties-section.tsx](src/windows/main/screens/write/components/tempo-properties-section.tsx).
- Clef, key-signature, and time-signature draw helpers in [src-tauri/src/instructions/draw/draw_clef.rs](src-tauri/src/instructions/draw/draw_clef.rs), [src-tauri/src/instructions/draw/draw_key_signature.rs](src-tauri/src/instructions/draw/draw_key_signature.rs), and [src-tauri/src/instructions/draw/draw_time_signature.rs](src-tauri/src/instructions/draw/draw_time_signature.rs) currently render black-only glyphs and emit no hit targets, so they cannot participate in renderer selection yet.
- Keyboard shortcuts in [src/windows/main/hooks/use-keyboard-shortcuts.ts](src/windows/main/hooks/use-keyboard-shortcuts.ts) derive a tone-only `selectionInfo` object and early-return for every other selection kind. Tempo deletion parity and new directive deletion support should extend this centralized hook while preserving existing pitch/articulation shortcuts as tone-only.
- The current properties panel in [src/windows/main/screens/write/components/properties-panel.tsx](src/windows/main/screens/write/components/properties-panel.tsx) supports `Tempo`, `null`, and a generic unsupported state. This cleanup should preserve the reusable scroll-shell/filler pattern already used by tempo sections while adding type-specific routing for time signature and key signature, and deciding whether clef remains unsupported for now.
- Time-signature editing has one important data gap: Rust `TimeSignature` includes `draw_type` in [src-tauri/src/score/entries/time_signature.rs](src-tauri/src/score/entries/time_signature.rs), but the FE `TimeSignature` type in [src/services/types.ts](src/services/types.ts) currently omits it. The properties panel cannot expose `C` / `cutC` without synchronizing that model.
- Flow initialization in [src-tauri/src/score/flows.rs](src-tauri/src/score/flows.rs) guarantees an opening 4/4 time signature on the flow master track and instrument-default clefs on each stave master track. Current clear commands diverge: time signatures no-op at tick 0, clefs also no-op at tick 0, and key signatures clear freely.
- Flow-master directives such as time signatures and key signatures are rendered on every stave. This slice should make every visible rendered instance clickable while returning the same logical selection payload, with popover anchoring intentionally resolving to the first/top-stave matching hit target for stable placement.
- Hidden time signatures should remain non-selectable because they have no visible rendered glyphs; this slice should not create invisible hit targets for hidden directives.

### Files to Reference

| File                                                                                                                                             | Purpose                                                                                                                                                                                |
| ------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [src/services/types.ts](src/services/types.ts)                                                                                                   | Frontend `Entry`, `TimeSignature`, and `Selection` types; currently missing selection variants for time signature, key signature, and clef, and missing `draw_type` on `TimeSignature` |
| [src/services/actions.ts](src/services/actions.ts)                                                                                               | Tauri action wrappers for selection setters and score-entry clear/create commands                                                                                                      |
| [src/services/actions.test.ts](src/services/actions.test.ts)                                                                                     | Contract coverage for invoke payloads, including existing tempo/time/key/clef commands                                                                                                 |
| [src/services/hit-test.ts](src/services/hit-test.ts)                                                                                             | Typed frontend bridge to `renderer_hit_test`; likely unchanged in implementation but should be verified against the expanded `Selection` union                                         |
| [src/services/hit-test.test.ts](src/services/hit-test.test.ts)                                                                                   | Service-level smoke coverage for the expanded hit-test selection shapes                                                                                                                |
| [src/windows/main/components/renderer.tsx](src/windows/main/components/renderer.tsx)                                                             | Hit-test-to-selection bridge on the React overlay                                                                                                                                      |
| [src/windows/main/components/renderer.test.tsx](src/windows/main/components/renderer.test.tsx)                                                   | Existing pattern for asserting hit-test-driven selection dispatch                                                                                                                      |
| [src/windows/main/hooks/use-keyboard-shortcuts.ts](src/windows/main/hooks/use-keyboard-shortcuts.ts)                                             | Centralized Delete/Backspace/Escape and tone edit shortcut logic                                                                                                                       |
| [src/windows/main/hooks/use-keyboard-shortcuts.test.ts](src/windows/main/hooks/use-keyboard-shortcuts.test.ts)                                   | Existing tone/tempo shortcut safety coverage                                                                                                                                           |
| [src/windows/main/screens/write/components/properties-panel.tsx](src/windows/main/screens/write/components/properties-panel.tsx)                 | Current properties routing and filler/empty-state structure                                                                                                                            |
| [src/windows/main/screens/write/components/properties-panel.test.tsx](src/windows/main/screens/write/components/properties-panel.test.tsx)       | Current expectations for empty, unsupported, and tempo-specific properties rendering                                                                                                   |
| [src/windows/main/hooks/use-selected-tempo.ts](src/windows/main/hooks/use-selected-tempo.ts)                                                     | Reference pattern for selection-derived domain lookup hooks                                                                                                                            |
| [src/windows/main/screens/write/components/tempo-properties-section.tsx](src/windows/main/screens/write/components/tempo-properties-section.tsx) | Reference pattern for compact selection-specific properties sections                                                                                                                   |
| [src-tauri/src/ui/sequencer.rs](src-tauri/src/ui/sequencer.rs)                                                                                   | Backend `Selection` enum and registered setter commands                                                                                                                                |
| [src-tauri/src/ui/mod.rs](src-tauri/src/ui/mod.rs)                                                                                               | Selection-to-popover coordinate lookup via exact hit-target equality                                                                                                                   |
| [src-tauri/src/renderer/hit_test.rs](src-tauri/src/renderer/hit_test.rs)                                                                         | Renderer hit-testing command returning typed selection payloads                                                                                                                        |
| [src-tauri/src/instructions/mod.rs](src-tauri/src/instructions/mod.rs)                                                                           | Draw orchestration for clefs, key signatures, time signatures, and tempos                                                                                                              |
| [src-tauri/src/instructions/draw/draw_tempo.rs](src-tauri/src/instructions/draw/draw_tempo.rs)                                                   | Reference implementation for directive highlight + hit-target emission                                                                                                                 |
| [src-tauri/src/instructions/draw/draw_clef.rs](src-tauri/src/instructions/draw/draw_clef.rs)                                                     | Current clef drawing logic with no selection support                                                                                                                                   |
| [src-tauri/src/instructions/draw/draw_key_signature.rs](src-tauri/src/instructions/draw/draw_key_signature.rs)                                   | Current key-signature drawing logic with no selection support                                                                                                                          |
| [src-tauri/src/instructions/draw/draw_time_signature.rs](src-tauri/src/instructions/draw/draw_time_signature.rs)                                 | Current time-signature drawing logic with no selection support                                                                                                                         |
| [src-tauri/src/score/entries/clef.rs](src-tauri/src/score/entries/clef.rs)                                                                       | Clef create/clear behavior, including current tick-0 no-op branch                                                                                                                      |
| [src-tauri/src/score/entries/key_signature.rs](src-tauri/src/score/entries/key_signature.rs)                                                     | Key-signature create/clear behavior for flow-master directives                                                                                                                         |
| [src-tauri/src/score/entries/time_signature.rs](src-tauri/src/score/entries/time_signature.rs)                                                   | Time-signature create/clear behavior and `draw_type` data model                                                                                                                        |
| [src-tauri/src/score/flows.rs](src-tauri/src/score/flows.rs)                                                                                     | Opening 4/4 flow initialization and instrument-default clef insertion                                                                                                                  |
| [src-tauri/src/lib.rs](src-tauri/src/lib.rs)                                                                                                     | Command imports/registration that must stay aligned with new selection setters                                                                                                         |

### Technical Decisions

- Tempo deletion must follow the exact existing tone deletion pattern.
- Time-signature properties should disable invalid `C` / `cutC` states instead of auto-correcting stored data.
- Key-signature properties content should remain visually blank for now.
- Clefs join the same selection/deletion workflow as tempo, time signature, and key signature.
- The exact new selection payloads for this slice are: `TimeSignature { flow_id, time_signature_id, tick }`, `KeySignature { flow_id, key_signature_id, tick }`, and `Clef { flow_id, stave_id, clef_id, tick }`.
- Deleting a clef at tick 0 should restore the instrument’s default clef for that stave.
- Deleting a time signature at tick 0 should not remove the opening signature; current code already preserves the default opening 4/4, and this cleanup should keep that invariant unless Step 2 uncovers a stronger architectural constraint.
- New selection variants should mirror directive ownership boundaries: time signatures and key signatures live on the flow master track and therefore need `flow_id`, entry id, and `tick`; clefs are stave-specific and therefore also need `stave_id` for deletion and lookup.
- The renderer/hit-test stack should stay generic. Selection support for time signatures, key signatures, and clefs should come from emitting `HitTarget`s plus branch additions in the existing FE click dispatcher, not from a new renderer API.
- `Delete` and `Backspace` should remain centralized in the keyboard-shortcut hook, with tone-only pitch/articulation shortcuts left untouched. For non-tick-0 deletions, the frontend shortcut flow should clear sequencer selection after the delete command resolves so stale selection does not survive removed entries.
- The time-signature properties section requires syncing `draw_type` into the FE `TimeSignature` model and adding a dedicated mutation path in the existing time-signature command surface. In this slice the only editable time-signature property is `draw_type`, using an exact command contract of `score_entries_time_signature_update(flow_id, time_signature_id, draw_type)`.
- The time-signature properties UI should use a single boolean switch labelled as symbol form. For 4/4, switch `on` maps to `CommonTime`; for 2/2, switch `on` maps to `SplitCommonTime`; switch `off` maps to `Regular`; all other signatures render the switch disabled.
- Key-signature properties should render inside the same horizontally scrollable shell as tempo properties, but the content area should be visually blank filler rather than an explanatory unsupported state.
- Clef parity in this slice is limited to selection, highlight, and delete-hotkey support; a dedicated clef properties panel remains out of scope.
- Clef tick-0 deletion cannot remain a no-op; it needs an explicit restore-default path that finds the owning instrument by `stave_id`, derives that stave’s index within `instrument.stave_ids`, loads `definition.staves[index].clef`, and mutates the opening clef entry in place so selection identity remains stable.
- Time-signature tick-0 deletion should remain protected as a no-op because flow creation already guarantees an opening 4/4 and there is no separate user-facing fallback workflow required for this cleanup.
- Adding frontend `TimeSignature.draw_type` does not change persisted `.soloc` schema because Rust already serializes that field today; this slice only aligns the TypeScript mirror and adds ephemeral UI selection payloads, so no migration is required.

## Implementation Plan

### Tasks

- [ ] Task 1: Extend FE/BE selection models for directive entries
  - File: [src/services/types.ts](src/services/types.ts)
  - Action: Add `TimeSignatureSelection`, `KeySignatureSelection`, and `ClefSelection` to the frontend `Selection` union using the exact payloads `type: "TimeSignature", flow_id, time_signature_id, tick`; `type: "KeySignature", flow_id, key_signature_id, tick`; and `type: "Clef", flow_id, stave_id, clef_id, tick`; also include `draw_type` on the frontend `TimeSignature` entry model.
  - Notes: These exact field names must mirror Rust serialization with no aliasing.
  - File: [src-tauri/src/ui/sequencer.rs](src-tauri/src/ui/sequencer.rs)
  - Action: Add matching Rust `Selection` variants and new setter commands named `ui_sequencer_selection_set_time_signature`, `ui_sequencer_selection_set_key_signature`, and `ui_sequencer_selection_set_clef`.
  - Notes: Keep serde-tagged shapes aligned with TypeScript and preserve existing tone/rest/tempo behavior.
  - File: [src-tauri/src/lib.rs](src-tauri/src/lib.rs)
  - Action: Import and register the new selection-set commands.
  - Notes: Follow existing snake_case command naming.

- [ ] Task 2: Make time signatures, key signatures, and clefs selectable in native rendering
  - File: [src-tauri/src/instructions/draw/draw_clef.rs](src-tauri/src/instructions/draw/draw_clef.rs)
  - Action: Thread selection context into `draw_clef()`, tint selected clefs with the existing accent color, and emit stave-aware hit targets with stable popover coordinates.
  - Notes: Hit targets must include `stave_id` because clef deletion is stave-scoped.
  - File: [src-tauri/src/instructions/draw/draw_key_signature.rs](src-tauri/src/instructions/draw/draw_key_signature.rs)
  - Action: Add selected-state coloring and flow-master key-signature hit-target emission on every visible stave instance, all returning the same `KeySignature` selection payload.
  - Notes: Reuse existing metrics/pattern geometry so hit boxes match visible accidentals, and keep popover anchoring stable by allowing the first/top-stave hit target to win.
  - File: [src-tauri/src/instructions/draw/draw_time_signature.rs](src-tauri/src/instructions/draw/draw_time_signature.rs)
  - Action: Add selected-state coloring and flow-master time-signature hit-target emission on every visible stave instance.
  - Notes: Cover regular, common time, cut time, and open draw types consistently; hidden time signatures must emit no hit target.
  - File: [src-tauri/src/instructions/mod.rs](src-tauri/src/instructions/mod.rs)
  - Action: Pass selection context and ownership ids into the clef, key-signature, and time-signature draw helpers.
  - Notes: Keep the existing tempo selection path as the reference implementation.
  - File: [src-tauri/src/renderer/hit_test.rs](src-tauri/src/renderer/hit_test.rs)
  - Action: Add focused tests proving the generic hit test returns the new selection variants.
  - Notes: No renderer API changes should be required beyond new `Selection` payloads.

- [ ] Task 3: Wire renderer click handling for new directive selections
  - File: [src/services/hit-test.ts](src/services/hit-test.ts)
  - Action: Verify the existing typed hit-test bridge remains correct with the expanded `Selection` union and adjust only if compile-time typing requires it.
  - Notes: Behavior should remain a thin pass-through.
  - File: [src/services/hit-test.test.ts](src/services/hit-test.test.ts)
  - Action: Add lightweight assertions covering `TimeSignature`, `KeySignature`, and `Clef` hit-test results.
  - Notes: This is mostly regression coverage for the widened selection union.
  - File: [src/windows/main/components/renderer.tsx](src/windows/main/components/renderer.tsx)
  - Action: Extend the hit-test branch table so `TimeSignature`, `KeySignature`, and `Clef` results dispatch the matching sequencer selection actions.
  - Notes: Preserve current panning fallback and current tempo/tone/rest behavior.
  - File: [src/services/actions.ts](src/services/actions.ts)
  - Action: Add new `actions.ui.sequencer.selectTimeSignature()`, `selectKeySignature()`, and `selectClef()` wrappers.
  - Notes: Keep payload argument order explicit and mirrored with Rust command signatures.
  - File: [src/services/actions.test.ts](src/services/actions.test.ts)
  - Action: Add invoke contract tests for the new selection wrappers.
  - Notes: Assert exact snake_case payloads.
  - File: [src/windows/main/components/renderer.test.tsx](src/windows/main/components/renderer.test.tsx)
  - Action: Add renderer interaction tests for all new hit-test result types.
  - Notes: Follow the existing mocked `hitTest()` pattern.

- [ ] Task 4: Generalize delete-key behavior across selectable directives
  - File: [src/windows/main/hooks/use-keyboard-shortcuts.ts](src/windows/main/hooks/use-keyboard-shortcuts.ts)
  - Action: Refactor the tone-only `selectionInfo` flow into selection-kind-aware handlers so `Delete` and `Backspace` remove selected tempo, time signature, key signature, and clef entries while keeping pitch/articulation shortcuts tone-only.
  - Notes: `Escape` should continue clearing selection for every selectable type; for non-tick-0 deletions that remove an entry, call `actions.ui.sequencer.clearSelection()` after the delete command resolves.
  - File: [src/services/actions.ts](src/services/actions.ts)
  - Action: Reuse existing score clear commands for tempo, time signature, key signature, and clef deletion.
  - Notes: No new frontend delete-specific APIs are needed if the current clear commands cover the behavior.
  - File: [src/windows/main/hooks/use-keyboard-shortcuts.test.ts](src/windows/main/hooks/use-keyboard-shortcuts.test.ts)
  - Action: Add coverage proving delete/backspace dispatch correctly for tempo, time signature, key signature, and clef selections, and proving tone-only edit shortcuts still do nothing for non-tone selections.
  - Notes: Retain current tone regression coverage.

- [ ] Task 5: Implement backend deletion invariants for tick-0 clefs and protected time signatures
  - File: [src-tauri/src/score/entries/clef.rs](src-tauri/src/score/entries/clef.rs)
  - Action: Replace the tick-0 early return with logic that restores the stave’s instrument-default clef when clearing the opening clef.
  - Notes: Resolve the owning instrument by finding the instrument whose `stave_ids` contains `stave_id`, derive the stave index within that instrument, load `get_instrument_definition(instrument.definition_id).staves[index].clef`, and mutate the existing tick-0 clef entry in place so the `clef_id` stays stable.
  - File: [src-tauri/src/score/flows.rs](src-tauri/src/score/flows.rs)
  - Action: Extract or reuse a helper that derives the default clef for a specific `stave_id` from the assigned instrument definition.
  - Notes: The helper must be index-aware so multi-stave instruments restore the correct stave-specific clef.
  - File: [src-tauri/src/score/entries/time_signature.rs](src-tauri/src/score/entries/time_signature.rs)
  - Action: Preserve and test the existing tick-0 no-op behavior for time-signature clear.
  - Notes: This is a deliberate invariant, not dead code.
  - File: [src-tauri/src/score/entries/key_signature.rs](src-tauri/src/score/entries/key_signature.rs)
  - Action: Add/adjust tests for ordinary key-signature clear behavior so directive-delete coverage stays balanced.
  - Notes: No tick-0 fallback is required for key signatures in this slice.

- [ ] Task 6: Add time-signature editing support for the properties panel
  - File: [src/services/actions.ts](src/services/actions.ts)
  - Action: Add a dedicated `actions.score.timeSignature.update(flow_id, time_signature_id, draw_type)` wrapper.
  - Notes: In this slice `draw_type` is the only editable time-signature property; do not add beats or beat-type editing.
  - File: [src-tauri/src/score/entries/time_signature.rs](src-tauri/src/score/entries/time_signature.rs)
  - Action: Add a backend command named `score_entries_time_signature_update(flow_id, time_signature_id, draw_type)` that mutates the existing entry in place.
  - Notes: Guard invalid `CommonTime`/`SplitCommonTime` combinations at the command layer as a safety check, and leave `beats` / `beat_type` untouched.
  - File: [src-tauri/src/lib.rs](src-tauri/src/lib.rs)
  - Action: Register the new time-signature update command.
  - Notes: Keep FE/BE command surfaces aligned.
  - File: [src/services/actions.test.ts](src/services/actions.test.ts)
  - Action: Add invoke contract coverage for the new time-signature update action.
  - Notes: Assert exact payload fields including `draw_type`.

- [ ] Task 7: Build selection-derived hooks and properties-panel sections
  - File: [src/windows/main/hooks/use-selected-time-signature.ts](src/windows/main/hooks/use-selected-time-signature.ts)
  - Action: Add a hook that resolves a selected time signature from the flow master track.
  - Notes: Mirror the `useSelectedTempo()` pattern.
  - File: [src/windows/main/hooks/use-selected-key-signature.ts](src/windows/main/hooks/use-selected-key-signature.ts)
  - Action: Add a hook that resolves a selected key signature from the flow master track.
  - Notes: Return `null` safely when selection or backing data is missing.
  - File: [src/windows/main/hooks/use-selected-clef.ts](src/windows/main/hooks/use-selected-clef.ts)
  - Action: Add a hook that resolves a selected clef from the target stave’s master track.
  - Notes: This primarily supports selection-aware UI/testing even if clef has no dedicated properties section yet.
  - File: [src/windows/main/screens/write/components/time-signature-properties-section.tsx](src/windows/main/screens/write/components/time-signature-properties-section.tsx)
  - Action: Build the time-signature properties section with a single boolean switch for symbol form mapped to `draw_type` and disabled for invalid signatures.
  - Notes: For 4/4, switch `on` = `CommonTime`; for 2/2, switch `on` = `SplitCommonTime`; switch `off` = `Regular`; all other signatures render the switch disabled.
  - File: [src/windows/main/screens/write/components/key-signature-properties-section.tsx](src/windows/main/screens/write/components/key-signature-properties-section.tsx)
  - Action: Add a blank filler section/component matching the existing properties-panel filler styling.
  - Notes: No editable controls in this slice.
  - File: [src/windows/main/screens/write/components/properties-panel.tsx](src/windows/main/screens/write/components/properties-panel.tsx)
  - Action: Route `TimeSignature` selections to the new time-signature section, `KeySignature` selections to the blank filler section, `Tempo` to the existing section, and leave `Clef` on the generic unsupported path.
  - Notes: Preserve the no-selection empty state and horizontal scroll shell; clef omission from properties is intentional and should be called out in component text/tests as out of scope rather than forgotten parity.

- [ ] Task 8: Add focused frontend tests for new selection hooks and properties UI
  - File: [src/windows/main/hooks/use-selected-time-signature.test.ts](src/windows/main/hooks/use-selected-time-signature.test.ts)
  - Action: Add hook tests for successful resolution and missing-data fallthrough.
  - Notes: Use `renderHook()` plus `createWrapper()` fixtures.
  - File: [src/windows/main/hooks/use-selected-key-signature.test.ts](src/windows/main/hooks/use-selected-key-signature.test.ts)
  - Action: Add hook tests for flow-master key-signature lookup.
  - Notes: Mirror the tempo hook test structure.
  - File: [src/windows/main/hooks/use-selected-clef.test.ts](src/windows/main/hooks/use-selected-clef.test.ts)
  - Action: Add hook tests for stave-master clef lookup and tick mismatch handling.
  - Notes: Ensure stave scoping is covered.
  - File: [src/windows/main/screens/write/components/properties-panel.test.tsx](src/windows/main/screens/write/components/properties-panel.test.tsx)
  - Action: Add assertions for time-signature routing, key-signature blank rendering, and clef unsupported rendering.
  - Notes: Keep existing empty/tempo coverage intact.
  - File: [src/windows/main/screens/write/components/time-signature-properties-section.test.tsx](src/windows/main/screens/write/components/time-signature-properties-section.test.tsx)
  - Action: Test enabled/disabled `C` / `cutC` behavior and update dispatch.
  - Notes: Cover 4/4, 2/2, and invalid signatures such as 3/4.

- [ ] Task 9: Add focused Rust tests for new selection and deletion behavior
  - File: [src-tauri/src/ui/sequencer.rs](src-tauri/src/ui/sequencer.rs)
  - Action: Add serialization tests for the new selection variants.
  - Notes: Preserve the existing tempo serialization test style.
  - File: [src-tauri/src/instructions/draw/draw_clef.rs](src-tauri/src/instructions/draw/draw_clef.rs)
  - Action: Add tests proving selected clefs change color and emit hit targets.
  - Notes: Cover full-size tick-0 and non-zero tick cases if metrics differ.
  - File: [src-tauri/src/instructions/draw/draw_key_signature.rs](src-tauri/src/instructions/draw/draw_key_signature.rs)
  - Action: Add tests for selected-state coloring and hit-target emission.
  - Notes: Cover zero-offset/no-visible-content behavior as well.
  - File: [src-tauri/src/instructions/draw/draw_time_signature.rs](src-tauri/src/instructions/draw/draw_time_signature.rs)
  - Action: Add tests for selected-state coloring and hit-target emission across regular/common/cut variants.
  - Notes: Ensure hidden time signatures still produce no selectable target.
  - File: [src-tauri/src/score/entries/clef.rs](src-tauri/src/score/entries/clef.rs)
  - Action: Add tests for tick-0 clear restoring the instrument-default clef.
  - Notes: Cover at least one single-stave and one multi-stave instrument scenario if helper extraction allows it.
  - File: [src-tauri/src/score/entries/time_signature.rs](src-tauri/src/score/entries/time_signature.rs)
  - Action: Add tests for the new update command and the retained tick-0 clear no-op behavior.
  - Notes: Include invalid draw-type guard coverage.

- [ ] Task 10: Update project documentation for the expanded directive-selection workflow
  - File: [docs/architecture.md](docs/architecture.md)
  - Action: Document that sequencer selection now includes tempo, time signature, key signature, and clef directives through renderer hit targets and shared selection state.
  - Notes: Include the tick-0 clef fallback and protected opening time-signature invariant.
  - File: [docs/component-inventory.md](docs/component-inventory.md)
  - Action: Add the new time-signature and key-signature properties-panel components/hooks.
  - Notes: Note that key-signature properties are intentionally blank and clef has no section yet.
  - File: [docs/data-models.md](docs/data-models.md)
  - Action: Update the documented FE/BE selection shapes and frontend `TimeSignature` model to include `draw_type`.
  - Notes: Explicitly state that this slice does not change persisted file schema, so no migration is required.

### Acceptance Criteria

- [ ] AC 1: Given a selected tempo marking in the write screen, when the user presses `Delete` or `Backspace`, then the app removes that tempo through the same shortcut pathway currently used for tone deletion.
- [ ] AC 2: Given a rendered time signature, key signature, or clef in the write screen, when the user clicks it, then the renderer hit test returns the correct typed selection payload and the item becomes the active sequencer selection.
- [ ] AC 3: Given a selected time signature, key signature, or clef, when the item is selected, then the native renderer highlights it with the same accent-color selection treatment used for other selectable score items.
- [ ] AC 4: Given a selected time signature or key signature, when the Properties tab is open, then the panel renders the correct type-specific content: editable time-signature controls for time signatures and blank filler content for key signatures.
- [ ] AC 5: Given a selected clef, when the Properties tab is open, then the panel does not crash and continues to show the generic unsupported-selection state because clef properties are intentionally out of scope for this slice.
- [ ] AC 6: Given a selected 4/4 time signature, when the user turns the symbol-form switch on, then the stored `draw_type` updates to `CommonTime`; and given a selected 2/2 time signature, when the user turns the same switch on, then the stored `draw_type` updates to `SplitCommonTime`.
- [ ] AC 7: Given a selected time signature that is not 4/4 or 2/2, when the Properties panel renders, then the `C` / `cutC` control is disabled and the app does not mutate the stored draw type.
- [ ] AC 8: Given a selected key signature or time signature away from tick 0, when the user presses `Delete` or `Backspace`, then the corresponding entry is cleared and selection remains consistent with the updated score state.
- [ ] AC 9: Given a selected clef away from tick 0, when the user presses `Delete` or `Backspace`, then the selected clef is removed from that stave master track.
- [ ] AC 10: Given a selected clef at tick 0, when the user presses `Delete` or `Backspace`, then the opening clef is replaced with that stave’s instrument-default clef rather than leaving the flow without an initial clef.
- [ ] AC 11: Given the opening time signature at tick 0 is selected, when the user presses `Delete` or `Backspace`, then the command is a safe no-op and the opening 4/4 time signature remains intact.
- [ ] AC 12: Given the codebase documentation is updated for this slice, when a fresh developer reads the architecture, component inventory, and data-model docs, then they can see which directive types participate in selection/properties workflows and which start-of-flow invariants are enforced.
- [ ] AC 13: Given a rendered key signature or time signature appears on multiple staves, when the user clicks any visible instance, then the app produces the same logical flow-master selection payload and keeps popover anchoring stable on the top-stave instance.
- [ ] AC 14: Given a time signature has `draw_type = Hidden`, when the score renders, then no visible hit target is emitted and the hidden time signature cannot be selected by clicking empty space.

### Documentation Impact

- [ ] Update [docs/architecture.md](docs/architecture.md) to describe expanded directive selection via renderer hit targets, shared sequencer selection state, and start-of-flow deletion invariants.
- [ ] Update [docs/component-inventory.md](docs/component-inventory.md) to list the new time-signature and key-signature properties-panel components and selection hooks.
- [ ] Update [docs/data-models.md](docs/data-models.md) to document the expanded `Selection` union and the frontend `TimeSignature.draw_type` field.
- [ ] Explicitly note in the documentation that this slice changes interaction behavior but does not change persisted `.soloc` schema, because `TimeSignature.draw_type` already exists in Rust-serialized score data and the new directive selection payloads live only in UI state, so no migration or `src-tauri/src/migrations/next.rs` change is required.

## Additional Context

### Dependencies

- No new external libraries are required.
- This slice depends on the existing tempo-selection foundation already present in the renderer, properties panel, and shared selection model.
- Time-signature properties depend on adding a backend update command and exposing `draw_type` in the frontend model; key-signature and clef selection depend on new Rust-side hit targets.
- Time-signature properties depend on a narrowly scoped update command that mutates only `draw_type` in place.
- Tick-0 clef fallback depends on resolving the owning instrument definition from the existing flow/stave relationships without altering persisted score schema.

### Testing Strategy

- Add frontend action tests for all new selection/update wrappers in [src/services/actions.test.ts](src/services/actions.test.ts).
- Add [src/services/hit-test.test.ts](src/services/hit-test.test.ts) coverage for the widened selection result shapes.
- Add renderer interaction tests in [src/windows/main/components/renderer.test.tsx](src/windows/main/components/renderer.test.tsx) for time-signature, key-signature, and clef hit results.
- Extend [src/windows/main/hooks/use-keyboard-shortcuts.test.ts](src/windows/main/hooks/use-keyboard-shortcuts.test.ts) to cover delete behavior for each directive selection type and to verify tone-only edit shortcuts remain scoped to tones.
- Add hook tests for `useSelectedTimeSignature()`, `useSelectedKeySignature()`, and `useSelectedClef()` using `renderHook()` plus app-state fixtures.
- Add component tests for the new time-signature properties section and the properties-panel routing outcomes for time signature, key signature, and clef selections.
- Add Rust unit tests for new selection serialization variants, directive draw hit targets/highlighting, tick-0 clef restore behavior, time-signature update behavior, and protected opening time-signature deletion.
- Manual verification: click-select each directive type in the write screen, confirm highlight state, confirm delete/backspace behavior, verify tick-0 clef restore on different instruments, verify tick-0 time-signature delete is a no-op, and verify the time-signature properties control is enabled only for 4/4 and 2/2.

### Notes

- Quick-scan finding: start-of-flow time signature deletion should not break today because [src-tauri/src/score/entries/time_signature.rs](src-tauri/src/score/entries/time_signature.rs#L321-L335) already returns early at tick 0, and new flows are initialized with a 4/4 time signature in [src-tauri/src/score/flows.rs](src-tauri/src/score/flows.rs#L146-L171).
- Quick-scan finding: start-of-flow clefs are created from instrument stave defaults during flow assignment in [src-tauri/src/score/flows.rs](src-tauri/src/score/flows.rs#L72-L103), but current clear logic in [src-tauri/src/score/entries/clef.rs](src-tauri/src/score/entries/clef.rs#L158-L181) also no-ops at tick 0, so the implementation will need an explicit restore-default path for your requested behavior.
- Highest-risk area: keeping FE and Rust `Selection` payloads perfectly mirrored while also preserving popover anchoring, because `ui_get_selection_coordinates()` relies on exact selection equality against stored hit targets.
- Highest-risk area: tick-0 clef fallback for multi-stave instruments, where the correct default clef must be derived per stave rather than per instrument globally.
- Highest-risk area: ensuring time-signature property edits preserve stable selection identity instead of replacing entries and invalidating the active selection.
- Selection payload names must not drift: `time_signature_id`, `key_signature_id`, and `clef_id` should be treated as fixed contract names across FE and Rust.
- Known limitation for this slice: clefs become selectable and deletable, but they do not gain a dedicated properties panel yet.
- Known limitation for this slice: key-signature properties intentionally remain blank, so selection support there is for consistency and future extensibility rather than new editing controls.
- Migration note: no persisted-data migration is needed because the only data-model change is frontend awareness of an already-persisted Rust field plus new non-persisted UI selection variants.
- Future follow-up candidate: add full time-signature editing beyond `draw_type`, plus dedicated clef and key-signature properties editing once the shared properties-panel foundation is in place.
