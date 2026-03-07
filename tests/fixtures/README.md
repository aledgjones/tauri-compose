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
│   ├── time-signatures/
│   │   ├── four-four.json
│   │   ├── three-four.json
│   │   └── changing.json
│   └── ...
└── built/                 ← gitignored .soloc output
    └── time-signatures/
        ├── four-four.soloc
        ├── three-four.soloc
        └── changing.soloc
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

### 2 — Understand IDs

Every object (player, instrument, stave, flow, track, entry) has a numeric `id`. IDs are unique within the whole document — never reuse a number. Use a simple incrementing scheme, e.g. 1–50 for infrastructure (layout, players, instruments, staves) and 100+ for entries (tones, time signatures, clefs, etc).

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

| `"type"`          | Struct        | Key fields                                             |
| ----------------- | ------------- | ------------------------------------------------------ |
| `"Tone"`          | Note/rest     | `tick`, `duration`, `pitch.int`, `velocity.int`        |
| `"TimeSignature"` | Time sig      | `tick`, `beats`, `beat_type`, `draw_type`, `groupings` |
| `"Clef"`          | Clef          | `tick`, `draw_as`, `pitch.int`, `offset`               |
| `"KeySignature"`  | Key signature | `tick`, `offset`, `mode`                               |
| `"Barline"`       | Barline       | `tick`, `barline_draw_type`                            |

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
> Simple = beats 1, 2, 3, or 4 (any beat type).
> Compound (6/8, 9/8, etc.) and Complex (5/4, 7/8, etc.) are `todo!()` and will **crash the app**.

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
  ├── master_id  → master Track (holds TimeSignature entries)
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

## Existing fixtures

| File                                     | Flows | What it tests                                                                                |
| ---------------------------------------- | ----- | -------------------------------------------------------------------------------------------- |
| `source/time-signatures/four-four.json`  | 4     | Quarter notes · Whole and half notes · Eighth note run (scale up/down) · Half + two quarters |
| `source/time-signatures/three-four.json` | 4     | Quarter notes · Dotted half note · Dotted quarter + eighth + quarter · Six eighths           |
| `source/time-signatures/changing.json`   | 3     | 4/4 → 3/4 · 3/4 → 4/4 · 4/4 → 2/4 → 4/4                                                      |

---

## Technical background

The builder lives in `src-tauri/src/bin/build_fixtures.rs`. It uses the shared
`src-tauri/src/soloc.rs` module (the same code used by the production save path), so fixtures
are guaranteed to be structurally identical to files saved by the app.

Container format: `SOLOC` magic + binary header + gzip-compressed JSON chunk.
See `_bmad-output/implementation-artifacts/tech-spec-binary-soloc-format.md` for the full spec.

When the `.soloc` container format changes, rebuild fixtures by running `npm run build:fixtures`.
