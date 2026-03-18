---
project_name: "compose"
user_name: "Aled"
date: "2026-03-04"
sections_completed:
  [
    "technology_stack",
    "language_rules",
    "framework_rules",
    "quality_rules",
    "workflow_rules",
    "anti_patterns",
  ]
status: "complete"
rule_count: 52
optimized_for_llm: true
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

### Frontend

- React 19.2.4 — functional components, hooks only
- TypeScript 5.9.3 — strict mode enabled (`noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`)
- Vite 7.3.1 — multi-page app with 8 HTML entry points
- Tailwind CSS 4.2.0 — via `@tailwindcss/postcss` plugin
- @mdi/js 7.4.47 + @mdi/react 1.6.1 — Material Design Icons
- classnames 2.5.1 — conditional CSS class composition
- big.js 7.0.1 — precision arithmetic
- SortableJS 1.15.7 — drag-and-drop reordering

### Backend (Rust / Tauri)

- Tauri 2.10.1 — `protocol-asset`, `macos-private-api` features
- Rust 2021 edition — release: LTO fat, codegen-units=1, opt-level=3
- skia-safe 0.93.0 (Metal) — direct native surface rendering via CAMetalLayer
- cpal 0.17.3 — audio device I/O
- symphonia 0.5.5 (FLAC) + rubato 1.0.1 — audio decoding & sample-rate conversion
- serde 1.0.228 + serde_repr 0.1.20 — numeric enum serialization matching TS enums
- rustc-hash 2.1.1 (FxHashMap) — used throughout instead of std HashMap
- rayon 1.10 — parallel iteration in the parse phase
- sled 0.34.7 — embedded key-value store

### Tooling

- ESLint 9.39.2 — flat config format (`eslint.config.js`)
- Prettier 3.8.1
- No test framework configured

## Critical Implementation Rules

### Language-Specific Rules

#### Core Principle: Performance

- Rust was chosen specifically for performance — every backend decision should prioritize speed
- Use `FxHashMap` (rustc-hash) everywhere, never `std::collections::HashMap` — it is measurably faster for our key types
- `rayon` parallel iterators used in parse phase for tone offsets, horizontal spacing, written durations
- Release profile: LTO fat, codegen-units=1, opt-level=3, stripped — maximum runtime performance

#### TypeScript

- All enums are numeric, mirroring Rust `serde_repr` enums — same variant names and integer values
- Type properties use `snake_case` matching Rust struct fields (e.g. `flow_id`, `layout_type`, `by_id`) — NOT camelCase
- `UniqueId = number`, `Tick = number`, `Ticks = number` — type aliases for semantic clarity
- All FE→BE calls go through `invoke()` from `@tauri-apps/api/core` with snake_case parameter names
- Rust `FxHashMap<K, V>` serializes to `Record<K, V>` in TypeScript
- No `any` — ESLint warns on explicit any usage
- Unused function args must be prefixed with `_`

#### Rust

- All Tauri commands: `#[tauri::command(rename_all = "snake_case")]`
- `AppContext` behind `Mutex` — lock, mutate, call `emit()` to push state to all FE windows
- Score uses `Space` unit system (staff spaces) — pixel conversion via `Converter::spaces_to_px()`
- `Entry` is a tagged enum (`#[serde(tag = "type")]`) with 7 variants: Barline, Clef, KeySignature, Tempo, TimeSignature, Tone, Tuplet
- `Tuplet` persists ratio metadata as `note_count`, `in_space_of`, `base_note_value`, `groupings`, and child `entries`; MVP child content is intentionally single-level, so nested tuplets are rejected by the owning backend type
- Dual-index storage pattern: `by_tick: FxHashMap<Tick, Vec<UniqueId>>` + `by_id: FxHashMap<UniqueId, Entry>`

### Framework-Specific Rules

#### React / Tauri Architecture

- All components are `FC<Props>` — no class components
- State source of truth is Rust `AppContext` behind `Mutex` — FE is a display layer only
- 4 React contexts: `AppContext`, `AudioContext`, `SamplerContext`, `TransportContext` — each with a typed hook that throws outside its provider
- State pushed to FE via Tauri events (`soloc://state/general`, `soloc://state/audio`, `soloc://state/sampler`)
- Transport polling uses `requestAnimationFrame` loop — no events during playback to keep transport thread silent
- Multi-window app: 8 HTML entry points, each mounts its own `AppStateProvider` in `boot.tsx`
- All mutations go through the `actions` object in `actions.ts` via `invoke()` — never manipulate state directly
- `ui.flow_id` is the one authoritative active-flow slot for score, Sequence, miniature piano rolls, and transport-derived tempo state; deliberate score-object selection may update it, but pan/zoom/hover/miss-to-clear must not
- The shared top bar is reserved for universal controls (tabs, layout, transport, tools); workflow-specific single-flow switching belongs inside the owning workflow surface such as Sequence
- UI styling: Tailwind utility classes + `classnames` — dark theme (neutral-900 base, sky-500 accents, transparent backgrounds)

#### Score Render Pipeline

The score page is rendered natively by Rust/Skia/Metal — all other UI (panels, buttons, inputs, toolbars) lives in the React/HTML layer.

1. **Score data** → Tracks of `Entry` variants (Tone, Clef, KeySignature, Tempo, TimeSignature, Barline, Tuplet) stored in dual-index `FxHashMap`s
2. **Flow preparation phase** (`instructions/render_plan.rs`) → derives ordered visible flow inputs from `layout.flow_ids` while preserving `score.flows.order`; the render path emits `render-timing-flow-preparation` plus `render-timing-flow-preparation-details` diagnostics before parse/draw work begins
3. **Per-flow parse/draw phase** (`instructions/parse/`, `instructions/draw/`) → each visible flow segment is parsed and drawn independently into `Vec<DrawInstruction>` plus `Vec<HitTarget>` using renderer-space `Space` units and px conversion via `Converter`
4. **Flow composition phase** (`instructions/mod.rs`) → Rust composes every visible flow segment into one shared page surface in `score.flows.order`, offsets each later segment by a fixed renderer-space gap that guarantees at least 20 CSS pixels at normal zoom, paints a separate white native page box for each composed flow so the gradient backdrop remains visible in the boundary gap, preserves each flow's native page-start chrome, and keeps combined hit targets in shared renderer space
5. **Native render** → `MetalRenderer` paints the score page only via Skia onto a `CAMetalLayer` attached to the native `NSView` — this is not the app UI, just the score sheet
6. **FE interaction** → a transparent `<div>` overlays the native score surface, reports bounds/resize, handles pan/zoom/click, and delegates hit-testing to Rust via `invoke("renderer_hit_test")`

- Draw instructions are cached on `RendererContext` and re-rendered on pan/zoom without recalculation
- Recalculation only triggered by score mutations (via `emit()` flow)
- The flow-preparation diagnostics contract is maintainer-facing and Rust-owned: valid recalculation passes report `render_mode`, prepared/composed/recalculated flow ids and counts, plus scope labels such as `single_flow`, `multiple_flows`, or `no_visible_flows`; reset paths continue to emit null-safe payloads so the debugger can show cleared/no-render state without inventing a parallel source of truth.
- Performance characterization for multi-flow work stays on that contract. Use the debugger timing panel plus `render-timing-flow-preparation-details` / `render-timing-surface-details` to compare the 3-instrument, 30-bar single-flow baseline against representative multi-flow render and local-edit passes; do not introduce a separate benchmark-only state model.
- The current release-like `npm run perf:render` result is the active representative single-flow baseline. Trigger a regression warning when that baseline scenario reaches or exceeds 2ms total median time, and reevaluate that threshold as performance-sensitive features land.
- The composed `renderer.width` from the shared surface remains the single authoritative horizontal extent for native pan clamping, while the debugger timing panel can expose shared-surface versus viewport dimensions plus per-page box offsets for multi-flow reachability checks without creating a frontend-owned scroll model.
- Horizontal spacing slots are for **flow spacing only** — they are not a generic anchoring mechanism for every marking.
- Prefer reusing an existing slot position when an element should align with current notation but should not push later notation horizontally.
- As a general rule, avoid adding new slots unless the element genuinely changes left-to-right score flow; tempo markings are the current reference case for this rule.

### Code Quality & Style Rules

#### File & Naming Conventions

- All TS/TSX files use **kebab-case** filenames (e.g. `icon-button.tsx`, `pitch-utils.ts`)
- All Rust files use **snake_case** filenames (e.g. `get_written_durations.rs`, `draw_notehead.rs`)
- React components use **PascalCase** exports (e.g. `IconButton`, `AppBar`)
- One component per file — each file exports a single `FC` so Fast Refresh stays reliable
- Barrel exports via `index.ts` for module folders (e.g. `state/index.ts`)

#### Code Organization

- `src/ui/` — reusable UI primitives (buttons, inputs, select, tabs, checkbox)
- `src/components/` — shared app-level components (app-bar, drag-scroll, snap-picker)
- `src/windows/<name>/` — per-window code with `boot.tsx`, `index.tsx`, optional `screens/`, `components/`, `hooks/`, `utils/`
- `src/services/` — business logic: `actions.ts`, `types.ts`, `utils.ts`, `state/` (provider + context)
- `src-tauri/src/score/` — score data model (players, instruments, flows, tracks, entries, layouts, engraves, components)
- `src-tauri/src/instructions/` — parse/ + draw/ for the render pipeline
- `src-tauri/src/renderer/` — Metal backend, converter, fonts, hit-test
- `src-tauri/src/playback/` — audio, mixer, sampler, transport, SCVI format, streamer

#### Formatting & Linting

- Prettier for TS/TSX (default config, `printWidth: 100` for JSON only)
- ESLint flat config with react-hooks, react-refresh, jsx-a11y, typescript-eslint plugins
- `react-refresh/only-export-components` enforced — don't mix component + non-component exports
- `@typescript-eslint/no-unused-vars` with `argsIgnorePattern: "^_"`
- `no-undef` disabled (TypeScript handles this)
- **Rust: all clippy warnings must be resolved** — treat clippy suggestions as errors, do not suppress with `#[allow(...)]` unless there is a documented justification. The compiler being happy is not enough; clippy must also be clean.

#### CSS / Styling

- Tailwind 4 utility classes inline — no CSS modules or styled-components
- Single global `styles.css` imports Tailwind, sets transparent background, no user-select, hidden scrollbars
- Custom theme font: `--font-bravura: Bravura` (music notation font)
- All `cursor: default` globally — native-feel desktop app, not a web app

### Development Workflow Rules

- Versioning: semver via custom release binary (`npm run prepare-release:{patch|minor|major}` then `npm run finalize-release`)
- Version kept in sync across `package.json`, `Cargo.toml`, and `tauri.conf.json` (all currently 1.25.0)
- Validate before committing: `npm run lint` + `npm run check-types` + `cargo clippy`
- The app saves user projects as `.soloc` files — this is the application's native save format, not a source code or config format
- Persisted-data migrations are authored in `src-tauri/src/migrations/next.rs` during development, promoted to a concrete version during `prepare-release:*`, and reseeded after `finalize-release`

### Critical Don't-Miss Rules

#### Architecture Gotchas

- NEVER render score elements in React/HTML — the score is rendered natively by Rust/Skia/Metal. The FE `Renderer` component is just a transparent overlay for pointer events
- NEVER mutate FE state directly for data that lives in Rust — all state changes must go through `invoke()` in `actions.ts`. **Exceptions:** truly isolated temporary UI state (e.g. local `useState` for hover/open/toggle) or performance-critical cases where round-tripping to Rust would be too slow (e.g. tracking pan coordinates via refs during drag)
- NEVER use `std::collections::HashMap` in Rust — always use `FxHashMap` from `rustc-hash`
- NEVER create React class components — functional `FC<Props>` only
- NEVER use camelCase for type properties that cross the FE↔BE boundary — use `snake_case` to match Rust structs

#### Serialization Gotchas

- Enums are serialized as integers via `serde_repr` — when adding a new enum variant, it MUST be added at the same position in both Rust and TypeScript with matching integer values
- `Entry` uses `#[serde(tag = "type")]` — the `type` field discriminates variants. Always handle all 7 variants when pattern-matching
- Adding a new field to a Rust struct that serializes to FE requires updating the corresponding TypeScript type in `types.ts` — they must stay in sync manually

#### Render Pipeline Gotchas

- All draw coordinates use `Space` units (staff spaces), not pixels — pixel conversion happens at render time via `Converter`
- Parse modules can use `rayon` for parallelism but draw modules push to `RendererContext` sequentially — don't parallelize the draw phase
- `DrawInstruction` coordinates are pre-converted to px when pushed — the Metal backend consumes them as-is
- Hit targets store coordinates in `Space` units — the hit-test function converts click coordinates to spaces before comparison
- Slot additions should be rare: slots are for elements that consume horizontal flow space, not for every visual annotation
- If a marking can align to an existing slot without pushing later notation, draw it relative to that slot instead of adding a new one

#### Multi-Window Gotchas

- Each window has its own `boot.tsx` and mounts its own `AppStateProvider` — state is pushed to ALL windows via `app.emit()`
- When opening a new window, use `WebviewWindow` from `@tauri-apps/api/webviewWindow` — not browser `window.open()`
- Secondary windows (instrument picker, preferences) close on blur via `tauri://blur` event listener

#### Audio Gotchas

- Transport state uses RAF polling during playback — never add event-based transport state updates, it will cause the transport thread to emit during playback
- Playback is another consumer of the shared tuplet model, not a second semantics engine. Transport code may implement scheduling mechanics, but it must not locally infer tuplet ownership, boundary meaning, projection, or continuity; if playback exposes a gap, extend the shared semantic seam instead of patching transport in isolation
- Custom binary instrument format (SCVI) — instruments are compressed/inspected via dedicated Rust CLI binaries, not runtime code

---

## Usage Guidelines

**For AI Agents:**

- Read this file before implementing any code in this project
- Follow ALL rules exactly as documented
- When in doubt, prefer the more restrictive option
- Update this file if new patterns emerge during implementation

**For Humans:**

- Keep this file lean and focused on agent needs
- Update when technology stack or conventions change
- Review periodically — remove rules that become obvious over time

Last Updated: 2026-03-18
