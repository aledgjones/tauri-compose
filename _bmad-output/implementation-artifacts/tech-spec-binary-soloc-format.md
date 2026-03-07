---
title: "Binary .soloc Project File Format"
slug: "binary-soloc-format"
created: "2026-03-07"
status: "ready-for-dev"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  [
    "Rust",
    "Tauri",
    "bincode 1.3",
    "serde",
    "serde_repr",
    "semver",
    "rustc-hash",
  ]
files_to_modify:
  - "src-tauri/src/files.rs"
  - "src-tauri/Cargo.toml"
  - "src-tauri/src/migrations/mod.rs"
code_patterns: ["bincode", "serde", "serde_repr", "tauri-command", "FxHashMap"]
test_patterns: ["no existing tests for files.rs — manual verification only"]
---

# Tech-Spec: Binary .soloc Project File Format

**Created:** 2026-03-07

## Overview

### Problem Statement

`.soloc` files are currently serialized as `gzip(minified JSON)`. JSON encodes all field names as strings on every object, which is verbose even after compression. This wastes disk space and adds unnecessary parse overhead on load.

### Solution

Replace the gzip+JSON payload with a binary format: a small header containing `MAGIC` bytes and the app version string, followed by a `bincode`-encoded `ProjectData` struct. No chunking required — the entire project state is one contiguous encoded blob. This yields significantly smaller files with faster serialize/deserialize.

### Scope

**In Scope:**

- New binary save (`save_project_to_file`) using `bincode`
- New binary load (`load_project_from_file`) using `bincode`
- Header stores the **app semver string** (e.g. `"1.19.0"`) rather than an opaque integer — matches the current migration system's approach
- Restructure `src-tauri/src/migrations/` as a **typed binary migration system**: keyed by semver string, migration fns operate on typed Rust structs (not `serde_json::Value`). Additive changes use `#[serde(default)]` with no migration fn. Breaking changes get a frozen struct snapshot + typed `fn(OldProjectData) -> ProjectData`.
- Add `bincode` crate dependency
- Remove `flate2` dependency (only used in `files.rs`)
- Remove the old JSON-based `migrate_project_data` function

**Out of Scope:**

- Reading old gzip+JSON `.soloc` files (complete replacement; no backward compat)
- UI changes
- Any change to the `Score`, `Ui`, `UniqueId`, or `Preferences` structs themselves
- Compression on top of bincode (bincode alone is smaller than gzip+JSON in practice; add later if benchmarks justify it)

---

## Context for Development

### Codebase Patterns

- All Tauri backend state lives in `src-tauri/src/`. Rust only; no TypeScript changes needed.
- `AppContext` (in `lib.rs`) holds `ui: Ui`, `score: Score`, `preferences: Preferences`, `next_id: UniqueId`, `current_file_path: Option<String>`.
- The save payload is `{ ui, score, next_id }` — `preferences` and `current_file_path` are excluded (same as today).
- `Score`, `Ui`, and `UniqueId` (= `u64`) all derive `serde::Serialize` + `serde::Deserialize` — bincode works transparently.
- **`serde_repr` enums**: `Tab`, `Tool`, and several screen-state enums use `#[derive(Serialize_repr, Deserialize_repr)]` with `#[repr(u8)]`. These serialize as their integer discriminant — bincode reads/writes the raw `u8`, fully compatible.
- **`FxHashMap` keys**: `LayoutType` (used as map key in `engraves`) is `#[repr(u8)]` with `Serialize_repr`. JSON requires string keys, but bincode serializes maps as length-prefixed pairs — any serializable key type works. No issue.
- **`f32` fields**: present in `engraves.rs` (e.g. `note_space_ratio`, beam slant values). Bincode encodes IEEE 754 binary directly — works fine.
- **No `#[serde(skip)]` fields** anywhere in `Score` or `Ui` — full round-trip guaranteed.
- **`flate2`**: imported only in `files.rs` — safe to remove from `Cargo.toml`.
- **`semver`**: used in `files.rs` (for migrations) and `src/bin/release.rs`. Crate stays; the `use semver::Version` import moves from `files.rs` to `migrations/mod.rs`.
- **`serde_json`**: used in `preferences.rs`, `score/players.rs`, `score/flows.rs`, and bin tools — stays as a dep, just removed from `files.rs` imports.
- **`migrations/v1_19_0.rs`**: existing JSON migration module. Becomes dead code after this change. Leave in place (historical reference); `migrations/mod.rs` rewrite will no longer reference it.
- **No existing tests** for `save_project_to_file` / `load_project_from_file` — manual verification is the testing approach.
- `save_project_to_file` and `load_project_from_file` are `#[tauri::command(rename_all = "snake_case")]` functions.
- `flate2` is only imported in `files.rs` — safe to remove from `Cargo.toml` once replaced.
- `serde_json` remains needed elsewhere (debug fixtures, SCVI manifest, etc.) — do not remove it.

### Files to Reference

| File                                  | Purpose                                                                |
| ------------------------------------- | ---------------------------------------------------------------------- |
| `src-tauri/src/files.rs`              | Current save/load implementation — full replacement target             |
| `src-tauri/src/lib.rs`                | `AppContext` struct definition; `migrations` mod declaration           |
| `src-tauri/src/migrations/mod.rs`     | Migration entry point — full rewrite to typed binary system            |
| `src-tauri/src/migrations/v1_19_0.rs` | Existing JSON migration (becomes dead code; leave in place)            |
| `src-tauri/src/score/mod.rs`          | `Score` struct — all fields serializable, no skip attributes           |
| `src-tauri/src/score/engraves.rs`     | `FxHashMap<LayoutType, Engrave>` + `f32` fields — both bincode-safe    |
| `src-tauri/src/score/entries/mod.rs`  | `FxHashMap<Tick, Vec<UniqueId>>` — bincode-safe                        |
| `src-tauri/src/ui/mod.rs`             | `Ui` struct — uses `serde_repr` enums (u8 discriminants, bincode-safe) |
| `src-tauri/src/utils.rs`              | `UniqueId = u64`                                                       |
| `src-tauri/Cargo.toml`                | Add `bincode = "1.3"`, remove `flate2`                                 |

### Technical Decisions

- **bincode** chosen over MessagePack (`rmp-serde`) for smallest output: bincode does not encode field names or type tags; output size is dominated by actual data values. `rmp-serde` with map keys would be larger.
- **No compression layer** on top: bincode-encoded project data is already very compact. The old format used gzip precisely because JSON field names are repetitive — that problem disappears with bincode. Add `zstd` later if profiling shows benefit.
- **Header layout** (variable, but tiny):
  ```
  [0..4]             MAGIC = b"SOLO"              (4 bytes)
  [4]                version_len: u8              (byte length of version string)
  [5..5+vlen]        app_version: str             (UTF-8, e.g. "1.19.0")
  [5+vlen..]         bincode payload              (variable)
  ```
  Storing the real semver app version means you can inspect a file and know exactly which app version wrote it. `version_len` is written as `u8::try_from(version_str.len()).expect("version string too long")` — panics at compile-time on absurdly long versions, safe in practice.
- **Migrations going forward**: bincode is **not self-describing** — it stores no field names or counts. This means `#[serde(default)]` does **not** work for additive changes in bincode (unlike serde_json). **Every struct change, including new fields, requires a migration entry.** The migration system uses "struct epochs": each registered entry is a `(breaking_at_version, fn(&[u8]) -> Result<ProjectData, String>)` pair, where the fn handles both deserializing the old byte layout (via a frozen intermediate struct) **and** converting to the current `ProjectData`. `apply` finds the first migration where `file_ver < breaking_at_version` and dispatches to it; if none matches, deserializes directly into the current struct. Files from future app versions are rejected with a clear error.
  - **All struct changes** (additive or breaking): require a migration fn + frozen old-struct snapshot.
  - The frozen struct lives in `migrations/` (e.g. `pre_x_y_z.rs`), derives `Serialize, Deserialize`, and matches the exact field layout of the old format.
  - Migration fn signature: `fn(bytes: &[u8]) -> Result<ProjectData, String>` — takes raw bytes, deserializes using the frozen struct, maps fields to current `ProjectData`.

---

## Implementation Plan

### Tasks

- [ ] **Task 1: Add `bincode` to `Cargo.toml`, remove `flate2`**
  - File: `src-tauri/Cargo.toml`
  - Action: Add `bincode = "1.3"`; remove `flate2 = "1.1.9"`
  - Notes: `flate2` confirmed only imported in `files.rs` — safe to remove

- [ ] **Task 2: Define `ProjectData` struct and `MAGIC` constant in `files.rs`**
  - File: `src-tauri/src/files.rs`
  - Action: Remove imports `flate2`, `MIGRATIONS`, `semver::Version`, `serde_json::Value`; add `use crate::migrations`, `use serde::{Deserialize, Serialize}`
  - Action: Add `const MAGIC: &[u8; 4] = b"SOLO";` and `pub struct ProjectData { pub ui: Ui, pub score: Score, pub next_id: UniqueId }` with `#[derive(Serialize, Deserialize)]`
  - Action: Delete the entire `migrate_project_data` function
  - Notes: `ProjectData` is `pub` so `migrations/` can reference it as `crate::files::ProjectData`. Do NOT add `use bincode;` — `bincode::` is used as a path expression directly.

- [ ] **Task 3: Rewrite `save_project_to_file`**
  - File: `src-tauri/src/files.rs`
  - Action: Replace function body to write: 4-byte `MAGIC` + 1-byte `version_len` (via `u8::try_from(...).expect(...)`) + UTF-8 semver string from `env!("CARGO_PKG_VERSION")` + `bincode::serialize(&project_data)` payload. Also spec atomic write: write to a temp file first (`file_path + ".tmp"`), then `fs::rename` to final path — prevents corrupt files on crash mid-write.
  - Notes: See full implementation below; post-save logic (`state.current_file_path`, `app.emit`) unchanged

  ```rust
  #[tauri::command(rename_all = "snake_case")]
  pub fn save_project_to_file(
      app: AppHandle,
      state: State<'_, AppContextState>,
      file_path: String,
  ) -> Result<(), String> {
      let mut state = state.lock().unwrap();

      let project_data = ProjectData {
          ui: state.ui.clone(),
          score: state.score.clone(),
          next_id: state.next_id,
      };

      let payload = bincode::serialize(&project_data)
          .map_err(|e| format!("Failed to serialize project data: {}", e))?;

      let version_str = env!("CARGO_PKG_VERSION").as_bytes();
      let version_len = u8::try_from(version_str.len())
          .expect("CARGO_PKG_VERSION too long for header");

      let mut file_bytes = Vec::with_capacity(4 + 1 + version_str.len() + payload.len());
      file_bytes.extend_from_slice(MAGIC);
      file_bytes.push(version_len);
      file_bytes.extend_from_slice(version_str);
      file_bytes.extend_from_slice(&payload);

      // Atomic write: write to temp then rename
      let tmp_path = format!("{}.tmp", file_path);
      fs::write(&tmp_path, &file_bytes)
          .map_err(|e| format!("Failed to write temp file: {}", e))?;
      fs::rename(&tmp_path, &file_path)
          .map_err(|e| format!("Failed to finalize file: {}", e))?;

      state.current_file_path = Some(file_path);
      app.emit("soloc://recents-changed", ()).ok();

      Ok(())
  }
  ```

- [ ] **Task 4: Rewrite `load_project_from_file`**
  - File: `src-tauri/src/files.rs`
  - Action: Replace function body to: validate `MAGIC`, read `version_len`, read version string, **reject future-version files with a clear error**, call `migrations::apply(file_app_version, &bytes[header_end..])`, populate state
  - Notes: Post-load logic (ephemeral reset, `emit`, `sync_sampler_with_score`) unchanged from current implementation. Future-version guard:
    ```rust
    let current_ver = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let file_ver = semver::Version::parse(file_app_version)
        .map_err(|_| format!("Invalid version in file header: '{}'", file_app_version))?;
    if file_ver > current_ver {
        return Err(format!(
            "This file was saved by a newer version of the app ({}). Please upgrade.",
            file_app_version
        ));
    }
    ```
    Add `use semver` to `files.rs` imports for this guard (separate from `migrations/mod.rs` which also uses it).

- [ ] **Task 5: Rewrite `migrations/mod.rs` — typed semver-keyed binary migration system**
  - File: `src-tauri/src/migrations/mod.rs`
  - Action: Replace entire file. Remove `mod v1_19_0` declaration (file stays on disk but is no longer compiled). Do NOT add `use bincode;` — use as a path expression.

  ```rust
  use semver::Version;
  use crate::files::ProjectData;

  /// Migration fn: takes raw bytes of the OLD format, deserializes using a frozen struct,
  /// and returns current ProjectData. Each fn owns both the deserialization and conversion.
  type MigrateFn = fn(bytes: &[u8]) -> Result<ProjectData, String>;

  /// Registered migrations in ASCENDING version order.
  /// Each entry: (version_where_struct_changed, migrate_fn).
  ///
  /// `apply` dispatches to the first entry where file_ver < breaking_ver.
  /// If no entry matches, the file was saved with the current struct — deserialize directly.
  ///
  /// NOTE: bincode is NOT self-describing. EVERY struct change (including new fields)
  /// requires a migration entry + frozen old-struct snapshot in a `pre_x_y_z.rs` module.
  /// #[serde(default)] does NOT work for additive changes in bincode.
  static MIGRATIONS: &[(&str, MigrateFn)] = &[
      // Example — uncomment and implement when first struct change ships:
      // ("2.0.0", pre_2_0_0::migrate),
  ];

  pub fn apply(file_version: &str, bytes: &[u8]) -> Result<ProjectData, String> {
      let file_ver = Version::parse(file_version)
          .map_err(|e| format!("Invalid version in file header '{}': {}", file_version, e))?;

      // Find applicable migration: first entry where file was saved before the breaking change
      for (breaking_version_str, migrate_fn) in MIGRATIONS {
          let breaking_ver = Version::parse(breaking_version_str)
              .map_err(|e| format!("Invalid migration version '{}': {}", breaking_version_str, e))?;
          if file_ver < breaking_ver {
              return migrate_fn(bytes);
          }
      }

      // No migration needed — file struct matches current ProjectData
      bincode::deserialize(bytes)
          .map_err(|e| format!("Failed to deserialize project data: {}", e))
  }
  ```

  **Pattern for adding a migration** (when `ProjectData` fields change):
  1. Create `src-tauri/src/migrations/pre_x_y_z.rs` with a frozen snapshot of the old `ProjectData` struct (exact field names + types as they were before the change)
  2. Implement `pub fn migrate(bytes: &[u8]) -> Result<crate::files::ProjectData, String>`: deserialize `bytes` into the frozen struct using `bincode::deserialize`, then construct and return current `ProjectData`
  3. Add `mod pre_x_y_z;` to this file and register `("x.y.z", pre_x_y_z::migrate)` in `MIGRATIONS`
  4. Bump `FORMAT_VERSION` comment in `files.rs` if desired (cosmetic)

- [ ] **Task 6: Build and verify**
  - Action: `cd src-tauri && cargo build`
  - Notes: Expect clean compile. Confirm: `flate2` removed, `bincode` resolves, no unused import warnings in `files.rs`

### Acceptance Criteria

- [ ] **AC 1:** Given a project is open, when the user saves to a `.soloc` file, then the file starts with bytes `[0x53, 0x4F, 0x4C, 0x4F]` (`"SOLO"`) followed by a 1-byte version-string length and the current app semver as UTF-8 bytes (e.g. `[0x06]` + `"1.19.0"`)

- [ ] **AC 2:** Given a valid binary `.soloc` file saved by the current app version, when `load_project_from_file` is called, then the project state is fully restored (`ui`, `score`, `next_id`) with no errors

- [ ] **AC 3 (deferred):** Given a valid binary `.soloc` file saved by an older app version with no struct changes, when `load_project_from_file` is called, then it loads successfully. *(Verifiable only after a future version ships binary files — note for future verification)*

- [ ] **AC 4 (deferred):** Given a valid binary `.soloc` file saved by an older version that has a registered migration, when `load_project_from_file` is called, then `migrate_fn` runs and the project loads correctly. *(Verifiable only after first migration is registered)*

- [ ] **AC 5:** Given an old gzip+JSON `.soloc` file, when `load_project_from_file` is called, then it returns an error containing `"Invalid .soloc magic"`

- [ ] **AC 6:** Given a corrupt or truncated file (< 5 bytes), when `load_project_from_file` is called, then it returns `"File too small to be a valid .soloc file"`

- [ ] **AC 7:** Given a file with valid magic but a truncated version header, when `load_project_from_file` is called, then it returns `"Truncated .soloc header"`

- [ ] **AC 8:** Given a `.soloc` file whose embedded version is greater than the current app version, when `load_project_from_file` is called, then it returns an error containing `"newer version"` and the file's version string

- [ ] **AC 9:** Given a save operation is interrupted mid-write (simulated by checking no `.tmp` file is left behind after a successful save), then the final `.soloc` file is complete and valid

---

## Additional Context

### Dependencies

- **Add**: `bincode = "1.3"` — battle-tested, `serde`-compatible binary serializer. Uses fixed-size integer encoding and no schema overhead.
- **Remove**: `flate2 = "1.1.9"` — confirm not used outside `files.rs` before removing.
- `serde_json` stays — used by debug fixtures, SCVI tooling, and other commands.
- `semver` stays — now used in `migrations/mod.rs` for version comparison (was previously used only in `files.rs`; import moves there).

### Testing Strategy

Manual test:

1. Open app → create a project → save → hex dump the file: confirm first 4 bytes = `53 4F 4C 4F` (`SOLO`), byte 5 = version string length (e.g. `06` for `"1.19.0"`), following bytes = ASCII semver string
2. Close and reopen the saved file → confirm project loads correctly with all state intact
3. Attempt to open an old gzip+JSON `.soloc` file → confirm error is surfaced (or at minimum logged — see frontend note)
4. Confirm no `.tmp` file is left behind after a successful save (atomic write cleanup)

Automated: no existing test harness for Tauri file I/O commands — out of scope for this spec.

### Notes

- `bincode` v1.3 (not v2) — v2 has a different API surface and is still stabilizing. v1.3 uses `bincode::serialize` / `bincode::deserialize` directly.
- **bincode is NOT self-describing.** Unlike serde_json, it stores no field names or counts. `#[serde(default)]` does NOT provide free additive-change compatibility. Every struct change (including new fields) requires a registered migration fn and a frozen old-struct snapshot.
- All structs in the payload chain already derive `Serialize + Deserialize`; no struct changes needed at initial implementation.
- The header semver string is written using `env!("CARGO_PKG_VERSION")` at compile time — always matches the binary that wrote the file.
- `MIGRATIONS` starts empty at initial implementation. `apply` simply deserializes into current `ProjectData`. The first entry is added when the first struct change ships.
- **`v1_19_0.rs`**: the existing JSON migration file becomes a dead `.rs` file on disk once `mod v1_19_0` is removed from `mod.rs`. Rust will not compile it. It is intentionally left on disk as a historical reference — do not delete it, but do not add a `mod` declaration for it either.
- **Frontend error surfacing:** `load_project_from_file` returns `Result<(), String>`. Check whether the frontend already handles error results from this Tauri command and surfaces them to the user (e.g. via a dialog). If not, a corrupted or incompatible file produces zero user feedback. This is pre-existing behaviour carried over from the JSON format — fixing it is out of scope for this spec but should be logged as a follow-up.
- **Atomic write** is implemented via write-to-temp + rename. On macOS/Linux, `fs::rename` is atomic at the OS level. On Windows it may fail if the target path already exists — if cross-platform atomic writes become a concern, use the `tempfile` crate's `NamedTempFile::persist` API.
