import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { Keyboard } from "./keyboard";
import { useSlotPattern, KeyType } from "../hooks/use-slots";
import { renderHook } from "@testing-library/react";

describe("Keyboard", () => {
  it("renders without crashing", () => {
    const { result } = renderHook(() => useSlotPattern(1));
    const { container } = render(<Keyboard slots={result.current} />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it("renders white keys (div elements for white keys)", () => {
    const { result } = renderHook(() => useSlotPattern(1));
    const slots = result.current;
    const whiteCount = slots.pattern.filter(
      (s) => s.type === KeyType.White,
    ).length;
    // Keyboard renders one div per white key
    const { container } = render(<Keyboard slots={slots} />);
    const allDivs = container.querySelectorAll("div");
    // The outer container + inner wrapper + white keys + black keys
    expect(allDivs.length).toBeGreaterThan(whiteCount);
  });

  it("renders black keys with dark background", () => {
    const { result } = renderHook(() => useSlotPattern(1));
    const { container } = render(<Keyboard slots={result.current} />);
    const blackKeys = container.querySelectorAll("div.bg-neutral-950");
    const expectedBlackCount = result.current.pattern.filter(
      (s) => s.type === KeyType.Black,
    ).length;
    expect(blackKeys).toHaveLength(expectedBlackCount);
  });

  it("sets container height from slots.height", () => {
    const { result } = renderHook(() => useSlotPattern(1));
    const slots = result.current;
    const { container } = render(<Keyboard slots={slots} />);
    const outer = container.querySelector("[style]") as HTMLElement;
    expect(outer.style.height).toBe(`${slots.height}px`);
  });
});
