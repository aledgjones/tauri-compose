---
title: "Research Basis For Epic 9: Multi-Instrument VST Support With HALion As The Reference Target"
slug: "investigate-multi-instrument-vst-support-with-halion"
created: "2026-04-10"
status: "promoted-to-epic-input"
type: "epic-research"
research_basis:
  - "Current Compose repo audit"
  - "Steinberg VST 3 technical documentation and SDK hosting examples"
  - "MuseScore VST and playback architecture prior art"
---

# Epic 9 Research Basis: Multi-Instrument VST Support With HALion As The Reference Target

**Created:** 2026-04-10

## Overview

### Problem Statement

Compose intentionally shipped a single-target-first MVP. That boundary kept the route model honest, but it also left HALion-style multi-timbral support unresolved. The current repo now contains four partial truths that need to be reconciled before any real multi-instrument work begins:

- the durable score model already allows `PluginPart` and `MidiChannel` targets;
- the live product path still emits only direct plugin-root route choices and rejects non-root assignments;
- the bounded native host seam can now attach some richer editors, but it still records HALion capability evidence as `NotProven` rather than proving advanced multi-part structure.
- the current bounded product posture still blocks HALion-family targets because they fail during load or initialization, so advanced-capability probing is not yet the first gate.

This investigation originally fed Epic 9 as an investigation-only scaffold. That investigation is now complete. The finished prior-art research and Story 9.1 evidence change the next step: Epic 9 should now harden the backend-owned VST3 host boundary using HALion-class failures as the reference case, not continue with verdict-only backlog stories. Current product posture remains blocked until host-core and compatibility work proves otherwise.

### Recommended Outcome

The immediate next step is now a tighter implementation sequence grounded in the completed research and the 2026-04-12 HALION reruns: keep Story 9.2's extracted host core as the canonical seam, replace probe-era guessed private-interface success paths with real host-owned public-contract hardening plus cleanup-in-place, and then rerun HALION-class initialization against that seam with one known-good control bundle for regression comparison. Scanner-style or broader compatibility-lane work is no longer the next story. Only after a bounded non-crashing load or initialization gate is cleared should Compose consider capability probing, multi-target route choices, or shared-slot follow-on work.

### Scope

**In scope:**

- HALion-family multi-timbral investigation as a bounded reference target.
- Load and initialization failure classification with phase-specific diagnostics.
- Steinberg-grounded host and routing contract review.
- Capability-fact discovery for units, bus-to-unit relations, and routing.
- A backend-owned path to say `not_proven`, `simple_mode_only`, `advanced_mode_proven`, or `blocked`.
- A story-ready follow-on plan for route-choice emission if proof exists.

**Out of scope:**

- Shipping shared-slot or multi-instrument UI.
- Changing the MVP support verdict.
- Claiming HALion is supported for editor hosting or playback.
- Broadening transport, audition, or reopen behavior.
- Non-macOS work or blanket multi-plugin compatibility expansion.

## Current Repo State

1. The persistent route model is already future-proofed.
   `src-tauri/src/score/playback.rs` defines `PlaybackRouteTargetMode::{Plugin, PluginPart, MidiChannel}` plus matching address modes.

2. The product path still collapses to direct plugin-root.
   `src-tauri/src/score/instruments/mod.rs` emits only one `Plugin output` choice for a populated slot, rejects non-root route assignment, and formats richer targets only as currently unsupported values.

3. Runtime preparation still refuses advanced route shapes.
   `src-tauri/src/playback/preparation.rs` only prepares routes whose target is `Plugin` and address is `Root`; everything else returns `None`.

4. Trust gating still treats richer routes as unsupported.
   `src-tauri/src/playback/runtime_activation.rs` carries `UnsupportedRouteShape` blockers for non-root routes.

5. The host shim is stronger than it was during the archived HALion investigation, but proof still stops at editor attachment.
   `src-tauri/src/playback/plugin_editor_poc.rs` now exposes `IPlugInterfaceSupport`, `IProgress`, `IUnitHandler`, `IUnitHandler2`, `IComponentHandlerBusActivation`, and `IEditControllerHostEditing`, yet the attached validation result still says capability evidence is only `NotProven`.

6. HALion-family targets are still blocked because current evidence says they fail during load or initialization.
   `docs/playback-support-baseline.md` and `src-tauri/src/playback/plugin_host_compatibility.rs` record that `HALion Sonic.vst3` crashes at a null internal pointer during controller initialization, while `HALion Sonic SE.vst3` returns a null plugin factory before the host can proceed.

7. Compose does not currently make a host-side simple versus advanced mode decision.
   There is no `setIoMode` call anywhere in `src-tauri/src`.

8. Compose does not yet interrogate plugin-side advanced-structure facts.
   There is no current use of plugin-side `IUnitInfo`, `getUnitByBus`, `selectUnit`, `getSelectedUnit`, or `getRoutingInfo` in the repo.

## Prior Art

### Steinberg official model

Steinberg's own documentation makes the core constraint explicit: multi-timbral instruments are not just "more channels". They are structured plug-ins whose host integration depends on plugin-reported units, routing, and strict call ordering.

| Source                                                                                   | Relevant takeaway                                                                                                                                                                                                                   | Compose implication                                                                                                                                                                      |
| ---------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Steinberg VST 3 Developer Portal, Complex Plug-in Structures / Multi-timbral Instruments | Steinberg distinguishes `simple mode` from advanced multi-timbral operation. Simple mode is mono-timbral, one input, one output, MIDI channel 0 only, and hosts can use multiple instances instead of one shared advanced instance. | Compose must stop treating "HALion-like plugin exists" as enough reason to emit part or channel targets. It needs an explicit simple-mode versus advanced-mode decision per plugin path. |
| Same Steinberg page                                                                      | If a host wants simple mode, `IComponent::setIoMode(...)` must be called before any other call.                                                                                                                                     | Compose currently has no `setIoMode` usage, so the investigation must explicitly decide whether the probe enters simple mode, advanced mode, or both.                                    |
| Same Steinberg page                                                                      | Advanced multi-timbral support relies on `IUnitInfo`, `IUnitHandler`, `IUnitData`, `IProgramListData`, `getUnitByBus`, `selectUnit`, `getSelectedUnit`, and `IComponent::getRoutingInfo`.                                           | Compose should gather plugin-derived capability facts from these interfaces rather than inventing route choices from enums or UI heuristics.                                             |
| Steinberg Host Requirements                                                              | Minimum host support includes the core host interfaces, `IPlugInterfaceSupport`, and MIDI-related interfaces when MIDI is supported.                                                                                                | Compose should compare its host shim against Steinberg's minimum contract before assuming HALion failures are "just HALion being difficult".                                             |
| Steinberg Workflow Diagrams and `IPlugFrame` docs                                        | Steinberg expects strict host call ordering and a real resize handshake: when a plug-in asks to resize, the host resizes the window and then calls `IPlugView::onSize()`.                                                           | Compose should continue treating editor lifecycle as a first-class host state machine, not just a view attachment trick.                                                                 |
| Steinberg SDK README and CMake                                                           | Steinberg ships hosting examples and a validator/test host in `public.sdk/samples/vst-hosting`.                                                                                                                                     | The investigation should use Steinberg's hosting examples and validator as direct reference implementations and comparison tools, not just prose docs.                                   |

### MuseScore and comparator hosts

MuseScore is useful here not because Compose should copy its UX, but because its architecture keeps instance identity, playback channelization, and editor lifecycle explicit.

| Source                                                                 | Relevant takeaway                                                                                                       | Compose implication                                                                                                                         |
| ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| MuseScore VST instance registry                                        | MuseScore registers instrument plug-ins by concrete instance id and track id, not by bundle family.                     | Compose should keep capability and editor state instance-owned and slot-owned, not bundle-owned.                                            |
| MuseScore `VstPluginInstance` / `createView()`                         | View creation is main-threaded and attached to a live plugin instance.                                                  | Compose should continue main-thread view work and avoid any design that tries to infer capability from inventory alone.                     |
| MuseScore engraving instrument/channel model                           | Instruments can own multiple playback channels while playback routing stays separate from notation semantics.           | Compose should keep score semantics, route intent, and plugin capability facts as distinct layers rather than overloading one route picker. |
| Existing repo comparator notes from MuseScore, Carla, Ardour, and JUCE | Robust hosts tie editor windows to concrete instances, guard resize feedback loops, and use explicit teardown ordering. | Any HALion follow-on must preserve backend-owned instance lifecycle and avoid overlapping reopen or optimistic UI-only state.               |

## Key Findings

1. The first blocker is crash-safe load and initialization, not multi-instrument routing UX.
2. Compose is structurally ready to represent advanced route intent, but not to prove it.
3. The largest missing seam is backend-owned capability discovery from a real plugin instance after it has reached a bounded non-crashing state.
4. HALion should remain the reference target for investigation, not the first product promise.
5. The current host shim is good enough to attempt richer diagnostics, but it does not yet prove advanced multi-timbral structure or routing.
6. The next correct slice is a real host-contract hardening spec, not more private-GUID shim experiments, shared-slot UX, or a new epic.

## Epic Seed Plan

Epic 9 should own the investigation work in Phases A and B plus a written verdict or follow-on recommendation. Phases C and D are possible future follow-ons only if the investigation produces bounded evidence strong enough to justify them.

### Phase A: Add a maintainer-only load and initialization failure classifier

**Goal:** determine how far a concrete reference plugin gets through bounded host setup before any capability claims are made.

**Deliverables:**

- A bounded backend probe that attempts and records the concrete load sequence:
  - module load and bundle entry;
  - plugin factory resolution;
  - component creation;
  - controller creation and initialization;
  - component-handler and connection-point setup;
  - view creation and attach if the earlier phases succeed.
- A structured failure or success result that preserves the first failing phase rather than collapsing everything into a generic blocked posture.
- A path to keep crash-prone plugins product-blocked while still allowing explicit maintainer-only diagnostic execution.

**Recommended status model:**

- `blocked_by_host_contract`
- `blocked_by_plugin_failure`
- `load_init_safe`

**Stop condition:**

If the probe cannot reach a bounded non-crashing `load_init_safe` state, stop. Do not continue to capability probing, route-choice emission, or shared-slot planning.

### Phase B: Add an advanced-capability probe after non-crashing load

**Goal:** determine what a concrete multi-timbral plugin actually reports once the host can load and initialize it safely enough to inspect real capability facts.

**Deliverables:**

- A bounded backend probe that can record:
  - whether `setIoMode` was attempted and with which mode;
  - whether the plugin exposes `IUnitInfo`;
  - unit count and unit-to-bus relations via `getUnitByBus`;
  - whether `getRoutingInfo` yields usable input-to-output mappings;
  - whether program-list and unit-selection interfaces are present;
  - whether editor attach succeeds independently of advanced capability proof.
- A structured capability result that is distinct from editor-host validation.

**Recommended status model:**

- `not_proven`
- `simple_mode_only`
- `advanced_mode_proven`

**Stop condition:**

If the probe cannot prove advanced mode from plugin-reported facts, Compose must not emit part or channel targets.

### Phase C: Keep route-choice emission backend-owned and proof-gated

**Goal:** only emit advanced route choices when live plugin proof exists.

**Deliverables:**

- `playback_route_choices_for_slot(...)` remains direct-root by default.
- A slot may emit `PluginPart` or `MidiChannel` choices only when live capability facts prove:
  - the plugin exposes a meaningful unit or routing structure;
  - the target-to-address relation is not ambiguous;
  - the route remains valid under current runtime activation and project reopen rules.
- React receives only verified choices; it never invents indices, part names, or channels.

**Stop condition:**

No guessed `PluginPart` or `MidiChannel` lists in frontend state or fixture-only backend code.

### Phase D: Add bounded shared-slot follow-on only after proof

**Goal:** revive the old Story 3.5 intent without distorting the rack-first model.

**Deliverables:**

- A future small story, not a whole epic by default, that:
  - reuses one existing slot across multiple instruments only through verified targets;
  - keeps slot lifecycle slot-owned and route lifecycle instrument-owned;
  - preserves per-instrument degraded state when a shared target becomes invalid.

**Stop condition:**

Do not start Phase D until Phase A can reach a bounded non-crashing load or initialization state and Phase B can produce plugin-derived advanced-capability facts for at least one concrete reference plugin.

## Recommended Implementation Boundary

### This research input should own

- A fresh HALion-oriented investigation artifact for the current repo state.
- Prior-art-grounded proof obligations for advanced multi-timbral support.
- A maintainer-only capability probe plan.
- The contract for when multi-target route choices may be emitted.

### This research input should not own

- Route picker UX changes.
- Shared-slot legibility UI.
- Transport or audition changes.
- Broad plugin-host compatibility work outside the reference target.
- Any documentation that implies HALion or multi-instrument playback is already supported.

## Files Most Likely To Change If This Spec Is Implemented

- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src-tauri/src/playback/plugin_host_compatibility.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src-tauri/src/score/playback.rs`
- `src/services/types.ts`
- `docs/architecture.md`
- `docs/data-models.md`
- `docs/development-guide.md`
- `docs/playback-support-baseline.md`
- `docs/project-context.md`
- `.github/copilot-instructions.md` only if the work establishes a new durable repo rule

## Acceptance Criteria

1. Given a HALion-like multi-timbral plugin is available locally, when the maintainer runs the bounded probe, then the repo records the concrete load or initialization phase reached and distinguishes plugin failure from host-contract failure.
2. Given the plugin still crashes or fails during load or initialization, when Compose computes support posture or route choices, then the target stays blocked and no advanced route choices are emitted.
3. Given the plugin reaches a bounded non-crashing `load_init_safe` state, when the maintainer runs the capability probe, then the repo records whether it is `simple_mode_only`, `advanced_mode_proven`, or still `not_proven`.
4. Given advanced mode is not proven, when the product computes route choices for a slot, then it still emits only direct plugin-root choices and does not guess parts or MIDI channels.
5. Given editor attachment succeeds but advanced capability is still unknown, when Compose records plugin validation, then editor-host usability and multi-instrument capability remain separate truths.
6. Given this work changes support posture or developer expectations, when the implementation is complete, then the relevant docs are updated in the same change and the support baseline remains explicit about what is still unproven.

## Verification Plan

- Add focused Rust coverage for:
  - load and initialization phase result serialization;
  - capability probe result serialization;
  - proof-gated route choice emission;
  - continued rejection of unsupported advanced route shapes when proof is absent.
- Keep a maintainer macOS runbook that exercises:
  - HALion load and initialization phase tracing;
  - HALion simple-mode probe;
  - HALion advanced-mode probe;
  - editor open, attach, close on the same live instance;
  - repeated reopen after probe teardown.
- Use Steinberg's hosting examples and validator as comparison references when a capability result or lifecycle phase looks suspicious.
- Do not treat a crash-free load or initialization result as evidence that part or channel routing is safe.
- Do not treat successful editor open as evidence that part or channel routing is safe.

## Documentation Impact

If this spec turns into implementation work, update:

- `docs/architecture.md` to define simple-mode versus advanced-mode capability proof and keep route truth backend-owned.
- `docs/data-models.md` to record any new capability-evidence payloads and the conditions under which advanced route choices exist.
- `docs/development-guide.md` to add the maintainer probe workflow and explicit stop conditions.
- `docs/playback-support-baseline.md` if the support claim or blocked posture for HALion changes.
- `docs/project-context.md` and `.github/copilot-instructions.md` only if the work establishes a new durable rule such as "advanced route choices must be plugin-derived proof only."

## Open Questions

1. Should the first probe path explicitly force Steinberg simple mode before attempting advanced mode, or should it inspect raw default behavior first and record both?
2. Do we want capability facts to be cached in machine-local inventory state, or only produced on explicit maintainer probe?
3. Should maintainer-only crash probing bypass the current product-level blocked-target gate, or should the first implementation temporarily preserve the gate and run diagnostics through a separate explicit command path?
4. Is HALion Sonic still the right reference target, or should this spec treat HALion-family behavior and a second reference plugin as a pair before any product claim moves?

## Internal References

- `_bmad-output/implementation-artifacts/archive/2026-03-pre-reset-vst-playback-cycle/tech-spec-investigate-halion-failure-before-story-3-5.md`
- `_bmad-output/implementation-artifacts/archive/2026-03-pre-reset-vst-playback-cycle/3-5-reuse-a-compatible-multi-part-plugin-instance-across-multiple-score-instruments.md`
- `src-tauri/src/score/playback.rs`
- `src-tauri/src/score/instruments/mod.rs`
- `src-tauri/src/playback/runtime_activation.rs`
- `src-tauri/src/playback/preparation.rs`
- `src-tauri/src/playback/plugin_inventory.rs`
- `src-tauri/src/playback/plugin_host_compatibility.rs`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `docs/architecture.md`
- `docs/playback-support-baseline.md`

## External Prior-Art References

- Steinberg VST 3 Developer Portal, Technical Documentation index: https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Index.html
- Steinberg, Complex Plug-in Structures / Multi-timbral Instruments: https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Complex+Structures/Index.html
- Steinberg, Minimum Host requirements for VST 3 support: https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Host+Requirements/Index.html
- Steinberg, VST 3 Workflow Diagrams: https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Workflow+Diagrams/Index.html
- Steinberg `IPlugFrame` documentation: https://steinbergmedia.github.io/vst3_doc/base/classSteinberg_1_1IPlugFrame.html
- Steinberg SDK hosting examples and validator packaging: https://github.com/steinbergmedia/vst3sdk/tree/main/public.sdk/samples/vst-hosting
- MuseScore VST instance registry: https://github.com/musescore/MuseScore/blob/master/src/framework/vst/ivstinstancesregister.h
- MuseScore live VST instance model: https://github.com/musescore/MuseScore/blob/master/src/framework/vst/internal/vstplugininstance.h
- MuseScore live VST instance implementation: https://github.com/musescore/MuseScore/blob/master/src/framework/vst/internal/vstplugininstance.cpp
- MuseScore instrument and playback-channel model: https://github.com/musescore/MuseScore/blob/master/src/engraving/dom/instrument.h
