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
- Vite 7.3.1 — multi-page app with 9 HTML entry points
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
- Tuplets follow one backend-owned semantic model across persistence, parsing, rendering, playback, and editing; do not add frontend-owned parallel tuplet truth or subsystem-local reinterpretations of ownership, boundaries, or continuity
- Preserve the tuplet timing invariant: host geometry stays compressed to the denominator span, but any crossing tone still consumes its full literal duration on the canonical expanded timeline
- Tuplet spacing is backend-owned and shared: collect overlapping anchored spans into one governed region, build one temporary rhythmic grid there at an LCM-derived resolution across the region and participating local lengths, include the ambient host-owned note-spacing slice in that same grid, aggregate all note-spacing demand there, then collapse the unified result back into both host spacing and anchored local spacing so simultaneous rhythms align instead of stretching independently
- Dual-index storage pattern: `by_tick: FxHashMap<Tick, Vec<UniqueId>>` + `by_id: FxHashMap<UniqueId, Entry>`

### Framework-Specific Rules

#### React / Tauri Architecture

- All components are `FC<Props>` — no class components
- State source of truth is Rust `AppContext` behind `Mutex` — FE is a display layer only
- 4 React contexts: `AppContext`, `AudioContext`, `PluginInventoryContext`, `TransportContext` — each with a typed hook that throws outside its provider; `PluginInventoryContext` is the backend-owned machine-local VST3 inventory channel, including persisted `editor_host_validation` diagnostics from real maintainer host attempts
- Slot-owned live plugin sessions are backend-only runtime objects keyed by `slot_id`; they own the real initialized VST instance for prepared slots, restore any portable `PlaybackSlotTarget.plugin_state` payload before editor-open, and remain the sole source from which `PlaybackActivationRuntime` readiness is derived
- `src-tauri/src/playback/engine.rs` is the backend-owned realtime playback-plane seam: it tracks render-active slot sessions, consumes backend-owned event queues, and processes silent-safe blocks on the persistent audio callback so editor-open and audition can share the same runtime path without creating a second owner
- Backend route truth now also owns score-context mix honesty: `resolved_playback_route_by_instrument_id` may expose `supported_mix_controls`, and the current MVP treats `volume`, `mute`, `solo`, and `pan` as honest on ready direct-plugin root routes only because the same backend seam now proves them end-to-end
- State pushed to FE via Tauri events (`soloc://state/general`, `soloc://state/audio`, `soloc://state/plugin_inventory`)
- Keep plugin bundle validity separate from bounded native-editor posture: `validation_failure` is for bundle/discovery failures, while editor-host policy blocks and real host-attempt diagnostics belong in `editor_host_validation`
- Transport polling uses `requestAnimationFrame` loop — no events during playback to keep transport thread silent
- Multi-window app: 9 HTML entry points, each mounts its own `AppStateProvider` in `boot.tsx`; the Sequence-menu `Plugin Manager...` window is the canonical app-scoped inventory browser, while the backend-owned `plugin-host-poc` window remains the bounded macOS native-editor handoff used from the Sequence score-context rack surface and the Sequence `Instruments` bottom panel is the score-context current-rack surface
- The Expression Map workflow is likewise app-scoped and window-owned: both the Sequence menu entry and the instrument-row launch affordance reuse one dedicated `expression-map` window, the left pane owns reusable-map search, selection, create, and delete behavior, and the right pane owns selected-map authoring, rename, and metadata edits with immediate auto-save rather than a separate save/apply flow
- All mutations go through the `actions` object in `actions.ts` via `invoke()` — never manipulate state directly
- `ui.flow_id` is the one authoritative active-flow slot for score, Sequence, miniature piano rolls, and transport-derived tempo state; deliberate score-object selection may update it, but pan/zoom/hover/miss-to-clear must not
- Shipped MVP tuplets remain triplet-first and popover-first: create the container through the existing popover flow, then edit owned tones through the shared backend seams rather than inventing a second tuplet-construction workflow in React
- The shared top bar is reserved for universal controls (tabs, layout, transport, tools); workflow-specific single-flow switching belongs inside the owning workflow surface such as Sequence
- Keep backend playback-route truth richer than the warning UI. `resolved_playback_route_by_instrument_id` may honestly distinguish `Unrouted`, empty-slot, missing-plugin, and other non-ready states, but the Sequence route picker must stay intentionally minimal: no warning badge for plain `--` or `N. --`, and compact warning chrome only for broken assigned plugin routes that need repair.
- Keep compact score-context playback controls legible without color alone. Icon-only transport buttons, Sequence panel tabs, and bounded mix toggles should expose explicit labels/titles and semantic pressed-state cues instead of relying on accent color as the only state signal.
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
- `src-tauri/src/playback/` — audio, mixer, plugin inventory, runtime activation, transport, VST3 support

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
- Persist only stable expression-map references in `.soloc` route data; reusable expression-map definitions remain app-scoped, machine-local assets outside project payloads
- Expression-map library metadata now expects explicit persisted fields such as `creator` and `version`; do not add compatibility defaults for omitted metadata unless a later migration plan explicitly restores that requirement
- Persisted-data migrations are authored in `src-tauri/src/migrations/next.rs` during development, promoted to a concrete version during `prepare-release:*`, and reseeded after `finalize-release`
- Current tuplet migration stance: older non-tuplet releases do not need a dedicated tuplet migration because they produced no tuplet-bearing payloads; add one only if a released older format genuinely needs transformation
- Tuplet regressions should stay close to the owning seam when local tests are enough: parser scenarios near parser logic, renderer regressions near draw/hit-test code, playback regressions near transport, and persistence regressions near file/load-save seams

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
- Supported `transport_play` is now a backend-owned audible score path on the product surface: it advances tick and tempo state whenever the active flow and tick are valid, snapshots supported direct-plugin root targets once at play-start, and submits score-derived note events into the continuous render engine only for those supported routes, without any sampler or SCVI fallback path
- Transport clocking must stay backend-owned and bounded: do not revert to one naive full-tick sleep loop. Keep the short queue window, short wait quantum, and explicit lead-frame scheduling model so the transport thread can absorb moderate scheduler jitter without rebuilding timing truth in the frontend.
- Pause is still backend-owned transport cleanup, not a second pause mode: clear the transport render target, flush active audio, join any prior transport thread, and republish truthful tempo for the paused tick before the next play or resume.
- Seek is part of that same backend-owned transport contract: active seek must clamp against the score's active flow, clear queued transport events plus active transport-owned notes from the old timeline, and resume only from a fresh preparation snapshot, while stopped seek must update truthful tick and tempo without starting playback.
- Treat stop, pause, and transport teardown as hard-reset paths over the same engine-owned transport work: clear queued transport events, retire active transport notes deterministically, and never let stale transport work survive into the next play-start.
- Blocked `transport_play` attempts must stay explicit backend no-ops: clear stale stop-signal state, leave transport stopped, and preserve backend route truth rather than inventing fallback playback, frontend-owned readiness, or compatibility-residue transport state.
- The continuous render engine now owns live slot processing on the audio callback. Transport may submit score-derived engine work and manage scheduling, but it must not take ownership of OS output-stream lifetime or become the canonical live processor seam.
- Keep score-owned mix intent and live mix capability separate: persisted `volume`, `pan`, `mute`, and `solo` remain the intent source, but frontend controls must follow backend-exposed route capability rather than assuming every persisted field is live and audible.
- Supported live mix edits must update the actual engine mix state used by in-flight playback, not only a transport-side mirror; solo changes in particular must recompute the effective mix truth for every supported route.
- Treat active playback as a bounded snapshot contract. Only `volume`, `pan`, `mute`, and `solo` on already-ready direct-plugin root routes are immediate in the current pass; tone-content edits, notation edits, and playback-topology edits must persist score intent immediately but defer audibility until pause, seek restart, stopped play-start, natural-end replay, or another transport-owned reset boundary.
- Audio device and sample-rate rebuilds are transport-owned reset events: stop transport playback first, clear engine transport work, republish tick and tempo from current score truth, and require the next play-start to rebuild its preparation snapshot at the new stream settings.
- Audio device and sample-rate rebuilds are transport-owned reset events: stop transport playback first, clear engine transport work, republish tick and tempo from current score truth, rebase backend queue timing state such as `last_queued_tick`, and require the next play-start to rebuild its preparation snapshot at the new stream settings.
- `prepare_supported_playback(...)` now treats `supported_targets` as the canonical public transport snapshot. If a caller needs one instrument-specific prepared target, use `supported_target_for_instrument(...)`; do not add a new public single-target compatibility field.
- Epic 4 audible-playback work now builds on the completed Story 1.3 single-instrument-VST-instance trust gate and the Story 1.4 through 1.6 slot-owned runtime-session foundation: one routed direct plugin route into a VST instance that hosts one playable instrument at a time, transport-readiness-only status at the gate itself, no legacy fallback ownership, slot-owned runtime sessions rebuilt from portable saved intent rather than persisted live runtime state, and completed maintainer manual proof on that architecture. Do not treat that completed gate as evidence of HALion-style multi-slot or other multi-timbral readiness, multi-instrument readiness, or already-shipped first-audible-playback behavior.
- Playback is another consumer of the shared tuplet model, not a second semantics engine. Transport code may implement scheduling mechanics, but it must not locally infer tuplet ownership, boundary meaning, projection, or continuity; if playback exposes a gap, extend the shared semantic seam instead of patching transport in isolation
- Nested tuplets, broader arbitrary ratios, and generalized sequencer-native tuplet editing remain explicit post-MVP work. Do not normalize them into current rules or documentation.
- Legacy sampler and SCVI runtime ownership have been removed from the product path. Do not reintroduce them as fallback playback, audition, or hidden readiness logic.
- Native editor opens from score-context rack surfaces must resolve through slot identity as well as bundle path so same-bundle slots remain distinct backend-owned targets during close, reopen, and teardown, and so editor attach remains a transient view over the slot-owned runtime session instead of replacing that ownership.

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

Last Updated: 2026-04-04
