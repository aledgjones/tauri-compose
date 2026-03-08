# Solo Compose — Development Guide

## Prerequisites

- **Node.js** v18+ — [Download](https://nodejs.org/)
- **Rust** (latest stable) — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Xcode Command Line Tools** (macOS) — `xcode-select --install`

## Installation

```bash
git clone https://github.com/aledjones/compose.git
cd compose
npm install
```

## Development Commands

| Command                 | Description                                                                       |
| ----------------------- | --------------------------------------------------------------------------------- |
| `npm run start:dev`     | Start Tauri in development mode (hot reload for frontend, Rust recompile on save) |
| `npm run start:release` | Build production release (.dmg)                                                   |
| `npm run dev`           | Vite dev server only (no Tauri — useful for frontend-only work)                   |
| `npm run build`         | TypeScript check + Vite production build                                          |
| `npm run check-types`   | TypeScript type checking (`tsc --noEmit`)                                         |
| `npm run lint`          | ESLint check on `src/**/*.{ts,tsx}`                                               |
| `npm run lint:fix`      | ESLint auto-fix                                                                   |
| `npm run format`        | Prettier format check                                                             |
| `npm run format:fix`    | Prettier auto-format                                                              |
| `npm run preview`       | Preview production build locally                                                  |

## Validation Before Commit

```bash
npm run lint && npm run check-types && npm test && cd src-tauri && cargo clippy
```

All four must pass. Clippy warnings are treated as errors — do not suppress with `#[allow(...)]`.

Pre-commit hooks (via `lint-staged` + `simple-git-hooks`) run ESLint and Prettier automatically on staged files. If hooks are not running after a fresh clone, register them with `npx simple-git-hooks`.

## Versioning

Version is kept in sync across three files:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

Use the custom release binary:

```bash
npm run release:patch   # 1.23.1 → 1.23.2
npm run release:minor   # 1.23.1 → 1.24.0
npm run release:major   # 1.23.1 → 2.0.0
```

The release tool must be compiled first: `npm run release:build`

Release scripts are human-only. Agents must never run `release:patch`, `release:minor`, `release:major`, or any equivalent release/tag/publish flow.

For persisted-data or load/save changes, use this order:

1. Implement and validate the feature.
2. Add any required migration under the next release version that will ship the change.
3. Have a human run the release command last so the version bump and release notes include the full slice.

Avoid manually editing synced version files just to reserve a migration target.

## Instrument Tools

| Command                           | Description                                               |
| --------------------------------- | --------------------------------------------------------- |
| `npm run generate-manifest`       | Generate instrument manifest JSON from sample directories |
| `npm run generate-manifest:build` | Compile the manifest generator                            |
| `npm run generate-scvi`           | Compress instruments into SCVI binary containers          |
| `npm run generate-scvi:build`     | Compile the SCVI compressor                               |
| `npm run inspect-scvi`            | Inspect SCVI container contents                           |
| `npm run inspect-scvi:build`      | Compile the SCVI inspector                                |

## Visual Regression Fixtures

See [tests/fixtures/README.md](../tests/fixtures/README.md) for full details.

Fixtures enable manual visual regression of score rendering: a human opens each generated `.soloc` file in the app and visually confirms rendering is correct.

| Command                  | Description                                           |
| ------------------------ | ----------------------------------------------------- |
| `npm run build:fixtures` | Convert all `tests/fixtures/source/*.json` → `.soloc` |

### Adding a new fixture

1. Create `tests/fixtures/source/<concern>/<name>.json` — a valid project JSON payload (see [tests/fixtures/README.md](../tests/fixtures/README.md))
2. Run `npm run build:fixtures`
3. Open the generated `tests/fixtures/built/<concern>/<name>.soloc` in the app and verify rendering
4. Commit the `.json` file only (`.soloc` files are gitignored)

### `soloc.rs` — shared container logic

The container encoding functions (`build_soloc_container`, `compress_project_payload`, `decompress_project_payload`) and all `.soloc` format constants live in `src-tauri/src/soloc.rs`. Both the production save path (`files.rs`) and the `build-fixtures` binary share this module via:

- `files.rs` — `use crate::soloc::*`
- `build_fixtures.rs` — `#[path = "../soloc.rs"] mod soloc`

When making changes to the container format, update `soloc.rs` and rebuild fixtures with `npm run build:fixtures`.

## Build Configuration

### Vite (Frontend)

- Multi-page app: 8 HTML entry points configured in `rollupOptions.input`
- Dev server on port 1420 (strict port)
- HMR enabled, ignores `src-tauri/` directory

### Tauri (Native Shell)

- macOS: Dark theme, overlay title bar, transparent background (`#0a0a0a`)
- Minimum macOS version: 10.14
- Bundle target: DMG
- Asset protocol enabled for cover images: `$DATA/Solo Compose/covers/**`

### Rust (Backend)

Release profile optimized for maximum performance:

- `panic = 'abort'`
- `codegen-units = 1`
- `lto = "fat"`
- `opt-level = 3`
- `strip = true`

## Testing

See [TESTING.md](../TESTING.md) for full details. Quick reference:

| Command                 | Description                                 |
| ----------------------- | ------------------------------------------- |
| `npm test`              | Run all TypeScript tests once               |
| `npm run test:watch`    | Watch mode                                  |
| `npm run test:coverage` | Run with v8 coverage report                 |
| `cargo test`            | Run all Rust unit tests (from `src-tauri/`) |

Test files are colocated next to the source files they test (e.g. `utils.ts` → `utils.test.ts`).

For **visual regression testing** (manual, human-driven), see [tests/fixtures/README.md](../tests/fixtures/README.md).

## Git Conventions

All commit messages must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<optional scope>): <description>

[optional body]
```

### Types

| Type       | Use for                                                  |
| ---------- | -------------------------------------------------------- |
| `feat`     | New user-facing feature                                  |
| `fix`      | Bug fix                                                  |
| `chore`    | Tooling, config, dependency updates (no production code) |
| `refactor` | Code restructure with no behaviour change                |
| `test`     | Adding or updating tests only                            |
| `docs`     | Documentation only                                       |
| `perf`     | Performance improvement                                  |
| `ci`       | CI/CD pipeline changes                                   |

### Examples

```
feat(sequencer): add swing quantisation
fix(pitch-utils): correct octave calculation for middle C
chore: upgrade Tauri to 2.11
test(duration): add round-trip coverage for from_ticks
docs: update development guide with testing setup
```

- Use the imperative mood in the description ("add" not "added")
- Keep the description under 72 characters
- Reference Jira tickets in the body when applicable: `SC-123`

## IDE Setup

Recommended: VS Code with extensions:

- Tauri
- rust-analyzer
- ESLint
- Prettier
- Tailwind CSS IntelliSense
