import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { useSlotPattern, SLOT_PATTERN, KeyType } from "./use-slots";

describe("SLOT_PATTERN", () => {
  it("returns 12 slots (one octave) at zoom 1", () => {
    expect(SLOT_PATTERN(1)).toHaveLength(12);
  });

  it("contains 7 white and 5 black keys per octave", () => {
    const pattern = SLOT_PATTERN(1);
    const whites = pattern.filter((s) => s.type === KeyType.White);
    const blacks = pattern.filter((s) => s.type === KeyType.Black);
    expect(whites).toHaveLength(7);
    expect(blacks).toHaveLength(5);
  });

  it("doubles heights at zoom 2", () => {
    const p1 = SLOT_PATTERN(1);
    const p2 = SLOT_PATTERN(2);
    expect(p2[0].height).toBe(p1[0].height * 2);
  });
});

describe("useSlotPattern", () => {
  it("returns 12 slots for 1 octave", () => {
    const { result } = renderHook(() => useSlotPattern(1));
    expect(result.current.pattern).toHaveLength(12);
  });

  it("returns 24 slots for 2 octaves", () => {
    const { result } = renderHook(() => useSlotPattern(2));
    expect(result.current.pattern).toHaveLength(24);
  });

  it("doubles slotHeight at zoomY=2", () => {
    const { result: r1 } = renderHook(() => useSlotPattern(1, 1));
    const { result: r2 } = renderHook(() => useSlotPattern(1, 2));
    expect(r2.current.slotHeight).toBe(r1.current.slotHeight * 2);
  });

  it("height equals slotHeight * pattern.length", () => {
    const { result } = renderHook(() => useSlotPattern(2, 1));
    expect(result.current.height).toBe(
      result.current.slotHeight * result.current.pattern.length,
    );
  });

  it("pattern includes both KeyType.White and KeyType.Black", () => {
    const { result } = renderHook(() => useSlotPattern(1));
    const hasWhite = result.current.pattern.some(
      (s) => s.type === KeyType.White,
    );
    const hasBlack = result.current.pattern.some(
      (s) => s.type === KeyType.Black,
    );
    expect(hasWhite).toBe(true);
    expect(hasBlack).toBe(true);
  });
});
