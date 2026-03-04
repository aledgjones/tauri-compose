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

All calls to `@tauri-apps/api/core`'s `invoke()` are intercepted by the manual mock at `src/__mocks__/@tauri-apps/api/core.ts`. Vitest resolves this automatically via the `__mocks__` convention.

To assert on IPC calls or control return values:

```ts
import { invoke } from "@tauri-apps/api/core";

// invoke is already a vi.fn() — shape it per test
vi.mocked(invoke).mockResolvedValueOnce({ bpm: 120 });
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

Coverage is collected with `@vitest/coverage-v8`. Run `npm run test:coverage` to generate a report in `coverage/`.

Thresholds are enforced at **5%** across statements, branches, functions, and lines. As the test suite grows, raise these in the `test.coverage.thresholds` block in [vite.config.ts](vite.config.ts).

## Pre-commit Hook

`lint-staged` runs automatically on staged files before each commit via `simple-git-hooks`:

- **`*.{ts,tsx}`** — `eslint --fix` then `prettier --write`
- **`*.{json,md,css,html}`** — `prettier --write`

If the hook is not running (e.g. after a fresh clone), register it manually:

```bash
npx simple-git-hooks
```
