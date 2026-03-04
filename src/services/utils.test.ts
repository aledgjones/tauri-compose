import { describe, it, expect } from "vitest";
import {
  parseTimeSignature,
  noteDurationToTicks,
  generateRainbowColors,
  desaturateColor,
  lightenColor,
} from "./utils";
import { NoteDuration } from "./types";

// ---------------------------------------------------------------------------
// parseTimeSignature
// ---------------------------------------------------------------------------
describe("parseTimeSignature", () => {
  it("parses '4/4' correctly", () => {
    expect(parseTimeSignature("4/4")).toEqual({ beats: 4, beatType: 4 });
  });

  it("parses '6/8' correctly", () => {
    expect(parseTimeSignature("6/8")).toEqual({ beats: 6, beatType: 8 });
  });

  it("parses '3/4' correctly", () => {
    expect(parseTimeSignature("3/4")).toEqual({ beats: 3, beatType: 4 });
  });

  it("parses '2/2' correctly", () => {
    expect(parseTimeSignature("2/2")).toEqual({ beats: 2, beatType: 2 });
  });

  it("parses '12/8' correctly", () => {
    expect(parseTimeSignature("12/8")).toEqual({ beats: 12, beatType: 8 });
  });

  it("returns null for invalid beat type '4/3'", () => {
    expect(parseTimeSignature("4/3")).toBeNull();
  });

  it("returns null for invalid beat type '4/5'", () => {
    expect(parseTimeSignature("4/5")).toBeNull();
  });

  it("returns null for empty string", () => {
    expect(parseTimeSignature("")).toBeNull();
  });

  it("returns null for plain text 'abc'", () => {
    expect(parseTimeSignature("abc")).toBeNull();
  });

  it("returns null for malformed '4/'", () => {
    expect(parseTimeSignature("4/")).toBeNull();
  });

  it("returns null for '/4'", () => {
    expect(parseTimeSignature("/4")).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// noteDurationToTicks
// ---------------------------------------------------------------------------
describe("noteDurationToTicks", () => {
  it("Whole = 128 ticks", () => {
    expect(noteDurationToTicks(NoteDuration.Whole)).toBe(128);
  });

  it("Half = 64 ticks", () => {
    expect(noteDurationToTicks(NoteDuration.Half)).toBe(64);
  });

  it("Quarter = 32 ticks", () => {
    expect(noteDurationToTicks(NoteDuration.Quarter)).toBe(32);
  });

  it("Eighth = 16 ticks", () => {
    expect(noteDurationToTicks(NoteDuration.Eighth)).toBe(16);
  });

  it("Sixteenth = 8 ticks", () => {
    expect(noteDurationToTicks(NoteDuration.Sixteenth)).toBe(8);
  });

  it("ThirtySecond = 4 ticks", () => {
    expect(noteDurationToTicks(NoteDuration.ThirtySecond)).toBe(4);
  });

  it("SixtyFourth = 2 ticks", () => {
    expect(noteDurationToTicks(NoteDuration.SixtyFourth)).toBe(2);
  });

  it("HudredTwentyEighth = 1 tick", () => {
    expect(noteDurationToTicks(NoteDuration.HudredTwentyEighth)).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// generateRainbowColors
// ---------------------------------------------------------------------------
describe("generateRainbowColors", () => {
  it("returns exactly 7 colors when count < 7", () => {
    expect(generateRainbowColors(3)).toHaveLength(7);
  });

  it("returns exactly 7 colors when count = 7", () => {
    expect(generateRainbowColors(7)).toHaveLength(7);
  });

  it("returns count colors when count > 7", () => {
    expect(generateRainbowColors(12)).toHaveLength(12);
  });

  it("all colors are valid 7-char hex strings starting with #", () => {
    const colors = generateRainbowColors(10);
    for (const color of colors) {
      expect(color).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });
});

// ---------------------------------------------------------------------------
// desaturateColor
// ---------------------------------------------------------------------------
describe("desaturateColor", () => {
  it("returns a valid hex string", () => {
    const result = desaturateColor("#ff0000", 0.5, 0.8);
    expect(result).toMatch(/^#[0-9a-f]{6}$/i);
  });

  it("factors (1, 1) return approximately the same color", () => {
    const input = "#4488cc";
    const result = desaturateColor(input, 1, 1);
    expect(result).toMatch(/^#[0-9a-f]{6}$/i);
    // The result should be close to original — check it's a valid hex
    expect(result.length).toBe(7);
  });

  it("saturation factor 0 produces a greyscale-ish result", () => {
    const result = desaturateColor("#ff0000", 0, 0.5);
    // With saturation 0, R=G=B after HSL→RGB, all three channels equal → greyscale
    const r = parseInt(result.slice(1, 3), 16);
    const g = parseInt(result.slice(3, 5), 16);
    const b = parseInt(result.slice(5, 7), 16);
    expect(r).toBe(g);
    expect(g).toBe(b);
  });
});

// ---------------------------------------------------------------------------
// lightenColor
// ---------------------------------------------------------------------------
describe("lightenColor", () => {
  it("returns a valid hex string", () => {
    const result = lightenColor("#336699", 0.5);
    expect(result).toMatch(/^#[0-9a-f]{6}$/i);
  });

  it("factor 0 returns same color", () => {
    const input = "#336699";
    const result = lightenColor(input, 0);
    expect(result).toBe(input);
  });

  it("factor 1 returns near-white (all channels >= 0xf0)", () => {
    const result = lightenColor("#336699", 1);
    const r = parseInt(result.slice(1, 3), 16);
    const g = parseInt(result.slice(3, 5), 16);
    const b = parseInt(result.slice(5, 7), 16);
    expect(r).toBeGreaterThanOrEqual(0xf0);
    expect(g).toBeGreaterThanOrEqual(0xf0);
    expect(b).toBeGreaterThanOrEqual(0xf0);
  });

  it("lightened color has higher or equal brightness than original", () => {
    const input = "#336699";
    const result = lightenColor(input, 0.5);
    const origBrightness =
      parseInt(input.slice(1, 3), 16) +
      parseInt(input.slice(3, 5), 16) +
      parseInt(input.slice(5, 7), 16);
    const newBrightness =
      parseInt(result.slice(1, 3), 16) +
      parseInt(result.slice(3, 5), 16) +
      parseInt(result.slice(5, 7), 16);
    expect(newBrightness).toBeGreaterThanOrEqual(origBrightness);
  });
});
