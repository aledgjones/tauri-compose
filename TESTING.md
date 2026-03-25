# Testing Guide

## Overview

This project uses [Vitest](https://vitest.dev) as the test runner for TypeScript/TSX, and Rust's built-in `cargo test` for the backend. All tests are expected to pass in CI before merging.

## Commands

| Command                      | Description                            |
| ---------------------------- | -------------------------------------- |
| `npm run test:js`            | Run all TypeScript tests once          |
| `npm run test:js:watch`      | Run tests in watch mode (TDD)          |
| `npm run test:js:coverage`   | Run JS tests with v8 coverage report   |
| `npm run test:js:logs`       | Run JS tests once with verbose logs    |
| `npm run test:js:logs:watch` | Watch mode with verbose logs           |
| `npm run test:rust`          | Run all Rust tests (from `src-tauri/`) |

## Single-instrument reset baseline

Story 1.3 is the explicit release-style gate before Epic 4 audible-playback work can start, and Story 1.4 extends that gate with slot-owned live runtime sessions. Here, single-instrument means one playable instrument per VST instance, even if that instance exposes multiple articulations or expressions; HALion-style multi-slot or multi-part instances remain out of scope. Treat it as a narrow trust-restoration proof, not as evidence of multi-instrument playback or audible transport support.

Automated evidence required for the current baseline:

- `cd src-tauri && cargo test runtime_activation -- --nocapture`
- `cd src-tauri && cargo test plugin_editor_poc -- --nocapture`
- `cd src-tauri && cargo test files::tests:: -- --nocapture`
- `npm run check-types`
- `npm run lint`

Manual proof required before Epic 4 stories are unblocked:

1. Prepare one routed score instrument against a supported single-instrument VST instance on the VST-only rack path.
2. Open the app and confirm the route resolves as transport-ready through the existing score-context or maintainer-facing diagnostics surfaces before any native editor window is opened.
3. Open the native editor from the routed rack slot, verify the host window and embedded editor stay in sync during resize behavior, close it, and reopen it in the same session without hangs while the slot remains transport-ready across the close/reopen cycle.
4. If two rack slots point at the same bundle, confirm the slot-scoped open action targets the intended slot identity rather than reusing the other slot's editor target.
5. Unassign or remove the routed slot and confirm the readiness state drops immediately instead of persisting until editor close.
6. Save the score, reopen the `.soloc` file, and confirm portable route intent persists while runtime readiness is re-resolved from current machine-local inventory rather than from serialized live host state.

Blocking rule:

- Epic 4 audible-playback implementation stays blocked until both the automated checks above pass and the manual proof flow has been completed with a go verdict.

## Native editor host validation subset

Use Story 2.7's bounded macOS editor-host claim, not a universal compatibility assumption.

- Known-good bounded-host examples: `BBC Symphony Orchestra.vst3` remains the primary control case, and recent successful maintainer runs also include `Reverb.vst3`, `ProEQ.vst3`, and `Originals - Intimate Grand Piano.vst3`
- Explicit unsupported targets in the current proof host: the current unsupported list includes `HALion Sonic.vst3`, `HALion Sonic SE.vst3`, `Groove Agent SE.vst3`, `Kontakt 7.vst3`, and `MT-PowerDrumKit.vst3`
- Expected unsupported behavior: inventory still reports a successfully discovered bundle as available, but the maintainer surfaces render it as `Unsupported`, hide the `Open native editor` action, and backend fallback protection still rejects direct requests without creating or reserving a plugin-host window for bundles on that unsupported list

Recommended validation commands for host-seam changes:

- `cd src-tauri && cargo test --lib plugin_editor_poc`
- `cd src-tauri && cargo clippy --lib -- -D warnings`

Recommended manual validation sequence for the bounded host seam:

1. Open `BBC Symphony Orchestra.vst3` through the Plugin Manager maintainer action and confirm the host window opens, attaches the editor, stays responsive, and can be closed and reopened in the same app session.
2. If the control-case plugin reports resize support, verify resize remains coherent; otherwise confirm the host stays bounded without pretending unsupported resize capability.
3. Confirm a bundle on the unsupported list such as a Steinberg-family plugin renders as `Unsupported` in the maintainer plugin surfaces, does not show the `Open native editor` action, and still cannot create a host window or stale reservation through a direct backend request.

## Multi-flow performance characterization

Use the existing Rust-owned debugger diagnostics as the primary evidence path for multi-flow performance work. Do not add ad hoc console logging or frontend-owned benchmark state when validating render scope.

Run `npm run perf:render` for the manual characterization pass. This command runs the dedicated standalone release binary at `src-tauri/src/bin/performance.rs`, exercises the same backend render-plan and shared-surface composition seams as the app, and avoids changing the shipped API surface. It does not replace the debugger-backed event path for end-to-end observability validation. By default it records 5 measured samples after 1 warm-up sample per scenario. Use `npm run perf:render -- --samples 10` to change the measured sample count and `npm run perf:render -- --json` for machine-readable output.

For durable branch or historical comparisons, prefer `npm run perf:render -- --samples 15 --json` and record the resulting medians in [docs/performance-baselines.md](./docs/performance-baselines.md). Keep the historical run log in that report append-only so benchmark snapshots remain comparable over time while the top-level trend summary stays current.

Representative validation scenarios:

1. Single-flow baseline: one visible flow representing the existing product standard of 3 instruments and 30 bars. Treat the current release-like `npm run perf:render` result as the active baseline, trigger a regression warning when the representative total median reaches or exceeds 2ms, and reevaluate that warning threshold as additional performance-sensitive features land.
2. Multi-flow render: two or three visible flows on the shared surface. Verify the debugger shows multi-flow prepared/composed scope and record the corresponding phase timings.
3. Multi-flow local edit: mutate one visible flow while reuse is available. Verify `render-timing-flow-preparation-details` reports a multi-flow prepared/composed set but a single-flow `recalculated_flow_ids` set. If reuse is unavailable, record the truthful broad fallback instead of masking it.

Debugger fields to review:

- `render-timing-flow-preparation`
- `render-timing-flow-preparation-details`
- `render-timing-surface-details`
- The existing Layout, Render, and GPU timing rows in the debugger window

Evidence reporting rules:

- Record whether each timing came from the supported dev validation environment or a release-like run.
- Call out broad fallback, no-op passes, or scaling cliffs explicitly.
- Keep the current release-like baseline sample in the story or adjacent maintainer-facing docs, and refresh it when material rendering features land.
- When a result is important enough to compare later, add a dated entry to [docs/performance-baselines.md](./docs/performance-baselines.md) with the commit, sample count, and scenario medians.
- Keep automated tests focused on scope, ordering, and reuse behavior. Do not turn the CI suite into a microbenchmark gate.

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

Components that call `useAppState()`, `useAudioState()`, `usePluginInventoryState()`, or `useTransportState()` need the shared app providers. Use `renderWithProviders` from `src/services/test-utils.tsx`:

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

Playback ownership regressions should stay seam-local. When validating playback readiness or recovery, derive results through `PlaybackActivationRuntime::from_score_and_inventory(...)` and adjacent file/load seams rather than adding score-local duplicate readiness logic in tests.

For the Story 1.3 reset baseline specifically, use `PlaybackActivationRuntime::single_instrument_trust_gate(...)` as the backend-owned trust seam. It is transport-readiness-only and should not be used to imply audible playback, multi-instrument support, or broader plugin-host compatibility than the currently supported single-instrument subset.

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
npm run test:js:coverage

# Print totals from last generated report
npm run test:js:coverage:summary
```

Thresholds are enforced in the `test.coverage.thresholds` block in [vite.config.ts](vite.config.ts):

| Metric     | Threshold |
| ---------- | --------- |
| Statements | 80%       |
| Branches   | 80%       |
| Functions  | 80%       |
| Lines      | 80%       |

As the test suite grows, raise these values — never lower them (ratchet rule).

## Rust Coverage

Coverage for the Tauri/Rust layer uses [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov).

```bash
# Install once
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

# Show summary table
npm run test:rust:coverage:summary

# Enforce thresholds (exits non-zero if below)
npm run test:rust:coverage
```

Thresholds are enforced via flags in the `test:rust:coverage` script in [package.json](package.json):

| Metric    | Threshold |
| --------- | --------- |
| Lines     | 62%       |
| Functions | 76%       |
| Regions   | 65%       |

As the Rust test suite grows, raise these values — never lower them (ratchet rule).

## Pre-commit Hook

`lint-staged` runs automatically on staged files before each commit via `simple-git-hooks`:

- **`*.{ts,tsx}`** — `eslint --fix` then `prettier --write`
- **`*.{json,md,css,html}`** — `prettier --write`

If the hook is not running (e.g. after a fresh clone), register it manually:

```bash
npx simple-git-hooks
```
