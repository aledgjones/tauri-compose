import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useHumanizedTime, useAsync } from "./utils";

const ONE_WEEK = 7 * 24 * 60 * 60 * 1000;

// ---------------------------------------------------------------------------
// useHumanizedTime
// ---------------------------------------------------------------------------
describe("useHumanizedTime", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns 'Just now' for a timestamp within 30 seconds", () => {
    const now = 1_700_000_000_000;
    vi.setSystemTime(now);
    const { result } = renderHook(() => useHumanizedTime(now - 5_000));
    expect(result.current).toBe("Just now");
  });

  it("returns a locale date string for a timestamp more than 1 week ago", () => {
    const now = 1_700_000_000_000;
    vi.setSystemTime(now);
    const when = now - ONE_WEEK - 1_000;
    const { result } = renderHook(() => useHumanizedTime(when));
    expect(result.current).toBe(new Date(when).toLocaleDateString());
  });

  it("returns a relative 'X ago' string for timestamps between 30s and 1 week", () => {
    const now = 1_700_000_000_000;
    vi.setSystemTime(now);
    const fiveMinutesAgo = now - 5 * 60 * 1_000;
    const { result } = renderHook(() => useHumanizedTime(fiveMinutesAgo));
    expect(result.current).toMatch(/ago$/);
  });

  it("updates when the interval fires", () => {
    const now = 1_700_000_000_000;
    vi.setSystemTime(now);
    // Start at 25s ago → "Just now"
    const when = now - 25_000;
    const { result } = renderHook(() => useHumanizedTime(when));
    expect(result.current).toBe("Just now");

    // Advance time so that `Date.now() - when` exceeds 30 seconds
    act(() => {
      vi.advanceTimersByTime(6_000);
      vi.setSystemTime(now + 6_000);
    });

    // Now 31s ago — should no longer be "Just now"
    expect(result.current).not.toBe("Just now");
    expect(result.current).toMatch(/ago$/);
  });
});

// ---------------------------------------------------------------------------
// useAsync
// ---------------------------------------------------------------------------
describe("useAsync", () => {
  it("calls the async function on mount", async () => {
    const asyncFn = vi.fn().mockResolvedValue("result");
    renderHook(() => useAsync(asyncFn));
    // Flush effects
    await act(async () => {});
    expect(asyncFn).toHaveBeenCalledOnce();
  });

  it("catches rejected promises and logs the error without throwing", async () => {
    const error = new Error("boom");
    const consoleSpy = vi
      .spyOn(console, "log")
      .mockImplementation(() => undefined);
    const asyncFn = vi.fn().mockRejectedValue(error);

    renderHook(() => useAsync(asyncFn));
    await act(async () => {});

    expect(consoleSpy).toHaveBeenCalledWith(error);
    consoleSpy.mockRestore();
  });
});
