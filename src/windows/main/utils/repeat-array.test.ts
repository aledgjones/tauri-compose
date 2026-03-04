import { describe, it, expect } from "vitest";
import { repeatArray } from "./repeat-array";

describe("repeatArray", () => {
  it("repeats array elements count times", () => {
    expect(repeatArray([1, 2], 3)).toEqual([1, 2, 1, 2, 1, 2]);
  });

  it("returns empty array for empty input", () => {
    expect(repeatArray([], 5)).toEqual([]);
  });

  it("returns empty array when count is 0", () => {
    expect(repeatArray([1], 0)).toEqual([]);
  });

  it("returns single copy when count is 1", () => {
    expect(repeatArray(["a", "b"], 1)).toEqual(["a", "b"]);
  });

  it("works with string elements", () => {
    expect(repeatArray(["x"], 4)).toEqual(["x", "x", "x", "x"]);
  });
});
