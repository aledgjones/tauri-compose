# Story 2.5: Open a Bounded Native Plugin Editor Host for Supported Plugins

Status: ready-for-dev

## Story

As a maintainer,
I want to open a supported plugin's native editor through a dedicated host window,
so that Compose can prove the macOS editor-host seam without mixing it into the main UI model.

## Acceptance Criteria

1. Given a known-good supported plugin is available in the inventory, when the user requests to open its native editor, then Compose opens a dedicated host window for that plugin editor and the editor is hosted through the bounded native window seam rather than HTML embedding.
2. Given the native editor seam is intentionally narrow, when this story is implemented, then the host path remains backend-owned and explicit and the frontend acts only as a thin trigger through the established command and window patterns.
3. Given native editor hosting is not guaranteed for every plugin, when the host path cannot provide a usable editor interaction, then Compose fails gracefully without destabilizing the rest of the app and playback usability outside that editor window is preserved.

## Tasks / Subtasks

- [ ] Task 1: Formalize the bounded open-editor flow on top of the existing native host POC seam (AC: 1, 2)
  - [ ] Reuse `plugin_editor_poc_open` and the dedicated `plugin-host-poc` window seam instead of inventing HTML embedding or a second native-host architecture.
  - [ ] Keep host attachment backend-owned in Rust, with the frontend limited to a deliberate trigger from the appropriate inventory/maintainer surface.
  - [ ] Ensure the targeted plugin path comes from the Rust-owned inventory model or explicit validated input, not ad hoc frontend path construction.
- [ ] Task 2: Add a bounded maintainer-facing trigger without widening into assignment or playback activation UX (AC: 1, 2)
  - [ ] Add a deliberate editor-open trigger from the appropriate app-scoped inventory workflow or dedicated maintainer harness.
  - [ ] Keep the action explicit and secondary; do not present native editor hosting as the default answer to routine playback or assignment problems.
  - [ ] Do not couple the editor-open trigger to score assignment state, transport playback, or plugin activation stories.
- [ ] Task 3: Fail gracefully for unsupported or unusable editor targets (AC: 3)
  - [ ] Surface open failure coherently when the plugin is invalid, unavailable, or cannot provide a usable editor attach path.
  - [ ] Avoid leaving orphaned host windows or ambiguous half-open state when attach fails.
  - [ ] Keep the failure local to the host path and preserve overall app responsiveness.
- [ ] Task 4: Add focused validation and documentation for the bounded host-open seam (AC: 1, 2, 3)
  - [ ] Add or extend backend tests around target resolution and explicit failure paths where the code is testable.
  - [ ] Add focused manual validation guidance for the open-editor path using a known-good plugin target.
  - [ ] Update developer-facing docs for the bounded editor-host trigger and its current limitations.

## Dev Notes

### Story Intent

- Story 2.5 proves that Compose can deliberately open a plugin-provided native editor through a dedicated backend-owned host window.
- This is not a production host abstraction, playback activation workflow, or assignment UX story.
- The goal is a credible bounded seam that can be opened on demand for supported plugins and fail safely otherwise.

### Scope Boundaries

- In scope:
  - Explicit open request for a supported plugin editor.
  - Dedicated host window owned by the backend/native seam.
  - Graceful failure when native host attachment is not possible.
- Out of scope:
  - Lifecycle hardening across resize/focus/close/reopen beyond what is minimally needed to open once. Story 2.6 owns that.
  - Validation matrix and known-good subset reporting. Story 2.7 owns that.
  - Score assignment, transport playback, or plugin activation.
  - Full Steinberg host abstraction, automation, preset/state persistence, or broad compatibility claims.

### Cross-Story Alignment

- Story 2.3 provides the app-scoped inventory surface that can later expose deliberate maintainer entry points.
- Story 2.5 should build only the bounded open seam; Story 2.6 stabilizes lifecycle and Story 2.7 validates failure handling and credibility.
- Keep editor-host work separate from Epic 3 assignment and later playback stories.

### Previous Story Intelligence

- `src-tauri/src/playback/plugin_editor_poc.rs` already contains a live macOS VST3 editor attach attempt using `libloading`, `vst3`, `IComponent`, `IEditController`, and `IPlugView`.
- The existing host state is intentionally minimal and stored entirely in Rust.
- The current frontend trigger still lives under `actions.sampler.openPluginEditorPoc()`, which is a legacy naming seam and should not become the product's long-term ownership model for VST workflows.
- Existing tests are sparse, so the story must lean on focused backend checks plus explicit manual validation.

### Architecture Compliance

- Native host ownership stays in Rust.
- Frontend remains a thin trigger layer only.
- The host window remains a dedicated app window, not an embedded HTML editor surface.
- VST3 plugins remain untrusted third-party code; failures must degrade predictably and locally.
- Do not extend sampler-era runtime state into the authoritative VST host lifecycle model.

### Current Relevant Seams

- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Core command, native attach logic, session storage, and cleanup path.
- `src-tauri/src/playback/vst3.rs`
  - Shared validation and bundle-binary helpers.
- `src-tauri/src/lib.rs`
  - Command registration.
- `src/services/actions.ts`
  - Current frontend command wrapper.
- `plugin-host-poc.html` and `src/windows/plugin-host-poc/`
  - Current dedicated host window shell.

### Implementation Guidance

- Build on the existing POC seam rather than replacing it.
- Keep the host window explicitly bounded and named as a proof seam until later stories justify broader claims.
- Prefer explicit path validation and inventory-derived targeting before native attach.
- If an attach fails after the window opens, fail in a way that is reviewable and recoverable rather than silent.
- Do not let the manager inventory tiles or score surfaces turn into editor-launch defaults; this action should feel deliberate and secondary.

### File Structure Requirements

- Backend work should stay centered in:
  - `src-tauri/src/playback/plugin_editor_poc.rs`
  - `src-tauri/src/playback/vst3.rs` only if shared helper updates are needed
  - `src-tauri/src/lib.rs` for bounded command wiring changes
- Frontend work will likely touch:
  - `src/services/actions.ts`
  - the appropriate app-scoped inventory/maintainer surface
  - `src/windows/plugin-host-poc/` only if the dedicated host shell needs bounded refinement

### Testing Requirements

- Add backend tests where feasible for:
  - explicit target resolution
  - invalid path handling
  - attach-path failure classification that does not orphan state
- Maintain an explicit manual validation checklist for:
  - opening a known-good plugin editor
  - verifying dedicated host window attach
  - confirming graceful failure for invalid targets
- Validate with:
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `cargo test --lib plugin_editor_poc`
  - `cargo clippy --lib -- -D warnings`

### Documentation Impact

- Update docs describing the current plugin editor host POC, trigger path, or limitations.
- Likely touch points:
  - `docs/development-guide.md`
  - `docs/architecture.md`
  - `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`

### References

- `_bmad-output/planning-artifacts/epics.md` — Story 2.5 acceptance criteria
- `_bmad-output/planning-artifacts/architecture.md` — plugin lifecycle/editing ownership and app/window lifecycle wiring
- `_bmad-output/planning-artifacts/ux-design-specification.md` — native plugin UI as a deliberate secondary handoff
- `_bmad-output/implementation-artifacts/tech-spec-macos-native-plugin-editor-host-poc.md`
- `src-tauri/src/playback/plugin_editor_poc.rs`
- `src-tauri/src/playback/vst3.rs`
- `src-tauri/src/lib.rs`
- `src/services/actions.ts`
- `plugin-host-poc.html`
- `src/windows/plugin-host-poc/`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Story selected explicitly as `2.5`.
- Repo inspection confirmed the bounded native host POC already exists and is substantial enough to anchor the story.
- Cross-story planning kept 2.5 focused on the initial open seam while deferring lifecycle hardening to 2.6 and validation credibility to 2.7.

### Completion Notes List

- Story context preserves the backend-owned host boundary.
- Story context explicitly avoids assignment, playback activation, and production-host overreach.
- Story context keeps native editor open as a deliberate secondary maintainer action, not a default workflow.

### File List

- `_bmad-output/implementation-artifacts/2-5-open-a-bounded-native-plugin-editor-host-for-supported-plugins.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
