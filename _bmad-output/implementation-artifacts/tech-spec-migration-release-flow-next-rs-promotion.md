---
title: "Migration release flow with next.rs promotion"
slug: "migration-release-flow-next-rs-promotion"
created: "2026-03-08"
status: "Completed"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  - "Rust 2021"
  - "Tauri 2"
  - "semver crate"
  - "serde_json"
  - "lazy_static"
files_to_modify:
  - "package.json"
  - "src-tauri/src/migrations/mod.rs"
  - "src-tauri/src/migrations/next.rs"
  - "src-tauri/src/files.rs"
  - "src-tauri/src/bin/release.rs"
  - "docs/development-guide.md"
  - "docs/data-models.md"
  - ".github/copilot-instructions.md"
code_patterns:
  - "Migration registry uses #[path] module aliases plus ordered (version, function) tuples in lazy_static"
  - "Project load path parses semver app_version from the .soloc header and from every registered migration key before applying migrations"
  - "Project load rewrites score.meta.app_version to CURRENT_APP_VERSION after migrations complete"
  - "Release binary is a side-effect-heavy Rust CLI that validates a clean git workspace, bumps synced versions, runs cargo check, generates release notes, then stages, commits, tags, and pushes in one flow"
  - "package.json exposes only release:patch|minor|major wrappers around the compiled release binary"
test_patterns:
  - "Rust unit tests colocated with migration modules"
  - "files.rs load-path tests validate app_version and migration behavior"
  - "Release tooling currently has helper functions but no dedicated automated tests"
---

# Tech-Spec: Migration release flow with next.rs promotion

**Created:** 2026-03-08

## Overview

### Problem Statement

The current migration workflow requires unreleased persisted-data changes to be assigned to a concrete future release version before the human release process runs. That makes development awkward, couples schema work to an assumed version too early, and leaves migration promotion to a manual release-time process.

### Solution

Introduce a cumulative development migration module at `src-tauri/src/migrations/next.rs` and split the human release flow into two explicit commands: a prepare-release phase that bumps versions, promotes `next.rs` into the concrete release migration, updates the registry, and generates release notes; then a separate commit/tag/push phase performed by a human after review.

### Scope

**In Scope:**

- Define `next.rs` as the single cumulative unreleased migration target during development.
- Update migration loading/registry rules so development can test pending migrations while release artifacts contain only concrete versioned migrations.
- Update the release tooling to promote `next.rs` to the actual release version during a human-triggered prepare-release step.
- Define the post-release reseeding of a fresh `next.rs` as part of `finalize-release`, after the release commit and tag are created.
- Update developer and agent documentation for the revised persisted-data workflow.

**Out of Scope:**

- Redesigning the underlying migration transformation API unless required to support the new flow.
- Changing non-migration release behavior beyond splitting it into prepare vs commit/tag/push phases.
- Changing `.soloc` container structure or non-migration persistence semantics.

## Context for Development

### Codebase Patterns

- `src-tauri/src/migrations/mod.rs` hard-codes shipped migrations through `#[path = "1.25.0.rs"] mod v1_25_0;` and a `lazy_static!` `Vec<(version, migrate_fn)>`, so adding `next.rs` affects both module declarations and registry ordering.
- `src-tauri/src/files.rs` uses `semver::Version::parse()` on the `.soloc` header `app_version` and on every registry key inside `migrate_project_data()`, so a literal `"next"` key cannot work without an explicit special-case or revised registry shape.
- `src-tauri/src/files.rs` always rewrites `score.meta.app_version` to `CURRENT_APP_VERSION` after migration and also rejects header/JSON version mismatches before migration runs, so any dev-time `next` strategy must preserve concrete semver values inside saved files.
- `src-tauri/src/bin/release.rs` currently has one monolithic human-triggered flow: verify clean git state, derive the bumped semver from `package.json`, update `package.json`/`Cargo.toml`/`tauri.conf.json`, run `cargo check`, generate release notes from git commits, then `git add`, `git commit`, `git tag`, and `git push`.
- `package.json` exposes only `release:patch`, `release:minor`, and `release:major` wrappers, so the proposed two-step human workflow will likely require new script entry points or new release-binary subcommands.
- Project docs and agent rules currently require migrations to target the next concrete shipped version during development, so this proposal changes both developer workflow and AI instructions.

### Files to Reference

| File                                 | Purpose                                                                                                 |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| `package.json`                       | Release script entry points that likely need to split into prepare-release vs finalize-release commands |
| `.release-prep.json`                 | Temporary prepare→finalize handoff manifest storing the prepared release facts to validate and consume  |
| `src-tauri/src/migrations/mod.rs`    | Current migration registry and module declarations                                                      |
| `src-tauri/src/migrations/1.25.0.rs` | Example shipped migration module with colocated unit tests                                              |
| `src-tauri/src/migrations/next.rs`   | New cumulative development-only migration module to be created and reseeded                             |
| `src-tauri/src/files.rs`             | Load-time migration application and version parsing rules                                               |
| `src-tauri/src/bin/release.rs`       | Human release automation that must be split and made migration-aware                                    |
| `docs/development-guide.md`          | Developer release workflow documentation                                                                |
| `docs/data-models.md`                | Migration-system documentation and persisted-data rules                                                 |
| `.github/copilot-instructions.md`    | Agent instructions that currently mandate version-targeted migrations during development                |

### Technical Decisions

- Development state is defined structurally: `src-tauri/src/migrations/next.rs` exists, `src-tauri/src/migrations/mod.rs` registers exactly one pending `next` migration descriptor, and `next.rs` is available for local/test execution.
- Prepared/tagged release state is defined structurally: `src-tauri/src/migrations/next.rs` does not exist, `src-tauri/src/migrations/mod.rs` contains only concrete versioned migration descriptors, and the release artifact/tag therefore ships with no `next` migration.
- `next.rs` uses logical key `"next"` in development semantics, but the implementation must reconcile that with the current semver-only migration loop in `files.rs`.
- `prepare-release` must determine whether a migration is pending from a single source of truth inside `next.rs`: an ordered collection of pending migration steps. An empty collection means scaffold-only state; one or more steps means a real migration exists to promote.
- `next.rs` must not remain in the released/tagged state; release preparation promotes it to a concrete versioned module before the release artifact is built.
- There is exactly one unreleased migration bucket at a time, and it may contain multiple migration steps for the upcoming shipped version.
- The human release process is split into two explicit triggers: (1) `prepare-release:{patch|minor|major}` — compute the target version, promote migrations, update synced version files, generate release notes, write a prep manifest, and leave changes for review; (2) `finalize-release` — validate the prep manifest, create the release commit and tag, reseed `next.rs`, create the post-release development commit, and push branch HEAD plus the tag.
- `prepare-release` computes the release version first, then uses that computed version for migration promotion and all synced file rewrites, then generates release notes and writes the prep manifest last.
- `finalize-release` consumes a temporary `.release-prep.json` manifest so it never has to guess which prepared version, promoted migration file, or release state it is finalizing.
- Reinstating a fresh `next.rs` happens inside `finalize-release` after the release tag is created, so the tagged release contains only concrete versioned migrations and no placeholder `next` entry.
- Post-release scaffold commits must be excluded from future generated release notes by a deterministic filter rule rather than allowed to pollute the next release note window.
- Because `release.rs` currently auto-commits and tags, implementation will likely need a command/API split rather than a simple additive hook.

## Implementation Plan

### Tasks

- [x] Task 1: Introduce an explicit pending-migration model that supports development-only `next` behavior without breaking semver-based file loading.
  - File: `src-tauri/src/migrations/mod.rs`
  - Action: Replace the raw `(version, migrate_fn)` tuple registry with a typed migration descriptor that can distinguish shipped semver migrations from the single pending `next` migration.
  - Notes: Preserve ordered execution semantics; the registry must allow the loader to apply all shipped migrations whose target version is greater than the file header version, then apply the pending migration only when the source tree is in development state (i.e. the registry still contains the `next` descriptor).
- [x] Task 2: Add the cumulative development migration template and register it for local/test execution.
  - File: `src-tauri/src/migrations/next.rs`
  - Action: Create a new scaffold module with the same `fn(Value) -> Result<Value, String>` signature, colocated unit tests, and an ordered collection of pending migration step functions that `migrate()` executes in sequence.
  - Notes: The ordered collection is the single source of truth for whether a migration exists to promote: an empty collection means scaffold-only state; one or more steps means a real migration exists. Avoid separate boolean flags that can drift from the actual code.
- [x] Task 3: Update file-loading migration selection so saved `.soloc` files keep concrete semver headers while development builds can still run the pending `next` migration.
  - File: `src-tauri/src/files.rs`
  - Action: Refactor `migrate_project_data()` to consume the new registry shape, continue parsing concrete file header versions with `semver`, and special-case the pending migration so the literal logical key `"next"` never has to be parsed as semver.
  - Notes: The loader must continue rejecting malformed file header versions and header/JSON mismatches; `score.meta.app_version` should still be rewritten to `CURRENT_APP_VERSION` after migration completes.
- [x] Task 4: Add regression coverage for shipped vs pending migration behavior.
  - File: `src-tauri/src/files.rs`
  - Action: Add or extend unit tests to prove that versioned migrations are applied only when the header version is older than the shipped target and that the pending `next` migration path can be invoked in development scenarios without corrupting version handling.
  - Notes: Include edge cases for files already at current version and for registry entries that mix shipped and pending migrations.
- [x] Task 5: Split release automation into a prepare phase and a finalize phase.
  - File: `src-tauri/src/bin/release.rs`
  - Action: Refactor the release binary into subcommands or equivalent command dispatch so it supports `prepare <patch|minor|major>` and `finalize` as separate human-triggered flows.
  - Notes: `prepare` should validate a clean git workspace, compute the release version first, inspect the pending step collection in `next.rs`, promote `next.rs` to the concrete versioned migration file only when real pending transforms exist, otherwise remove `next.rs` from the prepared release tree, rewrite `src-tauri/src/migrations/mod.rs` into shipped-only state with deterministic aliases like `#[path = "1.26.0.rs"] mod v1_26_0;`, bump synced version files, run any existing lockfile refresh step, generate release notes, and write `.release-prep.json` without committing or tagging.
- [x] Task 6: Define finalize-release behavior that preserves a clean tagged release while restoring post-release development state.
  - File: `src-tauri/src/bin/release.rs`
  - Action: Implement `finalize` so it assumes a reviewed prepared working tree, creates the release commit, tags that commit, then reseeds a fresh `next.rs` and updated registry state in a follow-up development commit before pushing branch HEAD and the new tag.
  - Notes: `finalize` must read and validate `.release-prep.json`, refuse to continue if the working tree no longer matches the prepared release facts, create the release commit and tag first, then reseed `next.rs`, restore the pending registry entry, remove the prep manifest, create the post-release development commit, and push branch HEAD plus the tag. The implementation must document failure recovery for the case where the release commit/tag succeeds but the reseed commit or push fails.
- [x] Task 7: Expose the split release workflow through package scripts.
  - File: `package.json`
  - Action: Replace or augment the current `release:patch|minor|major` scripts with prepare-phase entry points such as `prepare-release:patch|minor|major` plus a separate `finalize-release` command.
  - Notes: Keep `release:build` intact; document whether the legacy one-shot scripts are removed or retained as deprecated aliases.
- [x] Task 8: Update migration-system and release-process documentation to reflect the new workflow.
  - File: `docs/development-guide.md`
  - Action: Rewrite the persisted-data release guidance to describe development-time use of `next.rs`, the prepare-release command, human review/build expectations, and the finalize command.
  - Notes: Explicitly state that release scripts remain human-only and that the tag points to the concrete migration state without `next`.
- [x] Task 9: Update persistence architecture documentation for pending migration promotion semantics.
  - File: `docs/data-models.md`
  - Action: Replace the current version-targeted-during-development guidance with the new registry model: shipped versioned migrations plus a single cumulative pending `next` migration that is promoted during release preparation.
  - Notes: Document the order of operations, released-state invariants, and how `score.meta.app_version`/container header versions remain concrete semver values.
- [x] Task 10: Update AI-agent instructions to align with the new migration authoring rule.
  - File: `.github/copilot-instructions.md`
  - Action: Replace instructions that currently require targeting the next concrete release version during development with rules that direct persisted-data work to `src-tauri/src/migrations/next.rs`, then require the human prepare/finalize release flow to ship and reseed migrations.
  - Notes: Keep the prohibition on agents running release/tag/publish flows.
- [x] Task 11: Validate the full workflow end to end at the spec level.
  - File: `src-tauri/src/bin/release.rs`
  - Action: Add targeted tests or extract deterministic helper functions so promotion, prep-manifest generation/validation, release-note filtering, commit split behavior, failure recovery, and post-release reseeding can be verified without relying solely on manual git-side effects.
  - Notes: If the binary remains difficult to test directly, isolate file-transformation and state-transition logic into pure helpers and test those.

### Acceptance Criteria

- [ ] AC 1: Given development work introduces persisted-data changes before a release version is chosen, when migration code is added, then the changes are authored in `src-tauri/src/migrations/next.rs` instead of a guessed concrete version file.
- [ ] AC 2: Given a development build loads an older `.soloc` project that requires both shipped migrations and pending unreleased transforms, when `load_project_from_file()` runs, then shipped semver migrations execute in order and the pending `next` migration executes without requiring `"next"` to be parsed as a semver version.
- [ ] AC 3: Given a `.soloc` file header or embedded `score.meta.app_version` contains an invalid or mismatched concrete version, when the loader processes the file, then it still fails with the existing version-validation behavior rather than silently accepting malformed data.
- [ ] AC 4: Given the project is in normal development state, when the pending migration registry is inspected, then there is exactly one cumulative pending migration target represented by `src-tauri/src/migrations/next.rs` and one pending migration descriptor registered from `src-tauri/src/migrations/mod.rs`.
- [ ] AC 4.1: Given `src-tauri/src/migrations/next.rs` contains an empty pending-step collection, when `prepare-release` inspects pending migration state, then it deterministically recognizes that no concrete migration should be promoted for that release.
- [ ] AC 5: Given a human runs `prepare-release:{patch|minor|major}`, when the command completes successfully, then the target release version is computed first, synced version files are bumped to that exact version, any pending `next` migration steps are promoted into a concrete release migration file named for that exact version, the migration registry is rewritten into shipped-only state with deterministic Rust-safe aliases, release notes are updated, `.release-prep.json` is written, and no commit, tag, or push has happened yet.
- [ ] AC 5.1: Given `src-tauri/src/migrations/next.rs` contains no pending migration steps, when `prepare-release` completes successfully, then no new versioned migration file is created from the scaffold, `src-tauri/src/migrations/next.rs` is absent from the prepared release tree, and the prepared release state contains only the already-shipped migration set.
- [ ] AC 6: Given a human reviews and builds the prepared release state, when they run the finalize-release command, then the tool creates a release commit and tag for the prepared state, and that tagged release state contains no `next` migration file or registry entry.
- [ ] AC 6.1: Given `.release-prep.json` is missing, malformed, or no longer matches the prepared working tree, when `finalize-release` is run, then it fails before creating any new commit or tag.
- [ ] AC 7: Given finalize-release completes successfully, when the branch HEAD is inspected after the tag is created, then a fresh scaffolded `src-tauri/src/migrations/next.rs` with an empty pending-step collection and a corresponding pending registry entry exist again in a post-release development commit.
- [ ] AC 8: Given finalize-release pushes results to origin, when the remote receives the update, then the new release tag points to the concrete shipped migration commit and the branch HEAD points to the follow-up post-release development commit.
- [ ] AC 8.1: Given finalize-release succeeds through release commit and tag creation but fails before the post-release development commit or push, when the human inspects the repository, then the failure output clearly states the current state and the exact recovery action needed to restore the reseeded development scaffold and/or push the finished release.
- [ ] AC 9: Given the release scripts are listed in `package.json`, when a human inspects available release commands, then separate `prepare-release:{patch|minor|major}` and `finalize-release` entry points are present and the workflow is no longer exposed only as a single one-shot release command.
- [ ] AC 10: Given the migration and release workflow has changed, when documentation is updated, then [docs/development-guide.md](docs/development-guide.md), [docs/data-models.md](docs/data-models.md), and [.github/copilot-instructions.md](.github/copilot-instructions.md) explicitly describe the new `next.rs` authoring rule, prepare/finalize release phases, and the invariant that shipped releases do not contain `next`.
- [ ] AC 10.1: Given post-release scaffold commits are generated by `finalize-release`, when the next release notes are generated, then those scaffold commits are deterministically excluded from the changelog content.
- [ ] AC 11: Given the release binary has been refactored, when automated tests run, then migration-promotion, prep-manifest validation, release-note filtering, and post-release reseeding logic are covered by deterministic tests rather than only by manual release execution.
- [ ] AC 12: Given an older `.soloc` file that depends on a promoted release migration, when the shipped release build created by the prepare/finalize workflow loads that file, then it migrates successfully using only the concrete shipped migration set and no `next` migration entry is required.

### Documentation Impact

- Update [docs/development-guide.md](docs/development-guide.md) to describe the new two-command human release workflow, where `prepare` bumps versions/promotes migrations/generates release notes and `finalize` commits, tags, pushes, and restores the next development migration target after the release tag is created.
- Update [docs/data-models.md](docs/data-models.md) to document the new migration registry shape, the role of `src-tauri/src/migrations/next.rs`, the pending-step collection as the single source of truth for whether a migration exists to promote, and the rule that saved file versions remain concrete semver values even while unreleased transforms live under the logical pending migration bucket.
- Update release-process docs to describe `.release-prep.json`, what facts it stores, and how `finalize-release` consumes and removes it.
- Update [.github/copilot-instructions.md](.github/copilot-instructions.md) so AI agents stop targeting guessed future version files during development and instead use `next.rs` while still never running release commands.
- If script names change, update any related release references in [README.md](README.md) or other docs discovered during implementation so release guidance stays consistent across the repo.

## Additional Context

### Dependencies

- Rust `semver` parsing currently constrains migration key formats.
- Release automation depends on synced versions across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- Release-note generation depends on git commit history since the last tag and currently happens before the script creates the new release commit.
- Promotion logic depends on filesystem operations that rename or copy `src-tauri/src/migrations/next.rs` into a versioned module and rewrite `src-tauri/src/migrations/mod.rs` deterministically.
- Prepare/finalize handoff depends on a temporary `.release-prep.json` manifest that stores at least `previous_version`, `prepared_version`, `bump_type`, whether a migration was promoted, the promoted migration path if any, and enough prepared-state metadata to detect drift before finalize proceeds.
- Finalize logic depends on git command sequencing that can create a tagged release commit first and a post-release development commit second without forcing a third human-triggered command.

### Testing Strategy

- Validate development-time pending migration execution through Rust unit tests.
- Validate `files.rs` migration selection rules for development-time `next` behavior and for released builds that contain only concrete semver migrations.
- Validate release-preparation promotion logic, mod.rs alias generation, and prep-manifest creation with focused release-tool tests or extracted deterministic file transformation helpers, because the current binary is side-effect heavy.
- Validate that prepare-release leaves a reviewable working tree and that finalize-release performs the commit/tag/push phase separately.
- Validate finalize failure paths, including prep-manifest mismatch and post-tag failure recovery messaging.
- Validate load-path behavior for both concrete shipped migrations and the absence of `next` in released artifacts.
- Manual verification path: run prepare-release, inspect the promoted migration file and release notes, build/test the prepared state, run finalize-release, then verify the new tag points to the concrete release commit and branch HEAD contains the reseeded `next.rs` follow-up commit.

### Notes

- The release artifact should never ship with a placeholder `next` migration entry.
- The spec must define how the repo transitions from released state back to post-release development state without polluting the tagged release.
- This plan intentionally keeps the human workflow at two commands by having `finalize` create two commits in sequence: the tagged release commit and the post-release development reseed commit.
- The reseed scaffold commit message should use a deterministic marker that release-note generation explicitly filters out so the scaffolding churn does not appear in the next public release notes.
- If maintaining dual commits inside one finalize command proves too risky, implementation should preserve the release-state invariants first and then explicitly document why a third human-triggered reseed step would be required instead.

## Review Notes

- Adversarial review completed
- Findings: 10 total, 9 fixed, 1 skipped (`F8`, left undecided because it depends on whether older internal specs should be migrated now or in follow-up cleanup)
- Resolution approach: auto-fix
