# Story 2.6: Stabilize Native Editor Host Lifecycle Within a Session

Status: done

## Story

As a maintainer,
I want the bounded editor-host seam to survive focus, resize, close, and reopen behavior,
so that we can treat it as a usable platform seam rather than a one-shot proof.

## Acceptance Criteria

1. Given a supported plugin editor has been opened in the dedicated host window, when the user resizes the window, changes focus, closes it, and reopens it in the same session, then the host window continues to attach the editor correctly through each lifecycle step and the app is not left with an orphaned window, crashed host path, or lost editor ownership.
2. Given the native editor host has already been opened successfully in the current session, when the user closes the host window and later opens that same supported plugin editor again, then Compose reuses the bounded host flow coherently for the new open request and the reopened editor path does not require an app restart or manual cleanup to work.
3. Given Epic 2 is proving a bounded macOS editor-host seam rather than a full production host abstraction, when this story is complete, then lifecycle behavior is explicit and reviewable for the supported session interactions and later playback or assignment concerns remain outside this story's scope.

## Tasks / Subtasks

- [x] Task 1: Make session lifecycle behavior explicit in the bounded host seam (AC: 1, 2, 3)
  - [x] Preserve a single authoritative active host session and active plugin target within the app session.
  - [x] Ensure close, destroy, and reopen paths cleanly tear down and re-establish native host ownership.
  - [x] Prevent orphaned session state when lifecycle events arrive in awkward order.
- [x] Task 2: Support bounded focus and resize behavior that reflects plugin capability (AC: 1, 3)
  - [x] Keep plugin focus notifications coherent when host-window focus changes.
  - [x] Honor plugin-driven resize capability only when the plugin reports that its editor can resize.
  - [x] Avoid promising arbitrary production-host resizing behavior beyond the supported bounded session interactions.
- [x] Task 3: Preserve reopen and teardown safety within one app session (AC: 1, 2)
  - [x] Ensure closing a host window leaves the app able to reopen the same supported plugin editor again without restart.
  - [x] Preserve the teardown order needed to disconnect and terminate native host resources safely.
  - [x] Keep failures local to the host seam and do not leak stale window references or session ownership.
- [x] Task 4: Add focused lifecycle validation and maintainer-facing guidance (AC: 1, 2, 3)
  - [x] Add backend tests for lifecycle invariants and pure logic where feasible.
  - [x] Add or refine manual validation steps for focus, resize, close, destroy, and reopen.
  - [x] Update docs describing supported host-lifecycle behavior and known limitations.

## Dev Notes

### Story Intent

- Story 2.5 proves the native editor can open through the bounded host seam.
- Story 2.6 makes that seam reviewably usable across the lifecycle interactions maintainers need in a single session.
- This is still a bounded POC seam, not a full production host abstraction.

### Scope Boundaries

- In scope:
  - Focus, resize, close, destroy, and reopen behavior for the dedicated host window.
  - Session-local cleanup and reattach correctness.
  - Plugin-driven resize support only where the plugin can report it.
- Out of scope:
  - Broad compatibility claims. Story 2.7 owns validation framing.
  - Score assignment, playback runtime activation, or transport integration.
  - Production-grade host abstractions beyond the bounded session interactions in the acceptance criteria.

### Cross-Story Alignment

- Story 2.5 should provide the deliberate open seam; Story 2.6 hardens lifecycle around that seam.
- Story 2.7 then validates failure handling and credible support boundaries for a known-good subset.
- Keep lifecycle stabilization separate from later Epic 3 assignment and Epic 4 playback work.

### Previous Story Intelligence

- Current repo inspection shows the host window is still effectively fixed-size by default and lifecycle behavior relies on the existing cleanup path in `plugin_editor_poc.rs`.
- The current session is tracked through global session/target holders, so Story 2.6 should preserve and harden those invariants rather than inventing a parallel lifecycle model.
- Cleanup order matters: disconnect, remove view, terminate controller/component, then drop session ownership.

### Architecture Compliance

- Native host lifecycle remains backend-owned.
- All AppKit-native operations stay on the main thread.
- Frontend remains a trigger and shell only; it does not own host lifecycle truth.
- Session-local lifecycle hardening must not spill into transport/runtime or score-domain state.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Active session storage, focus sync, cleanup hooks, and native resize callbacks.
- `plugin-host-poc.html` and `src/windows/plugin-host-poc/`
  - Dedicated host window shell.
- `src-tauri/src/lib.rs`
  - Command registration and window/app lifecycle integration points as needed.

### Implementation Guidance

- Treat lifecycle support as stabilization of the existing bounded host seam, not as a rearchitecture.
- Make resize behavior capability-driven:
  - default conservative host behavior
  - allow plugin-driven resize when reported
  - keep unsupported resize interactions explicit rather than magical
- Be especially careful around reopen-after-close and app-shutdown cleanup paths, where stale session state is most likely.
- Keep session invariants obvious and reviewable; if there is one active target, there should be one coherent active host session.

### File Structure Requirements

- Work should stay centered in:
  - `src-tauri/src/playback/plugin_editor_poc.rs`
  - the dedicated host-window shell only if bounded window support needs refinement
- Do not spread Story 2.6 into assignment UI, plugin inventory truth, or playback transport files.

### Testing Requirements

- Add backend tests where feasible for:
  - active-session invariant behavior
  - close/reopen pure logic
  - resize/focus helper behavior that is testable without full native integration
- Maintain explicit manual validation for:
  - open
  - focus switch
  - resize
  - close
  - reopen in the same session
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `cargo test --lib plugin_editor_poc`
  - `cargo clippy --lib -- -D warnings`

### Documentation Impact

- Update docs describing supported host lifecycle behavior, especially the bounded status of resize/reopen support and any maintained limitations.
- Likely touch points:
  - `docs/development-guide.md`
  - `docs/architecture.md`
  - `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 2.6 acceptance criteria
- `_bmad-output/planning-artifacts/architecture.md` — plugin lifecycle/editing ownership and backend-owned window lifecycle
- `_bmad-output/implementation-artifacts/2-5-open-a-bounded-native-plugin-editor-host-for-supported-plugins.md`
- `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `plugin-host-poc.html`
- `src/windows/plugin-host-poc/`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.6`.
- Repo inspection confirmed lifecycle stabilization is the next clean increment after the open-host seam and before credibility validation.
- The story is anchored to the existing session holders, cleanup ordering, and plugin-driven resize/focus behavior already present in the code.
- Replaced the split target-only reservation with an explicit backend lifecycle record so one target reservation and one live session stay coherent through reserve, attach, close, destroy, and reopen.
- Fixed the initial native host sync ordering bug by storing the live session before issuing the first focus/size sync.
- Added unit coverage for lifecycle phase transitions, stale reservation recovery, and bounded geometry-sync behavior.

### Completion Notes List

- Story context keeps lifecycle hardening local to the bounded host seam.
- Story context makes resize support capability-driven rather than overclaimed.
- Story context preserves the separation from assignment and playback runtime work.
- `plugin_editor_poc.rs` now keeps one authoritative lifecycle reservation for the active plugin target, marks close as an explicit teardown phase, and clears stale reservations when no host window remains.
- Native session startup now stores the session before issuing the initial focus/size sync, while keeping geometry sync callbacks available so attached editors keep the correct size and position.
- Added focused Rust tests for lifecycle reservation transitions, stale close/reopen recovery, and bounded geometry-sync behavior.
- Updated maintainer-facing docs and the host tech spec to document bounded focus, resize, close, destroy, and same-session reopen behavior.
- Validation passed with `cargo test --lib plugin_editor_poc`, `cargo clippy --lib -- -D warnings`, `npm run lint`, `npm run check-types`, and the JS suite (`1134` tests passed).

### File List

- `_bmad-output/implementation-artifacts/2-6-stabilize-native-editor-host-lifecycle-within-a-session.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`
- `docs/architecture.md`
- `docs/development-guide.md`
- `src-tauri/src/playback/plugin_editor_poc.rs`

## Change Log

- 2026-03-22: Stabilized the bounded native editor host lifecycle by making target/session ownership explicit across attach, focus, resize, close, destroy, and same-session reopen; added focused Rust coverage; and updated maintainer-facing lifecycle guidance.
