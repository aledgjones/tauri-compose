# Story 1.5: Add Flow Labels to Multi-Flow Score View

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want each visible flow to show a small identifying label,
so that I can tell which flow I am reading without adding more visual noise or changing interaction behavior.

## Acceptance Criteria

1. Given two flows are rendered in the same score surface, when the score view is displayed, then each visible flow page shows a small label at its top-left corner, and the label includes the flow number and flow name.
2. Given visible flow labels are shown, when a user reads across the multi-flow score surface, then the labels help identify visible flows and their order, and no additional boundary treatment is added beyond the existing page box and inter-flow gap.
3. Given multiple flows are visible at compact desktop width with secondary panels collapsed, when the score remains the primary work surface, then each visible flow label remains visible and legible within the reduced workspace, and the shell preserves the same score-first layout structure without introducing a different navigation model.
4. Given visible flow labels are present, when the score view is rendered, then the label styling uses the product accent treatment in a restrained way, and the labels do not interfere with notation readability or obscure score content.
5. Given only one flow is visible, when the score view renders, then the label treatment does not degrade the single-flow reading experience, and the score remains visually clean.
6. Given a score element is selected, when selection context is shown in the bottom bar, then the selection display includes the selected flow number and name, and this does not change how selection, active flow, or editing behavior works.

## Tasks / Subtasks

- [ ] Add passive per-flow labels to the shared multi-flow score surface without changing interaction behavior (AC: 1, 2, 3, 4, 5)
  - [ ] Audit the existing shared-surface composition in `src-tauri/src/instructions/mod.rs` to identify the safest place to attach a small top-left label to each visible page box after page offsets are known.
  - [ ] Reuse existing flow metadata from the score model so each label shows the visible flow number based on `score.flows.order` plus the current flow title.
  - [ ] Keep the label as native score-surface chrome rendered by Rust alongside the page box, not as a React overlay or separate shell-level badge.
- [ ] Keep the label treatment visually restrained and notation-safe (AC: 3, 4, 5)
  - [ ] Render the label as a small accent-background chip positioned at the top-left corner of each visible flow page.
  - [ ] Ensure the chip remains readable without obscuring notation, colliding with existing page-start content, or adding new boundary markers beyond the current page box and inter-flow gap.
  - [ ] Preserve the current single-flow reading experience by keeping the label treatment minimal and confirming it does not make the score surface feel noisy.
- [ ] Extend the existing bottom-bar selection summary with flow context (AC: 6)
  - [ ] Audit `src/windows/main/components/selected-tone-info-display.tsx` and its supporting hooks to add the selected flow number and flow name to the existing selection summary text.
  - [ ] Reuse current sequencer selection state and `state.score.flows.by_id` data rather than introducing a separate active-flow or renderer-owned UI concept.
  - [ ] Keep this change limited to selection context presentation; do not change hit-testing, selection routing, or editing semantics.
- [ ] Add focused regression coverage and verification for passive flow labeling (AC: 1, 2, 3, 4, 5, 6)
  - [ ] Add Rust coverage near the multi-flow composition seam to verify labels are emitted for visible composed flows in order and do not add extra boundary treatment.
  - [ ] Add or update frontend tests for the bottom-bar selection summary so selected flow number and name appear when selection context is available.
  - [ ] Validate visually with one-flow and two-flow scores at normal working zoom and compact desktop width to confirm labels remain legible and non-intrusive.
- [ ] Update docs only if flow labels become part of the stable renderer contract
  - [ ] Update `docs/architecture.md` and `docs/project-context.md` only if the label-chip treatment or bottom-bar selection context becomes a documented long-term implementation rule.

## Dev Notes

### Story Foundation

- Story 1.2 established the first shared native score surface for two visible flows, Story 1.3 kept the composed width reachable through the existing pan path, and Story 1.4 refined the surface into separate white page boxes with a clear inter-flow gap.
- Story 1.5 should stay narrowly focused on passive orientation: a small flow label chip on each visible page plus flow context in the bottom bar when score selection context is shown.
- The story should not add more boundary treatment, should not highlight active flow in the renderer, and should not change hit-testing, selection routing, playback scope, or editing semantics.
- The practical definition of done is: the shared multi-flow page surface gains a restrained top-left label chip per visible page, the chip reflects visible flow order plus flow title, the bottom bar shows selected flow number and name as part of score selection context, and the single-flow experience remains clean.

### Developer Context Summary

- `compose_render_segments(...)` in `src-tauri/src/instructions/mod.rs` already owns per-page background rectangles and page offsets for each visible composed flow. That is the safest seam for attaching passive page-level label chrome after the final page positions are known.
- The render plan in `src-tauri/src/instructions/render_plan.rs` already preserves visible flow order from `score.flows.order`. Reuse that order for label numbering rather than inventing a new visible-flow index model in React.
- The current React score surface in `src/windows/main/components/renderer.tsx` is only a transparent interaction overlay. Do not place page labels there; renderer chrome belongs in the native Rust draw pipeline.
- `src-tauri/src/instructions/draw/draw_names.rs` demonstrates the existing text-measurement and text-drawing helpers used for native score-adjacent text. Reuse those font and measurement primitives where practical for a small chip label instead of introducing a separate frontend text system.
- The bottom bar in both `src/windows/main/screens/write/index.tsx` and `src/windows/main/screens/sequence/index.tsx` currently renders `SelectedToneInfoDisplay`, which is backed by `useSelectedToneInfo()` and only returns information for Tone and Rest selections. To satisfy AC6, the implementation likely needs to broaden this into a more general score-selection summary rather than keeping it tone/rest only.

### Previous Story Intelligence

- Reuse the Story 1.4 page-per-flow composition seam instead of inventing a new overlay, tab strip, or per-flow container.
- Treat the existing page box and inter-flow gap as the complete boundary solution. Story 1.5 should identify flows, not add more separators.
- Preserve the Story 1.3 shared-surface pan model and debugger verification path; the label chip should travel with the native page surface and should not introduce a separate layout or scroll coordinate system.
- Keep all interaction semantics where they are today. Epic 2 still owns cross-flow interaction behavior, active-flow communication, and edit-target clarity.
- Flow titles already exist in frontend state as `Flow.title`, so the story can use real document flow names rather than fallback-only numbering.

### Technical Requirements

- Keep Rust authoritative for score-surface chrome rendered on the page itself. The flow label chip should be emitted as native draw instructions alongside the page box.
- Derive label text from existing score data: visible flow number based on `score.flows.order` plus the flow title from `score.flows.by_id[flow_id].title`.
- Keep the label visually anchored to the top-left corner of each visible page box rather than to viewport coordinates or shell layout.
- Keep the chip small and restrained: accent-color background, readable text, and no collision with notation or existing page-start chrome.
- Do not add new horizontal spacing slots for the label chip. This is page chrome, not notation flow content, so it should align to the page box rather than push notation horizontally.
- Extend the bottom-bar selection summary using existing selection state and flow metadata. Do not introduce a separate renderer-owned concept of current flow.
- The safest AC6 interpretation is to support score-element selections that already participate in the bottom-bar summary path today and broaden that summary if needed so tempo, time signature, key signature, and clef selections can include flow context without changing their underlying selection semantics.
- No persistence or migration change is expected for this story.

### Architecture Compliance

- Multi-flow support remains a derived presentation layer over existing flow data. The label chip is page-level presentation chrome, not a persisted score-structure change.
- The score must continue to render through Rust/Skia/Metal. Do not implement page labels in React/HTML.
- Preserve the existing frontend/backend mutation boundary. The frontend may display flow context from state already pushed by Rust, but it should not become the source of truth for visible-flow ordering or page label ownership.
- Active-flow semantics remain distinct from visible-flow identification. This story should not expose or reinforce the internal active-flow mechanic in the renderer itself.
- Future page-layout evolution should remain possible. Keep the label-chip logic tied to composed page metadata so it can move with later page/frame work rather than assuming today’s two-flow fixed layout is permanent.

### Library / Framework Requirements

- Stay within the existing stack already in the repo: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, Rust 2021, `skia-safe` 0.93.x, `rayon` 1.10, and `rustc-hash`.
- Preserve snake_case across the frontend/backend boundary and keep any new diagnostics payloads typed and minimal if they are needed at all.
- Reuse existing renderer text helpers, tokenized text display, and selection hooks before creating new abstractions.
- No dependency upgrade or external library change is expected for this story.

### File Structure Requirements

- Primary backend seams:
  - `src-tauri/src/instructions/mod.rs`
  - `src-tauri/src/instructions/render_plan.rs`
  - `src-tauri/src/instructions/draw/draw_names.rs`
- Primary frontend seams:
  - `src/windows/main/components/selected-tone-info-display.tsx`
  - `src/windows/main/hooks/use-selected-tone-info.ts`
  - `src/windows/main/screens/write/index.tsx`
  - `src/windows/main/screens/sequence/index.tsx`
- Likely supporting frontend selection hooks if AC6 is broadened beyond tone/rest:
  - `src/windows/main/hooks/use-selected-tempo.ts`
  - `src/windows/main/hooks/use-selected-time-signature.ts`
  - `src/windows/main/hooks/use-selected-key-signature.ts`
  - `src/windows/main/hooks/use-selected-clef.ts`
- Supporting tests likely needed:
  - `src/windows/main/components/selected-tone-info-display.test.tsx`
- Likely out of scope unless implementation proves otherwise:
  - hit-testing services
  - renderer overlay event handling
  - playback transport
  - persistence and migrations

### Testing Requirements

- Add Rust tests near the shared-surface composition seam to verify label chips are emitted for visible composed flows in score order and that no extra page-boundary treatment is introduced.
- Add or update frontend tests for the bottom-bar selection summary so selected flow number and name appear when score selection context is available.
- Validate visually with one-flow and two-flow scores at normal working zoom and compact desktop width to confirm the chip remains legible, restrained, and non-intrusive.
- Validation target:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Git Intelligence Summary

- The most relevant recent implementation commits are `fc46ccdf feat(1.4): refine adjacent flow score coherence`, `1ee5350a feat(1.4): add per-flow page backgrounds`, `022d3573 feat(1.3): renderer scrolling`, and `b2f592b5 feat: render two flows on a shared score surface`.
- Those commits establish the exact seams this story should extend: page-per-flow native composition, shared-surface scrolling, and ordered visible-flow preparation.
- No recent commit suggests moving renderer chrome into React, so the strongest implementation precedent remains the native Rust draw/composition path.

### Project Context Reference

- `docs/project-context.md` requires the score surface to remain natively rendered in Rust/Skia/Metal, with React acting as the overlay and orchestration shell only.
- `docs/architecture.md` explicitly warns that horizontal spacing slots are for notation flow, not generic annotations. The flow label chip should therefore be page chrome aligned to existing page geometry, not a notation-spacing participant.
- The current shared-surface architecture already preserves ordered visible flows and page-per-flow composition. Story 1.5 should identify those pages more clearly without changing the shared-surface model.
- The bottom-bar summary already exists as a lightweight contextual UI. Extending that existing summary is preferable to inventing a new active-flow display concept.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 1, Story 1.5)
- Source: `_bmad-output/implementation-artifacts/1-3-scroll-across-the-combined-multi-flow-score-width.md` (shared-surface navigation and debugger precedent)
- Source: `_bmad-output/implementation-artifacts/1-4-make-adjacent-flows-read-as-one-coherent-score.md` (page-per-flow composition and inter-flow gap precedent)
- Source: `docs/architecture.md` (horizontal spacing rules, shared-surface composition expectations)
- Source: `docs/project-context.md` (native renderer ownership, FE/BE boundary, composed surface pipeline)
- Source: `src-tauri/src/instructions/mod.rs` (`compose_render_segments`, page backgrounds, shared-surface composition)
- Source: `src-tauri/src/instructions/render_plan.rs` (ordered visible-flow preparation)
- Source: `src-tauri/src/instructions/draw/draw_names.rs` (native text measurement/drawing helpers)
- Source: `src/windows/main/components/selected-tone-info-display.tsx` (existing bottom-bar selection summary)
- Source: `src/windows/main/hooks/use-selected-tone-info.ts` (current tone/rest selection summary hook)
- Source: `src/windows/main/hooks/use-selected-tempo.ts`, `use-selected-time-signature.ts`, `use-selected-key-signature.ts`, `use-selected-clef.ts` (current score-element selection hooks that may need unified flow-context display)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story closed on 2026-03-11 after live UX review rejected both proposed changes:
  - native per-page flow labels were considered visually noisy and not worth keeping on the score surface
  - expanded bottom-bar selection summaries were considered more confusing than helpful
- The implementation was fully reverted before merge; no renderer, frontend, or test changes from this attempt remain in the codebase.
- Closure rationale for future context:
  - do not reintroduce passive page labels by default; this story direction was explicitly rejected after seeing it in the product
  - do not broaden the bottom-bar selection summary with additional flow context unless a future story reframes the UX problem and validates a different presentation

### Completion Notes List

- Story scope refined interactively to avoid overdoing orientation treatment: no new boundary markers and no renderer-level active-flow highlight.
- Flow labels are intended as small top-left chips on each visible page using existing flow order and flow title metadata.
- Bottom-bar selection context is expected to surface flow number and name without changing selection semantics.
- No dependency upgrade, persistence change, or migration work is expected for this story.
- Closed without shipping after live review.
- Reverted the native flow-label implementation completely.
- Reverted the expanded bottom-bar selection-summary implementation completely.
- Kept the story artifact as historical context so future work can avoid repeating the same rejected UX direction.

### File List

- \_bmad-output/implementation-artifacts/1-5-add-flow-labels-to-multi-flow-score-view.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-03-11: story closed after live UX review rejected both page labels and expanded selection summaries; all implementation work from this attempt was reverted before merge.
