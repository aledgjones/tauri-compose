---
title: "Immediate Tempo Markings"
slug: "immediate-tempo-markings"
created: "2026-03-07"
status: "implemented"
stepsCompleted: [1, 2, 3, 4, 5, 6, 7]
tech_stack:
  - "TypeScript 5 + React 19 frontend with shared popover/action patterns"
  - "Rust 2021 + Tauri 2 backend with Skia/Metal score rendering"
  - "Vitest frontend unit tests and Rust #[cfg(test)] module tests"
files_to_modify:
  - "src/ui/popover.tsx"
  - "src/ui/popover.test.tsx"
  - "src/services/popovers.ts"
  - "src/services/popovers.test.ts"
  - "src/services/utils.ts"
  - "src/services/utils.test.ts"
  - "src/services/actions.ts"
  - "src/services/actions.test.ts"
  - "src/services/types.ts"
  - "src-tauri/src/score/entries/mod.rs"
  - "src-tauri/src/score/entries/tempo.rs"
  - "src-tauri/src/lib.rs"
  - "src-tauri/src/instructions/parse/get_horizontal_spacing.rs"
  - "src-tauri/src/instructions/mod.rs"
  - "src-tauri/src/instructions/draw/draw_tempo.rs"
  - "tests/fixtures/source/tempo/tempo.json"
  - "tests/fixtures/README.md"
code_patterns:
  - "New score directives are added as Rust Entry enum variants plus matching TypeScript types"
  - "Flow-wide directives live on the flow master track and are queried by tick during layout/draw"
  - "Frontend write-mode authoring uses PopoverType, keyboard shortcuts, strict string parsers, and actions.score IPC wrappers"
  - "Renderer text output can use Bravura for notation glyphs and Libre Baskerville for textual labels"
  - "Horizontal spacing reserves per-tick slot widths from entry metrics before draw instructions are emitted"
test_patterns:
  - "Frontend action wrappers are verified with invoke argument tests in src/services/actions.test.ts"
  - "Popover parsers use direct unit tests in src/services/popovers.test.ts"
  - "Popover UI behavior and keyboard shortcuts are covered in src/ui/popover.test.tsx"
  - "Rust entry modules keep inline behavior tests next to implementation"
  - "Rust draw helpers keep focused renderer instruction tests next to implementation"
---

# Tech-Spec: Immediate Tempo Markings

**Created:** 2026-03-07

## Overview

### Problem Statement

The score currently supports structural markings such as clefs, key signatures, and time signatures, but it does not support tempo markings attached to a specific musical tick in the score. Users need immediate tempo markings that can be authored at a selected point in the music, rendered above the top stave, and stored in score data so that future playback work can react to them.

### Solution

Add a new immediate tempo entry stored on the flow master track, rendered once above the top stave at its tick, and authored through a new strict popover workflow. Support four input/display forms in v1: `Allegro`, `Allegro q=120`, `Allegro (q=120)`, and `q=120`. Text tempo names will always resolve to a BPM under the hood, while explicit metronome forms store exact BPM values. Playback changes when transport crosses tempo markings should be designed as a follow-up spec, but the data model should support that future work cleanly.

V1 accepts a fixed named-tempo vocabulary and default BPM mapping: `Largo = 50`, `Adagio = 70`, `Andante = 90`, `Moderato = 110`, `Allegro = 130`, and `Presto = 180`. BPM entered explicitly through metronome syntax must be in the 20–400 range, matching the existing transport tempo limits. Parsing should remain as strict as the project’s existing popover parsing behavior.

### Scope

**In Scope:**

- New immediate tempo entry type for score data
- Storage on the flow master track
- Rendering above the top stave only
- Strict popover authoring at selected tick
- Supported forms: `Allegro`, `Allegro q=120`, `Allegro (q=120)`, `q=120`
- Derived default BPM for supported text-only tempo names
- Explicit BPM storage for metronome forms
- Supported named tempo vocabulary: `Largo`, `Adagio`, `Andante`, `Moderato`, `Allegro`, `Presto`
- Explicit BPM validation in the inclusive 20–400 range
- `clear` removes a tempo marking at a tick
- Creating a tempo marking at an occupied tick replaces the existing one
- Data model choices that keep future playback-following-tempo implementation straightforward
- Tick-0 tempo markings align with the musical start after opening clef/key/time material
- Tempo markings render only at the tick where they are entered and do not auto-repeat at later system starts in v1
- Renderer changes are covered by dedicated fixtures and rebuilt fixture output

**Out of Scope:**

- Gradual tempo changes such as `rit.`, `accel.`, or `rall.`
- Immediate relative tempo markings such as `meno mosso`, `più mosso`, `a tempo`, or `Tempo I`
- Dashed tempo extensions or spanner-style markings
- Playback interpolation between tempos
- Transport/playback automatically reacting to score tempo changes
- A unified tempo system that combines immediate and gradual changes into one entry type

## Context for Development

### Codebase Patterns

- Structural score entries are stored in Rust `Entries` collections with dual indexes: `by_tick` for lookup and `by_id` for storage, then mirrored to TypeScript via tagged `Entry` unions.
- Flow-wide directives such as time signatures live on the flow master track; `score_flows_create()` currently seeds the master track with a default 4/4 time signature only.
- New write-mode authoring features follow a stable frontend path: add a `PopoverType`, wire a keyboard shortcut in `src/ui/popover.tsx`, parse strict input in `src/services/popovers.ts`, and dispatch through `actions.score.*` Tauri commands.
- The Rust render pipeline separates spacing from drawing: `get_horizontal_spacing()` reserves tick-local slot widths from entry metrics, then `instructions/mod.rs` emits draw helpers during the stave contents phase.
- Text rendering already supports both notation glyphs (`FontFamily::Bravura`) and serif score text (`FontFamily::LibreBaskerville`), which is a good fit for mixed text + metronome tempo markings.
- Existing entry modules such as time signatures provide the clearest anchor pattern for create/clear/replace-at-tick behavior and for adding Tauri command exports through `src-tauri/src/lib.rs`.

### Files to Reference

| File                                                         | Purpose                                                                                            |
| ------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- |
| `src/ui/popover.tsx`                                         | Shared write popover UI, current Shift-based shortcuts, and dispatch on Enter                      |
| `src/ui/popover.test.tsx`                                    | UI tests for labels, shortcut behavior, and parser dispatch                                        |
| `src/services/popovers.ts`                                   | Strict parsing entry point for barline/clef/key/time popovers; best place for tempo syntax parsing |
| `src/services/popovers.test.ts`                              | Established parser test style for create/clear behavior                                            |
| `src/services/utils.ts`                                      | Existing parser utility location (`parseTimeSignature`) and likely home for a tempo parser/helper  |
| `src/services/utils.test.ts`                                 | Utility-level parsing tests for strict syntax rules                                                |
| `src/services/actions.ts`                                    | FE-to-BE command wrappers for score entry mutation IPC                                             |
| `src/services/actions.test.ts`                               | IPC wrapper verification pattern for new `actions.score.tempo` commands                            |
| `src/services/types.ts`                                      | TypeScript enums and tagged score entry unions that must stay in sync with Rust                    |
| `src-tauri/src/score/entries/mod.rs`                         | Core `Entry` enum and shared tick/id behavior for all score entries                                |
| `src-tauri/src/score/entries/time_signature.rs`              | Closest pattern for master-track create/clear/replace semantics and metrics-based spacing          |
| `src-tauri/src/score/components/text.rs`                     | Shared text/font primitives supporting Bravura and Libre Baskerville                               |
| `src-tauri/src/renderer/text_measurer.rs`                    | Existing text measurement helper for precise tempo-marking spacing                                 |
| `src-tauri/src/instructions/parse/get_horizontal_spacing.rs` | Slot-based spacing reservation that will need a tempo slot or equivalent accommodation             |
| `src-tauri/src/instructions/mod.rs`                          | Central draw loop where a new tempo draw pass should be inserted                                   |
| `src-tauri/src/instructions/draw/draw_time_signature.rs`     | Closest small draw-helper pattern to clone for a new tempo draw helper                             |
| `src-tauri/src/score/engraves.rs`                            | Existing engraving defaults and font choices that may inform tempo text styling                    |
| `src-tauri/src/lib.rs`                                       | Tauri command registration/import surface for any new score entry commands                         |
| `tests/fixtures/README.md`                                   | Fixture conventions and build rules for renderer-facing regression coverage                        |

### Technical Decisions

- Immediate tempo markings are a distinct entry type from gradual tempo changes.
- Tempo architecture should use a shared top-level tempo family with separate branches for `ImmediateTempo` and `GradualTempo`, rather than treating gradual tempo as just another immediate-tempo variant.
- Within `ImmediateTempo`, this spec establishes the path for future subtypes such as `ImmediateTempo.Absolute` and `ImmediateTempo.Relative`; this spec implements only the absolute branch.
- Tempo entries live on the flow master track, not per stave.
- Rendering occurs once per system context above the top stave rather than on every stave.
- V1 parsing can be strict and only recognize a limited supported vocabulary/syntax.
- Tempo text without explicit metronome syntax still stores a derived BPM; descriptive-only tempo entries are not allowed.
- Playback engine integration should be specified separately to reduce scope and risk.
- The current architecture supports tempo markings without touching note parsing logic; the main backend integration points are entry storage, spacing reservation, and draw-pass emission.
- Mixed tempo strings will likely need a small formatting model in Rust so textual words can use `LibreBaskerville` while the metronome notehead can use `Bravura`, instead of flattening everything into one font family.
- A new immediate-tempo parser should probably live beside `parseTimeSignature()` in `src/services/utils.ts`, with `parseTempoPopover()` in `src/services/popovers.ts` handling `clear` and dispatch rules.
- Flow creation does not currently seed a default tempo entry; v1 can avoid automatic insertion and let tempo markings be user-authored, which keeps the feature decoupled from existing transport default-tempo state.
- This spec implements **immediate absolute tempos only**, but the naming and structure should leave room for a future immediate-tempo family that can also contain **relative immediate tempos** without forcing a redesign.
- The persisted `Tempo` entry should be explicit rather than reconstructed from display text. At minimum it should store tick, resolved BPM, optional named-tempo label, display form, and structured metronome data sufficient to preserve `q=120` rendering without reparsing a raw string.
- The persisted v1 `Entry::Tempo` variant represents the **immediate absolute tempo** branch of the future tempo family. Its minimum stored field set should stay explicit and user-editable: `id`, `tick`, `text: Option<String>`, `beat_type: NoteDuration`, `dots: u8`, `bpm`, `hidden`, `show_text`, `show_metronome`, and `parenthesized`. This spec prefers a field-based display model over a rigid display enum so later editing can toggle visible parts on and off without losing stored tempo data.
- The tempo architecture should remain future-proof for a later split of `ImmediateTempo.Absolute` and `ImmediateTempo.Relative`, while keeping `GradualTempo` as a separate family.
- V1 named-tempo defaults are fixed to: `Largo = 50`, `Adagio = 70`, `Andante = 90`, `Moderato = 110`, `Allegro = 130`, `Presto = 180`.
- Explicit BPM input validates against the 20–400 range already used by transport tempo clamping.
- Strict parsing means exact supported forms only, consistent with existing popover parsers; unsupported casing, unsupported words, unsupported note values, malformed punctuation, and malformed spacing are rejected. The accepted grammar is intentionally exact for v1: `Allegro`, `Allegro q=120`, `Allegro (q=120)`, and `q=120`, with exactly one space between `Allegro` and `q=120`, exactly one space before `(` in the parenthesized form, and no leading/trailing whitespace normalization beyond current popover behavior.
- A tempo at tick 0 renders above the top stave aligned with the musical start after opening signatures, not in the far-left pre-signature area.
- Tempo markings appear only where the entry occurs; v1 does not automatically restate the current tempo at later system starts.
- Playback is explicitly out of scope: score tempo markings are notation/data only for now, and docs must call out that transport tempo remains independent until a follow-up playback spec is implemented.
- Invalid input follows current popover behavior in v1: reject the change, persist nothing, and rely on existing developer-facing error logging; user-facing validation is a separate follow-up improvement.
- Backend lookup helpers such as `get_tempo_at_tick()` and `get_tempo_before_tick()` should be included so future playback work can consume the same source of truth cleanly.

## Implementation Plan

### Tasks

- [x] Task 1: Add the immediate tempo backend entry and command surface
  - File: `src-tauri/src/score/entries/tempo.rs`
  - Action: Create a new immediate tempo entry module with the serialized data model, strict fields needed for text/BPM storage, metrics helpers, master-track lookup helpers, and Tauri create/clear commands that replace any existing tempo entry at the same tick.
  - Notes: Model immediate tempo independently from gradual tempo changes; store enough information to distinguish display forms (`Allegro`, `Allegro q=120`, `Allegro (q=120)`, `q=120`) while always keeping a resolved BPM. The schema should use explicit editable fields (`text`, `beat_type`, `dots`, `bpm`, `hidden`, `show_text`, `show_metronome`, `parenthesized`) rather than a derived display enum. Choose names and structure that can later accommodate the broader hierarchy `Tempo -> ImmediateTempo | GradualTempo`, with `ImmediateTempo -> Absolute | Relative`, without breaking this absolute-tempo implementation.
  - File: `src-tauri/src/score/entries/mod.rs`
  - Action: Register the new module and add `Entry::Tempo` to the tagged enum plus shared `id()`, `tick()`, and `set_tick()` handling.
  - Notes: Keep serialization aligned with the existing `Entry` pattern so the frontend tagged union stays in sync.
  - File: `src-tauri/src/lib.rs`
  - Action: Import and register the new tempo Tauri commands.
  - Notes: Follow the same snake_case command naming pattern as existing score entry commands.

- [x] Task 1a: Add reusable tempo lookup semantics for future consumers
  - File: `src-tauri/src/score/entries/tempo.rs`
  - Action: Provide master-track tempo lookup helpers such as `get_tempo_at_tick()` and `get_tempo_before_tick()` alongside create/clear behavior.
  - Notes: This avoids forcing the future playback-following-tempo work to invent a second tempo source of truth.

- [x] Task 2: Integrate tempo entries into score spacing and rendering
  - File: `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
  - Action: Reserve horizontal space for tempo markings on ticks that contain a tempo entry, either by adding a dedicated slot or by extending the existing slot model in a way that does not regress current entry spacing.
  - Notes: Use tempo metrics based on measured text width, not guessed constants. The spacing strategy must handle tick-0 interaction with opening signatures cleanly.
  - File: `src-tauri/src/instructions/draw/draw_tempo.rs`
  - Action: Add a dedicated draw helper that renders the textual tempo name in `LibreBaskerville` and the metronome note/BPM portion using the appropriate notation/text fonts.
  - Notes: The helper should support all four v1 forms and render above the top stave only. Tick-0 alignment should be to the musical start after opening signatures.
  - File: `src-tauri/src/instructions/mod.rs`
  - Action: Insert a tempo draw pass using flow master track entries and top-stave coordinates.
  - Notes: Avoid drawing the same tempo entry once per stave; it should appear once per system context and should not auto-repeat at later system starts in v1.

- [x] Task 3: Expose tempo entries and commands to the frontend type/action layer
  - File: `src/services/types.ts`
  - Action: Add the tempo entry type, any supporting enums/unions needed for display form or note-value representation, and include it in the `Entry` union.
  - Notes: Keep field names in `snake_case` to match Rust serialization.
  - File: `src/services/actions.ts`
  - Action: Add `actions.score.tempo.create()` and `actions.score.tempo.clearAtTick()` wrappers.
  - Notes: Mirror the command signatures from the Rust Tauri layer exactly.

- [x] Task 4: Add strict tempo parsing and popover dispatch rules
  - File: `src/services/utils.ts`
  - Action: Add a strict immediate tempo parser that recognizes only the supported v1 forms and derives default BPM values for approved text tempo names.
  - Notes: The parser should reject unsupported syntax, unsupported text labels, unsupported note values, malformed parentheses, malformed spacing/punctuation, out-of-range BPM values, and missing BPM values. Lock tests to the exact accepted strings `Allegro`, `Allegro q=120`, `Allegro (q=120)`, and `q=120`.
  - File: `src/services/popovers.ts`
  - Action: Add `PopoverType.Tempo` and `parseTempoPopover()` to handle `clear`, parsed create calls, and replacement semantics through the action layer.
  - Notes: Keep v1 intentionally strict; no fuzzy matching and no new user-facing validation UX in this iteration.

- [x] Task 5: Wire the tempo popover into write-mode UI
  - File: `src/ui/popover.tsx`
  - Action: Add the new popover label/icon metadata, keyboard shortcut handling, and Enter-key dispatch for tempo input.
  - Notes: Use the same selected-tick resolution path as other flow-level directives and bind the new shortcut to `Shift+T`.

- [x] Task 6: Add focused automated coverage for backend and frontend tempo behavior
  - File: `src/services/actions.test.ts`
  - Action: Add invoke-wrapper tests for tempo create/clear calls.
  - Notes: Match the existing score entry test structure.
  - File: `src/services/utils.test.ts`
  - Action: Add strict parser coverage for all supported forms, derived-BPM text forms, and rejected invalid inputs.
  - Notes: Include negative cases for unsupported tempo names, malformed metronome syntax, malformed spacing/punctuation, unsupported casing, and BPM values outside 20–400.
  - File: `src/services/popovers.test.ts`
  - Action: Add create/clear dispatch tests for the tempo popover parser.
  - Notes: Verify form-to-action mapping is exact.
  - File: `src/ui/popover.test.tsx`
  - Action: Add UI tests for tempo label rendering, `Shift+T` shortcut opening, and Enter-key dispatch.
  - Notes: Keep aligned with the existing popover test style.
  - File: `src-tauri/src/score/entries/tempo.rs`
  - Action: Add inline Rust tests for creation, replacement at the same tick, clear behavior, and any metrics/format helpers.
  - Notes: Ensure behavior matches existing master-track directive patterns.
  - File: `src-tauri/src/instructions/draw/draw_tempo.rs`
  - Action: Add draw-helper tests that verify instruction count/content for each supported form.
  - Notes: Cover mixed-font output, tick-0 placement after opening signatures, and non-repetition at later systems where practical.

- [x] Task 6a: Add renderer fixtures for tempo-marking regression coverage
  - File: `tests/fixtures/source/tempo/tempo.json`
  - Action: Add fixture flows covering tick-0 tempo, mid-flow tempo changes, all four supported forms, and representative named-tempo defaults.
  - Notes: Follow the fixture rules in `tests/fixtures/README.md`, keep time signatures simple only, and rebuild generated fixtures with `npm run build:fixtures`.
  - File: `tests/fixtures/README.md`
  - Action: Update fixture guidance if explicit tempo-entry authoring instructions are needed.
  - Notes: Keep the fixture documentation authoritative for future renderer-regression additions.

- [x] Task 7: Update project documentation and AI-agent guidance for the new entry type
  - File: `docs/data-models.md`
  - Action: Document the new `Tempo` entry type, its placement on the flow master track, and its core stored fields/forms.
  - Notes: Update any enum/union summaries that currently list only five entry variants.
  - File: `docs/architecture.md`
  - Action: Update the render-pipeline and score-data sections to mention tempo entries and the new tempo draw step.
  - Notes: Keep the architecture narrative accurate for future contributors.
  - File: `docs/project-context.md`
  - Action: Update AI-agent guidance that currently states there are only five `Entry` variants and that score data/render pipeline only cover the existing set.
  - Notes: This is required because project rules for agent implementation expectations are changing.
  - File: `README.md`
  - Action: Update user-facing feature summary language if tempo markings become a shipped notation feature.
  - Notes: If the feature is not surfaced in README feature bullets yet, document why no README update is needed before merge.

### Acceptance Criteria

- [x] AC 1: Given a selected score position in write mode, when the user presses `Shift+T`, then the tempo popover opens at the selected coordinates with a tempo-specific label.
- [x] AC 2: Given valid input `Allegro`, when the user submits the tempo popover, then a new immediate tempo entry is created on the flow master track at the selected tick with a derived BPM stored under the hood.
- [x] AC 3: Given valid input `Allegro q=120`, `Allegro (q=120)`, or `q=120`, when the user submits the tempo popover, then a new immediate tempo entry is created at the selected tick with the exact BPM value `120` stored and the display form preserved for rendering.
- [x] AC 4: Given a tick that already has an immediate tempo entry, when the user creates another tempo entry at the same tick, then the old entry is replaced and only one tempo entry remains at that tick.
- [x] AC 5: Given a tick that has an immediate tempo entry, when the user enters `clear` in the tempo popover, then the tempo entry at that tick is removed without affecting other score entries.
- [x] AC 6: Given unsupported or malformed tempo input, when the user submits the tempo popover, then no tempo entry is created, no existing tempo entry is modified, and the input is rejected using the same strict behavior pattern as existing popover parsers.
- [x] AC 7: Given an explicit BPM outside 20–400, when the user submits the tempo popover, then the input is rejected and no tempo entry is created or updated.
- [x] AC 8: Given a tempo marking at tick 0, when the score is rendered, then the marking appears above the top stave aligned with the musical start after opening clef/key/time material.
- [x] AC 9: Given a flow with an immediate tempo entry away from tick 0, when the score is rendered, then the tempo marking is drawn once above the top stave at the correct horizontal tick position and is not repeated on lower staves or later systems.
- [x] AC 10: Given tempo markings using all four supported forms, when the score is rendered, then text portions use score-text styling and metronome notation renders correctly for the supported quarter-note syntax.
- [x] AC 10: Given tempo markings using all four supported forms, when the score is rendered, then text portions use score-text styling and metronome notation renders correctly for the stored `beat_type`, `dots`, and `bpm` values.
- [x] AC 11: Given score tempo markings exist in a file, when the user reads the feature docs, then the documentation explicitly states that playback/transport tempo remains independent until a follow-up playback spec is implemented.
- [x] AC 12: Given the change is merged, when developers read the project docs and agent guidance, then the documented `Entry` variant list, flow-master-track directive guidance, render-pipeline description, and tempo-entry implementation expectations include tempo entries explicitly.
- [x] AC 13: Given the renderer-facing implementation is complete, when fixture coverage is run, then tempo-marking fixtures exist for tick 0, mid-flow changes, and all four supported forms, and `npm run build:fixtures` succeeds.
- [x] AC 14: Given the backend tempo model is implemented, when future tempo consumers need the current or prior immediate absolute tempo at a tick, then reusable helpers such as `get_tempo_at_tick()` and `get_tempo_before_tick()` are available on the master-track tempo implementation.

### Documentation Impact

- `docs/data-models.md` must be updated because the serialized score model and `Entry` variant list change from five variants to six, and tempo entry placement on the flow master track must be explicit.
- `docs/architecture.md` must be updated because the score render pipeline gains a new tempo draw concern and the data-flow description changes.
- `docs/project-context.md` must be updated because the AI-agent rules currently describe only five `Entry` variants and need to reflect the new implementation expectation.
- `README.md` should be reviewed for a feature-summary update because tempo markings are user-visible notation functionality; if left unchanged, the implementation PR should explain why.
- `tests/fixtures/README.md` should be updated if explicit tempo-entry fixture guidance is needed.
- User/developer docs must explicitly call out that immediate score tempo markings do not yet drive transport playback in this iteration.
- No persistence migration is required for this change because it adds a new serialized entry variant rather than rewriting the meaning of an existing field.
- Compatibility expectations must be documented explicitly: newer builds should continue to load older files unchanged; older builds are not expected to understand files containing the new `Tempo` entry variant.

## Additional Context

### Dependencies

- No new external libraries are required; the feature should reuse the existing React/Tauri action flow and Rust renderer text stack.
- The implementation depends on keeping Rust and TypeScript entry/type definitions synchronized.
- Playback-following-tempo is an intentional follow-up dependency, not part of this spec.
- Visual-regression confidence depends on the existing fixture build pipeline and its constraints.

### Testing Strategy

- Add strict utility/parser tests for supported tempo forms and rejected inputs.
- Add popover parser tests for create/replace/clear dispatch behavior.
- Add popover UI tests for the new label and shortcut dispatch.
- Add Rust entry tests for creation, replacement at the same tick, clear behavior, and master-track lookup helpers.
- Add Rust draw/spacing tests to verify tempo markings reserve width and emit expected text instructions.
- Add fixture coverage for renderer regression and rebuild fixtures with `npm run build:fixtures`.
- Run the existing frontend unit suite relevant to popovers/actions/utils and the Rust library tests after implementation to confirm no notation regressions.
- Manually verify a score containing tempo entries at tick 0 and mid-flow renders correctly and that the marking stays attached to the intended musical position.
- Manually verify that score tempo markings do not change transport tempo in v1 and that this limitation is documented clearly.

### Notes

- Future playback support should consume the same immediate tempo entries rather than introducing a second source of truth.
- Future immediate relative tempos such as `meno mosso` should extend the same immediate-tempo family rather than forcing this absolute-tempo work to be replaced.
- Gradual tempo changes should be modeled as a separate entry type rather than overloading immediate tempo entries with span/interpolation fields.
- The intended long-term hierarchy is: `Tempo` as the top-level family, split into `ImmediateTempo` and `GradualTempo`; `ImmediateTempo` can later split into `Absolute` and `Relative` without changing the decision that gradual tempo is a separate branch.
- The named-tempo BPM defaults in this spec are a deliberate product decision for v1 authoring, not a claim of universal musical standardization.
- The highest implementation risk is mixed-font rendering of textual tempo names plus metronome notation; if a single-string rendering approach proves too limiting, prefer a composed draw helper over weakening the stored model.
- Keep v1 parser strict and vocabulary-limited to avoid accidentally promising unsupported Italian text forms or note-value variants before the renderer and defaults table are ready.
- Deferred follow-up work includes user-facing validation messaging and playback-following-tempo behavior; neither should be quietly added to this implementation under the guise of polish.
- The field-based visibility model is intentionally permissive for editing. If a user enables `show_text` without supplying `text`, nothing textual renders; if `hidden` is true, nothing renders regardless of other flags. The implementation should preserve data rather than aggressively correcting user intent.
