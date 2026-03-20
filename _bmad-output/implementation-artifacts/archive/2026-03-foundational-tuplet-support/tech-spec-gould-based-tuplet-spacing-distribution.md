---
title: "Shared region-based tuplet spacing distribution"
slug: "gould-based-tuplet-spacing-distribution"
created: "2026-03-19"
status: "implemented"
stepsCompleted: [1, 2, 3, 4, 5, 6]
tech_stack:
  - "Rust 2021"
  - "Tauri 2"
  - "Skia/Metal score render pipeline"
  - "rayon-backed parse pipeline conventions"
files_to_modify:
  - "src-tauri/src/instructions/parse/get_horizontal_spacing.rs"
  - "src-tauri/src/instructions/mod.rs"
  - "src-tauri/src/instructions/draw/draw_tuplet.rs"
  - "_bmad-output/implementation-artifacts/3-4-lay-out-tuplets-legibly-within-the-existing-score-surface.md"
  - "docs/project-context.md"
code_patterns:
  - "Backend-owned horizontal spacing and slot model"
  - "Shared tuplet semantic model across parse, draw, and playback"
  - "Focused seam-local regressions for tuplet behavior"
  - "Downstream draw and hit-target consumers read resolved spacing x-positions rather than recomputing layout"
  - "Repo-owned restatement of the shared region-grid rule must be explicit so implementation does not depend on ambiguous historical wording"
test_patterns:
  - "Parser spacing regressions near get_horizontal_spacing.rs"
  - "Immediate downstream render or hit-target regressions only if spacing outputs change behavior"
  - "Existing draw_tuplet.rs unit coverage can validate bracket and hit-target behavior against changed spacing output"
---

# Tech-Spec: Shared region-based tuplet spacing distribution

**Created:** 2026-03-19

User clarification 2026-03-19: the spacing source-of-truth is the resolved `Slot::NoteSpacing` demand, not raw written-duration ratios by themselves. Any rule must preserve spacing pressure from accidentals, shunts, and other notehead-offsetting layout needs.

Follow-up clarification 2026-03-19: the layout owner cannot think about one tuplet in isolation. Any overlapping anchored spans must negotiate against one shared rhythmic grid before host and local spacing are finalized.

## Overview

### Problem Statement

The prior tuplet spacing distribution was too local because it let host and anchored layouts stretch independently. That produced correct-looking local tuplets in some cases, but simultaneous rhythms could still drift across tracks or diverge when extra spacing pressure came from accidentals and other resolved layout needs.

### Solution

Update the backend horizontal-spacing model so overlapping anchored spans share one temporary LCM-resolution rhythm grid, aggregate resolved note-spacing pressure there, and then project the result back into ordinary host spacing and anchored local spacing. Drawing and hit-target consumers continue to inherit the resolved spacing indirectly.

### Scope

**In Scope:**

- Rust note-spacing logic for tuplets in the shared score-surface spacing seam
- Region-based shared spacing resolution for overlapping anchored spans
- LCM-derived temporary grid resolution across host ticks and participating local lengths
- Focused regressions at the spacing seam and any immediately affected downstream draw or hit-target seams
- Documentation impact review for any durable rule change

**Out of Scope:**

- Frontend UX changes or new engraving controls
- Full-flow permanent high-resolution spacing storage
- Expanded tuplet feature scope beyond the shipped MVP semantic model
- Any parallel frontend-owned tuplet layout logic

## Context for Development

### Codebase Patterns

- Tuplets must continue to use the shared backend-owned semantic model across parsing, layout, drawing, playback, and persistence.
- The tuplet timing invariant stays intact: host geometry is compressed to the denominator span while duration on the canonical expanded timeline remains literal.
- Horizontal spacing changes should extend the existing `HorizontalSpacing` and `Slot` model rather than creating a tuplet-only layout path.
- Slot additions are exceptional and currently not justified by this request.
- Tests for tuplet spacing behavior should stay close to the owning seam unless a downstream seam must change to remain correct.
- The old even-distribution behavior was concentrated in the helper seams that widened host spacing and stretched anchored local spacing independently.
- Draw, stem, notehead, bracket, and hit-target code already consume `Slot::NoteSpacing` x-positions generically, so they should inherit a proportional rule without becoming new layout owners.
- Cross-context tie ordering uses host/local tick projection rather than spacing x-values, so proportional spacing should not redefine tie fragment ordering.
- `anchored_track.host_span` is the exclusive host-tick range `[start_tick, end_tick)` that the tuplet container compresses into on the host track.
- The implementation groups anchored participants by overlapping host spans, not by exact host-span equality, so partially overlapping tuplets resolve through one shared region.
- The temporary shared grid is not persisted; it exists only long enough to reconcile host spacing and anchored local spacing before widths are written back onto their actual tick structures.

### Files to Reference

| File                                                                                                     | Purpose                                                                                                                          |
| -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`                                             | Owns region collection, LCM grid construction, host/local width projection, and the remaining local stretch helper               |
| `src-tauri/src/instructions/draw/draw_tuplet.rs`                                                         | Maps local tuplet spacing x-positions into host coordinates for bracket bounds and hit targets                                   |
| `src-tauri/src/instructions/mod.rs`                                                                      | Builds anchored render contexts, stretches cloned local spacing for drawing, and uses spacing x-positions for notehead placement |
| `src-tauri/src/instructions/parse/get_stem_lengths.rs`                                                   | Uses resolved `Slot::NoteSpacing` x-values when computing stem geometry                                                          |
| `src-tauri/src/instructions/parse/get_written_durations.rs`                                              | Defines host-span vs local continuous tuplet timing semantics that proportional spacing must preserve                            |
| `_bmad-output/implementation-artifacts/3-4-lay-out-tuplets-legibly-within-the-existing-score-surface.md` | Existing story artifact that explicitly encodes even distribution and now needs superseding guidance                             |
| `docs/project-context.md`                                                                                | Repo guardrails for slots, backend ownership, performance, and tuplet timing invariants                                          |

### Technical Decisions

- Replace isolated host/local tuplet stretching with a shared region-resolution pass derived through LCM-based subdivision.
- Keep ownership in the Rust spacing seam; draw instructions and hit targets should consume the updated spacing result rather than recomputing ratios locally.
- Treat downstream renderer and hit-target behavior as validation surfaces, not new sources of spacing truth.
- Preserve existing MVP tuplet scope and avoid broadening ratio or nesting support as part of this change.
- Limit the algorithm change to `Slot::NoteSpacing` reconciliation; do not redesign unrelated slots or note-spacing rules.
- Preserve cumulative `HorizontalSpacing` x-position recomputation after any proportional width update so all downstream geometry continues to derive from one consistent map.
- Keep host geometry compressed to the tuplet denominator span and do not let Gould proportional spacing leak into a redefinition of canonical duration or tuplet ownership.
- Restate the rule in repo-owned language so implementation does not require access to external engraving text: simultaneous rhythmic material must resolve through one shared spacing truth rather than independent local stretching.
- Define the region rule explicitly: collect every anchored layout participant that has a realized layout, sort by host-span start/end, and merge any participants whose host spans overlap into one `AnchoredSpacingRegion`.
- Define the resolution rule explicitly: for a region of length `R` host ticks, compute `units_per_host_tick` as the LCM of each participant's `local_tick_count / gcd(local_tick_count, host_length)`; the temporary grid resolution is `R * units_per_host_tick`.
- Define the demand rule explicitly: seed the region grid with the host track's existing `Slot::NoteSpacing` widths across the region, projected evenly across each host tick's micro-units, then project each anchored layout's resolved `Slot::NoteSpacing` widths across its owned subrange of the same grid.
- Define the merge rule explicitly: each micro-unit keeps the maximum spacing demand contributed by any participant or host slice that covers it.
- Define the collapse rule explicitly: once the region grid is resolved, sum the micro-units back into host tick widths for the region and separately sum each participant's owned micro-units back into that anchored layout's local tick widths.
- Keep the temporary grid implementation-based rather than purely analytical: it is valid to materialize one micro-width per LCM unit for the bounded region so long as the work remains inside the backend spacing seam and downstream consumers still read ordinary spacing maps.

## Implementation Plan

### Tasks

- [x] Task 1: Replace isolated even-distribution helpers with a shared region-grid model
  - File: `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
  - Action: Introduce shared helpers to collect overlapping anchored regions, build one LCM-resolution grid per region, and rewrite both host and anchored local spacing from that resolved grid.
  - Notes: The live implementation uses `AnchoredSpacingRegion`, `SharedRhythmGrid`, `collect_anchored_spacing_regions()`, `build_shared_region_grid()`, and `finalize_shared_anchored_layouts()`.

- [x] Task 2: Resolve host spacing from the shared region grid without violating the compressed host geometry invariant
  - File: `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
  - Action: Seed the temporary region grid with the ambient host `Slot::NoteSpacing` widths, then collapse the resolved grid back into the host ticks across the region.
  - Notes: Host geometry remains compressed to the tuplet denominator span because only host tick widths are rewritten; canonical timing semantics remain unchanged.

- [x] Task 3: Apply the same resolved spacing truth to anchored local layouts used by rendering
  - File: `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
  - File: `src-tauri/src/instructions/mod.rs`
  - Action: Rewrite each anchored layout's local `Slot::NoteSpacing` widths from the same resolved region grid, and keep the render-time local stretch helper aligned with resolved local spacing when a cloned layout must match a wider host width.
  - Notes: The runtime helper `stretch_spacing_to_width_using_local_spacing()` preserves already-resolved local proportions rather than inventing a second distribution rule.

- [x] Task 4: Preserve downstream tuplet drawing and hit-target behavior against the new spacing map
  - File: `src-tauri/src/instructions/draw/draw_tuplet.rs`
  - Action: Keep runtime consumer logic anchored to resolved `HorizontalSpacing` x-positions and update tests that previously assumed uniform local spacing.
  - Notes: Downstream draw behavior remains a consumer seam, not a new layout owner.

- [x] Task 5: Replace obsolete even-distribution regression coverage with shared-grid expectations
  - File: `src-tauri/src/instructions/parse/get_horizontal_spacing.rs`
  - File: `src-tauri/src/instructions/draw/draw_tuplet.rs`
  - Action: Replace the old helper coverage with tests for shared-grid projection, LCM resolution, exact-span alignment, partial-overlap alignment, preserved local-spacing stretch behavior, and the existing consecutive-tuplet boundary regression.
  - Notes: The new seam coverage lives primarily in `get_horizontal_spacing.rs`, with `draw_tuplet.rs` retaining downstream bracket alignment coverage.

- [x] Task 6: Update the shipped story and developer-facing rule references for the new spacing contract
  - File: `_bmad-output/implementation-artifacts/3-4-lay-out-tuplets-legibly-within-the-existing-score-surface.md`
  - File: `docs/project-context.md`
  - Action: Do not silently rewrite historical delivery claims. Add an explicit supersession or follow-up note to the Story 3.4 artifact and keep `docs/project-context.md` aligned with the durable backend-owned shared-grid rule.
  - Notes: The implementation record must make it clear whether the historical story text was annotated, superseded, or intentionally left unchanged.

### Acceptance Criteria

- [x] AC 1: Given multiple anchored layouts whose host spans overlap, when horizontal spacing is finalized, then those participants resolve through one shared temporary rhythm grid instead of stretching independently.
- [x] AC 2: Given participants with different local tick counts over different host lengths, when the region grid is built, then the resolution uses an LCM-derived `units_per_host_tick` large enough to project each participant exactly onto the same grid.
- [x] AC 3: Given ambient host spacing already present in the affected region, when the region grid is built, then that host-owned `Slot::NoteSpacing` demand participates in the same negotiation as the anchored layouts rather than being replaced by tuplet-only math.
- [x] AC 4: Given the region grid has been resolved, when widths are written back, then the host track receives ordinary host tick widths and each anchored layout receives ordinary local tick widths; the shared high-resolution grid remains transient.
- [x] AC 5: Given a cloned anchored layout must stretch to a wider resolved host width at render time, when local stretching is applied, then the helper preserves the resolved local spacing shape instead of reintroducing even per-tick stretching.
- [x] AC 6: Given proportional/shared tuplet spacing has been applied, when tuplets are drawn and hit targets are produced, then bracket bounds, notehead positions, stem geometry, and tuplet hit targets remain aligned because downstream consumers continue to read the recomputed spacing x-map rather than recomputing spacing locally.
- [x] AC 7: Given consecutive tuplets, opening accidentals, or other previously covered tuplet-boundary cases, when the shared-grid rule lands, then the existing boundary-spacing protections remain intact.
- [x] AC 8: Given the spacing contract now differs from the prior Story 3.4 “even distribution” behavior, when the work is documented, then the updated artifact and developer-facing rules describe the shared region-grid contract explicitly.

## Additional Context

### Dependencies

- Existing tuplet parser and spacing contracts in the Rust render pipeline
- Existing Story 3.4 layout behavior, which needed to be superseded where it previously specified even distribution
- Existing draw and hit-target seams that already consume resolved spacing without owning the distribution rule
- Existing written-duration and host-span semantics in `get_written_durations.rs`, which remain the authoritative source of tuplet-local versus host-compressed timing
- Existing performance guardrail that render-sensitive changes should not casually degrade the representative single-flow baseline
- The repo-owned restatement of the shared-grid rule in this spec, which removes any requirement for the implementer to infer behavior from superseded intermediate designs

### Testing Strategy

- Start with focused Rust regressions around `get_horizontal_spacing.rs`.
- Replace the old even-distribution helper tests with shared-grid projection expectations.
- Add immediate downstream coverage only where proportional spacing changes bracket placement, drawing stability, or hit targets.
- Keep validation at the owning seam whenever local coverage is sufficient.
- Treat `draw_tuplet.rs` tests as the first downstream validation seam because they already exercise stretched anchored spacing and bracket bounds.
- Cover one exact shared-span case, one partially overlapping-span case, one local-spacing preservation case, and one boundary-adjacent scenario to prove the new region pass and the old seam protections both hold.
- Name the current live regressions so future maintainers can find them quickly: shared-grid resolution tests, exact-span alignment, partial-overlap alignment, local-spacing stretch preservation, downstream bracket alignment, and `consecutive_tuplets_use_next_tuplet_opening_as_forward_successor`.
- Run the standard Rust validation set after implementation: focused spacing tests first, then `cargo test --manifest-path src-tauri/Cargo.toml`, then `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### Notes

- The implemented rule change is a contract change inside the current spacing system, not a new persisted layout subsystem.
- The existing Story 3.4 artifact likely needs to be treated as prior behavior that this spec will explicitly replace.
- Based on the final implementation, the main algorithm seam is still narrower than the surface area of affected rendering: the spacing pass owns region negotiation, while drawing, stems, noteheads, and hit targets consume the resulting x-map.
- The spec intentionally keeps the shared region-grid rule inside the existing backend-owned `HorizontalSpacing` model so there is no second tuplet geometry truth in React or in renderer-side consumers.
- High-risk item: if future work regresses to exact host-span grouping instead of overlapping-region grouping, simultaneous rhythms can drift again in partial-overlap cases.
- High-risk item: if future work lets host spacing and anchored local spacing re-derive different distributions after region resolution, the score can drift into mismatched notehead, bracket, and hit-target positions even though each local function appears correct.
- Future consideration outside this scope: if later stories broaden supported tuplet patterns or region participants, they should extend the shared region negotiation rather than reintroducing tuplet-specific special cases.
