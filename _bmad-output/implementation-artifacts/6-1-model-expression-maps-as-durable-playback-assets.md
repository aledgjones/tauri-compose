# Story 6.1: Model Expression Maps As Durable Playback Assets

Status: ready-for-dev

## Story

As a composer,
I want expression maps to exist as durable reusable playback assets,
so that playback realism is based on a coherent model rather than transient per-selection settings.

## Acceptance Criteria

1. Given expression behavior is part of playback realism, when Compose introduces the initial expression-map capability, then expression maps are modeled as durable reusable assets rather than transient UI-only state, and the model preserves a path to richer score-linked behavior later.
2. Given score-context panels and app-scoped windows have different responsibilities, when expression-map architecture is surfaced in the product, then score-context surfaces show route linkage and current status, and the expression-map asset itself is owned outside transient selection context.
3. Given the MVP needs a bounded first implementation, when the initial model is defined, then it supports a constrained first-use path, and it does not lock the product into a manual-only dead end.

## Tasks / Subtasks

- [ ] Task 1: Add a durable backend-owned expression-map domain model under the existing score and playback seams (AC: 1, 3)
  - [ ] Define a reusable expression-map asset with stable ID, editable display name, and zero-or-more rule entries.
  - [ ] Define each rule as conditions plus actions so the model can stay generic rather than plugin-branded.
  - [ ] Keep the initial condition model ready for at least articulation and written-note-length inputs without promising broader score semantics yet.
  - [ ] Keep the initial action model ready for at least keyswitch-note, CC-message, no-op fallback, and note-length transformation outputs without promising arbitrary plugin-trigger breadth.

- [ ] Task 2: Anchor linkage to the future-proof playback-address seam while keeping current UX simple (AC: 1, 2, 3)
  - [ ] Extend the route-owned playback data model so an expression-map reference can be associated with the rack slot address concept, not only a transient selected instrument view.
  - [ ] Preserve the current MVP mental model that users effectively experience this as the route's Expression Map until multi-address plugin workflows are shipped.
  - [ ] Keep the data contract future-proof for multi-timbral plugins without claiming multi-instrument playback support in this story.

- [ ] Task 3: Persist project intent by reference only, while keeping the reusable library app-scoped (AC: 1, 2, 3)
  - [ ] Persist only stable expression-map references inside `.soloc` project data; do not embed the full reusable map definition in project files.
  - [ ] Keep the reusable map library app-scoped and machine-local for MVP, with a clean seam for later sharing, import, and export workflows.
  - [ ] Ensure a missing map reference clears to truthful unassigned behavior rather than preserving misleading UI state.
  - [ ] Preserve the current default playback path when no expression map is assigned or when an assigned map has zero rules.

- [ ] Task 4: Establish backend and frontend contract seams for later Epic 6 stories without over-scoping 6.1 (AC: 2, 3)
  - [ ] Add or extend Rust and TypeScript shared types so Story 6.2 can display route linkage honestly and Story 6.3 can own CRUD and library editing without reworking the persistence model.
  - [ ] Reuse the existing app-scoped window pattern and score-context relationship model as the intended architecture, but do not add a minimal trust UI shell in this story.
  - [ ] Keep Story 6.1 foundational only: no dedicated editor, no library-management workflow, and no audible playback changes yet.

- [ ] Task 5: Add seam-local proof and documentation for the new persistence and ownership contract (AC: 1, 2, 3)
  - [ ] Add backend tests proving expression maps serialize durably, project files store only references, and route linkage survives load and save correctly.
  - [ ] Add regressions proving missing-map references clear to truthful unassigned state and empty maps fall back to current default playback semantics.
  - [ ] Update architecture and persistence documentation to record the app-scoped library ownership, project-level reference persistence, and future-proof slot-address linkage model.

## Dev Notes

### Story Intent

- Story 6.1 is intentionally foundational. It should establish the durable model and persistence contract for Expression Maps without pulling CRUD UI, editor workflows, or audible behavior forward.
- The user-facing product promise at this stage is architectural, not experiential: Compose gains a trustworthy reusable Expression Map model that later stories can surface, edit, and use for playback.
- Trust for 6.1 is maintainer-level proof through stable data structures, load/save truth, and extension-ready seams rather than through a user-visible MVP shell.

### User Model Clarified

- An Expression Map is a reusable plugin-specific mapping profile in practical use, but the model itself stays generic: it translates score semantics into playback actions and does not intrinsically declare which plugin or instrument it belongs to.
- Users identify maps by user-provided map name. The content of the map makes it suitable for a plugin or patch; the system should not hardcode that ownership into the MVP model.
- The reusable map library is app-scoped and machine-local for MVP. Projects link to maps by stable reference only. Import/export and browsing are expected later, but 6.1 should only create the durable foundation they rely on.
- Linkage belongs conceptually to the playback route's slot address. For MVP UX this can still be surfaced later as the route's Expression Map, but the data model must already allow future multi-address plugin scenarios.
- Missing-map UX should stay quiet and truthful. If a referenced map no longer exists, later UI should simply show `--` rather than elaborate warning chrome.
- Empty maps are valid. They should fall back to the existing default playback path until later stories make richer behavior audible.

### Previous Story Intelligence

- Story 5.5 reinforced the repo rule that portable project intent lives in `.soloc`, while machine-local readiness and availability stay outside project truth. Story 6.1 should reuse that same split for reusable Expression Maps.
- Story 5.5 also confirmed that compact score-context surfaces must stay narrower than backend truth. Story 6.1 should preserve that discipline by defining the route-linkage seam now but leaving detailed surface behavior to Story 6.2.
- Recent Epic 5 work stayed concentrated in backend playback seams plus narrow frontend contracts. Story 6.1 should follow the same pattern and avoid introducing a premature editor window or debugging UI just to prove the model exists.

### This Story Owns

- A first-class durable Expression Map asset model with stable IDs, editable names, and generic rules composed from conditions plus actions.
- A persistence contract where project files store map references only, while reusable map definitions live outside project payloads.
- A future-proof linkage contract tied to route slot address rather than transient selection state.
- The initial condition and action vocabulary needed to support later articulation and note-length-driven playback behavior without overcommitting to broader score semantics yet.
- Seam-local regression proof and documentation for the new ownership and persistence model.

### This Story Does Not Own

- A dedicated Expression Map editor window, library browser, create/rename/duplicate/delete surface, or import/export product workflow. Those belong with Story 6.3 unless the epic is later re-cut.
- Score-context linkage presentation. Story 6.2 owns the user-facing route linkage surface.
- Audible playback behavior changes. Story 6.4 owns the first real playback effect.
- Automatic plugin detection, default-map auto-assignment, factory map bundles, or broad plugin-compatibility promises.
- A broad all-valid-VST3-triggers contract. MVP should stay bounded to proven actions such as keyswitch and CC, with room to extend later.
- Multi-instrument or multi-timbral playback support itself. The model must allow the concept, but this story must not claim the feature exists.

### Current Relevant Seams

- `src-tauri/src/score/playback.rs`
  - Current home of persisted playback rack, slot, route target, and route address data. This is the most likely anchor for future-proof Expression Map linkage.
- `src-tauri/src/score/mod.rs`
  - Owns the top-level score model that persists playback intent into `.soloc`.
- `src-tauri/src/files.rs`
  - Owns save/load behavior and should prove project files keep stable map references without embedding full reusable assets.
- `src-tauri/src/playback/preparation.rs`
  - Likely future consumer of linked Expression Map state once Story 6.4 begins affecting playback.
- `src-tauri/src/playback/transport.rs`
  - Current playback path still uses the existing default behavior today; Story 6.1 should not alter that path yet.
- `src/services/types.ts`
  - Must stay aligned with any new Rust payloads or enum additions.
- `src/services/actions.ts`
  - Contains the existing app-scoped window-opening pattern that Story 6.3 should reuse later, not recreate.
- `src/windows/plugin-manager/`
  - Current reference pattern for an app-scoped reusable asset window.

### Implementation Guidance

- Start with the domain contract, not the UI. The most important decision is the durable model shape: `ExpressionMap` with stable identity plus zero-or-more `ExpressionRule` entries, where each rule contains conditions and actions.
- Keep the initial condition vocabulary deliberately small but real:
  - articulation marking
  - written note length
- Keep the initial action vocabulary deliberately bounded but useful:
  - keyswitch note
  - CC message
  - no-op fallback
  - note-length transformation
- Do not encode plugin identity as a required first-class field in the durable model. Users choose the correct map manually for the chosen route in MVP.
- Do not persist entire map definitions into `.soloc`. The project should only remember which stable map ID is linked to the route-address concept.
- If a project references a map that no longer exists, clear the resolved assignment truthfully instead of preserving stale display state. Avoid broken-map warning noise unless later UX work proves it adds real value.
- Empty maps should be allowed and should preserve the current default playback path, not force fake placeholder rules.
- Keep the data model forward-compatible with later score-semantics expansion such as playing techniques, but do not design a speculative kitchen-sink rule language in 6.1.
- Reuse existing playback route ownership patterns. Do not create a parallel asset-linking subsystem when the rack, route, and address seams already exist.

### Risks And Regression Traps

- The biggest architectural trap is storing full reusable map definitions inside project files. That would collapse the distinction between app-scoped assets and project intent and make later sharing harder.
- A second trap is tying the map directly to selected score objects or transient UI state. That would violate the panel-versus-window ownership boundary already established in the UX and architecture docs.
- It would also be easy to over-promise trigger scope. Keep the MVP contract grounded in concrete, likely-needed actions such as keyswitch and CC rather than claiming arbitrary VST trigger breadth.
- Missing-map behavior can become noisy fast. The product direction here is intentionally quiet: truthful `--`, not a new diagnostic state machine.
- Route linkage must remain future-proof for multi-address plugins even though the product does not support that workflow yet. Avoid a model that hardcodes one map per plugin instance with no address distinction.

### Testing Requirements

- Add Rust tests near the owning playback and persistence seams for:
  - Expression Map serialization and deserialization.
  - Score save/load round-trips where `.soloc` preserves only stable map references.
  - Route linkage anchored to slot address data rather than transient UI-owned state.
  - Missing-map references resolving to truthful unassigned state.
  - Empty maps preserving the current default playback path.
- Add or extend TypeScript contract coverage near shared type seams only if payload shape changes need explicit frontend proof.
- No user-visible frontend test surface is required in 6.1 unless a minimal product surface is added later by explicit story change.
- Recommended validation commands once implemented:
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - `npm run check-types`
  - `npm run lint`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

### Documentation Impact

- Update docs when the durable Expression Map ownership model lands. This story changes persistence architecture and future playback expectations, so docs are required, not optional.
- Likely documentation touchpoints:
  - `docs/architecture.md`
  - `docs/data-models.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
- Document explicitly that:
  - Expression Maps are app-scoped reusable assets.
  - Projects persist stable references only.
  - Missing references resolve to unassigned truth.
  - Linkage is future-proofed around slot address even though MVP UX stays route-simple.
  - Audible expression behavior is not part of Story 6.1.

### Architecture Compliance

- Rust remains the authoritative owner of durable Expression Map data, route linkage truth, and project persistence semantics.
- `.soloc` files must keep portable project intent only. Do not persist machine-local library payloads into project files.
- The panel-versus-window scope rule remains intact: score-context surfaces eventually show linkage; app-scoped surfaces own reusable asset definitions.
- The rack-first playback model remains authoritative. Expression Map linkage extends the existing route and address seam rather than bypassing it.
- Story 6.1 must not reintroduce frontend-owned playback truth, SCVI-era fallback design, or plugin-specific architecture residue.

### Library And Framework Requirements

- Stay on the repo baselines already established in the project context: Rust 2021, Tauri 2.10.1, React 19.2.4, and TypeScript 5.9.3.
- Reuse existing serialization, state publication, and window-opening patterns already present in the repo.
- Keep Rust and TypeScript mirrors aligned if new enums, structs, or fields are introduced.

### File Structure Notes

- Durable model and persistence work belongs in backend score and playback seams under `src-tauri/src/`.
- Shared payload changes belong in the existing Rust and TypeScript type contracts rather than in ad hoc frontend convenience layers.
- Do not create a new UI window, panel, or temporary shell in 6.1 just to make the model visible.
- Keep tests seam-local to score playback and file load/save behavior.

### Git Intelligence Summary

- Recent commits show the branch still focused on compact targeted fixes rather than broad structural churn. Story 6.1 should keep that discipline and avoid dragging UI scope forward.
- Existing app-scoped asset patterns such as Plugin Manager already exist, so later Expression Map UI work should extend familiar window patterns instead of inventing a new ownership model.

### Latest Technical Information

- For MVP, the technically defensible trigger contract is narrower than all valid VST3 triggers. Real plugin articulation switching is plugin-specific, so Story 6.1 should model concrete proven actions first: keyswitch note and CC message.
- Program change and other trigger types can remain future extension points if real plugin support later justifies them.
- The current codebase contains no existing Expression Map domain model, which makes it especially important that 6.1 choose a clean durable contract now rather than accreting ad hoc fields later.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 6 and Story 6.1 acceptance criteria.
- `_bmad-output/planning-artifacts/prd.md`
  - Expression maps as future score-aware playback domain model.
- `_bmad-output/planning-artifacts/architecture.md`
  - Playback ownership boundaries, app-scoped window model, and reusable playback asset guidance.
- `_bmad-output/planning-artifacts/ux-design-specification.md`
  - Expression Map scope rules, panel-versus-window ownership, and canonical app-scoped entry-point guidance.
- `_bmad-output/implementation-artifacts/5-5-recover-gracefully-from-missing-or-unavailable-routes.md`
  - Immediate prior-story guidance on portable intent, quiet truthful UI, and backend-owned semantics.
- `src-tauri/src/score/playback.rs`
  - Current rack, route, and route-address persistence seams.
- `src-tauri/src/score/mod.rs`
  - Top-level score persistence model.
- `src-tauri/src/files.rs`
  - Load/save seam for portable project truth.
- `src-tauri/src/playback/preparation.rs`
  - Future playback preparation seam for linked Expression Map consumption.
- `src-tauri/src/playback/transport.rs`
  - Current default playback behavior that 6.1 should leave unchanged.
- `src/services/actions.ts`
  - Existing app-scoped window pattern for later Expression Map editor work.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- 2026-04-05: Story prepared after reviewing Epic 6 planning, PRD direction, architecture ownership rules, UX scope rules, Story 5.5 handoff, current playback route and address seams, and recent repository history.
- 2026-04-05: User clarification locked the key product boundaries: app-scoped reusable library, project-level reference persistence, route-address linkage, quiet missing-map UX, empty-map fallback, and a foundational-only Story 6.1.

### Completion Notes List

- Story context created and sprint status updated to ready-for-dev.
- Story 6.1 is intentionally foundational only. CRUD/library UI and audible behavior remain deferred to later Epic 6 stories.
- Durable rule shape clarified as conditions plus actions, bounded initially to articulation and note-length inputs with keyswitch, CC, no-op, and note-length transformation outputs.
- Persistence contract clarified so `.soloc` stores stable map references only, not reusable map definitions.

### File List

- `_bmad-output/implementation-artifacts/6-1-model-expression-maps-as-durable-playback-assets.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
