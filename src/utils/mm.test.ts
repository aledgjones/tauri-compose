import { describe, it, expect } from "vitest";
import { pxPerMm } from "./mm";

describe("pxPerMm", () => {
  it("is a positive number", () => {
    expect(typeof pxPerMm).toBe("number");
    expect(pxPerMm).toBeGreaterThan(0);
  });

  it("falls back to 3.78 in jsdom where clientWidth is 0", () => {
    // jsdom returns 0 for clientWidth, so getMM() falls back to 3.78
    expect(pxPerMm).toBe(3.78);
  });
});
