# Story 1.7: Keep Versioning and Offline Desktop Behavior Intact for Tuplet Files

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a maintainer,
I want tuplet-enabled files to follow existing versioning and desktop workflow rules,
so that the feature ships without destabilizing Compose's document and offline guarantees.

## Scope Note

- This story is the release-discipline and desktop-workflow hardening seam for Epic 1, not a second persistence rewrite.
- Story 1.6 already proved deterministic tuplet save/load behavior through the existing `.soloc` path. Story 1.7 preserves that path's versioning contract, offline workflow expectations, and future migration discipline.
- Do not run release scripts in this story. The goal is to keep the implementation compatible with the documented `prepare-release:*` and `finalize-release` flow, not to perform release preparation.
- Do not add any frontend-owned tuplet truth or alternate offline persistence layer. Tuplet state remains backend-owned and must continue to flow through the existing Rust/Tauri save, load, parse, and playback seams.

## Acceptance Criteria

1. Given tuplets alter persisted behavior, when tuplet-authored files are produced, then `.soloc` header versioning, `score.meta.app_version`, and migration registration remain consistent with repo release-flow rules, and the implementation does not bypass the project's documented migration discipline.
2. Given tuplet support is available, when a composer uses standard desktop open, save, and reopen workflows offline, then the full tuplet authoring path of popover creation plus sequencer note entry works without network dependency, and offline-first desktop behavior remains unchanged for the supported MVP workflow.
3. Given this epic is complete, when later editing and rendering epics build on it, then they rely on one exact backend-owned timing model shared across save/load, notation derivation, and playback, and no separate frontend truth layer is introduced.

## Tasks / Subtasks

- [x] Preserve the existing versioning contract for tuplet-bearing `.soloc` files (AC: 1)
  - [x] Verify the save path continues to stamp the same concrete semver into both the `.soloc` header and `score.meta.app_version` via the current `CURRENT_APP_VERSION` flow in `src-tauri/src/files.rs` and `src-tauri/src/soloc.rs`.
  - [x] Verify load still treats the container header `app_version` as the authoritative migration key, validates it against payload metadata, and applies migrations before typed deserialization.
  - [x] Review `src-tauri/src/migrations/mod.rs` and `src-tauri/src/migrations/next.rs` for explicit compatibility with the current tuplet payload contract, and only add migration work if a real older released payload would fail without transformation.
  - [x] Keep release discipline documentation accurate without running `prepare-release:*` or `finalize-release`.
- [x] Preserve offline-first desktop document workflows for tuplet-authored projects (AC: 2)
  - [x] Keep frontend open/save flows on the existing Tauri invoke path in `src/services/actions.ts`; do not introduce remote fetches, web persistence, or background sync assumptions.
  - [x] Verify backend file commands in `src-tauri/src/files.rs` continue to support local open, save, reopen, `current_file_path`, recent-file, and transport-sync behavior for tuplet-bearing projects.
  - [x] Add or extend focused tests near the owning seams proving tuplet files still round-trip through standard desktop file commands without online dependencies or workflow regressions.
- [x] Protect the shared backend-owned timing model for later tuplets work (AC: 3)
  - [x] Verify later seams continue to consume the same persisted tuplet data model rather than reconstructing timing or ownership in TypeScript.
  - [x] Keep `ui.flow_id` and other existing UI state rules intact; do not introduce a parallel frontend active-flow or tuplet-source-of-truth state path as part of file workflow handling.
  - [x] Update developer-facing docs if implementation clarifies the versioning, migration, or offline workflow contract for tuplet-bearing files.
- [x] Validate the touched workflow seams with the standard repo checks
  - [x] Run the relevant Rust coverage close to `src-tauri/src/files.rs` and migration ownership seams.
  - [x] Run `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`, `npm run lint`, and `npm run check-types` after implementation.

## Dev Notes

### Story Foundation

- Epic 1's purpose is not only to persist tuplets, but to ship them safely in Compose's native desktop document model.
- Story 1.6 established that tuplet-bearing projects already round-trip through the single typed `ProjectFile` plus `.soloc` path and that no speculative tuplet migration was needed for unreleased legacy data.
- Story 1.7 extends that seam by protecting release-flow rules, header and payload version consistency, offline-first workflows, and the one-backend-timing-model rule that later notation and playback work depends on.
- This story is the final guardrail before Epic 2 and later epics build on the persisted tuplet contract.

### Technical Requirements

- Keep `src-tauri/src/files.rs` as the authoritative load/save workflow owner for app-version validation, migration application, local file handling, and app-state application.
- Keep `src-tauri/src/soloc.rs` as the shared container-format owner for `.soloc` header encoding and payload transport.
- Preserve the current concrete semver discipline:
  - file header `app_version` remains a real semver string
  - `score.meta.app_version` remains aligned with the header value
  - pending development transforms live in `src-tauri/src/migrations/next.rs`, not in ad hoc load-path branching
- Preserve the current desktop workflow shape:
  - open/save/reopen operate on local file paths
  - `current_file_path` remains synchronized after load/save
  - transport state continues to resync from loaded score state
  - recent-file behavior continues to layer on top of the same Tauri file commands
- Offline-first means no network dependency is introduced for tuplet creation, sequencer note entry, save, load, reopen, or recent-file workflows.
- The same backend-owned tuplet data must remain the source for persistence, parser derivation, and later playback work. Do not reconstruct a separate TypeScript timing truth for tuplets.

### Architecture Compliance

- The architecture requires `.soloc` container header `app_version` to act as the authoritative migration key, with the JSON payload validated against it before applying migrations and before typed deserialization.
- Tuplet-bearing scores must continue to use the same `.soloc` transport and typed deserialize path as all other projects. No tuplet-specific repair path, alternate serializer, or frontend reconciliation layer is allowed.
- Rust remains the source of truth for score state, tuplet ownership, and timing semantics.
- Later notation and playback stories rely on the persisted tuplet model already established in Epic 1. Story 1.7 must not weaken that by adding workflow-specific alternate truth.

### Library / Framework Requirements

- Backend work stays in the current Rust/Tauri stack using `serde`, `semver`, the existing migration registry, and the shared `.soloc` helper module.
- Frontend file workflow work, if any, stays in the current Tauri invoke layer in `src/services/actions.ts`.
- No new external dependency is expected for this story.

### File Structure Requirements

- Primary backend ownership seams:
  - `src-tauri/src/files.rs`
  - `src-tauri/src/soloc.rs`
  - `src-tauri/src/migrations/mod.rs`
  - `src-tauri/src/migrations/next.rs`
- Frontend workflow touchpoints:
  - `src/services/actions.ts`
  - `src/services/actions.test.ts` if open/save/reopen invoke behavior changes or needs regression coverage
- Documentation touchpoints:
  - `docs/development-guide.md`
  - `docs/data-models.md`
  - `docs/architecture.md`
  - `docs/project-context.md` only if project guardrails or versioning summaries need clarification
- Keep notation, rendering, and playback implementation changes out of scope unless they are required to preserve the shared file-workflow contract.

### Testing Requirements

- Verify tuplet-bearing projects continue to save and load through the existing `.soloc` path with aligned header and payload app versions.
- Verify version mismatch and container validation behavior remain explicit; tuplets must not weaken rejection of invalid `.soloc` metadata.
- Verify migration behavior remains explicit and version-driven, with no hidden tuplet-specific repair logic.
- Verify load of tuplet-bearing projects still resets ephemeral state correctly, updates `current_file_path`, and syncs transport state.
- Verify frontend open/save/reopen flows still call the existing Tauri commands and do not introduce any network-backed behavior.
- Run the standard Rust and frontend validation commands after implementation.

### Previous Story Intelligence

- Story 1.6 already proved that tuplets round-trip safely through `ProjectFile` plus `.soloc`, preserved ownership for host-owned crossing tones and child-owned tuplet tones, and documented the current no-migration rationale.
- Story 1.6 also added regression coverage around reopen behavior, state reset, and transport sync. Story 1.7 should extend or reuse those seams instead of creating a new document-workflow harness.
- The no-migration conclusion from Story 1.6 is conditional, not permanent: retain migration review discipline and add real migration work only if an older released payload genuinely requires transformation.

### Git Intelligence Summary

- Recent relevant commits:
  - `c859a420 feat(1.6): tuplet file loading regression tests`
  - `4c8a33bf feat(1.5): reconcile tuplets over existing material`
  - `37a3ee54 feat(1.4): add tuplet note entry in sequencer`
- The recent pattern is disciplined seam-by-seam hardening in Rust-owned code paths. Continue that pattern here: strengthen the existing save/load and workflow seams rather than broadening scope into release automation or unrelated UI changes.

### Documentation Impact

- Confirm the docs still state that versioning is synced across `package.json`, `Cargo.toml`, and `tauri.conf.json`, and that human-only release scripts promote pending migrations.
- Confirm the docs still describe the `.soloc` header `app_version` as authoritative and the payload metadata as validated against it.
- Confirm the docs still describe tuplet-bearing files as using the same offline-first local file workflow and single typed deserialize path as non-tuplet projects.
- If implementation changes or clarifies any of those rules, update the relevant docs in the same change.

### Project Structure Notes

- Current repo guidance says versioning is kept in sync across `package.json`, `Cargo.toml`, and `tauri.conf.json`, currently `1.25.0`.
- Current repo guidance also says persisted-data migrations are authored in `src-tauri/src/migrations/next.rs` during development, then promoted by a human during release preparation.
- `src/services/actions.ts` currently routes open/save commands through Tauri `invoke()` calls and then updates recent files locally; that is the correct desktop workflow seam to preserve.
- `src-tauri/src/files.rs` already owns container parsing, version validation, migration application, local file path updates, and transport synchronization. Prefer extending tests there over inventing a detached workflow harness.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 1 / Story 1.7 acceptance criteria
- Source: `_bmad-output/planning-artifacts/prd.md` — desktop application requirements, offline capabilities, and migration expectations
- Source: `_bmad-output/planning-artifacts/architecture.md` — persisted `.soloc` contract and single backend-owned timing model
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — desktop platform strategy and offline-first workflow expectations
- Source: `docs/project-context.md` — current versioning rules and migration workflow summary
- Source: `docs/development-guide.md` — release and migration process, shared `.soloc` logic, and fixture/versioning workflow
- Source: `docs/data-models.md` — authoritative `.soloc` header version rules and migration system behavior
- Source: `docs/architecture.md` — current `.soloc` container and typed-deserialize load pipeline summary
- Source: `_bmad-output/implementation-artifacts/1-6-round-trip-tuplet-data-through-save-load-and-migration.md`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Inspect `src-tauri/src/files.rs`, `src-tauri/src/soloc.rs`, and the migration registry to confirm tuplet-bearing files still follow the documented header-version, payload-version, and migration pipeline.
- Reuse or extend focused tests in the existing file-workflow seams to protect offline local save/load, reopen state application, version validation, and migration behavior.
- Update docs only where implementation clarifies or changes the documented versioning, migration, or offline workflow contract.

### Debug Log References

- Verified existing regression coverage in `src-tauri/src/files.rs` already protects the authoritative header-versus-payload app-version contract, migration ordering, legacy non-tuplet load behavior, tuplet round-trips, and loaded-project transport/current-file-path reset behavior.
- Verified existing regression coverage in `src/services/actions.test.ts` already protects the offline desktop open/save/save-as/load/load-from-path invoke flow and recent-file updates without introducing any network-backed persistence path.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features`, `npm run lint`, and `npm run check-types` on 2026-03-15.

### Completion Notes List

- Story prepared to preserve repo versioning and offline desktop guarantees for tuplet-bearing files without widening scope into release execution.
- The controlling constraint is that tuplets remain inside the single Rust-owned `.soloc` plus typed `ProjectFile` pipeline with explicit migration review.
- Frontend workflow changes, if any, must stay limited to the existing local Tauri invoke seams and must not introduce a second persistence or timing truth.
- Existing implementation already satisfied the story's file-workflow and migration contract: `CURRENT_APP_VERSION` remains the save-time source for both header and payload metadata, load still validates header-versus-payload app-version before migration and typed deserialize, and `src-tauri/src/migrations/next.rs` needs no tuplet-specific step for currently released payloads.
- Existing test coverage from Story 1.6 already covered the required backend and frontend seams closely enough that no production code change was warranted for Story 1.7.
- Added an architecture-doc clarification that tuplet-bearing files keep the same offline-first local open/save/reopen/current-file-path/recent-file path and transport resync behavior as non-tuplet projects.

### File List

- `_bmad-output/implementation-artifacts/1-7-keep-versioning-and-offline-desktop-behavior-intact-for-tuplet-files.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/architecture.md`

### Change Log

- 2026-03-15: Validated existing tuplet versioning, migration, and offline desktop workflow seams; no production code changes were required, and developer-facing architecture guidance was clarified for the offline file-workflow contract.
