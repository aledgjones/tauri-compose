# Story 8.6: Prove Technique-Driven Playback Through Derived Context

Status: review

## Story

As a composer,
I want playing-technique markings to audibly influence routed playback through expression maps,
so that score-authored technique intent can drive realistic sample selection beyond note-local expression cues.

## Acceptance Criteria

1. Given a routed instrument has an expression map that matches derived playing-technique context, when I play supported score material through the existing transport-owned routed playback path, then the resolved active playing techniques contribute to the winning rule for each note, and the resulting playback behavior comes from the existing backend preparation plus transport pipeline rather than a new one-off playback path.
2. Given `pizz` and `arco` are the first exclusive-group proof of concept, when a `pizz` marking is placed at tick `t` and an `arco` marking is later placed at tick `u`, then notes beginning at tick `t` and later resolve the `pizz`-driven playback path until tick `u`, and notes beginning at tick `u` and later immediately stop resolving that `pizz`-driven path because the same exclusive-group resolver has switched the effective technique state.
3. Given the user should author score markings rather than direct playback state, when the first playing-technique-driven playback proof is delivered, then the system derives the audible result from score-authored technique markings plus the user-authored expression map, and it does not require users to manually author hidden playback-state regions outside the score model or separately manage exclusive-group playback logic themselves.

## Tasks / Subtasks

- [x] Task 1: Prove technique-driven rule resolution on the existing routed transport path (AC: 1, 3)
  - [x] Reuse `prepare_supported_playback_with_expression_maps(...)`, `prepare_supported_note_events_for_flow(...)`, `build_supported_playback_snapshot_with_expression_maps(...)`, and `transport_events_for_tick(...)` rather than creating a technique-specific playback seam.
  - [x] Keep `collect_scheduled_notes_from_prepared_events(...)` consuming prepared `tone.expression_map_context` and `resolve_expression_map_playback(...)` so the proof stays on the current backend preparation plus transport pipeline.
  - [x] Extend the existing transport proof harness in `src-tauri/src/playback/transport.rs` with a technique-specific linked-map scenario, either by widening `proof_expression_map_library_state()` or by adding a minimal sibling helper dedicated to the `pizz` or `arco` proof.
  - [x] If the proof exposes a genuine transport or preparation gap, fix it narrowly in the existing backend path without widening frontend, audition, or persistence scope.

- [x] Task 2: Prove exclusive-group switching from `pizz` to `arco` across ticks (AC: 1, 2)
  - [x] Add a transport-facing scenario with a stave master track that applies `pizz` at tick `t`, `arco` at tick `u`, and supported notes before and after `u`, using authored Expression Map rules whose resolved actions are audibly distinguishable.
  - [x] Assert that notes from tick `t` up to but not including tick `u` queue the `pizz`-driven playback path, and notes at tick `u` and later no longer queue that path because the prepared effective technique context has switched to `arco`.
  - [x] Preserve same-tick immediate effect at tick `u`; notes that begin exactly on the `arco` tick must resolve the updated rule without an extra grace region or transport-side lookahead hack.
  - [x] Keep the first audible technique proof bounded to `pizz` and `arco`; do not broaden this story into additional technique vocabulary, generalized region authoring, or new action families.

- [x] Task 3: Preserve score-authored semantics and existing rule-model boundaries (AC: 1, 2, 3)
  - [x] Preserve the Story 8.4 semantic guardrails: no implicit `arco` before the first marking, hidden entries still affect semantics, and same-tick same-group conflicts resolve deterministically by highest `UniqueId`.
  - [x] Preserve the Story 6.4 and 8.5 rule guardrails: conditions remain logical AND within a rule, rules remain top-down first-match-wins, and `default_actions` or plugin-default fallback behavior remains authoritative when no authored technique rule matches.
  - [x] Keep the audible result authored by score markings plus reusable Expression Map data; do not introduce hidden playback-state regions, transport-owned exclusive-group state, or plugin-specific hard-coded transport branches.
  - [x] Do not add new score-context playback UI, expression-map authoring UI, active-expression overlays, audition-specific technique paths, or mid-play semantic refresh behavior in Story 8.6.

- [x] Task 4: Add seam-local regression coverage and update maintained docs (AC: 1, 2, 3)
  - [x] Add or extend Rust tests in `src-tauri/src/playback/transport.rs` for technique-driven keyswitch selection, `pizz` to `arco` switching across ticks, and fallback behavior when no technique-specific rule matches.
  - [x] Add or extend preparation-level coverage only enough to prove the transport-facing scenario still consumes prepared `effective_playing_techniques` from `prepare_supported_note_events_for_flow(...)` instead of a transport-local score scan.
  - [x] Update `docs/architecture.md`, `docs/data-models.md`, and `docs/development-guide.md` to record that routed transport playback now has a shipped proof for technique-driven Expression Map rule resolution through derived context, while keeping the proof bounded to the `pizz` or `arco` exclusive-group MVP path.
  - [x] Update AI-agent instructions only if implementation establishes a new repo-wide rule beyond the existing prepared-context and transport-ownership guardrails.

## Dev Notes

### Story Intent

- Story 8.6 is the first audible proof that score-level playing-technique semantics can change routed transport playback through authored Expression Map rules.
- The infrastructure added in Stories 8.4 and 8.5 already prepares effective technique context and matches it in Expression Map rules. The purpose of 8.6 is to prove that authored technique rules reach the existing transport-owned audible path without introducing another playback architecture.
- This story should be proof-first rather than plumbing-first. If the current seams already satisfy the behavior, capture that truth with transport-level regression coverage and only make code changes where the proof reveals a concrete gap.

### User Model Clarified

- The user authors visible score markings such as `pizz` and `arco`, plus reusable Expression Map rules. The user does not author hidden playback-only regions or manual exclusive-group state.
- The audible change should come from the same route-linked Expression Map workflow already established in Epic 6, not from a plugin-specific shortcut or transport-only condition branch.
- The proof should be understandable in musical terms: `pizz` should keep selecting the `pizz`-authored playback path until a later `arco` marking takes over, and the handoff happens immediately on the `arco` tick.
- The first shipped proof remains intentionally bounded to the bowed-technique `pizz` or `arco` group.

### Previous Story Intelligence

- Story 6.4 already proved that routed transport playback can resolve Expression Map rules per note on the existing backend-owned path, with first-match-wins semantics, map-owned `default_actions`, and no separate expression playback engine.
- Story 8.4 established backend-owned effective playing-technique derivation on stave master tracks, including no implicit `arco`, hidden-still-semantic behavior, and highest-`UniqueId` same-tick conflict resolution.
- Story 8.5 extended the reusable Expression Map model so authored rules can match `effective_playing_techniques` through the existing condition model and dedicated app-scoped editor workflow.
- Recent commits `feat(8.5): expression map prep` and `feat(8.5): expression maps` mean the likely missing work for 8.6 is transport-facing proof and any narrow backend fixes revealed by that proof, not another expansion of the model or editor.

### This Story Owns

- The first transport-owned audible proof that authored playing-technique rules affect routed playback through the shared backend preparation plus transport path.
- A bounded `pizz` to `arco` proof scenario showing exclusive-group switching over time.
- Seam-local transport and preparation regression coverage that demonstrates authored technique rules reach scheduled playback events.
- Maintained documentation updates describing the shipped technique-driven transport proof and its bounded scope.

### This Story Does Not Own

- New playing-technique authoring UI, rendering behavior, hit targets, or properties-panel work; Stories 8.1 through 8.3 already own that scope.
- New Expression Map condition types, editor controls, or reusable-map persistence changes; Story 8.5 already owns that scope.
- A second technique-aware playback engine for audition, editor preview, or frontend-owned playback behavior.
- Mid-play semantic refresh that bypasses the bounded transport snapshot contract from Story 5.3.
- Broader technique vocabulary, generalized region systems, plugin-specific compatibility claims, or additional playback action families beyond what the current Expression Map model already supports.

### Current Relevant Seams

- `src-tauri/src/playback/transport.rs`
  - Owner of `build_supported_playback_snapshot_with_expression_maps(...)`, `transport_events_for_tick(...)`, `collect_scheduled_notes_from_prepared_events(...)`, and the current transport-facing Expression Map proof tests.
- `src-tauri/src/playback/preparation.rs`
  - Owner of `prepare_supported_playback_with_expression_maps(...)`, `prepare_supported_note_events_for_flow(...)`, and the preparation-level technique-context proof helpers such as `make_flow_score_with_track_and_master(...)`, `insert_playing_technique(...)`, and `expression_map_library_with_pizz_map()`.
- `src-tauri/src/playback/expression_maps.rs`
  - Owner of `resolve_expression_map_playback(...)`, `ExpressionMapPlaybackContext`, playing-technique condition matching, and fallback semantics.
- `src-tauri/src/score/entries/playing_technique.rs`
  - Owner of `PlayingTechniqueKind`, exclusivity-group behavior, and effective-technique context semantics.
- `docs/architecture.md`
  - Canonical owner of backend transport and preparation boundaries.
- `docs/data-models.md`
  - Canonical owner of the prepared Expression Map playback context and technique-related model notes.
- `docs/development-guide.md`
  - Canonical owner of playback workflow guardrails and story-specific behavior notes.

### Architecture Compliance

- Keep technique meaning backend-owned and preparation-owned. Transport must consume prepared note context, not derive persistent technique state on the fly.
- Keep routed playback rack-first and route-linked. The expression map remains a reusable asset linked from the instrument route; transport does not become a second asset owner.
- Keep React out of canonical technique-driven playback truth. No frontend-owned shadow state or UI-only derived playback model belongs in Story 8.6.
- Keep the supported transport path backend-owned and bounded: reuse the existing play-start snapshot, scheduled-note projection, and engine queue flow rather than adding a live-probing or per-tick state-derivation path.
- Preserve the existing no-fallback rule on the product path: no sampler, SCVI, or audition-only compatibility behavior should be reintroduced while proving technique-driven playback.

### Implementation Guidance

- Start from the existing transport proof harness in `src-tauri/src/playback/transport.rs`, which already proves articulation-driven Expression Map playback. Prefer extending that harness with a technique-proof map and master-track technique scenario rather than creating a new cross-cutting test framework.
- Use authored rules with audibly distinguishable actions, ideally different keyswitch notes for `pizz` and `arco`, so transport assertions can stay concrete at the queued-event level.
- Reuse `make_flow_score_with_track_and_master(...)` and `insert_playing_technique(...)` patterns from `preparation.rs` to build the technique scenario instead of reconstructing master-track setup from scratch in multiple places.
- If needed, add a transport-local sibling to `proof_expression_map_library_state()` that models the `pizz` to `arco` proof case. Keep helper scope local to the owning seam.
- Preserve same-tick immediacy by asserting behavior at the scheduled-note level, not by compensating with extra lead-time hacks or transport-owned cancellation logic.
- If the technique proof uncovers a gap, prefer the narrowest backend fix that keeps authored rule resolution, preparation ownership, and transport scheduling aligned.

### Risks And Regression Traps

- Re-deriving effective technique state inside `transport.rs` instead of consuming the prepared `expression_map_context` already attached to each note.
- Proving the story with a hard-coded transport branch for `pizz` or `arco` instead of authored Expression Map data.
- Accidentally broadening Story 8.6 into new UI, audition behavior, or generalized technique authoring because transport and audition seams live nearby.
- Breaking first-match-wins or `default_actions` behavior while trying to prove the technique path.
- Forgetting the no-implicit-`arco` rule and writing proof cases that assume bowed default state before the first marking.
- Missing same-tick immediate effect at the `arco` transition tick, which would undermine the exclusive-group model established in Story 8.4.
- Letting technique proof work leak across instruments or flows when one reusable map is shared.

### Testing Requirements

- Add or extend Rust transport tests for:
  - technique-driven keyswitch selection on the existing routed transport path;
  - `pizz` at tick `t` and `arco` at tick `u`, with notes on both sides of the transition;
  - same-tick immediate switch at tick `u`;
  - fallback behavior when no technique-specific rule matches.
- Add or extend preparation-level tests only enough to prove the transport proof still consumes prepared `effective_playing_techniques` rather than a transport-local scan.
- Preserve or extend seam-local shared-map independence coverage if any helper reuse touches the existing shared-map test path.
- Recommended validation commands once implemented:
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
  - `npm run check-types`
  - `npm run lint`

### Documentation Impact

- Update `docs/architecture.md` to record that the shared backend preparation plus transport pipeline now has a shipped technique-driven playback proof based on derived note context.
- Update `docs/data-models.md` to note that `ExpressionMapPlaybackContext.effective_playing_techniques` now has an audible routed-transport proof path, still bounded to the `pizz` or `arco` exclusive-group MVP case.
- Update `docs/development-guide.md` to capture the Story 8.6 guardrails: no new playback path, proof stays transport-owned, authored rules drive the audible result, and `pizz` to `arco` switching is immediate at the marking tick.
- Update AI-agent instructions only if implementation establishes a broader repo-wide practice that future stories must follow.

### Library And Framework Requirements

- Stay on the current repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, and TypeScript 5.9.3.
- No new dependencies should be introduced for Story 8.6.

### File Structure Notes

- Keep backend proof work under `src-tauri/src/playback/`, primarily `transport.rs` and any narrowly related preparation helper or test support.
- Keep technique-semantic ownership under `src-tauri/src/score/entries/` and `src-tauri/src/playback/preparation.rs`; do not create a frontend-owned mirror.
- Keep docs updates in the maintained `docs/` set rather than adding one-off story-only technical notes elsewhere.
- Keep story tracking under `_bmad-output/implementation-artifacts/` and update sprint status in the same pass.

### Git Intelligence Summary

- `feat(8.5): expression map prep` already carried `effective_playing_techniques` into prepared note events and `ExpressionMapPlaybackContext`, which means Story 8.6 should extend the existing prepared-context consumer path rather than replumbing transport ownership.
- `feat(8.5): expression maps` already added the `playing_technique` condition family and bounded `pizz` or `arco` authoring support, so the missing work should be transport-facing proof and any narrow corrective fixes only.
- The current recent history contains no new architecture reset for playback, reinforcing that 8.6 should be incremental proof on the shipped seams rather than another round of model redesign.

### Latest Technical Information

- No external library research is required for Story 8.6.
- Current repo truth as of 2026-04-09 is that `transport.rs` already resolves Expression Map actions from prepared note context, `preparation.rs` already proves that prepared playing-technique context reaches Expression Map rule resolution, and the likely missing 8.6 work is a technique-specific transport proof plus any minimal backend fix that proof exposes.
- The current transport proof helper `proof_expression_map_library_state()` is articulation-focused, so Story 8.6 will likely need a technique-proof sibling or extension with distinct authored `pizz` and `arco` actions.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 8 and Story 8.6 acceptance criteria.
- `_bmad-output/implementation-artifacts/6-4-drive-initial-realistic-playback-through-a-linked-expression-map.md`
  - Existing transport-owned Expression Map proof structure and bounded scope.
- `_bmad-output/implementation-artifacts/8-4-derive-effective-playing-technique-context-for-notes.md`
  - Effective-technique derivation rules and prepared-context ownership guardrails.
- `_bmad-output/implementation-artifacts/8-5-extend-expression-maps-to-match-playing-technique-context.md`
  - Playing-technique condition family, editor ownership, and matcher guardrails.
- `docs/project-context.md`
  - Repo-wide rules for backend ownership, transport behavior, and playback-domain boundaries.
- `docs/architecture.md`
  - Backend preparation and transport architecture.
- `docs/data-models.md`
  - Expression Map and playing-technique model notes.
- `docs/development-guide.md`
  - Playback and expression-map guardrails.
- `/memories/repo/expression-map-playing-technique-story-8-5.md`
  - Story 8.5 summary and derived-context ownership reminder.
- `/memories/repo/playing-technique-effective-context-story-8-4.md`
  - Story 8.4 semantic-resolution summary and transport-boundary reminder.
- `/memories/repo/expression-map-transport-proof-story-6-4.md`
  - Story 6.4 transport-proof summary and fallback semantics.
- `src-tauri/src/playback/expression_maps.rs`
  - Rule matcher and playing-technique condition support.
- `src-tauri/src/playback/preparation.rs`
  - Prepared playback targets, prepared note events, and technique test helpers.
- `src-tauri/src/playback/transport.rs`
  - Routed transport playback path and current expression-map proof harness.
- `src-tauri/src/score/entries/playing_technique.rs`
  - Playing-technique kinds and effective-context semantics.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-09: Story prepared after reviewing Epic 8 planning, the completed 6.4, 8.4, and 8.5 story artifacts, project-context rules, recent git history, and the current playback transport, preparation, and expression-map seams.
- 2026-04-09: Current repo state already proves prepared technique context reaches Expression Map rule resolution in `preparation.rs`; Story 8.6 therefore targets transport-facing audible proof and only narrow backend fixes if the proof exposes a gap.
- 2026-04-09: Scope held to the existing backend-owned routed transport path, bounded `pizz` or `arco` proof behavior, seam-local tests, and maintained docs updates.
- 2026-04-09: Added a bounded bowed-technique proof map plus transport regressions for keyswitch selection, `pizz` to `arco` switching on the marking tick, fallback before the first marking, and scheduled-note resolution after master-track removal.
- 2026-04-09: Full validation passed with `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `npm run check-types`, and `npm run lint`.

### Completion Notes List

- Confirmed the shipped backend-owned preparation plus transport path already satisfied Story 8.6; no new playback seam or transport-owned technique state was introduced.
- Added bounded bowed-technique proof coverage in `src-tauri/src/playback/transport.rs` for technique-driven keyswitch selection, `pizz` to `arco` switching at the marking tick, fallback before the first marking, and prepared-context scheduling after master-track removal.
- Added preparation-level proof coverage in `src-tauri/src/playback/preparation.rs` showing prepared context switches routed Expression Map rule resolution from `pizz` to `arco` on the authored `arco` tick.
- Updated `docs/architecture.md`, `docs/data-models.md`, and `docs/development-guide.md` to document the shipped technique-driven routed transport proof and its bounded `pizz` or `arco` scope.
- Validation completed successfully with `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `npm run check-types`, and `npm run lint`.

### File List

- `docs/architecture.md`
- `docs/data-models.md`
- `docs/development-guide.md`
- `src-tauri/src/playback/preparation.rs`
- `src-tauri/src/playback/transport.rs`
- `_bmad-output/implementation-artifacts/8-6-prove-technique-driven-playback-through-derived-context.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Change Log

- 2026-04-09: Created Story 8.6 implementation context and updated sprint tracking to ready-for-dev.
- 2026-04-09: Added bounded technique-driven transport and preparation proof coverage, documented the shipped routed playback proof, ran full validation, and moved the story to review.
