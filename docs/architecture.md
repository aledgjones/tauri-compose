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
│  draw_time_signature, draw_barline,          │
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

### Caching

- Draw instructions cached on `RendererContext` — re-rendered on pan/zoom without recalculation
- Recalculation only triggered by score mutations (via the `emit()` flow)

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
- **Streamer** — Per-packet disk streaming (FLAC decode → resample → push to `StreamBuffer`); pre-allocates ~0.5s stereo buffer
- **Mixer** — Fixed 64-slot voice array, lock-free slot claiming via `AtomicBool`; runs in cpal audio callback
- **SCVI Format** — Custom binary container: `MAGIC(4)` + `Header` + `META(MessagePack)` + `SIDX(sample index)` + `SDAT(FLAC data)`

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
