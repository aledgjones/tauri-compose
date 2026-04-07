# Solo Compose - MVP Playback Support Baseline

## Purpose

This document is the canonical maintainer-facing support baseline for MVP playback. It defines the only bounded subset Compose can treat as supported for release decisions today.

Use this artifact to answer three questions:

1. Which plugin bundles are currently inside the bounded MVP support claim?
2. Which scenarios must later Epic 7 validation stories execute before a release claim is credible?
3. Which discovered plugins or richer interactions remain outside the MVP claim until they are explicitly validated?

This baseline is intentionally narrower than "VST3 compatibility" in general. It is evidence-driven, macOS-only, VST3-only, and limited to rack-first routed playback through the current backend-owned transport path.

## Scope

- Platform: macOS only
- Plugin format: VST3 only
- Playback topology: rack-first routed playback on direct-plugin root routes
- Transport path: backend-owned `prepare_supported_playback_with_expression_maps(...)` plus `transport_play`
- Release posture: best-effort for sensible, spec-aligned plugins within this bounded subset only

Anything outside that scope is outside the current MVP support claim.

## Vocabulary

| Term                                       | Meaning                                                                                                                                                                                | Backend anchor                                                                                                                                        |
| ------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `known-good`                               | A concrete bundle plus scenario class that already has bounded positive evidence and is part of the representative release baseline.                                                   | Maintainer evidence plus backend truth such as `editor_host_validation.status = attached` and routed playback targets present in `supported_targets`. |
| `blocked/unsupported for the bounded seam` | A concrete bundle intentionally rejected by the current macOS native editor-host seam. This is a seam-specific block, not a blanket statement about the bundle in every possible host. | `src-tauri/src/playback/plugin_host_compatibility.rs` and `editor_host_validation.status = blocked`                                                   |
| `discovered but unvalidated`               | A bundle that the machine-local inventory can discover and mark available, but that is not yet part of the known-good support claim and has not been explicitly blocked.               | `src-tauri/src/playback/plugin_inventory.rs` with `availability = available`, no `validation_failure`, and no release-baseline evidence               |
| `future compatibility expansion`           | Any bundle, capability, or scenario intentionally left outside the MVP claim until a later story adds explicit evidence.                                                               | Future Epic 7 or post-MVP validation work                                                                                                             |

Do not collapse these terms together. In particular, "discovered" does not mean "supported," and "blocked in the bounded seam" does not mean "invalid bundle."

## Representative Plugin Baseline

### Release-baseline control bundles

| Bundle                                  | Current posture                                      | Baseline role                                                                                                | Notes                                                                                                                                     |
| --------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `BBC Symphony Orchestra.vst3`           | `discovered but unvalidated`                         | Primary control bundle for discovery, bounded native editor-host validation, and routed playback validation. | Story 7.2 manual host validation found intermittent open or hang behavior, so this bundle is not a bounded host `known-good` claim today. |
| `Originals - Intimate Grand Piano.vst3` | `discovered but unvalidated`                         | Secondary real-library control bundle and the second concrete BBC/Spitfire-family target in the baseline.    | Story 7.2 manual host validation found the same intermittent host instability as the BBC control bundle.                                  |
| `Reverb.vst3`                           | `known-good` for the bounded native editor-host seam | Host-behavior control bundle only.                                                                           | Do not use this bundle to broaden the routed score-playback claim beyond the current transport baseline.                                  |
| `ProEQ.vst3`                            | `known-good` for the bounded native editor-host seam | Host-behavior control bundle only.                                                                           | Do not use this bundle to broaden the routed score-playback claim beyond the current transport baseline.                                  |

## Story 7.2 Validation Results

Story 7.2 reuses this baseline as the only support matrix. The results below capture the evidence gathered so far without creating a second compatibility taxonomy.

### Discovery and rescan results

| Scenario                                         | Story 7.2 result | Evidence                                                                                                                                  | Support consequence                                                                                                     |
| ------------------------------------------------ | ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Startup cache reuse                              | Passed           | `cargo test plugin_inventory --lib -- --nocapture` passed, including startup cache reuse, empty-cache-present, and blocked-posture tests. | Startup remains bounded to cached-state validation and keeps blocked editor-host posture separate from bundle validity. |
| Startup cheap root sweep for newly added bundles | Passed           | `plugin_inventory_startup_discovers_new_uncached_bundles_during_bounded_sweep()` passed.                                                  | Startup can surface newly seen uncached bundles without widening into a full manual refresh path.                       |
| Manual refresh / full reprobe                    | Passed           | `manual_full_rescan_replaces_bounded_startup_snapshot_and_persists_new_truth()` passed.                                                   | Manual refresh remains the broader reprobe seam and preserves valid entries while refreshing machine-local truth.       |

Task 1 conclusion: existing seam-local inventory coverage was already sufficient. Story 7.2 did not add a second harness or broaden startup semantics.

### Bounded native editor-host results

| Bundle or scenario                                              | Story 7.2 result       | Evidence                                                                                                                                                                                         | Support consequence                                                                                                                  |
| --------------------------------------------------------------- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ |
| `BBC Symphony Orchestra.vst3` open/focus/close/reopen           | Unresolved instability | Maintainer manual validation on 2026-04-07 reported intermittent behavior: sometimes the editor opens, sometimes the UI hangs.                                                                   | Do not treat BBC Symphony Orchestra as `known-good` for the bounded host seam until the lifecycle instability is resolved.           |
| `Originals - Intimate Grand Piano.vst3` open/focus/close/reopen | Unresolved instability | Maintainer manual validation on 2026-04-07 reported the same intermittent behavior as the BBC control bundle.                                                                                    | Do not treat Originals - Intimate Grand Piano as `known-good` for the bounded host seam until the lifecycle instability is resolved. |
| Blocked bundle rejection                                        | Passed                 | Maintainer manually verified rejection for `HALion Sonic.vst3`, `HALion Sonic SE.vst3`, `Groove Agent SE.vst3`, `Kontakt 7.vst3`, and `MT-PowerDrumKit.vst3`; seam-local policy tests also pass. | The bounded seam still rejects these bundles before host attachment or visible host-window creation.                                 |
| Policy-gate and reservation semantics                           | Passed                 | `cargo test plugin_editor_poc --lib -- --nocapture` passed for blocked-target rejection, reservation cleanup, and reopen ownership tests.                                                        | Backend-owned host lifecycle evidence remains seam-local and is strong enough to support release review discussion.                  |

Story 7.2 hang verdict: unresolved. Intermittent hangs remain credible on the concrete BBC and Originals control bundles, and the MVP support claim must not ignore that instability.

### Concrete BBC / Spitfire library coverage note

For Epic 7, the planning requirement to validate against "BBC Spitfire libraries" is concretized to these two bundle targets:

- `BBC Symphony Orchestra.vst3`
- `Originals - Intimate Grand Piano.vst3`

Those are the relevant real-library coverage targets currently evidenced in repository docs. Epic 7 must validate against those concrete bundles, not against the family name alone.

### Blocked editor-host targets in the bounded seam

These bundles are intentionally blocked by the current macOS native editor-host seam. They may still appear in the machine-local inventory as discovered and available bundles, but they are outside the MVP support claim for bounded native editor hosting.

| Bundle                 | Bounded-seam posture                       | Current reason from owner seam                                                         |
| ---------------------- | ------------------------------------------ | -------------------------------------------------------------------------------------- |
| `HALion Sonic.vst3`    | `blocked/unsupported for the bounded seam` | Crashes at a null internal pointer during controller initialization.                   |
| `HALion Sonic SE.vst3` | `blocked/unsupported for the bounded seam` | Returns a null plugin factory.                                                         |
| `Groove Agent SE.vst3` | `blocked/unsupported for the bounded seam` | Crashes during controller initialization.                                              |
| `Kontakt 7.vst3`       | `blocked/unsupported for the bounded seam` | Crashes during component `createInstance`.                                             |
| `MT-PowerDrumKit.vst3` | `blocked/unsupported for the bounded seam` | Currently requires a teardown exception the bounded seam intentionally does not carry. |

The blocked list is owned by `src-tauri/src/playback/plugin_host_compatibility.rs`. If that seam changes, this baseline must be updated to match the live code path.

### Discovered but unvalidated boundary

Any other VST3 bundle that the inventory discovers is currently `discovered but unvalidated` unless this document explicitly places it in the known-good or blocked categories above.

That means:

- the bundle may be present in the machine-local inventory
- the bundle may validate structurally as a VST3 bundle
- the bundle may still be outside the MVP support claim

Compose must not make a release claim for those bundles until later validation stories add explicit evidence.

### Future compatibility expansion boundary

The following remain outside the current MVP claim until later work explicitly proves them:

- additional real-world libraries beyond the concrete baseline bundles above
- richer or more failure-prone hosts and multi-part plugin behavior
- broader transport realism claims beyond the current routed single-instrument transport path
- any Windows, Linux, AU, or non-VST3 posture

## Scenario Baseline

The baseline is repeatable only if each scenario names its owner seam, evidence source, and pass/fail meaning. Story 7.2 executes the discovery and bounded editor-host scenarios. Story 7.3 executes the transport-trust scenarios.

### Discovery and rescan scenarios for Story 7.2

| Scenario                                         | Owner seam                                   | Required setup                                                                               | Evidence to capture                                                                            | Pass meaning                                                                                                                   | Fail meaning                                                                                                                   |
| ------------------------------------------------ | -------------------------------------------- | -------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------ |
| Startup cache reuse                              | `src-tauri/src/playback/plugin_inventory.rs` | Start with a previously persisted machine-local inventory containing the baseline bundles.   | Emitted plugin inventory state and maintainer-visible inventory surfaces.                      | Baseline bundles appear without requiring manual refresh, and existing blocked posture remains separated from bundle validity. | Cached inventory is missing, stale, or loses the blocked-versus-available distinction.                                         |
| Startup cheap root sweep for newly added bundles | `src-tauri/src/playback/plugin_inventory.rs` | Add a new `.vst3` bundle under a standard macOS VST3 root between launches.                  | Startup inventory state after relaunch.                                                        | The newly seen bundle appears on next startup without needing a full refresh, while existing entries keep their current truth. | New bundles require a manual refresh or existing state regresses during startup refresh.                                       |
| Manual refresh / full reprobe                    | `src-tauri/src/playback/plugin_inventory.rs` | Trigger the Plugin Manager refresh action from a state that already contains cached results. | Refreshed inventory state, last-discovered timestamp, and maintainer-visible refresh feedback. | Refresh reprobes roots, preserves valid baseline entries, and truthfully updates blocked versus available posture.             | Refresh leaves stale results behind, drops valid entries, or rewrites seam-specific blocked state into generic bundle failure. |

### Bounded native editor-host scenarios for Story 7.2

| Scenario                      | Owner seam                                                                                              | Required setup                                                                                                  | Evidence to capture                                                             | Pass meaning                                                                                                                  | Fail meaning                                                                                                        |
| ----------------------------- | ------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Open known-good editor        | `src-tauri/src/playback/plugin_editor_poc.rs`                                                           | Prepare a routed slot for `BBC Symphony Orchestra.vst3` or another known-good host control bundle.              | Host window lifecycle plus `editor_host_validation` moving to `attached`.       | The host opens, attaches the editor, and records positive validation evidence for the concrete bundle.                        | No host window appears, attach fails, or evidence stays ambiguous.                                                  |
| Focus and responsiveness      | `src-tauri/src/playback/plugin_editor_poc.rs`                                                           | Keep the host window open and switch focus between main window and host window.                                 | Lifecycle logs and coherent UI behavior.                                        | Focus changes remain responsive and do not destabilize the attached editor.                                                   | Focus changes hang, detach the view, or corrupt lifecycle state.                                                    |
| Resize when supported         | `src-tauri/src/playback/plugin_editor_poc.rs`                                                           | Use a known-good bundle whose editor reports resize support.                                                    | Host resize behavior plus validation state remaining attached.                  | Resize stays coherent without stale content or resize loops.                                                                  | Resize breaks attachment, deadlocks, or fakes unsupported resize behavior.                                          |
| Close and same-session reopen | `src-tauri/src/playback/plugin_editor_poc.rs`                                                           | Open a known-good bundle, close it, then reopen it in the same app session.                                     | Lifecycle transitions and second open behavior.                                 | Close tears down cleanly, and reopen reattaches without stale reservation or restart.                                         | Close leaves stale ownership or reopen hangs/fails.                                                                 |
| Blocked bundle rejection      | `src-tauri/src/playback/plugin_host_compatibility.rs` plus `src-tauri/src/playback/plugin_inventory.rs` | Attempt the same editor-open flow on a concrete blocked bundle such as `HALion Sonic.vst3` or `Kontakt 7.vst3`. | `editor_host_validation.status = blocked` and absence of a created host window. | Compose rejects the attempt before host attachment and preserves the bundle as an inventory item rather than a broken bundle. | Compose creates a host window anyway, loses the blocked reason, or rewrites the state as generic bundle invalidity. |

### Transport-trust scenarios for Story 7.3

| Scenario                                                     | Owner seam                                                                         | Required setup                                                                                                           | Evidence to capture                                                                                         | Pass meaning                                                                                                                                                     | Fail meaning                                                                                                                       |
| ------------------------------------------------------------ | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Routed play start on supported target                        | `src-tauri/src/playback/preparation.rs` plus `src-tauri/src/playback/transport.rs` | Prepare one routed direct-plugin root target on a baseline bundle with valid audio output.                               | `supported_targets` snapshot at play-start plus audible transport behavior.                                 | The supported route is present in `supported_targets`, transport starts from backend truth, and score-derived note events are scheduled on that supported route. | Transport falls back to timing-only behavior, starts without a supported target, or invents frontend-owned readiness.              |
| Pause and resume from backend snapshot boundaries            | `src-tauri/src/playback/transport.rs`                                              | Start playback, pause, then resume from the paused tick.                                                                 | Tick/tempo republish behavior and fresh preparation snapshot on resume.                                     | Pause clears transport work cleanly and resume restarts from the paused tick on a fresh backend snapshot.                                                        | Pause leaves stale queued work alive or resume reuses stale runtime state.                                                         |
| Seek during stopped or active playback                       | `src-tauri/src/playback/transport.rs`                                              | Seek while stopped, then seek during active playback.                                                                    | Tick/tempo truth after seek and transport reset behavior.                                                   | Stopped seek only republishes truthful tick/tempo, and active seek clears old work then resumes only from a fresh supported snapshot.                            | Seek leaves stale notes queued, loses tick truth, or keeps playing from the old timeline.                                          |
| Bounded edit-during-playback                                 | `src-tauri/src/playback/transport.rs` plus `src-tauri/src/playback/preparation.rs` | During active playback, change `volume`, `pan`, `mute`, or `solo`; separately change score content or playback topology. | Audible mix response for supported live controls and deferred-effect behavior for structure/topology edits. | Live mix controls update immediately on already-ready supported routes, while score/topology edits defer to the next transport-owned reset boundary.             | Unsupported mid-pass rewrites happen, or live mix edits fail to reach the engine seam.                                             |
| Expression-map-aware routed playback on the current MVP path | `src-tauri/src/playback/preparation.rs` plus `src-tauri/src/playback/transport.rs` | Use a routed baseline bundle with a linked expression map on the currently supported transport path.                     | Play-start preparation snapshot and per-note action resolution behavior.                                    | The current bounded expression-map path remains transport-owned and uses the existing routed playback baseline without broadening the support claim.             | Expression-map playback creates a second playback model or obscures whether the underlying routed transport path is still trusted. |

## Evidence Rules For Release Decisions

A release-facing support claim is valid only when all of the following are true:

- the bundle is explicitly listed in this baseline as `known-good`
- the relevant discovery, host, or transport scenario has been executed against that bundle where required
- the evidence matches the owner seam named in this document
- no scenario result depends on frontend-owned fallback state or a second compatibility taxonomy
- any host verdict that shows unresolved instability is recorded explicitly instead of being rounded up to `known-good`

Reject an MVP support claim when any of the following is true:

- the bundle is only discovered, not validated
- the bundle is blocked in the bounded seam
- the scenario evidence is anecdotal rather than tied to the owner seam listed here
- the claim broadens from one known-good bundle or scenario to a whole plugin family without explicit proof

## Maintenance Rule

When Epic 7 or later work changes the bounded support subset, update this document first and then update any surrounding docs that reference it. `docs/development-guide.md` and `TESTING.md` should summarize or link, not carry a second drifting baseline.
