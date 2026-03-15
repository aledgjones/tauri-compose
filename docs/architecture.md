# Solo Compose — Architecture

## Overview

Solo Compose follows a **Rust-owned state with React display layer** architecture. The Rust backend is the single source of truth for all application state. The frontend is a thin UI shell that receives state updates via Tauri events and dispatches mutations via `invoke()` commands.

The score itself is rendered natively by Rust/Skia/Metal — all other UI (panels, buttons, toolbars) lives in the React/HTML layer.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri Shell                               │
│                                                                  │
│  ┌────────────────────────┐    ┌──────────────────────────────┐  │
│  │   React Frontend       │    │       Rust Backend            │  │
│  │                        │    │                               │  │
│  │  ┌──────────────────┐  │    │  ┌─────────────────────────┐ │  │
│  │  │ 8 HTML Windows   │  │    │  │ AppContext (Mutex)       │ │  │
│  │  │  - main          │  │    │  │  ├── Ui                  │ │  │
│  │  │  - debugger      │◄─┼─emit──┤  ├── Score               │ │  │
│  │  │  - inst-picker   │  │    │  │  ├── Preferences         │ │  │
│  │  │  - layout-settings│ │    │  │  └── next_id             │ │  │
│  │  │  - player-settings│ │    │  └─────────────────────────┘ │  │
│  │  │  - plugin-settings│ │    │                               │  │
│  │  │  - preferences   │  │    │  ┌─────────────────────────┐ │  │
│  │  │  - project-settings│ │   │  │ RendererContext (Mutex)  │ │  │
│  │  └──────────────────┘  │    │  │  ├── MetalRenderer       │ │  │
│  │           │             │    │  │  ├── DrawInstructions    │ │  │
│  │       invoke()          │    │  │  ├── HitTargets          │ │  │
│  │           │             │    │  │  └── Converter           │ │  │
│  │           ▼             │    │  └─────────────────────────┘ │  │
│  │  ┌──────────────────┐  │    │                               │  │
│  │  │ actions.ts       │──┼────┤  ┌─────────────────────────┐ │  │
│  │  │ (all mutations)  │  │    │  │ Audio / Sampler / Transport│ │
│  │  └──────────────────┘  │    │  │  ├── AudioContext        │ │  │
│  │                        │    │  │  ├── SamplerContext       │ │  │
│  └────────────────────────┘    │  │  ├── TransportContext     │ │  │
│                                │  │  └── TransportShared (Arc)│ │  │
│                                │  └─────────────────────────┘ │  │
│                                └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## State Management

### Rust Side (Source of Truth)

- **`AppContext`** — Behind `Mutex`, holds `Ui`, `Score`, `Preferences`, `next_id`, `current_file_path`
- **`RendererContext`** — Behind `Mutex`, holds `MetalRenderer`, `DrawInstruction[]`, `HitTarget[]`, `Converter`
- **`AudioContext`** — Behind `Mutex`, holds `cpal` host/device/stream, `MixerHandle`
- **`SamplerContext`** — Behind `Mutex`, holds plugin metadata, `Cache` (decoded instrument data)
- **`TransportContext`** — Behind `Mutex` + `TransportShared` (Arc with atomics for lock-free polling)

### Frontend Side (Display Layer)

Four React contexts mirror the backend state:

| Context            | Event Channel                         | Hook                  | Purpose                           |
| ------------------ | ------------------------------------- | --------------------- | --------------------------------- |
| `AppContext`       | `soloc://state/general`               | `useAppState()`       | UI state, score data, preferences |
| `AudioContext`     | `soloc://state/audio`                 | `useAudioState()`     | Device list, active device        |
| `SamplerContext`   | `soloc://state/sampler`               | `useSamplerState()`   | Plugin list, loaded plugins       |
| `TransportContext` | RAF polling via `transport_get_state` | `useTransportState()` | Playing, tick, tempo              |

### State Flow

1. User interacts with React UI
2. `actions.ts` calls `invoke("command_name", { params })` — all params use `snake_case`
3. Rust command locks `AppContext`, mutates state
4. Rust calls `state.emit()` → serializes state → `app.emit("soloc://state/general")` to ALL windows
5. Each window's `AppStateProvider` receives event via `listen()`, calls `setState()`
6. React re-renders

**Exception:** Transport state uses `requestAnimationFrame` polling during playback to keep the transport thread silent (no `app.emit()` during playback).

### Active-flow ownership

- `ui.flow_id` is the single authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport tempo resolution.
- Explicit Sequence-local flow switching updates `ui.flow_id` directly; the shared top bar does not own a workflow-specific flow selector.
- Deliberate score-object selection may hand off `ui.flow_id` to the selected object's flow through the backend selection seam.
- Passive score interactions such as pan, zoom, hover, and miss-to-clear do not change `ui.flow_id`.

---

## Score Render Pipeline

The score page is rendered natively — NOT in React/HTML.

```
Score Data (Entry variants in FxHashMaps)
         │
         ▼
┌─────────────────────────────────────────────┐
│  PARSE PHASE (19 modules, rayon-parallel)   │
│                                              │
│  get_measures → get_written_durations →      │
│  get_tone_offsets → get_horizontal_spacing → │
│  get_vertical_spacing → get_beams →          │
│  get_stem_directions → get_stem_lengths →    │
│  get_accidentals → get_articulations →       │
│  get_dots → get_shunts → get_barlines →      │
│  get_key_signatures → get_instruments →      │
│  get_vertical_spans → get_max_brackets_width │
│  → get_max_instrument_name_width →           │
│  get_tone_offset_info                        │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│  DRAW PHASE (20 modules, sequential)         │
│                                              │
│  Generates Vec<DrawInstruction> + Vec<HitTarget> │
│  All coordinates in Space units →             │
│  converted to px via Converter               │
│                                              │
│  draw_staves, draw_notehead, draw_stem,      │
│  draw_beam, draw_flag, draw_rest, draw_dot,  │
│  draw_clef, draw_key_signature,              │
│  draw_time_signature, draw_tempo, draw_barline, │
│  draw_accidentals, draw_articulations,       │
│  draw_ties, draw_ledger_lines, draw_names,   │
│  draw_braces, draw_brackets,                 │
│  draw_sub_brackets, draw_systemic_barline    │
└─────────────────────┬───────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────┐
│  NATIVE RENDER (MetalRenderer)               │
│                                              │
│  Skia paints onto CAMetalLayer               │
│  attached to NSView in main window           │
│                                              │
│  Frontend: transparent <div> overlay for     │
│  pointer events → delegates hit-testing      │
│  to Rust via invoke("renderer_hit_test")     │
└─────────────────────────────────────────────┘
```

### Coordinate System

- All score coordinates use **Space** units (staff spaces)
- Pixel conversion: `Converter::spaces_to_px(spaces) = spaces × mm_per_space × px_per_mm`
- Draw instructions are pre-converted to px when pushed to `RendererContext`
- Hit targets store coordinates in Space units — the hit-test converts click coords to spaces

### Flow-wide directives

- `TimeSignature`, `KeySignature`, and `Tempo` entries live on the flow master track.
- `Clef` entries live on stave master tracks.
- Tempo markings do not reserve horizontal spacing; they align to the existing time-signature slot when present, otherwise to the accidental slot, and render once above the top stave in the draw phase.
- Tempo, time-signature, key-signature, and clef markings now emit native renderer hit targets and participate in the shared sequencer selection model as `Selection::Tempo`, `Selection::TimeSignature`, `Selection::KeySignature`, and `Selection::Clef`, so score clicks, highlight state, and popover coordinates stay inside the Rust render pipeline instead of being approximated in React.
- Flow-master time signatures and key signatures render a hit target on every visible stave instance, but each instance maps back to the same logical selection payload; popover anchoring intentionally resolves to the first/top-stave match.
- Hidden tempo, time-signature, key-signature, and clef entries do not emit draw instructions or hit targets, so they remain non-selectable until they are visible again.
- Selected tempo, time-signature, key-signature, and clef markings render with the same sky accent used by selected tones and rests.
- Delete/Backspace handling for these directive selections is centralized in the write-screen keyboard shortcut hook: non-opening directives clear the selection after deletion, tick-0 time signatures remain protected, and tick-0 clef deletion restores the stave's instrument-default opening clef in place.
- Immediate absolute score tempo markings now drive transport tempo during playback and while stopped-state transport position is queried.

### Write-screen properties editing

- The write-screen bottom-panel `Properties` tab now hosts a reusable properties shell that switches by sequencer selection type.
- Directive property panels share common section primitives so tempo, time signature, key signature, and clef selections render with a consistent layout.
- Tempo selections edit through `TempoPropertiesSection` and include a shared `Common` section with the `Hidden` toggle.
- Time-signature selections edit symbol form through `TimeSignaturePropertiesSection`, which only enables `CommonTime` / `SplitCommonTime` for valid 4/4 and 2/2 signatures, and also expose the shared `Hidden` toggle.
- Key-signature selections now route to `KeySignaturePropertiesSection`, which currently exposes the shared `Common` section for hidden state without any key-signature-specific controls yet.
- Clef selections now route to `ClefPropertiesSection`, which currently exposes the shared `Common` section for hidden state without any clef-specific controls yet.
- Tempo text commits on blur or Enter; beat type, BPM, show/hide toggles, parenthesized state, and directive hidden state all apply through dedicated backend update commands.
- Unsupported selection types still render an explicit empty state instead of misleading controls.

### Horizontal spacing slots

- Slots in `get_horizontal_spacing()` are for **flow spacing**: they define left-to-right space that must push later notation over.
- A slot should exist only when an element participates in the score's horizontal flow and changes where the next items must start.
- Existing slots already cover the current flow-spacing model for clefs, key signatures, time signatures, barlines, accidentals, and note spacing.
- As a general rule, new markings should **not** get new slots unless they truly change notation flow. Tempo markings are the reference example: they align to existing slots instead of creating their own.
- If an element is visually anchored to an existing horizontal position and does not need to push later notation, prefer drawing it relative to an existing slot rather than adding another one.

### Caching

- Draw instructions cached on `RendererContext` — re-rendered on pan/zoom without recalculation
- Recalculation only triggered by score mutations (via the `emit()` flow)
- Recalculation now enters a flow-preparation seam before parse/draw work: `instructions/render_plan.rs` filters visible flows from `layout.flow_ids`, preserves `score.flows.order`, keeps the active-flow draw target explicit, and emits `render-timing-flow-preparation` plus `render-timing-flow-preparation-details` so the debugger can verify the ordered multi-flow preparation path without moving composition state into React.
- That timing contract now also carries explicit render-mode and flow-scope metadata for prepared, composed, and recalculated flow sets. Rust remains the only owner of those diagnostics, and cleared/no-render states continue to propagate as null-safe reset payloads rather than frontend-derived timing state.
- Story 1.2/1.6 extend that seam into a shared-surface composition step: Rust builds every visible flow segment independently in `score.flows.order`, offsets each later segment by a deterministic renderer-space gap of at least 20 CSS pixels at normal zoom, and keeps the combined page width authoritative for native pan/zoom constraints.
- Story 1.4 refines that shared-surface step into a page-per-flow composition rule: each composed flow now paints its own white native page box, the gradient backdrop stays visible in the fixed inter-flow gap, and the second visible flow keeps its native page-start chrome so the adjacent page still reads like a real score page rather than a cropped continuation fragment.
- Story 1.3 keeps that combined renderer width as the authoritative horizontal clamp input as well, so wheel and drag navigation traverse the same shared native surface; the debugger timing panel now also reports shared-surface and viewport sizes for reachability checks.
- Story 3.5 keeps performance characterization on that same path: maintainers validate a single-flow 3-instrument/30-bar baseline, representative multi-flow renders, and local-edit recalculation passes through the debugger timing panel and Rust-owned flow-preparation diagnostics rather than through a separate benchmark subsystem. The current release-like single-flow result is the working baseline, and a representative total median at or above 2ms now triggers a regression warning until that threshold is reevaluated.

---

## Audio Pipeline

```
┌──────────────┐     ┌──────────────────────────────────┐
│  Transport   │     │  Streamer (per note-on)           │
│  Thread      │────►│                                   │
│              │     │  1. Read FLAC from .scvi container │
│  Walks score │     │  2. Decode via symphonia           │
│  tick by tick│     │  3. Resample via rubato            │
│              │     │  4. Pitch-shift, envelope,         │
│              │     │     volume/pan                     │
│              │     │  5. Push to StreamBuffer           │
└──────────────┘     └───────────────┬──────────────────┘
                                     │
                                     ▼
                     ┌──────────────────────────────────┐
                     │  Mixer (audio callback thread)    │
                     │                                   │
                     │  64-slot lock-free voice list      │
                     │  Drains StreamBuffers → output     │
                     │  via cpal audio device             │
                     └──────────────────────────────────┘
```

### Key Components

- **Transport** — Uses atomics (`AtomicBool`, `AtomicU64`) for lock-free state sharing; instrument mix params via `RwLock<FxHashMap>`
- **Tempo resolution** — Playback snapshots the score's flow master-track tempo data at play/resume start, resolves the active BPM once per tick via a bounded resolver, and silently falls back to 120 BPM when no score tempo applies
- **Streamer** — Per-packet disk streaming (FLAC decode → resample → push to `StreamBuffer`); pre-allocates ~0.5s stereo buffer
- **Mixer** — Fixed 64-slot voice array, lock-free slot claiming via `AtomicBool`; runs in cpal audio callback
- **SCVI Format** — Custom binary container: `MAGIC(4)` + `Header` + `META(MessagePack)` + `SIDX(sample index)` + `SDAT(FLAC data)`

### Transport tempo semantics

- Playback start resolves tempo from the latest immediate absolute tempo at-or-before the current tick before the transport thread starts, so mid-flow starts report the correct inherited BPM immediately.
- The transport thread re-resolves tempo once per playback tick from its play-start score snapshot, applies tempo changes exactly on the matching tick boundary, and accumulates tick deadlines from prior timing state to reduce drift.
- `transport_get_state()` reports truthful stopped-state tempo for the current tick as well, so stopped seek and stop-to-tick-0 transitions show the effective score tempo instead of stale playback tempo.
- Pause/resume behaves like a fresh playback start from the current tick: pause retires the old thread, and resume re-snapshots score tempo data so edits made while paused are reflected.
- Live edits during active playback still do not affect the in-flight run; a fresh play/resume cycle is required because transport playback uses a play-start score snapshot.
- Tempo property editing also changes persisted tempo load/save semantics: `Tempo.text` is now stored as a required string, and migration `1.25.0` normalizes older `text: null` payloads to `""` during project load.

---

## Multi-Window Architecture

Each of the 8 windows has:

- Its own HTML entry point (e.g., `instrument-picker.html`)
- Its own `boot.tsx` mounting `AppStateProvider`
- Independent React tree receiving state via `app.emit()` to all windows

Secondary windows (instrument-picker, preferences, etc.) close on blur via `tauri://blur` event listener. New windows are opened via `WebviewWindow` from `@tauri-apps/api/webviewWindow`.

---

## File Format & Persistence

- **Project files:** `.soloc` — custom binary container with `SOLOC` magic, container version, authoritative header `app_version`, a chunk directory, and a required gzip-compressed `PROJECT_JSON` chunk containing `{ ui, score, next_id }`
- **Recent files:** Stored in `sled` embedded database at app config directory
- **Migration system:** `migrations/mod.rs` holds version-gated JSON migration functions; the loader uses the container header `app_version` as the authoritative migration key, validates it against `score.meta.app_version`, applies migrations to `serde_json::Value`, then performs a single typed deserialize before mutating app state
- **Tuplet persistence contract:** Tuplet-bearing scores use that same single `.soloc` transport and typed deserialize path. Load does not create a parallel tuplet repair layer, split crossing tones into extra stored events, or normalize ownership away from the rule that tones live where they start.
- **Current migration stance for tuplets:** No tuplet-specific step is present in `src-tauri/src/migrations/next.rs` because pre-tuplet releases produced no tuplet-bearing payloads. Legacy non-tuplet files still validate and migrate exactly as before; add a tuplet migration only if a released older persisted format requires a real compatibility transform.
- **Instrument plugins:** Discovered at runtime from `$DATA_DIR/Solo Compose/plugins/` — each is an SCVI binary container

---

## Managed State Summary (Tauri)

| State                   | Type                      | Purpose                     |
| ----------------------- | ------------------------- | --------------------------- |
| `AppContextState`       | `Mutex<AppContext>`       | Core app state              |
| `RendererContextState`  | `Mutex<RendererContext>`  | Score rendering             |
| `AudioContextState`     | `Mutex<AudioContext>`     | Audio device I/O            |
| `SamplerContextState`   | `Mutex<SamplerContext>`   | Instrument cache            |
| `TransportContextState` | `Mutex<TransportContext>` | Playback control            |
| `TransportSharedState`  | `Arc<TransportShared>`    | Lock-free transport polling |
