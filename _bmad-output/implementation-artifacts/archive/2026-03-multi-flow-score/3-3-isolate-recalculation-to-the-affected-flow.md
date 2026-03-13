# Story 3.3: Isolate Recalculation to the Affected Flow

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want local score edits to avoid unnecessary recalculation in other visible flows,
so that multi-flow score view stays responsive and predictable as the rendered document grows.

This story narrows Epic 3 from observability and regression safety into actual invalidation behavior. The current backend can already tell maintainers which visible flows were prepared and composed, but local edits still enter the render path through a boolean full-recalculate flag and `FlowRenderPlan` currently reports every visible flow as recalculated. Story 3.3 should replace that broad assumption with a truthful, backend-owned flow-scoped recalculation path for local edits, while preserving the shared-surface render model and notation behavior.

## Acceptance Criteria

1. Given multiple flows are visible in score view, when a local edit affects only one flow, then notation derivation and related recalculation are scoped to the affected flow where possible and unrelated visible flows do not re-derive unnecessarily.
2. Given instrumentation is available for multi-flow recalculation, when a local edit is exercised in one visible flow, then the diagnostic output shows the recalculation boundary clearly and maintainers can verify that unrelated flows were not included without cause.
3. Given a single-flow score, when the updated recalculation path runs, then current behavior remains correct and this change does not introduce regressions in existing render or notation behavior.
4. Given this isolation increment is validated, when future multi-flow interactions build on it, then they inherit a narrower recalculation scope and no persisted score semantic change is required.

## Tasks / Subtasks

- [x] Audit the existing full-pass recalculation path and identify where a mutation can already name its affected flow or track (AC: 1, 2, 3, 4)
  - [x] Confirm how `AppContext::emit(..., recalculate: bool)` and `AppContext::render(...)` in `src-tauri/src/lib.rs` currently force a full visible-flow recomputation for score mutations.
  - [x] Identify which score mutation commands already carry enough locality to infer an affected flow directly, including flow-owned commands in `src-tauri/src/score/entries/tempo.rs`, `src-tauri/src/score/entries/barline.rs`, and `src-tauri/src/score/entries/time_signature.rs` plus track-owned tone commands in `src-tauri/src/score/entries/tone.rs`.
  - [x] Separate truly global mutations from local flow edits so the story narrows recalculation only where the change is actually flow-local.
- [x] Introduce a backend-owned recalculation scope model that can represent local and broad passes truthfully (AC: 1, 2, 3)
  - [x] Replace or extend the current boolean recalculate flag with an explicit render-request or invalidation payload that can distinguish a full visible-flow pass from an affected-flow-only pass.
  - [x] Keep this scope Rust-owned and aligned with the existing `FlowRenderPreparationDiagnostics` contract in `src-tauri/src/instructions/render_plan.rs`; do not invent a frontend-owned scope model.
  - [x] Preserve current single-flow and no-visible-flow behavior, including null-safe reset diagnostics.
- [x] Restrict parse and draw work to the affected flow where the mutation is flow-local (AC: 1, 3, 4)
  - [x] Update the render plan and instruction-generation seam so local edits recalculate only the targeted visible flow while still composing the shared surface in `score.flows.order`.
  - [x] Reuse previously valid non-edited flow outputs where possible instead of re-deriving their notation data on every local edit.
  - [x] Preserve hit-target correctness, active-flow behavior, and existing shared-surface layout rules.
- [x] Extend diagnostics so maintainers can verify the narrowed recalculation boundary without inference (AC: 2, 3)
  - [x] Ensure `render-timing-flow-preparation-details` continues to report prepared, composed, and recalculated flow ids/counts/scopes, but now reflects the real narrowed recalculation set for local edits.
  - [x] Keep `render-timing-surface-details` and the debugger panel contract stable unless a small extension is necessary to expose reused versus recalculated flow segments clearly.
  - [x] Preserve reset/no-render behavior and avoid ad hoc console logging or temporary instrumentation.
- [x] Add focused regression coverage around flow-scoped invalidation and composition reuse (AC: 1, 2, 3, 4)
  - [x] Add Rust tests around `src-tauri/src/instructions/render_plan.rs` and related render seams that prove local edits can report a single recalculated flow while multi-flow composition remains ordered and correct.
  - [x] Add or extend tests in `src-tauri/src/instructions/mod.rs` or nearby render seams to prove unaffected flows keep their existing composed output and hit targets when a different visible flow is recalculated.
  - [x] Extend debugger-facing frontend tests only if backend payload shape or display rules change materially.
- [x] Validate the narrowed recalculation path end to end (AC: 1, 2, 3, 4)
  - [x] Run `npm run lint`.
  - [x] Run `npm run check-types`.
  - [x] Run `npm run test:rust`.
  - [x] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

## Dev Notes

### Story Foundation

- Epic 3 is about preserving notation trust while making multi-flow rendering faster, more isolated, and easier to evolve safely.
- Story 3.1 made render and recalculation scope visible. Story 3.2 added notation-safety regression protection. Story 3.3 is the first story that should actually narrow recalculation behavior rather than only observe or protect it.
- This story must keep multi-flow work as a presentation/orchestration change over existing flow data. It is not permission to rewrite notation logic, score persistence, or frontend state ownership.

### Developer Context Summary

- `src-tauri/src/lib.rs` currently enters the render path through `AppContext::emit(..., recalculate: bool)` and `AppContext::render(...)`; when `recalculate` is `true`, the backend prepares a flow render plan and calls `generate_render_instructions(...)` for the whole visible shared surface.
- `src-tauri/src/instructions/render_plan.rs` currently treats `recalculated_flow_ids()` as equivalent to `composed_flow_ids()`. That was truthful for Story 3.1 because the renderer still recalculated every visible flow, but it is the core seam that Story 3.3 now needs to refine.
- Most score mutations still call `state.emit(&app, &mut renderer, true)` even when the underlying command is local to a single flow or track. Commands such as `score_entries_tempo_create/update/remove`, `score_entries_barline_*`, and `score_entries_time_signature_*` already receive a `flow_id`. Tone mutations in `src-tauri/src/score/entries/tone.rs` currently receive `track_id`, so this story likely needs a safe backend mapping from track-local edits to the owning flow.
- `src-tauri/src/instructions/mod.rs` already composes independently rendered flow segments into one shared surface and preserves combined hit targets in renderer space. That seam should remain authoritative; the goal is to avoid re-deriving unaffected segments, not to reintroduce separate per-flow frontends or break shared-surface composition.
- `src/windows/debugger/components/performance-timings.tsx` already renders flow-preparation diagnostics and shared-surface details. Story 3.3 should keep maintainers using that existing debugger surface rather than inventing another observability path.

### Technical Requirements

- Introduce an explicit backend invalidation/recalculation scope that can distinguish:
  - a full visible-flow pass for global mutations
  - a single affected-flow pass for local edits
  - existing single-flow and no-visible-flow cases
- Keep recalculation scope truthful. Do not claim a narrowed recalculation boundary unless the backend truly avoided re-deriving unrelated visible flows.
- Preserve the existing shared-surface render model in `score.flows.order`; only the recalculation boundary should narrow.
- Preserve written-duration behavior, notation semantics, hit-target correctness, selection continuity, active-flow semantics, and current document behavior.
- Prefer reusing existing rendered/derived outputs for unaffected flows over inventing a second source of truth or frontend cache.
- Global edits that genuinely affect all visible flows may still broaden recalculation. This story is about isolation where possible, not about forcing single-flow treatment for every mutation.
- No persistence, migration, or `.soloc` semantic change is expected. Do not touch `src-tauri/src/migrations/next.rs` unless implementation discovery proves older files would break, which would contradict the planned story boundary.

### Architecture Compliance

- Rust remains the single source of truth for score state, render orchestration, and diagnostics payload generation.
- The native score surface remains the only notation render path; React continues to act as the overlay and debugger shell.
- `ui.flow_id` remains the single authoritative active-flow slot. Do not introduce a parallel frontend-owned active-flow or recalculation-selection state.
- Flow-local invalidation should reuse the existing backend command/event boundary. Do not move recalculation decisions into the frontend and do not derive scope from debugger UI state.
- The implementation should extend existing modules before creating new top-level architecture. The likely seams are `src-tauri/src/lib.rs`, `src-tauri/src/instructions/render_plan.rs`, `src-tauri/src/instructions/mod.rs`, and the score command modules that currently emit broad recalculations.

### Library / Framework Requirements

- Stay within the repo-pinned stack documented in `docs/project-context.md`: React 19.2.4, TypeScript 5.9.3, Vite 7.3.1, Tauri 2.10.1, and Rust 2021.
- Keep frontend/backend payload fields in `snake_case` and preserve typed alignment between Rust diagnostics structs and TypeScript debugger consumers.
- Preserve current performance-oriented backend choices such as `FxHashMap`, the native Rust/Skia/Metal render path, and the existing command/event model.
- No dependency upgrade is expected for this story.
- External web research was not available in this environment; repository docs and source are the authoritative baseline.

### Project Structure Notes

- Primary invalidation and render-entry seams:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/instructions/render_plan.rs`
  - `src-tauri/src/instructions/mod.rs`
- Likely score mutation seams that should stop defaulting to broad recalculation when the change is flow-local:
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/score/entries/tempo.rs`
  - `src-tauri/src/score/entries/barline.rs`
  - `src-tauri/src/score/entries/time_signature.rs`
  - `src-tauri/src/score/entries/key_signature.rs`
  - `src-tauri/src/score/entries/clef.rs`
- Existing debugger seams for diagnostics validation:
  - `src/windows/debugger/components/performance-timings.tsx`
  - `src/windows/debugger/components/performance-timings.test.tsx`
- Supporting docs that already describe the intended render contract:
  - `docs/project-context.md`
  - `docs/architecture.md`

### File Structure Requirements

- Extend existing render and score-command modules before adding any new top-level files.
- If a new render-request or invalidation type is introduced, keep it close to the current render-entry and render-plan code instead of creating a disconnected feature layer.
- Keep debugger changes limited to the existing performance timing component/test seam if payload shape changes.
- Do not create a frontend cache or feature-local state store for recalculation scope.

### Testing Requirements

- Add Rust tests that prove a local visible-flow edit reports only the affected flow in `recalculated_flow_ids` while preserving the full prepared/composed visible set.
- Add Rust tests that prove single-flow passes still report correct recalculation scope and that no-visible-flow reset behavior remains null-safe.
- Add or extend composition tests to show non-edited flows retain correct draw instructions and hit targets when another visible flow is recalculated.
- Extend debugger UI tests only if the displayed diagnostics contract changes.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 3.1 explicitly warned that recalculation scope was still truthful only because the backend recomputed the unified visible-flow pass. Story 3.3 should change that backend behavior instead of papering over it in diagnostics.
- Story 3.1 also established the maintainer-facing debugger contract for prepared/composed/recalculated flow visibility. Keep that contract Rust-owned and incremental.
- Story 3.2 established the notation-safety guardrail. Story 3.3 should rely on that safety net while changing invalidation behavior, and it should not introduce a new parser framework or notation reinterpretation.
- Recent Epic 2 and 3 work consistently extended backend-owned contracts and paired them with focused regression tests. Follow that same pattern here.

### Git Intelligence Summary

- `700e7dda test(3.2): add multi-flow notation safety regressions` confirms the latest completed story strengthened backend-owned safety seams rather than adding UI-owned approximations.
- `b091f8ff feat(2.1): flow rendering debug info` reinforces the repo pattern of using debugger-facing diagnostics to make backend render behavior inspectable.
- `61d9a08c chore(3.1): prep` indicates Epic 3 has been staged deliberately, with observability first and optimization next.
- `bca5d7e9 chore: epic 2 retro` confirms the interaction epic is closed, so this story should focus on render isolation rather than reopening Epic 2 behavior.
- `4d0dadb8 feat(2.5): preserve active flow across score and sequence` reinforces the project rule that shared behavior should keep a single backend-owned source of truth rather than parallel state.

### Project Context Reference

- `docs/project-context.md` defines the render pipeline, the shared-surface composition model, and the rule that recalculation diagnostics remain Rust-owned.
- `docs/architecture.md` states that recalculation only occurs on score mutations today and that later Epic 3 work should narrow that boundary without changing the broader shared-surface model.
- `_bmad-output/planning-artifacts/epics.md` defines Story 3.3 as flow-local recalculation isolation with no persisted semantic change.
- `_bmad-output/implementation-artifacts/3-1-add-multi-flow-render-and-recalculation-visibility.md` explicitly records that recalculation scope is currently broad and that later invalidation work must narrow it truthfully.
- `_bmad-output/implementation-artifacts/3-2-add-regression-protection-for-multi-flow-notation-safety.md` provides the notation-safety guardrails this story should preserve while optimizing invalidation.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 3, Story 3.3 acceptance criteria)
- Source: `docs/project-context.md` (render pipeline, shared-surface composition, diagnostics ownership, stack versions)
- Source: `docs/architecture.md` (caching and recalculation expectations)
- Source: `_bmad-output/implementation-artifacts/3-1-add-multi-flow-render-and-recalculation-visibility.md` (current diagnostics boundary and truthful reporting constraint)
- Source: `_bmad-output/implementation-artifacts/3-2-add-regression-protection-for-multi-flow-notation-safety.md` (notation-safety guardrails)
- Source: `src-tauri/src/lib.rs` (render entry point and recalculate flag)
- Source: `src-tauri/src/instructions/render_plan.rs` (prepared/composed/recalculated flow reporting)
- Source: `src-tauri/src/instructions/mod.rs` (shared-surface composition and hit-target composition)
- Source: `src-tauri/src/score/entries/tone.rs` (track-local edits currently triggering broad recalculation)
- Source: `src-tauri/src/score/entries/tempo.rs` (flow-local edits currently triggering broad recalculation)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Current broad recalculation entry point: `src-tauri/src/lib.rs`
- Current render-scope diagnostics seam: `src-tauri/src/instructions/render_plan.rs`
- Current shared-surface composition seam: `src-tauri/src/instructions/mod.rs`
- Existing debugger consumer: `src/windows/debugger/components/performance-timings.tsx`

### Implementation Plan

- Extend the backend render entry point with an explicit recalculation request so score mutations can choose full-pass or affected-flow scope without moving that decision into the frontend.
- Teach the render plan and diagnostics to distinguish prepared/composed visible flows from the narrower set actually recalculated in a pass.
- Reuse cached non-edited flow segments during shared-surface composition and broaden back to a full pass automatically when no compatible cache is available.
- Route flow-owned entry commands directly to flow-scoped invalidation and map track-owned tone edits back to their owning flow before emitting.

### Completion Notes List

- Story context created from Epic 3.3 planning artifacts, current render pipeline source, Story 3.1 observability notes, Story 3.2 notation guardrails, and recent git history.
- The repo does not contain `_bmad/core/tasks/validate-workflow.xml`, so checklist validation could not be executed through the expected BMAD validator task path; the story was validated manually against the create-story checklist and workflow requirements.
- Added `FlowRenderRecalculationRequest` and updated `AppContext` render/emit helpers so local score edits can request a flow-scoped pass while existing global mutations still use the full visible-flow path.
- Updated `FlowRenderPlan` diagnostics to report the actual recalculated flow set, then added renderer-side segment caching and automatic broadening when reuse is unavailable so diagnostics remain truthful.
- Routed tone, tempo, barline, time signature, key signature, and clef mutations through flow-scoped invalidation and added regression tests covering targeted recalculation, hidden-flow no-op scope, cache fallback, and reuse of unaffected hit targets and instructions.
- Follow-up review fixes keep hidden-flow no-op requests empty even when no reusable segment cache exists and remove the remaining frontend-supplied `flow_id` from step-wise tone pitch shifting so the backend derives flow ownership from `track_id` consistently.
- Review validation re-ran focused frontend tests plus `npm run lint`, `npm run check-types`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` after the fixes landed.
- Validation passed with `npm run lint`, `npm run check-types`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### File List

- `_bmad-output/implementation-artifacts/3-3-isolate-recalculation-to-the-affected-flow.md`
- `_bmad-output/implementation-artifacts/3-4-keep-non-edited-flows-visually-stable-during-local-updates.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/services/actions.test.ts`
- `src/services/actions.ts`
- `src-tauri/src/instructions/mod.rs`
- `src-tauri/src/instructions/parse/get_written_durations.rs`
- `src-tauri/src/instructions/render_plan.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/renderer/mod.rs`
- `src-tauri/src/score/entries/barline.rs`
- `src-tauri/src/score/entries/clef.rs`
- `src-tauri/src/score/entries/key_signature.rs`
- `src-tauri/src/score/entries/tempo.rs`
- `src-tauri/src/score/entries/time_signature.rs`
- `src-tauri/src/score/entries/tone.rs`
- `src/windows/main/hooks/use-keyboard-shortcuts.test.ts`
- `src/windows/main/hooks/use-keyboard-shortcuts.ts`

## Senior Developer Review (AI)

### Reviewer

Aled

### Date

2026-03-11

### Outcome

Approved

### Summary

- Fixed the hidden-target regression so cache absence no longer broadens an empty recalculation scope into a full visible-flow pass.
- Removed the remaining frontend-owned `flow_id` input from `score_entries_tone_pitch_shift_by_steps` and now derive the owning flow from backend track ownership before reading key signatures or emitting recalculation.
- Reconciled the story record with the actual changed-file set and captured the review-driven validation rerun.

## Change Log

- 2026-03-11: Added backend-owned flow-scoped recalculation requests, truthful recalculation diagnostics, shared-surface segment reuse, and regression coverage for local multi-flow score edits.
- 2026-03-11: Resolved code-review findings by preserving hidden-flow no-op scope without cache reuse, removing the remaining frontend-supplied flow scope for tone step shifting, and updating the story audit trail.
