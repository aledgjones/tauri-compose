# Debugger score fixtures

This folder contains JSON fixture files used by the score debugger.

This document has a dual purpose:

- help humans add and maintain fixtures consistently
- give coding agents a concrete contract for how fixture files in this folder should be prepared

## What belongs here

Each JSON file in this folder is auto-discovered by the debugger and shown as a selectable fixture.

Use this folder for:

- focused notation regression examples
- entry-specific galleries such as clefs, barlines, key signatures, and time signatures
- realistic musical excerpts that exercise current engraving behavior
- longer composed examples that stress spacing, bar structure, and multi-staff rendering

Do not use this folder for:

- temporary scratch data
- invalid JSON
- speculative formats not supported by the current fixture loader

## Current design principles

### 1. Prefer entry-type grouping

Fixture files should usually be named by notation concern:

- `accidentals.json`
- `barlines.json`
- `clefs.json`
- `key-signatures.json`
- `time-signatures.json`
- `written-durations.json`

If a fixture is not entry-specific, it should still have a clear single purpose, for example:

- an original composed stress-test piece
- a realistic excerpt collection

### 2. Keep flows focused

A flow should usually demonstrate one thing clearly.

Good:

- `End-start repeat with a clef change`
- `Cancellation to C from three sharps`
- `Common time redraw`

Less good:

- `Mixed notation stuff`
- one flow that combines many unrelated behaviors unless the collision itself is the test

Exception:

- a single reference flow is acceptable when the purpose is side-by-side comparison, such as a barline draw-type overview

### 3. Make cases useful

The `cases` array is the index into a flow.
Each case should point to a musically meaningful measure and have a name that tells the reader what to inspect.

Good case names:

- `Hidden time signature with zero-width spacing`
- `End-start repeat aligned with a key-signature change`
- `Subject in the upper voice`

## Loader expectations

The debugger loads every `*.json` file in this folder.
There is no manifest file.
That means:

- every JSON file here should be intentional
- filenames should be stable and descriptive
- deleting or renaming a file changes debugger-visible fixture inventory

## File format

Each fixture file must match this shape:

```json
{
  "id": "unique-id",
  "name": "Visible name",
  "description": "Short summary",
  "payload": {
    "title": "Debugger Fixture — Name",
    "activeTab": 1,
    "instruments": [],
    "flows": []
  },
  "cases": []
}
```

## Top-level fields

### `id`

- machine-readable
- unique across all files in this folder
- use lowercase kebab-case

### `name`

- human-readable fixture name shown in the UI
- should be short and stable

### `description`

- one sentence describing what the fixture is for
- say what is being exercised, not implementation details

### `payload.title`

- visible debugger title
- prefer the form `Debugger Fixture — ...`

### `payload.activeTab`

- currently set to `1` in existing fixtures
- keep consistent unless there is a known reason to change it

## Instrument definitions

`payload.instruments` declares reusable instrument identities used by parts.

Example:

```json
{
  "id": "piano",
  "definitionId": "keyboard.piano"
}
```

Rules:

- `id` must be unique within the file
- `parts[].instrumentId` must reference one of these ids
- reuse the same instrument declaration across flows in the same file
- use existing app definition ids

Common examples used so far:

- `strings.violin`
- `strings.viola`
- `strings.violoncello`
- `keyboard.piano`
- `unpitched-percussion.snare`

## Flows

Each flow is a self-contained score example.

Shape:

```json
{
  "title": "Flow title",
  "flowLength": 256,
  "timeSignatures": [],
  "barlines": [],
  "keySignatures": [],
  "parts": []
}
```

### Required flow fields

- `title`
- `flowLength`
- `timeSignatures`
- `parts`

### Optional flow fields

- `barlines`
- `keySignatures`

## Tick model and measure planning

Fixtures in this folder currently use these timing assumptions:

- quarter note = `32` ticks
- in `4/4`, one bar = `128` ticks
- in `3/4`, one bar = `96` ticks

Examples:

- 2 bars of `4/4` => `flowLength: 256`
- 8 bars of `4/4` => `flowLength: 1024`
- 64 bars of `3/4` => `flowLength: 6144`

Agent instruction:

- derive `flowLength` from the real bar count rather than guessing
- place barlines, signature changes, and cases on exact measure boundaries unless the purpose is an off-boundary edge case

## Time signatures

Time signatures are required for every flow.

Example:

```json
{
  "tick": 0,
  "beats": 4,
  "beatType": 4
}
```

Optional `drawType` may be used when the fixture is specifically testing time-signature rendering variants.

Use time-signature fixtures for:

- numeric form
- common time
- cut time
- open time
- hidden time signatures

Important:

- hidden time signatures are valid as a rendering case
- hidden time signatures should still be placed intentionally at the relevant tick

## Barlines

Use `barlines` when a flow needs explicit boundary rendering.

Guidelines:

- keep a valid opening barline at tick `0` when testing repeat openings
- when testing a boundary type inside a simple example, prefer placing it at the start of measure 2
- add a closing final barline when it helps make the example readable

Use barline collision flows when specifically testing end-start repeat interaction with:

- clef changes
- key-signature changes
- time-signature changes
- combinations of the above

## Key signatures

Key-signature fixtures should make the visual intent obvious.

Good patterns:

- one flow for introduction of a signature
- one flow for cancellation to zero offset
- one flow for repeat-boundary interaction
- one flow showing clef-dependent vertical placement

When testing cancellation to zero offset:

- ensure there is a previous nonzero key signature
- place the zero-offset signature where the natural-cancellation symbols should appear

## Clefs

Clef fixtures should model actual clef changes.

Rules:

- do not encode absence using a hidden clef type
- if there is no clef change, omit the clef entry
- use clef changes only where the flow intends to test clef rendering or a boundary collision

## Parts

Each part belongs to one instrument and optionally one stave.

Examples:

```json
{
  "instrumentId": "violin",
  "tones": []
}
```

```json
{
  "instrumentId": "piano",
  "staveIndex": 1,
  "tones": []
}
```

Guidelines:

- single-staff instruments typically omit `staveIndex`
- grand-staff instruments such as piano use `staveIndex: 0` and `staveIndex: 1`
- keep cross-staff examples readable rather than overly dense

## Tones

Each tone needs:

- `tick`
- `duration`
- `pitchInt`

Optional fields used so far:

- `accidental`
- `velocity`
- `articulations`

Guidelines:

- prefer rhythms that align clearly with the current time signature unless the flow is explicitly testing syncopation or splitting
- for long-form musical fixtures, vary rhythms enough to exercise notation logic, but avoid unreadable density
- use explicit `accidental` only when you are intentionally testing or forcing a spelling

## Cases

Each case points at one measure within one flow.

Shape:

```json
{
  "flowTitle": "Flow title",
  "name": "What to inspect",
  "measure": 2,
  "timeSignature": "4/4"
}
```

Rules:

- `flowTitle` must match a flow title exactly
- `measure` is 1-based
- `timeSignature` should reflect the displayed context for that case
- use multiple cases for long flows so the UI remains navigable

Agent instruction:

- after creating or editing flows, always update `cases` in the same change
- never leave orphaned case references to deleted or renamed flows

## Naming conventions

### File names

- lowercase kebab-case
- plural when the file is a category: `barlines.json`
- descriptive singular concept for bespoke fixtures only when clearer

### Flow titles

- sentence case
- specific and inspection-oriented
- include the collision or transition being tested when relevant

### IDs

- lowercase kebab-case
- stable over time

## How to build a new fixture

### Human workflow

1. Decide the notation concern.
2. Choose whether it belongs in an existing entry-type file or a new purpose-specific file.
3. Reuse existing instrument ids and pitch ranges where possible.
4. Sketch the bar plan in ticks first.
5. Add `timeSignatures` and any boundary changes.
6. Add parts and tones.
7. Add cases for the important measures.
8. Validate the JSON.
9. Open it in the debugger and visually inspect the intended measures.

### Agent workflow

1. Read the existing files in this folder before introducing a new pattern.
2. Preserve the current schema exactly.
3. Prefer the smallest coherent change.
4. Keep one file centered on one notation domain unless the user asked for a broader musical example.
5. If a file has been user-edited recently, re-read it before patching.
6. After editing, validate the file for JSON errors.
7. Summarize the structural change, not raw JSON content.

## Fixture quality checklist

Before finishing, verify all of the following:

- JSON is valid
- `id` is unique
- flow titles are unique within the file
- `cases[].flowTitle` matches a real flow title
- `flowLength` matches the actual intended bar count
- instrument ids are declared and referenced correctly
- boundary ticks are on the intended measures
- the fixture purpose is obvious from file name, flow name, and description
- realistic excerpts are original or public domain

## Preferred evolution path

When a file starts becoming confusing:

- split by entry type first
- split by collision type second
- use dedicated long-form files for composed or realistic examples
- avoid returning to broad catch-all gallery files unless the broad comparison is itself the point

## In short

For humans:

- make fixtures easy to inspect
- keep them musical and intentional
- name them by what they prove

For agents:

- read before editing
- keep schema and conventions stable
- prefer focused flows, exact measure math, and validated JSON
