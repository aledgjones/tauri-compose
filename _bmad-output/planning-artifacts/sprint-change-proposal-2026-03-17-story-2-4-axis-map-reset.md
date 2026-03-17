# Sprint Change Proposal: Reset Story 2.4 to an Address-Lattice Implementation Path

Date: 2026-03-17
Prepared by: John, Product Manager
Mode: Batch
Change trigger: Story 2.4 implementation proved that the current host-ruler-based snap model breaks down once rendering and gesture hit-testing must stay coherent across tuplet-morphed regions.

## 1. Issue Summary

Story 2.4 was implemented along a path that tried to preserve the existing host-tick snapping model and layer tuplet-specific cursor translation, boundary repair, and debug instrumentation on top.

That approach produced working experiments and useful evidence, but it is not the right architectural center for the story. The result is a growing amount of boundary-specific logic attempting to reconcile:

- host-ruler pixel math
- tuplet-local spacing
- render projection
- move hit-testing
- resize hit-testing

The underlying issue is not one more boundary bug. The issue is that the sequencer does not currently have one authoritative horizontal spacing model when tuplets morph the local spacing.

The corrected direction is:

- keep the top tick bar as the host-track ruler
- keep the tuplet container rendering as the visual cue for local subdivision
- introduce one authoritative sequencer address lattice for rendering, hit-testing, and snapping

This is a controlled implementation reset, not a product-direction change. Story 2.4 acceptance criteria remain valid.

## 2. Impact Analysis

### Story Impact

- Story 2.4 should move from `review` or `ready-for-dev` back to `in-progress`.
- The current in-progress implementation should not be iterated further as the primary solution.
- The story artifact should be updated so the implementation notes and task guidance require the address-lattice approach.

### Artifact Impact

- `sprint-status.yaml` must reflect that Story 2.4 is active work again.
- The Story 2.4 implementation artifact must record the architectural correction.
- No epic renumbering or new story IDs are required.

### Code Impact

- The current frontend projection and gesture code changes are largely wrong-path work and are good candidates for reset.
- Debugging additions are optional and can be discarded with the rest of the exploratory implementation.
- Rust and action seam changes should be re-evaluated rather than automatically preserved; some may still be required if the corrected implementation still needs explicit ownership-transfer context at commit time.

## 3. Recommended Approach

Recommended path: controlled reset of Story 2.4 implementation, followed by reimplementation on the corrected address-lattice architecture.

### Why This Path

- It prevents further investment in a boundary-fix model that does not address the root cause.
- It keeps the original product intent and acceptance criteria intact.
- It allows the reimplementation to be simpler: one spacing model for both tone geometry and gesture resolution.
- It preserves the intended UX separation:
  - host tick bar is the ruler
  - tuplet overlay is the local cue
  - the canonical address lattice is the editing authority

### Explicit Non-Goal

This proposal does not change the product decision that the top tick bar remains visually non-tuplet-scaled.

## 4. Detailed Change Proposals

### 4.1 Story State Correction

- Move Story 2.4 to `in-progress`.
- Record that the prior implementation path is being discarded for architectural reasons, not because the story itself was incorrectly scoped.

### 4.2 Story Guidance Correction

Update Story 2.4 implementation guidance to require:

- one authoritative sequencer address lattice
- shared render, hit-test, and snap use of that canonical model
- replacement of host spacing by tuplet-local address boundaries inside tuplet-covered spans
- no parallel host and tuplet hit-test tracks for the same region
- host tick bar treated as visual-only reference for tuplets

### 4.3 Cleanup Guidance

Reset the current Story 2.4 implementation changes in the working tree, especially the exploratory projection, cursor, and debugging code.

Re-evaluate, rather than blindly preserve, any backend/action seam changes that may still be needed for final ownership transfer or backend validation under the address-lattice model.

## 5. Recommended File Triage

### Likely Reset

- `src/windows/main/components/sequencer-track.tsx`
- `src/windows/main/components/sequencer-tuplet-utils.ts`
- `src/windows/main/components/sequencer-track-tone.tsx`
- `src/windows/main/components/sequencer-track.test.tsx`
- `src/windows/main/components/sequencer-tuplet-utils.test.ts`
- `src/windows/debugger/index.tsx`
- `src/windows/debugger/index.test.tsx`
- `src/services/sequencer-debug.ts`

### Re-evaluate Against Corrected Design

- `src/services/actions.ts`
- `src/services/actions.test.ts`
- `src-tauri/src/score/entries/tone.rs`
- `src-tauri/src/score/entries/tuplet.rs`
- `src-tauri/src/automation/registry.rs`

## 6. Approval / Handoff

If approved, the next implementation slice should:

1. clean the current Story 2.4 code changes out of the working tree
2. preserve only any backend/action seam changes that are still justified by the address-lattice design
3. reimplement Story 2.4 from the corrected artifact guidance

This proposal is intentionally narrow. It corrects the implementation path without reopening Story 2.4 scope or backlog structure.
