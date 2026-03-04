import { describe, it, expect } from "vitest";
import { getTheme } from "./shared";
import type { Variant } from "./shared";

describe("getTheme", () => {
  it("returns selected theme for any variant when isSelected=true", () => {
    const variants: Variant[] = [
      "standard",
      "interactive",
      "filled",
      "filled-dark",
      "outline",
    ];
    for (const variant of variants) {
      expect(getTheme(variant, true)).toBe("bg-sky-500 text-black");
    }
  });

  it("returns empty string for standard variant when not selected", () => {
    expect(getTheme("standard", false)).toBe("");
    expect(getTheme("standard")).toBe("");
  });

  it("returns filled theme for filled variant", () => {
    const theme = getTheme("filled", false);
    expect(theme).toContain("bg-neutral-800");
    expect(theme).toContain("text-white");
  });

  it("returns filled-dark theme for filled-dark variant", () => {
    const theme = getTheme("filled-dark", false);
    expect(theme).toContain("bg-neutral-950");
    expect(theme).toContain("text-white");
  });

  it("returns interactive theme for interactive variant", () => {
    const theme = getTheme("interactive", false);
    expect(theme).toContain("bg-neutral-900");
    expect(theme).toContain("text-white");
  });

  it("returns outline theme for outline variant", () => {
    const theme = getTheme("outline", false);
    expect(theme).toContain("border");
    expect(theme).toContain("text-white");
  });
});
