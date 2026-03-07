# Solo Compose — Data Models

## Overview

The score data model is the heart of Solo Compose. All musical data lives in Rust structs serialized to the frontend via serde. The model follows a **dual-index storage pattern** and a **tagged enum entry system** that are critical to understand.

---

## Core Data Model Hierarchy

```
Score
├── Config            — Global configuration (auto-count styles)
├── Meta              — Project metadata (title, composer, dates, etc.)
├── Players           — Ordered collection of Players (Solo or Section)
│   └── Player        — Has ordered list of Instrument IDs
├── Instruments       — Collection of Instruments (by_id only)
│   └── Instrument    — Definition, stave IDs, mixer params, plugin assignment
├── Flows             — Ordered collection of musical movements
│   └── Flow          — Player assignments, length in ticks, staves
│       └── Staves    — Per-instrument staves within a flow
│           └── Stave — Lines config, master track ID, user track IDs
├── Tracks            — Collection of all Tracks (by_id only)
│   └── Track         — Contains Entries (the actual musical content)
│       └── Entries   — Dual-indexed: by_tick + by_id
│           └── Entry — Tagged enum: Tone | Barline | Clef | KeySignature | TimeSignature
├── Layouts           — Ordered collection of score layouts
│   └── Layout        — Type (FullScore/Part), player and flow assignments
└── Engraves          — Engraving settings per layout type
    └── Engrave       — Spacing, fonts, padding, bracketing rules
```

---

## Dual-Index Storage Pattern

The central data access pattern used throughout. Entries are stored in two complementary maps for O(1) lookup by either dimension:

```rust
pub struct Entries {
    pub by_tick: FxHashMap<Tick, Vec<UniqueId>>,  // "What's at tick X?"
    pub by_id: FxHashMap<UniqueId, Entry>,        // "What is entry Y?"
}
```

- **`by_tick`** — Given a tick position, returns all entry IDs at that position. Used by the parse/draw pipeline to iterate through the score chronologically.
- **`by_id`** — Given an entry ID, returns the full entry. Used for direct access, mutations, and serialization to the frontend.

This same pattern appears in other collections:

- `Players { order: Vec<UniqueId>, by_id: FxHashMap<UniqueId, Player> }`
- `Flows { order: Vec<UniqueId>, by_id: FxHashMap<UniqueId, Flow> }`
- `Layouts { order: Vec<UniqueId>, by_id: FxHashMap<UniqueId, Layout> }`
- `Instruments { by_id: FxHashMap<UniqueId, Instrument> }` (order derived from player order)
- `Tracks { by_id: FxHashMap<UniqueId, Track> }` (order derived from flow/stave structure)

**Important:** All maps use `FxHashMap` from `rustc-hash`, never `std::collections::HashMap`.

---

## Entry System (Tagged Enum)

`Entry` is the fundamental unit of musical content. It uses `#[serde(tag = "type")]` for discriminated serialization:

```rust
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entry {
    Barline(Barline),
    Clef(Clef),
    KeySignature(KeySignature),
    TimeSignature(TimeSignature),
    Tone(Tone),
}
```

Every variant shares common interface methods:

- `id() → UniqueId`
- `tick() → Tick`
- `set_tick(tick: Tick)`

### Entry Variants

#### Tone

The audible musical note. **Never directly drawn** in the score — the parse pipeline converts tones into noteheads, stems, beams, etc.

| Field           | Type            | Description                     |
| --------------- | --------------- | ------------------------------- |
| `id`            | `UniqueId`      | Unique identifier               |
| `tick`          | `Tick` (u64)    | Position in the score timeline  |
| `duration`      | `Ticks` (u64)   | Length in subdivision ticks     |
| `pitch`         | `Pitch`         | MIDI-like pitch with accidental |
| `velocity`      | `Velocity`      | Playback velocity (0–127)       |
| `articulations` | `Articulations` | Set of articulation markings    |

#### Barline

| Field               | Type              | Values                                                                        |
| ------------------- | ----------------- | ----------------------------------------------------------------------------- |
| `id`                | `UniqueId`        |                                                                               |
| `tick`              | `Tick`            |                                                                               |
| `barline_draw_type` | `BarlineDrawType` | Single, Double, Triple, Dashed, Final, StartRepeat, EndRepeat, EndStartRepeat |

#### Clef

| Field     | Type           | Description                                        |
| --------- | -------------- | -------------------------------------------------- |
| `id`      | `UniqueId`     |                                                    |
| `tick`    | `Tick`         |                                                    |
| `draw_as` | `ClefDrawType` | Hidden, G, F, C, Percussion                        |
| `pitch`   | `Pitch`        | The pitch the clef represents (G for treble, etc.) |
| `offset`  | `i8`           | Visual offset from middle staff line               |

#### KeySignature

| Field    | Type               | Description                |
| -------- | ------------------ | -------------------------- |
| `id`     | `UniqueId`         |                            |
| `tick`   | `Tick`             |                            |
| `mode`   | `KeySignatureMode` | Major, Minor               |
| `offset` | `i8`               | -7 to +7 (flats to sharps) |

#### TimeSignature

| Field       | Type                    | Description                                        |
| ----------- | ----------------------- | -------------------------------------------------- |
| `id`        | `UniqueId`              |                                                    |
| `tick`      | `Tick`                  |                                                    |
| `beats`     | `u8`                    | Numerator                                          |
| `beat_type` | `NoteDuration`          | Denominator (Quarter=4, Eighth=8, etc.)            |
| `draw_type` | `TimeSignatureDrawType` | Hidden, Regular, CommonTime, SplitCommonTime, Open |
| `groupings` | `Vec<u8>`               | Beat grouping subdivisions                         |

---

## Component Types

### Pitch

```rust
pub struct Pitch {
    pub int: u8,          // MIDI note number (C0=12, C4=60)
    pub accidental: Accidental,  // DoubleSharp(2), Sharp(1), Natural(0), Flat(-1), DoubleFlat(-2)
}
```

Pitch-to-accidental mapping uses a precomputed `ACCIDENTAL_MAP: FxHashMap<(i8, KeySignatureMode), [Accidental; 12]>` covering all 15 key signatures in both major and minor modes.

### Duration & Tick System

```rust
pub type Tick = u64;   // absolute position in score timeline
pub type Ticks = u64;  // duration/length value

// Ticks per quarter note = 32
// NoteDuration::Whole = 128 ticks
// NoteDuration::Quarter = 32 ticks
// NoteDuration::Eighth = 16 ticks
// ... down to HundredTwentyEighth = 1 tick
```

### Velocity

```rust
pub struct Velocity { int: u8 }  // 0–127 MIDI velocity
```

### Articulations

```rust
pub struct Articulations(pub FxHashSet<Articulation>);
```

Articulations enforce **exclusivity groups** — only one from each group can be active:

- **Staccato group:** Staccato, Tenuto, Staccatissimo, StaccatoTenuto
- **Accent group:** Accent, Marcato
- **Stress group:** Stressed, Unstressed

---

## Score Structure Entities

### Player

```rust
pub struct Player {
    pub id: UniqueId,
    pub player_type: PlayerType,     // Solo or Section
    pub instrument_ids: Vec<UniqueId>,
}
```

### Instrument

```rust
pub struct Instrument {
    pub id: UniqueId,
    pub definition_id: String,        // e.g. "strings.violin"
    pub instrument_type: InstrumentType,  // Melodic or Percussive
    pub long_name: String,
    pub short_name: String,
    pub stave_ids: Vec<UniqueId>,
    pub count: Option<String>,        // Auto-generated: "I", "II" or "1", "2"
    pub assigned_plugin_id: Option<String>,
    pub volume: u8,                   // 0–255 mixer volume
    pub pan: u8,                      // 0–127 pan (64=center)
    pub mute: bool,
    pub solo: bool,
}
```

Instrument definitions are hardcoded in `defs.rs` via `lazy_static!`. Each definition specifies the instrument family path, names, stave count, default clef, and pitch assignments. Categories include: Brass, Keyboards, Percussion, Strings, Woodwinds.

### Flow

```rust
pub struct Flow {
    pub id: UniqueId,
    pub title: String,
    pub player_ids: FxHashSet<UniqueId>,  // players in this flow
    pub length: Ticks,                     // default: 32 × 4 × 16 = 2048 (16 bars of 4/4)
    pub master_id: UniqueId,               // master track for flow-level entries
    pub staves: Staves,                    // per-instrument staves
}
```

### Stave

```rust
pub struct Stave {
    pub id: UniqueId,
    pub lines: Vec<u8>,       // e.g. [1,1,1,1,1] for standard 5-line staff
    pub master_id: UniqueId,  // master track for clefs, key/time sigs
    pub track_ids: Vec<UniqueId>,  // user tracks (tones, barlines)
}
```

### Track

```rust
pub struct Track {
    pub id: UniqueId,
    pub entries: Entries,  // dual-indexed entry storage
}
```

Each stave has:

- A **master track** — contains clef, key signature, time signature entries
- One or more **user tracks** — contain tones and barlines

### Layout

```rust
pub struct Layout {
    pub id: UniqueId,
    pub layout_type: LayoutType,  // FullScore or Part
    pub title: String,
    pub player_ids: FxHashSet<UniqueId>,
    pub flow_ids: FxHashSet<UniqueId>,
}
```

### Engrave

Engraving settings control the visual appearance of the score. Stored **per layout type** (FullScore vs Part). Key settings include spacing parameters (in Space units and mm), font configuration, bracketing approach (None/Orchestral/SmallEnsemble), and note spacing ratios.

---

## Relationships Diagram

```
Players (ordered)
  └─► Player
       └─► instrument_ids ──► Instruments.by_id
                                 └─► Instrument
                                      └─► stave_ids ──► Flow.staves.by_id
                                                          └─► Stave
                                                               ├─► master_id ──► Tracks.by_id (clefs, key/time sigs)
                                                               └─► track_ids ──► Tracks.by_id (tones, barlines)
                                                                                  └─► Track.entries
                                                                                       ├─► by_tick[tick] → [id, id, ...]
                                                                                       └─► by_id[id] → Entry variant

Flows (ordered)
  └─► Flow
       ├─► player_ids (which players participate)
       ├─► master_id ──► Tracks.by_id (flow-level time/key sigs)
       └─► staves.by_id ──► (instrument-level staves)

Layouts (ordered)
  └─► Layout
       ├─► player_ids (which players to show)
       └─► flow_ids (which flows to include)

Engraves
  └─► by_layout_type
       ├─► FullScore → Engrave settings
       └─► Part → Engrave settings
```

---

## Serialization Rules

1. All enums use `serde_repr` (integer serialization) — TS enums must use matching integer values
2. `Entry` uses `#[serde(tag = "type")]` — the `type` field discriminates variants in JSON
3. `FxHashMap<K, V>` serializes to `Record<K, V>` in TypeScript
4. `FxHashSet<K>` appears as arrays in serialized JSON but is a set in Rust
5. All struct field names use `snake_case` — TypeScript types must match exactly
6. `UniqueId = number`, `Tick = number`, `Ticks = number` in TypeScript

---

## Auto-Counting System

When multiple instruments of the same type exist (e.g., two violins), the `Score::update_counts()` method automatically assigns count labels. Counts are formatted per player type using `Config.auto_count_styles`:

- **Solo players:** Arabic ("1", "2") or Roman ("I", "II")
- **Section players:** Arabic or Roman (independently configured)

---

## Migration System

Persisted projects are stored as `.soloc` containers with this high-level layout:

```text
[SOLOC magic][container_version][app_version_len][app_version][chunk_count]
[chunk directory entries]
[chunk payloads]
```

In container version `1`, the loader requires exactly one logical project payload chunk identified by `u32::from_le_bytes(*b"JSON")`. That `PROJECT_JSON` chunk is gzip-compressed and contains the canonical minified JSON document:

```json
{"ui": ..., "score": ..., "next_id": ...}
```

The container header `app_version` is the authoritative migration key. The JSON payload still stores `score.meta.app_version` for display and inspection, but load rejects the file if the header value and JSON value differ. Unknown future chunk types are ignored after directory validation, while unsupported flag bits, duplicate required chunks, and out-of-bounds chunk ranges fail explicitly.

File migrations are defined in `src-tauri/src/migrations/mod.rs`:

```rust
pub static ref MIGRATIONS: Vec<Migration> = vec![];
```

Each migration is a function `fn(Value) -> Result<Value, String>` that transforms the JSON data. Migrations run sequentially on file load when the container header `app_version` is lower than the registered migration version. There are currently no active migrations registered. After all migrations, `score.meta.app_version` is rewritten to the current binary version so the JSON payload stays aligned with the header value written on the next save.

Any persisted-data shape change, `.soloc` payload change, or load/save semantic change must include a migration review. If older files would not load correctly without transformation, add or update a migration in `src-tauri/src/migrations/` and register it in `mod.rs`.
