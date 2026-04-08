---
title: "SC-183 flow switcher placement in sequence left panel"
type: "bugfix"
created: "2026-04-04"
baseline_commit: "e166799c7000ad46be9e84240474eed35403d017"
status: "done"
context:
  - "docs/project-context.md"
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** In the Sequence workflow, the sticky header above the instrument overview list currently leaves an empty spacer aligned with the main tick bar. That wastes the most appropriate place for workflow-local flow switching and keeps the active-flow selector away from the instrument list it controls.

**Approach:** Replace the empty spacer above the instrument overview list with the existing flow selector in the Sequence workflow's left panel, aligned with the tick-bar header. Keep the selector visible whenever at least one flow exists, preserve `ui.flow_id` as the only active-flow source, and leave unrelated toolbar surfaces unchanged.

## Boundaries & Constraints

**Always:** Preserve `ui.flow_id` as the only active-flow source of truth; keep flow switching inside the Sequence workflow surface instead of the shared top bar; reuse the existing `actions.ui.setActiveFlowId` path and current flow fallback logic; keep the selector visible whenever at least one valid flow exists.

**Ask First:** Any change that would introduce a second flow selector in Sequence, move flow switching into the shared top bar, or require broader refactoring of the left-panel sticky layout beyond this header slot replacement.

**Never:** Do not add new flow state, do not move flow switching into the shared app bar, do not change flow naming, do not repurpose the Write workflow toolbar for this bug, and do not broaden this fix into the other Ready bugs.

## I/O & Edge-Case Matrix

| Scenario      | Input / State                                           | Expected Output / Behavior                                                                                                                                                                    | Error Handling |
| ------------- | ------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| HAPPY_PATH    | Sequence screen is visible and at least two flows exist | The flow selector renders in the sticky left-panel header slot above the instrument overview list, aligned with the main tick bar, and still changes `ui.flow_id` through the existing action | N/A            |
| SINGLE_FLOW   | Sequence screen is visible and only one flow exists     | The flow selector still renders in the sticky header, showing the sole flow without leaving an empty spacer block                                                                             | N/A            |
| FALLBACK_FLOW | `ui.flow_id` is null or stale but flows exist           | The selector still displays the fallback active flow resolved by the existing hook and changing it updates the active flow normally                                                           | N/A            |

</frozen-after-approval>

## Code Map

- `src/windows/main/screens/sequence/components/instrument-overview.tsx` -- owns the Sequence left-panel sticky header and the current spacer/select slot above the instrument overview list.
- `src/windows/main/screens/sequence/components/instrument-overview.test.tsx` -- should cover the Sequence header placement and flow-selector behavior.
- `src/windows/main/index.test.tsx` -- already guards workflow-specific flow-switch placement and may need expectation updates if the rendered placement semantics change.

## Tasks & Acceptance

**Execution:**

- [x] `src/windows/main/screens/sequence/components/instrument-overview.tsx` -- replace the sticky header spacer above the instrument overview list with the existing flow selector and keep the fallback/empty-state behavior intact, including the single-flow case -- fixes the visible placement bug at the owning seam.
- [x] `src/windows/main/screens/sequence/components/instrument-overview.test.tsx` -- add or update tests to assert the Sequence left-panel header shows the flow selector when flows exist, including the single-flow case, and that changes still dispatch through the existing action -- prevents a layout fix from regressing flow switching.
- [x] `src/windows/main/index.test.tsx` -- confirm workflow-specific flow switching still stays out of the shared top bar while remaining available in the Sequence workflow -- preserves the current architecture rule.

**Acceptance Criteria:**

- Given the Sequence workflow has one or more flows, when the instrument overview header renders, then the flow selector appears in the spacer slot above the instrument list and aligns with the main tick-bar header.
- Given multiple flows are available in the Sequence workflow, when the user changes the selector, then the header still dispatches `actions.ui.setActiveFlowId` with the selected flow id.
- Given the Sequence workflow has exactly one available flow, when the instrument overview header renders, then the selector still shows that flow and the header does not fall back to an empty spacer.
- Given the Sequence workflow has zero available flows, when the instrument overview header renders, then the empty-state screen still appears instead of a broken selector.

## Spec Change Log

## Documentation Impact

No documentation update is required for SC-183 because this is a local Sequence-panel placement fix with unchanged architecture, persistence, and user-facing capability. The spec still records that assessment explicitly so doc impact is not left implicit.

## Design Notes

The smallest correct fix is to reuse the existing Sequence left-panel flow-switch surface inside `instrument-overview.tsx` instead of introducing a second selector or moving ownership into another workflow surface. That keeps the control next to the instrument list it filters, preserves the single `ui.flow_id` contract, and avoids broadening the fix into shared toolbar layout work.

## Verification

**Commands:**

- `npm run test:js -- src/windows/main/screens/sequence/components/instrument-overview.test.tsx src/windows/main/index.test.tsx` -- expected: both updated frontend test files pass.
- `npm run check-types` -- expected: no TypeScript regressions from the toolbar prop/layout changes.

## Suggested Review Order

**Flow resolution and header placement**

- Harden the active-flow fallback before rendering the header selector.
  [`instrument-overview.tsx:24`](../../src/windows/main/screens/sequence/components/instrument-overview.tsx#L24)

- Keep the selector visible in the sticky spacer slot whenever a flow exists.
  [`instrument-overview.tsx:111`](../../src/windows/main/screens/sequence/components/instrument-overview.tsx#L111)

**Acceptance coverage**

- Cover the mixed-validity fallback case that previously collapsed to no-flow UI.
  [`instrument-overview.test.tsx:135`](../../src/windows/main/screens/sequence/components/instrument-overview.test.tsx#L135)

- Prove the single-flow header still shows the selector.
  [`instrument-overview.test.tsx:185`](../../src/windows/main/screens/sequence/components/instrument-overview.test.tsx#L185)

- Verify selector changes still dispatch the shared active-flow action.
  [`instrument-overview.test.tsx:212`](../../src/windows/main/screens/sequence/components/instrument-overview.test.tsx#L212)

**Architecture guardrail**

- Confirm flow switching remains Sequence-local and out of the shared top bar.
  [`index.test.tsx:104`](../../src/windows/main/index.test.tsx#L104)
