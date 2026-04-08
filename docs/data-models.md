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
│   └── Instrument    — Definition, stave IDs, mixer params, persisted playback assignment intent
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

| Field             | Type            | Description                                                    |
| ----------------- | --------------- | -------------------------------------------------------------- |
| `id`              | `UniqueId`      | Unique identifier for the container                            |
| `tick`            | `Tick`          | Host-track start tick                                          |
| `note_count`      | `u8`            | Number of base-note units written inside the tuplet            |
| `in_space_of`     | `u8`            | Number of base-note units the tuplet occupies in host time     |
| `base_note_value` | `NoteDuration`  | Base printed note value for the tuplet grouping                |
| `groupings`       | `Vec<u8>`       | Generic grouping metadata for current and future tuplet shapes |
| `entries`         | `TupletEntries` | MVP child entries whose tones start inside the tuplet span     |

The Rust enum stores the variant as `Tuplet(Box<Tuplet>)` to keep the top-level `Entry` model sized safely in memory. MVP tuplet child storage now uses a dedicated backend wrapper that keeps the persisted JSON field shape stable while rejecting nested `Entry::Tuplet` values. Nested tuplets are therefore not part of supported MVP structure.

Tuplet-bearing projects save and reopen through the same typed `ProjectFile` payload and `.soloc` container path used by non-tuplet projects. The persistence seam does not split crossing tones at load time, rewrite durations just because a tuplet boundary exists, or rebuild tuplet ownership in a frontend-specific pass. Host-owned tones that start before the tuplet remain on the host track, child-owned tones that start inside the tuplet remain in `Tuplet.entries`, and repeated save/load cycles are expected to keep the persisted structure deterministic.

Host and local tick spans are derived rather than stored twice:

- `consumed_ticks = in_space_of * base_note_value.to_ticks()`
- `local_length_ticks = note_count * base_note_value.to_ticks()`

For an eighth-note triplet, that means `note_count = 3`, `in_space_of = 2`, and `base_note_value = Eighth`, which derives the same `48` local ticks projected into `32` host ticks.

In the current MVP authoring flow, the existing popover system creates tuplets as backend-owned host-track containers. The popover opens with an empty text field at the current selection tick, but confirmation only succeeds when the active selection is a tone or rest that already carries instrument-track ownership data. The frontend collects transient learned syntax such as `3e/2` (`3` eighths in the space of `2`); it does not keep a parallel draft tuplet timing model.

Supported MVP tuplet suffixes are:

- `q` for quarter-note triplets (`3q/2`)
- `e` for eighth-note triplets (`3e/2`)
- `s` for sixteenth-note triplets (`3s/2`)
- `t` for thirty-second-note triplets (`3t/2`)

Broader arbitrary ratios remain deferred in the shipped MVP. Durable docs should describe only the supported triplet-first entry path above unless a later post-MVP story expands the parser and UI together.

Supported triplet enforcement currently lives in the popover parser path. This matches the existing creation-command pattern in the codebase: frontend creation flows validate product-specific input, and the lower-level Rust create commands validate target existence plus state mutation only.

Ownership rules:

- A tone is stored exactly once.
- Tone ownership is determined by where the tone starts.
- Tones that start inside a tuplet live in the tuplet's child `entries`.
- Tones that start outside remain on the host track even if their duration crosses the tuplet span.
- MVP tuplets are explicitly single-level. Child `entries` cannot contain nested tuplets, and inner clef, key-signature, time-signature, and tempo directives remain deferred.

Tuplet creation over existing host material follows the same ownership rule. When the backend creates a tuplet, it inspects the consumed host span and moves only the tones whose starts fall inside that span into `Tuplet.entries`. Those carried tones keep their stored durations unchanged and keep their timing as literal copied local offsets relative to the tuplet start; creation does not respell written values, approximate binary replacements, or split crossing tones. Any unfilled local tuplet time remains derived absence of tones rather than a persisted rest payload.

No tuplet-specific migration is required in the current development cycle. Files created before tuplet support do not contain persisted tuplet payloads, so legacy non-tuplet projects continue through the existing version-validation and migration flow unchanged. Future migration work is only needed if a released older payload can no longer represent the persisted tuplet contract correctly.

Tuplet note insertion still extends the existing track-level tone mutation seam rather than introducing a parallel tuplet-note API:

- frontend: `actions.score.tuplet.create(track_id, tick, note_count, in_space_of, base_note_value, groupings)`
- backend command: `score_entries_tuplet_create(track_id, tick, note_count, in_space_of, base_note_value, groupings)`
- frontend: `actions.score.tone.create(track_id, tick, duration, pitch_int, accidental, velocity, articulations, tuplet_id?)`
- backend command: `score_entries_tone_create(..., tuplet_id: Option<UniqueId>)`
- score mutation: `Track::create_tone(..., tuplet_id: Option<UniqueId>)`

The addressing model remains explicit as `track_id -> optional tuplet_id -> tone_id`. Host-track tones keep track-relative ticks and durations. Tuplet child tones keep tuplet-local ticks and durations. Insertion does not translate child timing into host ticks or persist explicit rest entries for unused local space.

Deletion and slicing follow that same ownership model. Removing a tuplet-owned tone removes only that child entry and leaves the vacated local span as derived empty tuplet space. Removing a tuplet container removes the host-track `Tuplet` entry and its child `entries` together; MVP does not dissolve child tones back onto the host track or synthesize rests.

Tone slicing also stays on the shared tone mutation seam. The slice request now carries both the current owning context and the target owner for the new tail start. Rust resolves both addresses against the same continuous tuplet/host lattice used by sequencer editing, shortens the original tone to the head duration, and inserts exactly one new stored tail tone in the ownership context determined by the tail start. Crossing a tuplet boundary therefore does not create any extra persisted fragments beyond the intentional two-tone slice result.

The shipped sequencer editing surface remains intentionally narrow: after popover creation, contributors should expect tone-focused selection, insertion, reposition/resize within the supported ownership lattice, deletion, and slicing on the shared seams above. Empty tuplet slots remain derived absence of tones rather than selectable stored objects, and MVP does not expose generalized arbitrary tuplet reshaping or nested-container editing UX.

Story 1.5 stops at ownership reconciliation and persisted timing carry-over. Epic 3 now defines the parser-side tuplet notation contract that later renderer and spacing stories consume, while visible engraving chrome and spacing resolution remain separate downstream concerns.

### Tuplet Notation Derivation Contract

The persisted tuplet container is not itself the printed-score notation result. Parser derivation must materialize a separate notation-layer contract that preserves tuplet-local meaning without mutating persisted ownership.

That derived contract keeps three concepts distinct:

- `semantic address` — the logical position of a notation entry in its owning context, either host or tuplet-local
- `written duration` — the duration used for notation spelling, rest splitting, beaming, dots, and related written-value decisions inside the local frame
- `host occupancy` — the start/end span that the anchored local notation region occupies in surrounding host time, together with the projection rule used to map local ticks back into host space

For supported MVP tuplets, the intended parser output is a thin anchored-region model:

- the tuplet remains an isolated local `NotationTrack`
- that anchored region also carries the host-span anchor needed to place the local unit back into the main flow

Conceptually, a derived tuplet region has this shape:

```rust
type SemanticAddress = {
    owner_id: Option<UniqueId>,
    tick: Tick,
};

type HostSpan = {
    start_tick: Tick,
    end_tick: Tick,
};

type AnchoredNotationTrack = {
    owner_id: UniqueId,
    host_span: HostSpan,
    notation: NotationTrack,
};
```

These names are illustrative rather than prescriptive Rust types. The important rule is that downstream consumers must not have to recover tuplet-local semantics from a single overloaded host `tick + duration` pair. The local unit should stay a normal `NotationTrack` so existing helpers and manipulation patterns continue to apply. `owner_id` is intentionally generic in this derived-contract sketch: for tuplets it is the tuplet container id, but the anchored-region shape is written so it can be reused later for other bounded notation units with minimal structural change.

Derivation rules for this contract:

- The host track treats the governed tuplet span as an implicit placeholder rather than as ordinary host notation content.
- The tuplet is derived as its own local metric frame, then anchored back into host ordering through host-span metadata.
- Tuplet span and crossing-duration consumption are separate concerns: the container still occupies the denominator-sized host span, but a host-owned tone crossing that boundary consumes its full literal duration on the canonical expanded timeline rather than being shortened to the compressed host geometry.
- Tuplet-local rest and note spelling reuse the existing split-unit logic against the equivalent local frame instead of inventing a separate tuplet-only spelling system.
- A completely empty triplet is a special readability case and derives one base-note rest per local slot.
- Boundary-crossing tones remain one stored tone. Derivation may emit multiple local notations or host-side companions at host/tuplet boundaries, but continuity still follows stored tone identity.
- Adjacent tuplets remain separate isolated units even when their host spans touch exactly.

This contract is intentionally parse-layer only. It does not change persisted `.soloc` ownership rules, does not add persisted rest entries for empty local slots, and does not move renderer spacing or bracket-drawing decisions into the persistence model.

### Sequencer Selection (UI-only)

Sequencer selection state is mirrored between Rust and TypeScript and is **not** persisted into `.soloc` files. The selection union now includes:

- `Tone { flow_id, instrument_id, stave_id, track_id, tone_id, tuplet_id? }`
- `Rest { flow_id, instrument_id, stave_id, track_id, tick }`
- `Tempo { flow_id, tempo_id, tick }`
- `TimeSignature { flow_id, time_signature_id, tick }`
- `KeySignature { flow_id, key_signature_id, tick }`
- `Clef { flow_id, stave_id, clef_id, tick }`
- `Tuplet { flow_id, instrument_id, stave_id, track_id, tuplet_id, tick }`

These directive-selection payloads exist only in UI state so the native renderer can round-trip hit targets, highlight state, keyboard deletion, and popover anchoring. `Tone.tuplet_id` is optional and carries only the minimal child-context handoff needed when a newly inserted tuplet tone is selected. `Rest.tuplet_id` is also optional and follows the same rule for renderer-owned rest hit targets: when it is absent, `Rest.tick` is a host-track tick; when it is present, `Rest.tick` is a tuplet-local tick inside that container. `Tuplet` selection also lets creation hand selection back through the existing `apply_selection` path, which keeps `ui.flow_id` as the single authoritative active-flow slot. None of these UI-only payloads change persisted project JSON or require a migration.

This is intentional abstraction rather than a temporary workaround. Sequencer selection treats tuplets as nested timing contexts: position remains `tick`, and optional context determines which timeline that tick belongs to. The UI must therefore derive presentation-only slot markers from `tick + tuplet_id` instead of persisting a second slot-ordinal field.

That selection model does not imply sequencer-level rest rendering, rest selection, or tuplet-container selection. In the sequencer, empty tuplet space remains absence of child tones, and clicks on that blank space behave like other non-note background interactions rather than creating rest or tuplet targets. Derived rest glyphs and visible notation-object selection belong to the printed-score renderer, where notation is materialized explicitly.

For sequencer presentation, tuplets now render directly from the host-track `Tuplet` entry rather than from a frontend-owned draft region. The visible host span is derived from `in_space_of * noteDurationToTicks(base_note_value)`, the track body renders the faint region tint plus the compressed local slot markers, and child tones render inside that same region using tuplet-local ticks projected through the tuplet's local slot geometry. The `3:2` annotation lives in the tick bar as a bottom-anchored ruler annotation rather than as floating text inside the lane. The current sequencer UI does not depend on dedicated tuplet selection or empty-slot hit geometry; the shared `Tuplet` selection payload remains part of the broader UI contract for renderer-driven notation interactions, while the sequencer itself stays tone-focused.

Playback is a separate consumer of this same persisted model. For shipped MVP behavior, transport scheduling reads tuplets from the active flow only, using `ui.flow_id` as the authoritative playback scope boundary rather than synthesizing cross-flow tuplet state.

---

## Component Types

### Pitch

```rust
pub struct Pitch {
    pub int: u8,          // MIDI note number (C0=12, C4=60)
    pub accidental: Accidental,  // DoubleSharp(2), Sharp(1), Natural(0), Flat(-1), DoubleFlat(-2)
}
```

MCP `score.tone.create` and `score.tone.setPitch` use this same `Pitch.int` contract for `pitch_int`. For MCP-only clients, the practical reference points are `C0 = 12` and `C4 = 60`. Passing `accidental = null` allows the backend to infer enharmonic spelling from the active key signature; passing an explicit accidental forces the stored spelling.

For MCP-only score construction, gaps usually remain persisted absence of tones rather than explicit rest entries, and multi-staff instruments such as piano expose separate playable track ids per stave through `score.lookup.getInstrumentTracks`.

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
    pub playback_route: Option<InstrumentPlaybackRoute>,
    pub volume: u8,                   // 0–255 mixer volume
    pub pan: u8,                      // 0–127 pan (64=center)
    pub mute: bool,
    pub solo: bool,
}
```

`Instrument.playback_route` is the durable score-owned link from an instrument into the persisted score rack. The rack stores portable slot targets, while the route stores the instrument's intended destination and engine-facing address payload without reintroducing frontend-owned truth.

```rust
pub struct ScorePlayback {
    pub rack: PlaybackRack,
}

pub struct PlaybackRack {
    pub order: Vec<UniqueId>,
    pub by_id: FxHashMap<UniqueId, PlaybackRackSlot>,
}

pub struct PlaybackRackSlot {
    pub id: UniqueId,
    pub target: Option<PlaybackSlotTarget>,
}

pub struct PlaybackSlotTarget {
    pub plugin_id: String,
}

pub struct InstrumentPlaybackRoute {
    pub slot_id: UniqueId,
    pub target: PlaybackRouteTarget,
    pub address: PlaybackRouteAddress,
    pub expression_map_id: Option<String>,
}

pub struct PlaybackRouteTarget {
    pub mode: PlaybackRouteTargetMode,     // Plugin | PluginPart | MidiChannel
    pub index: Option<u8>,
}

pub struct PlaybackRouteAddress {
    pub mode: PlaybackRouteAddressMode,    // Root | Part | MidiChannel
    pub index: Option<u8>,
}

pub struct PlaybackRouteChoice {
    pub key: String,
    pub label: String,
    pub target: PlaybackRouteTarget,
    pub address: PlaybackRouteAddress,
}
```

Portable rack and route intent are distinct from machine-local readiness. A rack slot is a first-class score object and may exist before any plugin is assigned to it, which lets slot preparation, route authoring, and later runtime activation remain separate bounded workflows. `InstrumentPlaybackRoute.expression_map_id` extends that same durable route-address seam: the project stores only a stable reusable expression map reference on the route, not the full app-scoped map definition. The backend emits resolved score-context state separately through `soloc://state/general` so React can present one Rust-owned answer instead of re-deriving route health from scattered fields.

The reusable expression map library is app-scoped and machine-local:

```rust
pub struct ExpressionMap {
    pub id: String,
    pub name: String,
    pub creator: String,
    pub version: u32,
    pub default_actions: Vec<ExpressionMapAction>,
    pub rules: Vec<ExpressionMapRule>,
}

pub struct ExpressionMapRule {
    pub id: String,
    pub conditions: Vec<ExpressionMapCondition>,
    pub actions: Vec<ExpressionMapAction>,
}

pub enum ExpressionMapCondition {
    NoOp { id: String },
    Articulation { id: String, ... },
    WrittenNoteLength { id: String, ... },
}

pub enum ExpressionMapAction {
    KeyswitchNote { id: String, ... },
    ControlChange { id: String, ... },
    NoOp { id: String },
    NoteLengthTransform { id: String, ... },
}

pub struct ExpressionMapLibraryState {
    pub order: Vec<String>,
    pub by_id: FxHashMap<String, ExpressionMap>,
}
```

The MVP rule vocabulary is intentionally bounded but extension-ready:

Each reusable expression map also carries a stable app-scoped `id` generated by the Rust backend as a UUID. The editor shows that id as read-only reference data rather than a user-authored field.

Each reusable rule, condition, and action carries a stable internal `id`. Those ids are not user-facing fields, but they are part of the persisted reusable-map model so selection, reorder, and nested edits follow durable identity instead of array position.

Rust owns the reusable expression-map data model and all mutation lookup. React is a thin control layer that issues targeted commands such as “set map name”, “set map creator”, “set map version”, “add rule”, “move rule”, “set rule name”, “add condition”, “set condition type”, “update condition”, the equivalent rule-action mutations keyed by `expression_map_id`, `rule_id`, and nested item ids, plus dedicated map-level default-action mutations keyed by `expression_map_id` and default action id. The frontend no longer constructs partial nested defaults or pushes rebuilt parent arrays back through a vague whole-map patch path.

Nested ids are part of the required persisted expression-map contract, not an optional compatibility layer. Rule, condition, and action payloads without ids are invalid.

- conditions: no-op, articulation, written note length
- actions: keyswitch note, control change, no-op, note-length transform

New rules and newly added conditions scaffold with a `NoOp` condition placeholder. That placeholder is an explicit non-matching editor state, replacing the older null-articulation sentinel so unfinished rules do not accidentally imply an articulation-specific condition.

Each reusable expression map also owns a required `default_actions` list. Those actions run when no authored rule matches, which makes plugin-specific reset or baseline behavior explicit without inventing a fake catch-all rule inside the ordered rule list. New maps default that list to one `NoOp` entry so plugin-default playback remains the baseline until the user authors something more specific.

Missing reusable-map references do not create a separate broken-link persistence state. The score may still contain the stored reference, but the resolved score-context payload clears back to unassigned truth when the app-scoped library no longer contains that id.

The first runtime layer for that split is intentionally non-persisted and same-session only:

```rust
pub struct PlaybackActivationRuntime {
    activated_slot_target_by_slot_id: FxHashMap<UniqueId, ActivatedSlotTarget>,
    activated_instrument_route_by_instrument_id: FxHashMap<UniqueId, ActivatedInstrumentRoute>,
}
```

`PlaybackActivationRuntime` lives inside backend app state rather than `.soloc` data. It is rebuilt from persisted rack or route intent plus machine-local inventory truth after slot assignment, slot unassignment, slot removal, route assignment, route clearing, project load, and inventory refresh. Story 3.8 keeps that runtime bounded to single-target plugin-output routes only, and Story 5.4 confirms that project reopen reuses the same rebuild contract without persisting extra activation payloads, active transport playback, queued work, or machine-local inventory truth into score files.

The backend also emits per-slot `PlaybackRouteChoice` lists through `route_target_options_by_slot_id` on `soloc://state/general`. Sequence must consume that Rust-owned contract when offering route targets; if Rust can only prove the direct plugin output for a slot, React must expose only that direct choice and must not invent speculative part or MIDI-channel options. Separate plugin-assignment pickers remain bounded to plugins whose inventory entries are both `availability === "available"` and not `editor_host_validation.status === "blocked"`.

```rust
pub struct ResolvedPlaybackRoute {
    pub status: ResolvedPlaybackRouteStatus,   // Unrouted | Ready | NeedsAttention
    pub slot_id: Option<UniqueId>,
    pub slot_target: Option<PlaybackSlotTarget>,
    pub effective_plugin_id: Option<String>,   // Present only when current runtime compatibility is available
    pub target: Option<PlaybackRouteTarget>,
    pub address: Option<PlaybackRouteAddress>,
}
```

```rust
pub struct ResolvedExpressionMapLink {
    pub status: ResolvedExpressionMapLinkStatus, // Unassigned | Assigned
    pub expression_map_id: Option<String>,
    pub expression_map_name: Option<String>,
}
```

`ResolvedPlaybackRoute` is availability-aware. `effective_plugin_id` is exposed only when the routed slot target currently resolves through the same-session backend activation runtime, so unrouted instruments, missing slots, slots with no assigned plugin, missing plugins, and deferred plugin-part or MIDI-channel workflows degrade to explicit `NeedsAttention` or `Unrouted` states instead of silently falling through legacy runtime paths. Multiple instruments can intentionally share one slot by pointing distinct `Instrument.playback_route` entries at the same `slot_id`.

`ResolvedExpressionMapLink` is intentionally quieter than route readiness. Assigned reusable maps surface their id and current display name for future score-context UI, but missing-library references resolve to `Unassigned` instead of adding a separate warning state. Story 6.1 does not make empty maps audible; they remain valid reusable assets whose presence must preserve the current default playback path until later Epic 6 work consumes their rules during transport preparation.

The ownership split is strict: the rack owns slot lifecycle, slots own plugin assignment, and instruments own only their explicit route into an existing slot. Removing or unrouting an instrument must not implicitly delete the referenced slot.

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

User tracks can now also contain `Tuplet` entries. A tuplet container participates in the same `entries.by_tick` and `entries.by_id` indices as any other first-class entry, while its child `entries` use an MVP-safe wrapper with the same persisted JSON shape but no nested `Tuplet` children. Printed-score tuplet notation is derived later from that persisted structure; it is not stored directly in `Track.entries`.

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

Adding `Entry::Tuplet` was an additive persisted-schema change. Legacy `.soloc` files remain loadable because older files simply omit the new tagged variant. Nested tuplet child content remains unsupported MVP data and is rejected during deserialization rather than preserved as active structure.

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
