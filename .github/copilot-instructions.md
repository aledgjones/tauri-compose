# Copilot / AI agent quick instructions — Compose

Target: make an AI coding agent immediately productive in this repository.
Keep changes concise, deterministic and always respect the Rust ↔ TypeScript contract.

## Big picture (what matters)

- Desktop app: React + Vite frontend (TypeScript) + Tauri Rust backend (Skia renderer, audio engine).
- Frontend <-> Backend bridge: Tauri commands (Rust `#[tauri::command]`) invoked from `src/services/actions.ts` using `invoke(...)`.
- Rendering pipeline lives in Rust: parse → instructions → draw (see `src-tauri/src/instructions/*` and `src-tauri/src/renderer/*`).
- Frontend state and UI live under `src/` (notably `src/services/` and `src/windows/main/`).

## Fast onboarding map (open these first)

- UI + app shell: `src/windows/main/` (screens, components)
- Shared frontend types: `src/services/types.ts`
- Frontend actions → Tauri commands: `src/services/actions.ts`
- Rust score model & logic: `src-tauri/src/score/` (entries, tracks, components)
- Renderer / engraving: `src-tauri/src/instructions/` and `src-tauri/src/renderer/`
- Fonts / Bravura: `src/assets/fonts/*` and `src-tauri/src/renderer/fonts.rs`
- Entry points: `src/main.tsx` (frontend), `src-tauri/src/lib.rs` / `main.rs` (backend)

## Developer workflows (commands you will use)

- Start full dev app (builds Rust + hot reload): npm run start:dev
- Run frontend vite only: npm run dev
- Type-check frontend: npm run check-types
- Rust quick check (no-link): cd src-tauri && cargo check
- Lint frontend: npm run lint / npm run lint:fix
- Run Rust unit tests (crate): cd src-tauri && cargo test
- Build production app: npm run start:release (tauri build)

> MUST: Always run `npm run check-types` and `cd src-tauri && cargo check` after making changes. You are not finished until related errors _and_ warnings are fixed.

## Project-specific conventions & patterns

- Tauri commands are `snake_case` in Rust (e.g. `score_entries_tone_articulation_set`) and mapped in `src/services/actions.ts` to friendly names (e.g. `actions.score.tone.setArticulation`).
- Keep backend authoritative for domain rules (examples: articulation exclusivity & tick-unification live in Rust `src-tauri/src/score/*`).
- Shared model changes must be mirrored in both languages:
  - Rust: `src-tauri/src/score/components/...`
  - TS: `src/services/types.ts`
  - Actions: add/modify mapping in `src/services/actions.ts`
- Renderer expects pre-parsed instruction data; when changing drawing semantics update both parse + draw code in `src-tauri/src/instructions/`.
- Prefer non‑mutating APIs where practical (example: `Articulations::toggle` returns a new set to avoid borrow problems).
- Use `rustc_hash::FxHashMap`/`FxHashSet` in Rust score code for performance and consistency.

## Common edit checklist (feature / bugfix PR)

1. Update Rust model/logic under `src-tauri/src/score/` (if domain change).
2. Add or modify `#[tauri::command]` if front-end needs to call it.
3. Mirror type changes in `src/services/types.ts` and add action in `src/services/actions.ts`.
4. Update / add UI component under `src/ui/` or `src/windows/main/screens/`.
5. Update renderer (Rust) only if drawing/engraving semantics changed (`src-tauri/src/instructions/*`).
6. Run: `npm run check-types` && `cd src-tauri && cargo check` — resolve **all** related errors and warnings before proceeding.
7. Manual visual QA in dev app (`npm run start:dev`).

## Integration points & gotchas (be precise)

- Serialization: Rust uses Serde; many domain types are serialized as-transparent newtypes. Changing Rust serialization WILL require TS type updates.
- Keyboard shortcuts: look at `src/windows/main/hooks/use-keyboard-shortcuts.ts` for mappings that call `actions` directly.
- Fonts/glyphs: Bravura glyphs are used in renderer and UI (`font-bravura` class + `src-tauri/src/renderer/fonts.rs`).
- Icon strategy: `IconButton` for MDI/SVG; `BravuraIconButton` for SMuFL/Bravura glyphs (new component).

## Examples (copy/paste patterns)

- Add a Tauri command and expose it to the UI:
  - Rust: add `#[tauri::command] fn score_entries_my_action(...) { ... }` in `src-tauri/src/score/entries/*.rs`.
  - Frontend: add mapping to `src/services/actions.ts` that calls `invoke("score_entries_my_action", {...})`.
  - Types: update `src/services/types.ts` if arguments or return types changed.

- Renderer change (draw new glyph): update parser `src-tauri/src/instructions/parse/*` and drawer `src-tauri/src/instructions/draw/*` (keep parse → draw contract intact).

## PR checklist for AI agents

- Update both Rust and TS types when touching shared models.
- Run `npm run check-types` and `cd src-tauri && cargo check && cargo test` locally; fix all errors and warnings and include the outputs in the PR description. Do **not** mark the work as done while type-check or cargo check produce errors or new warnings.

---

If any area above is unclear or you want more examples (Tauri command templates, renderer hooks, or where to add tests), tell me which section to expand and I'll iterate.
