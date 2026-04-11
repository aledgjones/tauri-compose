# Solo Compose — Development Guide

## Prerequisites

- **Node.js** v18+ — [Download](https://nodejs.org/)
- **Rust** (latest stable) — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Xcode Command Line Tools** (macOS) — `xcode-select --install`

## Installation

```bash
git clone https://github.com/aledjones/compose.git
cd compose
npm install
```

## BMAD Python Bootstrap

macOS Homebrew Python is PEP 668 managed, so `python3 -m pip install --user ...` can fail for BMAD helper dependencies such as `PyYAML`.

Use the repo-local wrapper instead of invoking `bmad_init.py` with the system interpreter:

```bash
./scripts/bmad-init load --project-root "$PWD" --all
```

Or through npm:

```bash
npm run bmad:init -- load --project-root "$PWD" --all
```

The wrapper creates `.venv-bmad/` on first run, installs the pinned BMAD Python requirements there, and then executes the BMAD init script through that interpreter. This avoids modifying the system Python and removes the recurring `ModuleNotFoundError: No module named 'yaml'` failure mode.

## Development Commands

| Command                 | Description                                                                       |
| ----------------------- | --------------------------------------------------------------------------------- |
| `npm run start:dev`     | Start Tauri in development mode (hot reload for frontend, Rust recompile on save) |
| `npm run start:release` | Build production release (.dmg)                                                   |
| `npm run dev`           | Vite dev server only (no Tauri — useful for frontend-only work)                   |
| `npm run build`         | TypeScript check + Vite production build                                          |
| `npm run check-types`   | TypeScript type checking (`tsc --noEmit`)                                         |
| `npm run lint`          | ESLint check on `src/**/*.{ts,tsx}`                                               |
| `npm run lint:fix`      | ESLint auto-fix                                                                   |
| `npm run format`        | Prettier format check                                                             |
| `npm run format:fix`    | Prettier auto-format                                                              |
| `npm run preview`       | Preview production build locally                                                  |

## MCP Actions POC

The app exposes MCP directly over streamable HTTP on localhost.

Example VS Code MCP config:

```json
{
  "servers": {
    "soloc": {
      "type": "http",
      "url": "http://127.0.0.1:4767/mcp"
    }
  }
}
```

1. Add the `soloc` server to your VS Code MCP config
2. Start the app with `npm run start:dev`

The app hosts a localhost-only MCP endpoint at `http://127.0.0.1:4767/mcp` and also exposes a local automation API at `http://127.0.0.1:4767/api/v1/automation`.
The MCP endpoint exposes:

- `compose_get_started` as the first-call onboarding guide

In addition, the app now generates one MCP tool per automation action using the `compose_action_` prefix. For example, `score.flow.create` is exposed as `compose_action_score_flow_create`, and tuplet container creation is exposed as `compose_action_score_tuplet_create`.

`compose_get_started` should be the first tool call for score-construction work from MCP-only clients. It returns the recommended call order plus the onboarding guidance needed before listing or invoking actions, including a nested `domainModel` section that covers ids, wiring, ticks, and score graph relationships.

Generated `compose_action_*` tools are the action surface. They are discoverable through standard MCP `tools/list`, publish named-parameter schemas, and dispatch through the existing automation registry.

For pitch entry, `pitch_int` uses MIDI note numbers. The main reference points are `C0 = 12` and `C4 = 60`. The `accidental` field controls or overrides enharmonic spelling, and when `accidental` is `null` the backend can infer spelling from the active key signature.

For note entry, keep the score wiring order explicit:

1. Create flows, instruments, and players.
2. Assign instruments to players and players to flows/layouts.
3. Call `compose_action_score_lookup_getInstrumentTracks` to resolve the playable `track_id` values for the target flow and instrument.
4. Write tones only to user track ids returned by that lookup. Do not write tones to flow master tracks or stave master tracks.

If the lookup returns multiple staves for one instrument, write notes to the separate staff track ids it returns. Piano is the common case: right-hand material belongs on the upper-staff track id and left-hand material belongs on the lower-staff track id.

Ordinary empty spans do not require explicit rest objects from MCP clients. In normal score construction, gaps are persisted as absence of tones and later materialize as derived rests in notation consumers.

Tuplet authoring through MCP follows the same rule. Create the tuplet container on the owning user track, then create child tones on that same track with `tuplet_id` set to the returned container id.

The important timing rule is that tuplet containers and tuplet child tones use different coordinate frames:

- the tuplet container `tick` is a host-track tick
- the tuplet container occupies the compressed host span implied by `note_count`, `in_space_of`, and `base_note_value`
- child tones inside the tuplet use tuplet-local ticks, not absolute host ticks
- child tone durations remain literal in the tuplet-local frame; they are not rewritten to the compressed host span

For example, a 3:2 eighth-note triplet in 4/4 starting on beat 2 uses quarter = 32 ticks and eighth = 16 ticks:

- host start tick: `32`
- host span: `32` ticks total, because 3 eighths fit in the space of 2 eighths
- local tuplet length: `48` ticks total, because the child timeline still contains 3 literal eighth notes
- child note ticks: `0`, `16`, `32`
- child note durations: `16`, `16`, `16`

In MCP terms, that sequence looks like this:

1. Create the container with `compose_action_score_tuplet_create(track_id, 32, 3, 2, 8, [3])`.
2. Create the first tuplet tone with `compose_action_score_tone_create(track_id, 0, 16, pitch, accidental, velocity, articulations, tuplet_id)`.
3. Create the second and third tones at local ticks `16` and `32` with the same literal eighth duration `16`.

This means MCP-only clients should not pre-compress child ticks to the host span. For the example above, child ticks `0, 16, 32` are correct; host ticks `32, 42.66..., 53.33...` are not.

Ownership also stays explicit when tuplets overlap existing material:

- tones that start inside the tuplet belong in the tuplet child `entries`
- tones that start before the tuplet remain host-owned even if their durations cross the tuplet span
- host-owned crossing tones keep their literal durations; only the container geometry is compressed on the host track

The same start-position ownership rule applies outside tuplets as well. When a tone crosses a structural boundary, its owner is determined by where it starts, not by the furthest point its duration reaches.

Two minimal MCP-only authoring examples are worth keeping in mind:

1. Single-staff note entry
   - create and assign the score entities
   - resolve the playable `track_id`
   - create a quarter-note `C4` at tick `0` with `compose_action_score_tone_create(track_id, 0, 32, 60, null, velocity, articulations)`

2. Piano note entry
   - resolve the piano lookup
   - use the upper-staff `track_id` for right hand and the lower-staff `track_id` for left hand
   - do not merge both hands onto one note track just because piano is one instrument

When in doubt, use this pattern for MCP-only score construction: `compose_get_started` -> create and assign score entities -> `compose_action_score_lookup_getInstrumentTracks` -> `compose_action_score_tuplet_create` -> `compose_action_score_tone_create(..., tuplet_id)`.

This is intentionally a developer-facing automation surface. It is localhost-only and app-hosted.

For release packaging, no extra MCP sidecar binary is required because the app itself hosts the MCP endpoint.

## Validation Before Commit

```bash
npm run lint && npm run check-types && npm test && cd src-tauri && cargo clippy
```

All four must pass. Clippy warnings are treated as errors — do not suppress with `#[allow(...)]`.

## Bounded Edit-During-Playback Contract

The transport path is intentionally snapshot-based during active playback.

- Immediate in the current audible pass is limited to `volume`, `pan`, `mute`, and `solo` on already-ready direct-plugin root routes.
- Score-content edits and playback-topology edits persist immediately in canonical score state, but their audible effect is deferred until the next safe backend boundary.
- Supported refresh boundaries are pause, seek restart, stopped play-start, natural-end replay, and explicit backend transport resets.
- Do not add mid-pass queue rewrites, frontend-owned pending-edit inference, or ad hoc runtime refreshes from score mutation commands.

Pre-commit hooks (via `lint-staged` + `simple-git-hooks`) run ESLint and Prettier automatically on staged files. If hooks are not running after a fresh clone, register them with `npx simple-git-hooks`.

## macOS Native Editor Handoff

The bounded native editor handoff is macOS-only. Keep it distinct from the dedicated Plugin Manager window so the canonical inventory browser stays focused on user-facing discovery, health, and refresh workflows.

The POC resolves its target in this order:

- inventory-backed `plugin_id` resolved in Rust from the machine-local VST3 inventory
- explicit `plugin_path` passed to the backend command
- `SOLOC_PLUGIN_EDITOR_POC_VST3_PATH`
- first valid `.vst3` bundle found under `~/Library/Audio/Plug-Ins/VST3`
- first valid `.vst3` bundle found under `/Library/Audio/Plug-Ins/VST3`

Set the env var if you want to override the default macOS discovery path:

```bash
export SOLOC_PLUGIN_EDITOR_POC_VST3_PATH="/Library/Audio/Plug-Ins/VST3/My Test Plugin.vst3"
npm run start:dev
```

What the current bounded handoff does:

- validates the configured path before creating the host window
- prefers Rust-owned plugin inventory targeting when the request provides a plugin-backed `plugin_id`
- creates or focuses a dedicated backend-owned Tauri window labeled `plugin-host-poc`
- keeps one authoritative backend-owned host lifecycle reservation for one active plugin target inside the app session, self-heals stale reservations when the window is gone, and treats close/destroy as explicit teardown phases rather than frontend-owned state
- loads the VST3 bundle binary from the macOS bundle, creates a component/controller pair, synchronizes component state into the controller when the plugin exposes separate objects, connects `IConnectionPoint` when available, requests `ViewType::kEditor`, and attaches the returned `IPlugView` to the native AppKit content area
- performs the initial focus/size sync only after Rust stores the live host session, so reopen and same-session reattach follow the same backend-owned session path instead of depending on a one-shot attach side effect
- defaults the host window to non-resizable until the plugin explicitly reports that its editor can resize via `IPlugView::canResize()`, while still forwarding bounded geometry sync callbacks needed to keep the attached editor positioned correctly inside the host surface
- keeps the bounded host state in Rust and logs create, focus, resize, close, destroy, and reopen lifecycle events

The supported user-facing entry point for this bounded handoff is the Sequence score-context rack action on a prepared slot. Keep that handoff distinct from the canonical Sequence-menu `Plugin Manager...` inventory window, and do not treat the native editor seam as the default answer to assignment, playback, or routine plugin troubleshooting workflows.

### Maintainer load/init probe

Story 9.1 adds a maintainer-only diagnostic command that reuses the same backend-owned host build and editor-attach order, but bypasses the ordinary blocked-target gate for investigation only.

The preferred workflow is now the Debugger window panel, which can target either a discovered inventory plugin or an explicit bundle path, prepares the persisted backend probe log before invoking the risky host probe, and renders both the returned validation payload and the last saved log snapshot.

If you need to script it directly, the same backend command is still available:

```ts
import { invoke } from "@tauri-apps/api/core";

const probe = await invoke("plugin_editor_poc_probe_load_init", {
  plugin_path: "/Library/Audio/Plug-Ins/VST3/Steinberg/HALion Sonic.vst3",
});
```

You can also pass an inventory-backed `plugin_id` instead of `plugin_path`. The command updates the machine-local inventory entry's `editor_host_validation` payload and returns the same validation object.

If you want a durable breadcrumb before the risky host call starts, `plugin_editor_poc_prepare_probe_log` resolves the target, resets the persisted log, and returns the exact on-disk log path plus current contents. The debugger now calls this preflight automatically before `plugin_editor_poc_probe_load_init`.

The backend also writes a persisted log before each risky maintainer probe phase and keeps appending phase transitions as the probe advances. Reopen the Debugger window after a crash to inspect that saved log and its exact file path, or fetch the same path-plus-contents snapshot programmatically with `plugin_editor_poc_read_probe_log`.

Both probe-log commands now also return an optional backend-derived `crash_hint` when the last persisted phase entry is still `status=starting`. The debugger uses that field to show a maintainer warning such as a likely hard crash during `controller_initialize` without re-deriving phase truth in React.

The maintainer probe now also supports multiple controller-init profiles through the Debugger window:

- `default`: preserves the current bounded host contract for comparison.
- `controller_init_host_only_lifecycle_aligned`: keeps the normal full host context for factory and component setup, but passes a pure `IHostApplication` context into `controller.initialize()`. For the empty Steinberg wrapper-marker probes it still reports `kNoInterface`, but now returns a balanced non-null `FUnknown` fallback object so the experiment can distinguish a null-release crash from a broader host-surface requirement.

Earlier maintainer-only controller-init profile permutations were retired after repeated HALion reruns left the failure boundary at `controller_initialize`. The remaining non-default profile keeps the debugger focused on the current blind spot: what HALion queries from a host-only init-time context.

Use the non-default profile only for investigation. It must not change ordinary blocked posture, ordinary editor-open behavior, or the product support claim until a later change deliberately promotes a proved-safe fix.

During the active maintainer probe, the backend now also logs host callbacks such as `IHostApplication::getName`, `IHostApplication::createInstance`, and direct `FUnknown::queryInterface` requests to the same persisted log and `MAINTAINER_PROBE` stderr stream. Recognized Steinberg interfaces are logged by name, and unmapped requests now fall back to raw IID hex such as `unknown_interface(70A4156F-6E6E4026-989148BF-AA60D8D1)`. That gives the next HALion run a way to show whether the controller is touching a standard host seam we missed or a private extension before the crash.

The first resolved HALion-only unknown IIDs were `IVst3ToVst2Wrapper`, `IVst3ToAAXWrapper`, and `IVst3ToAUWrapper`. Those look like Steinberg wrapper-detection probes rather than ordinary native-VST3 host services. The current host-only maintainer profile still reports `kNoInterface` for them, but it now hands back a balanced non-null `FUnknown` fallback object because HALion may be releasing the returned COM object without trusting the result code. That fallback is investigation-only and must not be treated as proof that Compose implements any real wrapper environment.

After HALion proved that it reaches `IHostApplication::getName` during `controller_initialize`, the next bounded maintainer profile removes even `IPlugInterfaceSupport` from the init-time context. The Rust `vst3` binding aliases `kResultTrue` to `kResultOk`, so changing only the `getName` success code would not materially change host behavior.

Those same maintainer probe events are now mirrored directly to the Rust process stderr with a `MAINTAINER_PROBE` prefix and an explicit flush on every event. When running `npm run start:dev`, watch the terminal for lines such as `[MAINTAINER_PROBE][timestamp][phase][status] ...` during the crashy path.

The maintainer-facing nested payload is `editor_host_validation.maintainer_load_init_probe`. It records:

- `status`: `blocked_by_host_contract`, `blocked_by_plugin_failure`, or `load_init_safe`
- `last_successful_phase`
- `first_failing_phase`
- `first_failing_message`
- ordered `phase_results`

Use the probe with these rules:

- If the result is not `load_init_safe`, stop there. Do not start capability probing, route-choice experiments, or shared-slot work.
- If the result is `load_init_safe`, ordinary product support posture is still unchanged until a later Epic 9 story proves plugin-derived capability and explicitly updates the support claim.
- Keep HALion-family bundles as the multi-instrument reference case. `BBC Symphony Orchestra.vst3` remains part of the existing single-target control matrix, not multi-instrument proof.
- Treat the stored result as machine-local maintainer evidence only. Do not write it into `.soloc` files or surface it as a user-facing support claim.

Plugin inventory startup behavior:

- Compose persists the machine-local VST3 inventory under the app config directory and reuses that cache on startup so known plugins are visible immediately.
- Startup validation is intentionally bounded: it rechecks cached bundle paths and related cached entry state, then does one cheap root listing sweep to spot uncached `.vst3` bundles and probes only those newly seen bundles.
- Startup cache reuse is not `.soloc` persistence. Project files remain portable score data only.
- The canonical app-scoped browse surface for that inventory is the Sequence-menu `Plugin Manager...` window.
- The Sequence bottom panel now owns the current score rack display and the ordinary score-context slot-preparation workflow, including an add-slot control, empty-slot creation, and slot-local plugin pickers backed by Rust-owned inventory truth. Ordinary choices stay bounded to plugins that are both available and not editor-host blocked.
- Sequence instrument overview rows are the bounded score-context route-authoring workflow: choose a slot from the current score rack, or switch back to the empty `--` state. If a selected score slot still exists but no longer has a plugin assigned, the picker keeps that slot visible as `N. --` so the user can see that a specific slot remains selected even though it is not currently playable, and that empty-slot selection does not surface a needs-attention badge.
- Keep backend playback truth and user-facing warning chrome separate. The backend should continue to distinguish `Unrouted`, empty-slot, missing-plugin, and other non-ready route states honestly, but the Sequence route control stays intentionally minimal: plain `--` for no assigned route, `N. --` for a selected empty slot, and the compact warning icon/title only for genuinely broken assigned plugin routes that the user should repair.
- The dedicated Plugin Manager exposes the canonical `Refresh plugin inventory` action for the explicit full reprobe workflow and shows refresh-in-progress plus refreshed-state feedback.
- The dedicated Plugin Manager remains the place for inventory browsing, health visibility, and refresh; the Sequence rack panel is not a substitute for those app-scoped responsibilities, but it is the correct in-score surface for creating an empty slot and assigning or updating that slot from currently usable plugins.
- Compact playback controls inside score context must stay legible without color alone. Keep icon-only tabs, transport buttons, and bounded mix toggles named through explicit labels or titles, and expose selected toggle state semantically rather than asking the user to infer it from accent color alone.
- Same-session playback activation is now backend-owned and single-target only. Refresh the runtime activation seam by mutating score rack or route intent through the existing Rust commands or by loading a project; do not add frontend-owned activation caches or one-shot setup flows.
- Inventory refresh also rebuilds same-session activation truth. If a previously activated slot target becomes unavailable, keep the route in the existing score-context rack workflow and let backend-emitted readiness fall to `NeedsAttention` instead of repairing it through a Plugin-Manager-first side path.
- Transport-facing and audition-facing playback preparation must go through `src-tauri/src/playback/preparation.rs` via `prepare_supported_playback(...)`. That seam consumes `AppContext.playback_activation_runtime` plus the active audio sample rate, preserves explicit resolved-route states, and snapshots the currently supported direct-plugin root targets once at play-start so transport never re-queries readiness inside the render loop. Treat that public multi-target snapshot as canonical; if a caller needs one instrument-specific view, derive it through `supported_target_for_instrument(...)` instead of reintroducing a public single-target compatibility field.
- Supported `transport_play` is no longer timing-only. It must remain the backend-owned audible score path for the current MVP: reuse the shared preparation seam, start transport whenever the active flow and tick are valid, schedule score-derived note events only for the snapshot's supported targets, and feed those events into `src-tauri/src/playback/engine.rs` instead of inventing an audition-only or frontend-owned playback path.
- Note-time score semantics belong in backend preparation or event-collection seams, not in `transport.rs`. When transport or later expression-map work needs playing-technique meaning, derive it from the owning stave master track before scheduling and carry it forward on the prepared note context instead of walking raw score directives inside transport timing code.
- The current playing-technique semantics are explicit: there is no implicit `Arco` before the first marking on a stave, hidden playing-technique entries still affect derived playback context, and conflicting same-group same-tick markings resolve by highest `UniqueId`.
- The transport clock must stay backend-owned and bounded. Do not go back to one full wall-clock sleep per tick; keep the short queue window, short wait quantum, and lead-frame scheduling model so transport can absorb moderate scheduler jitter without moving timing ownership into the frontend or audio callback.
- Treat `transport_pause` as transport-owned cleanup, not a frontend pause mode: it must clear the engine render target, flush active audio, join any prior transport thread, and republish the truthful stopped-state tempo at the paused tick before the next `transport_play`.
- Treat `transport_seek` as the same backend-owned transport family. When playback is active, seek must stop and join the in-flight transport thread, clear stale queued transport events plus active transport-owned notes from the old timeline, clamp and republish tick/tempo truth through the active `ui.flow_id`, and continue only from a fresh `prepare_supported_playback(...)` snapshot. When playback is paused or stopped, seek must only republish truthful tick and tempo without arming playback.
- Treat pause, stop, and natural transport teardown as the same hard-reset family: clear queued transport events, retire active transport notes deterministically, and ensure resume starts from the paused tick on a fresh backend snapshot rather than reusing stale engine state.
- Treat blocked `transport_play` or resume attempts as explicit backend no-ops through the same preparation seam. Clear stale transport stop flags, keep transport stopped, and preserve resolved `Unrouted` or `NeedsAttention` truth instead of inventing fallback playback or a frontend-owned readiness cache.
- The concrete playback-plane seam now lives in `src-tauri/src/playback/engine.rs`. Keep render-active ownership, backend event queues, and per-block slot-session processing there rather than hiding live audio production in `transport.rs`, React, or the editor-host POC.
- Expression maps remain a later extension over that transport-owned audible path. Do not make Epic 6 the first owner of real transport audio or split transport note scheduling into a second playback model. The first Epic 6 audible proof should stay transport-only, consume the durable linked map model, and preserve first-match-wins rule order rather than inventing an audition-specific expression path.
- Story 6.1 is foundational only: persist stable expression map references on route addresses inside `.soloc`, keep reusable map definitions in the app-scoped machine-local library under the app config directory, resolve missing library ids quietly back to unassigned score-context truth, and preserve the current default playback path when no map or an empty map is assigned.
- Story 6.2 is the first visible Expression Map surface: keep assignment and clearing in the Sequence instrument row only, collapse row truth to assigned-name or `--`, keep the adjacent launch action disabled when unassigned, and treat the opened Expression Map window as the handoff point into the app-scoped workflow rather than a score-context editor.
- Story 6.3 makes that workflow real: the Sequence menu entry and the instrument-row launch affordance must reuse the same dedicated Expression Map window, the left pane stays intentionally lightweight with search, selection, create, and delete, and the right pane is the selected-map authoring surface with auto-save instead of a manual apply/save ceremony.
- Story 6.3's current shipped rule semantics are the intentionally simplified editor model: rules evaluate top-down and stop at the first match, conditions within one rule are logical AND, articulation conditions use one row with `==` or `!=`, written-note-length conditions use one row with `<`, `<=`, `==`, `!=`, `>=`, or `>`, compound intent is expressed through multiple rows, and multiple actions may execute inside one matching rule.
- Story 6.4 is the first audible Expression Map proof and it stays transport-only: resolve linked maps at play-start through the same preparation snapshot, evaluate rules per played note with top-down first-match-wins semantics, prove the bounded articulation-plus-note-length case on the existing routed VST transport path, emit keyswitch notes and note-length transforms there, keep authored CC actions structurally supported but not newly proven through audition, and use the map-owned default action list when no rule matches. New maps still seed that default list with `NoOp`, so plugin-default routed playback remains the baseline until the map author says otherwise.
- Story 8.5 extends that same reusable rule vocabulary with one derived playing-technique condition family. Keep it on the existing primitive row model with `==` or `!=`, bound the first value set to `pizz` and `arco`, combine it with other rows through the same logical-AND and first-match-wins rules, and consume the prepared `effective_playing_techniques` context instead of scanning raw score markings inside Expression Map matching or transport.
- Story 8.6 is the first audible playing-technique proof on that same path: keep routed transport playback on the existing preparation plus scheduled-note pipeline, prove that `pizz` stays active until a later `arco` and that the switch is immediate for notes that start on the `arco` tick, and preserve map-default or plugin-default fallback when no technique rule matches. Do not add hidden playback regions, transport-owned technique state, audition-specific technique paths, or extra UI to satisfy this proof.
- Do not move Expression Map ownership into the bottom `PlaybackRackPanel`. Route-address-aware linkage may be backend-owned, but the user-facing control stays on the compact instrument row and the reusable asset remains app-scoped.
- Route-backed mix honesty now flows through that same backend seam: derive supported controls from `resolved_playback_route_by_instrument_id`, reuse score-owned `volume` / `pan` / `mute` / `solo` as intent only, and expose only the controls the current route can actually honor.
- The current MVP surface now matches the proven direct-plugin root mix path: ready direct-plugin root routes may expose `volume`, `mute`, `solo`, and `pan`, and each one must continue to flow through the backend-owned effective-mix seam rather than frontend-owned assumptions.
- The current audition workflow is a bounded pitch preview, not transport-lite playback: play one score-derived pitch for 500ms at moderate velocity with a default legato articulation, but inject those note events into the continuous engine instead of offline-rendering a burst buffer.
- Approved audition triggers are score-context only: tone selection through the renderer or sequencer, pitch changes through keyboard commands or sequencer vertical dragging, and accidental changes through the left-panel control or keyboard command.
- Keep stream/device lifetime in `src-tauri/src/playback/audio.rs`. Transport may submit engine state or later note events, but it must not take over OS stream creation, audio callback ownership, or canonical live plugin processing.
- Audio device and sample-rate rebuilds must stop transport through the transport-owned reset seam before rebuilding the stream, then republish tick and tempo truth from the current score state, rebase backend queue timing state such as `last_queued_tick`, and require the next `transport_play` to use a fresh snapshot.
- Score-context mix mutations for supported direct-plugin root routes must update both backend transport truth and the live engine mix state immediately; do not rely on a transport-only mirror that would require restarting playback to hear the change.
- Reopen restoration now restores portable rack and route intent across app restarts through the existing backend file-load seam. The load path clears stale deferred playback-refresh state, republishes truthful stopped transport state from the reopened score, and rebuilds runtime readiness from current machine-local inventory.
- Reopen remains a reset boundary, not a resume boundary: do not persist active transport playback, queued transport work, or live runtime host objects into `.soloc` files, and do not treat project reopen as a hidden resume-on-reopen workflow.

What it does not yet do:

- guarantee that every VST3 bundle will attach successfully; the current bounded validation claim is narrower than universal VST3 compatibility
- implement a production-ready Steinberg host contract, automation, preset/state persistence, or transport integration
- provide full keyboard or wheel coverage across all plugins

Story 7.2 maintainer validation found intermittent hangs on the two real-library control bundles. Story 7.2a identified three structural root causes (component not deactivated on detach, missing per-bundle session uniqueness, stale container views) and applied fixes in `plugin_editor_poc.rs`. The bundles remain `discovered but unvalidated` until maintainer manual revalidation confirms stability. See `docs/playback-support-baseline.md` for the current verdict and evidence.

## Canonical MVP Playback Baseline

Maintainer-facing playback support claims now live in `docs/playback-support-baseline.md`.

Use that document as the single source of truth for:

- the concrete known-good bundle list
- the concrete blocked bundle list for the bounded macOS native editor-host seam
- the meaning of `known-good`, `blocked/unsupported for the bounded seam`, `discovered but unvalidated`, and `future compatibility expansion`
- the release-gating discovery, host, and transport scenarios for Epic 7

Keep this guide focused on setup, architecture, and local workflow details. Do not add a second drifting support matrix here.

## Versioning

Version is kept in sync across three files:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

Use the custom release binary:

```bash
npm run prepare-release:patch   # prepares 1.25.0 → 1.25.1
npm run prepare-release:minor   # prepares 1.25.0 → 1.26.0
npm run prepare-release:major   # prepares 1.25.0 → 2.0.0
npm run finalize-release        # commits, tags, pushes, then reseeds next.rs
```

The release tool must be compiled first: `npm run release:build`

Release scripts are human-only. Agents must never run `prepare-release:patch`, `prepare-release:minor`, `prepare-release:major`, `finalize-release`, or any equivalent release/tag/publish flow.

For persisted-data or load/save changes, use this order:

1. Implement and validate the feature.
2. Add any required pending migration steps in `src-tauri/src/migrations/next.rs`.
3. Have a human run `prepare-release:*`, review and validate the prepared tree, then run `finalize-release` last so promotion, version sync, tagging, and post-release reseeding stay consistent.

Avoid manually editing synced version files just to reserve a migration target.

### Persisted-data release flow

- Development state keeps exactly one cumulative pending migration scaffold at `src-tauri/src/migrations/next.rs`.
- `prepare-release:*` computes the target version first, promotes any real pending steps from `next.rs` into a concrete versioned migration file, removes `next.rs` from the prepared release tree, updates synced version files, refreshes `Cargo.lock`, regenerates release notes, and writes `.release-prep.json` for finalize-time validation.
- If `next.rs` only contains the empty scaffold, prepare removes it from the prepared release tree and ships no new migration file for that release.
- `finalize-release` validates `.release-prep.json`, creates the concrete release commit and tag, then restores a fresh empty `next.rs` scaffold plus pending registry entry in a follow-up development commit before pushing branch HEAD and the new tag.
- Tagged releases must never contain `src-tauri/src/migrations/next.rs`.

## Instrument Tools

| Command                           | Description                                               |
| --------------------------------- | --------------------------------------------------------- |
| `npm run generate-manifest`       | Generate instrument manifest JSON from sample directories |
| `npm run generate-manifest:build` | Compile the manifest generator                            |

### Playback reset notes

- Compose no longer ships sampler-context, SCVI-runtime, or sampler-audition ownership on the product path.
- Treat plugin inventory plus `PlaybackActivationRuntime::from_score_and_inventory(...)` as the readiness seam, then derive any transport or audition handoff through `prepare_supported_playback(...)` instead of rebuilding playback preparation inside transport or probe code.
- Do not broaden audition into phrase playback, play/pause/seek semantics, or a plugin-only proof action while implementing the current first-audible story.
- Do not restore sampler-style fallback actions, events, or hidden runtime bridges when implementing later VST stories.

## Visual Regression Cases

Compose no longer keeps a committed visual-fixture corpus under `tests/fixtures/`.

For manual renderer checks, generate the smallest temporary score or `.soloc` case needed for the behavior under review, verify it locally, and discard the temporary artifact when the check is finished. Prefer seam-local automated tests over adding new committed manual-regression assets.

### `soloc.rs` — shared container logic

The container encoding functions (`build_soloc_container`, `compress_project_payload`, `decompress_project_payload`) and all `.soloc` format constants live in `src-tauri/src/soloc.rs`.

- `files.rs` — `use crate::soloc::*`

When making changes to the container format, update `soloc.rs` and validate the relevant load and save paths directly.

## Build Configuration

### Vite (Frontend)

- Multi-page app: 9 HTML entry points configured in `rollupOptions.input`
- Dev server on port 1420 (strict port)
- HMR enabled, ignores `src-tauri/` directory

### Tauri (Native Shell)

- macOS: Dark theme, overlay title bar, transparent background (`#0a0a0a`)
- Minimum macOS version: 10.14
- Bundle target: DMG
- Asset protocol enabled for cover images: `$DATA/Solo Compose/covers/**`

### Rust (Backend)

Release profile optimized for maximum performance:

- `panic = 'abort'`
- `codegen-units = 1`
- `lto = "fat"`
- `opt-level = 3`
- `strip = true`

## Testing

See [TESTING.md](../TESTING.md) for full details. Quick reference:

| Command                    | Description                                 |
| -------------------------- | ------------------------------------------- |
| `npm run test:js`          | Run all TypeScript tests once               |
| `npm run test:js:watch`    | Watch mode                                  |
| `npm run test:js:coverage` | Run with v8 coverage report                 |
| `npm run test:js:logs`     | Run JS tests once with verbose logs         |
| `npm run test:rust`        | Run all Rust unit tests (from `src-tauri/`) |

Test files are colocated next to the source files they test (e.g. `utils.ts` → `utils.test.ts`).

For manual visual regression checks, create temporary local cases on demand rather than committing fixture corpora.

## Playing-Technique Authoring Guardrails

- Treat the shipped playing-technique workflow as a native score-directive path. Visible markings render in the Rust score renderer, participate in native hit testing, and use the shared backend-owned selection model.
- Open the bounded playing-technique popover with `Shift+P` from any current score selection that carries a tick.
- Keep the accepted popover vocabulary intentionally narrow: `pizz` and `arco` only. Do not add aliases such as `pizz.` or a broader technique dictionary without an explicit follow-up story.
- If the current selection already has a `stave_id`, create the new playing-technique marking on that stave. If the selection only carries a tick, such as tempo, key signature, or time signature, resolve creation to the first stave in the selected flow's existing score order.
- When the popover is submitted from an already selected playing-technique marking, create the new marking at the same tick under the existing coexistence rules instead of mutating the selected entry in place.
- In the write-screen bottom-panel `Properties` tab, selected playing-technique markings route through the shared properties shell and expose only the `Common` `Hidden` toggle in the shipped panel scope.
- Hidden playing-technique entries remain non-rendered and non-selectable. Do not add frontend-only hidden-state affordances for them.
- Multiple visible same-tick playing-technique markings on one stave currently reuse the same canonical x and y anchor, even if that means the text overlaps. Treat collision avoidance, horizontal spreading, or vertical stacking as future work rather than implied shipped behavior.
- `Delete` and `Backspace` remove only the currently selected playing-technique entry through the remove-by-id seam. Do not replace that with a clear-at-tick behavior that would remove sibling same-tick markings.

## Tuplet Authoring Guardrails

- Treat the shipped authoring path as popover-first container creation plus sequencer editing of owned tones inside the resulting region. Do not imply a second freeform tuplet-construction workflow in the sequencer.
- Limit contributor-facing scope claims to the shipped triplet-first parser suffixes (`3q/2`, `3e/2`, `3s/2`, `3t/2`) unless a later story expands both UI and backend support together.
- Keep tone creation track-owned end to end. Extend `actions.score.tone.create(...)`, `score_entries_tone_create(...)`, and `Track::create_tone(...)` with optional tuplet context instead of introducing a parallel tuplet-note mutation API.
- Treat tuplets as one shared semantic model across persistence, parsing, rendering, and playback. Later consumers may adapt the model to their own outputs, but they must not locally reinvent tuplet ownership, boundary meaning, projection, or continuity.
- Keep tuplet creation on that same backend-owned seam. Container creation lives on `actions.score.tuplet.create(...)` / `score_entries_tuplet_create(...)`, where the backend reconciles existing material by moving only start-owned host tones into `Tuplet.entries` with unchanged durations and tuplet-local offsets; do not perform notation, rendering, playback, or resize semantics early in the creation path.
- Keep tone deletion and slicing on the shared backend-owned seams as well. Member deletion stays on `actions.score.tone.remove(...)` / `score_entries_tone_duration_remove(...)` with optional tuplet context, and container deletion lives on `actions.score.tuplet.remove(...)` / `score_entries_tuplet_remove(...)` rather than on UI-only stopgaps.
- Treat ticks and durations as relative to their immediate parent. Host tones use track-relative timing; tuplet child tones use tuplet-local timing.
- Keep tuplet container span separate from tone-duration consumption. A `3:2` eighth-note triplet still covers `2` host eighths as container geometry, but a crossing tone's duration is consumed literally on the canonical expanded timeline rather than being compressed to the denominator span.
- Phrase the invariant this way when handing work to humans or agents: "Tuplet geometry is compressed on the host track, but crossing duration is literal across tuplet boundaries." Use that rule for parser logic, sequencer geometry, slicing, and notation derivation.
- Phrase the playback carry-forward rule this way when handing work to humans or agents: "Playback is a consumer of the shared tuplet model, not a second semantics engine." Use that rule to keep transport scheduling logic from re-deriving tuplet semantics in local code.
- When writing inside a tuplet from the sequencer, derive local insertion positions directly from the tuplet region geometry. Do not compute a host tick first and then translate it back into tuplet-local time.
- Apply that same ownership-aware addressing rule to slicing: derive the split point from the canonical sequencer address lattice, pass the current owner plus the target owner context into the backend slice seam, and let Rust decide the persisted head/tail ownership by start tick.
- Preserve `ui.flow_id` as the authoritative active-flow slot. If a tuplet child tone needs selection handoff, carry only the minimal optional `tuplet_id` needed for direct lookup.
- Apply that same rule to renderer-owned rest selection when tuplet context is present: `tick` stays the single positional source of truth, while optional `tuplet_id` tells consumers whether the tick lives on the host track or inside a tuplet-local timeline. Do not introduce separate slot-index or alternate selection-position fields for tuplet UI state.
- In the sequencer, rests remain absence-of-note model state rather than a selectable or highlighted rendered object. Empty tuplet space is likewise non-selectable there; preserve only the tuplet-region metadata needed for local write-tool timing, not a selectable container or extra empty-slot hit boxes. Visible rest and tuplet selection belong to the printed-score renderer once notation interactions are materialized.
- Playback remains scoped to the active flow in MVP. Tuplet work must not use playback as a pretext to widen transport scope or introduce parallel flow-selection state.
- No dedicated tuplet migration is currently needed for legacy non-tuplet files because pre-tuplet releases produced no tuplet-bearing payloads. Reevaluate that only when a released older persisted format truly needs transformation.
- Nested tuplets, broader arbitrary ratios, and richer sequencer-native tuplet manipulation remain deferred post-MVP work. Do not smuggle those claims into implementation or docs as if they were already shipped.
- Keep tuplet regression coverage near the owning seam whenever existing local tests are sufficient. Prefer extending parser, renderer, playback, persistence, or editing tests in place over building a new cross-cutting tuplet test framework.
- Preserve the MVP single-level child-content rule in both code and tests. Reintroducing nested tuplets requires explicit post-MVP planning rather than incremental opportunistic support.
- Do not add frontend-owned parallel tuplet state or ad hoc load-time repair logic. If a seam exposes a gap, extend the shared backend-owned model or add the required migration instead of papering over it locally.

## Rhythm Rules

- Treat Elaine Gould as the notation authority when adding or changing written-duration behavior.
- Tuplet timing has a hard invariant: the container keeps its normal host span, while crossing tones consume duration literally through the tuplet's expanded local frame. Do not rewrite a crossing tone's total duration just because part of it passes through a compressed tuplet region.
- Supported compound-meter written-duration work is currently limited to eighth-based compound signatures such as `6/8`, `9/8`, `12/8`, and larger `x/8` signatures where `x` is a multiple of three greater than three.
- In supported compound meters, the visible big beat is a dotted quarter. Notes or chords that exactly occupy one big beat should stay unsplit unless a barline forces a split.
- Notes and chords that cross a compound big-beat boundary must split at that boundary so the big-beat structure stays visible. Sounding material must preserve tie continuity and full chord membership across the split.
- Full-bar rests stay full-bar rests regardless of meter. Non-full-bar rests that cross a compound big-beat boundary must split at that boundary.
- Full-bar compound notes may stay unsplit only when the resulting value is drawable in standard notation, such as a dotted half in `6/8`. Non-drawable full-bar values, such as `9/8`, must split into the largest drawable values first.
- Parser regressions for rhythm rules belong in `src-tauri/src/instructions/parse/get_written_durations.rs` as explicit scenario tests with canonical expected notation fragments, not as snapshot-style or loosely asserted coverage.
- Shared parser test harness code for written-duration scenarios belongs under `src-tauri/src/instructions/parse/test_support/`; keep only the reusable expectation normalization, fixture builders, and scenario runners there, with `get_written_durations_test_harness.rs` as the current home.
- Keep the musical scenario tables themselves in `src-tauri/src/instructions/parse/get_written_durations.rs` so the behavior examples stay next to the parser logic they document, and keep low-level helper tests local instead of forcing them through the scenario harness.
- New scenario-table regressions should follow the shared harness pattern, for example:

```rust
let four_four = make_time_signature_for_test(0, 4, NoteDuration::Quarter);

assert_written_duration_scenarios(vec![Scenario {
	name: "four straight quarter notes",
	time_signature: four_four.clone(),
	flow_length: four_four.ticks_per_bar(),
	tone_specs: vec![(1, 0, q, 60), (2, q, q, 62), (3, q * 2, q, 64), (4, q * 3, q, 65)],
	expectations: vec![
		notation_expectation(0, q, false, vec![1], vec![]),
		notation_expectation(q, q, false, vec![2], vec![]),
		notation_expectation(q * 2, q, false, vec![3], vec![]),
		notation_expectation(q * 3, q, false, vec![4], vec![]),
	],
}]);
```

- When compound rewriting can emit `SixtyFourth` or `HudredTwentyEighth` notes or rests, add parser-level scenario coverage for those values instead of relying only on draw tests.
- Beaming regressions that depend on compound grouping belong in `src-tauri/src/instructions/parse/get_beams.rs` and should assert the exact beat-group breaks that the written-duration output implies.

## Git Conventions

All commit messages must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<optional scope>): <description>

[optional body]
```

### Types

| Type       | Use for                                                  |
| ---------- | -------------------------------------------------------- |
| `feat`     | New user-facing feature                                  |
| `fix`      | Bug fix                                                  |
| `chore`    | Tooling, config, dependency updates (no production code) |
| `refactor` | Code restructure with no behaviour change                |
| `test`     | Adding or updating tests only                            |
| `docs`     | Documentation only                                       |
| `perf`     | Performance improvement                                  |
| `ci`       | CI/CD pipeline changes                                   |

### Examples

```
feat(sequencer): add swing quantisation
fix(pitch-utils): correct octave calculation for middle C
chore: upgrade Tauri to 2.11
test(duration): add round-trip coverage for from_ticks
docs: update development guide with testing setup
```

- Use the imperative mood in the description ("add" not "added")
- Keep the description under 72 characters
- Reference Jira tickets in the body when applicable: `SC-123`

## IDE Setup

Recommended: VS Code with extensions:

- Tauri
- rust-analyzer
- ESLint
- Prettier
- Tailwind CSS IntelliSense
