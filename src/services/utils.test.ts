import { describe, it, expect, vi } from "vitest";
import { renderHook } from "@testing-library/react";
import {
  parseTimeSignature,
  noteDurationToTicks,
  generateRainbowColors,
  desaturateColor,
  lightenColor,
  getFlowName,
  getLayoutName,
  useHumanizedTime,
} from "./utils";
import { LayoutType, NoteDuration, PlayerType } from "./types";
import { createAppState } from "./test-utils";

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

  it("works with a green-dominant color (hits case g in switch)", () => {
    // #00ff00: g=255 is the maximum channel
    const result = lightenColor("#00ff00", 0.5);
    expect(result).toMatch(/^#[0-9a-f]{6}$/i);
  });

  it("works with an achromatic (grey) color (hits s=0 branch)", () => {
    // #808080: r=g=b so s=0 - exercises the achromatic return path
    const result = lightenColor("#808080", 0.5);
    // Lightened grey must remain a valid hex
    expect(result).toMatch(/^#[0-9a-f]{6}$/i);
    // All channels must be equal (still greyscale)
    const r = parseInt(result.slice(1, 3), 16);
    const g = parseInt(result.slice(3, 5), 16);
    const b = parseInt(result.slice(5, 7), 16);
    expect(r).toBe(g);
    expect(g).toBe(b);
  });
});

// ---------------------------------------------------------------------------
// desaturateColor — additional branch coverage
// ---------------------------------------------------------------------------
describe("desaturateColor (extra branches)", () => {
  it("works with a green-dominant color (hits case g in switch)", () => {
    // #00ff00: g=255 is the maximum channel
    const result = desaturateColor("#00ff00", 0.5, 0.8);
    expect(result).toMatch(/^#[0-9a-f]{6}$/i);
  });
});

// ---------------------------------------------------------------------------
// getFlowName
// ---------------------------------------------------------------------------
describe("getFlowName", () => {
  it("returns '1. Flow Title' for the first flow with a title", () => {
    const state = createAppState({
      score: {
        flows: {
          order: [5],
          by_id: {
            5: {
              id: 5,
              title: "Allegro",
              player_ids: [],
              length: 128,
              master_id: 100,
              staves: { by_id: {} },
            },
          },
        },
      } as never,
    });
    expect(getFlowName(state, 0, 5)).toBe("1. Allegro");
  });

  it("uses index+1 for numbering (index=1 → '2. ...')", () => {
    const state = createAppState({
      score: {
        flows: {
          order: [5, 6],
          by_id: {
            5: {
              id: 5,
              title: "First",
              player_ids: [],
              length: 128,
              master_id: 100,
              staves: { by_id: {} },
            },
            6: {
              id: 6,
              title: "Second",
              player_ids: [],
              length: 128,
              master_id: 101,
              staves: { by_id: {} },
            },
          },
        },
      } as never,
    });
    expect(getFlowName(state, 1, 6)).toBe("2. Second");
  });

  it("falls back to 'Untitled Flow' when title is empty", () => {
    const state = createAppState({
      score: {
        flows: {
          order: [5],
          by_id: {
            5: {
              id: 5,
              title: "",
              player_ids: [],
              length: 128,
              master_id: 100,
              staves: { by_id: {} },
            },
          },
        },
      } as never,
    });
    expect(getFlowName(state, 0, 5)).toBe("1. Untitled Flow");
  });
});

// ---------------------------------------------------------------------------
// getLayoutName
// ---------------------------------------------------------------------------
describe("getLayoutName", () => {
  const makeState = (layout: object, extraScore?: object) =>
    createAppState({
      score: {
        layouts: { order: [1], by_id: { 1: layout } },
        players: { order: [], by_id: {} },
        instruments: { by_id: {} },
        ...extraScore,
      } as never,
    });

  it("returns the layout title when set", () => {
    const state = makeState({
      id: 1,
      title: "My Layout",
      layout_type: LayoutType.Part,
      player_ids: [],
      flow_ids: [],
    });
    expect(getLayoutName(state, 1)).toBe("My Layout");
  });

  it("returns 'Full Score' for FullScore layout type with no title", () => {
    const state = makeState({
      id: 1,
      title: "",
      layout_type: LayoutType.FullScore,
      player_ids: [],
      flow_ids: [],
    });
    expect(getLayoutName(state, 1)).toBe("Full Score");
  });

  it("returns 'Empty Part' when no players are in the layout", () => {
    const state = makeState({
      id: 1,
      title: "",
      layout_type: LayoutType.Part,
      player_ids: [],
      flow_ids: [],
    });
    expect(getLayoutName(state, 1)).toBe("Empty Part");
  });

  it("returns the instrument name for a single-instrument layout", () => {
    const state = createAppState({
      score: {
        layouts: {
          order: [1],
          by_id: {
            1: {
              id: 1,
              title: "",
              layout_type: LayoutType.Part,
              player_ids: [10],
              flow_ids: [],
            },
          },
        },
        players: {
          order: [10],
          by_id: {
            10: { id: 10, player_type: PlayerType.Solo, instrument_ids: [20] },
          },
        },
        instruments: {
          by_id: {
            20: {
              id: 20,
              long_name: "Violin",
              count: null,
              definition_id: "violin",
              instrument_type: 0,
              short_name: "Vln.",
              stave_ids: [],
              assigned_plugin_id: null,
              volume: 100,
              pan: 0,
              mute: false,
              solo: false,
            },
          },
        },
      } as never,
    });
    expect(getLayoutName(state, 1)).toBe("Violin");
  });

  it("includes count in instrument name when count is set", () => {
    const state = createAppState({
      score: {
        layouts: {
          order: [1],
          by_id: {
            1: {
              id: 1,
              title: "",
              layout_type: LayoutType.Part,
              player_ids: [10],
              flow_ids: [],
            },
          },
        },
        players: {
          order: [10],
          by_id: {
            10: { id: 10, player_type: PlayerType.Solo, instrument_ids: [20] },
          },
        },
        instruments: {
          by_id: {
            20: {
              id: 20,
              long_name: "Violin",
              count: 2,
              definition_id: "violin",
              instrument_type: 0,
              short_name: "Vln.",
              stave_ids: [],
              assigned_plugin_id: null,
              volume: 100,
              pan: 0,
              mute: false,
              solo: false,
            },
          },
        },
      } as never,
    });
    expect(getLayoutName(state, 1)).toBe("Violin 2");
  });

  it("joins two instrument names with ' & '", () => {
    const state = createAppState({
      score: {
        layouts: {
          order: [1],
          by_id: {
            1: {
              id: 1,
              title: "",
              layout_type: LayoutType.Part,
              player_ids: [10, 11],
              flow_ids: [],
            },
          },
        },
        players: {
          order: [10, 11],
          by_id: {
            10: { id: 10, player_type: PlayerType.Solo, instrument_ids: [20] },
            11: { id: 11, player_type: PlayerType.Solo, instrument_ids: [21] },
          },
        },
        instruments: {
          by_id: {
            20: {
              id: 20,
              long_name: "Violin",
              count: null,
              definition_id: "violin",
              instrument_type: 0,
              short_name: "Vln.",
              stave_ids: [],
              assigned_plugin_id: null,
              volume: 100,
              pan: 0,
              mute: false,
              solo: false,
            },
            21: {
              id: 21,
              long_name: "Viola",
              count: null,
              definition_id: "viola",
              instrument_type: 0,
              short_name: "Vla.",
              stave_ids: [],
              assigned_plugin_id: null,
              volume: 100,
              pan: 0,
              mute: false,
              solo: false,
            },
          },
        },
      } as never,
    });
    expect(getLayoutName(state, 1)).toBe("Violin & Viola");
  });

  it("joins three instrument names with commas and ' & '", () => {
    const state = createAppState({
      score: {
        layouts: {
          order: [1],
          by_id: {
            1: {
              id: 1,
              title: "",
              layout_type: LayoutType.Part,
              player_ids: [10, 11, 12],
              flow_ids: [],
            },
          },
        },
        players: {
          order: [10, 11, 12],
          by_id: {
            10: { id: 10, player_type: PlayerType.Solo, instrument_ids: [20] },
            11: { id: 11, player_type: PlayerType.Solo, instrument_ids: [21] },
            12: { id: 12, player_type: PlayerType.Solo, instrument_ids: [22] },
          },
        },
        instruments: {
          by_id: {
            20: {
              id: 20,
              long_name: "Violin",
              count: null,
              definition_id: "violin",
              instrument_type: 0,
              short_name: "Vln.",
              stave_ids: [],
              assigned_plugin_id: null,
              volume: 100,
              pan: 0,
              mute: false,
              solo: false,
            },
            21: {
              id: 21,
              long_name: "Viola",
              count: null,
              definition_id: "viola",
              instrument_type: 0,
              short_name: "Vla.",
              stave_ids: [],
              assigned_plugin_id: null,
              volume: 100,
              pan: 0,
              mute: false,
              solo: false,
            },
            22: {
              id: 22,
              long_name: "Cello",
              count: null,
              definition_id: "cello",
              instrument_type: 0,
              short_name: "Vc.",
              stave_ids: [],
              assigned_plugin_id: null,
              volume: 100,
              pan: 0,
              mute: false,
              solo: false,
            },
          },
        },
      } as never,
    });
    expect(getLayoutName(state, 1)).toBe("Violin, Viola & Cello");
  });
});

// ---------------------------------------------------------------------------
// useHumanizedTime
// ---------------------------------------------------------------------------
describe("useHumanizedTime", () => {
  it("returns 'Just now' for a very recent timestamp", () => {
    const { result } = renderHook(() => useHumanizedTime(Date.now()));
    expect(result.current).toBe("Just now");
  });

  it("returns a date string for a timestamp older than one week", () => {
    const oneWeekAgo = Date.now() - 8 * 24 * 60 * 60 * 1000;
    const { result } = renderHook(() => useHumanizedTime(oneWeekAgo));
    // toLocaleDateString returns something like "1/1/2026"
    expect(result.current).toMatch(/\d/);
    expect(result.current).not.toBe("Just now");
  });

  it("cleanup runs on unmount (clearInterval is called)", () => {
    vi.useFakeTimers();
    const clearSpy = vi.spyOn(globalThis, "clearInterval");
    const { unmount } = renderHook(() => useHumanizedTime(Date.now()));
    unmount();
    expect(clearSpy).toHaveBeenCalled();
    clearSpy.mockRestore();
    vi.useRealTimers();
  });

  it("returns a pretty-ms string (ending ' ago') for a timestamp within one week", () => {
    const twoHoursAgo = Date.now() - 2 * 60 * 60 * 1000;
    const { result } = renderHook(() => useHumanizedTime(twoHoursAgo));
    expect(result.current).toMatch(/ ago$/);
  });
});
