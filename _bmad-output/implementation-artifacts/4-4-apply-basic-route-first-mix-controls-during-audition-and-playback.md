# Story 4.4: Apply Basic Route-First Mix Controls During Audition And Playback

Status: ready-for-dev

## Story

As a composer,
I want basic route-first mix controls to affect audible output,
so that I can make bounded playback adjustments within Compose while keeping the user model tied to routes.

## Acceptance Criteria

1. Given a routed instrument participates in auditioning or supported score playback, when I adjust a supported basic mix control in Compose, then the control affects audible output according to the real route capability exposed by the active VST path, and the user-facing control remains route-first in meaning.
2. Given some control semantics may still cross slot boundaries internally, when mix behavior is implemented, then the product preserves the route-first user model, and any required slot-level mechanics remain behind the scenes.
3. Given a requested mix control is not truly supported by the active route capability, when the playback surface is shown, then Compose does not pretend the control is available, and the bounded MVP mixer surface remains aligned with actual supported behavior.

## Tasks / Subtasks

- [ ] Task 1: Define the bounded backend-owned mix-capability seam for current direct-plugin routes (AC: 1, 2, 3)
  - [ ] Start from `src-tauri/src/score/instruments/mod.rs`, `src-tauri/src/playback/transport.rs`, `src-tauri/src/playback/preparation.rs`, and `src-tauri/src/playback/runtime_activation.rs` instead of inventing a frontend-owned capability model.
  - [ ] Reuse the persisted instrument mix fields (`volume`, `pan`, `mute`, `solo`) as the score-owned intent source while making backend truth explicit about which of those controls are actually honored by the current route path.
  - [ ] Keep the MVP bounded to the current single-target direct-plugin route model; do not broaden this story into multi-target, multi-slot, or multi-timbral semantics.
  - [ ] Keep route-first naming in the product surface even if live implementation still resolves through slot-owned mechanics.
- [ ] Task 2: Make supported mix controls affect both audition and transport-backed audible output (AC: 1, 2)
  - [ ] Ensure the current continuous engine path in `src-tauri/src/playback/engine.rs` and the transport path consume consistent backend-owned mix parameters rather than letting audition and transport drift.
  - [ ] Preserve the current transport trust gate in `prepare_supported_playback(...)`; Story 4.4 must not widen transport beyond one supported routed instrument.
  - [ ] Keep audition able to target a ready route directly, while still applying the same supported mix behavior the user set in score context.
  - [ ] Do not move output-stream lifetime, engine ownership, or canonical live plugin processing into transport or React while wiring mix behavior through the active playback path.
- [ ] Task 3: Surface only honest, compact controls in the existing score-context UI (AC: 1, 2, 3)
  - [ ] Start from the current Sequence score-context surfaces in `src/windows/main/components/mixer.tsx`, `src/windows/main/components/mixer-channel.tsx`, and `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`.
  - [ ] Treat the currently surfaced controls as the default bounded set: volume, mute, and solo should remain available only if they work end-to-end on the active route; pan should stay hidden until it is truly supported through the same honest path.
  - [ ] Keep readiness and capability presentation compact; do not add a DAW-style mixer, plugin-manager-first workflow, or explanatory panel whose main purpose is to prove acceptance criteria literally.
  - [ ] Preserve the current warning semantics: plain unrouted `--` and empty-slot `N. --` remain non-warning states, while broken assigned plugin routes keep the compact warning treatment.
- [ ] Task 4: Add seam-local regression coverage for mix behavior and capability honesty (AC: 1, 2, 3)
  - [ ] Extend Rust tests near `src-tauri/src/playback/transport.rs`, `src-tauri/src/playback/audition.rs`, `src-tauri/src/playback/engine.rs`, and `src-tauri/src/score/instruments/mod.rs` to prove supported controls affect audible output and unsupported controls are not falsely exposed.
  - [ ] Add or extend frontend tests around `src/services/actions.test.ts`, `src/windows/main/components/mixer.test.tsx`, `src/windows/main/components/mixer-channel.test.tsx`, and `src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx` so UI interactions stay aligned with backend-owned route capability and readiness truth.
  - [ ] Cover the intended asymmetry explicitly: a ready route may still audition even when transport is blocked by the single-instrument trust gate, but the mix-control surface must remain honest about the current transport boundary.
  - [ ] Validate that mix changes during active playback or immediate re-audition do not create crashes, stale engine state, or frontend-owned caches of playback truth.
- [ ] Task 5: Update docs and agent guidance when the bounded route-first mix contract becomes concrete (AC: 1, 2, 3)
  - [ ] Update `docs/architecture.md` and `docs/development-guide.md` if Story 4.4 sharpens where mix capability is derived and how the engine versus transport paths consume mix parameters.
  - [ ] Update `docs/project-context.md` and `.github/copilot-instructions.md` if the story adds a durable rule about which score-context controls are honest for the current MVP and how unsupported controls must remain hidden.

## Dev Notes

### Story Intent

- Story 4.4 is the first bounded mix-control slice on top of Stories 4.1 through 4.3.
- The product goal is not “build a mixer.” The product goal is to let a composer make a few honest playback adjustments inside Compose without breaking the route-first mental model or inventing fake capability.
- This story must keep the app score-first, rack-first, and trust-first. If a control is not actually supported through the active direct-plugin route, the correct behavior is to not present it as usable.

### Epic And Story Context

- Epic 4 is proving first audible playback inside the core score workflow, not turning Compose into a DAW-like host surface.
- Story 4.1 established the canonical backend playback-preparation seam.
- Story 4.2 established the bounded score-context audition contract.
- Story 4.3 made the continuous render engine concrete so audition, mounted plugin UI interaction, and later transport work share one trustworthy playback-plane seam.
- Story 4.5 follows this story and should refine the compact score-context workflow around these controls rather than reinventing playback surfaces.

### Previous Story Intelligence

- Story 4.3 confirmed that the concrete playback-plane seam now lives in `src-tauri/src/playback/engine.rs`; new live mix behavior should be expressed through that engine path, not hidden in React or re-buried in `transport.rs`.
- Story 4.3 also preserved the explicit split between ready-route audition and the narrower single-instrument transport trust gate. Story 4.4 must not blur those two behaviors into one misleading UX model.
- Story 4.2 and 4.3 both reinforced that explicit `Unrouted` and `NeedsAttention` route states remain backend-owned and honest; mix work must not silently treat unresolved routes as partially playable.

### This Story Owns

- Defining the bounded backend-owned route-capability answer for current mix controls.
- Applying supported mix parameters consistently to the active audible paths used by audition and transport-backed playback.
- Keeping the user model route-first even when backend mechanics still resolve through slot-owned runtime sessions.
- Ensuring the score-context mixer surface is honest about what the current route can do.

### This Story Does Not Own

- Multi-route transport playback, multi-slot balancing, or a generalized mixer architecture.
- DAW-style meters, buses, inserts, sends, automation, or persistent plugin-level mixing UX.
- A transport redesign, transport trust expansion, or broader play/pause/resume/seek work that belongs to Epic 5.
- Inventing a frontend-owned route capability cache, fallback readiness model, or plugin-only proof path.

### Architecture Compliance

- Rust remains the only owner of route truth, playback preparation, playback activation, transport state, and engine-owned runtime playback behavior.
- The architecture must stay rack-first: racks own slot lifecycle, slots own plugin assignment or unassignment, and instruments only own routes to existing slots.
- Keep backend playback truth and user-facing warning chrome separate. The backend may distinguish multiple non-ready states, while the score-context surface stays intentionally compact.
- The current MVP remains single-target-first for transport. Story 4.4 must not accidentally widen `prepare_supported_playback(...)` beyond one supported direct-plugin target.
- Keep stream/device lifetime in `src-tauri/src/playback/audio.rs` and render-active ownership in `src-tauri/src/playback/engine.rs`.

### Current Relevant Seams

- `src-tauri/src/score/instruments/mod.rs`
  - Owns the existing score mutation commands for `volume`, `pan`, `mute`, and `solo`, and already mirrors live mix state into transport-shared data.
- `src-tauri/src/playback/transport.rs`
  - Owns the current `InstrumentMixParams` live transport snapshot and the transport trust boundary.
- `src-tauri/src/playback/engine.rs`
  - Owns the concrete continuous render path that Story 4.4 must use for live audible mix behavior.
- `src-tauri/src/playback/audition.rs`
  - Owns the bounded score-context audition path and must not drift away from the same supported mix semantics.
- `src-tauri/src/playback/preparation.rs`
  - Owns the canonical backend playback-preparation seam and the current single-target trust gate.
- `src-tauri/src/playback/runtime_activation.rs`
  - Owns backend activation truth and is the strongest existing basis for a bounded route-capability answer.
- `src/windows/main/components/mixer.tsx`
  - Owns the current score-context mixer surface and should remain compact.
- `src/windows/main/components/mixer-channel.tsx`
  - Owns the currently exposed channel controls and is the right place to keep or refine only those controls that are truly supported.
- `src/windows/main/screens/sequence/components/instrument-overview-row.tsx`
  - Owns route-context UI and current readiness presentation.
- `src/windows/main/screens/sequence/components/playback-readiness.tsx`
  - Owns the compact readiness badge semantics; Story 4.4 should preserve its bounded warning model.

### Implementation Guidance

- Prefer a small backend-owned capability answer over implicit UI assumptions. If the repo does not yet have a dedicated “supported mix controls” type, add one where Rust remains authoritative rather than deriving it in React.
- Preserve the current route-first mental model in user language. A composer should still think “this instrument route supports these adjustments,” not “this slot host instance exposes these internal controls.”
- Keep the bounded set deliberately small. The current UI already supports volume, mute, and solo; pan exists in persisted state but is not yet surfaced in the current mixer UI. Only expose pan if Story 4.4 makes it end-to-end honest.
- Keep transport and audition behavior aligned where they overlap, but do not force them into false equivalence. The story should preserve that audition can work on a ready route while transport start still obeys the stricter single-instrument trust gate.
- Do not add a second playback topology, second readiness resolver, or second mix state owner in React, tests, or docs.

### Risks And Regression Traps

- Letting audition ignore mix parameters while transport honors them would create silent behavioral drift and user confusion.
- Treating persisted instrument mix fields as proof of supported live behavior would violate AC3 if the active route path cannot actually honor them.
- Surfacing pan just because the backend command exists, without proving end-to-end audible behavior, would create fake capability.
- Broadening Story 4.4 into a full mixer redesign would violate the score-first and compact-workflow constraints.
- Using transport gating as the only source of truth for mix availability would incorrectly hide valid ready-route audition behavior.

### Testing Requirements

- Add backend coverage proving:
  - supported route-first mix parameters affect audition through the continuous engine;
  - supported route-first mix parameters affect transport-backed playback without widening the trust gate;
  - unsupported controls are not exposed as supported capability for the current route path;
  - unresolved routes remain explicit `Unrouted` or `NeedsAttention` instead of silently accepting mix changes as live playback proof.
- Add frontend coverage proving:
  - the current score-context mixer surface only presents controls the backend-capability answer supports;
  - volume, mute, and solo interactions call the existing backend-owned actions and reflect backend state updates;
  - warning chrome remains limited to genuinely broken assigned plugin routes, not plain unrouted or empty-slot states;
  - any UI messaging around transport-versus-audition behavior stays compact and does not invent extra proof-only surfaces.
- Validation commands for implementation should include:
  - `npm run test:js -- src/services/actions.test.ts src/windows/main/components/mixer.test.tsx src/windows/main/components/mixer-channel.test.tsx src/windows/main/screens/sequence/components/instrument-overview-row.test.tsx`
  - `npm run lint`
  - `npm run check-types`
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`

### Documentation Impact

- Update docs in the same implementation change if Story 4.4 makes the bounded route-capability and live mix-consumption contract explicit.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if new agent-facing honesty rules for compact mix controls become durable project policy

### Git Intelligence Summary

- Recent Epic 4 sequence confirms the current implementation baseline:
  - `3fe6a7e4 feat(4.3): audio playback`
  - `258a2775 chore: update plan to fix audtion holistically`
  - `3ce3e57f feat(4.2): auditioning`
  - `a98b6cc4 feat(4.1): playback prep`
- Story 4.4 should extend this existing engine-plus-preparation path rather than introducing a new playback lane.

### Latest Technical Information

- Repo baseline remains aligned with current external guidance:
  - Tauri 2 command guidance still supports unique commands defined in separate modules and registered once through `tauri::generate_handler!`, with `rename_all = "snake_case"` preserving the repo’s frontend argument shape.
  - CPAL continues to model playback through host/device/stream selection and a high-priority output callback fed by a `StreamConfig`, which matches the repo’s current `AudioContext` ownership model.
- No library upgrade, alternate host stack, or frontend-owned event channel is required for Story 4.4.

### Project Structure Notes

- Backend mix behavior should stay under `src-tauri/src/playback/` and score-owned mutation seams under `src-tauri/src/score/`.
- Frontend changes should remain limited to existing score-context surfaces under `src/windows/main/` and the command bridge in `src/services/`.
- Do not add new windows, new app-scoped mixer workflows, or a second route-management surface for this story.

### References

- `_bmad-output/planning-artifacts/epics.md` - Epic 4 and Story 4.4 acceptance criteria.
- `_bmad-output/planning-artifacts/architecture.md` - route-first ownership, compact score-context UX boundaries, and playback-plane guardrails.
- `_bmad-output/planning-artifacts/ux-design-specification.md` - score-first playback, direct routing, progressive disclosure, and anti-DAW guidance.
- `_bmad-output/planning-artifacts/prd.md` - Epic 4 first-audible goals and bounded MVP framing.
- `docs/project-context.md` - backend-owned playback truth, slot-owned sessions, and compact warning semantics.
- `docs/architecture.md` - canonical playback preparation seam, continuous engine, and transport boundary.
- `docs/development-guide.md` - validation commands and transport/audition implementation rules.
- `/memories/repo/playback-rack-first-model.md` - rack-first slot ownership and empty-slot readiness semantics.
- `/memories/repo/playback-readiness-ui-semantics.md` - compact warning UI rules for `--`, `N. --`, and broken assigned plugin routes.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story creation analysis covered Epic 4 acceptance criteria, current transport and engine ownership, existing score-context mixer and route UI seams, route-readiness UI rules, and recent Epic 4 commit sequence.

### Completion Notes List

- Created Story 4.4 as an implementation-ready guide for bounded route-first mix behavior on top of the current preparation and continuous-engine seams.

### File List

- `_bmad-output/implementation-artifacts/4-4-apply-basic-route-first-mix-controls-during-audition-and-playback.md`
