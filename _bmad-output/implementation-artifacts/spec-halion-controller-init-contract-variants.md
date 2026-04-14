---
title: "Investigate HALion Controller Initialize Contract Variants"
type: "bugfix"
created: "2026-04-10"
status: "done"
context:
  - "../../docs/project-context.md"
  - "../../docs/development-guide.md"
  - "tech-spec-investigate-multi-instrument-vst-support-with-halion.md"
baseline_commit: "e9e8535a"
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** HALion Sonic still hard-crashes during `controller_initialize`, and the current maintainer tooling only proves that the crash happens after controller creation. That is not enough to tell whether the real blocker is pre-init handler timing, an over-broad optional host interface surface, or some other bounded host-contract mismatch.

**Approach:** Extend the maintainer-only load/init probe with explicit controller-initialize contract variants that can change handler timing and host interface advertisement without touching ordinary product behavior. Surface the selected variant and its resulting phase trace in the debugger so maintainers can identify which bounded contract change actually moves the failure boundary before we promote any fix into the normal host seam.

## Boundaries & Constraints

**Always:** Keep ordinary blocked posture, route-choice behavior, preparation, runtime activation, and normal editor-open behavior unchanged. Apply variant logic only inside the maintainer probe path. Keep backend-owned diagnosis truth in Rust; React may render selected profile and results but must not invent failure interpretations.

**Ask First:** If the only apparent fix requires changing ordinary product-facing host behavior before a maintainer variant proves it. If HALion only progresses by adding new host callbacks or interface contracts that materially widen the bounded seam beyond controller-init investigation.

**Never:** Do not claim HALion is supported, do not add shared-slot or multi-instrument route behavior, do not introduce frontend-owned crash diagnosis logic, and do not leave the maintainer workflow console-only.

## I/O & Edge-Case Matrix

| Scenario                            | Input / State                                                                           | Expected Output / Behavior                                                                                                                                                          | Error Handling                                                                                                                                                                   |
| ----------------------------------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Default profile                     | Maintainer runs the probe with the default controller-init contract                     | Probe preserves current bounded behavior, records the selected profile, and returns the same lifecycle evidence as today                                                            | If the app still dies, the persisted log and crash hint must include the selected profile so restart inspection stays actionable                                                 |
| Host-only lifecycle-aligned profile | Maintainer runs the probe with the host-only lifecycle-aligned controller-init contract | Probe records whether HALion still dies inside `controller_initialize` after the init-time context is reduced to `IHostApplication`, without affecting ordinary editor-host posture | If the host-only variant still fails, the result must remain investigation-only and keep normal blocked posture intact while exposing direct init-time `queryInterface` evidence |

</frozen-after-approval>

## Code Map

- `../../src-tauri/src/playback/plugin_editor_poc.rs` -- owner seam for maintainer probe requests, controller-init ordering, host interface advertisement, persisted probe logs, and focused Rust tests.
- `../../src/services/types.ts` -- frontend mirror of maintainer probe request/result profile fields.
- `../../src/services/actions.ts` -- Tauri bridge for maintainer probe profile selection.
- `../../src/windows/debugger/index.tsx` -- maintainer control surface for selecting and running controller-init variants without the console.
- `../../src/windows/debugger/index.test.tsx` -- debugger coverage for profile selection, result display, and persisted-log refresh behavior.
- `9-1-classify-reference-multi-instrument-plugin-load-and-initialization-failures.md` -- active Story 9.1 artifact that should record the new bounded investigation step.

## Tasks & Acceptance

**Execution:**

- [x] `../../src-tauri/src/playback/plugin_editor_poc.rs` -- add maintainer-only controller-init probe profiles that can vary handler timing and optional host interface advertisement during `plugin_editor_poc_probe_load_init`, record the active profile in probe results and logs, and keep the normal editor-open path on the current contract -- this is the narrowest way to distinguish the two leading HALion crash hypotheses without broadening product behavior.
- [x] `../../src/services/types.ts` -- add the shared profile metadata needed to move maintainer probe selection and persisted-log outcome across the FE/BE boundary -- the debugger must render backend-owned profile truth instead of inferring which contract was used.
- [x] `../../src/services/actions.ts` and `../../src/windows/debugger/index.tsx` -- let maintainers choose the controller-init profile in the debugger, pass it through prepare/read/run commands, and show the selected profile alongside the returned probe evidence -- this keeps investigation off the console and makes profile-to-outcome comparisons practical.
- [x] `../../src-tauri/src/playback/plugin_editor_poc.rs`, `../../src/services/actions.test.ts`, and `../../src/windows/debugger/index.test.tsx` -- add focused Rust and frontend coverage for default versus host-only lifecycle-aligned profile wiring and persisted-log rendering -- the new investigation seam needs regression protection because it sits in the active playback host owner.
- [x] `../../docs/development-guide.md` and `9-1-classify-reference-multi-instrument-plugin-load-and-initialization-failures.md` -- document the controller-init investigation variants and sync the active story notes with the new bounded fix-finding step -- maintainer workflow and story artifacts must stay aligned with code.

**Acceptance Criteria:**

- Given a maintainer runs the HALion load/init probe with either available controller-init profile, when the probe starts, then the selected profile is recorded in backend-owned probe data and visible after restart through the debugger.
- Given the host-only lifecycle-aligned profile changes only the maintainer probe's init-time host context, when ordinary inventory or editor-host behavior is computed, then current blocked product posture and normal host behavior remain unchanged.
- Given the host-only lifecycle-aligned profile still leaves HALion crashing inside `controller_initialize`, when maintainers inspect the result, then the repo records enough profile-specific evidence to decide whether the remaining likely root cause sits in direct init-time COM interface negotiation rather than broader lifecycle ordering.

## Spec Change Log

- 2026-04-10 to 2026-04-11: Several bounded maintainer-only controller-init profile permutations were rerun manually. Every one still stopped at `controller_initialize`, which ruled out handler timing, stricter lifecycle ordering, reduced optional host exposure, and a handler-free init context as sufficient explanations on their own.
- 2026-04-11: HALion then proved it reaches `IHostApplication::getName` during `controller_initialize`, but the Rust `vst3` binding aliases `kResultTrue` to `kResultOk`, so a get-name return-code tweak would be a no-op. The next bounded maintainer-only experiment adds `controller_init_host_only_lifecycle_aligned` to remove `IPlugInterfaceSupport` from the init-time context entirely while preserving the strictest proven runtime ordering.
- 2026-04-11: Matching macOS `DiagnosticReports` for the host-only lifecycle-aligned reruns show the same `EXC_BAD_ACCESS (KERN_INVALID_ADDRESS at 0x40)` inside the `HALion Sonic` binary while Compose is still on the `IPluginBase::initialize` stack, so the remaining bounded question is which init-time host response HALion fails to tolerate immediately before that plugin-side null dereference.
- 2026-04-11: The next bounded maintainer-only iteration keeps the host-only lifecycle-aligned profile isolated to `controller.initialize()`, but adds a balanced non-null `FUnknown` fallback object for `IVst3ToVst2Wrapper`, `IVst3ToAAXWrapper`, and `IVst3ToAUWrapper` while still returning `kNoInterface`, so the rerun can test whether HALion is crashing because it releases unsupported wrapper-probe results without checking for null.

## Design Notes

Use the smallest profile set that still isolates the current hypotheses in this slice:

- `default`: today’s maintainer probe contract, preserved for comparison.
- `controller_init_host_only_lifecycle_aligned`: keep the lifecycle-aligned runtime ordering and post-init handler wiring, but pass a pure `IHostApplication` object into `controller.initialize()` so HALion cannot rely on `IPlugInterfaceSupport` during the crashy init call.

That keeps the investigation legible. Earlier retired profile permutations already ruled out handler timing, processor-runtime ordering, extra optional host exposure, and `IComponentHandler` visibility on the init context as sufficient explanations for HALion's controller-init crash. The `getName` callback breadcrumb ruled out a trivial return-code mismatch because this binding treats `kResultTrue` and `kResultOk` identically. The remaining host-only lifecycle-aligned profile now changes only the controller-init host context while keeping factory and component setup on the ordinary full host. It still does not claim to implement those wrapper environments, but it no longer returns a null COM object for `IVst3ToVst2Wrapper`, `IVst3ToAAXWrapper`, or `IVst3ToAUWrapper`. If the crash boundary moves after that change, the remaining evidence points at HALion releasing unsupported wrapper-probe results without checking for null rather than at broader lifecycle reshuffling.

Run the maintainer probe from the debugger against HALion Sonic with `default` and `controller_init_host_only_lifecycle_aligned`, and confirm the selected profile is visible in the UI and in the persisted log after restart.

## Verification

**Commands:**

- `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc --lib -- --nocapture` -- expected: new probe-profile tests pass and existing maintainer probe coverage stays green.
- `npm run check-types` -- expected: new profile request/result fields stay aligned across Rust and TypeScript.
- `npm run lint` -- expected: debugger and action changes stay lint-clean.

**Manual checks (if no CLI):**

- Run the maintainer probe from the debugger against HALion Sonic with `default` and `controller_init_host_only_lifecycle_aligned`, and confirm the selected profile is visible in the UI and in the persisted log after restart.

## Suggested Review Order

**Backend probe flow**

- Start at the owner seam that now orchestrates the bounded maintainer probe.
  [`plugin_editor_poc.rs:4183`](../../src-tauri/src/playback/plugin_editor_poc.rs#L4183)

- Wrapper-marker probes stay unsupported but no longer hand HALion a null COM object.
  [`plugin_editor_poc.rs:2866`](../../src-tauri/src/playback/plugin_editor_poc.rs#L2866)

- Crash hints now survive callback-note lines after a phase has started.
  [`plugin_editor_poc.rs:286`](../../src-tauri/src/playback/plugin_editor_poc.rs#L286)

**Shared contract and wiring**

- The nested maintainer payload formalizes first-failure evidence for inventory consumers.
  [`plugin_inventory.rs:94`](../../src-tauri/src/playback/plugin_inventory.rs#L94)

- Tauri wires the prepare, run, and read commands into one backend entry surface.
  [`lib.rs:1235`](../../src-tauri/src/lib.rs#L1235)

- The frontend mirror exposes probe profile, result, and persisted-log shapes.
  [`types.ts:796`](../../src/services/types.ts#L796)

- Action helpers keep all FE-to-BE probe calls on the shared invoke path.
  [`actions.ts:191`](../../src/services/actions.ts#L191)

**Debugger surface and maintainer guidance**

- The debugger rehydrates persisted probe state before any new risky run.
  [`index.tsx:45`](../../src/windows/debugger/index.tsx#L45)

- The maintainer trigger path now prepares logs, runs the probe, and refreshes evidence.
  [`index.tsx:271`](../../src/windows/debugger/index.tsx#L271)

- The runbook explains controller-only host context and the `FUnknown` fallback experiment.
  [`development-guide.md:196`](../../docs/development-guide.md#L196)

- The active story log records the wrapper-probe fallback iteration in the story artifact.
  [`9-1-classify-reference-multi-instrument-plugin-load-and-initialization-failures.md:257`](9-1-classify-reference-multi-instrument-plugin-load-and-initialization-failures.md#L257)

**Regression coverage**

- Rust tests pin the non-null wrapper fallback behavior directly.
  [`plugin_editor_poc.rs:6388`](../../src-tauri/src/playback/plugin_editor_poc.rs#L6388)

- Rust tests also pin crash-hint recovery when later info lines are appended.
  [`plugin_editor_poc.rs:6626`](../../src-tauri/src/playback/plugin_editor_poc.rs#L6626)

- Debugger tests cover persisted-log hydration and the maintainer run path.
  [`index.test.tsx:83`](../../src/windows/debugger/index.test.tsx#L83)
