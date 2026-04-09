# Story 8.4: Derive Effective Playing-Technique Context For Notes

Status: done

## Story

As a maintainer,
I want Compose to derive active playing-technique context for notes,
so that playback and later consumers can use score meaning that persists beyond a single tone.

## Acceptance Criteria

1. Given playing-technique entries may stay in effect across later notes, when score meaning is prepared for playback-facing consumers, then Compose derives effective note-time technique context from the relevant master-track markings, and notes are no longer limited to only immediate tone-local expression inputs such as articulation and written duration.
2. Given the first proof-of-concept technique family is an exclusive bowed-technique group, when `pizz` and `arco` markings are resolved, then Compose treats them as mutually exclusive entries in the same semantic group, and the derived note context exposes the effective playback-facing condition for that group rather than forcing expression maps to interpret raw cancellation logic.
3. Given a `PlayingTechnique` entry takes effect from the tick where it is defined, when note context is resolved for tones that begin at the same tick, then those tones see the updated effective technique state immediately, and later notes continue to inherit that state until a later marking in the same exclusive group changes it.
4. Given later technique families may be more complex than `pizz` and `arco`, when the first derived context shape is introduced, then it establishes one scalable resolver pattern for exclusive technique groups, and it stays deterministic enough for rule matching and testing.
5. Given semantic resolution should not be owned by transport timing code, when active playing-technique context is derived, then that derivation happens in a backend-owned score-semantics or preparation seam before transport execution, and transport consumes already-resolved note context rather than inventing persistent technique meaning on the fly.

## Tasks / Subtasks

- [x] Task 1: Define one backend-owned effective playing-technique resolver for exclusive groups (AC: 2, 4)
  - [x] Add exclusivity-group helper behavior near `PlayingTechniqueKind` so `Pizz` and `Arco` resolve as one bowed-technique family without persisting group metadata on entries.
  - [x] Introduce a derived note-context carrier such as `EffectivePlayingTechniques` or `EffectivePlayingTechniqueContext` that can represent no active technique before the first marking and multiple active techniques across future groups without changing raw score persistence.
  - [x] Apply the user clarification that hidden playing-technique entries still affect playback semantics even though they do not render or emit hit targets.
  - [x] Apply the user clarification that same-tick conflicting entries in the same exclusive group resolve deterministically with last-created wins; use a stable durable ordering rule such as highest `UniqueId` at the tick rather than frontend or transport ordering.

- [x] Task 2: Derive note-time technique context inside the preparation-owned playback event seam (AC: 1, 3, 5)
  - [x] Extend the current playback-tone event collection path so note events are prepared with the relevant stave-master playing-technique state, not just tone-local articulations and written duration.
  - [x] Keep the derivation backend-owned before transport execution by moving the resolver into `src-tauri/src/playback/tuplet_projection.rs` or a focused sibling helper used by preparation, and thread in the owning stave master track or a precomputed resolver instead of letting `transport.rs` walk raw score state.
  - [x] Ensure a technique marking takes effect at its own tick so all tones that begin at the same tick see the updated state immediately.
  - [x] Ensure later notes inherit the last effective same-group marking on that stave until another marking in that group replaces it.

- [x] Task 3: Plumb the derived context forward without widening Story 8.4 into expression-map authoring or audible-proof work (AC: 1, 2, 4, 5)
  - [x] Extend `PlaybackToneEvent` and `ExpressionMapPlaybackContext` so prepared note-time context can carry derived playing-technique meaning alongside articulations and written duration.
  - [x] Keep transport and audition consumers thin: they may read already-prepared context, but they must not derive persistent playing-technique semantics on the fly.
  - [x] Reuse the shared `prepare_supported_playback_with_expression_maps(...)` handoff so transport and audition stay aligned on note-time context without inventing a second semantics path.
  - [x] Do not add new expression-map condition types, editor UI, or audible rule-matching behavior in this story; those stay in Stories 8.5 and 8.6.

- [x] Task 4: Add seam-local regression coverage and update maintained docs (AC: 1, 2, 3, 4, 5)
  - [x] Add Rust unit coverage around the effective-technique resolver for default-no-technique state, same-tick immediate application, same-group replacement over time, same-tick conflicting same-group entries, hidden-entry semantic participation, same-stave independence, multi-stave independence, and no cross-flow leakage.
  - [x] Extend playback-event or preparation coverage so collected note events carry the resolved technique context before transport scheduling, including tones inside tuplets and host-track tones sharing the same flow.
  - [x] Extend transport or audition-facing regression coverage only enough to prove they consume prepared note-time context rather than re-deriving technique state in timing code.
  - [x] Update `docs/architecture.md`, `docs/data-models.md`, and `docs/development-guide.md` to describe the new backend-owned derivation seam, the no-implicit-technique default, the hidden-still-semantic rule, and the deterministic same-tick conflict rule.

### Review Findings

- [x] [Review][Patch] Playing-technique derivation still starts in transport instead of the shared preparation seam [src-tauri/src/playback/transport.rs:534]
- [x] [Review][Patch] Preparation-owned consumer coverage for the new note context is still missing [src-tauri/src/playback/preparation.rs:253]
- [x] [Review][Patch] The playing-technique data-model docs still describe a persisted schema addition even though the change is derived-only [docs/data-models.md:163]

## Dev Notes

### Story Intent

- Story 8.4 is the first backend semantic-resolution pass over durable `PlayingTechnique` score entries.
- The core job is not to make playback audibly different yet. It is to derive deterministic note-time technique meaning that later playback consumers and expression maps can trust.
- The highest-risk failure mode is leaving semantic ownership in `transport.rs` or forcing later expression-map rules to reconstruct cancellation and same-tick precedence from raw score markings.

### User Clarifications

- Before any playing-technique marking appears on a stave, the effective bowed-technique context is empty rather than implicitly `arco`.
- Hidden playing-technique markings still affect derived playback context. Hidden changes visibility and selectability, not semantic effect.
- If conflicting techniques from the same exclusive group exist at the same tick on one stave, the last-created marking wins deterministically.

### This Story Owns

- A backend-owned resolver pattern for exclusive-group playing-technique semantics.
- A prepared note-time context shape that carries derived playing-technique meaning forward with each note event.
- Deterministic rules for no-marking default state, same-tick immediate effect, hidden-entry semantics, and same-group conflict resolution.
- Regression coverage and documentation for the new preparation-owned semantic seam.

### This Story Does Not Own

- New playing-technique authoring UI, renderer behavior, hit targets, or properties-panel controls.
- Expression-map condition types, expression-map editor UI, or rule-matching support for playing techniques; that is Story 8.5.
- The first audible proof that playing techniques drive routed playback; that is Story 8.6.
- Mid-play semantic refresh beyond the existing bounded transport snapshot contract from Story 5.3.
- Broader technique vocabulary, alias parsing, or new persisted fields on `PlayingTechnique` entries.

### Current Relevant Seams

- `src-tauri/src/score/entries/playing_technique.rs`
  - Owner of `PlayingTechniqueKind`, persisted entry shape, same-tick duplicate replacement for identical kinds, and basic track lookup helpers.
- `src-tauri/src/score/components/articulation.rs`
  - Best reference for deriving exclusivity-group behavior from enum variants rather than persisting cancellation state directly.
- `src-tauri/src/playback/tuplet_projection.rs`
  - Current owner of `PlaybackToneEvent` and `collect_track_playback_events(...)`; today it only carries tone-local articulation and written-duration data.
- `src-tauri/src/playback/preparation.rs`
  - Canonical transport and audition handoff; note-time semantic derivation should feed this seam rather than transport-local state.
- `src-tauri/src/playback/transport.rs`
  - Current consumer that builds `ExpressionMapPlaybackContext` from prepared tone events; 8.4 should make this consumer thinner, not smarter.
- `src-tauri/src/playback/audition.rs`
  - Secondary consumer of the shared preparation snapshot; it should inherit the same derived note-time context without a separate resolution path.
- `src-tauri/src/playback/expression_maps.rs`
  - Owner of `ExpressionMapPlaybackContext` and current articulation or written-duration rule matching; 8.4 may extend the context shape, but 8.5 owns new condition matching behavior.
- `docs/architecture.md`
  - Canonical backend-ownership and preparation or transport boundary rules.
- `docs/data-models.md`
  - Canonical persisted-model and derived semantics documentation for `PlayingTechnique` and expression-map context.
- `docs/development-guide.md`
  - Maintained workflow guardrails for playback preparation and playing-technique behavior.

### Architecture Compliance

- Keep semantic ownership backend-owned and preparation-owned. `transport.rs` must consume already-resolved note context rather than derive durable technique meaning on the fly.
- Keep raw score persistence unchanged. Do not add persisted exclusivity-group ids, cancellation flags, default-technique flags, or playback-routing state to `PlayingTechnique`.
- Preserve the current score or UI split: hidden playing-technique entries remain non-rendered and non-selectable in the notation workflow, but user clarification for Story 8.4 says they still affect derived playback semantics.
- Keep React out of canonical note-time technique state. Any consumer-facing projection must come from backend-prepared context.
- Preserve the bounded transport snapshot contract from Story 5.3: score-structure edits, including playing-technique edits, persist immediately but do not rewrite in-flight transport semantics mid-pass.
- Do not widen Story 8.4 into expression-map authoring or matching UI. The new context shape exists so later stories can consume it cleanly.

### Implementation Guidance

- Prefer adding exclusivity metadata as helper behavior on `PlayingTechniqueKind` or a focused sibling type, analogous to `Articulation::exclusivity_group()`, rather than hardcoding `pizz` or `arco` switches in transport.
- Because `collect_track_playback_events(track)` currently only sees the tone track, expect to widen that seam or add a sibling helper that also has access to the owning stave master track or a precomputed per-stave resolver.
- Keep the derived note-time payload scalable. A set-like or dedicated context wrapper for active effective techniques is preferable to a one-off boolean or raw cancellation flag because later technique groups may coexist.
- Resolve same-tick same-group conflicts with a stable durable rule. Prefer highest `UniqueId` at the tick over incidental iteration order so the answer survives save or load and test replays.
- Apply same-tick effect before evaluating notes at that tick. All tones that start at the marking tick should see the updated state immediately, regardless of note ordering within the tick.
- Hidden entries remain semantic. Do not filter them out just because the renderer or hit-test path skips them.
- Keep no-technique default explicit. Before the first relevant marking on a stave, note-time context should carry no active technique for that group rather than inventing implicit `arco`.
- Keep the new context plumbing additive to current rule evaluation. Extending `ExpressionMapPlaybackContext` now is acceptable, but adding `ExpressionMapCondition` variants or changing first-match-wins behavior belongs to Story 8.5.
- If performance becomes a concern, precompute per-stave effective technique state during preparation rather than scanning the full master track from scratch for every note in transport code.

### Risks And Regression Traps

- Leaving semantic derivation in `transport.rs`, which would violate the explicit story boundary and duplicate logic across transport and audition.
- Treating playing techniques like tone-local articulations instead of stave-master directives that persist across later notes.
- Filtering hidden entries out of semantic resolution because hidden already suppresses draw and hit-target behavior.
- Using unstable same-tick ordering, which would make conflicting same-group markings non-deterministic across reloads or test runs.
- Introducing an implicit `arco` default before the first marking, which would contradict the recorded user clarification and make later rule matching less explicit.
- Widening 8.4 into expression-map condition matching, playback proof behavior, or mid-play live refresh work that belongs to Stories 8.5 and 8.6.
- Breaking the current tuplet-aware note-event path while threading in master-track semantics; tuplet projection must continue to carry note-time context without re-deriving tuplet meaning in transport.
- Forgetting multi-stave isolation. Playing-technique resolution must stay tied to the owning stave master track and must not leak across staves or flows.

### Testing Requirements

- Add Rust unit tests for playing-technique exclusivity resolution covering:
  - no active technique before the first marking
  - immediate same-tick effect for notes at the marking tick
  - inheritance until the next same-group marking
  - hidden entries still affecting semantic resolution
  - conflicting same-group same-tick entries resolving last-created wins
  - independent resolution across multiple staves and flows
- Extend playback-event collection tests so both host-track tones and tuplet-contained tones carry derived technique context from the owning stave master track.
- Add preparation-level coverage proving the derived context is available before transport scheduling and is shared by both transport and audition preparation consumers.
- Keep transport and audition regression additions bounded to ownership checks: prove they consume prepared context rather than recomputing semantics, but do not claim new audible behavior in 8.4.
- Update or add expression-map context tests only as needed to keep the extended `ExpressionMapPlaybackContext` compile-safe and deterministic without yet adding playing-technique matching rules.

### Documentation Impact

- Update `docs/architecture.md` to record that effective playing-technique note context is derived in a backend-owned preparation or score-semantics seam before transport execution.
- Update `docs/data-models.md` to describe the no-implicit-technique default, the derived exclusive-group semantics, and the deliberate distinction between hidden visual state and hidden semantic effect for playing techniques.
- Update `docs/development-guide.md` to capture the Story 8.4 guardrails: no implicit `arco`, hidden entries still affect playback semantics, and conflicting same-group same-tick entries resolve last-created wins.
- Update AI-agent instructions only if implementation establishes a broader repo-wide rule that should govern future backend semantics beyond this feature line.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- No new dependencies should be introduced for Story 8.4.

### File Structure Notes

- Keep score-entry type metadata and exclusivity helpers near the existing playing-technique owner seam under `src-tauri/src/score/entries/`.
- Keep note-event derivation and preparation wiring under `src-tauri/src/playback/`; do not create a frontend-owned mirror of derived technique context.
- Keep story tracking under `_bmad-output/implementation-artifacts/` and update sprint status in the same pass.

### Git Intelligence Summary

- `feat(8.1): playing technique setup` established the durable `PlayingTechnique` entry family, the bounded `Pizz` or `Arco` vocabulary, and the rule that semantic meaning is derived later from the inner kind rather than persisted on the entry.
- `feat(8.2): playback technique rendering` added renderer, selection, popover, and delete workflows plus the visible same-tick canonical-anchor behavior, but intentionally deferred derived note semantics.
- `feat(8.3): selection panel` kept playing-technique properties intentionally narrow with only the shared `Hidden` toggle, reinforcing that 8.4 should stay backend semantics plus tests or docs rather than widening into more UI.
- Epic 6 already proved that transport resolves expression-map rules from prepared per-note context using articulation and written duration, so 8.4 should extend that prepared context rather than inventing a new playback model.

### Latest Technical Information

- No external library research is needed for Story 8.4.
- Current repo truth as of 2026-04-09 is that `ExpressionMapPlaybackContext` only carries `articulations` and `written_duration_ticks`, `collect_track_playback_events(...)` only sees tone-track data, and `transport.rs` currently constructs the expression-map context per note. Story 8.4 should correct that ownership boundary by carrying derived playing-technique context forward from preparation.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 8 and Story 8.4 acceptance criteria.
- `_bmad-output/implementation-artifacts/8-1-add-durable-playing-technique-score-entries.md`
  - Durable playing-technique data model, same-tick coexistence rules, and derived-meaning guardrails.
- `_bmad-output/implementation-artifacts/8-2-render-and-select-playing-technique-markings-above-the-stave.md`
  - Visible playing-technique workflow, same-tick overlap behavior, and shared selection model.
- `_bmad-output/implementation-artifacts/8-3-add-playing-technique-properties-panels.md`
  - Current `Hidden` properties workflow boundary and the non-rendered, non-selectable hidden-entry contract on the UI side.
- `docs/architecture.md`
  - Preparation or transport ownership rules and app-scoped expression-map architecture.
- `docs/data-models.md`
  - `PlayingTechnique` persistence rules and current expression-map condition vocabulary.
- `docs/development-guide.md`
  - Existing transport, expression-map, and playing-technique authoring guardrails.
- `src-tauri/src/score/entries/playing_technique.rs`
  - `PlayingTechniqueKind`, track lookup helpers, and current entry ownership.
- `src-tauri/src/score/components/articulation.rs`
  - Exclusivity-group reference pattern.
- `src-tauri/src/playback/tuplet_projection.rs`
  - Current playback note-event collection seam.
- `src-tauri/src/playback/preparation.rs`
  - Canonical prepared playback snapshot seam.
- `src-tauri/src/playback/transport.rs`
  - Current per-note expression-map context consumer that should become thinner.
- `src-tauri/src/playback/audition.rs`
  - Shared preparation consumer for bounded pitch audition.
- `src-tauri/src/playback/expression_maps.rs`
  - `ExpressionMapPlaybackContext` and rule-matching seam that 8.5 will extend.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-09: Story prepared from Epic 8 planning, Stories 8.1 through 8.3 artifacts, current playback preparation and expression-map seams, and repo architecture or development docs.
- 2026-04-09: User clarified that the default bowed-technique context is empty before the first marking, hidden playing-technique entries still affect playback semantics, and conflicting same-group same-tick entries resolve last-created wins.
- 2026-04-09: Story scope held to backend semantic derivation, prepared note-context plumbing, seam-local regression coverage, and docs updates; expression-map condition matching remains Story 8.5 and audible proof remains Story 8.6.
- 2026-04-09: Implemented a stave-master effective playing-technique resolver, threaded prepared note-time context through `PlaybackToneEvent` and `ExpressionMapPlaybackContext`, and removed transport-owned context assembly.
- 2026-04-09: Added resolver and playback-event regression coverage for same-tick application, hidden semantic participation, same-stave shared context, multi-stave independence, and no cross-flow leakage; updated maintained playback docs.
- 2026-04-09: Validation passed with `cargo test`, `cargo clippy --tests -- -D warnings`, `npm run lint`, and `npm run check-types`.
- 2026-04-09: Review follow-up moved flow-scoped note-event derivation into `src-tauri/src/playback/preparation.rs`, added ownership-focused preparation and transport regression coverage, and clarified that the story remains migration-free.

### Completion Notes List

- Added `PlayingTechniqueExclusivityGroup`, `EffectivePlayingTechniqueContext`, and `EffectivePlayingTechniqueResolver` to keep bowed-technique semantics score-owned and deterministic.
- Extended prepared playback tone events so `ExpressionMapPlaybackContext` now carries effective playing-technique meaning alongside articulations and written duration before transport scheduling.
- Moved flow-scoped note-event derivation into `src-tauri/src/playback/preparation.rs` so transport consumes already-prepared playback events instead of rebuilding effective technique state from stave master tracks.
- Added seam-local Rust coverage for no-technique default state, same-tick application, hidden semantic participation, same-stave shared context, multi-stave independence, no cross-flow leakage, and tuplet-aware note-event carry-forward.
- Added preparation-owned and transport-consumer ownership checks proving prepared note events carry the derived technique context before scheduling.
- Updated maintained docs to record the backend-owned derivation seam, empty-before-first-marking behavior, hidden-still-semantic rule, highest-`UniqueId` same-tick conflict resolution, and the explicit no-migration posture for this derived-only change.
- Cleared two existing draw-module clippy blockers so the required backend quality gate could pass unchanged after the story work.
- Validation completed: targeted `cargo test` checks for preparation, transport, and tuplet projection plus `cargo clippy --tests -- -D warnings` all passed.

### File List

- `_bmad-output/implementation-artifacts/8-4-derive-effective-playing-technique-context-for-notes.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src-tauri/src/instructions/draw/draw_playing_technique.rs`
- `src-tauri/src/instructions/draw/draw_tempo.rs`
- `src-tauri/src/playback/expression_maps.rs`
- `src-tauri/src/playback/preparation.rs`
- `src-tauri/src/playback/transport.rs`
- `src-tauri/src/playback/tuplet_projection.rs`
- `src-tauri/src/score/entries/playing_technique.rs`

### Change Log

- 2026-04-09: Started Story 8.4, advanced sprint tracking to in-progress, and implemented backend-owned effective playing-technique derivation plus prepared note-context plumbing.
- 2026-04-09: Added regression coverage, updated maintained docs, cleared the remaining backend clippy blockers in the draw seams, and moved the story to review after all validations passed.
- 2026-04-09: Addressed code-review follow-up by moving flow-scoped note-event derivation into the preparation seam, adding ownership-focused preparation and transport tests, clarifying the derived-only no-migration docs, and closing the review findings.
