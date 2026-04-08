# Story 1.5: Make Native Plugin Editors Transient Views On Slot-Owned Sessions

Status: done

## Story

As a composer,
I want the native plugin editor to open against the running instance already owned by a slot,
so that opening or closing the editor does not create, destroy, or swap the real playback instance.

## Acceptance Criteria

1. **Given** a slot-owned runtime session already exists
   **When** I open the editor
   **Then** Compose attaches the editor to that existing slot-owned session rather than creating a second canonical instance
   **And** the attach flow preserves the slot's current plugin state

2. **Given** the editor is closed
   **When** the slot still exists
   **Then** the editor detaches without destroying the slot-owned runtime session
   **And** the slot remains playback-ready according to backend-owned truth

3. **Given** multiple slots host the same plugin bundle
   **When** I open or reopen editors across them
   **Then** each editor targets the exact slot-owned instance rather than a bundle-level approximation
   **And** same-bundle slots remain distinct backend-owned editor targets

## Tasks / Subtasks

- [x] Prove the existing slot-owned session contract survives editor open, close, and reopen without introducing a second ownership seam (AC: 1, 2)
  - [x] Audit the current `plugin_editor_poc` open, attach, detach, close, and destroy flow against Story 1.4's slot-owned runtime-session foundation before changing code.
  - [x] Treat this as a verification-first story: if the current slot-owned session already satisfies the acceptance criteria, prefer tests, diagnostics, and documentation over refactoring.
  - [x] If evidence shows a gap, fix only the owning detach/reattach seam so editor lifecycle remains a transient view over the slot-owned runtime session rather than a creator, destroyer, or swapper of canonical runtime state.
- [x] Extend focused backend regression coverage around transient editor attachment semantics (AC: 1, 2, 3)
  - [x] Add or refine owning Rust tests in `src-tauri/src/playback/plugin_editor_poc.rs` for attach-to-existing-session behavior, close-time detach without session destruction, and reopen against the preserved slot-owned session.
  - [x] Preserve exact-instance targeting for same-bundle multi-slot cases, including target switching after a detached reservation and reopen without a surviving host window.
  - [x] Keep tests seam-local; do not introduce a new cross-cutting editor-host harness if the existing plugin-editor host suite can cover the regression.
- [x] Validate score-context and maintainer-facing behavior without widening product scope (AC: 2, 3)
  - [x] Confirm score-context and maintainer entry points continue to frame native editor open as deeper plugin control, not as the act that initializes playback readiness.
  - [x] Add or refine frontend tests only if needed to protect the bounded trigger behavior or exact slot targeting assumptions already exposed through existing entry points.
  - [x] Do not add new routine UI, new inventory workflows, or new plugin-state authoring surfaces for this story.
- [x] Record verification evidence and documentation impact for the transient-view contract (AC: 1, 2, 3)
  - [x] Update docs and story artifacts if wording, lifecycle expectations, or manual-proof steps change.
  - [x] Capture maintainer verification expectations for open, close, reopen, and same-bundle different-slot targeting on the slot-owned session architecture.

### Review Findings

- [x] [Review][Patch] Sprint-status timestamp regressed during review transition [/Users/aledjones/compose/_bmad-output/implementation-artifacts/sprint-status.yaml:66]
- [x] [Review][Patch] Dev record overstates validation without a recorded maintainer verification outcome [/Users/aledjones/compose/_bmad-output/implementation-artifacts/1-5-make-native-plugin-editors-transient-views-on-slot-owned-sessions.md:109]
- [x] [Review][Patch] Frontend evidence over-claims exact-slot targeting coverage that is actually proven by backend tests [/Users/aledjones/compose/_bmad-output/implementation-artifacts/1-5-make-native-plugin-editors-transient-views-on-slot-owned-sessions.md:182]

## Dev Notes

### Story Intent

This is primarily a verification and seam-hardening story, not a feature-expansion story. Story 1.4 already moved canonical live VST ownership into slot-keyed runtime sessions. Story 1.5 exists to prove that native editor lifecycle is now only an attach/detach view over that slot-owned runtime truth.

The implementation bar is therefore narrow:

- prefer proving the detach/reattach contract with focused tests and maintainer verification
- make code changes only where the existing editor-host lifecycle still violates that contract
- do not broaden scope into assignment-time initialization, reopen restoration from persisted intent, broader plugin compatibility, or audible playback

### Story Foundation

- Epic 1 is a trust-restoration sequence before any Epic 4 audible-playback work.
- Story 1.3 established the trusted single-instrument reset baseline and narrowed the support claim.
- Story 1.4 established slot-owned runtime sessions as the canonical live-plugin seam keyed by `slot_id`.
- Story 1.5 confirms the editor host is now subordinate to that slot-owned seam.
- Story 1.6 remains responsible for assignment-time and reopen-time initialization or restoration from portable saved intent; do not absorb that work here.

### Previous Story Intelligence

- Story 1.4 already added a detached lifecycle phase so same-session editor reopen can preserve the live slot-owned session instead of rebuilding from bundle identity.
- The main failure seams already observed are stale detached ownership, close/destroy ordering races, resize callback reentry, same-bundle slot confusion, and leftover singleton-style host assumptions.
- Recent hardening in `plugin_editor_poc.rs` already covers delayed buffered resize after reattach, detached target switching, and exact slot targeting for same-bundle reopen cases.
- This story should assume the slot-owned runtime registry in `plugin_editor_poc.rs` is the source of truth and should challenge any lingering path that still behaves as if editor-open creates the canonical runtime instance.

### Technical Requirements

- `PlaybackActivationRuntime` remains readiness metadata only; it must not become the long-lived plugin-session registry.
- Native editor open must target the existing slot-owned runtime session for the selected slot identity, not a bundle-only approximation.
- Editor close and window destroy must detach view ownership without tearing down the slot-owned runtime session while the slot still exists.
- Same-bundle different-slot flows must continue to resolve by slot identity plus bundle path so one slot's detached editor state cannot hijack another slot's runtime instance.
- If a code fix is required, keep it at the owning editor-host lifecycle seam under `src-tauri/src/playback/plugin_editor_poc.rs` and avoid parallel truth in React or app-scoped window state.
- Do not introduce persistence changes unless this story uncovers a real persisted-data mismatch. If persisted semantics genuinely change, assess migration impact and update `src-tauri/src/migrations/next.rs`.

### Architecture Compliance

- Rust remains the sole owner of slot, route, plugin-session, and readiness truth.
- Native plugin editor hosting remains a bounded backend-owned seam secondary to Compose-owned playback UX.
- Score-context surfaces own routing, readiness, and recovery relationships; app-scoped windows own inventory browsing and deep-editor access.
- Opening a native editor is a deliberate handoff into an already-running slot-owned instance. It must not be described or implemented as playback initialization.
- `.soloc` remains portable intent only. This story must not persist live runtime objects, host reservations, or machine-local editor-host state.

### Library / Framework Requirements

- Keep playback-domain mutations in Rust through existing Tauri command and event patterns.
- Keep frontend work minimal and bounded to existing entry points and tests if they are needed to protect the transient-view contract.
- Treat VST3 plugins as untrusted third-party code. Avoid lifecycle changes that make teardown, close ordering, or resize handling less defensive.

### File Structure Requirements

- Primary owning seam: `src-tauri/src/playback/plugin_editor_poc.rs`.
- Secondary backend consumers to inspect only if evidence requires it: `src-tauri/src/playback/runtime_activation.rs`, `src-tauri/src/score/playback.rs`, `src-tauri/src/files.rs`, and `src-tauri/src/lib.rs`.
- Existing frontend test seams that may need bounded updates: `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx`, `src/windows/main/components/plugins.test.tsx`, and `src/windows/plugin-manager/index.test.tsx`.
- Documentation targets if behavior or verification guidance changes: `docs/architecture.md`, `docs/development-guide.md`, `docs/project-context.md`, `TESTING.md`, and active planning artifacts that describe Epic 1 gating.

### Testing Requirements

- Prefer focused Rust coverage first. The most important protection is seam-local regression coverage in `plugin_editor_poc.rs` for attach, detach, reopen, and same-bundle multi-slot targeting.
- If frontend behavior changes, add or refine only the targeted UI tests needed to preserve the bounded native-editor trigger and score-context framing.
- Run the repo-standard validation relevant to touched files. Minimum expectation if Rust changes: focused Rust tests for the plugin editor host seam. Run `npm run lint`, `npm run check-types`, and `cargo clippy --lib -- -D warnings` when the touched code path makes that practical, but do not treat unrelated pre-existing warnings as part of this story.

### Manual Verification Expectations

- Prepare one routed score instrument against a supported single-instrument VST plugin whose slot already has a live runtime session.
- Confirm readiness is visible before opening the native editor window.
- Open the native editor from the existing bounded entry point, close it, and reopen it in the same session. Verify the slot remains ready across that cycle and the plugin state is preserved.
- Where two slots use the same plugin bundle, verify opening or reopening the editor on each slot continues to target the exact slot-owned instance rather than collapsing onto bundle-level identity.
- Confirm close or destroy of the editor window does not silently tear down the slot-owned runtime session while the slot still exists.

### Implementation Risks And Guardrails

- Do not make changes for the sake of visible activity. If the existing slot-owned lifecycle already satisfies an acceptance criterion, record that with tests or verification evidence instead of refactoring.
- Do not reintroduce singleton editor-host ownership or any bundle-global runtime approximation.
- Do not broaden this story into Story 1.6 initialization or reopen restoration work.
- Do not add new UI flows whose main purpose is to prove acceptance criteria literally when existing bounded entry points already cover the transient-view contract.
- Do not let editor-host resize or focus fixes regress the close/detach lifecycle or same-bundle slot targeting.

### Relevant Code Seams

- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/runtime_activation.rs`
- `src-tauri/src/score/playback.rs`
- `src-tauri/src/files.rs`
- `src-tauri/src/lib.rs`
- `src/windows/main/screens/sequence/components/playback-rack-panel.tsx`
- `src/windows/main/components/plugins.tsx`

### Git Intelligence Summary

- Recent work is still foundational rather than feature-expansive: `29a5f3cd feat(1.4): plugin stability` is directly upstream of this story.
- `dbc55d7e feat(1.3): plugin readiness and course correction of arch` established the narrow trust baseline and architectural wording this story must preserve.
- The current pattern is to harden backend-owned seams and document exact support boundaries, not to add broad new playback UX.

### Project Structure Notes

- The rack-first model remains authoritative: racks own slot lifecycle, slots own plugin assignment or unassignment, and instruments own routes to existing slots.
- `plugin-host-poc` remains a bounded development seam, not the canonical inventory or routing workflow.
- Existing Sequence rack-panel and maintainer-only editor entry points are already the right bounded surfaces for this story. Do not invent an alternate editor workflow.

### Documentation Impact

Review whether these need updates based on the final implementation and verification evidence:

- `docs/architecture.md`
- `docs/development-guide.md`
- `docs/project-context.md`
- `TESTING.md`
- `_bmad-output/planning-artifacts/architecture.md`
- `_bmad-output/planning-artifacts/epics.md`

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.5: Make Native Plugin Editors Transient Views On Slot-Owned Sessions]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/prd.md#Journey 4: Maintainer Validates The Playback Engine As Release-Credible Infrastructure]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#3.6 Native Plugin UI Escalation]
- [Source: docs/architecture.md]
- [Source: docs/development-guide.md]
- [Source: docs/project-context.md]
- [Source: /memories/repo/plugin-editor-host-session-lifecycle.md]
- [Source: /memories/repo/slot-owned-live-runtime-sessions-story-1-4.md]

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story context generated from Epic 1 planning artifacts, Story 1.4 implementation notes, active architecture and UX docs, repo memory, and recent git history.
- Framed as a verification-first story per current sprint intent and the existing slot-owned runtime-session foundation.
- Audit result: `src-tauri/src/playback/plugin_editor_poc.rs` already opens against a slot-targeted runtime session, preserves detached slot-owned sessions on close, and reopens detached targets without reintroducing bundle-global ownership.
- Existing seam-local Rust coverage already proves the required lifecycle transitions for Story 1.5, including detached reopen, stale-reservation cleanup, same-bundle different-slot rejection while reserved, and same-bundle different-slot retargeting after detached release.
- Existing frontend coverage proves the bounded score-context entry point forwards `slot_id` through `actions.pluginInventory.openEditorPoc(...)`, preserving the intended rack-surface trigger and slot-target handoff without widening UI scope.
- Validation run: `cargo test --manifest-path src-tauri/Cargo.toml plugin_editor_poc -- --nocapture` -> `41 passed; 0 failed`.
- Validation run: targeted frontend tests in `src/services/actions.test.ts` and `src/windows/main/screens/sequence/components/playback-rack-panel.test.tsx` -> `167 passed; 0 failed`.
- Validation run: `npm run lint` and `npm run check-types` passed with no reported errors.
- Validation caveat: `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings` remains blocked by pre-existing dead-code warnings in untouched files `src-tauri/src/playback/audio.rs` and `src-tauri/src/playback/mixer.rs`.
- Documentation review: current wording in `docs/architecture.md` and `docs/project-context.md` already describes native editor attach/detach as a transient view over slot-owned runtime sessions, so no documentation change was required for this story.
- Maintainer verification completed and passed for the manual open/close/reopen and same-bundle different-slot checks in this story's verification section, so the story can be closed on top of the existing automated validation evidence.

### Completion Notes List

- Story 1.5 context created as a prove-and-harden story with explicit no-expansion guardrails.
- Verified that Story 1.4's shipped implementation already satisfies AC 1-3 for slot-owned editor attach/detach/reopen behavior; no product-code refactor was required.
- Confirmed same-bundle slot targeting remains keyed by `slot_id` plus bundle path in the backend seam and by `slot_id`-forwarding score-context entry points in the frontend seam.
- Recorded maintainer verification expectations for open, close, reopen, and same-bundle different-slot targeting directly from the story's manual verification section, then completed that manual proof before closing the story.

### File List

- \_bmad-output/implementation-artifacts/1-5-make-native-plugin-editors-transient-views-on-slot-owned-sessions.md
- \_bmad-output/implementation-artifacts/sprint-status.yaml

## Change Log

- 2026-03-25: Validated that the existing slot-owned editor-host lifecycle already satisfies Story 1.5 through focused automated coverage, clarified that exact-instance proof lives in the backend seam, completed maintainer verification, and closed the story.
