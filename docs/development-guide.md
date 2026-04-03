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
- Same-session playback activation is now backend-owned and single-target only. Refresh the runtime activation seam by mutating score rack or route intent through the existing Rust commands or by loading a project; do not add frontend-owned activation caches or one-shot setup flows.
- Inventory refresh also rebuilds same-session activation truth. If a previously activated slot target becomes unavailable, keep the route in the existing score-context rack workflow and let backend-emitted readiness fall to `NeedsAttention` instead of repairing it through a Plugin-Manager-first side path.
- Transport-facing and audition-facing playback preparation must go through `src-tauri/src/playback/preparation.rs` via `prepare_supported_playback(...)`. That seam consumes `AppContext.playback_activation_runtime` plus the active audio sample rate, preserves explicit resolved-route states, and keeps the current MVP bounded to one supported direct-plugin target.
- The current audition workflow is a bounded pitch preview, not transport-lite playback: play one score-derived pitch for 1 second at moderate velocity with a default legato articulation.
- Approved audition triggers are score-context only: tone selection through the renderer or sequencer, pitch changes through keyboard commands or sequencer vertical dragging, and accidental changes through the left-panel control or keyboard command.
- Reopen restoration across app restarts is still Epic 5 scope. Story 3.8 covers same-session runtime preparation only.

What it does not yet do:

- guarantee that every VST3 bundle will attach successfully; the current bounded validation claim is narrower than universal VST3 compatibility
- implement a production-ready Steinberg host contract, automation, preset/state persistence, or transport integration
- provide full keyboard or wheel coverage across all plugins

Current bounded validation subset:

- Known-good bounded-host examples: `BBC Symphony Orchestra.vst3` remains the primary control case, and recent successful maintainer runs also include `Reverb.vst3`, `ProEQ.vst3`, and `Originals - Intimate Grand Piano.vst3`.
- HALion Sonic posture: repeated bounded-host repros now crash with the same `EXC_BAD_ACCESS` null dereference inside `HALion Sonic` during controller `initialize`, so `HALion Sonic.vst3` is again policy-blocked before host attachment instead of leaving the crash path available in maintainer UI flows.
- Explicitly policy-blocked targets in the current bounded host: the bounded native editor-host seam now blocks `HALion Sonic.vst3`, `HALion Sonic SE.vst3`, `Groove Agent SE.vst3`, `Kontakt 7.vst3`, and `MT-PowerDrumKit.vst3` before host attachment. Those entries remain available as inventory items when their bundles validate, but the bounded host path keeps that compatibility evidence separate from bundle validation failure.
- Structured host diagnostics: the bounded host now records phase-aware native-editor results such as `policy_gate`, `module_load`, `component_create`, `controller_initialize`, `view_create`, and `view_attach`. Successful attaches record that editor-host usability is proven while multi-part capability remains `not_proven` until a later bounded seam exposes plugin-derived facts.
- Current host-interface tradeoff: the host shim now advertises `IUpdateHandler` because Steinberg-family controllers are more likely to depend on it during initialization. The COM wrapper has a fixed interface tuple limit, so this bounded seam intentionally no longer advertises `IDataExchangeHandler` in the queryable host surface.

Manual validation checklist:

1. Start the app on macOS with either `SOLOC_PLUGIN_EDITOR_POC_VST3_PATH` set to an existing `.vst3` bundle or at least one valid `.vst3` bundle installed under the standard VST3 directories.
2. Confirm startup first reuses any existing machine-local cache in the Sequence `Plugins` bottom panel without requiring the refresh action.
3. If you add a new `.vst3` bundle under the standard discovery roots between launches, confirm the next startup surfaces it without requiring refresh.
4. If you need to force a broader discovery pass, trigger `plugin_inventory_discover` from the Plugin Manager `Refresh plugin inventory` action and verify the machine-local VST3 inventory updates there.
5. Exercise the bounded native editor-host seam through the score-context rack editor-open flow for a known-good available plugin.
6. Confirm the dedicated host window opens and remains responsive.
7. Confirm the Rust logs show create/focus events and, for a compatible plugin, that the plugin editor UI appears inside the host window rather than the previous placeholder-only surface.
8. If the plugin reports that its editor can resize, resize the window and confirm the native host area stays attached without leaving stale content behind. If the plugin does not report resize support, confirm the host window remains bounded instead of pretending arbitrary resize support exists.
9. Switch focus between the main window and the host window and confirm the app remains responsive and the plugin editor retains coherent focus behavior.
10. Close the host window, confirm the Rust logs show teardown without leaving stale host ownership behind, then reopen the same plugin in the same session.
11. Try the same action against an invalid or unavailable target via invalid `SOLOC_PLUGIN_EDITOR_POC_VST3_PATH` or an equivalent backend-targeted harness, and confirm the failure stays local to the host attempt without leaving an orphaned host window.
12. Try the same action against a blocked Steinberg-family target such as `HALion Sonic.vst3`, `HALion Sonic SE.vst3`, or `Groove Agent SE.vst3`, and confirm backend fallback protection rejects direct requests without creating or reserving a plugin-host window.
13. Try the same action against another blocked bundle such as `Kontakt 7.vst3` or `MT-PowerDrumKit.vst3`, and confirm the same policy-gate behavior holds without needing a live host attempt.

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
