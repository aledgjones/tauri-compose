---
title: "Comprehensive Test Coverage Expansion"
slug: "comprehensive-test-coverage-expansion"
created: "2026-03-04"
status: "implementation-complete"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  [
    "vitest",
    "happy-dom",
    "@testing-library/react",
    "@testing-library/user-event",
    "@vitest/coverage-v8",
  ]
files_to_modify:
  # Phase 1 — Test Infrastructure & Utilities
  - "src/services/test-utils.tsx" # NEW — shared context wrapper + fixture factories
  # Phase 2 — FE Service Layer Tests
  - "src/services/popovers.test.ts" # NEW — popover parser tests
  - "src/services/hit-test.test.ts" # NEW — IPC wrapper test
  # Phase 3 — UI Primitive Tests (Buttons, Inputs, Text)
  - "src/ui/buttons/button.test.tsx" # NEW
  - "src/ui/buttons/icon-button.test.tsx" # NEW
  - "src/ui/buttons/bravura-icon-button.test.tsx" # NEW
  - "src/ui/buttons/shared.test.ts" # NEW
  - "src/ui/input/text-input.test.tsx" # NEW
  - "src/ui/input/number-input.test.tsx" # NEW
  - "src/ui/input/index.test.tsx" # NEW
  - "src/ui/textarea.test.tsx" # NEW
  - "src/ui/menu.test.tsx" # NEW
  - "src/ui/popover.test.tsx" # NEW
  - "src/ui/text/index.test.tsx" # NEW
  # Phase 4 — Shared Component Tests
  - "src/components/app-bar.test.tsx" # NEW
  - "src/components/select-by-image.test.tsx" # NEW
  - "src/components/not-found.test.tsx" # NEW
  - "src/components/drag-scroll.test.tsx" # NEW
  # Phase 5 — Window Hooks & Utilities
  - "src/windows/main/utils/repeat-array.test.ts" # NEW
  - "src/windows/main/hooks/use-ticks.test.ts" # NEW
  - "src/windows/main/hooks/use-slots.test.ts" # NEW
  - "src/windows/main/hooks/use-bottom-panel-state.test.ts" # NEW
  - "src/windows/main/hooks/use-selected-tone-info.test.ts" # NEW
  - "src/windows/main/hooks/use-keyboard-shortcuts.test.ts" # NEW
  # Phase 6 — Context & State Provider Tests
  - "src/services/state/context.test.tsx" # NEW
  # Phase 7 — Window Component Tests (using context wrapper)
  - "src/windows/main/components/transport.test.tsx" # NEW
  - "src/windows/main/components/zoom-widget.test.tsx" # NEW
  - "src/windows/main/components/fader.test.tsx" # NEW
  - "src/windows/main/components/keyboard.test.tsx" # NEW
  - "src/windows/main/components/tools.test.tsx" # NEW
  - "src/windows/main/components/selected-tone-info-display.test.tsx" # NEW
  # Phase 8 — Rust Score Model Tests
  - "src-tauri/src/score/components/pitch.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/components/articulation.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/components/measurements.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/components/velocity.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/tracks.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/entries/barline.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/entries/clef.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/entries/key_signature.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/entries/time_signature.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/entries/tone.rs" # add #[cfg(test)] module
  # Phase 9 — Rust Parse Pipeline Tests
  - "src-tauri/src/instructions/parse/get_barlines.rs" # add #[cfg(test)] module
  - "src-tauri/src/instructions/parse/get_key_signatures.rs" # add #[cfg(test)] module
  - "src-tauri/src/instructions/parse/get_shunts.rs" # add #[cfg(test)] module
  - "src-tauri/src/instructions/parse/get_accidentals.rs" # add #[cfg(test)] module
  - "src-tauri/src/instructions/parse/get_stem_directions.rs" # add #[cfg(test)] module
  - "src-tauri/src/instructions/parse/get_written_durations.rs" # add #[cfg(test)] module
  - "src-tauri/src/instructions/parse/get_vertical_spacing.rs" # add #[cfg(test)] module
  - "src-tauri/src/instructions/parse/get_tone_offsets.rs" # add #[cfg(test)] module
  # Phase 10 — Coverage Threshold Ratchet
  - "vite.config.ts" # bump coverage thresholds
  - "TESTING.md" # update with new patterns
code_patterns:
  - "FC<Props> functional components only"
  - "snake_case type properties crossing FE↔BE boundary"
  - "FxHashMap everywhere in Rust (never std HashMap)"
  - "invoke() calls via actions object in actions.ts"
  - "Numeric enums mirrored between TS and Rust via serde_repr"
  - "kebab-case TS/TSX filenames, snake_case Rust filenames"
  - "One component per file"
  - "Test files colocated next to source"
  - "4 React contexts: AppContext, AudioContext, SamplerContext, TransportContext"
test_patterns:
  - "Vitest + happy-dom for TS/TSX"
  - "#[cfg(test)] modules for Rust"
  - "Test files colocated next to source, no __tests__/ subdirectories"
  - "@testing-library/react for component tests"
  - "Manual mocks in src/__mocks__/ for Tauri API"
  - "Shared renderWithProviders helper for context-dependent components"
  - "vi.mock for actions module in popover/hook tests"
---

# Tech-Spec: Comprehensive Test Coverage Expansion

**Created:** 2026-03-04

## Overview

### Problem Statement

The testing foundation (Phases 1-3 of the previous spec) established Vitest, React Testing Library, and Rust `#[cfg(test)]` patterns, but overall coverage sits at ~15% statements / ~12% branches on the FE and only 4 Rust test modules. With increasing AI-automated development, the codebase needs comprehensive test coverage to serve as a safety net — preventing regressions when agents modify code. The target is pragmatic 80% coverage on both FE and Rust sides, testing real behavior rather than padding numbers.

### Solution

Systematically expand test coverage across 10 phases:

1. Build shared test infrastructure (context wrappers, fixture factories)
2. Test the service layer (popover parsers, IPC wrappers)
3. Test all remaining UI primitives (buttons, inputs, text, menu)
4. Test shared components (app-bar, select-by-image, drag-scroll)
5. Test window-level hooks and utilities
6. Test context/state layer
7. Test window components using the context wrapper
8. Expand Rust score model tests (pitch, articulation, tracks, all entry types)
9. Expand Rust parse pipeline tests (barlines, key signatures, shunts, accidentals, stems, written durations, vertical spacing, tone offsets)
10. Ratchet coverage thresholds to match actual coverage

### Scope

**In Scope:**

- All UI primitives in `src/ui/` not yet tested (buttons, inputs, textarea, menu, popover, text)
- All shared components in `src/components/` not yet tested
- All popover parser functions in `src/services/popovers.ts`
- All window hooks in `src/windows/main/hooks/`
- Utility functions (`repeat-array.ts`)
- Context hooks (throw-outside-provider behavior)
- Window components that can be tested with a context wrapper
- All Rust score model pure methods (pitch, articulation, measurements, velocity, tracks, all entry subtypes)
- All Rust parse pipeline public functions
- Coverage threshold ratchet

**Out of Scope:**

- `actions.ts` — 887 lines of thin `invoke()` wrappers with no logic; testing would just verify mock calls with zero behavioral value
- `provider.tsx` — event listener wiring + RAF polling; would require mocking Tauri event system + timers with high complexity / low value
- `renderer.tsx` — transparent overlay for Skia surface; DOM-only, all meaningful logic lives in Rust
- `sequencer-track.tsx` (653 lines) — deeply coupled to canvas-like interactions + multiple contexts; defer to integration/E2E
- Window `boot.tsx` files — one-line React mount calls
- Window `index.tsx` screen routers — primarily layout composition
- Rust Tauri command functions (take `State<Mutex<AppContext>>` + emit) — require constructing full app context; pure data model methods provide the same safety
- Rust `lib.rs`, `main.rs` — app bootstrap, not testable in isolation
- Rust renderer backends (`metal.rs`) — platform-specific GPU code
- Rust playback modules — audio I/O, real-time constraints
- CSS/Tailwind visual regression

## Context for Development

### Codebase Patterns

- Current coverage: FE ~15% statements, ~12% branches. Rust: 4 `#[cfg(test)]` modules.
- 9 existing test files covering: `pitch-utils`, `utils`, `sequencer-utils`, `hooks`, `checkbox`, `switch`, `tabs`, `select`, `snap-picker`
- All FE→BE calls go through `actions.ts` (888 lines) via `invoke()` — single mock point for IPC
- 4 React contexts (`AppContext`, `AudioContext`, `SamplerContext`, `TransportContext`) — components consuming these need a provider wrapper in tests
- `initialAppState` in `context.tsx` provides a complete fixture shape for testing
- Popover parsers call `actions.score.*` methods — test by mocking the `actions` module
- Window hooks depend on `useAppState()` — test via context wrapper or by mocking the state module
- Rust: ~18,800 lines across 99 files. Parse pipeline (19 modules) has pure public functions that are excellent test targets
- `Pitch` struct has 10+ pure methods (base, steps, half_steps, octave, steps_between, shift_by_steps, shift_by_octaves, from_int)
- `Articulations.toggle()` enforces exclusivity groups — critical behavioral test
- `Track.insert_entry/shift_entry/remove_entry` — pure data structure operations

### Files to Reference

| File                                                        | Purpose                                                                                |
| ----------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `src/services/state/context.tsx`                            | `initialAppState` fixture data + context hooks (220 lines)                             |
| `src/services/popovers.ts`                                  | 4 parser functions: barline, clef, key signature, time signature (627 lines)           |
| `src/services/actions.ts`                                   | All `invoke()` calls (887 lines) — mock target for popover tests                       |
| `src/services/hit-test.ts`                                  | IPC wrapper (18 lines) — uses invoke mock                                              |
| `src/ui/buttons/button.tsx`                                 | Simple button (30 lines)                                                               |
| `src/ui/buttons/icon-button.tsx`                            | Icon button with variant theming (40 lines)                                            |
| `src/ui/buttons/bravura-icon-button.tsx`                    | Music notation icon button (51 lines)                                                  |
| `src/ui/buttons/shared.ts`                                  | `getTheme()` function + `Variant` type (26 lines)                                      |
| `src/ui/buttons/smufl.ts`                                   | SMuFL glyph enum (30 lines) — reference only, no logic to test                         |
| `src/ui/input/text-input.tsx`                               | Text input with label, focus state (88 lines)                                          |
| `src/ui/input/number-input.tsx`                             | Number input with Big.js precision + arrow keys (124 lines)                            |
| `src/ui/input/index.tsx`                                    | Discriminated union input component (45 lines)                                         |
| `src/ui/textarea.tsx`                                       | Auto-resize textarea (50 lines)                                                        |
| `src/ui/menu.tsx`                                           | Menu with nested sections (67 lines)                                                   |
| `src/ui/popover.tsx`                                        | Score element input popover (265 lines)                                                |
| `src/ui/text/index.tsx`                                     | `TokenizedText` — symbol tokenizer (42 lines)                                          |
| `src/ui/text/provider.tsx`                                  | `TokenizationProvider` — needs AppContext (61 lines)                                   |
| `src/components/app-bar.tsx`                                | Title bar with Tauri window title sync (31 lines)                                      |
| `src/components/drag-scroll.tsx`                            | Drag-to-scroll container (118 lines)                                                   |
| `src/components/select-by-image.tsx`                        | Image-based selector (49 lines)                                                        |
| `src/components/not-found.tsx`                              | Static placeholder (11 lines)                                                          |
| `src/windows/main/utils/repeat-array.ts`                    | Array repetition utility (7 lines)                                                     |
| `src/windows/main/hooks/use-ticks.ts`                       | Tick/measure calculation from time signatures (67 lines)                               |
| `src/windows/main/hooks/use-slots.ts`                       | Piano key pattern generator (59 lines)                                                 |
| `src/windows/main/hooks/use-bottom-panel-state.ts`          | Sequencer panel state derivation (133 lines)                                           |
| `src/windows/main/hooks/use-selected-tone-info.ts`          | Selected tone info derivation (95 lines)                                               |
| `src/windows/main/hooks/use-keyboard-shortcuts.ts`          | Keyboard shortcut dispatch (286 lines)                                                 |
| `src-tauri/src/score/components/pitch.rs`                   | `Pitch` + `Accidental` + `ACCIDENTAL_MAP` (718 lines)                                  |
| `src-tauri/src/score/components/articulation.rs`            | `Articulation` enum + `Articulations` toggle (120 lines)                               |
| `src-tauri/src/score/components/measurements.rs`            | `BoundingBox`, `PaddingMm`, `PaddingSpaces`, `Point` (90 lines)                        |
| `src-tauri/src/score/components/velocity.rs`                | `Velocity` wrapper (17 lines)                                                          |
| `src-tauri/src/score/tracks.rs`                             | `Track` insert/shift/remove operations (76 lines)                                      |
| `src-tauri/src/score/entries/barline.rs`                    | `Barline`, `BarlineDrawType.metrics()`, `Track.get_barline_at_tick()` (163 lines)      |
| `src-tauri/src/score/entries/clef.rs`                       | `Clef.metrics()`, `Clef.glyph()`, `Track.get_clef_at_tick()` (176 lines)               |
| `src-tauri/src/score/entries/key_signature.rs`              | `KeySignature.accidentals()`, `KeySignature.metrics()` (274 lines)                     |
| `src-tauri/src/score/entries/time_signature.rs`             | `TimeSignature` default + methods (337 lines)                                          |
| `src-tauri/src/score/entries/tone.rs`                       | `Tone`, `Track.get_tones_at_tick()` (490 lines)                                        |
| `src-tauri/src/instructions/parse/get_barlines.rs`          | `get_barlines()` + `BarlineDrawDef` (122 lines)                                        |
| `src-tauri/src/instructions/parse/get_key_signatures.rs`    | `get_key_signatures()` (45 lines)                                                      |
| `src-tauri/src/instructions/parse/get_shunts.rs`            | `note_shunts_in_chord()` + `get_note_shunts()` (125 lines)                             |
| `src-tauri/src/instructions/parse/get_accidentals.rs`       | `fits_in_slot()`, `sort_accidentals_alternate()`, `is_accidental_needed()` (272 lines) |
| `src-tauri/src/instructions/parse/get_stem_directions.rs`   | `get_span_stem_direction()` (109 lines)                                                |
| `src-tauri/src/instructions/parse/get_written_durations.rs` | `Notation` methods (920 lines)                                                         |
| `src-tauri/src/instructions/parse/get_vertical_spacing.rs`  | `get_vertical_spacing()` (75 lines)                                                    |

### Technical Decisions

- **Shared `renderWithProviders` helper:** A `test-utils.tsx` file provides a `renderWithProviders(ui, options?)` function that wraps components in `AppContext.Provider` (and optionally `AudioContext`, `SamplerContext`, `TransportContext`). Uses `initialAppState` as default, with overrides via deep merge. This eliminates per-test boilerplate for context-dependent components.
- **`vi.mock` for `actions` in popover tests:** Instead of mocking individual `invoke` calls, mock the entire `../actions` module so popover parsers can assert the correct `actions.score.*` method was called with the right arguments. This tests the parsing/dispatching logic without IPC.
- **`vi.mock` for `@tauri-apps/api/window`:** Components like `AppBar` call `getCurrentWindow()` — mock this to return a stub with `setTitle: vi.fn()`.
- **Pure Rust methods first, command functions out of scope:** Rust Tauri commands require `State<Mutex<AppContext>>` + `app: tauri::AppHandle` which can't be constructed in unit tests without spinning up a full Tauri runtime. Pure data model methods and parse pipeline functions give the same regression safety without the ceremony.
- **Notation struct methods tested in isolation:** `Notation` has ~15 pure methods (`is_rest`, `is_dotted`, `base_duration`, `glyph`, `rest_glyph`, etc.) that can be tested by constructing `Notation` instances directly.
- **Coverage thresholds ratcheted in final phase:** After all tests land, run coverage, and bump thresholds to match actual numbers. This prevents regression.

## Implementation Plan

### Phase 1: Test Infrastructure & Utilities

#### Story 1.1: Create Shared Test Utilities

- [x] Task 1.1.1: Create `renderWithProviders` helper
  - File: `src/services/test-utils.tsx` — **NEW**
  - Action: Create a test utility file that exports:
    - `renderWithProviders(ui, options?)` — wraps `render()` from `@testing-library/react` with all 4 context providers. Accepts optional `appState`, `audioState`, `samplerState`, `transportState` overrides that are shallow-merged with the defaults from `context.tsx`.
    - Re-export `screen`, `within`, `waitFor` from `@testing-library/react` and `userEvent` from `@testing-library/user-event` for convenience.
    - Export a `createAppState(overrides?)` factory that deep-merges overrides into `initialAppState`.
  - Notes: Use `structuredClone(initialAppState)` to avoid cross-test mutation. The function signature:
    ```tsx
    type RenderOptions = {
      appState?: Partial<AppState>;
      audioState?: Partial<AudioState>;
      samplerState?: Partial<SamplerState>;
      transportState?: Partial<TransportState>;
    };
    export function renderWithProviders(
      ui: React.ReactElement,
      options?: RenderOptions,
    );
    ```
    Deep merge should handle nested objects (e.g. `{ ui: { sequencer: { tool: Tool.Draw } } }` should merge into the full state without clobbering other `ui` fields). Use a simple recursive merge or spread — no lodash dependency needed. Import initial states from `./state/context`.

### Phase 2: FE Service Layer Tests

#### Story 2.1: Popover Parser Tests

- [x] Task 2.1.1: Test `parseBarlinePopover`
  - File: `src/services/popovers.test.ts` — **NEW**
  - Action: Mock `./actions` module with `vi.mock`. Write tests for:
    - `"clear"` → calls `actions.score.barline.clearAtTick(flow_id, tick)`
    - `"|"`, `"single"`, `"normal"` → calls `actions.score.barline.create(flow_id, tick, BarlineDrawType.Single)`
    - `"||"`, `"double"` → `BarlineDrawType.Double`
    - `"|||"`, `"triple"` → `BarlineDrawType.Triple`
    - `":"`, `"dashed"` → `BarlineDrawType.Dashed`
    - `"|:"`, `"start"` → `BarlineDrawType.StartRepeat`
    - `":|"`, `"end"` → `BarlineDrawType.EndRepeat`
    - `":|:"`, `":||:"`, `"end-start"`, `"endstart"` → `BarlineDrawType.EndStartRepeat`
    - Unknown input → throws `Error`
  - Notes: The mock should make `actions.score.barline.create` and `actions.score.barline.clearAtTick` be `vi.fn()` stubs. Reset mocks in `beforeEach`. Use fixed `flow_id = 1` and `tick = 128` for all tests.

- [x] Task 2.1.2: Test `parseClefPopover`
  - File: `src/services/popovers.test.ts` (append)
  - Action: Write tests for:
    - `"clear"` → calls `actions.score.clef.clear(flow_id, tick, stave_id)`
    - `"treble"`, `"g"`, `"G"`, `"sol"` → calls `actions.score.clef.create(flow_id, tick, stave_id, 67, 2, ClefDrawType.G)`
    - `"bass"`, `"f"`, `"F"`, `"fa"` → `(53, -2, ClefDrawType.F)`
    - `"alto"`, `"ca"`, `"CA"` → `(60, 0, ClefDrawType.C)`
    - `"tenor"`, `"ct"`, `"CT"` → `(60, -2, ClefDrawType.C)`
    - Unknown input → throws `Error`
  - Notes: Use fixed `stave_id = 10` for all clef tests.

- [x] Task 2.1.3: Test `parseKeySignaturePopover`
  - File: `src/services/popovers.test.ts` (append)
  - Action: Write tests for:
    - `"clear"` → calls `actions.score.keySignature.clearAtTick(flow_id, tick)`
    - Major sharps: `"C#"` / `"7s"` / `"7#"` → `(KeySignatureMode.Major, 7)`, down to `"G"` / `"1s"` / `"1#"` → `(Major, 1)`
    - `"C"` / `"open"` / `"atonal"` / `"0s"` / `"0f"` / `"0#"` / `"0b"` → `(Major, 0)`
    - Major flats: `"F"` / `"1f"` / `"1b"` → `(Major, -1)`, down to `"Cb"` / `"7f"` / `"7b"` → `(Major, -7)`
    - Minor keys: `"a#"` → `(Minor, 7)`, `"a"` → `(Minor, 0)`, `"ab"` → `(Minor, -7)`
    - Unknown input → throws `Error`
  - Notes: Test at least one input per key signature offset. For brevity, don't test every alias — test the primary plus one alias per group.

- [x] Task 2.1.4: Test `parseTimeSignaturePopover`
  - File: `src/services/popovers.test.ts` (append)
  - Action: Write tests for:
    - `"4/4"` → calls `actions.score.timeSignature.create(flow_id, tick, 4, NoteDuration.Quarter, TimeSignatureDrawType.Regular)`
    - `"6/8"` → `(6, NoteDuration.Eighth, Regular)`
    - `"3/4"` → `(3, NoteDuration.Quarter, Regular)`
    - `"2/2"` → `(2, NoteDuration.Half, Regular)`
    - `"clear"` → calls `actions.score.timeSignature.clearAtTick(flow_id, tick)`
    - `"c"` → `(4, NoteDuration.Quarter, CommonTime)`
    - `"cutc"` → `(2, NoteDuration.Half, SplitCommonTime)`
    - Unknown input → throws `Error`
  - Notes: The function calls `parseTimeSignature()` first (already tested in `utils.test.ts`), then dispatches based on beat type. This tests the dispatch logic.

#### Story 2.2: IPC Wrapper Test

- [x] Task 2.2.1: Test `hitTest` function
  - File: `src/services/hit-test.test.ts` — **NEW**
  - Action: Write tests:
    - Calls `invoke("renderer_hit_test", { x, y, pan_x, pan_y, zoom })` with correct args
    - Returns the resolved value from invoke
  - Notes: The `invoke` mock from `src/__mocks__/@tauri-apps/api/core.ts` is already set up. Use `invoke.mockResolvedValueOnce(...)` to set return values.

### Phase 3: UI Primitive Tests (Buttons, Inputs, Text)

#### Story 3.1: Button Component Tests

- [x] Task 3.1.1: Test `getTheme` function
  - File: `src/ui/buttons/shared.test.ts` — **NEW**
  - Action: Write tests for:
    - Each `Variant` ("standard", "interactive", "filled", "filled-dark", "outline") returns correct class string
    - `isSelected = true` returns `"bg-sky-500 text-black"` regardless of variant
    - `isSelected = false` returns variant-specific classes
  - Notes: Pure function, no React needed.

- [x] Task 3.1.2: Test `Button` component
  - File: `src/ui/buttons/button.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders children text
    - Clicking fires `onClick`
    - `isDisabled` applies `opacity-25` class
    - Custom `className` is applied
  - Notes: Simple component, no context dependency.

- [x] Task 3.1.3: Test `IconButton` component
  - File: `src/ui/buttons/icon-button.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders without crashing
    - Clicking fires `onClick`
    - `isDisabled` applies `pointer-events-none`
    - `isSelected` applies selected theme
    - Different variants apply correct theme classes
  - Notes: Uses `@mdi/react` `MdiIcon` — it renders an SVG. Don't assert on SVG internals.

- [x] Task 3.1.4: Test `BravuraIconButton` component
  - File: `src/ui/buttons/bravura-icon-button.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders without crashing with a `SMuFL` glyph
    - Clicking fires `onClick`
    - `title` prop sets the button's `title` attribute
    - `isDisabled` applies `pointer-events-none`
    - `isSelected` applies selected theme
  - Notes: Uses `font-bravura` for rendering — the glyph is just a Unicode character in a span.

#### Story 3.2: Input Component Tests

- [x] Task 3.2.1: Test `TextInput` component
  - File: `src/ui/input/text-input.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders with initial value
    - Typing fires `onChange` with new value
    - Label renders when provided
    - Focus state changes label color (via class)
    - `readOnly` prop makes input read-only
    - `onEnter` fires on Enter key
    - `onBlur` fires on blur
    - `units` suffix text renders
    - `variant` prop switches between "default" and "dark" styling
  - Notes: Use `userEvent.type()` for input simulation. Use `userEvent.tab()` or `fireEvent.blur()` for blur.

- [x] Task 3.2.2: Test `NumberInput` component
  - File: `src/ui/input/number-input.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders value with correct decimal places
    - ArrowUp key increments by `step`
    - ArrowDown key decrements by `step`
    - Blur with valid input calls `onChange` with parsed number
    - Blur with invalid input reverts to original value
    - `readOnly` prop prevents editing
  - Notes: Uses `Big.js` for precision — test that `decimalPlaces` is respected (e.g. `step=0.1, decimalPlaces=2` shows `"1.00"`).

- [x] Task 3.2.3: Test `Input` discriminated union component
  - File: `src/ui/input/index.test.tsx` — **NEW**
  - Action: Write tests:
    - `type="text"` renders `TextInput`
    - `type="number"` renders `NumberInput`
  - Notes: Thin routing component — test just the delegation.

#### Story 3.3: Other UI Primitives

- [x] Task 3.3.1: Test `Textarea` component
  - File: `src/ui/textarea.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders with value and label
    - Input fires `onChange`
    - Focus state changes label color
    - `readOnly` prop sets `contentEditable` correctly
  - Notes: Has auto-resize logic via `scrollHeight` — happy-dom may not fully support this, test the callback behavior rather than visual height.

- [x] Task 3.3.2: Test `Menu` component
  - File: `src/ui/menu.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders all menu items
    - Selected item has active styling (`bg-neutral-800 border-sky-500`)
    - Clicking a menu item fires `onSelect` with correct value
    - Sections render when a selected item has them
  - Notes: Generic component `<T extends Key>` — test with concrete types.

- [x] Task 3.3.3: Test `TokenizedText` component
  - File: `src/ui/text/index.test.tsx` — **NEW**
  - Action: Write tests:
    - Plain text renders unchanged
    - Text with `${sharp}` token renders with `font-bravura` span
    - Multiple tokens in one string render correctly
    - Unknown tokens render as-is (not stripped)
  - Notes: Requires `TokenizationContext` provider. Create a minimal test provider with a known symbol map using `TokenizationContext.Provider` directly (don't use `TokenizationProvider` which needs `AppContext`).

#### Story 3.4: Popover Component Test

- [x] Task 3.4.1: Test `Popover` component
  - File: `src/ui/popover.test.tsx` — **NEW**
  - Action: Write tests using `renderWithProviders`:
    - When `popover` is null, nothing renders
    - When `popover` is set, input appears at correct position
    - Typing updates the input value
    - Pressing Escape closes the popover and fires `setPopover(null)`
    - Pressing Enter with valid barline input calls `parseBarlinePopover`
    - Pressing Enter with invalid input logs error, doesn't close
    - Blur closes the popover
    - Correct icon and label show for each `PopoverType`
  - Notes: Mock `../services/actions` and `../services/popovers` modules. The component uses `useAppState()` so it needs the context wrapper. Build a minimal app state with a selection to provide `tick` and `flow_id`.

### Phase 4: Shared Component Tests

#### Story 4.1: Remaining Shared Components

- [x] Task 4.1.1: Test `NotFound` component
  - File: `src/components/not-found.test.tsx` — **NEW**
  - Action: Write test:
    - Renders "Nothing here, yet..." text
  - Notes: Trivial static component. One test for smoke.

- [x] Task 4.1.2: Test `SelectByImage` component
  - File: `src/components/select-by-image.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders all options as buttons with images
    - Selected option has active styling
    - Clicking an option fires `onChange` with correct value
    - Label text renders when provided
  - Notes: Generic component — test with string values.

- [x] Task 4.1.3: Test `AppBar` component
  - File: `src/components/app-bar.test.tsx` — **NEW**
  - Action: Write tests using `renderWithProviders`:
    - Renders drag region div
    - Calls `getCurrentWindow().setTitle()` with formatted title
    - Title updates when `current_file_path` changes
  - Notes: Mock `@tauri-apps/api/window` — `getCurrentWindow` returns `{ setTitle: vi.fn() }`.

- [x] Task 4.1.4: Test `DragScroll` component
  - File: `src/components/drag-scroll.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders children
    - `onScroll` callback fires when scrolled
    - Initial `x`/`y` props set scroll position on mount
  - Notes: The pointer drag behavior requires `pointermove`/`pointerup` simulation which is fragile in happy-dom. Test the scroll callback and rendering. Skip drag simulation as marginal value.

### Phase 5: Window Hooks & Utilities

#### Story 5.1: Pure Utility Tests

- [x] Task 5.1.1: Test `repeatArray` utility
  - File: `src/windows/main/utils/repeat-array.test.ts` — **NEW**
  - Action: Write tests:
    - `repeatArray([1, 2], 3)` → `[1, 2, 1, 2, 1, 2]`
    - `repeatArray([], 5)` → `[]`
    - `repeatArray([1], 0)` → `[]`
    - `repeatArray(["a", "b"], 1)` → `["a", "b"]`
  - Notes: Pure function, no dependencies.

#### Story 5.2: Hook Tests

- [x] Task 5.2.1: Test `useTicks` hook
  - File: `src/windows/main/hooks/use-ticks.test.ts` — **NEW**
  - Action: Write tests using `renderHook`:
    - Given `null` flow and master → returns `{ tickWidth: 0, entries: [], length: 0 }`
    - Given a flow with length 512 and a 4/4 time signature at tick 0 → returns bar ticks at positions 0, 128, 256, 384 and beat ticks between them
    - Given `zoom = 2` → tick width doubles
    - Sub-beats render for time signatures with beat types < Eighth
  - Notes: Construct inline fixture objects matching the `Flow` and `Track` types. The hook is pure `useMemo` — no context dependency, just data in → data out.

- [x] Task 5.2.2: Test `useSlotPattern` hook
  - File: `src/windows/main/hooks/use-slots.test.ts` — **NEW**
  - Action: Write tests using `renderHook`:
    - `useSlotPattern(1)` returns pattern with 12 slots (one octave)
    - `useSlotPattern(2)` returns 24 slots (two octaves)
    - `zoomY = 2` doubles all heights
    - Pattern contains correct mix of Black and White key types (7 white, 5 black per octave)
    - `height` equals `slotHeight * pattern.length`
  - Notes: Also export and test `SLOT_PATTERN` directly as a pure function.

- [x] Task 5.2.3: Test `useBottomPanelState` hook
  - File: `src/windows/main/hooks/use-bottom-panel-state.test.ts` — **NEW**
  - Action: Write tests using `renderHook` + context wrapper:
    - Returns `instrumentOptions` derived from players → instruments
    - Returns `flowId` from state or first flow as fallback
    - Returns `instrumentId` from state or first instrument as fallback
    - Returns `trackId` from state or first track as fallback
    - Returns correct `tool` and `snapValue` from sequencer state
  - Notes: Must wrap in `AppContext.Provider` with a state fixture that has players, instruments, and flows populated.

- [x] Task 5.2.4: Test `useSelectedToneInfo` hook
  - File: `src/windows/main/hooks/use-selected-tone-info.test.ts` — **NEW**
  - Action: Write tests using `renderHook` + context wrapper:
    - Returns `null` when no selection
    - Returns `null` when selection references missing track
    - Returns `{ noteName: "Rest", instrumentName, trackLabel }` for Rest selection
    - Returns correct `noteName` (via `getPitchName`) for Tone selection
    - Returns correct `instrumentName` and `trackLabel`
  - Notes: Build fixture with players/instruments/flows/tracks populated. The hook uses `getPitchName` which is already tested — just verify it's called correctly.

- [x] Task 5.2.5: Test `useKeyboardShortcuts` hook
  - File: `src/windows/main/hooks/use-keyboard-shortcuts.test.ts` — **NEW**
  - Action: Write tests using `renderHook` + context wrapper:
    - With no selection, no shortcuts are registered (no errors)
    - With a Tone selection, ArrowUp+Option dispatches `actions.score.tone.shiftPitchByStep(..., 1)`
    - ArrowDown+Option dispatches shift by step -1
    - ArrowUp+Cmd+Option dispatches shift by octave +1
    - Backspace dispatches `actions.score.tone.remove`
    - Escape dispatches `actions.ui.sequencer.clearSelection`
    - `]` dispatches `setArticulation(Staccato)`
    - Unknown key doesn't fire any action
  - Notes: Mock `../../../services/actions` module. Dispatch keyboard events via `window.dispatchEvent(new KeyboardEvent('keydown', { key, metaKey, altKey }))`. Build a fixture state with a valid Tone selection.

### Phase 6: Context & State Tests

#### Story 6.1: Context Hook Tests

- [x] Task 6.1.1: Test context hooks throw outside provider
  - File: `src/services/state/context.test.tsx` — **NEW**
  - Action: Write tests:
    - `useAppState()` throws `"useAppState must be used within an AppStateProvider"` when rendered outside `AppContext.Provider`
    - `useAudioState()` throws correct error outside `AudioContext.Provider`
    - `useSamplerState()` throws correct error outside `SamplerContext.Provider`
    - `useTransportState()` throws correct error outside `TransportContext.Provider`
    - `useAppState()` returns state when rendered inside `AppContext.Provider`
  - Notes: Use `renderHook` with `expect(() => ...).toThrow(...)`. Suppress console.error during the throw tests with `vi.spyOn(console, 'error').mockImplementation(() => {})`.

### Phase 7: Window Component Tests

#### Story 7.1: Window Components (Context-dependent)

- [x] Task 7.1.1: Test `Transport` component
  - File: `src/windows/main/components/transport.test.tsx` — **NEW**
  - Action: Write tests using `renderWithProviders`:
    - Renders play/pause/stop buttons
    - Renders tempo display
    - Renders current position display
    - Clicking play calls `actions.transport.play()`
    - Clicking stop calls `actions.transport.stop()`
  - Notes: Mock `../../../services/actions`. The component uses `useAppState` + `useTransportState` — provide both via `renderWithProviders`.

- [x] Task 7.1.2: Test `ZoomWidget` component
  - File: `src/windows/main/components/zoom-widget.test.tsx` — **NEW**
  - Action: Write tests using `renderWithProviders`:
    - Renders current zoom percentage
    - Zoom in button calls `actions.ui.renderer.setZoom` with increased value
    - Zoom out button calls correct zoom decrease
    - Reset button resets to 100%
  - Notes: Mock actions module.

- [x] Task 7.1.3: Test `Fader` component
  - File: `src/windows/main/components/fader.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders fader control
    - Drag interaction calls `onChange` with new value
  - Notes: Fader is a slider-like UI. Test the rendering and callback, not the visual position.

- [x] Task 7.1.4: Test `Keyboard` component
  - File: `src/windows/main/components/keyboard.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders keyboard keys
    - Contains both black and white keys
  - Notes: This component renders piano keys for the sequencer. Test that the key pattern renders correctly.

- [x] Task 7.1.5: Test `Tools` component
  - File: `src/windows/main/components/tools.test.tsx` — **NEW**
  - Action: Write tests using `renderWithProviders`:
    - Renders Select and Draw tool buttons
    - Current tool is visually selected
    - Clicking tool buttons calls the correct action
  - Notes: Mock actions module.

- [x] Task 7.1.6: Test `SelectedToneInfoDisplay` component
  - File: `src/windows/main/components/selected-tone-info-display.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders note name, instrument name, and track label when given info
    - Renders nothing or placeholder when no tone is selected
  - Notes: This is likely a thin display component that receives props. Check its actual interface to determine if it uses context directly.

### Phase 8: Rust Score Model Tests

#### Story 8.1: Component Tests

- [x] Task 8.1.1: Add `#[cfg(test)]` module to `pitch.rs`
  - File: `src-tauri/src/score/components/pitch.rs`
  - Action: Append `#[cfg(test)] mod tests {}` block with tests for:
    - `Accidental::to_glyph()` — all 5 variants return correct Unicode glyph
    - `Pitch::new(60, Natural)` — basic construction
    - `Pitch::from_int(60, key_sig_c_major)` → `Pitch { int: 60, accidental: Natural }`
    - `Pitch::from_int(61, key_sig_c_major)` → `Pitch { int: 61, accidental: Sharp }` (C#)
    - `Pitch::from_int(61, key_sig_2_flats)` → `Pitch { int: 61, accidental: Flat }` (Db)
    - `Pitch::base()` — Sharp(61) → 60, Flat(61) → 62, Natural(60) → 60, DoubleSharp(62) → 60, DoubleFlat(60) → 62
    - `Pitch::half_steps()` — C=0, D=2, E=4, F=5, G=7
    - `Pitch::steps()` — C=0, D=1, E=2, F=3, G=4, A=5, B=6
    - `Pitch::octave()` — midi 60 (C4) → octave 4, midi 72 → octave 5
    - `Pitch::steps_between(C4, E4)` → 2, `(C4, C5)` → 7
    - `Pitch::shift_by_octaves(C4, 1)` → C5 (midi 72), `(-1)` → C3 (midi 48)
    - `Pitch::shift_by_steps(C4, 1, C_major)` → D4, `(E4, 1, C_major)` → F4 (half step E→F)
    - `ACCIDENTAL_MAP` entry count — verify expected number of key signature mappings exist
  - Notes: Construct `KeySignature` fixtures for C Major (offset 0), 2 flats, etc. This is the most important Rust test file — `Pitch` is used everywhere in the score model.

- [x] Task 8.1.2: Add `#[cfg(test)]` module to `articulation.rs`
  - File: `src-tauri/src/score/components/articulation.rs`
  - Action: Append tests for:
    - `Articulations::new()` is empty
    - `toggle(Staccato)` on empty → contains Staccato
    - `toggle(Staccato)` when already contains Staccato → removes it
    - `toggle(Staccatissimo)` when contains Staccato → removes Staccato, adds Staccatissimo (same group)
    - `toggle(Accent)` when contains Staccato → both present (different groups)
    - `exclusivity_group` — Staccato returns STACCATO_GROUP, Accent returns ACCENT_GROUP
    - `len()`, `is_empty()`, `contains()` behave correctly
  - Notes: The exclusivity group invariant is critical behavioral logic. Test it thoroughly.

- [x] Task 8.1.3: Add `#[cfg(test)]` module to `measurements.rs`
  - File: `src-tauri/src/score/components/measurements.rs`
  - Action: Append tests for:
    - `BoundingBox::total_width()` → `width + right_padding`
    - `PaddingMm::to_spaces(mm_per_space)` — converts all 4 sides correctly
    - `PaddingSpaces::new()` — constructs with correct fields
  - Notes: Simple math, but foundational types used everywhere.

- [x] Task 8.1.4: Add `#[cfg(test)]` module to `velocity.rs`
  - File: `src-tauri/src/score/components/velocity.rs`
  - Action: Append tests for:
    - `Velocity::new(100).int()` → `100`
    - `Velocity::new(0).int()` → `0`
    - `Velocity::new(127).int()` → `127`
  - Notes: Trivial wrapper — minimal tests for completeness.

#### Story 8.2: Track Operations Tests

- [x] Task 8.2.1: Add `#[cfg(test)]` module to `tracks.rs`
  - File: `src-tauri/src/score/tracks.rs`
  - Action: Append tests for:
    - `Track::new(id)` — creates track with empty entries
    - `insert_entry` — inserts entry, appears in `by_id` and `by_tick`
    - `insert_entry` multiple entries at same tick — both appear in `by_tick[tick]`
    - `shift_entry` — moves entry to new tick, old tick no longer contains it, new tick does
    - `shift_entry` to same tick — no-op, no duplication
    - `remove_entry` — entry removed from both `by_id` and `by_tick`, returns removed entry
    - `remove_entry` with non-existent id — returns `None`, doesn't panic
    - `Tracks::new()` — creates empty `by_id` map
  - Notes: Use `Entry::Tone(...)` test fixtures to insert. These are critical data structure operations — the dual-index invariant (by_id + by_tick stay in sync) must be tested.

#### Story 8.3: Entry Subtype Tests

- [x] Task 8.3.1: Add `#[cfg(test)]` module to `barline.rs`
  - File: `src-tauri/src/score/entries/barline.rs`
  - Action: Append tests for:
    - `BarlineDrawType::metrics()` — each variant returns correct `BoundingBox` (width, height, right_padding)
    - `Barline::new(id, tick, draw_type)` — construction
    - `Track::get_barline_at_tick()` — returns barline when present, `None` when absent
  - Notes: Build a `Track` fixture, insert a `Barline` entry, query.

- [x] Task 8.3.2: Add `#[cfg(test)]` module to `clef.rs`
  - File: `src-tauri/src/score/entries/clef.rs`
  - Action: Append tests for:
    - `Clef::metrics(true)` and `Clef::metrics(false)` — full vs reduced size
    - `Clef::metrics()` for `ClefDrawType::Hidden` returns width 0
    - `Clef::glyph()` — G returns treble, F returns bass, Hidden returns None
    - `Clef::default()` — returns C clef at tick 0
    - `Track::get_clef_at_tick()` — returns clef when present, `None` when absent
  - Notes: Use concrete Pitch values for clef construction.

- [x] Task 8.3.3: Add `#[cfg(test)]` module to `key_signature.rs`
  - File: `src-tauri/src/score/entries/key_signature.rs`
  - Action: Append tests for:
    - `KeySignature::accidentals()` — offset 3 (sharps) returns 3 sharps, offset -2 (flats) returns 2 flats, offset 0 returns empty set
    - `KeySignature::metrics()` — offset 3 returns width 3.0, offset 0 returns width 0.0 with no padding
    - `KeySignature::new()` — construction
    - `KeySignature::default()` — returns Major, offset 0
  - Notes: Verify the sharps/flats returned match the SHARPS/FLATS constants.

- [x] Task 8.3.4: Add `#[cfg(test)]` module to `time_signature.rs`
  - File: `src-tauri/src/score/entries/time_signature.rs`
  - Action: Append tests for:
    - `TimeSignature::default()` — returns 4/4 hidden
    - `TimeSignature::new(...)` — construction with all params
    - `Track::get_time_signature_at_tick()` — returns time sig when present, `None` when absent
    - Time signature type classification if such method exists
  - Notes: Check the file for any additional pure methods beyond construction and default.

- [x] Task 8.3.5: Add `#[cfg(test)]` module to `tone.rs`
  - File: `src-tauri/src/score/entries/tone.rs`
  - Action: Append tests for:
    - `Tone::new(...)` — construction with all 6 fields
    - `Track::get_tones_at_tick(tick)` — returns tones at matching tick, empty vec when none
    - `Track::first_articulations_at_tick(tick)` — returns first tone's articulations
    - `Track::apply_articulations_to_tick(tick, articulations)` — updates all tones at tick
  - Notes: Insert multiple tones at the same tick to test multi-tone queries.

### Phase 9: Rust Parse Pipeline Tests

#### Story 9.1: Parse Function Tests

- [x] Task 9.1.1: Add `#[cfg(test)]` module to `get_barlines.rs`
  - File: `src-tauri/src/instructions/parse/get_barlines.rs`
  - Action: Append tests for:
    - Given flow with no custom barlines → returns default barline defs at measure boundaries
    - Given flow with a double barline at tick 128 → returns `BarlineDrawDef` with `draw_type: Some(Double)` at that tick
    - Given EndStartRepeat barline with a time sig at same tick → `split_by_signature` is true
  - Notes: Requires constructing `Flow`, `Tracks`, and `Measures` fixtures. Reuse the fixture pattern from `get_measures` tests.

- [x] Task 9.1.2: Add `#[cfg(test)]` module to `get_key_signatures.rs`
  - File: `src-tauri/src/instructions/parse/get_key_signatures.rs`
  - Action: Append tests for:
    - Given a flow with key sig offset 3 (A major) at tick 0 → returns entry at tick 0 with `negate: false`
    - Given a key sig change back to C major (offset 0) at tick 128 → returns entry with `negate: true` (cancels previous sharps)
  - Notes: Construct `Flow` and `Tracks` with key signature entries in the master track.

- [x] Task 9.1.3: Add `#[cfg(test)]` module to `get_shunts.rs`
  - File: `src-tauri/src/instructions/parse/get_shunts.rs`
  - Action: Append tests for `note_shunts_in_chord`:
    - Single tone → `Shunt::None`
    - Two tones a step apart, stem up → first `None`, second `Post`
    - Two tones a step apart, stem down → first `Pre`, second `None`
    - Three tones a step apart (cluster) → alternating shunts
  - Notes: Test the core `note_shunts_in_chord` function directly with constructed fixtures. This is pure algorithmic logic — no IO.

- [x] Task 9.1.4: Add `#[cfg(test)]` module to `get_accidentals.rs`
  - File: `src-tauri/src/instructions/parse/get_accidentals.rs`
  - Action: Append tests for:
    - `sort_accidentals_alternate` — alternates IDs from top and bottom
    - `is_accidental_needed` — accidental needed when note differs from key sig, not needed when it matches
    - `fits_in_slot` — returns true/false based on offset proximity
  - Notes: Test the helper functions individually. Skip the full `get_accidentals` function as it requires the full `NotationByTrack` pipeline — test the building blocks instead.

- [x] Task 9.1.5: Add `#[cfg(test)]` module to `get_stem_directions.rs`
  - File: `src-tauri/src/instructions/parse/get_stem_directions.rs`
  - Action: Append tests for `get_span_stem_direction`:
    - All notes above middle → returns `Up`
    - All notes below middle → returns `Down`
    - Equal up/down → uses furthest note to break tie
  - Notes: Construct `Beam` and `NotationTrack` fixtures with known offsets.

- [x] Task 9.1.6: Add `#[cfg(test)]` module to `get_written_durations.rs`
  - File: `src-tauri/src/instructions/parse/get_written_durations.rs`
  - Action: Append tests for `Notation` struct methods:
    - `is_rest()` — true when no tones, false when tones present
    - `is_dotted()` — true for duration 48 (dotted quarter), false for 32 (quarter)
    - `is_writable(ticks)` — true for power-of-2 durations (128, 64, 32...), false for others
    - `base_duration()` — dotted quarter (48) → Some(32), non-dotted → None
    - `base_to_note_duration()` — maps to correct `NoteDuration` variant
    - `note_glyph()` — returns correct Unicode glyph for each duration
    - `rest_glyph()` — returns correct rest glyph
    - `glyph_metrics()` — returns correct `BoundingBox` for rest vs note vs whole
  - Notes: `Notation::new(tick, duration)` creates a rest (empty tones). Push tones to make it a note. Test the whole vs half vs quarter distinction.

- [x] Task 9.1.7: Add `#[cfg(test)]` module to `get_vertical_spacing.rs`
  - File: `src-tauri/src/instructions/parse/get_vertical_spacing.rs`
  - Action: Append tests for:
    - Single instrument with one stave → height equals stave lines - 1
    - Two instruments → includes `instrument_spacing` between them
    - Instrument with two staves → includes `stave_spacing` between staves
    - Stave y-position accounts for accumulated heights
  - Notes: Construct lightweight `Flow`, `Instrument`, and `Engrave` fixtures.

### Phase 10: Coverage Threshold Ratchet & Docs

#### Story 10.1: Update Thresholds and Documentation

- [x] Task 10.1.1: Run coverage report and ratchet thresholds
  - File: `vite.config.ts`
  - Action: After all tests pass, run `npm run test:coverage`, note actual coverage percentages, and bump the `thresholds` object in `vite.config.ts` to match (rounded down to nearest 5%). Target: statements ≥ 60%, branches ≥ 50%, functions ≥ 55%, lines ≥ 60%.
  - Notes: These numbers reflect pragmatic coverage — some files (boot.tsx, index.tsx routers, provider.tsx) are legitimately excluded from unit tests. Adjust based on actual numbers.

- [x] Task 10.1.2: Update `TESTING.md` with new patterns
  - File: `TESTING.md`
  - Action: Add sections for:
    - `renderWithProviders` usage pattern with example
    - Mocking the `actions` module for service/hook tests
    - Mocking `@tauri-apps/api/window` for window-dependent components
    - Context wrapper pattern for hook tests
    - Updated coverage thresholds
  - Notes: Keep it concise and example-driven.

### Acceptance Criteria

#### Phase 1

- [x] AC 1.1: Given `src/services/test-utils.tsx`, when imported, `renderWithProviders` wraps a component in all 4 context providers and `createAppState` produces a deep-merged state.

#### Phase 2

- [x] AC 2.1: Given `popovers.test.ts`, when tests run, then all 4 popover parsers are tested with valid inputs that verify the correct `actions.score.*` method is called with the right arguments, and invalid inputs throw errors.
- [x] AC 2.2: Given `hit-test.test.ts`, when tests run, then `hitTest` calls `invoke("renderer_hit_test")` with correct argument mapping.

#### Phase 3

- [x] AC 3.1: Given button test files, when tests run, then `Button`, `IconButton`, `BravuraIconButton` render correctly, fire onClick, and apply variant/disabled/selected styling.
- [x] AC 3.2: Given input test files, when tests run, then `TextInput` handles typing/blur/enter, `NumberInput` handles arrow key increment/decrement with precision, and `Input` routes to the correct sub-component.
- [x] AC 3.3: Given `textarea.test.tsx`, `menu.test.tsx`, and text tests, when tests run, components render correctly and handle user interaction.
- [x] AC 3.4: Given `popover.test.tsx`, when tests run, then the Popover component shows/hides correctly, dispatches to the right parser on Enter, and closes on Escape/blur.

#### Phase 4

- [x] AC 4.1: Given shared component test files, when tests run, then `NotFound`, `SelectByImage`, `AppBar`, and `DragScroll` render correctly and handle interactions.

#### Phase 5

- [x] AC 5.1: Given `repeat-array.test.ts`, when tests run, then array repetition works for normal, empty, and zero-count cases.
- [x] AC 5.2: Given hook test files, when tests run, then `useTicks` calculates correct bar/beat positions, `useSlotPattern` generates correct piano key patterns, `useBottomPanelState` derives correct panel state, `useSelectedToneInfo` returns correct tone info, and `useKeyboardShortcuts` dispatches correct actions.

#### Phase 6

- [x] AC 6.1: Given `context.test.tsx`, when tests run, then all 4 context hooks throw the correct error when used outside their provider and return state when used inside.

#### Phase 7

- [x] AC 7.1: Given window component test files, when tests run, then `Transport`, `ZoomWidget`, `Fader`, `Keyboard`, `Tools`, and `SelectedToneInfoDisplay` render correctly with context data and fire correct actions on interaction.

#### Phase 8

- [x] AC 8.1: Given `cargo test`, when tests execute, then `pitch.rs` tests cover `base()`, `steps()`, `half_steps()`, `octave()`, `steps_between()`, `shift_by_steps()`, `shift_by_octaves()`, `from_int()`, and `Accidental::to_glyph()`.
- [x] AC 8.2: Given `cargo test`, when tests execute, then `articulation.rs` tests verify `toggle()` enforces exclusivity groups and `Articulations` collection methods work correctly.
- [x] AC 8.3: Given `cargo test`, when tests execute, then `tracks.rs` tests verify `insert_entry`/`shift_entry`/`remove_entry` maintain dual-index invariant.
- [x] AC 8.4: Given `cargo test`, when tests execute, then all 5 entry subtype modules have tests for construction, pure methods, and `Track::get_*_at_tick()` queries.

#### Phase 9

- [x] AC 9.1: Given `cargo test`, when tests execute, then parse pipeline tests cover `get_barlines`, `get_key_signatures`, `note_shunts_in_chord`, accidental helpers, `get_span_stem_direction`, `Notation` methods, and `get_vertical_spacing`.

#### Phase 10

- [x] AC 10.1: Given updated `vite.config.ts` thresholds, when `npm run test:coverage` is run with coverage below the new thresholds, then the test run fails.
- [x] AC 10.2: Given updated `TESTING.md`, when an AI agent reads it, it includes `renderWithProviders` pattern, actions mocking pattern, and context wrapper examples.

## Additional Context

### Dependencies

**No new npm dependencies needed.** All required testing packages were installed in the foundation spec (Vitest, React Testing Library, happy-dom, @vitest/coverage-v8).

**No new Cargo dependencies needed.** Rust tests use the built-in test harness with existing crate dependencies.

### Testing Strategy

Phases 1→10, roughly ordered by:

1. **Foundation first** (context wrapper) — unlocks component testing
2. **High-value service logic** (popovers) — core editing functionality
3. **UI primitives** (buttons, inputs) — broad coverage, low complexity
4. **Shared components** — breadth
5. **Window hooks** — business logic living in hooks
6. **Context layer** — error boundary behavior
7. **Window components** — the bulk, using the context wrapper
8. **Rust model** — core data model safety
9. **Rust parse pipeline** — algorithmic safety
10. **Ratchet** — lock in the gains

Each phase is independently shippable. Start with Phases 1-2 (highest value), then sweep through 3-9, finish with 10.

### Notes

- `actions.ts` is explicitly excluded — it's 887 lines of `invoke()` wrappers with zero branching logic. Mocking invoke and verifying `toHaveBeenCalledWith` for 80+ methods is pure ceremony.
- `provider.tsx` is excluded — testing event listener wiring + RAF polling against mocked Tauri APIs has very high mocking complexity for minimal behavior verification. The provider's logic is validated implicitly by the components that consume context.
- Window `boot.tsx` files are each ~5 lines that call `createRoot().render(...)` — untestable without a DOM mount and not worth the setup.
- The `NoteDuration` enum typo (`HudredTwentyEighth`) persists — do NOT rename it.
- Phase ordering is flexible — Phases 3-7 (FE) and Phases 8-9 (Rust) can be parallelized if multiple agents are working.
- After all phases complete, expected FE coverage should be in the 65-80% range (depending on how much window component coverage is achievable). Rust coverage should be in the 60-80% range across tested modules.
