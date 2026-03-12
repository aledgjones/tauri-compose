# Story 4.2: Preserve Native Desktop Document Behavior Around Multi-Flow Score View

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a macOS user,
I want multi-flow score view to coexist with normal desktop document behavior,
so that the application still feels like a reliable native document tool.

This story is about desktop behavior around the score, not just file serialization. Multi-flow score view must continue to work with the existing File menu, open/save/open-recent flows, current-file-path state, compact-window shell behavior, and the rule that the shared top bar stays reserved for universal controls rather than workflow-specific single-flow switching.

## Acceptance Criteria

1. Given the application supports multi-flow score rendering, when a user opens or saves documents through the existing native-style file workflows, then those workflows remain consistent with current application behavior and multi-flow score view does not force a different document interaction model.
2. Given supported score documents are opened through desktop document workflows, when the app handles them through its existing file entry points, then file association expectations remain intact and the multi-flow feature does not break the normal document-opening path.
3. Given a single-flow document workflow, when it is exercised after the multi-flow changes, then current native desktop behavior remains correct and no regression is introduced in established file handling patterns.
4. Given the main window is resized into the compact desktop layout tier, when secondary panels collapse or expand around the score view, then document, playback, and tool workflows remain reachable through the same top-level shell structure and multi-flow score view remains the primary visible work surface without introducing a different navigation model.
5. Given the shared top bar is visible across all workflows, when the multi-flow feature is exercised across score, sequencer, and playback contexts, then the top bar continues to present only universal tools and information and workflow-specific flow selectors do not occupy persistent shared shell chrome.
6. Given this desktop-workflow increment is validated, when the broader feature is reviewed, then the score-view enhancement remains compatible with the app's macOS-first document model and no network dependency or platform expansion is introduced.

## Tasks / Subtasks

- [x] Audit current native document entry points and shell behavior (AC: 1, 2, 3, 4, 5, 6)
  - [x] Inspect `src-tauri/src/lib.rs`, `src-tauri/src/files.rs`, and `src/services/state/provider.tsx` to confirm how File menu actions, open-recent events, current-file-path updates, and menu refresh behavior currently work.
  - [x] Inspect `src/windows/main/index.tsx` and related shell components to confirm the current shared top-bar contract and compact-layout behavior around the score surface.
  - [x] Identify whether any desktop file-association behavior is currently configured in packaging or handled only through existing open entry points, and preserve current expectations rather than inventing new platform-specific scope.
- [x] Preserve native-style document workflows while multi-flow score view is active (AC: 1, 2, 3)
  - [x] Keep the existing File menu actions, open/save/open-recent flows, and current-file-path state coherent when multi-flow rendering is present.
  - [x] Ensure opening a project through existing app entry points still lands the user in the normal main-window workflow with the score surface as the primary workspace.
  - [x] Preserve single-flow document behavior exactly; multi-flow support must not alter baseline desktop workflows for projects that never show multiple flows.
- [x] Preserve the shared shell contract in compact and normal layouts (AC: 4, 5)
  - [x] Keep the shared top bar limited to universal controls such as tabs, layout selection, transport, and tools.
  - [x] Ensure workflow-specific flow selectors remain local to their owning workflow surfaces instead of reappearing in persistent shared chrome.
  - [x] Verify panel collapse/expand behavior still leaves document, playback, and tool workflows reachable without introducing an alternate navigation model.
- [x] Add focused regression coverage and manual desktop verification (AC: 1, 2, 3, 4, 5, 6)
  - [x] Extend frontend tests around provider menu listeners, shell layout, and any top-bar control placement that this story touches.
  - [x] Add or extend backend tests only if menu-event or file-entry wiring changes materially in `src-tauri/src/lib.rs` or `src-tauri/src/files.rs`.
  - [x] Add manual macOS verification notes for open, save, open recent, compact-window resizing, and any supported Finder/document-entry workflow currently expected by the app.
- [x] Update docs if this story formalizes stable desktop-shell rules (AC: 5, 6)
  - [x] Update `docs/project-context.md`, `docs/architecture.md`, and `docs/project-overview.md` if implementation changes or formalizes maintainer expectations about shared shell chrome, menu behavior, or native desktop document handling.

## Dev Notes

### Story Foundation

- Story 4.1 protects the file format and round-trip behavior. Story 4.2 widens the guardrail to the surrounding desktop workflow: menus, open entry points, top-bar behavior, and compact-window shell behavior.
- Epic 2 already established that workflow-specific single-flow switching belongs inside the owning workflow surface rather than in the shared top bar. Story 4.2 must preserve that shell rule while validating document workflows around multi-flow score view.
- The outcome should feel boring in the best way: multi-flow score view exists, but the application still behaves like the same macOS-first document tool.

### Developer Context Summary

- `src-tauri/src/lib.rs` builds the macOS app menu, including File menu items for New, Open, Open Recent, Save, and Save As, and emits the corresponding `soloc://menu/*` events.
- `src/services/state/provider.tsx` listens for `soloc://menu/save`, `soloc://menu/save-as`, `soloc://menu/open`, and `soloc://menu/open-recent`, and it invokes `refresh_menus` after initialization.
- `src-tauri/src/files.rs` owns current-file-path state, recent-files storage, and menu refresh behavior, including Save enablement and Open Recent menu updates.
- `src/windows/main/index.tsx` is the main shell seam where shared top-bar controls live; this is the place where universal-vs-workflow-specific control placement must remain disciplined.
- `src-tauri/tauri.conf.json` currently configures the app bundle and window shape but does not expose an obvious explicit file-association block in the checked-in config. Story 4.2 should therefore preserve current behavior and document any discovered association path instead of assuming hidden platform expansion work.

### Technical Requirements

- Preserve the existing File menu interaction model and current-file-path behavior while multi-flow score view is active.
- Keep Open Recent behavior truthful and synchronized with recent-file storage; multi-flow state must not create a special-case document path.
- Keep the shared top bar reserved for universal controls and information only.
- Preserve the score as the dominant work surface in both normal and compact layouts; panel collapse/expand should not force a different navigation model for multi-flow use.
- Do not introduce platform expansion, cloud dependency, or new windowing/navigation concepts.
- Preserve single-flow behavior and existing desktop workflows for projects that never use multi-flow presentation.
- If file-association behavior needs changes, keep them narrowly scoped to existing packaging/document-entry paths and document the impact clearly.

### Architecture Compliance

- The app remains macOS-first, desktop-first, and offline-first.
- Shared shell chrome remains universal; workflow-specific controls belong in local workflow surfaces.
- Document entry points remain backend-owned and event-driven through the existing Tauri command/menu model.
- Multi-flow remains a score presentation enhancement, not a new document interaction model.

### Library / Framework Requirements

- Preserve the current Tauri desktop shell and event wiring instead of adding browser-style alternatives.
- Reuse existing `invoke()`-based commands and event listeners for menu-driven open/save workflows.
- Preserve React shell structure and Tailwind-based layout patterns already established in the main window.
- External web research was not available in this environment; repository docs and current source are the authoritative baseline.

### Project Structure Notes

- Primary backend desktop-workflow seams:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/files.rs`
  - `src-tauri/tauri.conf.json`
- Primary frontend shell/document seams:
  - `src/services/state/provider.tsx`
  - `src/services/actions.ts`
  - `src/windows/main/index.tsx`
  - `src/windows/main/components/transport.tsx`
- Supporting docs:
  - `docs/project-context.md`
  - `docs/architecture.md`
  - `docs/project-overview.md`

### File Structure Requirements

- Extend existing menu, provider, and shell components before creating new document-workflow abstractions.
- Avoid adding separate multi-flow shell chrome or alternate compact-layout navigation.
- Keep platform-specific packaging/config changes minimal and well documented if they are required at all.
- Place regression coverage next to the existing provider, actions, and main-shell tests.

### Testing Requirements

- Add or extend tests for File menu event handling, open/save/open-recent command wiring, and Save enablement if those seams change.
- Add or extend shell-level tests proving the shared top bar remains universal and workflow-specific flow selection does not reappear there.
- Manually verify compact-window behavior, panel collapse/expand, open/save/open recent flows, and any supported Finder/document-entry behavior on macOS.
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`

### Previous Story Intelligence

- Story 4.1 established that file integrity and round-tripping should remain unchanged. Story 4.2 should build on that by keeping the surrounding desktop workflow equally unchanged from the user's perspective.
- Epic 2 already pushed active-flow selectors out of shared chrome. Story 4.2 should treat that as an existing shell rule to preserve, not a design debate to reopen.

### Git Intelligence Summary

- `ae16e160 chore: refactor render trigger API` suggests the rendering lifecycle is still evolving, which increases the importance of keeping desktop entry points and shell behavior stable while render changes continue underneath.
- The recent Epic 3 commits all strengthened correctness and observability without changing the app's desktop posture. Story 4.2 should follow the same pattern.

### Project Context Reference

- `_bmad-output/planning-artifacts/ux-design-specification.md` says the shared top bar should contain universal tools and information only, with workflow-local flow selectors inside the owning workflow surface.
- `_bmad-output/planning-artifacts/architecture.md` treats desktop document workflow as a first-class architectural concern mapped to `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, and related shell seams.
- `docs/project-context.md` and `docs/project-overview.md` both position the product as a macOS-first desktop application with native-feeling file workflows and stable shell logic.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` (Epic 4, Story 4.2)
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` (shared top-bar and compact-shell rules)
- Source: `_bmad-output/planning-artifacts/architecture.md` (desktop workflow mapping)
- Source: `docs/project-context.md` (desktop-first, shell, and active-flow rules)
- Source: `docs/project-overview.md` (desktop application framing)
- Source: `src-tauri/src/lib.rs` (File menu wiring)
- Source: `src-tauri/src/files.rs` (current file path, recents, menu refresh)
- Source: `src/services/state/provider.tsx` (menu event listeners and menu refresh)
- Source: `src/windows/main/index.tsx` (shared top-bar shell)
- Source: `src-tauri/tauri.conf.json` (bundle and window configuration)

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Audit the existing Tauri File menu, provider event listeners, current-file-path refresh path, and shared main-window shell to confirm the story can preserve behavior by strengthening guardrails rather than adding new desktop workflow code.
- Add frontend regression coverage that locks in the main-window menu listener wiring and refresh behavior in `src/services/state/provider.tsx`.
- Add main-shell regression coverage that proves workflow-local flow selection does not leak back into the shared top bar while compact and normal workflow surfaces keep their existing local controls.

### Completion Notes List

- Story 4.2 was created from the Epic 4 desktop-workflow acceptance criteria, current Tauri menu wiring, and the repo's existing shell/UX rules.
- The story treats file associations conservatively: preserve current behavior and only touch packaging/document-entry seams if implementation discovery proves they are the active path.
- The repository does not contain `_bmad/core/tasks/validate-workflow.xml`, so the BMAD validator could not be invoked; the create-story checklist was applied manually while preparing this artifact.
- External web research was not available in this environment; repository docs and source were used as the authoritative baseline.
- Audited `src-tauri/src/lib.rs`, `src-tauri/src/files.rs`, and `src/services/state/provider.tsx` and confirmed the existing native-style file workflow already flows through the same File menu events, open-recent events, current-file-path updates, and `refresh_menus` path regardless of multi-flow rendering.
- Audited `src/windows/main/index.tsx`, `src/windows/main/screens/write/index.tsx`, and `src/windows/main/screens/sequence/index.tsx` and confirmed the shared top bar remains limited to universal controls while workflow-local flow switching stays in local workflow surfaces.
- Confirmed `src-tauri/tauri.conf.json` does not define a new explicit file-association block; the story preserves the current packaging/document-entry scope rather than expanding platform behavior.
- Added frontend regression coverage in `src/services/state/provider.test.tsx` for main-window menu listener registration, open/save/open-recent routing, secondary-window listener exclusions, and menu refresh behavior.
- Added main-shell regression coverage in `src/windows/main/index.test.tsx` to keep workflow-specific flow switching out of the shared top bar while allowing local workflow surfaces to manage their own flow selection.
- No backend code changes or migrations were required because the desktop workflow and file entry paths already satisfied the story constraints.
- No docs changes were required because the existing project context and story notes already describe the stable shared-shell and desktop-workflow rules that this implementation now enforces with tests.
- Manual macOS verification notes: verify File > Open, File > Save, File > Open Recent, and current-file-path title updates from the main window; resize the window through compact panel states in Write and Sequence and confirm score, playback, and tool workflows remain reachable; verify any currently supported Finder or document-entry path still lands in the normal main window without alternate navigation.
- Story accepted on 2026-03-12 and marked done.
- Validation completed with `npm run lint`, `npm run check-types`, `npm run test:js`, `npm run test:rust`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`.

### File List

- `_bmad-output/implementation-artifacts/4-2-preserve-native-desktop-document-behavior-around-multi-flow-score-view.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `src/services/state/provider.test.tsx`
- `src/windows/main/index.test.tsx`

### Change Log

- 2026-03-12: Audited the desktop menu and shell seams for multi-flow compatibility and added frontend regression coverage for main-window menu wiring, menu refresh behavior, and shared-top-bar flow-switcher boundaries.
- 2026-03-12: Story accepted and status updated from review to done.
