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
npm run lint && npm run check-types && cd src-tauri && cargo clippy
```

All three must pass. Clippy warnings are treated as errors — do not suppress with `#[allow(...)]`.

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

## Instrument Tools

| Command                           | Description                                               |
| --------------------------------- | --------------------------------------------------------- |
| `npm run generate-manifest`       | Generate instrument manifest JSON from sample directories |
| `npm run generate-manifest:build` | Compile the manifest generator                            |
| `npm run generate-scvi`           | Compress instruments into SCVI binary containers          |
| `npm run generate-scvi:build`     | Compile the SCVI compressor                               |
| `npm run inspect-scvi`            | Inspect SCVI container contents                           |
| `npm run inspect-scvi:build`      | Compile the SCVI inspector                                |

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

No test framework is currently configured. Validation relies on:

- TypeScript strict mode (`noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`)
- ESLint (react-hooks, react-refresh, jsx-a11y, typescript-eslint)
- `cargo clippy` for Rust
- Manual testing

## IDE Setup

Recommended: VS Code with extensions:

- Tauri
- rust-analyzer
- ESLint
- Prettier
- Tailwind CSS IntelliSense
