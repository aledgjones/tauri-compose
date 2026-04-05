# Story 6.4: Drive Initial Realistic Playback Through A Linked Expression Map

Status: ready-for-dev

## Story

As a composer,
I want an initial linked expression-map behavior to affect playback,
so that routed score playback sounds more musically meaningful than a raw default trigger path.

## Acceptance Criteria

1. Given a routed instrument has an Expression Map configured through the supported MVP path, when I start transport playback of supported written material, then the linked Expression Map influences playback through the existing routed VST transport path, and at least one resolved winning expression rule per played note is supported through the defined model.
2. Given full score-linked expression sophistication exceeds MVP scope, when the initial realistic playback path is implemented, then the supported behavior stays bounded to a clear first-use case based on articulation plus note length, and it still proves that score meaning can drive playback behavior.
3. Given future realism work must build on this foundation rather than replace it, when the initial Expression Map playback path is delivered, then the implementation extends the durable asset model and existing backend-owned preparation and transport path, and it does not hard-code a one-off bypass around the model.

## Tasks / Subtasks

- [ ] Task 1: Extend the backend-owned playback preparation and transport path to resolve Expression Map rules for supported notes (AC: 1, 3)
  - [ ] Reuse the existing Rust-owned preparation and transport seams rather than creating a parallel expression-playback path.
  - [ ] Resolve the linked Expression Map for each supported routed instrument on the transport-owned playback path.
  - [ ] Evaluate rules top-down, stop at the first match, and fall through to the fixed default rule when nothing else matches.
  - [ ] Keep Expression Map playback truth backend-owned and derived from the durable linked asset model.

- [ ] Task 2: Prove the bounded first-use case through articulation plus note-length rule matching (AC: 1, 2)
  - [ ] Support the canonical proof case where short staccato notes resolve to a staccato sample path, while longer staccato notes resolve to a legato or default sample path with shortened performed duration.
  - [ ] Restrict the first audible proof to transport playback only; do not widen 6.4 into a separate audition-specific expression system.
  - [ ] Preserve the authored rule model from 6.3 so the audible proof comes from real linked Expression Map data rather than hard-coded transport exceptions.

- [ ] Task 3: Support the agreed MVP audible action set without over-claiming broader trigger coverage (AC: 1, 2, 3)
  - [ ] Prove audible keyswitch-driven sample selection on the transport path.
  - [ ] Prove audible note-length transformation on the transport path.
  - [ ] Keep CC-authored rules structurally supported by the model without requiring 6.4 to prove every authored action type audibly if the bounded first-use case does not need them.
  - [ ] When no authored rule matches, preserve the current plugin-default routed playback path through the fixed fallback rule.

- [ ] Task 4: Preserve per-instrument independence when maps are shared across routed instruments (AC: 1, 3)
  - [ ] Ensure multiple instruments can share the same Expression Map asset while still resolving different winning rules per note according to each instrument's written material.
  - [ ] Ensure one instrument's resolved winning rule does not leak into another instrument's playback state.
  - [ ] Keep the proof plugin-agnostic in product wording even if local validation uses a known-good BBC plugin setup.

- [ ] Task 5: Keep 6.4 free of premature UI scope while adding strong transport-facing proof and docs (AC: 1, 2, 3)
  - [ ] Do not add new active-expression UI, sequencer tracks, playback overlays, or extra score-context panels in this story.
  - [ ] Add backend and integration tests proving linked maps affect transport playback through the existing routed VST path.
  - [ ] Update planning and developer docs to record that the first Epic 6 audible proof is transport-only, first-match-wins, and bounded to the agreed articulation-plus-note-length case.

## Dev Notes

### Story Intent

- Story 6.4 is the first story that makes Expression Maps audibly matter.
- The purpose is not full expressive playback coverage. The purpose is a trustworthy first proof that the durable linked map model can drive routed transport playback through score meaning.
- This story must stay on the existing backend-owned routed transport path. Expression Maps extend that path; they do not become a second playback architecture.

### User Model Clarified

- The first audible proof should be obvious by ear: appropriate notation should produce a more appropriate sample behavior than the raw plugin default path.
- The canonical proof case is intentionally small but musical:
  - short staccato note -> staccato sample path
  - longer staccato note -> legato or default sample path with shortened performed duration
- Users do not need new UI in 6.4. The proof is audible, not visual.
- If no rule matches, playback should simply continue through the current plugin-default routed path.
- Shared maps remain reusable assets, but playback resolution is still independent per instrument and per played note.

### Previous Story Intelligence

- Story 6.1 established the durable model, route-address-aware linkage seam, and bounded action vocabulary.
- Story 6.2 established row-level linkage selection while keeping user-facing state intentionally narrow.
- Story 6.3 established the real authoring workflow, first-match-wins rule order, logical AND conditions, fixed fallback rule, and authored action set. Story 6.4 must consume those authored assets as-is rather than bypassing them with transport-only shortcuts.
- Epic 4 and Epic 5 already established that supported audible playback must stay on the backend-owned preparation plus transport path. 6.4 must extend that same path, not add a frontend-owned or audition-only expression engine.

### This Story Owns

- The first audible Expression Map proof on the existing routed transport path.
- Rule resolution for the bounded articulation-plus-note-length MVP case.
- Audible keyswitch-driven selection plus note-length transformation on transport playback.
- Per-instrument independence when maps are shared.
- Strong backend and transport-facing proof with no new user-facing playback-expression UI.

### This Story Does Not Own

- A second expression playback system for audition. Keep 6.4 transport-only unless later work explicitly broadens scope.
- New active-expression UI, sequencer expression lanes, debugger-first product surfaces, or additional score-context panels.
- General plugin support claims. Epic 7 still owns the supported baseline and release-credible validation language.
- Full expressive sophistication such as phrase/slur context, dynamics-as-expression logic, or broader trigger/action combinations beyond the agreed MVP proof path.
- Any hard-coded one-off mapping path that bypasses the durable asset model, linked map selection, or 6.3 rule semantics.

### Current Relevant Seams

- `src-tauri/src/playback/preparation.rs`
  - Existing canonical preparation seam that should remain the source of supported routed playback targets.
- `src-tauri/src/playback/transport.rs`
  - Existing backend-owned routed transport playback path that should consume resolved Expression Map behavior.
- `src-tauri/src/playback/engine.rs`
  - Existing render-active engine seam that transport already feeds.
- `src-tauri/src/playback/audition.rs`
  - Important non-target seam for 6.4. Do not make Epic 6's first audible proof depend on an audition-specific expression path.
- `src-tauri/src/score/playback.rs`
  - Durable linked-map and route truth that transport resolution must consume.
- `src-tauri/src/files.rs`
  - Existing persistence seam proving projects reopen with linked map references.
- `src/services/types.ts`
  - Shared frontend mirror if any new backend-facing playback debug or linkage payloads become necessary.

### Implementation Guidance

- Reuse the existing transport-owned note scheduling path and resolve expression behavior as part of the backend-owned event preparation or projection flow.
- Keep the bounded first-use case explicit. The story does not need to prove every authored rule feature audibly.
- Preserve first-match-wins semantics exactly as authored in the editor. Do not re-derive precedence in transport.
- Treat the fallback rule as the audible meaning of use-the-plugin-default-path when nothing else matches.
- Keep plugin-specific proof local to validation, not to the product contract. The user may validate against BBC plugins, but the story should remain about the generic system behavior.
- Ensure shared maps remain stateless reusable assets. Runtime resolution should be per instrument and per played note, not a mutable global expression state that leaks between instruments.

### Risks And Regression Traps

- The largest architectural trap is bypassing the durable asset model and hard-coding a one-off staccato transport branch just to prove the concept.
- Another trap is widening 6.4 into an audition path because audition already exists nearby. The user explicitly bounded this proof to transport.
- A third trap is turning rule resolution into shared mutable instrument state that can leak between instruments using the same map.
- It is also easy to over-claim support. 6.4 should prove one bounded musical case well rather than pretending the whole expression system is already general-purpose.
- Note-length transformation must stay musically bounded and deterministic. Do not let it destabilize transport timing ownership or queued-event correctness.

### Testing Requirements

- Add backend and/or integration tests for:
  - first-match-wins resolution on the transport-owned playback path;
  - the canonical staccato-plus-note-length proof case;
  - fallback behavior when no authored rule matches;
  - per-instrument independence when two instruments share one map but need different winning rules;
  - transport playback remaining on the existing supported routed path rather than creating a second expression-specific path.
- If useful, add seam-local tests near rule projection or event preparation rather than creating a cross-cutting expression-only harness.
- Recommended validation commands once implemented:
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `npm run check-types`
  - `npm run lint`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

### Documentation Impact

- Update planning and developer docs so Epic 6 no longer implies broader state taxonomies, one-window expression editing, or audition-first expression proof.
- Likely documentation touchpoints:
  - `_bmad-output/planning-artifacts/epics.md`
  - `_bmad-output/planning-artifacts/prd.md`
  - `_bmad-output/planning-artifacts/ux-design-specification.md`
  - `docs/development-guide.md`
- Document explicitly that:
  - the first audible proof is transport-only;
  - the bounded proof case is articulation plus note length;
  - first-match-wins rule order remains authoritative in playback;
  - no active-expression UI is part of MVP Epic 6.

### Architecture Compliance

- Backend transport ownership remains intact. Expression Maps extend the current preparation plus transport pipeline rather than replacing it.
- The rack-first model remains intact. Expression behavior still depends on route-linked map selection and existing supported routed VST targets.
- Durable map assets remain the only source of playback rule truth; transport must not own a parallel expression definition.
- UI remains out of the critical playback loop. No frontend-owned active-expression state belongs in this story.

### Library And Framework Requirements

- Stay on the repo baselines already established in the project context: Rust 2021, Tauri 2.10.1, React 19.2.4, and TypeScript 5.9.3.
- Reuse the existing backend preparation, engine, and transport seams that already own routed transport playback.

### File Structure Notes

- Audible Expression Map behavior belongs under the existing backend playback seams in `src-tauri/src/playback/` and adjacent score playback structures.
- Keep tests seam-local to transport, preparation, and durable playback model owners.
- Do not create a new frontend window or panel in 6.4.

### Git Intelligence Summary

- Existing Epic 4 and Epic 5 work already established that routed audible playback should flow through backend-owned preparation plus transport seams. 6.4 should continue that discipline instead of widening into a second playback architecture.
- The current codebase already keeps audition and transport related but distinct. 6.4 should preserve that distinction and keep the first expression proof on transport only.

### Latest Technical Information

- The authored Expression Map model is broader than the first audible proof required here. 6.4 should prove the minimum musical case cleanly instead of overcommitting the whole action set.
- Plugin-specific libraries may validate the concept locally, but product wording should remain generic until Epic 7 formalizes the supported baseline.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 6 and Story 6.4 baseline direction.
- `_bmad-output/implementation-artifacts/6-1-model-expression-maps-as-durable-playback-assets.md`
  - Durable model, bounded action set, and route-linkage foundation.
- `_bmad-output/implementation-artifacts/6-2-show-expression-map-linkage-in-score-context.md`
  - Row linkage UX and quiet unassigned-state contract.
- `_bmad-output/implementation-artifacts/6-3-create-one-dedicated-expression-map-editor-window.md`
  - Authoring workflow, first-match-wins semantics, and fixed fallback rule.
- `docs/development-guide.md`
  - Existing transport-owned playback path and Epic 6 extension boundary.
- `/memories/repo/epic4-playback-preparation-story-4-1.md`
  - Preparation and transport seam ownership guidance.
- `/memories/repo/playback-rack-first-model.md`
  - Rack-first route ownership constraints.
- `src-tauri/src/playback/preparation.rs`
  - Existing supported routed playback snapshot seam.
- `src-tauri/src/playback/transport.rs`
  - Existing routed transport playback path.
- `src-tauri/src/playback/audition.rs`
  - Existing non-target audition seam.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-05: Story prepared after reviewing Epic 6 planning, Stories 6.1 through 6.3, the current backend preparation and transport seams, and repo guidance that Epic 6 must extend the transport-owned audible path rather than replace it.
- 2026-04-05: User clarification locked the 6.4 proof shape: transport-only, first-match-wins per played note, canonical staccato-plus-note-length proof case, keyswitch plus note-length transformation as the audible action set, no new UI, and per-instrument independence even when maps are shared.

### Completion Notes List

- Story context created and sprint status updated to ready-for-dev.
- Story 6.4 stays intentionally bounded: one musical proof case on transport rather than full expressive playback scope.
- Higher-level planning docs were updated to match the tighter Epic 6 story sequence and the row/library/editor/window decisions already made in this session.

### File List

- `_bmad-output/implementation-artifacts/6-4-drive-initial-realistic-playback-through-a-linked-expression-map.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/planning-artifacts/epics.md`
- `_bmad-output/planning-artifacts/prd.md`
- `_bmad-output/planning-artifacts/ux-design-specification.md`
- `docs/development-guide.md`
