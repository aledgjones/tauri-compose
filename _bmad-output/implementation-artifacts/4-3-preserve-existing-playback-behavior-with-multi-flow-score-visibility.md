# Story 4.3: Preserve Existing Playback Behavior with Multi-Flow Score Visibility

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want existing playback behavior to keep working while multiple flows are visible in score view,
so that multi-flow reading does not break the current listening workflow.

This story keeps playback deliberately conservative. Score view may show multiple visible flows, but the current playback model remains scoped to one active flow. The implementation must preserve that rule across transport state, keyboard-accessible controls, tempo synchronization, and score visibility without implying that MVP multi-flow score view is also multi-flow playback.

## Acceptance Criteria

1. Given multiple flows are visible in the score view, when a user uses the existing playback workflow, then playback continues to behave according to the current product model and the transport continues to play the current single active flow while other visible flows remain visible for reading.
2. Given the app uses a single active flow for current playback-related workflows, when the score view shows multiple visible flows, then playback remains scoped according to the existing active-flow model and the UI communicates that playback scope clearly enough that the user can predict which flow will sound.
3. Given a single-flow score and the existing playback workflow, when playback is exercised after the multi-flow changes, then current playback behavior remains correct and the same transport controls and playback cues remain usable in the single-flow case.
4. Given focus is on the transport controls while multiple flows are visible, when the user starts or stops playback through the existing keyboard-accessible transport workflow, then focus remains stable on the transport control surface and the active playback scope remains clear while the score continues to show multiple flows.
5. Given this playback-preservation increment is validated, when multi-flow score view and playback are exercised together, then the feature remains compatible with current audio-device-based playback expectations and the playback UI continues to communicate the single active playback scope clearly while multiple flows remain visible in score view.

## Tasks / Subtasks

- [ ] Audit the current active-flow and transport seams before changing behavior (AC: 1, 2, 3, 4, 5)
  - [ ] Inspect `src-tauri/src/playback/transport.rs`, `src/services/state/provider.tsx`, `src/services/actions.ts`, and the transport-related main-window components to confirm how playback scope is currently derived and surfaced.
  - [ ] Confirm that `ui.flow_id` remains the single authoritative active-flow slot for playback-relevant workflows.
  - [ ] Identify any current UI cues that imply playback scope so the story can preserve or improve them without inventing a second playback-state model.
- [ ] Preserve single-active-flow playback while score view is multi-flow-visible (AC: 1, 2, 3, 5)
  - [ ] Keep transport playback scoped to the current active flow rather than the full visible multi-flow sequence.
  - [ ] Ensure multi-flow score visibility does not cause the transport to infer or synthesize multi-flow playback behavior.
  - [ ] Preserve the current single-flow behavior exactly for projects with only one flow.
- [ ] Keep playback state and UI communication coherent (AC: 2, 4, 5)
  - [ ] Preserve the current transport polling model and active-flow-derived tempo synchronization.
  - [ ] Ensure focus remains stable on transport controls during keyboard-driven playback start/stop while the score continues to show multiple flows.
  - [ ] If UI clarification is needed, keep it minimal and aligned with existing shell patterns rather than adding a new top-level playback mode switch.
- [ ] Add focused regression coverage around transport and active-flow coherence (AC: 1, 2, 3, 4, 5)
  - [ ] Extend backend tests around `src-tauri/src/playback/transport.rs` if active-flow resolution or tempo-snapshot behavior changes.
  - [ ] Extend frontend provider, transport, and playback-line tests only where needed to prove playback state remains coherent with multi-flow visibility.
  - [ ] Add manual verification notes covering play, pause, stop, seek, keyboard transport control, and active-flow changes while multiple flows are visible.
- [ ] Update docs only if playback behavior or maintainer expectations become more explicit (AC: 5)
  - [ ] Update `docs/project-context.md`, `docs/architecture.md`, and `TESTING.md` if implementation clarifies the stable rule that score view may be multi-flow-visible while playback remains single-active-flow scoped.

## Dev Notes

### Story Foundation

- The epic explicitly keeps cross-flow playback out of MVP scope. Story 4.3 is therefore a preservation story, not a playback-expansion story.
- Epic 2 already established that `ui.flow_id` is the shared active-flow concept across score, sequence, and transport-facing consumers. Story 4.3 should preserve that model while multi-flow score view remains visible.
- The desired user experience is simple: users can read multiple flows in score view while playback still behaves exactly as the app currently does for one active flow.

### Developer Context Summary

- `src-tauri/src/playback/transport.rs` already derives playback scope from the active flow, falling back to the first flow when needed.
- `src/services/state/provider.tsx` keeps transport state synchronized through a `requestAnimationFrame` polling loop and re-queries transport state when active-flow-derived tempo inputs change.
- `src/services/actions.ts` owns the authoritative FE-to-BE transport commands: `transport_play`, `transport_pause`, `transport_stop`, `transport_seek`, and `transport_get_state`.
- `src/windows/main/components/playback-line.tsx` and the main-window transport components are current UI seams where playback state is made visible; preserve those cues rather than inventing a second playback indicator for multi-flow score view.
- `docs/architecture.md` already documents that playback tempo is derived from the active flow's master-track tempo data and that active playback uses a play-start score snapshot. Story 4.3 should not violate that contract.

### Technical Requirements

- Keep playback scoped to the single active flow even while multiple flows are visible in score view.
- Preserve `ui.flow_id` as the one authoritative active-flow slot that transport-related consumers resolve against.
- Preserve the current transport polling model; do not add event-based transport updates during playback.
- Preserve stopped-state tempo reporting, play/resume tempo snapshot behavior, and active-flow-derived tempo sync.
- Preserve keyboard-accessible transport behavior and stable focus on the transport control surface.
- Do not introduce a new top-level multi-flow playback mode, playlist concept, or flow-sequence transport behavior in this story.
- Preserve single-flow playback behavior without regression.
- No persistence or migration change is expected.

### Architecture Compliance

- Score visibility and playback scope remain distinct concerns.
- Rust remains the source of truth for transport state and active-flow resolution.
- React remains a consumer and presenter of transport state, not the owner of playback scope.
- Multi-flow score view remains a presentation enhancement over the existing score model and transport pipeline.

### Library / Framework Requirements

- Reuse the current Tauri command boundary and transport polling pattern.
- Preserve `snake_case` command names and typed frontend transport state.
- Keep playback UI work inside the existing transport and state-provider seams rather than adding new global state layers.
- External web research was not available in this environment; repository docs and source were used as the authoritative baseline.

### Project Structure Notes

- Primary backend playback seams:
  - `src-tauri/src/playback/transport.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/ui/mod.rs`
- Primary frontend playback seams:
  - `src/services/actions.ts`
  - `src/services/state/provider.tsx`
  - `src/services/types.ts`
  - `src/windows/main/components/transport.tsx`
  - `src/windows/main/components/playback-line.tsx`
- Supporting workflow seams:
  - `src/windows/main/index.tsx`
  - `src/windows/main/screens/sequence/components/instrument-overview.tsx`

### File Structure Requirements

- Extend the existing transport backend and provider-driven frontend transport state before adding any new playback abstraction.
- Keep playback-scope clarification in existing transport or workflow-local UI surfaces; do not add new persistent shared shell chrome for this story.
- Avoid touching persistence or renderer composition code unless a true playback regression requires a narrowly targeted fix.

### Testing Requirements

- Add or extend backend tests around active-flow resolution in `src-tauri/src/playback/transport.rs` if that logic changes.
- Add or extend provider and transport component tests to prove transport state stays coherent when multiple flows are visible.
- Extend playback-line or related UI tests only if playback cues change materially.
- Manually verify play, pause, stop, seek, and keyboard transport actions while multiple flows are visible, including active-flow changes between flows.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 4.1 protects persisted document behavior and Story 4.2 protects the surrounding desktop shell. Story 4.3 should preserve playback with the same mindset: keep the current contract intact while multi-flow visibility exists around it.
- Epic 2's active-flow work is directly relevant here. Playback should continue to follow the shared active flow instead of introducing its own transport-only flow concept.

### Git Intelligence Summary

- `ae16e160 chore: refactor render trigger API` suggests render behavior is still moving, which makes it even more important not to accidentally couple transport semantics to visible-flow composition.
- The recent Epic 3 commits stayed disciplined about not broadening scope. Story 4.3 should keep that discipline by preserving single-active-flow playback rather than taking on cross-flow audio behavior.

### Project Context Reference

- `docs/project-context.md` says `ui.flow_id` is the one authoritative active-flow slot for score, Sequence, miniature piano rolls, and transport-derived tempo state.
- `docs/project-context.md` and `docs/architecture.md` both require transport polling through `requestAnimationFrame` rather than event emissions during playback.
- `_bmad-output/planning-artifacts/ux-design-specification.md` and `_bmad-output/planning-artifacts/epics.md` both support keeping playback coherent and readable without making multi-flow visibility imply multi-flow playback.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 4, Story 4.3)
- Source: `docs/project-context.md` (active-flow and transport polling rules)
- Source: `docs/architecture.md` (tempo resolution and playback snapshot behavior)
- Source: `src-tauri/src/playback/transport.rs` (active-flow resolution and playback state)
- Source: `src/services/state/provider.tsx` (transport polling and active-flow-derived tempo sync)
- Source: `src/services/actions.ts` (transport commands)
- Source: `src/windows/main/components/playback-line.tsx` (existing playback cue)
- Source: `src/windows/main/screens/sequence/components/instrument-overview.tsx` (active-flow-driven single-flow workflow surface)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Completion Notes List

- Story 4.3 was created from the Epic 4 playback-preservation scope and the repo's current transport/active-flow architecture.
- The story explicitly rejects cross-flow playback expansion for this MVP slice and treats any such work as out of scope.
- The repository does not contain `_bmad/core/tasks/validate-workflow.xml`, so the BMAD validator could not be invoked; the create-story checklist was applied manually while preparing this artifact.
- External web research was not available in this environment; repository docs and source were used as the authoritative baseline.

### File List

- `_bmad-output/implementation-artifacts/4-3-preserve-existing-playback-behavior-with-multi-flow-score-visibility.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
