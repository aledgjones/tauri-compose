---
title: "macOS Native Plugin Editor Host POC"
slug: "macos-native-plugin-editor-host-poc"
created: "2026-03-20"
status: "implemented-partial"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  - "Rust 2021"
  - "Tauri 2.10.3 with macos-private-api"
  - "objc2 / AppKit / Foundation / Quartz / Metal"
  - "React 19 + TypeScript + Vite multi-window frontend"
  - "Vitest + renderWithProviders frontend tests"
  - "Candidate host crates: vst3 0.3.0 bindings, augmented_baseview 0.1.3"
files_to_modify:
  - "src-tauri/src/lib.rs"
  - "src-tauri/src/playback/mod.rs"
  - "src-tauri/src/playback/plugin_editor_poc.rs"
  - "src-tauri/Cargo.toml"
  - "src-tauri/capabilities/default.json"
  - "src/services/actions.ts"
  - "src/windows/main/components/plugins.tsx"
  - "src/windows/main/components/plugins.test.tsx"
  - "plugin-host-poc.html"
  - "src/windows/plugin-host-poc/boot.tsx"
  - "docs/development-guide.md"
  - "docs/architecture.md"
code_patterns:
  - "Frontend commands flow through actions.ts invoke wrappers using snake_case"
  - "Secondary windows are opened from React with WebviewWindow and getByLabel dedupe"
  - "Backend commands are registered centrally in lib.rs generate_handler"
  - "macOS native views are attached by reaching through Tauri WebviewWindow to NSWindow/contentView"
  - "Experimental backend seams fit best as focused Rust modules wired through playback/mod.rs or a sibling native-host module"
test_patterns:
  - "Vitest component tests with renderWithProviders"
  - "Tauri window APIs mocked with vi.mock in window/component tests"
  - "POC native host validation will need explicit manual macOS runbook steps in addition to unit coverage"
---

# Tech-Spec: macOS Native Plugin Editor Host POC

**Created:** 2026-03-20

## Overview

### Problem Statement

Compose previously proved instrument playback, but did not prove that a native plugin editor can be hosted reliably inside a Tauri-managed macOS window. The open question is not HTML embedding; it is whether the app can create a native child window or surface, attach a plugin-provided AppKit view into that native hierarchy, and keep focus, resize, and teardown behavior stable enough to justify deeper host integration work.

### Solution

Build a minimal macOS-only proof of concept that opens a dedicated Tauri window for a plugin editor and mounts the plugin's native editor view into the window's native AppKit content hierarchy. Keep the POC VST3-first, backend-driven, and intentionally detached from transport, score assignment, and broader playback architecture so it proves window-hosting principles with the smallest viable surface area.

### Scope

**In Scope:**

- macOS-only, debug/dev-only proof of concept
- VST3-first native plugin editor hosting path
- Dedicated Tauri-owned native window or surface for the editor
- Backend command to open a test plugin editor by explicit plugin path or fixed test target
- Native editor lifecycle: create, attach, show, focus, resize, close, and destroy cleanly
- Minimal frontend trigger or debug entry point if one is needed to exercise the backend host path
- Basic instrumentation/logging to confirm lifecycle events and failure modes
- Documentation impact assessment for the experimental host seam and any developer setup needed to run the POC

**Out of Scope:**

- HTML or React embedding of the plugin editor UI
- Production-ready plugin hosting architecture
- Cross-platform editor hosting
- Transport playback integration
- Instrument assignment or score-driven plugin lifecycle
- Preset management, automation, MIDI routing, or state persistence for plugin editors
- Multiple simultaneous editor windows unless required as part of clean teardown validation

## Context for Development

### Codebase Patterns

- Frontend window launches follow a consistent pattern: React code opens `WebviewWindow`, uses `getByLabel()` to avoid duplicate windows, and passes only minimal routing/query state.
- Frontend-to-backend mutations and commands are centralized in `src/services/actions.ts` using `invoke()` with snake_case command names and argument keys.
- Tauri backend commands are registered centrally in `src-tauri/src/lib.rs` via one `generate_handler![]` block, so experimental commands belong there rather than being dynamically discovered.
- Existing native macOS interop already reaches through Tauri `WebviewWindow` to the AppKit `NSWindow`, gets the `contentView`, and inserts a custom `NSView` subview. That is the strongest in-repo proof that the editor POC should use direct AppKit parenting rather than inventing a parallel embedding model.
- Current plugin UX is SCVI metadata plus audition playback. There is no retained live plugin-host lifecycle today, which confirms this work is a clean experimental seam rather than an extension of the sampler cache path.
- Frontend tests use Vitest plus `renderWithProviders`, and window APIs are mocked directly with `vi.mock("@tauri-apps/api/webviewWindow")`, so any small frontend trigger added for the POC should follow that testing pattern.
- The project favors focused backend-owned truth and explicit seams over broad abstraction layers, so the POC should start with one explicit macOS host command and one explicit test window rather than a generalized plugin-host framework.

### Files to Reference

| File                                           | Purpose                                                                                                                      |
| ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `src/windows/main/components/plugins.tsx`      | Existing pattern for opening a plugin-related child window from the frontend                                                 |
| `src/windows/main/components/plugins.test.tsx` | Existing Vitest coverage pattern for plugin window launch behavior                                                           |
| `src/services/actions.ts`                      | Canonical frontend `invoke()` wrapper pattern for new backend commands                                                       |
| `src/windows/plugin-settings/index.tsx`        | Current plugin settings window content; useful contrast for what remains HTML-only and what should stay untouched by the POC |
| `src/windows/plugin-settings/index.test.tsx`   | Existing plugin window component test pattern                                                                                |
| `src-tauri/src/renderer/backends/metal.rs`     | Existing macOS-native `NSWindow`/`NSView` access and subview insertion pattern                                               |
| `src-tauri/src/playback/sampler.rs`            | Confirms current plugin concept is SCVI metadata and audition, not a live VST host                                           |
| `src-tauri/src/playback/mod.rs`                | Current playback module insertion point for a focused experimental host seam                                                 |
| `src-tauri/src/lib.rs`                         | Tauri setup and command registration seam for adding experimental backend commands                                           |
| `src-tauri/Cargo.toml`                         | Existing Tauri/macOS private API and `objc2` dependency baseline                                                             |
| `src-tauri/capabilities/default.json`          | Window capability pattern for plugin-related child windows                                                                   |
| `docs/project-context.md`                      | Project-wide architecture and implementation rules that constrain the POC                                                    |

### Technical Decisions

- Target `VST3` first, but keep the first seam explicitly experimental and narrow: one editor host path, one window label, one open command, one close path.
- Treat the POC as `macOS-only`; do not add Windows/Linux abstractions or shared platform traits yet.
- Use a dedicated editor-host window rather than repurposing the existing plugin-settings UX. The current plugin-settings screen is HTML-based and useful for metadata/audition, while the POC should avoid mixing concerns.
- Prefer a backend-owned host lifecycle with a minimal frontend trigger. The frontend should request the POC window; the backend should own editor instantiation, AppKit parenting, teardown, and logging.
- Reuse Tauri as the native shell, but attach the plugin editor as a direct AppKit subview in the content hierarchy. That matches the existing Metal renderer pattern and keeps the proof focused on native view hosting inside a Tauri-managed window.
- The likely smallest viable implementation route is a new focused Rust module such as `src-tauri/src/playback/plugin_editor_poc.rs`, wired through `playback/mod.rs` and `lib.rs`, plus a dedicated empty or near-empty host window entrypoint such as `plugin-host-poc.html`.
- `vst3` is a plausible low-level binding candidate but is intentionally unsafe and low abstraction; it is suitable only if the POC keeps the host surface narrow. `augmented_baseview` is a candidate fallback if direct AppKit parenting through Tauri proves too awkward, but it should not be the first assumption because the repo already has direct AppKit access.
- If the POC requires direct AppKit coordination on the main thread, keep that explicit in the host seam and manual validation notes rather than hiding it behind frontend assumptions.
- If the POC remains in the repository after validation, document developer prerequisites, macOS-only limitations, experimental status, and any plugin path/setup expectations in the relevant docs.

## Implementation Plan

### Tasks

- [x] Task 1: Introduce an isolated macOS-only experimental editor-host backend seam
  - File: `src-tauri/src/playback/mod.rs`
  - Action: Export a new experimental module for the plugin editor host POC.
  - Notes: Keep the new seam explicit and narrow so it does not appear to be a production-ready host abstraction.

- [x] Task 2: Implement the backend command surface for the native editor POC
  - File: `src-tauri/src/playback/plugin_editor_poc.rs`
  - Action: Create a focused Rust module that defines the experimental host state, validates the target plugin path or test identifier, opens or resolves the dedicated Tauri host window, attaches the plugin editor native AppKit view, and manages close/teardown behavior.
  - Notes: Scope this to macOS with compile-time guards. Own lifecycle in Rust, not in React. Include logging for create, attach, resize, focus, close, and destroy events. Current implementation validates the VST3 bundle path, resolves a bundle binary from the macOS package, loads the factory through `vst3` + `libloading`, creates a minimal host application / component handler / plug frame, synchronizes component state into the edit controller when the plugin uses separate objects, connects component/controller `IConnectionPoint` pairs when available, requests `ViewType::kEditor`, attaches the returned `IPlugView` into an AppKit subview, logs lifecycle events, promotes the target reservation to an attached session only after the session is stored, keeps the host window resize affordance gated on `IPlugView::canResize()`, preserves bounded geometry sync callbacks for attached-editor positioning, and falls back to the standard macOS VST3 install roots when no explicit path or env override is provided. Manual validation is still required before claiming broader compatibility across real plugins.

- [x] Task 3: Register the experimental backend command in the app command graph
  - File: `src-tauri/src/lib.rs`
  - Action: Import the POC command(s) and add them to the central `tauri::generate_handler![]` registration.
  - Notes: Keep naming explicit, such as `plugin_editor_poc_open`, so the command remains clearly experimental.

- [x] Task 4: Add any required dependency and platform gating for the host proof
  - File: `src-tauri/Cargo.toml`
  - Action: Add the minimal dependency set needed for the chosen implementation route, starting with direct AppKit/Tauri parenting and adding `vst3` bindings if the POC opens the editor directly through VST3 APIs.
  - Notes: Do not add broad cross-platform host frameworks unless the direct route fails. If a fallback crate such as `augmented_baseview` is selected, document why it is necessary. The current proof now uses the existing Tauri + objc2/AppKit baseline plus direct `vst3` and `libloading` dependencies for bundle loading and editor creation.

- [x] Task 5: Define the dedicated POC host window shell
  - File: `src-tauri/capabilities/default.json`
  - Action: Extend window capabilities to include a dedicated experimental host window label pattern.
  - Notes: Keep permissions limited to what the POC needs.

- [x] Task 6: Add a minimal frontend action wrapper for the experimental command
  - File: `src/services/actions.ts`
  - Action: Add a small `invoke()` wrapper that asks the backend to open the native plugin editor POC for a selected plugin path or fixed test target.
  - Notes: The wrapper should remain thin; lifecycle stays backend-owned.

- [x] Task 7: Add a dedicated host window entrypoint with minimal or empty HTML chrome
  - File: `plugin-host-poc.html`
  - Action: Create a dedicated multi-page app entrypoint for the experimental host window.
  - Notes: Keep this page intentionally minimal; it exists to give Tauri a window shell, not to render the plugin UI.

- [x] Task 8: Add a minimal boot file for the host window entrypoint
  - File: `src/windows/plugin-host-poc/boot.tsx`
  - Action: Mount a minimal provider tree or no-op host component needed for the Tauri window to load correctly.
  - Notes: Avoid introducing a full React UI. If a minimal status shell is needed, keep it non-interfering with the native host surface.

- [x] Task 9: Wire a controlled trigger into the existing plugin UI
  - File: `src/windows/main/components/plugins.tsx`
  - Action: Add an explicit control or alternative path that opens the experimental native editor host instead of the existing HTML plugin-settings window.
  - Notes: Preserve the current plugin-settings behavior unless the POC is intentionally replacing it behind a temporary experimental trigger.

- [x] Task 10: Cover the frontend trigger behavior with existing Vitest patterns
  - File: `src/windows/main/components/plugins.test.tsx`
  - Action: Add or update tests to verify the experimental trigger path, including window label/use or backend action invocation behavior.
  - Notes: Follow existing `WebviewWindow` and action mocking patterns already used in the file.

- [x] Task 11: Decide whether to retain, reuse, or bypass the current plugin-settings window
  - File: `src/windows/plugin-settings/index.tsx`
  - Action: Either leave the current HTML plugin-settings screen untouched or document a narrow change if it becomes the launch point for the POC.
  - Notes: The preferred route is to leave it intact and keep the POC in its own window/surface.

- [x] Task 12: Document the experimental workflow and architecture impact
  - File: `docs/development-guide.md`
  - Action: Add developer instructions for running the macOS-only POC, including prerequisites, plugin path expectations, and manual validation steps.
  - Notes: Mark the workflow as experimental and macOS-only.

- [x] Task 13: Document the architecture boundary of the native host proof
  - File: `docs/architecture.md`
  - Action: Add a brief description of the experimental native plugin editor host seam, including that it uses a Tauri native window shell and AppKit subview attachment rather than HTML embedding.
  - Notes: Explicitly state that this is a proof seam, not the final plugin-host architecture.

### Acceptance Criteria

- [x] AC 1: Given the app is running on macOS in a build that includes the experimental POC, when the user triggers the native plugin editor proof for a valid test plugin, then a dedicated Tauri-managed host window opens without crashing the main app.
- [ ] AC 2: Given the host window opens for a valid test plugin, when the backend initializes the editor host, then the plugin's native editor view is attached into the macOS window content hierarchy rather than rendered through HTML.
- [ ] AC 3: Given the plugin editor view is attached, when the user interacts with visible plugin controls, then the editor responds to pointer and keyboard focus normally enough to demonstrate live interaction.
- [ ] AC 4: Given the host window is open with a plugin editor attached, when the window is resized, then the native host surface and attached editor remain visible and do not leave the app in a broken or obviously stale state.
- [ ] AC 5: Given the host window is open, when focus moves between the main window and the editor host window, then focus transitions do not freeze the app or orphan the editor window.
- [ ] AC 6: Given the host window is open, when the user closes the host window, then the plugin editor host tears down cleanly without crashing, hanging, or preventing a later reopen attempt.
- [ ] AC 7: Given the user closes the host window after one successful run, when the user triggers the POC again, then the app can reopen the experimental editor host within the same app session.
- [x] AC 8: Given the user triggers the POC with an invalid or unavailable plugin path, when the backend validates the request, then the app fails gracefully with a logged error or surfaced failure state and no orphaned host window.
- [x] AC 9: Given the experimental trigger exists in the frontend, when automated frontend tests run, then the trigger path is covered using the existing Vitest `renderWithProviders` and Tauri mocking patterns.
- [x] AC 10: Given the feature remains in-repo after implementation, when a developer reads the documentation, then the development guide and architecture docs clearly describe the macOS-only experimental status, setup expectations, and what the POC does and does not prove.

### Implementation Status Note

- The repo now contains a backend-owned macOS native host seam that validates a `.vst3` bundle path, discovers a fallback target from the standard macOS VST3 install roots, loads the bundle binary with `libloading`, creates a minimal VST3 component/controller/editor stack with `vst3`, and attempts to attach the returned editor view into an AppKit subview inside a dedicated host window.
- The lifecycle seam now keeps one authoritative active target reservation for the session, marks close as an explicit teardown phase, clears stale reservations when no host window survives, and performs the initial focus/size sync only after the live session has been stored.
- Resize remains capability-driven at the window level: the host window stays bounded by default, while the backend still forwards bounded geometry sync callbacks needed to keep the attached editor positioned correctly.
- Story 2.7 narrows the credibility claim with one explicit unsupported list rather than a supported allowlist. The bounded host seam now presumes discovered bundles are eligible unless they have demonstrated host-seam failures severe enough to add them to the unsupported list; current blocked examples are `HALion Sonic.vst3`, `Groove Agent SE.vst3`, `HALion Sonic SE.vst3`, and `Kontakt 7.vst3`.
- The proof is still experimental. It does not yet prove compatibility across arbitrary plugins or complete the full Steinberg host contract for every interaction path, so manual validation and follow-up hardening are still required before promoting any ACs beyond the currently verified scope.

## Additional Context

### Dependencies

Deep-investigation result:

- Existing repo baseline already includes the core macOS-native pieces needed for a proof: `tauri` with `macos-private-api`, `objc2`, `objc2-app-kit`, `objc2-foundation`, `objc2-quartz-core`, and `objc2-metal`.
- `vst3 = 0.3.0` is available as a low-level Rust binding layer for the VST 3 API. Its docs make clear that it provides COM bindings rather than a high-level host abstraction, which is acceptable for a narrow experimental POC but raises implementation complexity.
- `augmented_baseview = 0.1.3` is available as an audio-plugin-UI-oriented windowing crate and is the strongest candidate fallback if the Tauri/AppKit route needs a purpose-built plugin-editor host window abstraction.
- Recommendation for the spec: start with direct Tauri/AppKit hosting as the primary implementation route, and treat `augmented_baseview` as a fallback path to evaluate only if the direct route fails on lifecycle or view-parenting constraints.
- The POC depends on a valid local VST3 plugin that exposes a native editor on macOS; selecting and documenting one known-good test plugin is part of making the proof reproducible.
- The implementation depends on the existing multi-window Vite/Tauri structure, which means adding a new HTML entrypoint and ensuring it is bundled with the desktop app.

### Testing Strategy

Testing strategy should use three layers:

- Small frontend Vitest coverage for the trigger path, following existing `WebviewWindow` mocking patterns in `src/windows/main/components/plugins.test.tsx`.
- Focused Rust unit coverage where practical for non-UI pieces such as argument validation, window-label policy, and lifecycle state guards in the experimental host module.
- Mandatory manual macOS validation checklist for the true POC proof: open window, attach editor, interact with controls, resize, refocus main window and return, close editor, reopen editor, and confirm the app does not stall or leave orphaned state.

Suggested manual validation sequence:

1. Launch the app on macOS with the POC enabled.
2. Trigger the experimental editor host for the known-good test plugin.
3. Verify the host window opens and the native editor becomes visible.
4. Interact with several editor controls to confirm live input handling.
5. Resize the host window and confirm the editor remains usable.
6. Switch focus between the main window and the host window several times.
7. Close the host window and verify the app remains responsive.
8. Reopen the host window in the same session and verify the editor can attach again.
9. Trigger the POC with an invalid plugin target and verify graceful failure behavior.

### Notes

- This POC is specifically about proving native window/surface hosting constraints inside Tauri on macOS.
- Success is defined as a native editor window opening, attaching the editor view, allowing interaction, handling resize/focus correctly enough for a minimal proof, and closing without destabilizing the app.
- Documentation impact is required as part of the final spec: if the POC introduces a retained dev workflow or experimental architecture seam, the spec must identify which docs to update and the acceptance criteria for those updates.
- Highest-risk technical area: plugin-editor lifecycle ownership across Tauri window teardown, AppKit subview parenting, and any main-thread restrictions imposed by the VST3 editor path.
- Second highest-risk area: macOS event/focus behavior when mixing a Tauri webview shell with a foreign native plugin editor view.
- The POC should not be judged as a transport or plugin-host architecture success beyond proving the native editor lifecycle. Audio routing, MIDI, automation, preset management, and persistence remain separate future work.
- If the direct Tauri/AppKit route fails but `augmented_baseview` succeeds, that outcome is still valuable: the spec should allow the implementation to record that Tauri-native parenting was the hypothesis under test and that a dedicated audio-plugin windowing layer became necessary.
