# Technical Research Brief: Testing & Tooling Modernization for Compose

**Prepared by:** Mary (Business Analyst) | **Date:** 4 March 2026

---

## Goal

Add a testing foundation and modernize tooling so AI-assisted development can't break existing functionality.

## 1. Current Stack Snapshot

| Layer    | Technology                             | Files                             |
| -------- | -------------------------------------- | --------------------------------- |
| Frontend | React 19.2 + TypeScript 5.9            | ~100 `.ts`/`.tsx` files           |
| Backend  | Rust (Tauri 2.x)                       | ~99 `.rs` files                   |
| Build    | Vite 7.3                               | Multi-entry (8 HTML entry points) |
| Styling  | Tailwind CSS 4.2 + PostCSS             |                                   |
| Linting  | ESLint 9 (flat config) + Prettier 3.8  | Already modernized format         |
| State    | React Context (`src/services/state/`)  |                                   |
| IPC      | Tauri `invoke()` calls to Rust backend |                                   |

**Key finding: Zero tests exist today** — no test runner, no test files, no test dependencies. The Rust side also has no `#[cfg(test)]` blocks.

---

## 2. Code Testability Analysis

The codebase has a natural testability gradient:

| Category                   | Examples                                                                       | Testability                              | Priority            |
| -------------------------- | ------------------------------------------------------------------------------ | ---------------------------------------- | ------------------- |
| **Pure utility functions** | `pitch-utils.ts` (pitch math), `utils.ts` (`convertTimestampToHuman`), `mm.ts` | ⭐⭐⭐ Trivially testable                | **P1 — Start here** |
| **Type definitions**       | `types.ts` (450 lines of enums, interfaces)                                    | N/A — validated by TS compiler           | —                   |
| **DOM query helpers**      | `sequencer-utils.ts`, `hit-test.ts`, `popovers.ts`                             | ⭐⭐ Testable with jsdom/happy-dom       | P2                  |
| **React UI components**    | `ui/` (checkbox, select, popover, tabs, etc.)                                  | ⭐⭐ Testable with React Testing Library | P2                  |
| **React hooks**            | `useHumanizedTime`, `useAsync`, custom hooks in `ui/hooks/`                    | ⭐⭐ Testable with `renderHook`          | P2                  |
| **Tauri IPC actions**      | `actions.ts` (888 lines of `invoke()` calls)                                   | ⭐ Requires mocking `@tauri-apps/api`    | P3                  |
| **Full window screens**    | `windows/main/`, `windows/preferences/`, etc.                                  | ⭐ Integration-level, complex setup      | P3                  |

---

## 3. Recommended Testing Stack

**Vitest** is the unambiguous choice for this stack:

| Tool                                                 | Why                                                                                                                                                  |
| ---------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Vitest**                                           | Native Vite integration (zero config needed), same transform pipeline, blazing fast HMR-aware test runs, built-in coverage via `@vitest/coverage-v8` |
| **happy-dom** (environment)                          | Lighter and faster than jsdom, sufficient for this app's DOM needs                                                                                   |
| **React Testing Library** (`@testing-library/react`) | Standard for testing React components by user behavior, not implementation                                                                           |
| **`@testing-library/user-event`**                    | Realistic user interaction simulation                                                                                                                |
| **`@vitest/coverage-v8`**                            | V8-based coverage — fast, accurate, no instrumentation step                                                                                          |

**Why not Jest?** Jest requires separate Babel/SWC transforms, doesn't share Vite's config/aliases/plugins, and adds significant configuration overhead. Vitest reuses your existing `vite.config.ts` out of the box.

---

## 4. Recommended Linting/Tooling Modernizations

The current setup is already in decent shape — ESLint 9 flat config is modern. But there are clear improvements:

| Area                     | Current                                          | Recommendation                                                                                                    |
| ------------------------ | ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| **ESLint config**        | Manual flat config with old-style plugin imports | Migrate to `typescript-eslint` v8 unified config (`tseslint.config(...)`) — cleaner, type-checked rules available |
| **Prettier integration** | Standalone script only                           | Add `eslint-config-prettier` to avoid rule conflicts                                                              |
| **Vite ESLint plugin**   | `@nabla/vite-plugin-eslint` (overlay in dev)     | Keep as-is — useful for dev feedback                                                                              |
| **Pre-commit hooks**     | None                                             | Add **lint-staged** + **simple-git-hooks** (lighter than husky) for automatic lint/format on commit               |
| **CI-ready scripts**     | Partial (`lint`, `format`, `check-types`)        | Add `test`, `test:coverage`, `test:ci` scripts; unify a `ci` script that runs all checks                          |

---

## 5. Recommended Implementation Approach

### Phase 1: Foundation (Barry's immediate work)

1. Install Vitest + happy-dom + React Testing Library + coverage
2. Add `vitest.config.ts` (or extend `vite.config.ts`)
3. Add `test`, `test:watch`, `test:coverage` scripts
4. Write first tests against `pitch-utils.ts` — pure functions, immediate wins, establishes patterns
5. Add tests for `utils.ts` pure functions (`convertTimestampToHuman`, utility helpers)
6. Add a Tauri mock helper for `@tauri-apps/api` invoke calls

### Phase 2: Component & Hook Coverage

7. Test UI primitives (`checkbox`, `select`, `switch`, `tabs`)
8. Test custom hooks
9. Test composite components (`snap-picker`, `app-bar`)

### Phase 3: Guardrails for AI Development

10. Set up coverage thresholds (start low, ratchet up)
11. Add lint-staged + simple-git-hooks for pre-commit checks
12. Create a unified `ci` script: `check-types && lint && test:coverage`
13. Document test patterns in a `TESTING.md` for AI agents to follow

---

## 6. Key Config Notes

- **Vite multi-entry**: The app has 8 HTML entry points — test config should just focus on `src/**` and ignore entry HTML files
- **Tauri mocking**: `actions.ts` wraps all IPC in a clean `actions` object — this is easily mockable. Create a `__mocks__/@tauri-apps/api.ts` manual mock
- **State context**: `initialAppState` in `context.tsx` is a large default state object — excellent as test fixture data
- **No routing**: Each window is a separate HTML entry, not a router — simplifies component testing (no router provider needed)
- **Strict TS**: Already enabled — tests should also be strictly typed
