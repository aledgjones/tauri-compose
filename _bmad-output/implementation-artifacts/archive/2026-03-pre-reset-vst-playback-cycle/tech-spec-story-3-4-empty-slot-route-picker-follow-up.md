---
title: "Story 3.4 Empty Slot Route Picker Follow-up"
type: "bugfix"
created: "2026-03-23"
status: "done"
baseline_commit: "6d6e6ef89280a665b50a3a3f4dc01e180b5095f4"
context:
  - "docs/project-context.md"
  - "docs/architecture.md"
  - "_bmad-output/implementation-artifacts/3-4-select-a-part-or-channel-when-assigning-a-compatible-multi-part-plugin.md"
---

# Story 3.4 Empty Slot Route Picker Follow-up

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The current Story 3.4 follow-up still treats existing empty rack slots as a recoverable route error and still renders target/address text in the instrument overview. That breaks the intended score-context UX because selecting an existing empty slot should remain a valid slot selection such as `1. --`, not an error state.

**Approach:** Keep the rack-first model unchanged, but make the Sequence instrument route picker operate as a slot picker only. Existing empty rack slots remain selectable and persist as numbered empty selections such as `1. --`, unrouted instruments stay at `--`, and the row removes target/address detail for now. The readiness badge should only surface unusable assigned-plugin states, not empty-slot selections.

## Boundaries & Constraints

**Always:** Preserve backend ownership of route truth. Keep slot deletion behavior as currently intended. Treat an existing empty selected slot as a valid non-error selection. Keep the picker bounded to current score-context slots only. Update tests to reflect live slot-label refresh after slot plugin changes.

**Ask First:** None for this slice.

**Never:** Reintroduce missing-slot recovery UX for slot deletion paths that now clear routes. Do not change persisted route schema, route command shape, or slot lifecycle semantics. Do not reintroduce target/address UI into the instrument overview for this slice.

## I/O & Edge-Case Matrix

| Scenario            | Input / State                                                                 | Expected Output / Behavior                                                                                              | Error Handling |
| ------------------- | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | -------------- |
| Empty selected slot | Instrument route points at an existing rack slot whose plugin target is empty | Route picker shows the selected slot label as `N. --`; no error badge is shown; the slot remains selected until changed | N/A            |
| Unrouted instrument | Instrument has no playback route                                              | Route picker shows `--` with no selected slot number                                                                    | N/A            |
| Removed slot        | A slot is removed after being routed                                          | Instrument becomes unrouted and frontend does not show stale missing-slot recovery UI for that deletion path            | N/A            |
| Slot plugin update  | User assigns or removes a plugin on an already-selected slot                  | The selected picker label refreshes between `N. --` and `N. Plugin Name` without losing the slot selection              | N/A            |

</frozen-after-approval>

## Code Map

- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` -- builds the slot-only picker and selected-value logic for the instrument row
- `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts` -- keeps empty selected slots out of the needs-attention badge path
- `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` -- slot-picker, empty-slot, and live label refresh coverage
- `src-tauri/src/score/instruments/mod.rs` -- backend route assignment and resolution for existing empty slots
- `docs/architecture.md` -- route/readiness semantics for score-context playback state
- `docs/development-guide.md` -- user-facing Sequence route authoring guidance

## Tasks & Acceptance

**Execution:**

- [x] `src/windows/main/screens/sequence/components/instrument-overview-row.tsx` -- make the row picker slot-only and preserve selected-slot identity for empty slots without showing target/address detail -- so row UX matches the clarified product rule
- [x] `src/windows/main/screens/sequence/components/playback-readiness-presentation.ts` -- treat existing empty selected slots as non-error state -- so the badge only appears for unusable assigned-plugin routes
- [x] `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` -- add and update tests for empty selected slot labels, empty-slot assignment, and live slot-label refresh after plugin changes -- so the intended UX is enforced
- [x] `src-tauri/src/score/instruments/mod.rs` -- allow direct routes to existing empty slots and resolve them without needs-attention status -- so FE and BE agree on persistent empty-slot selection
- [x] `docs/architecture.md` and `docs/development-guide.md` -- document the slot-only row picker and `N. --` behavior for selected empty slots -- so docs match the implemented UX

**Acceptance Criteria:**

- Given an instrument route points at an existing empty slot, when Sequence renders the instrument row, then the picker shows that numbered slot as `N. --` instead of the unrouted `--` option and does not show a needs-attention badge.
- Given an instrument has no route, when Sequence renders the instrument row, then the picker still shows the unrouted `--` option without implying a selected slot.
- Given a selected slot changes between empty and assigned plugin state, when Sequence re-renders from backend state, then the picker label refreshes between `N. --` and `N. Plugin Name` while preserving the selected slot.
- Given a routed slot is removed, when backend removal clears the route, then frontend and tests treat the instrument as unrouted rather than surfacing stale missing-slot deletion UI.

## Spec Change Log

- 2026-03-23: Closed the follow-up as done after fixing the row picker to use backend-owned route choices, rejecting unsupported non-root payloads for empty slots in Rust, and validating the current explicit-route command contract.

## Verification

**Commands:**

- `npm run test:js -- instrument-overview-row playback-rack-panel` -- expected: updated Sequence component tests pass
- `cargo test --lib score::instruments` -- expected: slot lifecycle and route tests pass
- `npm run check-types` -- expected: no TypeScript regressions in picker state or presentation helpers

**Completed:**

- `runTests src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` -- passed (23/23)
- `npm run check-types` -- passed
- `cd src-tauri && cargo test --lib score::instruments` -- passed (53/53)
- `cd src-tauri && cargo check --lib` -- passed
