# Testing Guide

## Overview

This project uses [Vitest](https://vitest.dev) as the test runner for TypeScript/TSX, and Rust's built-in `cargo test` for the backend. All tests are expected to pass in CI before merging.

## Commands

| Command                 | Description                            |
| ----------------------- | -------------------------------------- |
| `npm test`              | Run all TypeScript tests once          |
| `npm run test:watch`    | Run tests in watch mode (TDD)          |
| `npm run test:coverage` | Run tests with v8 coverage report      |
| `cargo test`            | Run all Rust tests (from `src-tauri/`) |

## File Naming

TypeScript test files sit next to the source file they test, using the `.test.ts` or `.test.tsx` extension:

```
src/
  services/
    utils.ts
    utils.test.ts
  ui/
    checkbox.tsx
    checkbox.test.tsx
```

Rust tests live in `#[cfg(test)] mod tests {}` blocks at the bottom of the source file they test.

## TypeScript Unit Tests (Pure Functions)

Use plain `describe` / `it` / `expect` — globals are enabled, no imports needed.

```ts
// src/services/utils.test.ts
import { parseTimeSignature } from "./utils";

describe("parseTimeSignature", () => {
  it("parses '4/4' correctly", () => {
    expect(parseTimeSignature("4/4")).toEqual({ numerator: 4, denominator: 4 });
  });
});
```

## Component Tests

Use `@testing-library/react` with `render`, `screen`, and `userEvent`. The DOM environment is `happy-dom`.

```tsx
// src/ui/switch.test.tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Switch } from "./switch";

describe("Switch", () => {
  it("calls onChange when clicked", async () => {
    const onChange = vi.fn();
    render(<Switch checked={false} onChange={onChange} />);
    await userEvent.click(screen.getByRole("switch"));
    expect(onChange).toHaveBeenCalledWith(true);
  });
});
```

## Hook Tests

Use `renderHook` and `act` from `@testing-library/react`. Use `vi.useFakeTimers()` for hooks that depend on time.

```tsx
// src/services/hooks.test.tsx
import { renderHook, act } from "@testing-library/react";
import { useHumanizedTime } from "./utils";

it("returns 'Just now' for a recent timestamp", () => {
  vi.useFakeTimers();
  const { result } = renderHook(() => useHumanizedTime(Date.now()));
  expect(result.current).toBe("Just now");
  vi.useRealTimers();
});
```

## Tauri IPC Mock

All calls to `@tauri-apps/api/core`'s `invoke()` are intercepted by the manual mock at `src/__mocks__/@tauri-apps/api/core.ts`. **The auto-mock is NOT loaded automatically** — every test file that uses Tauri APIs must include an explicit factory call:

```ts
// Required in every file that calls invoke()
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

import { invoke } from "@tauri-apps/api/core";

// Shape per test
vi.mocked(invoke).mockResolvedValueOnce({ bpm: 120 });
```

## Context-Aware Component Tests (renderWithProviders)

Components that call `useAppState()`, `useAudioState()`, `useSamplerState()`, or `useTransportState()` need all four React contexts. Use `renderWithProviders` from `src/services/test-utils.tsx`:

```tsx
import { renderWithProviders, createAppState } from '../../../services/test-utils';

it('renders selected tone info', () => {
  const appState = createAppState({
    ui: { sequencer: { selection: { type: 'Tone', ... } } } as never,
  });
  renderWithProviders(<MyComponent />, { appState });
  expect(screen.getByText(/Piano/)).toBeInTheDocument();
});

it('renders nothing when no selection', () => {
  renderWithProviders(<MyComponent />); // uses default state
  expect(screen.getByText('No selection')).toBeInTheDocument();
});
```

`createAppState(overrides?)` deep-merges overrides into `initialAppState`. Arrays are **replaced**, not merged.

## Context-Aware Hook Tests (createWrapper)

For hooks that call context selectors, pass `createWrapper()` as the `wrapper` option:

```tsx
import { renderHook } from "@testing-library/react";
import { createWrapper, createAppState } from "../../../services/test-utils";

it("returns null when no selection", () => {
  const { result } = renderHook(() => useSelectedToneInfo(), {
    wrapper: createWrapper(),
  });
  expect(result.current).toBeNull();
});
```

## Mocking the Actions Module

Components that import from `services/actions` should mock the entire module with vi.fn() stubs. Always use the factory overload:

```tsx
vi.mock("../../../services/actions", () => ({
  actions: {
    transport: {
      play: vi.fn(),
      pause: vi.fn(),
      stop: vi.fn(),
    },
  },
}));

import { actions } from "../../../services/actions";

it("calls play on click", async () => {
  renderWithProviders(<Transport />);
  await userEvent.click(screen.getByRole("button", { name: /play/i }));
  expect(vi.mocked(actions.transport.play)).toHaveBeenCalledOnce();
});
```

## Rust Tests

Tests are co-located in `#[cfg(test)] mod tests {}` blocks at the bottom of each source file. Run them with `cargo test` from the `src-tauri/` directory.

```rust
// src-tauri/src/renderer/converter.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaces_to_px_applies_formula() {
        let c = Converter::new(16.0, 1.0);
        assert_eq!(c.spaces_to_px(2.0), 32.0);
    }
}
```

Use the `matches!` macro for enums that don't implement `Debug`:

```rust
assert!(matches!(NoteDuration::from_ticks(128), Some(NoteDuration::Quaver)));
```

## Coverage

Coverage for the TypeScript/React layer uses [`@vitest/coverage-v8`](https://vitest.dev/guide/coverage).

```bash
# Generate report in coverage/
npm run test:coverage

# Print totals from last generated report
npm run test:coverage:summary
```

Thresholds are enforced in the `test.coverage.thresholds` block in [vite.config.ts](vite.config.ts):

| Metric     | Threshold |
| ---------- | --------- |
| Statements | 70%       |
| Branches   | 60%       |
| Functions  | 75%       |
| Lines      | 70%       |

As the test suite grows, raise these values — never lower them (ratchet rule).

## Rust Coverage

Coverage for the Tauri/Rust layer uses [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov).

```bash
# Install once
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

# Show summary table
npm run test:coverage:rust:summary

# Enforce thresholds (exits non-zero if below)
npm run test:coverage:rust
```

Thresholds are enforced via flags in the `test:coverage:rust` script in [package.json](package.json):

| Metric    | Threshold |
| --------- | --------- |
| Lines     | 10%       |
| Functions | 30%       |
| Regions   | 15%       |

As the Rust test suite grows, raise these values — never lower them (ratchet rule).

## Pre-commit Hook

`lint-staged` runs automatically on staged files before each commit via `simple-git-hooks`:

- **`*.{ts,tsx}`** — `eslint --fix` then `prettier --write`
- **`*.{json,md,css,html}`** — `prettier --write`

If the hook is not running (e.g. after a fresh clone), register it manually:

```bash
npx simple-git-hooks
```
