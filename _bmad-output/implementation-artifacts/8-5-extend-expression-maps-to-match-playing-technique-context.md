# Story 8.5: Extend Expression Maps To Match Playing-Technique Context

Status: done

## Story

As a composer,
I want expression maps to match derived playing-technique context,
so that reusable maps can translate score-level technique meaning into library-specific switching behavior.

## Acceptance Criteria

1. Given expression maps already match bounded note-time context such as articulation and written duration, when playing-technique semantics are added to the derived note context, then expression-map conditions can also match active playing techniques, and they reuse the existing multi-condition and first-match-wins rule model rather than inventing a separate region-specific map system.
2. Given the playing-technique proof of concept is built on derived exclusive-group semantics rather than raw score markings, when expression-map support is extended, then the Rust and TypeScript expression-map data structures, persistence contract, and editor authoring workflow are updated to support the new derived playing-technique condition family, and the first implementation remains bounded to the conditions needed by the `pizz` and `arco` proof of concept.
3. Given multiple playing techniques may be active together, when a rule is evaluated, then the condition system can combine technique matches with other context such as articulation or note length, and more specific combined rules can be ordered above broader fallback rules.
4. Given expression maps are the user-authored bridge between musical meaning and plugin behavior, when this condition family is introduced, then the map remains a translation layer over derived score meaning rather than becoming the owner of persistent score-technique state, and the design stays compatible with multiple libraries that need different switching behavior for the same marking or future exclusive technique group.

## Tasks / Subtasks

- [x] Task 1: Add one backend-owned playing-technique condition family to the reusable Expression Map model (AC: 1, 2, 4)
  - [x] Extend `ExpressionMapCondition` in Rust and TypeScript with a new stable-id-bearing condition variant that targets one bounded playing-technique value.
  - [x] Reuse the existing equality-comparator row model unless implementation proves a narrower type is required; the condition must express membership of one derived playing technique, not exact equality of the whole effective context.
  - [x] Thread the new condition through the existing create, set-type, and update command path in `src-tauri/src/playback/expression_maps.rs` and `src-tauri/src/lib.rs` rather than creating a parallel mutation API.
  - [x] Assess additive persistence impact for the app-scoped Expression Map library and keep `.soloc` route persistence unchanged because project files still store only `expression_map_id` references.

- [x] Task 2: Match derived playing-technique context in rule evaluation without duplicating score semantics (AC: 1, 3, 4)
  - [x] Extend `expression_map_condition_matches(...)` so technique conditions read `ExpressionMapPlaybackContext.effective_playing_techniques`.
  - [x] Keep rules logical-AND within one rule and top-down first-match-wins across rules, allowing technique conditions to combine with articulation and written-note-length rows.
  - [x] Match against prepared derived note-time meaning from Story 8.4, including no-implicit-`arco`, hidden-still-semantic behavior, and same-tick conflict resolution, without scanning raw `PlayingTechnique` entries inside expression-map matching.
  - [x] Keep the first condition vocabulary bounded to `pizz` and `arco`; do not widen into raw marking cancellation logic, score-region systems, or new playback topology.

- [x] Task 3: Extend the app-scoped Expression Map editor for technique condition authoring (AC: 2, 3, 4)
  - [x] Add the new condition type to the existing `conditionTypeOptions` and rule-card authoring flow in `src/windows/expression-map/`.
  - [x] Render the condition as one primitive row with the established comparator-plus-value pattern, using musical labels for `pizz` and `arco` and preserving auto-save through targeted backend commands.
  - [x] Keep the dedicated Expression Map window as the only authoring surface; do not move technique editing into score-context rows, the Sequence rack panel, or plugin-owned UIs.
  - [x] Preserve explicit `NoOp` placeholder behavior for unfinished rows and the current `default_actions` workflow when no authored rule matches.

- [x] Task 4: Add seam-local regression coverage and maintained docs updates (AC: 1, 2, 3, 4)
  - [x] Add Rust unit coverage in `src-tauri/src/playback/expression_maps.rs` for technique-only matches, combined technique-plus-articulation or technique-plus-note-length matches, first-match-wins ordering with technique-specific rules above broader fallbacks, additive condition-payload decode, and default-action fallback when technique rules do not match.
  - [x] Add or extend preparation- or transport-facing tests only enough to prove the prepared technique context reaches rule resolution through the existing backend-owned path without re-deriving score semantics in transport.
  - [x] Add frontend coverage for the new condition type option, editor rendering, and update or set-type flows in the Expression Map window.
  - [x] Update `docs/architecture.md`, `docs/data-models.md`, and `docs/development-guide.md` to record the new condition family, its derived-context ownership, its bounded `pizz` or `arco` scope, and the unchanged app-scoped editor ownership model.

## Dev Notes

### Story Intent

- Story 8.5 extends the reusable Expression Map rule vocabulary so maps can respond to backend-derived playing-technique meaning prepared in Story 8.4.
- The critical scope line is that the map matches derived note-time context; it does not own score-technique state, raw marking resolution, or the first audible proof.
- This story is model plus authoring plus matcher support. The first audible proof that technique-driven rules change routed playback stays in Story 8.6.

### User Model Clarified

- Authors should think in musical terms: a rule can match `playing technique is pizz` or `playing technique is not arco` alongside articulation and note length.
- Because multiple technique groups may coexist in future, a condition row should match membership of one active technique within the derived context rather than attempt to assert that the full context equals one value.
- The editor remains the same app-scoped Expression Map window. Score-context UI continues to link maps, not author technique logic.
- `pizz` and `arco` are the only required technique values in this story; the design should stay extensible without surfacing broader vocabulary yet.

### Previous Story Intelligence

- Story 6.3 established the dedicated app-scoped Expression Map window, primitive one-row condition authoring, auto-save, and targeted backend mutation commands keyed by stable nested ids.
- Story 6.4 established top-down first-match-wins playback resolution, map-owned `default_actions`, and the backend-owned rule matcher in `src-tauri/src/playback/expression_maps.rs`.
- Story 8.4 established `EffectivePlayingTechniqueContext`, no implicit `arco` before the first marking, hidden-still-semantic behavior, highest-`UniqueId` same-tick conflict resolution, and preparation-owned note-event derivation.
- HEAD already contains `feat(8.5): expression map prep`, which extended prepared note events and `ExpressionMapPlaybackContext` with `effective_playing_techniques`; Story 8.5 should build on that groundwork rather than replumbing transport.

### This Story Owns

- A new Expression Map condition family that matches backend-derived playing-technique context.
- Rust and TypeScript model updates for that condition family.
- App-scoped editor support for authoring the new condition type in the existing Expression Map window.
- Rule-matcher support, regression tests, and docs updates for the new condition family.

### This Story Does Not Own

- The first audible proof that technique-authored rules change routed playback; Story 8.6 owns that.
- New playing-technique score-entry UI, selection behavior, or properties-panel behavior; Stories 8.1 through 8.3 already own those.
- Any React-owned semantic resolution, raw score-marking scans inside the expression-map layer, or transport-owned reconstruction of same-tick technique state.
- Broader technique vocabulary, generalized region systems, or library-specific special cases beyond the bounded `pizz` and `arco` proof of concept.

### Current Relevant Seams

- `src-tauri/src/playback/expression_maps.rs`
  - Owner of `ExpressionMapCondition`, condition scaffolding, rule matching, library persistence, and most seam-local Expression Map tests.
- `src-tauri/src/lib.rs`
  - Owner of the existing targeted Tauri commands for condition add, set-type, update, and remove.
- `src-tauri/src/playback/preparation.rs`
  - Canonical transport or audition handoff and current owner of `prepare_supported_note_events_for_flow(...)`, which now supplies prepared note events with `effective_playing_techniques`.
- `src-tauri/src/playback/transport.rs`
  - Existing consumer of prepared note events and resolved Expression Map actions; it should stay a consumer, not a second semantic owner.
- `src-tauri/src/score/entries/playing_technique.rs`
  - Owner of `PlayingTechniqueKind`, `EffectivePlayingTechniqueContext`, exclusivity-group resolution, and the bounded `pizz` or `arco` semantics from Story 8.4.
- `src/services/types.ts`
  - TypeScript mirror for the reusable Expression Map schema.
- `src/windows/expression-map/model.ts`
  - Existing condition-type and option catalog used by the app-scoped Expression Map window.
- `src/windows/expression-map/expression-map-rule-card.tsx`
  - Existing primitive condition-row rendering for articulation and written-note-length authoring.
- `src/windows/expression-map/index.tsx`
  - Existing auto-save orchestration for set-type and update flows.
- `docs/architecture.md`
  - Canonical owner of backend or app-scoped Expression Map boundaries and preparation-owned note context.
- `docs/data-models.md`
  - Canonical owner of the reusable Expression Map schema and derived playing-technique model notes.
- `docs/development-guide.md`
  - Canonical owner of Expression Map workflow and playback guardrails.

### Implementation Guidance

- Model the new condition as matching one selected `PlayingTechniqueKind` against `ExpressionMapPlaybackContext.effective_playing_techniques`. The matcher should test whether the selected technique is present in the derived context, not whether the entire context equals one exact value.
- Reuse the current primitive condition-row design and comparator language where practical. A `==` or `!=` membership-style condition over one technique value is aligned with the existing articulation row and keeps the rule model small.
- Keep the condition additive to the current reusable-map schema. The library already persists stable ids for rules, conditions, and actions; extend that contract rather than adding index-based or whole-map replacement flows.
- Preserve the preparation-owned semantic boundary from Story 8.4. The matcher must consume already-derived note-time meaning; do not inspect raw `PlayingTechnique` entries, hidden flags, or same-tick cancellation behavior inside `expression_maps.rs`.
- Keep `.soloc` persistence unchanged. Projects continue to store only `InstrumentPlaybackRoute.expression_map_id`; the app-scoped library owns the reusable condition payload.
- Keep the first value set bounded to `pizz` and `arco`, but avoid baking those names into a dead-end boolean model. The condition shape should stay compatible with later `PlayingTechniqueKind` expansion.
- Preserve `NoOp` as the scaffolded placeholder for unfinished rows and keep current `default_actions` semantics unchanged when no authored rule matches.
- Avoid widening Story 8.5 into Story 8.6. This story should enable rule matching for the new condition family, not claim or prove audible technique-driven transport behavior beyond seam-local end-to-end checks.

### Risks And Regression Traps

- Re-deriving playing-technique state inside the Expression Map matcher or transport instead of consuming the prepared context from Story 8.4.
- Treating the new condition as exact-context equality rather than single-technique membership, which would block future multi-group technique context.
- Introducing a new condition-authoring surface outside the existing app-scoped Expression Map window.
- Breaking current first-match-wins, logical-AND, `NoOp`, or `default_actions` behavior while adding the new condition family.
- Forgetting to keep Rust and TypeScript condition unions in sync, including stable nested ids and serialized type strings.
- Adding compatibility residue or migrations unnecessarily. Existing project files do not embed Expression Map library payloads, and additive condition support should not require `.soloc` migration.

### Testing Requirements

- Add Rust unit tests near `src-tauri/src/playback/expression_maps.rs` for:
  - a technique condition matching `pizz` when the prepared context contains `Pizz`;
  - a technique condition not matching before any technique is active;
  - combined technique-plus-articulation and technique-plus-written-note-length rules using logical AND;
  - first-match-wins ordering where a more specific technique-combination rule sits above a broader fallback rule;
  - additive decode of persisted library payloads containing the new condition type;
  - continued `default_actions` fallback when technique conditions do not match.
- Add preparation- or transport-facing tests only enough to prove the new condition consumes the prepared `effective_playing_techniques` path already produced by `prepare_supported_note_events_for_flow(...)`.
- Add frontend tests for:
  - the new condition type appearing in `conditionTypeOptions`;
  - the Expression Map rule card rendering the technique comparator and value controls when selected;
  - set-type and update flows dispatching the expected payload shape through `actions.expressionMaps`.
- Recommended validation commands once implemented:
  - `npm run test:js`
  - `npm run check-types`
  - `npm run lint`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

### Documentation Impact

- Update `docs/architecture.md` to record that Expression Maps can now match backend-derived playing-technique context while preserving the app-scoped library or editor ownership split and the preparation-owned semantic boundary.
- Update `docs/data-models.md` to add the new `ExpressionMapCondition` variant, document that it matches derived `EffectivePlayingTechniqueContext` membership rather than raw score markings, and note that `.soloc` files still persist only `expression_map_id` references.
- Update `docs/development-guide.md` to capture the Story 8.5 guardrails: consume prepared technique context, preserve first-match-wins and logical-AND semantics, keep the dedicated Expression Map window as the only authoring surface, and bound the first value set to `pizz` and `arco`.
- Update AI-agent instructions only if implementation establishes a broader repo-wide rule for future derived semantic condition families; otherwise no instruction-file change is required.

### Architecture Compliance

- Rust remains the owner of durable Expression Map truth, condition persistence, and rule matching.
- Prepared note-time technique context remains backend-owned and preparation-owned; React and transport consume it but do not derive it.
- Expression Maps remain app-scoped reusable assets. Score-context surfaces own linkage to a map, not map authoring or library CRUD.
- The rack-first route model remains unchanged: route-linked map references stay on instrument routes, while reusable map payloads remain app-scoped machine-local assets.
- The new condition family must extend existing targeted command and state-emission flows instead of introducing whole-map replacement or frontend-owned shadow state.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, and TypeScript 5.9.3.
- No new dependency should be introduced for Story 8.5.

### File Structure Notes

- Keep backend condition-model and matcher changes under `src-tauri/src/playback/`.
- Keep `PlayingTechniqueKind` or effective-context semantics under the existing score-entry owner seam in `src-tauri/src/score/entries/`.
- Keep the TypeScript schema mirror in `src/services/types.ts`.
- Keep authoring UI changes within `src/windows/expression-map/`; do not create a second technique-condition editor surface elsewhere.
- Keep story tracking under `_bmad-output/implementation-artifacts/` and update sprint status in the same pass.

### Git Intelligence Summary

- `feat(8.5): expression map prep` already extended `ExpressionMapPlaybackContext` and the preparation-owned note-event path with `effective_playing_techniques`; implementation should extend that prepared context consumer path instead of moving semantics back into transport.
- Story 6.3 already moved nested Expression Map edits onto targeted backend commands keyed by stable ids. Reuse those commands and ids rather than rebuilding nested arrays in React.
- Story 6.4 already established top-down first-match-wins evaluation and map-owned `default_actions`, so Story 8.5 should only widen the condition vocabulary, not redefine resolution order or fallback behavior.
- Story 8.4 already settled the semantic rules that matter here: no implicit `Arco`, hidden entries stay semantic, and same-tick same-group conflicts resolve by highest `UniqueId`.

### Latest Technical Information

- No external library research is required for Story 8.5.
- Current repo truth as of 2026-04-09 is that `ExpressionMapPlaybackContext` in Rust already carries `effective_playing_techniques`, but `ExpressionMapCondition` currently supports only `NoOp`, `Articulation`, and `WrittenNoteLength`. The missing work is the new condition family plus editor and test support around that already-prepared context.
- The current app-scoped Expression Map window already supports primitive condition rows, `NoOp` scaffolding, and auto-save through existing condition add, set-type, and update commands. Story 8.5 should extend that workflow rather than inventing new mutation surfaces.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 8 and Story 8.5 acceptance criteria.
- `_bmad-output/planning-artifacts/architecture.md`
  - App-scoped Expression Map ownership, preparation-owned note context, and backend truth boundaries.
- `_bmad-output/planning-artifacts/prd.md`
  - Durable Expression Map direction and score-aware playback-domain scope.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Expression Map app-window ownership, panel-versus-window boundaries, and musical authoring model.
- `_bmad-output/implementation-artifacts/6-3-create-one-dedicated-expression-map-editor-window.md`
  - App-scoped editor ownership, primitive condition rows, and targeted backend mutation pattern.
- `_bmad-output/implementation-artifacts/6-4-drive-initial-realistic-playback-through-a-linked-expression-map.md`
  - First-match-wins playback resolution, `default_actions`, and backend-owned matcher seam.
- `_bmad-output/implementation-artifacts/8-4-derive-effective-playing-technique-context-for-notes.md`
  - Prepared effective technique context, no implicit `arco`, hidden semantic effect, and transport-boundary guardrails.
- `docs/architecture.md`
  - Expression Map library ownership and preparation-owned note-context boundary.
- `docs/data-models.md`
  - `PlayingTechnique` and Expression Map schema documentation.
- `docs/development-guide.md`
  - Expression Map workflow and playback guardrails.
- `src-tauri/src/playback/expression_maps.rs`
  - Condition model, scaffolding, matcher, and Expression Map library tests.
- `src-tauri/src/lib.rs`
  - Tauri command seam for condition add, set-type, update, and remove.
- `src-tauri/src/playback/preparation.rs`
  - Prepared note-event flow carrying `effective_playing_techniques`.
- `src-tauri/src/score/entries/playing_technique.rs`
  - `PlayingTechniqueKind` and effective-context semantics.
- `src/services/types.ts`
  - TypeScript mirror of the Expression Map condition union.
- `src/windows/expression-map/model.ts`
  - Existing condition-type options and editor value catalogs.
- `src/windows/expression-map/expression-map-rule-card.tsx`
  - Current primitive condition-row rendering.
- `/memories/repo/expression-map-library-ownership.md`
  - Route-reference-only `.soloc` contract and quiet missing-link behavior.
- `/memories/repo/expression-map-transport-proof-story-6-4.md`
  - Backend-owned resolution order and prepared-context playback path.
- `/memories/repo/playing-technique-effective-context-story-8-4.md`
  - Prepared technique-context handoff and Story 8.4 semantic guardrails.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-09: Story prepared after reviewing Epic 8 planning, Stories 6.3, 6.4, and 8.4, current Expression Map code paths, project-context rules, and repo memory covering Expression Map ownership plus playing-technique derivation.
- 2026-04-09: Current repo state already includes prepared `effective_playing_techniques` on `ExpressionMapPlaybackContext`; this story therefore focuses on the condition family, authoring workflow, matcher support, docs, and tests rather than another preparation refactor.
- 2026-04-09: Scope held to derived-context matching and app-scoped editor updates. Audible technique-driven playback proof remains Story 8.6.
- 2026-04-09: Implemented additive `playing_technique` condition support in Rust and TypeScript, kept nested-id mutation flows backend-owned, and matched prepared `effective_playing_techniques` membership without scanning raw score entries inside expression-map resolution.
- 2026-04-09: Added seam-local regression coverage in `expression_maps.rs`, a preparation-path proof that prepared technique context reaches rule resolution, frontend tests for the new authoring row, and doc updates covering derived-context ownership plus bounded `pizz`/`arco` scope.

### Completion Notes List

- Added a new additive `playing_technique` condition family to the reusable Expression Map model in Rust and TypeScript, using stable nested ids and the existing targeted add, set-type, and update mutation path.
- Extended backend rule matching to evaluate bounded `pizz`/`arco` membership against prepared `ExpressionMapPlaybackContext.effective_playing_techniques`, preserving logical-AND within a rule, top-down first-match-wins across rules, and default-action fallback behavior.
- Extended the dedicated Expression Map window to author playing-technique rows with the existing comparator-plus-value pattern and musical `pizz` / `arco` labels while keeping `NoOp` placeholders and app-scoped window ownership unchanged.
- Added Rust, preparation-path, and frontend regression coverage for the new condition family and updated architecture, data-model, and development-guide docs to record the derived-context ownership boundary and unchanged `.soloc` route-reference contract.
- Validation completed with `npm run check-types`, `npm run lint`, `npm run test:js`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`.

### File List

- `_bmad-output/implementation-artifacts/8-5-extend-expression-maps-to-match-playing-technique-context.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src-tauri/src/playback/expression_maps.rs`
- `src-tauri/src/playback/preparation.rs`
- `src/services/types.ts`
- `src/windows/expression-map/expression-map-rule-card.tsx`
- `src/windows/expression-map/index.test.tsx`
- `src/windows/expression-map/model.test.ts`
- `src/windows/expression-map/model.ts`

### Change Log

- 2026-04-09: Created Story 8.5 implementation context and updated sprint tracking to `ready-for-dev`.
- 2026-04-09: Implemented bounded playing-technique expression-map conditions across backend matching, dedicated editor authoring, seam-local regression coverage, and supporting docs; story advanced to `review`.
