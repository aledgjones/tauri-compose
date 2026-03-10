# Visual Regression Fixtures

This directory contains test fixtures for **manual visual regression** of Solo Compose.
The flow is:

1. An agent (or developer) authors a **JSON project payload** and commits it to `source/`
2. Running `npm run build:fixtures` compiles each JSON file into an openable **`.soloc` file** in `built/`
3. Aled opens each `.soloc` file in the app and visually verifies that rendering is correct

Automated visual regression is **out of scope** for now — this is a human-in-the-loop process.

---

## Directory layout

```
tests/fixtures/
├── README.md              ← you are here
├── source/                ← committed JSON payloads (grouped by concern)
│   ├── articulations/
│   │   └── articulations.json
│   ├── clefs/
│   │   └── clefs.json
│   ├── key-signatures/
│   │   └── key-signatures.json
│   ├── ledger-lines/
│   │   └── ledger-lines.json
│   ├── rhythms/
│   │   ├── written-durations-four-four.json
│   │   ├── written-durations-three-four.json
│   │   ├── written-durations-three-four-six-eight-comparison.json
│   │   └── written-durations-compound.json
│   ├── rests/
│   │   └── rests.json
│   ├── tempo/
│   │   └── tempo.json
│   └── time-signatures/
│       ├── four-four.json
│       ├── three-four.json
│       └── changing.json
└── built/                 ← gitignored .soloc output
    ├── articulations/
    ├── clefs/
    ├── key-signatures/
    ├── ledger-lines/
    ├── rhythms/
    ├── rests/
    ├── tempo/
    └── time-signatures/
```

Source files are committed. Built files are **gitignored** and must be regenerated locally before opening in the app.

---

## Running the fixture builder

```bash
npm run build:fixtures
```

This compiles the Rust `build-fixtures` binary (first run only), then converts every `source/**/*.json` file into a matching `built/**/<name>.soloc`.

The script is incremental-friendly — re-running only takes a few seconds once compiled.

---

## Opening a fixture in the app

```bash
npm run start:dev
```

Then **File → Open** and navigate to `tests/fixtures/built/<concern>/<name>.soloc`.

The file opens exactly as if you had saved it from the app. The `ui.tab` field in the JSON controls which tab opens (0 = Setup, 1 = Write, 2 = Engrave, 3 = Play, 4 = Print). Most fixtures should open to Write (1).

---

## Authoring a new fixture

### 1 — Understand the JSON format

A fixture JSON is a complete **project payload** with three top-level keys:

```json
{
  "next_id": <number>,
  "score": { ... },
  "ui": { ... }
}
```

`next_id` must be higher than every `id` value used anywhere in the document.

More generally: fixture JSON must follow the **current score structure** exactly, not just roughly resemble an older saved project.
If the runtime score schema changes, hand-authored fixtures may need to be updated to include newly-required fields before they will open again.

### 2 — Understand IDs

Every object (player, instrument, stave, flow, track, entry) has a numeric `id`. IDs are unique within the whole document — never reuse a number. Use a simple incrementing scheme, e.g. 1–50 for infrastructure (layout, players, instruments, staves) and 100+ for entries (tones, time signatures, clefs, etc).

**Stave identity note:** stave IDs are owned by the instrument and reused across every flow that includes that instrument. That means the same stave `id` can appear under multiple `flow.staves.by_id` maps in a fixture JSON, but it still refers to the same logical stave identity rather than a new independently-numbered object.

### 3 — Ticks

Time is measured in **ticks**. The tick system is:

| Duration       | Ticks |
| -------------- | ----- |
| Whole note     | 128   |
| Half note      | 64    |
| Quarter note   | 32    |
| Eighth note    | 16    |
| Sixteenth note | 8     |
| 32nd note      | 4     |

A 4/4 bar = 4 × 32 = **128 ticks**. A 3/4 bar = 3 × 32 = **96 ticks**.

`flow.length` must equal the total number of ticks across all bars in the flow.

### 4 — Grouping concerns and flows

- **One JSON file per concern** (e.g. `time-signatures.json`, `clefs.json`)
- **One flow per test case** within a file
- Related variations can share a flow (e.g. a mix of rhythmic patterns in a single 3/4 flow)
- Keep flow titles descriptive — they appear in the UI

### 5 — Using a real project as a template

The easiest way to get the correct JSON shape for a new fixture:

1. Build and open the app (`npm run start:dev`)
2. Create the score you want in the app
3. Save it (`Cmd+S`) — produces a `.soloc` file
4. Extract the JSON: `zcat <path/to/file.soloc> | python3 -m json.tool > my-fixture.json`
5. Edit the JSON to clean up the title/subtitle, then move it to `tests/fixtures/source/<concern>/`
6. Run `npm run build:fixtures`

Using a freshly-saved project as the starting point matters because fixture JSON is expected to match the current persisted score structure.
If you instead copy an older fixture or hand-assemble JSON from memory, deserialisation can fail on newly-added required fields.

### 6 — Key JSON fields

#### `score.meta`

```json
"meta": {
  "app_version": "1.23.3",   ← MUST match the current app version (see Cargo.toml)
  "title": "My Fixture",
  "subtitle": "Visual Regression Fixture",
  ...
}
```

`app_version` must match the current `version` in `src-tauri/Cargo.toml`. The builder will fail with a clear error if this field is missing.

#### Entry types and their `type` tag

Entries in tracks use `"type"` as a serde tag:

| `"type"`          | Struct        | Key fields                                                   |
| ----------------- | ------------- | ------------------------------------------------------------ |
| `"Tone"`          | Note/rest     | `tick`, `duration`, `pitch.int`, `velocity.int`              |
| `"TimeSignature"` | Time sig      | `tick`, `beats`, `beat_type`, `draw_type`, `groupings`       |
| `"Tempo"`         | Tempo marking | `tick`, `text`, `beat_type`, `dots`, `bpm`, visibility flags |
| `"Clef"`          | Clef          | `tick`, `draw_as`, `pitch.int`, `offset`                     |
| `"KeySignature"`  | Key signature | `tick`, `offset`, `mode`                                     |
| `"Barline"`       | Barline       | `tick`, `barline_draw_type`                                  |

#### `TimeSignature` `draw_type` values

| Value | Meaning       |
| ----- | ------------- |
| `0`   | Hidden        |
| `1`   | Regular (4/4) |
| `2`   | Common time   |
| `3`   | Split common  |
| `4`   | Open          |

#### `Clef` `draw_as` values

| Value | Clef           |
| ----- | -------------- |
| `1`   | G (treble)     |
| `2`   | F (bass)       |
| `3`   | C (alto/tenor) |
| `4`   | Percussion     |

#### `KeySignature` placement

Key signatures go on the **flow master track** (the same track as `TimeSignature`), not on stave master tracks. To add a key signature, include it alongside the time sig in `by_tick`:

```json
"by_tick": { "0": [timeSigId, keySigId] }
```

`offset`: number of sharps (positive) or flats (negative). Range −7 to +7. `mode`: `0` = Major, `1` = Minor.

#### Tempo placement

Tempo entries also go on the **flow master track**. They can share a tick with `TimeSignature` or `KeySignature` entries.

```json
"by_tick": { "0": [timeSigId, tempoId] }
```

Use these v1 fields:

- `text`: string or `null`
- `beat_type`: `4` for quarter-note metronome marks in current fixtures
- `dots`: `0`
- `bpm`: integer in the inclusive `20..400` range
- `hidden`, `show_text`, `show_metronome`, `parenthesized`: booleans controlling display

Tempo markings are still useful notation fixtures, but they are no longer playback-neutral: transport playback now follows immediate absolute score tempos from the flow master track.

#### `Clef` `offset` values

The `offset` field is the visual offset from the middle stave line (line 3). Positive = lower on the staff (toward line 1), negative = higher.

| Clef       | `draw_as` | `pitch.int` | `offset` |
| ---------- | --------- | ----------- | -------- |
| G (treble) | `1`       | `67`        | `2`      |
| F (bass)   | `2`       | `53`        | `-2`     |
| C (alto)   | `3`       | `60`        | `0`      |
| C (tenor)  | `3`       | `60`        | `-2`     |
| Percussion | `4`       | `60`        | `0`      |

Clef entries go on the **stave master track**, not the flow master track. A mid-flow clef change is simply a second `Clef` entry at the desired tick.

#### Articulations

`Tone.articulations` is an array of integers (can be empty `[]` or list multiple):

| Value | Name            |
| ----- | --------------- |
| `0`   | Staccato        |
| `1`   | Tenuto          |
| `2`   | Staccatissimo   |
| `3`   | Staccato Tenuto |
| `4`   | Accent          |
| `5`   | Marcato         |
| `6`   | Stressed        |
| `7`   | Unstressed      |

#### Rests

Rests are **not** explicit entries. The renderer automatically fills any gap between `Tone` entries with the appropriate rest symbols. To produce a rest, simply omit a `Tone` entry for that time span within the flow.

#### `Pitch` `accidental` values

| Value | Meaning      |
| ----- | ------------ |
| `2`   | Double sharp |
| `1`   | Sharp        |
| `0`   | Natural      |
| `-1`  | Flat         |
| `-2`  | Double flat  |

#### `beat_type` for `TimeSignature`

Uses the `NoteDuration` repr: `1`=whole, `2`=half, `4`=quarter, `8`=eighth, `16`=sixteenth.

> **Important**: Only **Simple** time signatures are currently supported in the renderer.
> Simple = beats 1, 2, 3, or 4.
> Compound signatures used by the app, such as 6/8, 9/8, and 12/8, are now valid in fixtures and should be covered with representative variants.
> Complex signatures (5/4, 7/8, etc.) are still unsupported and should not be used in fixtures.

#### Wiring overview

A minimal one-stave instrument requires this set of objects:

```
Layout (id=0, always present)
  └── player_ids, flow_ids

Player (id=N)
  └── instrument_ids

Instrument (id=N)
  └── stave_ids

Flow (id=N)
  ├── master_id  → master Track (holds TimeSignature / KeySignature / Tempo entries)
  ├── player_ids
  └── staves.by_id
        └── Stave (id=N)
              ├── master_id  → stave master Track (holds Clef entries)
              └── track_ids  → voice Track (holds Tone / Barline entries)

score.tracks.by_id
  ├── flow master track
  ├── stave master track(s)
  └── voice track(s)
```

---

## Common gotchas and learnings

These are things that were non-obvious during fixture authoring. Read this section before authoring a new file to avoid repeating the same investigation.

### Rests are implicit — they are not entries in the track

This is the most surprising thing about the data model. There is **no `"Rest"` entry type**. Rests are rendered automatically for any tick range within a bar that has no `Tone` entry. The renderer calls `Notation.is_rest()` which simply checks whether the tone list for that time slot is empty.

Practical implications:

- **Whole-bar rest**: leave the voice track's `by_tick` empty (`{}`). The bar is fully empty, renderer emits a whole-bar rest symbol.
- **Half rest at start of bar**: put a `Tone` at tick 64 only. Ticks 0–63 have no entry → half rest rendered automatically.
- **Alternating eighth note / eighth rest**: place `Tone` entries at ticks 0, 32, 64, 96, …; skip ticks 16, 48, 80, 112, … The renderer fills the skipped slots with eighth rests.
- **Empty bass stave**: a voice track with no notes still needs `"entries": { "by_tick": {}, "by_id": {} }`. Do NOT omit the `entries` key or the `by_tick`/`by_id` maps — the deserialiser expects them.

### Time signatures: Simple and Compound are valid fixture coverage, Complex is still blocked

Fixtures should now cover both Simple and Compound time signatures where the renderer supports them.
That includes common compound variants such as 6/8, 9/8, and 12/8, plus any other compound signatures the current app can open correctly.

Complex signatures (5/4, 7/8, etc.) are still not valid fixture targets. Avoid them until the renderer and load path support them end to end.

### `by_tick` keys are JSON strings, not numbers

Even though ticks are integers, the `by_tick` map's keys must be JSON strings:

```json
"by_tick": { "0": [100], "32": [101], "128": [102] }
```

Not `{ 0: [100] }`. This is a Rust `BTreeMap<u32, Vec<u32>>` serialised via serde, which serialises map keys as strings.

### Multiple entries at the same tick — `by_tick` values are arrays

A tick can have more than one entry. For example, `TimeSignature`, `KeySignature`, and `Tempo` can share tick 0:

```json
"by_tick": { "0": [100, 101] },
"by_id": {
  "100": { "type": "TimeSignature", ... },
  "101": { "type": "KeySignature", ... }
}
```

For key changes mid-flow, add a second `KeySignature` entry at the target tick alongside any other entries at that tick.

### Deserialisation errors usually mean the fixture no longer matches the current score structure

If opening a fixture fails with an error like `Failed to deserialize project file: missing field 'hidden'`, the fixture JSON is missing a field that now exists in the runtime score schema.

The most common current example is directive entries needing an explicit `hidden` flag:

- `TimeSignature`
- `Clef`
- `KeySignature`
- `Tempo`

For these entries, set `"hidden": false` unless you are intentionally testing hidden rendering.
When in doubt, re-save a similar score from the current app and diff that JSON against the fixture you are editing.

### Key signatures live on the flow master track, not stave masters

`KeySignature` entries belong to the same track as `TimeSignature` — the **flow master track** (`flow.master_id`). This is a different track from the stave master tracks (which hold `Clef` entries). Getting the track wrong means the key signature is silently ignored.

### Clefs live on the stave master track, not the flow master

Conversely, `Clef` entries go on the **stave master track** (`stave.master_id`), not the flow master. A mid-flow clef change is just a second `Clef` entry at the target tick in the same stave master track.

### C clef offset direction is counterintuitive

The `offset` field on a `Clef` entry is the visual offset from the middle stave line, with **positive = toward line 1 (lower on the page)**. This is the opposite of what you might expect:

- `offset: 0` → middle line = alto C clef
- `offset: -2` → tenor position (two steps higher on the page)

This confused the sign convention during authoring. Use the table in the Key JSON fields section above.

### Stave IDs are shared across all flows in a file

A piano has stave IDs 3 and 4. All flows in the same fixture file reference the same stave IDs. The flow's `staves.by_id` block is just a local wiring declaration for that flow — each flow re-declares staves 3 and 4 independently. Do not try to allocate new stave IDs per flow; keep them the same throughout the file.

### Use a clean per-flow ID scheme

With multiple flows per file, ID collisions are easy. A scheme that works well:

- **Flow IDs**: 10, 20, 30, 40, … (multiples of 10)
- **Infrastructure tracks**: flow_id+1 (master), flow_id+2 (stave3 master), flow_id+3 (voice3), flow_id+4 (stave4 master), flow_id+5 (voice4)
- **Entries**: 100–199 for flow 10, 200–299 for flow 20, etc.
- **`next_id`**: one above the highest ID used (e.g. 500 for four flows)

This keeps IDs visually scannable and avoids accidental clashes when adding new flows.

### The `groupings` field on `TimeSignature` is required

Every `TimeSignature` entry needs a `groupings` array. It describes how beats group for beam slurring:

| Time sig | `groupings` |
| -------- | ----------- |
| 4/4      | `[2, 2]`    |
| 3/4      | `[3]`       |
| 2/4      | `[2]`       |

Omitting it causes a deserialisation error.

---

## Existing fixtures

| File                                                                    | Flows | What it tests                                                                                |
| ----------------------------------------------------------------------- | ----- | -------------------------------------------------------------------------------------------- |
| `source/time-signatures/four-four.json`                                 | 4     | Quarter notes · Whole and half notes · Eighth note run (scale up/down) · Half + two quarters |
| `source/time-signatures/three-four.json`                                | 4     | Quarter notes · Dotted half note · Dotted quarter + eighth + quarter · Six eighths           |
| `source/time-signatures/changing.json`                                  | 3     | 4/4 → 3/4 · 3/4 → 4/4 · 4/4 → 2/4 → 4/4                                                      |
| `source/clefs/clefs.json`                                               | 3     | Treble + bass clefs · Alto C clef · Mid-flow treble → bass clef change                       |
| `source/key-signatures/key-signatures.json`                             | 4     | G major (1 sharp) · F major (1 flat) · A minor · Mid-flow key change (C → D major)           |
| `source/rhythms/written-durations-four-four.json`                       | 29    | 4/4 written-duration parser scenarios, one flow per scenario title                           |
| `source/rhythms/written-durations-three-four.json`                      | 24    | 3/4 written-duration parser scenarios, one flow per scenario title                           |
| `source/rhythms/written-durations-three-four-six-eight-comparison.json` | 24    | Comparison flows that show one 3/4 bar followed by the same music after a 6/8 change         |
| `source/rhythms/written-durations-compound.json`                        | 17    | Compound-meter written-duration parser scenarios across 6/8, 9/8, 12/8, and 15/8             |
| `source/rests/rests.json`                                               | 4     | Whole bar rest · Half rests · Quarter rests · Eighth rests (alternating note/rest)           |
| `source/articulations/articulations.json`                               | 4     | Staccato · Tenuto · Accent + Marcato · Mixed articulations (one note each type)              |
| `source/ledger-lines/ledger-lines.json`                                 | 3     | High notes ascending into treble ledger lines · Low notes into bass ledger lines · Extremes  |

The written-duration rhythm fixtures now cover the existing simple-meter and compound-meter corpora from `get_written_durations.rs`.
The 3/4-versus-6/8 comparison fixture reuses the 3/4 source corpus and adds a mid-flow `TimeSignature` change so the same tones can be visually compared under both meters.

---

## Technical background

The builder lives in `src-tauri/src/bin/build_fixtures.rs`. It uses the shared
`src-tauri/src/soloc.rs` module (the same code used by the production save path), so fixtures
are guaranteed to be structurally identical to files saved by the app.

Container format: `SOLOC` magic + binary header + gzip-compressed JSON chunk.
See `_bmad-output/implementation-artifacts/tech-spec-binary-soloc-format.md` for the full spec.

When the `.soloc` container format changes, rebuild fixtures by running `npm run build:fixtures`.
