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
│           └── Entry — Tagged enum: Tone | Barline | Clef | KeySignature | Tempo | TimeSignature | Tuplet
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
    Tempo(Tempo),
    TimeSignature(TimeSignature),
    Tone(Tone),
    Tuplet(Box<Tuplet>),
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
| `hidden`  | `bool`         | If true, the entry persists but does not render    |
| `draw_as` | `ClefDrawType` | G, F, C, Percussion                                |
| `pitch`   | `Pitch`        | The pitch the clef represents (G for treble, etc.) |
| `offset`  | `i8`           | Visual offset from middle staff line               |

#### KeySignature

| Field    | Type               | Description                                     |
| -------- | ------------------ | ----------------------------------------------- |
| `id`     | `UniqueId`         |                                                 |
| `tick`   | `Tick`             |                                                 |
| `hidden` | `bool`             | If true, the entry persists but does not render |
| `mode`   | `KeySignatureMode` | Major, Minor                                    |
| `offset` | `i8`               | -7 to +7 (flats to sharps)                      |

#### TimeSignature

| Field       | Type                    | Description                                     |
| ----------- | ----------------------- | ----------------------------------------------- |
| `id`        | `UniqueId`              |                                                 |
| `tick`      | `Tick`                  |                                                 |
| `hidden`    | `bool`                  | If true, the entry persists but does not render |
| `beats`     | `u8`                    | Numerator                                       |
| `beat_type` | `NoteDuration`          | Denominator (Quarter=4, Eighth=8, etc.)         |
| `draw_type` | `TimeSignatureDrawType` | Regular, CommonTime, SplitCommonTime, Open      |
| `groupings` | `Vec<u8>`               | Beat grouping subdivisions                      |

The frontend `TimeSignature` mirror now includes both `draw_type` and `hidden`, because the write-screen Properties tab can switch valid 4/4 and 2/2 signatures between stacked numerals and common-time symbols while also hiding the persisted directive without changing its musical meaning.

#### Tempo

Immediate absolute tempo markings live on the **flow master track** so the renderer and future playback-following-tempo work can query a single flow-wide source of truth.

| Field            | Type           | Description                                                 |
| ---------------- | -------------- | ----------------------------------------------------------- |
| `id`             | `UniqueId`     |                                                             |
| `tick`           | `Tick`         | Tick where the tempo marking is attached                    |
| `text`           | `String`       | Stored label text; empty string represents no visible label |
| `beat_type`      | `NoteDuration` | Metronome beat value (`Quarter` in v1 authoring)            |
| `dots`           | `u8`           | Augmentation dots for the metronome note                    |
| `bpm`            | `u16`          | Resolved beats-per-minute value                             |
| `hidden`         | `bool`         | If true, the entry persists but does not render             |
| `show_text`      | `bool`         | Controls whether the textual label renders                  |
| `show_metronome` | `bool`         | Controls whether the metronome note and BPM render          |
| `parenthesized`  | `bool`         | Wraps the metronome portion in parentheses                  |

Supported v1 display forms are:

- `Allegro`
- `Allegro q=120`
- `Allegro (q=120)`
- `q=120`

Metronome-only tempos now persist as `text: ""` plus `show_text: false` rather than `text: null`.

Tempo entries drive both score rendering and transport tempo resolution, and selected tempo entries can be edited from the write-screen `Properties` tab.

This schema changed in `1.25.0`; older `.soloc` payloads that still contain `text: null` are normalized during load by migration `1.25.0`.

Unreleased directive visibility work also adds `hidden: bool` to clef, key-signature, and time-signature entries, and migrates legacy time signatures that previously encoded hidden state as `draw_type: 0` into `hidden: true` plus `draw_type: Regular` in `src-tauri/src/migrations/next.rs`.

#### Tuplet

Tuplets are persisted as first-class container entries on the host track. The container owns the exact rhythmic context for notes that start inside the tuplet span, while keeping the existing dual-index storage model both at the host track and inside the tuplet.

| Field             | Type           | Description                                                    |
| ----------------- | -------------- | -------------------------------------------------------------- |
| `id`              | `UniqueId`     | Unique identifier for the container                            |
| `tick`            | `Tick`         | Host-track start tick                                          |
| `note_count`      | `u8`           | Number of base-note units written inside the tuplet            |
| `in_space_of`     | `u8`           | Number of base-note units the tuplet occupies in host time     |
| `base_note_value` | `NoteDuration` | Base printed note value for the tuplet grouping                |
| `groupings`       | `Vec<u8>`      | Generic grouping metadata for current and future tuplet shapes |
| `entries`         | `Entries`      | Nested child entries whose tones start inside the tuplet span  |

The Rust enum stores the variant as `Tuplet(Box<Tuplet>)` to keep the recursive `Entry -> Tuplet -> Entries -> Entry` model sized safely in memory. The persisted JSON remains the same tagged-enum shape and does not gain a second wrapper field.

Host and local tick spans are derived rather than stored twice:

- `consumed_ticks = in_space_of * base_note_value.to_ticks()`
- `local_length_ticks = note_count * base_note_value.to_ticks()`

For an eighth-note triplet, that means `note_count = 3`, `in_space_of = 2`, and `base_note_value = Eighth`, which derives the same `48` local ticks projected into `32` host ticks.

In the current MVP authoring flow, the existing popover system creates tuplets as backend-owned host-track containers. The frontend only collects transient popover input in an explicit learned syntax such as `3e/2` (`3` eighths in the space of `2`) and dispatches a create command; it does not keep a parallel draft tuplet timing model.

Supported MVP tuplet suffixes are:

- `q` for quarter-note triplets (`3q/2`)
- `e` for eighth-note triplets (`3e/2`)
- `s` for sixteenth-note triplets (`3s/2`)
- `t` for thirty-second-note triplets (`3t/2`)

Ownership rules:

- A tone is stored exactly once.
- Tone ownership is determined by where the tone starts.
- Tones that start inside a tuplet live in the tuplet's child `entries`.
- Tones that start outside remain on the host track even if their duration crosses the tuplet span.
- MVP tuplets currently persist only nested musical content; inner clef, key-signature, time-signature, and tempo directives remain deferred.

### Sequencer Selection (UI-only)

Sequencer selection state is mirrored between Rust and TypeScript and is **not** persisted into `.soloc` files. The selection union now includes:

- `Tone { flow_id, instrument_id, stave_id, track_id, tone_id }`
- `Rest { flow_id, instrument_id, stave_id, track_id, tick }`
- `Tempo { flow_id, tempo_id, tick }`
- `TimeSignature { flow_id, time_signature_id, tick }`
- `KeySignature { flow_id, key_signature_id, tick }`
- `Clef { flow_id, stave_id, clef_id, tick }`
- `Tuplet { flow_id, instrument_id, stave_id, track_id, tuplet_id, tick }`

These directive-selection payloads exist only in UI state so the native renderer can round-trip hit targets, highlight state, keyboard deletion, and popover anchoring. `Tuplet` selection also lets creation hand focus back through the existing `apply_selection` path, which keeps `ui.flow_id` as the single authoritative active-flow slot. None of these UI-only payloads change persisted project JSON or require a migration.

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

Each flow has:

- A **flow master track** — contains flow-wide directives such as key signatures, time signatures, and tempo markings

Each stave has:

- A **stave master track** — contains clef entries
- One or more **user tracks** — contain tones and barlines

User tracks can now also contain `Tuplet` entries. A tuplet container participates in the same `entries.by_tick` and `entries.by_id` indices as any other first-class entry, and its child `entries` recurse through that same structure.

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

Adding `Entry::Tuplet` is an additive persisted-schema change. Legacy `.soloc` files remain loadable without a backfill migration because older files simply omit the new tagged variant; newly saved files may include it once tuplet authoring lands.

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
pub static ref MIGRATIONS: Vec<MigrationDescriptor> = vec![
    MigrationDescriptor::shipped("1.25.0", v1_25_0::migrate),
    MigrationDescriptor::pending_next(next::migrate),
];
```

Each migration is a function `fn(Value) -> Result<Value, String>` that transforms the JSON data. Shipped migrations run sequentially on file load when the container header `app_version` is lower than the registered concrete semver target. Development builds may also register a single logical pending migration bucket from `src-tauri/src/migrations/next.rs`; that pending migration runs after all shipped migrations without trying to parse `"next"` as semver. After all migrations, `score.meta.app_version` is rewritten to the current binary version so the JSON payload stays aligned with the header value written on the next save.

Recommended persisted-data workflow:

1. Implement the feature and validate whether persistence changed.
2. If migration is required, add ordered pending step functions to `src-tauri/src/migrations/next.rs` and register them in its `PENDING_STEPS` collection.
3. Have a human run `prepare-release:*` to compute the concrete target version, promote any real pending steps into a shipped migration file, and write `.release-prep.json`.
4. After reviewing and validating the prepared tree, have a human run `finalize-release` so the release commit/tag ships only concrete migrations and the follow-up development commit restores a fresh empty `next.rs` scaffold.

Any persisted-data shape change, `.soloc` payload change, or load/save semantic change must include a migration review. If older files would not load correctly without transformation, add or update a migration in `src-tauri/src/migrations/` and register it in `mod.rs`. Saved file header versions and `score.meta.app_version` values remain concrete semver strings even while unreleased transforms live in the logical pending `next` bucket.
