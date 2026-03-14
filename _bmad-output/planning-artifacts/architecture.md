---
stepsCompleted:
  - step-01-context
  - step-02-brownfield-analysis
  - step-03-core-model
  - step-04-boundaries
  - step-05-migration
  - step-06-regression
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/archive/2026-03-multi-flow-score/prd.md
  - docs/architecture.md
  - docs/data-models.md
  - docs/development-guide.md
  - docs/project-context.md
  - _bmad-output/implementation-artifacts/archive/2026-03-multi-flow-score/tech-spec-compound-meter-written-duration-and-beaming-support.md
  - _bmad-output/implementation-artifacts/archive/2026-03-multi-flow-score/notation-edge-cases-written-durations.md
  - src-tauri/src/score/components/duration.rs
  - src-tauri/src/score/entries/mod.rs
  - src-tauri/src/score/entries/tone.rs
  - src-tauri/src/score/entries/time_signature.rs
  - src-tauri/src/score/entries/tempo.rs
  - src-tauri/src/score/tracks.rs
  - src-tauri/src/files.rs
  - src-tauri/src/soloc.rs
  - src-tauri/src/playback/transport.rs
  - src-tauri/src/ui/sequencer.rs
  - src-tauri/src/instructions/parse/get_written_durations.rs
  - src-tauri/src/instructions/parse/get_beams.rs
  - src-tauri/src/instructions/parse/get_horizontal_spacing.rs
workflowType: "architecture"
project_name: "compose"
user_name: "Aled"
date: "2026-03-12"
status: "draft"
classification:
  projectType: desktop_app
  domain: music_notation
  complexity: high
  projectContext: brownfield
lastEdited: "2026-03-12"
---

# Architecture Decision Document - Foundational Tuplet Support

`_bmad-output/planning-artifacts/prd.md` is the active requirements source for this document. `_bmad-output/planning-artifacts/archive/2026-03-multi-flow-score/prd.md` remains completed background context only.

## Executive Summary

The recommended architecture for foundational tuplet support uses a track-local container model.

A tuplet is a first-class entry on an instrument track that consumes a host-track span and contains a local child track-like event stream. Child content keeps normal written values. The tuplet wrapper retains the musical context that makes those written values meaningful inside warped time:

- consumed host duration
- local printed duration
- base written unit
- grouping metadata

This preserves the current Compose mental model of tracks as streams of events, keeps Rust as the source of truth, avoids frontend-owned rhythmic state, and limits exact fractional math to projection seams instead of forcing every persisted note in the score to adopt a new global duration primitive.

## Brownfield Problem Statement

Compose currently assumes one flat tick space with `32 ticks per quarter` and uses that same timing lattice for:

- persisted tone positions and durations
- parser derivation of written durations
- spacing calculations
- hit-testing references
- playback stepping

That is sufficient for binary notation and unsafe for exact triplets. Foundational tuplet support must solve exact musical timing without turning tuplets into visual decoration or approximate playback hacks.

The agreed direction is to keep the underlying data model tick-based while introducing a clean tuplet-span structure that expresses tuplets as bounded local metric containers inside a host track while still owning child tone entries.

## Agreed Model Invariants

These are the core rules the architecture now depends on.

## Non-Negotiable Rules

- One tone is always represented by exactly one stored event.
- A tone is owned by where it starts.
- If a tone starts inside a tuplet span, it is stored in that tuplet's child `entries`.
- If a tone starts outside a tuplet span, it is stored on the host track, even if it extends into the tuplet.
- A host-track tone may not start at a tick already occupied by a tuplet span.
- Crossing a tuplet boundary never by itself forces a storage split.
- Boundary splits are introduced only by notation, rendering, or playback derivation.
- If a tone's start position moves across the tuplet boundary, ownership moves with that start position, but the tone still remains one stored event.

## Ownership Decision Table

| Case                                               | Where tone starts                          | Where it is stored                                                  | What derivation does                                                                |
| -------------------------------------------------- | ------------------------------------------ | ------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Tone fully outside tuplet                          | Host track                                 | Host track                                                          | Draws normally; no tuplet split needed                                              |
| Tone starts outside and extends into tuplet        | Host track                                 | Host track                                                          | Splits only in derived notation/playback at the tuplet boundary                     |
| Tone starts inside tuplet and stays inside         | Tuplet child `entries`                     | Tuplet child `entries`                                              | Draws normally inside the tuplet                                                    |
| Tone starts inside tuplet and extends out          | Tuplet child `entries`                     | Tuplet child `entries`                                              | Splits only in derived notation/playback at the tuplet boundary                     |
| Tone start is moved across the boundary by editing | Whichever side now owns the new start tick | Ownership changes to the new start side, but still one stored event | Derivation follows the new ownership and still introduces any needed boundary split |

Plain-language summary:

- Start position decides ownership.
- Duration crossing a tuplet boundary leaves ownership unchanged.
- Drawing and scheduling introduce boundary-aware fragments from a single stored tone.

### 1. Tuplet Governs A Host-Track Window As A Container

A tuplet is a first-class container entry that marks a host-time window and owns child content inside that window. Host ticks keep their ordinary global meaning throughout the track, including inside the tuplet span.

Implications:

- A tuplet entry governs a host-track time interval.
- Host ticks retain the same meaning inside and outside the tuplet, and the track keeps one global length model.
- Tuplets contain child tone entries for tones whose start position lies inside the governed interval.
- Ordinary same-track note entry may not start on the host track at a tick occupied by a tuplet span; if the tone starts there, it must be created in the tuplet child frame.
- Notes may extend into or out of the interval, with boundary splitting and tie continuity handled the same way tones are split across barlines.
- Creating a tuplet over existing material establishes the tuplet span first; ownership of every tone entry is then determined by the tick where that tone starts, while any needed boundary split is deferred to derivation.

### 1a. Storage Rule For Boundary-Crossing Tones

This rule should be explicit because it is easy to describe imprecisely.

- Each tone is represented by exactly one stored event.
- A tone is owned by the context where it starts.
- If a tone starts inside the tuplet, it belongs in the tuplet child `entries`, even if its duration extends out of the tuplet.
- If a tone starts outside the tuplet, it remains a host-track tone, even if its duration extends into the tuplet.
- Crossing the tuplet boundary never by itself forces a storage split.
- The split is introduced only when notation, rendering, or playback derives boundary-aware fragments from that persisted tone.

This keeps the track model simple while still allowing the score and transport to respect the tuplet boundary musically.

### 2. Tuplet Containers Behave Like Local Metric Frames

The tuplet container should feel like a small local metric frame for derivation purposes:

- it has a local tick space
- it has a local printed length
- it stores child tone entries in a track-like event stream
- it defines local grouping rules over that child content
- rests remain derived from absence of tones inside the governed local slots

This keeps the model familiar while still letting tuplets own the notes that musically belong inside them.

### 3. Written Values Stay Written Values

A note value follows the same literal tick-to-written-duration mapping on both sides of a tuplet boundary.

Examples:

- a written eighth inside a triplet remains a written eighth
- a written quarter inside a triplet remains a written quarter

Literal mapping rule:

- `16` ticks is always an eighth-note worth of written duration
- `32` ticks is always a quarter-note worth of written duration
- the tuplet changes how those ticks project through musical time, not how those tick counts map to written note values

Refinement:

- existing written values are preserved unchanged when structurally valid inside the child frame
- if a note crosses a tuplet boundary, it must still be split at that boundary
- the in-tuplet segment is represented as normal notation inside the child frame and the out-of-tuplet segment remains in its owning context

This preserves both simplicity and musical correctness.

### 4. Tuplet Must Retain Musical Context Explicitly

Tuplets retain musical intent and provide the local metric context needed for derivation.

The wrapper must retain enough information to answer:

- what span it consumes in host time
- what printed span it represents locally
- what the base written unit is
- how the local grouping should behave

### 5. `tone_id` Remains Authoritative For Tones

Tone selection and mutation authority remains tone-id based. Local tick and other nested location data are lookup aids, not identity.

### 6. Host-Level Directives Stay Out Of MVP Tuplet Spans

For MVP:

- tempo, clef, key-signature, and time-signature entries remain host-level concerns
- host-level directive support at inner tuplet positions is deferred
- this is a refinement problem for later work, not a model-breaking flaw

### 7. Barlines Are Parse-Time Structure, Not Storage Structure

Tracks remain event streams. Cross-barline tuplets are therefore not a storage-model contradiction. They are a future parse and notation problem.

## Recommended Persisted Data Model

## Tuplet As A New Track Entry Variant

Add a new persisted entry variant on instrument tracks:

```rust
Entry::Tuplet(Tuplet)
```

Recommended persisted shape:

```rust
pub struct Tuplet {
    pub id: UniqueId,
    pub tick: Tick,
    pub consumed_ticks: Ticks,
    pub local_length_ticks: Ticks,
    pub base_note_value: NoteDuration,
    pub groupings: Vec<u8>,
  pub entries: Entries,
}
```

Where:

- `tick` is the host-track start position
- `consumed_ticks` is the host-track duration the tuplet governs as a local rhythmic window
- `local_length_ticks` is the printed local duration inside that tuplet frame
- `base_note_value` retains the musical unit the tuplet is built from
- `groupings` expresses the local metric grouping, with sensible defaults
- `entries` stores the child tone stream for tones whose start position lies inside the tuplet span

Authoritative note entries remain in their owning context: host track for tones that start outside tuplets and child `entries` for tones that start inside tuplets. A tone may cross a tuplet boundary without being storage-split. The tuplet entry supplies the local rhythmic context used when preparing note entry, deriving notation fragments, hit targets, ties, and playback timing.

### Example: Eighth-Note Triplet

For an eighth-note triplet in the time of two eighths:

- `tick = 32` if it starts on beat 2 in `2/4`
- `consumed_ticks = 32`
- `local_length_ticks = 48`
- `base_note_value = Eighth`
- `groupings = [3]`

This means the tuplet frame contains `48` local ticks of printed notation that map into `32` host ticks of performed time.

## Why This Model Fits Compose

This model preserves several existing Compose strengths:

- tracks remain streams of entries
- rests remain derived from absence of tones
- notation stays derived from musical structure
- score truth remains backend-owned
- tuplets become first-class structural entries rather than renderer annotations

It also avoids a repo-wide rewrite of all persisted note timing into global rational durations.

## Child Track Shape

The child should mirror the current track mental model closely enough that parser logic can recurse into it without needing a totally separate notation worldview.

Recommended MVP child-content scope:

- tones
- nested tuplets
- anything naturally attached to tones such as articulations through the tone itself

Deferred from MVP child content:

- clefs
- key signatures
- tempo entries
- time signatures

This keeps MVP simple while preserving a path to nested tuplets.

## Projection Model

Tuplets do not remove fractional calculations completely. They localize them.

Inside the child:

- all timing is ordinary local tick timing
- parser logic can think in written values and local spacing

At the boundary:

- child local positions are projected into consumed host time

Projection formula:

$$
projected\_host\_tick = tuplet.tick + local\_tick \times \frac{consumed\_ticks}{local\_length\_ticks}
$$

For the eighth-note triplet example:

$$
projected\_host\_tick = tuplet.tick + local\_tick \times \frac{32}{48} = tuplet.tick + local\_tick \times \frac{2}{3}
$$

This means the system stays tick-based in storage, but exact arithmetic is still required when flattening tuplet internals into host musical time for playback and certain parser decisions.

## Creation And Reconciliation Rules

## Tuplet Creation Adds A Container And Assigns Ownership By Start Position

Creating a tuplet over existing same-track material adds the tuplet span and assigns storage ownership by tone start position.

Agreed MVP behavior:

- tuplets govern host-track material in their span
- notes whose start position lies inside the tuplet are stored in the child frame
- existing written values are preserved unchanged when possible
- content beyond the tuplet span remains outside unchanged
- new note entry inside the span prepares onsets and durations through the active tuplet multiplier, similar to how tempo context affects later projection without changing the musical intent of the prepared note
- user-facing duration edits may pass across the tuplet boundary as one continuous drag gesture, with the system reconciling ownership under the hood

Ownership rule for MVP:

- each tone remains one stored event
- the owning context is chosen by the tone's start position
- derivation introduces any needed boundary-aware fragments

### Cross-Boundary Notes

If a note crosses the tuplet boundary, the authoritative persisted tone remains one continuous tone with one duration. The split happens when notation, rendering, or playback derives the tone through the tuplet boundary, preserving continuity through ties and projection.

Agreed example:

- in `2/4`, a half note beginning at bar start and a triplet created on beat 2
- keep the tone persisted as one continuous host-track tone
- derive one fragment before the tuplet and one fragment inside the tuplet when drawing or scheduling
- let the remaining local tuplet space derive to rest(s)

Symmetric rule:

- if a tone starts inside the tuplet and extends outward, it remains one child entry inside the tuplet
- derivation then creates the in-tuplet and out-of-tuplet fragments from that single child entry

The parser, renderer, and playback projection perform the same kind of boundary-aware split Compose already uses around barlines and ties, with tuplet spans acting as another local metric boundary. A user can drag a tone duration across the tuplet area as one interaction while persisted storage remains one continuous tone.

The important implementation consequence is:

- track storage answers "what tone exists and how long is it?"
- storage ownership answers "where does this tone start?"
- derivation answers "how should that tone be shown or scheduled when it crosses a tuplet boundary?"

### Derivation Rules

- derivation preserves visible written values for any segment that already fits its owning context
- derivation introduces splits at metric boundaries such as tuplet edges and barlines
- boundary splitting changes which derived fragment belongs to which context
- prepared written durations keep the same musical meaning inside and outside the tuplet
- tuplet context changes projection through time, while tick counts keep the same written-duration mapping

Concrete implication:

- if a crossing tone leaves `16` ticks of remainder inside the tuplet, that remainder is drawn as an eighth

Edit-time storage rule:

- edit commit preserves one stored event per tone
- ownership changes only when the tone's start position crosses the tuplet boundary

## Sequencer UX Contract

Tuplet creation is not complete at the container-creation seam alone. The backend and sequencer must together provide a deterministic handoff from popover confirmation into sequencer-local editing.

Required sequencer contract for MVP:

- confirming tuplet creation reveals the governed region immediately on the active track
- the created tuplet container becomes the active selection on completion
- the visible region stays bounded and legible before any child notes are entered
- the region includes a centered `3:2` annotation, anchored to the governed span and allowed to remain sticky at the top of the sequencer viewport while that span is in view
- the annotation must not rely on a solid occluding background that hides notes underneath it
- the sequencer shows slightly compressed internal tick markers inside the governed span so tuplet-local slot structure is visible without changing the meaning of surrounding host ticks
- empty local slots remain targetable and visually quieter than real notes, but they must not collapse away after creation or after deleting a child tone

These are product obligations, not optional polish. They define how the tuplet container becomes actionable in the current sequencer-first workflow.

## Focus, Feedback, And Accessibility Contract

Rust remains authoritative for score truth and edit validation, but some tuplet UX requirements are intentionally frontend-local and ephemeral.

Required interaction contract for MVP:

- popover focus lands on the ratio field when the tuplet popover opens
- the popover closes cleanly on accepted input, blur, or Escape
- unsupported tuplet edits follow the product's existing rejection behavior and surface local inline feedback at the interaction point; MVP does not require a tuplet-specific global error flow
- sequencer selection states for tuplet container, child note, and empty slot must remain distinguishable through boundary or placement treatment in addition to color so the active target is still understandable under accessibility tooling and reduced-color perception

This keeps backend ownership intact while making the UX contract explicit enough that implementation does not drift into hidden focus jumps or color-only state communication.

## Worked Example: `2/4` Half Note Crossing A Beat-2 Triplet

This example pressure-tests the agreed model with a concrete case.

### Musical Setup

- bar context: `2/4`
- host tick lattice: `32 ticks per quarter`
- user creates an eighth-note triplet on beat 2
- user intent includes a half note beginning at bar start

The key agreed UX rule for this case is:

- split at the tuplet boundary during derivation
- preserve written values inside the tuplet when structurally valid
- let the tuplet container context provide the musical meaning of the derived local content

Related example:

- if a quarter note is moved right by an eighth so only `16` ticks remain before the tuplet boundary and `16` ticks continue inside the tuplet
- the derived notation should read as an eighth rest, then an eighth note tied to a triplet eighth note, then two triplet eighth rests
- this follows the literal tick-mapping rule

### State Before Tuplet Creation

Host track entries:

```text
Tone 101
- tick: 0
- duration: 64
- meaning: written half note from bar start
```

At this point there is no tuplet entry. The track is a normal flat event stream.

### User Action

Create an eighth-note triplet at beat 2.

Tuplet definition:

- host `tick = 32`
- `consumed_ticks = 32`
- `local_length_ticks = 48`
- `base_note_value = Eighth`
- `groupings = [3]`

This means:

- the tuplet consumes the second quarter of the bar in host time
- the child behaves like a local printed span of three eighths

### Reconciliation Rule Applied

The original half note crosses the tuplet boundary at host tick `32`, so derivation must split it at that boundary even though persisted track state keeps one continuous tone.

Persisted state:

```text
Host track
- Tone 101: tick 0, duration 64
- Tuplet 201: tick 32, consumed_ticks 32, local_length_ticks 48,
  base_note_value Eighth, groupings [3]
```

Notes:

- persisted track state contains one half-note tone with one duration
- derivation produces a quarter-note fragment before the tuplet and a quarter-note fragment inside the tuplet frame
- the final local eighth of the tuplet frame is empty and therefore derives to rest during parsing

Derived result facts:

- storage remains one tone across the boundary
- the tuplet-side note value is a quarter
- the tuplet-side rest is derived from absence of tone content

### Derived Notation Result

The resulting notation should read as:

```text
beat 1: quarter note
beat 2 tuplet: quarter note tied from beat 1, then eighth rest
```

Inside the tuplet frame:

- the derived written quarter occupies the first two local eighth slots
- the final local eighth slot is empty and therefore renders as rest
- the tuplet bracket/number expresses that these three local eighth slots occupy one host quarter beat

### Parser Interpretation

The parser handles this in two layers.

Host layer:

- sees a half note beginning at bar start
- sees a tuplet container beginning at host tick `32`

Tuplet-aware derivation layer:

- evaluates the continuous tone against the tuplet boundary
- derives a written quarter from the tone portion that projects through local duration `0..32`
- derives a final local eighth rest from the empty tail span `32..48`

The parser derives two notation fragments from one stored tone because the tuplet boundary requires that reading.

Flattened result:

- host quarter notation before the tuplet
- tuplet-side quarter notation plus tuplet-side rest notation inside the tuplet frame
- tie continuity preserved across the host-to-tuplet boundary as a parser result over one persisted tone and two derived fragments

### Spacing Interpretation

Ambient host spacing provides width for the consumed second beat.

Tuplet-local spacing then computes child requirements for:

- one written quarter occupying two child eighth slots
- one derived child eighth rest in the final slot
- tuplet bracket and number

The final width rule is:

- if the child notation plus bracket needs more width than the ambient consumed host span, host spacing expands
- if surrounding host spacing is already wider, the child local x positions stretch to fill the available width

This is the same widest-needed-spacing principle already used in the host score.

### Selection Interpretation

Selection stays authority-first.

Host-side fragment:

- authoritative identity: `tone_id = 101`

Tuplet-side fragment:

- authoritative identity: `tone_id = 101`
- lookup context: `track_id`, `tuplet_path = [201]`, `local_tick = 0`, fragment boundary at host tick `32`

Derived child rest:

- no persisted tone id
- lookup context only: `track_id`, `tuplet_path = [201]`, `local_tick = 32`

Tuplet container:

- authoritative identity: `tuplet_id = 201`

### Playback Interpretation

Playback remains host-time based, but note preparation and projection are tuplet-aware.

Projection formula:

$$
projected\_host\_tick = 32 + local\_tick \times \frac{32}{48}
$$

For the derived tuplet-side quarter-note fragment:

- onset: local tick `0` projects to host tick `32`
- duration: local duration `32` projects to host duration $32 \times \frac{32}{48} = \frac{64}{3}$ host ticks

For the derived tuplet-side rest:

- onset: local tick `32` projects to host tick $32 + \frac{64}{3}$
- duration: local duration `16` projects to host duration $16 \times \frac{32}{48} = \frac{32}{3}$ host ticks

So playback sees the second beat as:

- sounded continuation over the first two triplet eighths
- silence over the final triplet eighth

This demonstrates the main architectural point:

- derived notation can stay in ordinary written values
- exact fractional math is still required when projecting tuplet-governed fragments back into host musical time

### Why This Example Matters

This example validates that the agreed model can express all of the following without changing the core rules:

- tuplets govern host-track space through a bounded local metric container
- tuplet-local derivation stays track-like
- written values are preserved when possible
- rests remain derived from absence of tones
- selection remains tone-id authoritative
- spacing can either expand or fit ambient width
- playback remains exact by projection rather than by flattening the entire data model into global rational storage

## Deletion Rule

Agreed MVP deletion behavior:

- deleting the tuplet deletes everything inside it

No dissolve-back-to-host-time behavior is required for MVP.

## Parser Architecture

## Recursive Local Parse, Then Flatten

The parser should treat tuplets as recursive local notation frames.

Recommended flow:

1. Parse the host track as an event stream.
2. When a `Tuplet` entry is encountered, parse its child `Entries` inside the local frame defined by:
   - `local_length_ticks`
   - `base_note_value`
   - `groupings`
3. Produce normal local notations from that child event stream.
4. Flatten the result back into host ordering with tuplet context and projected timing.

This avoids a special-case “triplet parser” while still preserving the existing notation pipeline shape.

## Local Metric Frame

Tuplets should conceptually behave like small isolated bars for notation purposes, but not literally embed full flow-level time-signature semantics.

Recommended local-frame semantics for MVP:

- local length
- local grouping boundaries
- base written unit

This is enough to support:

- written-duration interpretation
- beaming decisions
- spacing layout
- future nested tuplets

## Rests

Rests remain derived from absence of tones inside the child frame, exactly as they do on host tracks.

That means no explicit rest entries are required in tuplet storage.

## Rendering Architecture

## Tuplet Rendering

Rendering should continue to be fully backend-owned.

Recommended render behavior:

- derive child notation and child local spacing normally
- map child local x positions into the consumed host span
- draw tuplet number and bracket from wrapper metadata
- preserve current note/chord rendering semantics inside the child frame

Tuplets are therefore not a separate visual layer. They are normal notation rendered inside a warped container.

## Spacing Strategy

The current host spacing model already behaves like a widest-needed-spacing system. Tuplets can fit that model.

Recommended spacing flow:

1. Build a local spacing grid for the child frame using child notation.
2. Compute the child’s total required width.
3. Compare that width to the ambient host width across the consumed span.
4. If child width exceeds host width, expand host spacing.
5. If host width is already wider, stretch the child layout to fit.

This matches the agreed requirement that a triplet may either:

- demand more width than the default host span
- or fit naturally inside already-wide ambient spacing

## Selection And Hit Testing

## Tone Selection

Tone identity remains `tone_id`.

Recommended tone selection payload shape for future tuplets:

- `track_id`
- `tuplet_path`
- `tone_id`
- optional `local_tick` for lookup and anchoring

Where:

- `tuplet_path` expresses nested containment when needed
- `local_tick` is not authoritative identity

## Tuplet Selection

Tuplets themselves should be selectable by `tuplet_id`.

Recommended container selection payload:

- `track_id`
- `tuplet_path`
- `tuplet_id`

This supports native hit-testing for brackets and numbers without replacing tone-id-based authority.

## Playback Architecture

## MVP Playback Direction

Playback should continue to think in host musical time, but tuplet child note events must be projected into host time before scheduling.

Recommended MVP playback seam:

- collect child note events from tuplets together with host note events outside tuplets
- project their local onsets and durations into host time using the tuplet warp
- order them alongside host events by projected host position

This keeps tempo handling naturally host-level.

## Tempo And Future Ramps

Tempo remains pinned to host timeline positions.

This is the correct separation of concerns:

- tuplets map local notation time into host musical time
- tempo maps host musical time into wall-clock time

That architecture is compatible with future first-class ritardando and accelerando work because tempo curves remain a host-time concern rather than something owned by tuplet-local notation frames.

## Host-Level Directives Beyond MVP

The agreed direction is:

- do not solve nested attachment of host-level directives in MVP
- do not treat that as a model failure
- revisit it later as a refinement problem

Important future example:

- clef changes remain host-level in effect because they apply across instrument tracks and voices
- later, a clef may need to attach at an inner tuplet beat even though it remains host-level in scope

This future problem should be deferred. The current tuplet container model does not block it.

## Migration Impact Analysis

Migration is still required, but the scope is narrower than a repo-wide exact-duration rewrite.

Required persisted changes for MVP:

- add `Entry::Tuplet`
- add `Tuplet` serialization
- add child `Entries` serialization for notes authored wholly inside tuplets
- update load/save behavior to support tuplet metadata plus continuous cross-boundary tones

The intended persisted invariant is:

- child `entries` are for tones whose start position lies inside the tuplet
- host track entries are for tones whose start position lies outside the tuplet
- every tone remains one stored event
- load/save must not force a crossing tone into multiple stored tones merely because a tuplet boundary intersects it

Not required for MVP:

- rewriting every persisted host tone duration into a new global rational type
- redesigning host-level directive locations

Recommended `src-tauri/src/migrations/next.rs` work during implementation:

- additive schema support for tuplet entries
- no-op migration for legacy files that do not contain tuplets
- validation that recursive child `Entries` deserialize safely
- validation that cross-boundary continuous tones remain intact through load/save

## Regression Strategy

Regression coverage must protect both new tuplet behavior and existing non-tuplet behavior.

## Parser Regressions

Add explicit parser scenarios for:

- isolated eighth-note triplets
- isolated quarter-note triplets
- host-note consumption into triplet child space
- boundary splitting where a host note crosses into a tuplet
- nested tuplets in parser-only coverage if the data model permits them
- unchanged current simple-meter and compound-meter behavior outside tuplets

## Spacing Regressions

Add spacing tests that prove:

- tuplet-local width can expand host spacing when required
- tuplet-local layout can stretch to ambient host width when host spacing is already larger

## Rendering Regressions

Add renderer and hit-test coverage for:

- tuplet bracket and number draw instructions
- note positions projected correctly within the tuplet-governed host span
- `Tuplet` hit targets
- tone hit targets still authoritative by `tone_id`

## Playback Regressions

Add transport coverage for:

- triplet onsets projected exactly into host musical time
- triplet durations scheduled correctly under host tempos
- no drift relative to surrounding host notes

## Persistence Regressions

Add load/save coverage for:

- tuplet entries round-tripping through `.soloc`
- tuplet span metadata and child entries deserializing correctly
- cross-boundary continuous tones round-tripping without forced storage splits
- legacy non-tuplet files loading unchanged

## MVP Boundaries

In scope for MVP:

- track-local tuplets on instrument tracks
- tuplet-container creation behavior
- self-contained triplets
- nested tuplet-capable data model
- tone-owned properties inside tuplets
- rendering, spacing, hit-testing, and playback support for tuplet-governed notation and timing

Out of scope for MVP:

- host-level directives at inner tuplet positions
- cross-barline tuplets as a fully supported notation behavior
- delete-and-dissolve behavior
- generalized inner-span directive ecosystem inside tuplets

## Final Recommendation

Proceed with foundational tuplet support using a tuplet-container entry model.

The authoritative architectural statement is:

- tuplets are host-track entries that govern host time through a bounded local metric container and may own child tone entries
- authoritative tone events are always stored as single entries, with ownership determined by where they start
- boundary splitting changes derived ownership at the tuplet edge, not the written meaning of equal-duration segments on either side of that edge
- the wrapper retains musical context through governed span, local span, base notation, and grouping metadata
- parser, renderer, spacing, selection, and playback all operate by deriving tuplet-local fragments from persisted tones and child content, then projecting them back into host musical time

The simplest statement of the rule is:

- the track stores tones
- tuplets store container context plus child content for tones that start inside them
- drawing and scheduling are where crossing tones get split into boundary-aware fragments

This matches the agreed UX direction, preserves Compose’s event-stream model, and avoids a technical dead end without over-solving future directive-location work today.
