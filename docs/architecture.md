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
│  │  │ 9 HTML Windows   │  │    │  │ AppContext (Mutex)       │ │  │
│  │  │  - main          │  │    │  │  ├── Ui                  │ │  │
│  │  │  - debugger      │◄─┼─emit──┤  ├── Score               │ │  │
│  │  │  - inst-picker   │  │    │  │  ├── Preferences         │ │  │
│  │  │  - layout-settings│ │    │  │  └── next_id             │ │  │
│  │  │  - player-settings│ │    │  └─────────────────────────┘ │  │
│  │  │  - plugin-host-poc│ │    │                               │  │
│  │  │  - plugin-manager │ │    │                               │  │
│  │  │  - preferences   │  │    │  ┌─────────────────────────┐ │  │
│  │  │  - project-settings│ │   │  │ RendererContext (Mutex)  │ │  │
│  │  └──────────────────┘  │    │  │  ├── MetalRenderer       │ │  │
│  │           │             │    │  │  ├── DrawInstructions    │ │  │
│  │       invoke()          │    │  │  ├── HitTargets          │ │  │
│  │           │             │    │  │  └── Converter           │ │  │
│  │           ▼             │    │  └─────────────────────────┘ │  │
│  │  ┌──────────────────┐  │    │                               │  │
│  │  │ actions.ts       │──┼────┤  ┌─────────────────────────┐ │  │
│  │  │ (all mutations)  │  │    │  │ Audio / Transport        │ │  │
│  │  └──────────────────┘  │    │  │  ├── AudioContext        │ │  │
│  └────────────────────────┘    │  │  ├── TransportContext     │ │  │
│                                │  │  └── TransportShared (Arc)│ │  │
│                                │  └─────────────────────────┘ │  │
│                                └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## State Management

### Rust Side (Source of Truth)

- **`AppContext`** — Behind `Mutex`, holds `Ui`, `Score`, non-persisted same-session playback activation runtime, `Preferences`, `next_id`, `current_file_path`
- **`RendererContext`** — Behind `Mutex`, holds `MetalRenderer`, `DrawInstruction[]`, `HitTarget[]`, `Converter`
- **`AudioContext`** — Behind `Mutex`, holds `cpal` host/device/stream, `MixerHandle`
- **`PluginInventoryContext`** — Behind `Mutex`, holds the machine-local VST3 discovery inventory persisted under the app config directory, reuses that cached inventory during startup, revalidates cached entries, does one bounded root sweep for uncached bundle paths, probes only newly seen bundles, and reserves full root-wide reprobe for the explicit `plugin_inventory_discover` maintenance workflow before emitting refreshed backend-owned inventory truth to the frontend
- **`TransportContext`** — Behind `Mutex` + `TransportShared` (Arc with atomics for lock-free polling)

Score-instrument playback topology is now split into three backend-owned layers:

- persisted `Score.playback.rack` plus per-instrument `Instrument.playback_route` on the score domain store portable rack and route intent only
- non-persisted slot-owned live plugin sessions on the backend playback domain are keyed by `slot_id`, hold the real initialized VST runtime objects for prepared slots, restore any portable `PlaybackSlotTarget.plugin_state` payload before editor-open, are rebuilt from portable saved intent plus machine-local inventory truth, and are torn down at slot unassign, slot removal, inventory invalidation, reset, or project replacement instead of editor-window close
- non-persisted continuous render state in `src-tauri/src/playback/engine.rs` is the realtime playback-plane seam: it tracks which live slot sessions are render-active, owns backend event queues for immediate audition and later transport state handoff, and keeps empty blocks flowing through already-mounted sessions so editor-playable plugins can sound without turning the editor seam into the playback owner
- non-persisted `PlaybackActivationRuntime` on `AppContext` is a readiness snapshot derived from those live slot sessions; it is rebuilt after slot assignment, slot unassignment, slot removal, route assignment, route clearing, project load, inventory refresh, and reset without being written into `.soloc`
- non-persisted playback preparation in `src-tauri/src/playback/preparation.rs` is the canonical transport and audition handoff: it combines the current `PlaybackActivationRuntime`, the explicit resolved route states, and the active audio sample rate into one backend-owned preparation artifact, snapshots the supported direct-plugin root targets once per play-start, and remains reusable without creating frontend-owned or probe-owned playback truth. For the current audition story, that handoff still drives a bounded pitch-preview contract: one score-derived pitch, 500ms duration, moderate velocity, default legato articulation, and score-context triggers only.
- emitted `resolved_playback_route_by_instrument_id` on `soloc://state/general` stores the backend-computed score-context status (`Unrouted`, `Ready`, `NeedsAttention`) plus the currently activated effective plugin id, resolved slot target, and explicit route metadata needed by later stories
- emitted `route_target_options_by_slot_id` on `soloc://state/general` tells Sequence which explicit route targets are currently valid for each prepared slot so React does not invent plugin-part or MIDI-channel choices locally

That split is deliberate. `.soloc` files must stay portable and must not persist machine-local inventory truth, while React score surfaces must still receive one authoritative backend answer instead of inventing local route-health rules. The rack owns slot lifecycle, slots may exist without an assigned plugin, and instruments only own routes to already-existing slots. When a plugin exposes transferable component state, Compose persists only that minimal portable restore blob alongside the slot target so the runtime session can be rebuilt to the intended instrument or patch selection without serializing live host objects. Story 3.8 narrows same-session runtime preparation to currently supported single-target routes only: a routed instrument is `Ready` only when its referenced slot target has an active backend-owned runtime destination in the current app session. Routed-to-empty-slot, missing-slot, unavailable-plugin, and deferred multi-target route cases remain explicit backend-owned `NeedsAttention` or `Unrouted` states. Reopen restoration across app restarts remains Epic 5 scope rather than part of this activation seam.

The Epic 1 ownership correction is now part of the active architecture: `PlaybackActivationRuntime` is readiness and targeting metadata, not the long-lived live plugin-session registry. Each prepared slot owns one runtime-only live plugin session keyed by slot identity and rebuilt from portable saved intent rather than persisted live runtime state. Native editor windows attach to or detach from that existing slot-owned session instead of creating the canonical instance on open.

### Frontend Side (Display Layer)

Four React contexts mirror the backend state:

| Context                  | Event Channel                         | Hook                        | Purpose                                                          |
| ------------------------ | ------------------------------------- | --------------------------- | ---------------------------------------------------------------- |
| `AppContext`             | `soloc://state/general`               | `useAppState()`             | UI state, score data, resolved playback route state, preferences |
| `AudioContext`           | `soloc://state/audio`                 | `useAudioState()`           | Device list, active device                                       |
| `PluginInventoryContext` | `soloc://state/plugin_inventory`      | `usePluginInventoryState()` | Machine-local VST3 inventory                                     |
| `TransportContext`       | RAF polling via `transport_get_state` | `useTransportState()`       | Playing, tick, tempo                                             |

The dedicated plugin inventory state is now the backend-owned machine-local VST3 discovery seam. Startup reuses the persisted inventory immediately, revalidates cached entries, performs one cheap directory sweep to spot uncached bundles, probes only those newly seen bundles, and still leaves full root-wide reprobe behavior to the explicit manual refresh workflow. Inventory refresh and ordinary score playback mutations rebuild the same-session runtime activation state first, and both general readiness plus the audible transport snapshot consume that VST-owned backend answer instead of carrying a second availability resolver. The canonical app-scoped browse surface for that inventory is now the dedicated Plugin Manager window opened from the Sequence menu, where the shared `Refresh plugin inventory` control surfaces refresh-in-progress and refreshed-state feedback. Epic 2 through Epic 6 must continue building VST-owned assignment, runtime, and transport-adjacent contracts rather than reintroducing frontend-owned playback truth or a second legacy runtime path.
The Sequence bottom panel now owns the current score rack display and score-context slot preparation. It shows backend-owned score-context slots as horizontally stacked rack cards, creates empty slots from an in-panel add control, and assigns or updates each slot's plugin through a slot-local picker backed by the Rust-owned usable inventory seam. Instrument overview rows expose the bounded route-authoring workflow directly as a slot picker, while the rack cards stay focused on slot preparation. When a route still points at an existing empty slot, the row keeps that slot identity visible as a numbered empty label such as `1. --` instead of collapsing back to the unrouted `--` state, and only unusable assigned-plugin routes surface the needs-attention badge. Plugin Manager remains the app-scoped inventory browser rather than the default answer for routine score-route repair or ordinary slot preparation.
Those compact score-context playback surfaces must also remain legible without color alone: icon-only tabs, transport buttons, and bounded mix toggles should carry explicit names/tooltips and semantic selected-state cues instead of depending on accent color as the primary signal.

### State Flow

1. User interacts with React UI
2. `actions.ts` calls `invoke("command_name", { params })` — all params use `snake_case`
3. Rust command locks `AppContext`, mutates state
4. Rust calls `state.emit()` → serializes state, including backend-resolved playback route status → `app.emit("soloc://state/general")` to ALL windows
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

### Tuplet notation contract

- Shipped MVP tuplets are created through the existing popover flow, then edited through the sequencer's existing tone and container seams. The durable contract is popover-first container creation plus backend-owned sequencer editing, not a separate freeform tuplet construction model.
- Supported shipped ratios are currently triplet-first (`3q/2`, `3e/2`, `3s/2`, `3t/2`). Broader arbitrary ratios remain deferred and should not be implied by architecture prose.
- Persisted `Tuplet` entries are not themselves printed-score notation output. The parse phase derives notation from the authoritative host-track container and its child `entries`.
- The local derived unit should remain a normal `NotationTrack`, not a second tuplet-only notation structure. This preserves existing parser helpers and keeps downstream consumers working against the same base notation type.
- The only extra contract around that local `NotationTrack` is a thin anchor carrying the host-span projection metadata needed to place the local unit back into the surrounding flow.
- Conceptually, the parser output for a bounded notation region follows this shape:

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

- `owner_id` is intentionally generic in the architecture contract. For tuplets it is the tuplet container id, but the same anchored-region pattern can be reused later for other bounded notation units with minimal structural change.
- The contract must preserve three separate meanings: local semantic address, written duration used for notation decisions inside the local frame, and projected host occupancy used by spacing and draw consumers.
- The parse architecture is therefore: treat the governed host span as an implicit placeholder, derive the tuplet as an isolated local `NotationTrack`, then project that local unit back into host ordering through explicit host-span metadata rather than through host-only `tick + duration` arithmetic.
- Boundary-crossing tones remain one stored tone. Derivation may produce multiple local notations or host-side companions at the boundary, but continuity still follows stored tone identity.
- A completely empty triplet is a special readability case and should derive one base-note rest per local slot. Outside that case, tuplets reuse the existing local split-unit logic for note and rest spelling.
- Later renderer and spacing work should consume this anchored local `NotationTrack` directly. They should not reconstruct tuplet meaning from flattened host ticks, and they should not introduce a parallel frontend-owned notation model.
- Sequencer interactions remain tone-focused in MVP: empty tuplet space is derived rather than persisted, blank tuplet space is non-selectable there, and delete/slice/selection continue to flow through the existing shared tone and container seams.
- Nested tuplets, generalized sequencer-native tuplet tooling beyond the shipped seams, and any cross-flow expansion of tuplet playback remain explicit post-MVP work.

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

Supported `transport_play` is now score-driven and audible on the product path for the bounded MVP route shape. It advances tick and tempo state from a play-start score snapshot, reuses the backend-owned playback preparation artifact derived from `PlaybackActivationRuntime` and the current audio sample rate, and submits projected score note events into the continuous render engine for the snapshot's supported direct-plugin root targets without any sampler or SCVI fallback path. Transport start is gated only by valid flow and tick state; unsupported or non-ready routes fall through as scheduling no-ops inside that prebuilt snapshot rather than blocking the transport clock itself.

Story 5.1 tightens the reset contract on that same path. `transport_pause`, stop/reset flows, and natural transport teardown now hard-reset engine-owned transport work: queued transport events are cleared, transport-active slot state is disarmed, active transport notes are retired deterministically, and resume always restarts from the paused tick through a fresh `prepare_supported_playback(...)` snapshot instead of reusing stale render-target or queue state. Audio device and sample-rate rebuilds also reuse that transport-owned reset path before the next play-start so stale timing assumptions cannot survive a stream rebuild, while score-context `volume`, `pan`, `mute`, and `solo` mutations continue to flow through the backend-owned effective-mix seam into the live engine state during playback.

Story 4.3 makes the realtime playback plane concrete without widening transport scope. The persistent output stream in `src-tauri/src/playback/audio.rs` now calls into a backend-owned continuous render engine before draining any legacy mixer voices. That engine owns render-active slot state, backend event queues, and per-block processing for slot-owned `PluginHostSession`s, while transport owns only score-derived scheduling/state transitions and feeds note events into that engine instead of taking over stream lifetime or plugin-session ownership. The native editor remains a transient attachment over the already-running slot session.

Story 1.3 added an explicit single-instrument reset trust gate on top of that path, and Stories 1.4 through 1.6 plus maintainer manual proof completed it. Here, single-instrument means a VST instance that hosts one playable instrument at a time, even if that instance exposes multiple articulations, expressions, or mic mixes; HALion-style multi-slot or multi-part instances remain out of scope. The backend still exposes that trust gate as explicit readiness truth through `PlaybackActivationRuntime`, but transport no longer treats it as a hard start gate: it snapshots every currently ready direct-plugin root route on play-start and sounds those routes together by feeding projected score events into the continuous engine. Later Epic 5 trust work and Epic 6 expression maps extend this transport-owned audible path rather than replacing it.

```
┌──────────────┐
│  Transport   │
│  Thread      │
│              │
│  Walks score │
│  tick by tick│
│              │
└──────────────┘
```

### Key Components

- **Transport** — Uses atomics (`AtomicBool`, `AtomicU64`) for lock-free state sharing; instrument mix params via `RwLock<FxHashMap>`
- **Tempo resolution** — Playback snapshots the score's flow master-track tempo data at play/resume start, resolves the active BPM once per tick via a bounded resolver, and silently falls back to 120 BPM when no score tempo applies
- **Playback preparation** — `prepare_supported_playback(...)` is the canonical backend seam for transport and audition work; it preserves explicit `Unrouted` and `NeedsAttention` states, captures the active audio sample rate, exposes route-backed `supported_mix_controls`, and snapshots the currently supported direct-plugin root targets once per play-start so transport can fan score events out without rechecking readiness inside the render loop
- **Continuous render engine** — `src-tauri/src/playback/engine.rs` keeps render-active slot sessions alive on the audio callback with backend-owned event queues and silent-safe blocks, applies the prepared effective route mix for the active slot, and lets audition, mounted plugin UI interaction, and transport-driven score playback share one playback-plane seam
- **Audition contract** — the first-audible audition workflow is intentionally narrow: it previews one score-derived pitch for 500ms at moderate velocity using a default legato articulation, injects note events into the continuous engine, and is triggered only from score-context tone selection, pitch edits, or accidental edits rather than from transport-like phrase playback
- **Current MVP score-context mix controls** — `volume`, `mute`, `solo`, and `pan` are exposed as honest controls when the active resolved route is a ready direct-plugin root path; pan currently behaves as a backend-owned stereo balance control on the rendered slot output and solo suppresses other ready supported routes through the same effective-mix seam

### Transport tempo semantics

- Playback start resolves tempo from the latest immediate absolute tempo at-or-before the current tick before the transport thread starts, so mid-flow starts report the correct inherited BPM immediately.
- The transport thread re-resolves tempo once per playback tick from its play-start score snapshot, applies tempo changes exactly on the matching tick boundary, and accumulates tick deadlines from prior timing state to reduce drift.
- `transport_get_state()` reports truthful stopped-state tempo for the current tick as well, so stopped seek and stop-to-tick-0 transitions show the effective score tempo instead of stale playback tempo.
- Pause/resume behaves like a fresh playback start from the current tick: pause clears the transport render target, flushes active audio, joins any prior transport thread, republishes truthful tempo for the paused tick, and resume re-snapshots score tempo data so edits made while paused are reflected.
- Seek uses that same transport-owned reset model instead of an in-thread cursor jump: active seek clamps through `ui.flow_id`, cancels queued transport events and active transport-owned notes from the old timeline, then restarts from the target tick on a fresh play-start snapshot, while stopped seek only republishes truthful tick and tempo.
- Blocked transport starts remain explicit backend no-ops: if the shared preparation seam cannot produce a supported target, transport stays stopped, clears stale stop-signal state, and does not invent fallback playback or mutate resolved route truth.
- Live edits during active playback still do not affect the in-flight run; a fresh play/resume cycle is required because transport playback uses a play-start score snapshot.
- Tuplet playback consumes the same shared tuplet model used by persistence and notation, and shipped MVP playback remains scoped to the active flow selected through `ui.flow_id`; tuplets do not widen playback to inactive flows.
- Tempo property editing also changes persisted tempo load/save semantics: `Tempo.text` is now stored as a required string, and migration `1.25.0` normalizes older `text: null` payloads to `""` during project load.

---

## Multi-Window Architecture

Each of the 9 windows has:

- Its own HTML entry point (e.g., `instrument-picker.html`)
- Its own `boot.tsx` mounting `AppStateProvider`
- Independent React tree receiving state via `app.emit()` to all windows

Secondary windows (instrument-picker, preferences, etc.) close on blur via `tauri://blur` event listener. New windows are opened via `WebviewWindow` from `@tauri-apps/api/webviewWindow`.

The backend-owned `plugin-host-poc` window is the one exception to the normal frontend-owned open flow: React triggers a thin backend command from the Sequence score-context rack action, and Rust creates or focuses the host window so the backend can own macOS-native attachment and teardown. Plugin Manager no longer carries editor-launch responsibility; instead, the target resolves in Rust from the backend-owned machine-local inventory first, then from an explicit request, then `SOLOC_PLUGIN_EDITOR_POC_VST3_PATH`, then the standard macOS VST3 install roots under `~/Library/Audio/Plug-Ins/VST3` and `/Library/Audio/Plug-Ins/VST3`. Story 1.3 hardens the score-context rack path further: rack-originated editor opens resolve through the score-owned slot id as well as the bundle path, so multiple slots using the same plugin bundle remain distinct native-host targets instead of collapsing into one bundle-level reservation. Rust now keeps one authoritative host lifecycle reservation for one active plugin target in the app session, promotes that reservation to an attached session only after the native host session is stored, marks close as an explicit teardown phase, and clears stale reservations when no host window remains. It validates that the chosen target is a local `.vst3` bundle, loads the bundle binary in Rust, creates a minimal VST3 component/controller/editor stack, synchronizes component state into the controller when needed, connects component/controller `IConnectionPoint` pairs when the plugin exposes them, and attaches the returned `IPlugView` to a native AppKit subview in the Tauri content hierarchy. Resize remains capability-driven: the host window stays bounded unless the plugin editor reports `canResize()`, and focus/resize sync stays backend-owned on the main thread. This is now the supported bounded native-editor handoff for plugin-specific depth inside Compose, but it is still not the canonical inventory surface and not the template for long-term frontend-owned VST workflows.

---

## File Format & Persistence

- **Project files:** `.soloc` — custom binary container with `SOLOC` magic, container version, authoritative header `app_version`, a chunk directory, and a required gzip-compressed `PROJECT_JSON` chunk containing `{ ui, score, next_id }`
- **Recent files:** Stored in `sled` embedded database at app config directory
- **Migration system:** `migrations/mod.rs` holds version-gated JSON migration functions; the loader uses the container header `app_version` as the authoritative migration key, validates it against `score.meta.app_version`, applies migrations to `serde_json::Value`, then performs a single typed deserialize before mutating app state
- **Tuplet persistence contract:** Tuplet-bearing scores use that same single `.soloc` transport and typed deserialize path. Load does not create a parallel tuplet repair layer, split crossing tones into extra stored events, or normalize ownership away from the rule that tones live where they start.
- **Desktop file-workflow contract:** Tuplet-bearing projects keep the same offline-first local open, save, reopen, current-file-path, recent-file, and transport-resync path as non-tuplet projects through the existing Tauri file commands; no network-backed persistence or alternate frontend-owned file state is introduced.
- **Current migration stance for tuplets:** No tuplet-specific step is present in `src-tauri/src/migrations/next.rs` because pre-tuplet releases produced no tuplet-bearing payloads. Legacy non-tuplet files still validate and migrate exactly as before; add a tuplet migration only if a released older persisted format requires a real compatibility transform.
- **Machine-local VST3 inventory:** Stored outside `.soloc` files in a `sled` database under the app config directory; this backend-owned inventory is machine-local discovery state, not portable project data. Startup reuses that cache, degrades stale cached entries in place, and can append newly seen bundles from one bounded root sweep, while explicit manual refresh remains the seam that may perform broader discovery work.
- **Experimental editor-host target:** The native plugin-host POC does not reuse the SCVI instrument catalog. It resolves a local VST3 bundle from the backend-owned machine-local VST3 inventory when a plugin-backed request provides a plugin id, otherwise from an explicit request, `SOLOC_PLUGIN_EDITOR_POC_VST3_PATH`, or the standard macOS VST3 install roots. Compose now maintains that inventory as the authoritative discovery seam, while the editor-host POC keeps a separate backend-owned single-session lifecycle reservation so close, destroy, and reopen stay local to that bounded native host seam rather than widening into transport or assignment state.

---

## Managed State Summary (Tauri)

| State                         | Type                            | Purpose                      |
| ----------------------------- | ------------------------------- | ---------------------------- |
| `AppContextState`             | `Mutex<AppContext>`             | Core app state               |
| `RendererContextState`        | `Mutex<RendererContext>`        | Score rendering              |
| `AudioContextState`           | `Mutex<AudioContext>`           | Audio device I/O             |
| `PluginInventoryContextState` | `Mutex<PluginInventoryContext>` | Machine-local VST3 inventory |
| `TransportContextState`       | `Mutex<TransportContext>`       | Playback control             |
| `TransportSharedState`        | `Arc<TransportShared>`          | Lock-free transport polling  |
