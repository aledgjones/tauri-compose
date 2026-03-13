---
title: "Containerized .soloc Project File Format"
slug: "soloc-container-format"
created: "2026-03-07"
status: "Completed"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  ["Rust", "Tauri", "serde_json", "flate2", "semver", "custom binary container"]
files_to_modify:
  - "src-tauri/src/files.rs"
  - "src-tauri/src/migrations/mod.rs (only if a small helper/comment adjustment is useful; migration model remains app-version-chain based)"
  - "src-tauri/src/lib.rs (only if visibility or wiring changes are needed)"
  - "docs/project-overview.md"
  - "docs/architecture.md"
  - "docs/data-models.md"
  - ".github/copilot-instructions.md"
code_patterns:
  ["serde_json::Value", "gzip", "semver", "tauri-command", "container header"]
test_patterns: ["no existing tests for files.rs — manual verification only"]
---

# Tech-Spec: Containerized .soloc Project File Format

**Created:** 2026-03-07

## Overview

### Problem Statement

`.soloc` files currently behave like `gzip(minified JSON)`. This is workable, but the format feels closer to a raw transport dump than a deliberate project-file container. It also leaves little structure for future additions like thumbnails, preview metadata, embedded manifests, or other sidecar payloads.

### Solution

Keep JSON as the canonical persisted project model and keep the **container header app version** as the migration key, but wrap the JSON inside a lightweight custom `.soloc` container.

Initial implementation stores:

- a small binary header with `MAGIC`, container version, app version, and chunk count
- one required compressed JSON project chunk

The container is deliberately extensible so future versions can add optional chunks such as:

- thumbnail image
- preview metadata
- embedded diagnostic payloads
- additional manifest data

### Scope

**In Scope:**

- Keep JSON as the persisted data model
- Keep semver app version as the migration key, with the container header as the authoritative source
- Replace the implicit `gzip(JSON)` blob with a formal `.soloc` container
- Add a binary header and chunk table
- Store the current project JSON in a required compressed chunk
- Load by: parse container → extract JSON chunk → decompress → migrate `serde_json::Value` → typed deserialize once
- Save atomically via temp-file + rename
- Reject malformed containers with explicit errors
- Design the format so optional chunks can be added later without redesigning the whole file format
- Require header `app_version` and JSON `score.meta.app_version` to remain in sync
- Update developer/project documentation to describe the new container format and migration rules
- Update AI agent instructions so persisted-data structure changes always require a corresponding migration review/update

**Out of Scope:**

- Encryption or password protection
- Embedding thumbnails in the first implementation
- Replacing JSON migrations with typed binary migrations
- Moving to `bincode`, SQLite, protobuf, or zip-package storage
- Frontend UI for previews/thumbnails
- Backward compatibility for pre-container legacy `gzip(JSON)` `.soloc` files

---

## Context for Development

### Codebase Patterns

- All Tauri backend state lives in `src-tauri/src/`.
- The current save payload is effectively `{ ui, score, next_id }`.
- `score.meta.app_version` already exists and should remain in the JSON payload for display/inspection purposes, but after this change the container header `app_version` becomes the authoritative migration key.
- Current load behavior parses JSON into `serde_json::Value`, migrates that value, then deserializes subtrees. That general migration-first model is worth preserving.
- The current implementation partially applies state if one subtree deserializes and another does not. The new containerized format should tighten that flow: deserialize the whole migrated document into a single persisted project struct, and fail the load if that typed deserialize fails.
- `flate2` is already present and suitable for keeping gzip compression in the initial container format.
- `semver` is already used and should remain the migration-routing mechanism.
- `serde_json` remains the correct persistence format because it is migration-friendly and less tightly coupled to Rust struct layout than raw binary serialization.

### Files to Reference

| File                              | Purpose                                                              |
| --------------------------------- | -------------------------------------------------------------------- |
| `src-tauri/src/files.rs`          | Current save/load implementation — primary rewrite target            |
| `src-tauri/src/migrations/mod.rs` | Existing semver migration chain — should stay JSON-first             |
| `src-tauri/src/lib.rs`            | `AppContext` and Tauri command wiring                                |
| `src-tauri/src/score/meta.rs`     | `score.meta.app_version` source in persisted payload                 |
| `src-tauri/Cargo.toml`            | Only update if a small helper dep is needed; initial plan needs none |
| `docs/project-overview.md`        | High-level product/file-format description                           |
| `docs/architecture.md`            | Persistence architecture summary                                     |
| `docs/data-models.md`             | Migration system and persisted model documentation                   |
| `.github/copilot-instructions.md` | AI agent guardrails and repo-specific implementation requirements    |

---

## Format Design

### Design Principles

1. **JSON stays canonical.** This keeps migrations maintainable.
2. **Container header app version is the migration key.** Not every app version needs a migration entry.
3. **Container version tracks outer-file structure only.** It changes only if the binary container layout changes.
4. **Chunks enable future extensibility.** The file can later carry thumbnails or metadata without replacing the core format.
5. **One typed deserialize after migration.** No more piecemeal state assignment.
6. **Header/version consistency is mandatory.** Header `app_version` and JSON `score.meta.app_version` must match; mismatch means the file is invalid.

### Version Authority

- The container header `app_version` is the sole migration-routing key.
- The JSON payload field `score.meta.app_version` is retained as part of the canonical document for display and inspection.
- On save, both values must be normalized to `env!("CARGO_PKG_VERSION")`.
- On load, if header `app_version` and JSON `score.meta.app_version` differ, the file must be rejected as invalid before migrations are applied.
- After migrations run, the migrated JSON `score.meta.app_version` should be rewritten to the current app version so the persisted document remains aligned with the header value that would be written on the next save.

### Container Layout

The `.soloc` file becomes:

```text
[container header]
[chunk directory entries]
[chunk payloads]
```

### Container Header

Initial header layout:

```text
[0..5]    MAGIC = b"SOLOC"          (5 bytes)
[5]       container_version = 1      (u8)
[6]       app_version_len            (u8)
[7..]     app_version UTF-8 bytes    (variable)
[next]    chunk_count                (u16 LE)
```

Notes:

- `MAGIC` identifies this as the formal `.soloc` container.
- `container_version` is for the outer binary wrapper, not the project schema.
- `app_version` is the authoritative migration-routing key.
- `chunk_count` allows later expansion beyond one payload.
- This header is variable-length because `app_version` is variable-length UTF-8.

### Chunk Directory Entry

Each chunk gets a fixed-size directory record:

```text
type_code      u32 LE
flags          u16 LE
offset         u64 LE
length         u64 LE
```

Initial flags usage:

- bit 0 = compressed with gzip
- all other bits are reserved and must be `0`; any nonzero reserved flag bit is an invalid file in v1

Initial chunk types:

- `PROJECT_JSON = u32::from_le_bytes(*b"JSON")` — required

Encoding note:

- Chunk type codes are four-byte ASCII tags stored on disk exactly as those ASCII bytes.
- For Rust constants, use `u32::from_le_bytes(*b"JSON")` rather than a bare hex literal, so the code and on-disk bytes stay unambiguous.

Reserved future chunk types:

- `THUMBNAIL_PNG`
- `THUMBNAIL_JPEG`
- `PREVIEW_META`
- `DEBUG_INFO`

Implementation note: the first release should reject duplicate `PROJECT_JSON` chunks, fail if the required chunk is missing, and ignore unknown chunk types whose flags are otherwise valid.

### Required Project JSON Chunk

The required chunk contains:

- minified UTF-8 JSON bytes
- gzip-compressed in v1

The JSON document itself should continue to carry:

- `ui`
- `score`
- `next_id`

`score.meta.app_version` should be normalized to `env!("CARGO_PKG_VERSION")` before save so the payload metadata stays aligned with the container header.

On load, if the extracted JSON does not contain `score.meta.app_version` matching the header `app_version`, the file should be rejected before migration.

---

## Load and Save Flow

### Save Flow

1. Build persisted project document from app state
2. Normalize `score.meta.app_version` to current app version and copy that same version into the container header
3. Serialize to minified JSON
4. Compress JSON with gzip
5. Build header + one chunk directory entry + payload
6. Write to a temp path in the same directory as the destination file
7. Rename temp path over the final `.soloc` path so the replacement is atomic on the target filesystem
8. Update current file path and emit recents event

### Load Flow

1. Read file bytes
2. Validate `MAGIC`
3. Validate `container_version`
4. Read header `app_version`
5. Read and validate chunk directory
6. Locate required `PROJECT_JSON` chunk
7. Extract and decompress JSON payload
8. Parse to `serde_json::Value`
9. Validate that JSON `score.meta.app_version` exists and matches the header `app_version`
10. Run semver migration chain using the header `app_version`
11. Deserialize migrated JSON into a single persisted project struct
12. Only if successful, apply state to app context
13. Reset ephemeral UI state as current load flow already does
14. Emit state and sync sampler

### Directory and Bounds Validation

During load, the parser must also:

- reject files too small to contain the required header prefix
- reject headers whose declared chunk directory extends past the file end
- reject any chunk entry whose `offset + length` overflows `u64` or points outside the file bounds
- reject duplicate required chunks
- reject nonzero reserved flag bits
- ignore unknown chunk types only after their directory entry has been validated as structurally sound

### Why this flow

This preserves the good part of the current design:

- migrations run against flexible JSON first

While fixing the risky part:

- no partial state assignment from multiple independent `if let Ok(...)` branches
- header version is authoritative while still keeping JSON metadata visible and synchronized

---

## Implementation Plan

### Persisted DTO

Introduce a single persisted DTO in `files.rs`:

```rust
#[derive(Serialize, Deserialize)]
pub struct ProjectFile {
    pub ui: Ui,
    pub score: Score,
    pub next_id: UniqueId,
}
```

This is still close to the current payload, but load should deserialize into `ProjectFile` once, then apply it to state atomically.

### Tasks

- [x] **Task 1: Define container constants and DTO**
  - File: `src-tauri/src/files.rs`
  - Add `MAGIC`, `CONTAINER_VERSION`, chunk-type constants, and `ProjectFile`
  - Define chunk type constants with `u32::from_le_bytes(*b"....")` style helpers for unambiguous on-disk encoding
  - Keep JSON serialization via `serde_json`

- [x] **Task 2: Replace raw gzip blob save with structured container save**
  - File: `src-tauri/src/files.rs`
  - Serialize `ProjectFile` to minified JSON
  - gzip-compress JSON bytes
  - Build header, chunk directory, and payload bytes
  - Write atomically via temp path + rename in the same destination directory
  - Ensure header `app_version` and JSON `score.meta.app_version` are written with the same value

- [x] **Task 3: Replace raw gzip blob load with structured container load**
  - File: `src-tauri/src/files.rs`
  - Parse header and chunk directory
  - Extract `PROJECT_JSON`
  - Decompress and parse JSON
  - Validate header/json app-version consistency before migration
  - Run existing semver migration chain on `serde_json::Value` using the header `app_version`
  - Deserialize once into `ProjectFile`
  - Apply to state only after the full typed load succeeds

- [x] **Task 4: Tighten migration/load semantics**
  - File: `src-tauri/src/files.rs`
  - Remove the current partial subtree assignment pattern
  - If migrated JSON cannot deserialize into `ProjectFile`, return an error and leave state unchanged
  - If header and JSON versions differ, return an error and leave state unchanged

- [x] **Task 5: Keep migrations app-version keyed**
  - File: `src-tauri/src/migrations/mod.rs`
  - Preserve the current semver-chain model
  - Do not introduce a second schema-version migration system for the JSON payload
  - Container version is only for outer file-layout changes
  - No redesign of the migration registry is expected; changes here should be minimal or unnecessary unless a helper extraction is useful

- [x] **Task 6: Validate and document future chunk behavior**
  - File: `src-tauri/src/files.rs`
  - Unknown chunk types should be ignored for now unless they conflict with required chunks
  - Unknown/reserved flag bits should fail explicitly
  - Missing required `PROJECT_JSON` chunk should fail clearly
  - Duplicate required chunks should fail clearly
  - Bounds and integer-overflow checks should fail clearly before any chunk payload slicing

- [x] **Task 7: Build and manually verify**
  - Action: `cd src-tauri && cargo build`
  - Confirm save/load still works and malformed files return explicit errors
  - Confirm legacy raw `gzip(JSON)` `.soloc` files are intentionally rejected

- [x] **Task 8: Update project documentation**
  - Files: `docs/project-overview.md`, `docs/architecture.md`, `docs/data-models.md`
  - Replace references to `.soloc` as raw `gzip(JSON)` with the new containerized format description
  - Document that the container header `app_version` is the authoritative migration key
  - Document that `score.meta.app_version` remains in the payload but must match the header value
  - Document the rule that persisted-data shape changes require a migration review and, when needed, a new migration entry

- [x] **Task 9: Update AI agent instructions**
  - File: `.github/copilot-instructions.md`
  - Add an explicit project rule for AI agents: any change to persisted project data structures, serialized `.soloc` payload shape, or load/save semantics must include migration impact assessment
  - Require AI agents to add or update a migration whenever a persisted-data change would make older files load incorrectly without transformation
  - Require AI agents to update relevant docs when changing file format or migration behavior

---

## Acceptance Criteria

- [ ] **AC 1:** Given a project is saved, the resulting `.soloc` file starts with `SOLOC` magic and a valid container header
- [ ] **AC 2:** Given a valid v1 container file, load extracts the required JSON chunk, migrates it by app version, and restores project state successfully
- [ ] **AC 2a:** Given a raw legacy `gzip(JSON)` `.soloc` file without `SOLOC` magic, load fails rather than attempting backward compatibility
- [ ] **AC 3:** Given a file with invalid magic, load returns an error containing `Invalid .soloc magic`
- [ ] **AC 4:** Given a file with unsupported `container_version`, load returns an explicit unsupported-container-version error
- [ ] **AC 5:** Given a file missing the required `PROJECT_JSON` chunk, load returns an explicit missing-project-chunk error
- [ ] **AC 6:** Given a file with a corrupted compressed payload, load returns a decompression or parse error and leaves current state unchanged
- [ ] **AC 7:** Given migrated JSON cannot deserialize into `ProjectFile`, load fails and leaves current state unchanged
- [ ] **AC 8:** Given a successful save, no temp file remains and the final `.soloc` file is complete
- [ ] **AC 9:** Given a future file containing extra unknown optional chunks, the loader ignores those chunks and still loads the required project chunk successfully
- [ ] **AC 10:** Given a future implementation adds thumbnail chunks, the v1 container design already supports embedding them without redesigning the file format
- [ ] **AC 11:** Given header `app_version` and JSON `score.meta.app_version` differ, load returns an explicit version-mismatch error and leaves current state unchanged
- [ ] **AC 12:** Given any chunk entry sets nonzero reserved flag bits, load returns an explicit unsupported-flags error
- [ ] **AC 13:** Given a chunk entry whose bounds overflow or point outside file bounds, load returns an explicit bounds error before payload slicing
- [ ] **AC 14:** Project documentation is updated to describe the new `.soloc` container format, authoritative header versioning, and migration expectations
- [ ] **AC 15:** AI agent instructions explicitly require migration consideration for any persisted-data structure change and documentation updates for file-format changes

---

## Error Contract

Suggested explicit error messages:

- `File too small to be a valid .soloc file`
- `Invalid .soloc magic`
- `Unsupported .soloc container version: {version}`
- `Truncated .soloc header`
- `Truncated .soloc chunk directory`
- `Invalid .soloc version mismatch between header and project metadata`
- `Missing required PROJECT_JSON chunk`
- `Duplicate PROJECT_JSON chunk`
- `Unsupported .soloc chunk flags: {flags}`
- `Chunk points outside file bounds`
- `Chunk bounds overflow`
- `Failed to decompress project payload: ...`
- `Failed to parse project JSON: ...`
- `Failed to migrate project data: ...`
- `Failed to deserialize project file: ...`

---

## Technical Decisions

### Why not `bincode`

Raw binary serialization is smaller, but it couples the on-disk format too tightly to Rust struct layout and makes schema evolution much more cumbersome.

### Why not zip-package first

A zip package is a valid future direction, but it is heavier than necessary for the current need. A small custom container achieves the same “professional project file” feel while staying close to the current implementation.

### Why keep app version as migration key

This matches the current migration model and keeps the mental model simple:

- only app versions that change persisted data need migration entries
- semver ordering naturally applies chained migrations
- the container header value provides one authoritative migration key while the JSON metadata remains visible to humans and tooling

### Why add a separate container version

The app version describes the project payload lineage. The container version describes the outer binary wrapper. These are different concerns.

Examples:

- app version `1.24.0` may still use container version `1`
- app version `1.30.0` may still use container version `1`
- only a true file-wrapper redesign should bump container version

---

## Future Extensions

The container is intentionally designed for later additions such as:

- `THUMBNAIL_PNG` chunk for file previews in recents/browser UIs
- `PREVIEW_META` chunk for title/composer/updated-at without fully loading project JSON
- `DEBUG_INFO` chunk for support bundles or recovery metadata
- optional integrity chunk or checksum strategy
- optional alternate compression algorithm flag in a future container version

The key point: **the first implementation should only write one required JSON chunk, but it should not paint the format into a corner.**

---

## Testing Strategy

Manual verification:

1. Save a project and inspect the first bytes to confirm `SOLOC` magic
2. Confirm the header includes expected container version, app version length, and chunk count
3. Confirm the header `app_version` matches JSON `score.meta.app_version`
4. Reopen the file and verify full state restoration
5. Corrupt the header and confirm explicit load failure
6. Corrupt the payload and confirm explicit load failure with no partial state mutation
7. Confirm successful save leaves no `.tmp` file behind
8. Confirm a handcrafted container with an extra unknown chunk still loads if `PROJECT_JSON` is valid
9. Confirm a handcrafted container with mismatched header/json versions fails explicitly
10. Confirm a handcrafted container with nonzero reserved flag bits fails explicitly
11. Confirm a legacy raw `gzip(JSON)` `.soloc` file is rejected because backward compatibility is intentionally out of scope

Automated tests are out of scope for this spec, but the load helper functions should be written in a way that makes later unit coverage practical.

## Documentation Follow-Through

This implementation should ship with documentation updates, not as a separate cleanup pass.

Required documentation outcomes:

- `docs/project-overview.md` describes `.soloc` as a custom container with a header, chunk directory, and compressed JSON project chunk
- `docs/architecture.md` describes the persistence layer as containerized `.soloc` plus version-gated JSON migrations keyed by header `app_version`
- `docs/data-models.md` documents the authoritative-version rule, header/JSON version-sync requirement, and the expectation that persisted-data schema changes must be paired with migration review
- `.github/copilot-instructions.md` instructs AI agents that any persisted-data structure change must explicitly consider migrations and update docs when persistence behavior changes

Recommended wording for AI-agent guidance:

- If you change any persisted project-data structure, serialized JSON shape, or `.soloc` file-format behavior, you must assess migration impact.
- If older project files would fail to load correctly without transformation, add or update a migration in `src-tauri/src/migrations/` and register it in `mod.rs`.
- Do not merge persisted-data changes without corresponding migration and documentation updates when they are required.

---

## Notes

## Review Notes

- Adversarial review completed
- Findings: 10 total, 8 fixed, 2 skipped
- Resolution approach: auto-fix
- Validation completed in-session with `cargo build` and `cargo test --lib --manifest-path /Users/aledjones/compose/src-tauri/Cargo.toml`
- Full UI-driven manual save/load exercise was not run in this session

- This spec intentionally pivots away from the earlier `bincode` direction.
- The goal is a more deliberate, professional `.soloc` file format without sacrificing migration simplicity.
- JSON remains the durable project model; the binary container is the delivery wrapper.
- This design is specifically chosen to keep the door open for thumbnails and other embedded artifacts later.
