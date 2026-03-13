---
title: "Testing Foundation & Tooling Modernization"
slug: "testing-foundation-tooling-modernization"
created: "2026-03-04"
status: "phases-1-3-complete"
stepsCompleted: [1, 2, 3]
tech_stack:
  [
    "vitest",
    "happy-dom",
    "@testing-library/react",
    "@testing-library/user-event",
    "@vitest/coverage-v8",
    "typescript-eslint v8",
    "lint-staged",
    "simple-git-hooks",
  ]
files_to_modify:
  # Phase 1 — Testing Foundation
  - "package.json" # add test deps + scripts
  - "vite.config.ts" # extend with test config (or new vitest.config.ts)
  - "tsconfig.json" # add test include paths
  - "src/services/pitch-utils.test.ts" # NEW — pure function tests
  - "src/services/utils.test.ts" # NEW — pure function tests (parseTimeSignature, noteDurationToTicks, color utils)
  - "src/services/sequencer-utils.test.ts" # NEW — DOM query helper tests
  - "src/__mocks__/@tauri-apps/api/core.ts" # NEW — Tauri invoke mock
  - "src-tauri/src/score/components/duration.rs" # add #[cfg(test)] module
  - "src-tauri/src/renderer/converter.rs" # add #[cfg(test)] module
  - "src-tauri/src/instructions/parse/get_measures.rs" # add #[cfg(test)] module
  - "src-tauri/src/score/entries/mod.rs" # add #[cfg(test)] module
  # Phase 2 — Component & Hook Testing
  - "src/ui/checkbox.test.tsx" # NEW
  - "src/ui/switch.test.tsx" # NEW
  - "src/ui/tabs.test.tsx" # NEW
  - "src/ui/select.test.tsx" # NEW
  - "src/services/hooks.test.tsx" # NEW — useHumanizedTime, useAsync
  - "src/components/snap-picker.test.tsx" # NEW
  # Phase 3 — Tooling & Guardrails
  - "eslint.config.js" # modernize to typescript-eslint v8 unified config
  - "package.json" # simple-git-hooks + lint-staged config added as package.json fields
  - "TESTING.md" # NEW — test patterns & conventions doc
  - "docs/development-guide.md" # updated Testing section + new Git Conventions section
  # Phase 4 — E2E Smoke Tests (DEFERRED — see Phase 4 notes)
code_patterns:
  - "FC<Props> functional components only"
  - "snake_case type properties crossing FE↔BE boundary"
  - "FxHashMap everywhere in Rust (never std HashMap)"
  - "invoke() calls via actions object in actions.ts"
  - "Numeric enums mirrored between TS and Rust via serde_repr"
  - "kebab-case TS/TSX filenames, snake_case Rust filenames"
  - "One component per file"
  - "Test files colocated next to source (e.g. utils.ts → utils.test.ts)"
test_patterns:
  - "Vitest + happy-dom for TS/TSX"
  - "#[cfg(test)] modules for Rust"
  - "Test files colocated next to source, no __tests__/ subdirectories"
  - "@testing-library/react for component tests"
  - "Manual mocks in src/__mocks__/ for Tauri API"
  - "WebdriverIO + tauri-driver for E2E smoke tests"
---

# Tech-Spec: Testing Foundation & Tooling Modernization

**Created:** 2026-03-04

## Overview

### Problem Statement

Zero tests exist across ~100 TS/TSX and ~99 Rust files. No safety net for AI-assisted or manual development. ESLint config uses legacy `@typescript-eslint/eslint-plugin` + `@typescript-eslint/parser` imports instead of the modern unified `typescript-eslint` v8 API. No pre-commit guardrails to catch issues before they land.

### Solution

Add Vitest + React Testing Library for frontend testing, establish Rust `#[cfg(test)]` foundations for backend testing, modernize ESLint to `typescript-eslint` v8 unified config, add `lint-staged` + `simple-git-hooks` for pre-commit quality gates, and architect a lightweight E2E smoke layer using `tauri-driver` + WebdriverIO.

### Scope

**In Scope:**

- Phase 1: Vitest setup, pure function tests (TS), Rust `#[cfg(test)]` foundations, Tauri invoke mock helper
- Phase 2: React component testing (UI primitives), custom hook testing, composite component tests
- Phase 3: ESLint modernization to `typescript-eslint` v8, pre-commit hooks via `lint-staged` + `simple-git-hooks`, coverage thresholds, test pattern documentation (TESTING.md)
- Phase 4: E2E smoke test architecture with `tauri-driver` + WebdriverIO (lightweight critical-path tests only)

**Out of Scope:**

- CI pipeline / unified CI script (deferred until pipeline is added)
- Full comprehensive Rust test coverage (foundations and patterns only)
- Full E2E test suite (Phase 4 is a thin smoke layer, not comprehensive coverage)
- Score rendering visual assertions (Skia/Metal surface is invisible to WebDriver)

## Context for Development

### Codebase Patterns

- Multi-page Vite app with 8 HTML entry points — tests focus on `src/**`, not entry HTML files
- All FE→BE calls go through `actions.ts` (888 lines) via `invoke()` from `@tauri-apps/api/core` — single mock point for IPC
- State pushed to FE via Tauri events — `initialAppState` in `src/services/state/context.tsx` (221 lines) provides excellent test fixture data
- No routing — each window is a separate HTML entry with its own `boot.tsx`, simplifying component testing (no router provider needed)
- Strict TypeScript enabled (`strict: true`, `noUnusedLocals`, `noUnusedParameters`) — tests must also be strictly typed
- All TS/TSX files use kebab-case filenames; Rust files use snake_case
- One component per file, `FC<Props>` only, hooks only
- `react-refresh/only-export-components` enforced — test files must not be in same file as components
- 4 React contexts: `AppContext`, `AudioContext`, `SamplerContext`, `TransportContext` — components may require provider wrappers in tests
- Rust crate is `compose_lib` (lib target) — tests run via `cargo test` natively
- Rust uses `FxHashMap` everywhere; test fixtures must use `FxHashMap::default()` not `HashMap::new()`
- Score data model: `Entry` is a tagged enum with 5 variants (`Barline`, `Clef`, `KeySignature`, `TimeSignature`, `Tone`)
- Dual-index storage: `by_tick: FxHashMap<Tick, Vec<UniqueId>>` + `by_id: FxHashMap<UniqueId, Entry>`
- Parse pipeline (19 modules in `src-tauri/src/instructions/parse/`) — pure transformation functions, excellent Rust test candidates
- `Converter` struct (`src-tauri/src/renderer/converter.rs`) — pure `spaces_to_px`/`px_to_spaces` math, trivially testable
- `NoteDuration` enum (`src-tauri/src/score/components/duration.rs`) — `from_ticks()`, `to_ticks()`, `half()`, `spacing_ratio()` — pure methods, ideal first Rust tests

### Files to Reference

| File                                               | Purpose                                                                       |
| -------------------------------------------------- | ----------------------------------------------------------------------------- |
| `src/services/pitch-utils.ts`                      | Pure pitch math functions (75 lines) — P1 test target                         |
| `src/services/utils.ts`                            | Mixed: pure functions + hooks (352 lines) — P1 pure fns, P2 hooks             |
| `src/services/sequencer-utils.ts`                  | DOM query helper (30 lines) — P1 test with happy-dom                          |
| `src/services/hit-test.ts`                         | IPC wrapper (18 lines) — example of code needing Tauri mock                   |
| `src/services/actions.ts`                          | All `invoke()` calls (887 lines) — mock target                                |
| `src/services/types.ts`                            | All type/enum definitions (449 lines) — test fixture reference                |
| `src/services/state/context.tsx`                   | `initialAppState` fixture data (220 lines)                                    |
| `src/ui/checkbox.tsx`                              | Simple UI primitive (42 lines, no a11y attrs) — P2 component test             |
| `src/ui/switch.tsx`                                | UI primitive with a11y (`role="switch"`, `aria-checked`) (77 lines) — P2 test |
| `src/ui/tabs.tsx`                                  | Generic component `<T extends Key>` (48 lines) — P2 test                      |
| `src/ui/select.tsx`                                | Complex UI with dropdown state (158 lines) — P2 test                          |
| `src/utils/mm.ts`                                  | DOM measurement (12 lines) — needs happy-dom                                  |
| `src-tauri/src/score/components/duration.rs`       | `NoteDuration` enum + methods (106 lines) — P1 Rust test                      |
| `src-tauri/src/renderer/converter.rs`              | `Converter` space↔px math (28 lines) — P1 Rust test                           |
| `src-tauri/src/instructions/parse/get_measures.rs` | Pure measure calculation (28 lines) — P1 Rust test                            |
| `src-tauri/src/score/entries/mod.rs`               | `Entry` enum + `Entries` struct (78 lines) — P1 Rust test                     |
| `src-tauri/src/score/components/pitch.rs`          | `Pitch`, `Accidental`, `ACCIDENTAL_MAP` (718 lines) — Rust test candidate     |
| `src-tauri/src/utils.rs`                           | `UniqueId` type + `now()` helper (10 lines) — foundational types              |
| `vite.config.ts`                                   | Vite config to extend for Vitest                                              |
| `eslint.config.js`                                 | Legacy ESLint flat config to modernize                                        |
| `package.json`                                     | Scripts + deps to add                                                         |
| `tsconfig.json`                                    | Strict TS config — tests must comply                                          |

### Technical Decisions

- **Vitest over Jest:** Native Vite integration, shares transform pipeline and path aliases, zero-config with existing `vite.config.ts`. No separate Babel/SWC transform needed.
- **happy-dom over jsdom:** Lighter and faster, sufficient for this app's DOM needs (no complex browser APIs required).
- **Test files colocated next to source:** Test files live directly alongside the source file they test (e.g. `utils.ts` → `utils.test.ts`) rather than in `__tests__/` subdirectories or a top-level `tests/` folder. Keeps tests immediately discoverable without an extra directory hop.
- **Tauri mock in `src/__mocks__/`:** Vitest supports `__mocks__` directories for manual module mocks. A mock for `@tauri-apps/api/core` intercepts all `invoke()` calls.
- **simple-git-hooks over husky:** Lighter footprint, zero dependency, configured via `simple-git-hooks` field in `package.json` (not a separate `.simple-git-hooks.json` file).
- **`typescript-eslint` v8 unified config:** Replaces separate `@typescript-eslint/eslint-plugin` + `@typescript-eslint/parser` with single `typescript-eslint` import and `tseslint.config(...)` API. Cleaner, enables type-checked rules.
- **WebdriverIO for E2E:** Most mature JS WebDriver client. Pairs with `tauri-driver` which wraps the native WebView's WebDriver protocol. Tests run against the real compiled app binary.
- **E2E as smoke only:** Score rendering is Skia/Metal — invisible to WebDriver. Unit/integration tests cover ~90% of value. E2E covers IPC contract, multi-window lifecycle, and critical user journeys only.
- **Rust `#[cfg(test)]` in-module tests:** Standard Rust convention. Tests live inside the source file in a `#[cfg(test)] mod tests {}` block. No separate test files needed for unit tests.

## Implementation Plan

### Phase 1: Testing Foundation

#### Story 1.1: Install Vitest & Configure Test Runner

- [x] Task 1.1.1: Install test dependencies
  - File: `package.json`
  - Action: Add devDependencies: `vitest`, `@vitest/coverage-v8`, `happy-dom`, `@testing-library/react`, `@testing-library/user-event`, `@testing-library/jest-dom`
  - Notes: Use latest stable versions. All are devDependencies only.

- [x] Task 1.1.2: Add test scripts to package.json
  - File: `package.json`
  - Action: Add scripts: `"test": "vitest run"`, `"test:watch": "vitest"`, `"test:coverage": "vitest run --coverage"`
  - Notes: `vitest run` executes once (CI-friendly), `vitest` runs in watch mode (dev-friendly).

- [x] Task 1.1.3: Add Vitest config to vite.config.ts
  - File: `vite.config.ts`
  - Action: Add `/// <reference types="vitest/config" />` at top. Add `test` block to the config object:
    ```ts
    test: {
      globals: true,
      environment: 'happy-dom',
      include: ['src/**/*.test.{ts,tsx}'],
      coverage: {
        provider: 'v8',
        include: ['src/**/*.{ts,tsx}'],
        exclude: ['src/**/*.test.{ts,tsx}', 'src/__mocks__/**', 'src/vite-env.d.ts'],
      },
      setupFiles: ['./src/test-setup.ts'],
    }
    ```
  - Notes: No separate `vitest.config.ts` needed — extending existing `vite.config.ts` shares aliases/plugins automatically. The current config uses `defineConfig(async () => ({...}))` — the `test` block goes inside the returned object alongside `plugins`, `server`, and `build`. Place it after the `build` block:
    ```ts
    export default defineConfig(async () => ({
      plugins: [...],
      server: {...},
      build: {...},
      test: {
        // ... test config here
      },
    }));
    ```

- [x] Task 1.1.4: Create test setup file
  - File: `src/test-setup.ts` — **NEW**
  - Action: Create with `import '@testing-library/jest-dom/vitest'` to register custom DOM matchers globally.
  - Notes: This enables `.toBeVisible()`, `.toHaveAttribute()`, etc. in all test files without per-file imports.

- [x] Task 1.1.5: Add Vitest types to test setup
  - File: `src/test-setup.ts`
  - Action: Add `/// <reference types="vitest/globals" />` as the first line of the test setup file. Do **NOT** add a `compilerOptions.types` array to `tsconfig.json` — doing so would override TypeScript's automatic `@types/*` inclusion and break the build (e.g. losing `@types/react` globals).
  - Notes: The triple-slash directive is scoped to the test setup file, which Vitest loads before all tests. This cleanly provides global `describe`, `it`, `expect` types without affecting the main build.

#### Story 1.2: Create Tauri IPC Mock

- [x] Task 1.2.1: Create manual mock for `@tauri-apps/api/core`
  - File: `src/__mocks__/@tauri-apps/api/core.ts` — **NEW**
  - Action: Create mock that exports a vi.fn() `invoke` function. The mock should:
    - Export `const invoke = vi.fn()` as a named export
    - Default to resolving `undefined` (safe no-op)
    - Allow per-test override via `invoke.mockResolvedValueOnce(...)` or `invoke.mockImplementation(...)`
  - Notes: Vitest auto-resolves `__mocks__` relative to the source root. The directory structure must match the module path exactly: `src/__mocks__/@tauri-apps/api/core.ts`.

#### Story 1.3: Pure Function Tests — TypeScript

- [x] Task 1.3.1: Test `pitch-utils.ts` functions
  - File: `src/services/pitch-utils.test.ts` — **NEW**
  - Action: Write tests for all 5 exported functions:
    - `naturalizePitch(pitch)` — test all 5 accidental cases (DoubleSharp through DoubleFlat) + default
    - `getPitchLetter(pitch)` — test C/D/E/F/G/A/B mapping from base pitch mod 12, plus non-natural-note input returning `""`
    - `getAccidentalString(accidental)` — test all 5 accidental enum values return correct token strings
    - `getPitchOctave(pitch)` — test octave calculation (`Math.floor(int / 12) - 1`)
    - `getPitchName(pitch)` — integration test combining letter + accidental + octave (e.g. `{int: 60, accidental: Accidental.Natural}` → `"C4"`)
  - Notes: Import `Pitch` and `Accidental` types from `../types`. Use real enum values from `types.ts`. The `Accidental` enum uses signed integers centered on zero: `DoubleSharp = 2, Sharp = 1, Natural = 0, Flat = -1, DoubleFlat = -2`.

- [x] Task 1.3.2: Test `utils.ts` pure functions
  - File: `src/services/utils.test.ts` — **NEW**
  - Action: Write tests for pure functions only (no hooks — those are Phase 2):
    - `parseTimeSignature(input)` — valid inputs (`"4/4"` → `{beats: 4, beatType: 4}`), invalid beat types (`"4/3"` → `null`), malformed strings (`"abc"`, `""`, `"4/"`) → `null`
    - `noteDurationToTicks(dur)` — test all 8 `NoteDuration` enum values map to correct tick counts (Whole=128 through HudredTwentyEighth=1), plus default case → 0
    - `generateRainbowColors(count)` — test returns array of hex strings, minimum 7 colors even when count < 7, correct count when count > 7, all strings start with `#` and are 7 chars
    - `desaturateColor(hex, satFactor, lightFactor)` — test returns valid hex string, `(1, 1)` factors return same-ish color, `(0, 0.5)` returns greyscale-ish
    - `lightenColor(hex, factor)` — test returns valid hex, factor 0 returns same color, factor 1 returns near-white
  - Notes: Import `NoteDuration` from `../types`. Color function tests should validate format and reasonable output ranges, not exact RGB values (floating-point rounding).

- [x] Task 1.3.3: Test `sequencer-utils.ts` DOM query
  - File: `src/services/sequencer-utils.test.ts` — **NEW**
  - Action: Write tests for `getSequencerClickTarget(target)`:
    - Given element with `data-tone-id="42"` ancestor → returns `{type: "Tone", id: 42}`
    - Given element with no `data-tone-id` ancestor → returns `null`
    - Given element IS the `data-tone-id` element → returns correct target
  - Notes: Uses happy-dom environment (set in vitest config). Create DOM elements directly with `document.createElement()`. The function uses `target.closest("[data-tone-id]")` so test both direct and ancestor scenarios.

#### Story 1.4: Rust Test Foundations

- [x] Task 1.4.1: Add `#[cfg(test)]` module to `duration.rs`
  - File: `src-tauri/src/score/components/duration.rs`
  - Action: Append `#[cfg(test)] mod tests {}` block with tests for:
    - `NoteDuration::from_ticks(ticks)` — all 8 valid tick values return `Some(correct_variant)`, invalid tick (e.g. `99`) returns `None`
    - `NoteDuration::to_ticks()` — all 8 variants return correct tick count, verify round-trip: `NoteDuration::from_ticks(dur.to_ticks()) == Some(dur)`
    - `NoteDuration::half()` — verify each duration halves correctly (Whole→Half, Half→Quarter, etc.)
    - `Direction::to_modifier()` — `Up` returns `-1`, `Down` returns `1`
    - `NoteDuration::spacing_ratio(ratio, is_dotted)` — spot-check a few values, verify dotted multiplier is 1.2x
  - Notes: Use `#[test]` attribute on each test function. Use `assert_eq!` for value comparisons. Add `#[should_panic]` test for `HudredTwentyEighth.half()`.

- [x] Task 1.4.2: Add `#[cfg(test)]` module to `converter.rs`
  - File: `src-tauri/src/renderer/converter.rs`
  - Action: Append `#[cfg(test)] mod tests {}` block with tests for:
    - `Converter::spaces_to_px(spaces)` — given known `px_per_mm` and `mm_per_space`, verify calculation: `spaces * mm_per_space * px_per_mm`
    - `Converter::px_to_spaces(px)` — inverse of above
    - Round-trip: `converter.px_to_spaces(converter.spaces_to_px(5.0))` ≈ `5.0` (within f32 epsilon)
  - Notes: Construct `Converter::new(px_per_mm, mm_per_space)` with known values. Use `assert!((result - expected).abs() < f32::EPSILON)` for float comparisons.

- [x] Task 1.4.3: Add `#[cfg(test)]` module to `get_measures.rs`
  - File: `src-tauri/src/instructions/parse/get_measures.rs`
  - Action: Append `#[cfg(test)] mod tests {}` block with tests for:
    - Given a flow with a single 4/4 time signature at tick 0 and length 512 → returns measures at ticks 0, 128, 256, 384
    - Given a flow with a time signature change (4/4 at 0, 3/4 at 256) → correct measure boundaries
  - Notes: Requires constructing test `Flow` and `Tracks` fixtures with their full field sets:
    - `Flow { id, title: String, player_ids: FxHashSet<UniqueId>, length: Ticks, master_id: UniqueId, staves: Staves }` — use `FxHashSet::default()` for `player_ids`, `Staves::new()` for `staves`
    - `Tracks { by_id: FxHashMap<UniqueId, Track> }` — create a master `Track` with time signature entries inserted via `track.insert_entry(Entry::TimeSignature(...))`
    - `TimeSignature::new(id, tick, beats, beat_type, draw_type, groupings)` takes 6 params — use `TimeSignature::new(1, 0, 4, NoteDuration::Quarter, TimeSignatureDrawType::Hidden, None)` for a 4/4 signature
      Example fixture helper:
    ```rust
    fn test_flow(master_id: UniqueId, length: Ticks) -> Flow {
        Flow { id: 1, title: String::new(), player_ids: FxHashSet::default(), length, master_id, staves: Staves::new() }
    }
    ```
    Use `FxHashMap::default()` for all hash maps and `FxHashSet::default()` for hash sets.

- [x] Task 1.4.4: Add `#[cfg(test)]` module to `entries/mod.rs`
  - File: `src-tauri/src/score/entries/mod.rs`
  - Action: Append `#[cfg(test)] mod tests {}` block with tests for:
    - `Entries::new()` — creates empty `by_tick` and `by_id` maps
    - `Entry::id()` — returns correct id for each variant (construct one of each variant, verify `entry.id()` matches)
    - `Entry::tick()` — returns correct tick for each variant
    - `Entry::set_tick(new_tick)` — mutates tick and `entry.tick()` reflects new value
  - Notes: Construct test instances of each variant. Each has specific required fields beyond `id` and `tick`:
    - `Barline { id, tick, barline_draw_type: BarlineDrawType }` — use `BarlineDrawType::Single`
    - `Clef { id, tick, draw_as: ClefDrawType, pitch: Pitch, offset: i8 }` — use `ClefDrawType::G`, `Pitch::new(67, Accidental::Natural)`, offset `0`
    - `KeySignature { id, tick, mode: KeySignatureMode, offset: i8 }` — use `KeySignatureMode::Major`, offset `0`
    - `TimeSignature { id, tick, beats, beat_type: NoteDuration, draw_type: TimeSignatureDrawType, groupings: Vec<u8> }` — use `TimeSignature::default()` or `TimeSignature::new(...)` with 6 params
    - `Tone { id, tick, duration: Ticks, pitch: Pitch, velocity: Velocity, articulations: Articulations }` — use concrete values for all 6 fields
      Import required types from sibling modules (`barline::*`, `clef::*`, `key_signature::*`, `time_signature::*`, `tone::*`) and component modules (`pitch::*`, `duration::*`).

### Phase 2: Component & Hook Testing

#### Story 2.1: UI Primitive Component Tests

- [x] Task 2.1.1: Test `Checkbox` component
  - File: `src/ui/checkbox.test.tsx` — **NEW**
  - Action: Write tests using React Testing Library:
    - Renders without crashing
    - When `value=true`, check icon is visible
    - When `value=false`, check icon is not rendered
    - Clicking toggles: fires `onChange` with `!value`
    - Custom `className` is applied
    - Has `role="checkbox"` or accessible name for screen readers (note: current implementation lacks explicit a11y attributes — consider adding `role="checkbox"` and `aria-checked` as part of the test-writing process or flagging as a follow-up improvement)
    - `onPointerDown` callback fires when provided
  - Notes: Import `render`, `screen` from `@testing-library/react`, `userEvent` from `@testing-library/user-event`. Use `vi.fn()` for `onChange` and `onPointerDown` callbacks. The `Checkbox` component currently has no `role` or `aria-*` attributes unlike `Switch` — test the a11y gap as a documented finding, even if the component doesn't pass those assertions today.

- [x] Task 2.1.2: Test `Switch` component
  - File: `src/ui/switch.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders with `role="switch"` attribute
    - `aria-checked` reflects `value` prop
    - `aria-label` reflects `label` prop
    - Clicking fires `onChange` with `!value`
    - Label and description text render when provided
    - `small` variant changes sizing classes
    - `onPointerDown` callback fires when provided
  - Notes: The `Switch` component has good a11y attributes — use `screen.getByRole('switch')` for querying. Use `vi.fn()` for `onPointerDown` callback.

- [x] Task 2.1.3: Test `Tabs` component
  - File: `src/ui/tabs.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders all options as buttons
    - Selected tab has active styling class (`text-black bg-sky-500`)
    - Clicking a tab fires `onChange` with that option's value
    - Works with generic type parameter (test with string and number values)
  - Notes: `Tabs` is generic (`<T extends Key>`) — test with concrete types.

- [x] Task 2.1.4: Test `Select` component
  - File: `src/ui/select.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders with placeholder when value is null
    - Dropdown is closed by default
    - Clicking opens dropdown, shows all options
    - Selecting an option fires `onChange` and closes dropdown
    - Clicking outside closes dropdown
    - `disabled` prop prevents interaction
    - `direction` prop works (up/down)
  - Notes: Uses `useState` internally for `isOpen` and a `mousedown` event listener for outside clicks. Use `userEvent.click()` for interactions.

#### Story 2.2: Custom Hook Tests

- [x] Task 2.2.1: Test `useHumanizedTime` hook
  - File: `src/services/hooks.test.tsx` — **NEW**
  - Action: Write tests using `renderHook` from `@testing-library/react`:
    - Given timestamp within 30 seconds → returns `"Just now"`
    - Given timestamp > 1 week ago → returns locale date string
    - Given timestamp between 30s and 1 week → returns relative time string (e.g. `"5 minutes ago"`)
    - Hook updates on interval (use `vi.useFakeTimers()` to advance time)
  - Notes: Mock `Date.now()` with `vi.setSystemTime()` for deterministic results. Import `renderHook` from `@testing-library/react`. Remember to call `vi.useRealTimers()` in `afterEach`.

- [x] Task 2.2.2: Test `useAsync` hook
  - File: `src/services/hooks.test.tsx` (append to same file)
  - Action: Write tests:
    - Calls async function on mount
    - Catches and logs errors without throwing
    - Re-runs when deps change
  - Notes: Use `vi.fn()` returning a resolved/rejected promise. Use `console.log` spy to verify error handling.

#### Story 2.3: Composite Component Test

- [x] Task 2.3.1: Test `SnapPicker` component
  - File: `src/components/snap-picker.test.tsx` — **NEW**
  - Action: Write tests:
    - Renders all SNAP_OPTIONS (Quarter, Eighth, Sixteenth, Thirty-Second)
    - Current value is displayed
    - Changing selection fires `onChange` with correct `NoteDuration` value
  - Notes: `SnapPicker` wraps `Select` — test the composition, not `Select` internals. Import `NoteDuration` from `../../services/types`.

### Phase 3: Tooling Modernization & Guardrails

#### Story 3.1: Modernize ESLint Config

- [x] Task 3.1.1: Install unified `typescript-eslint` v8 package
  - File: `package.json`
  - Action: Install `typescript-eslint` (unified package). Install `eslint-config-prettier`. Remove `@typescript-eslint/eslint-plugin` and `@typescript-eslint/parser` from devDependencies (they're bundled in the unified package now).
  - Notes: The unified `typescript-eslint` package re-exports the plugin and parser internally.

- [x] Task 3.1.2: Rewrite ESLint config to unified API
  - File: `eslint.config.js`
  - Action: Replace current config with `typescript-eslint` v8 unified API:

    ```js
    import tseslint from "typescript-eslint";
    import reactHooks from "eslint-plugin-react-hooks";
    import reactRefresh from "eslint-plugin-react-refresh";
    import jsxA11y from "eslint-plugin-jsx-a11y";
    import eslintConfigPrettier from "eslint-config-prettier";

    export default tseslint.config(
      ...tseslint.configs.recommended,
      {
        files: ["**/*.{js,jsx,ts,tsx}"],
        plugins: {
          "react-hooks": reactHooks,
          "react-refresh": reactRefresh,
          "jsx-a11y": jsxA11y,
        },
        rules: {
          ...reactHooks.configs.recommended.rules,
          ...jsxA11y.configs.recommended.rules,
          "react-refresh/only-export-components": [
            "warn",
            { allowConstantExport: true },
          ],
          "@typescript-eslint/no-unused-vars": [
            "error",
            { argsIgnorePattern: "^_" },
          ],
          "@typescript-eslint/explicit-function-return-type": "off",
          "@typescript-eslint/explicit-module-boundary-types": "off",
          "@typescript-eslint/no-explicit-any": "warn",
          "no-undef": "off",
        },
      },
      eslintConfigPrettier,
      { ignores: ["dist/**", "node_modules/**", "src-tauri/**"] },
    );
    ```

  - Notes: The `tseslint.config(...)` wrapper replaces the raw array export. `eslintConfigPrettier` goes last to disable ESLint rules that conflict with Prettier formatting (e.g. `indent`, `quotes`, `semi`). **Important:** Adding `eslint-config-prettier` may silently disable some rules that were previously active. After migration, run `npx eslint-config-prettier 'src/**/*.{ts,tsx}'` to list which rules are being turned off. Verify none of those rules were relied upon for logic checks (most are purely stylistic). Removes the manual `parser`/`parserOptions` — typescript-eslint handles this internally.

- [x] Task 3.1.3: Verify ESLint still passes
  - Action: Run `npm run lint` and verify zero new errors introduced by the config migration.
  - Notes: This is a refactor — behavior should be identical. If new errors appear, they're likely rules that were silently not applied before.

#### Story 3.2: Pre-commit Hooks

- [x] Task 3.2.1: Install lint-staged and simple-git-hooks
  - File: `package.json`
  - Action: Add devDependencies: `lint-staged`, `simple-git-hooks`. Add `"simple-git-hooks"` and `"lint-staged"` config sections to `package.json`:
    ```json
    "simple-git-hooks": {
      "pre-commit": "npx lint-staged"
    },
    "lint-staged": {
      "*.{ts,tsx}": ["eslint --fix", "prettier --write"],
      "*.{json,md,css,html}": ["prettier --write"]
    }
    ```
  - Notes: After install, run `npx simple-git-hooks` once to register the git hook. Add a `"prepare": "simple-git-hooks"` script so hooks are set up after `npm install`. **CI guard:** The `prepare` script will error in environments without a `.git` directory (e.g. CI containers, Docker builds). Wrap the script to handle this gracefully:
    ```json
    "prepare": "simple-git-hooks || true"
    ```
    Alternatively, use `"prepare": "npx simple-git-hooks 2>/dev/null || true"` to suppress error output.

#### Story 3.3: Coverage Thresholds

- [x] Task 3.3.1: Set initial coverage thresholds
  - File: `vite.config.ts`
  - Action: Add `thresholds` to the `test.coverage` config:
    ```ts
    thresholds: {
      statements: 5,
      branches: 5,
      functions: 5,
      lines: 5,
    }
    ```
  - Notes: Start intentionally low. These represent a ratchet — only increase, never decrease. After Phase 2 tests are in, review actual coverage and bump thresholds to match.

#### Story 3.4: Test Pattern Documentation

- [x] Task 3.4.1: Create `TESTING.md`
  - File: `TESTING.md` — **NEW**
  - Action: Document all test conventions established in Phases 1-2:
    - Test file naming: `*.test.ts` / `*.test.tsx` colocated next to the source file (not in `__tests__/` subdirectories)
    - Test runner: Vitest with happy-dom environment
    - Component testing: React Testing Library patterns (render, screen, userEvent)
    - Hook testing: `renderHook` pattern
    - Mocking Tauri IPC: how to use the `@tauri-apps/api/core` mock
    - Rust testing: `#[cfg(test)]` conventions, test fixture patterns
    - Coverage: how to run, current thresholds
    - Example test for each category (pure function, component, hook, Rust)
  - Notes: This file is specifically for AI agents to reference when writing new tests. Keep it concise and pattern-focused, not tutorial-style.

### Phase 4: E2E Smoke Tests — DEFERRED

> **Deferred 2026-03-04:** `tauri-driver` v2 only supports Linux. This project targets macOS only at present. All Phase 4 work has been reverted (`tests/e2e/` removed, WDIO packages uninstalled). Revisit if a Linux CI pipeline is added or when `tauri-driver` gains macOS support.

#### Story 4.1: WebdriverIO + tauri-driver Setup

- [ ] Task 4.1.1: Install `tauri-driver`
  - Action: Run `cargo install tauri-driver` to install the Tauri WebDriver bridge binary.
  - Notes: This is a system-level install, not a project dependency. Requires a built app binary to test against.

- [ ] Task 4.1.2: Install WebdriverIO dependencies
  - File: `package.json`
  - Action: Add devDependencies: `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework`, `@wdio/spec-reporter`
  - Notes: These are dev-only. Consider putting E2E deps in a separate `tests/e2e/package.json` if they bloat install time. E2E test files use Mocha globals (`describe`, `it`) which conflict with Vitest globals — ensure E2E files are excluded from Vitest's `include` pattern (already handled by `src/**/*.test.{ts,tsx}` not matching `tests/e2e/`). Add `tests/e2e/tsconfig.json` with Mocha types:
    ```json
    {
      "extends": "../../tsconfig.json",
      "compilerOptions": { "types": ["@wdio/mocha-framework", "node"] },
      "include": ["./**/*.ts"]
    }
    ```

- [ ] Task 4.1.3: Create WebdriverIO config
  - File: `tests/e2e/wdio.conf.ts` — **NEW**
  - Action: Create WDIO config that:
    - Uses `tauri-driver` as the WebDriver backend (port 4444)
    - Points to the built app binary — on macOS this is inside the `.app` bundle: `src-tauri/target/release/bundle/macos/Solo Compose.app/Contents/MacOS/Solo Compose` (or `target/debug/Solo Compose` for debug). Use an environment variable or auto-detection to resolve the correct path per platform.
    - Sets reasonable timeouts (app startup can be slow)
    - Logs to `tests/e2e/logs/`
  - Notes: The config should start `tauri-driver` as a child process in `onPrepare` and kill it in `onComplete`. **Important:** Verify `tauri-driver` compatibility with Tauri v2 + WKWebView on macOS before implementation — as of writing, `tauri-driver` v2 support may require the latest release. Test locally with `tauri-driver --port 4444` against a debug build first. See Tauri v2 testing docs for the exact setup.

- [ ] Task 4.1.4: Add E2E test scripts
  - File: `package.json`
  - Action: Add scripts: `"test:e2e": "wdio tests/e2e/wdio.conf.ts"`
  - Notes: E2E tests are run separately from unit tests — they require a built binary.

#### Story 4.2: First Smoke Test

- [ ] Task 4.2.1: Write app launch smoke test
  - File: `tests/e2e/app-launch.test.ts` — **NEW**
  - Action: Write a minimal smoke test:
    - App window opens successfully
    - Title bar is present
    - App doesn't crash within 5 seconds
    - Basic DOM element from the main window is queryable
  - Notes: This is the bare minimum — proves the E2E pipeline works. More tests can be added incrementally.

### Acceptance Criteria

#### Phase 1

- [x] AC 1.1: Given a fresh clone and `npm install`, when `npm test` is run, then Vitest executes and all tests pass with zero failures.
- [x] AC 1.2: Given the Vitest config, when `npm run test:coverage` is run, then a coverage report is generated showing coverage for `src/**/*.{ts,tsx}` files.
- [x] AC 1.3: Given `pitch-utils.test.ts`, when tests run, then all 5 exported functions are tested with correct assertions for all `Accidental` enum values and pitch calculations.
- [x] AC 1.4: Given `utils.test.ts`, when tests run, then `parseTimeSignature` is tested with valid inputs (`"4/4"`, `"6/8"`, `"3/4"`), invalid beat types (`"4/3"`), and malformed strings (`""`, `"abc"`, `"4/"`).
- [x] AC 1.5: Given `utils.test.ts`, when tests run, then `noteDurationToTicks` is tested for all 8 `NoteDuration` values, verifying correct tick counts (Whole=128 through HudredTwentyEighth=1).
- [x] AC 1.12: Given `utils.test.ts`, when tests run, then `desaturateColor` is tested to verify it returns a valid hex string, that factors `(1, 1)` return a near-identical color, and factors `(0, 0.5)` produce a desaturated result.
- [x] AC 1.13: Given `utils.test.ts`, when tests run, then `lightenColor` is tested to verify it returns a valid hex string, that factor `0` returns the same color, and factor `1` returns near-white.
- [x] AC 1.6: Given `sequencer-utils.test.ts`, when tests run in happy-dom, then `getSequencerClickTarget` correctly identifies elements with `data-tone-id` attribute and returns `null` for elements without it.
- [x] AC 1.7: Given the Tauri mock at `src/__mocks__/@tauri-apps/api/core.ts`, when any test imports `invoke` from `@tauri-apps/api/core`, then the mock `vi.fn()` is used instead of the real Tauri API.
- [x] AC 1.8: Given `cargo test` run in `src-tauri/`, when tests execute, then `duration.rs` tests pass for `from_ticks`, `to_ticks`, `half`, `to_modifier`, and `spacing_ratio`.
- [x] AC 1.9: Given `cargo test` run in `src-tauri/`, when tests execute, then `converter.rs` tests verify `spaces_to_px` and `px_to_spaces` are mathematical inverses within f32 precision.
- [x] AC 1.10: Given `cargo test` run in `src-tauri/`, when tests execute, then `get_measures.rs` tests verify correct measure boundary calculation for 4/4 and time signature changes.
- [x] AC 1.11: Given `cargo test` run in `src-tauri/`, when tests execute, then `entries/mod.rs` tests verify `Entry::id()`, `Entry::tick()`, and `Entry::set_tick()` for all 5 entry variants.

#### Phase 2

- [x] AC 2.1: Given `checkbox.test.tsx`, when tests run, then `Checkbox` component renders correctly for both `value=true` and `value=false`, and clicking fires `onChange` with toggled value.
- [x] AC 2.2: Given `switch.test.tsx`, when tests run, then `Switch` component has correct `role="switch"`, `aria-checked`, and `aria-label` attributes, and clicking fires `onChange`.
- [x] AC 2.3: Given `tabs.test.tsx`, when tests run, then all tab options render, the selected tab has active styling, and clicking fires `onChange` with the correct value.
- [x] AC 2.4: Given `select.test.tsx`, when tests run, then dropdown opens on click, option selection fires `onChange` and closes the dropdown, and clicking outside closes it.
- [x] AC 2.5: Given `hooks.test.tsx`, when `useHumanizedTime` is tested with fake timers, then it returns `"Just now"` for timestamps within 30 seconds, a locale date for timestamps > 1 week old, and relative time for timestamps in between.
- [x] AC 2.6: Given `hooks.test.tsx`, when `useAsync` is tested, then it calls the async function on mount and catches rejected promises without throwing.
- [x] AC 2.7: Given `snap-picker.test.tsx`, when tests run, then `SnapPicker` renders all 4 snap options and selecting one fires `onChange` with the correct `NoteDuration` value.

#### Phase 3

- [x] AC 3.1: Given the modernized `eslint.config.js`, when `npm run lint` is run, then zero new errors are introduced compared to the pre-migration baseline. Note: `eslint-config-prettier` intentionally disables formatting-related rules — verify with `npx eslint-config-prettier` that only stylistic (not logic) rules are affected.
- [x] AC 3.2: Given `eslint.config.js`, when inspected, then it uses `typescript-eslint` unified import and `tseslint.config(...)` API, with `eslint-config-prettier` applied last.
- [x] AC 3.3: Given a staged `.ts` file with a lint error, when `git commit` is run, then `lint-staged` runs ESLint and Prettier on the staged file, and the commit is blocked if lint errors remain.
- [x] AC 3.4: Given `TESTING.md` exists at project root, when an AI agent reads it, then it contains clear examples for: pure function tests, component tests, hook tests, Tauri mock usage, and Rust `#[cfg(test)]` patterns.
- [x] AC 3.5: Given coverage thresholds of 5% for all metrics, when `npm run test:coverage` is run and coverage drops below thresholds, then the test run fails.

#### Phase 4

- [ ] AC 4.1 _(deferred)_: Given `tauri-driver` installed and app binary built, when `npm run test:e2e` is run, then WebdriverIO starts `tauri-driver`, launches the app, and the smoke test passes.
- [ ] AC 4.2 _(deferred)_: Given `app-launch.test.ts`, when the test runs, then it verifies the app window opens, a DOM element is queryable, and the app doesn't crash within 5 seconds.

## Additional Context

### Dependencies

**New npm devDependencies (Phase 1-2):**

- `vitest` — test runner
- `@vitest/coverage-v8` — coverage provider
- `happy-dom` — DOM environment
- `@testing-library/react` — component testing utilities
- `@testing-library/user-event` — realistic user interaction simulation
- `@testing-library/jest-dom` — custom DOM matchers (`.toBeVisible()`, `.toHaveAttribute()`, etc.)

**New npm devDependencies (Phase 3):**

- `typescript-eslint` — unified v8 package (replaces `@typescript-eslint/eslint-plugin` + `@typescript-eslint/parser`)
- `eslint-config-prettier` — disables ESLint rules that conflict with Prettier
- `lint-staged` — run linters on staged files only
- `simple-git-hooks` — lightweight git hooks manager

**New npm devDependencies (Phase 4) — DEFERRED:**

> Phase 4 is deferred. `tauri-driver` v2 is Linux-only; project currently targets macOS only. These packages have not been installed.

**Rust (no new Cargo dependencies needed):** Tests use `#[cfg(test)]` with existing crate dependencies. No additional test framework required — Rust's built-in test harness is sufficient.

### Testing Strategy

Phase 1 → Phase 2 → Phase 3 → Phase 4, each building on the previous.

- **Phase 1** establishes the test runner (Vitest), Tauri mock, and first tests against pure functions (TS: `pitch-utils`, `utils`; Rust: `duration`, `converter`, `get_measures`, `entries`). Sets the pattern.
- **Phase 2** adds component tests (checkbox, switch, tabs, select) and hook tests (`useHumanizedTime`, `useAsync`). Establishes React Testing Library patterns.
- **Phase 3** modernizes ESLint, adds pre-commit hooks, sets initial coverage thresholds, and documents all test patterns in `TESTING.md` for future AI agent use.
- **Phase 4** sets up WebdriverIO + `tauri-driver` for thin E2E smoke tests (app launch, basic flow creation). Architecture only — not comprehensive coverage.

### Notes

- Research brief: `_bmad-output/planning-artifacts/research/technical-testing-and-tooling-research-2026-03-04.md`
- Project context: `docs/project-context.md`
- The `NoteDuration` enum has a typo (`HudredTwentyEighth` instead of `HundredTwentyEighth`) — this is intentional/legacy, do NOT rename it as it would break serialization compatibility
- `popovers.ts` (627 lines) contains parsers like `parseBarlinePopover` that ARE testable but depend on `actions` — would need mocking. Defer to after Phase 2.
- `mm.ts` measures physical mm-to-px via DOM — testable with happy-dom but the value is marginal (it just creates and measures a div)
