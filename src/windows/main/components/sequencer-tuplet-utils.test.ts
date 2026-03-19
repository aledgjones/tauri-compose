import { describe, expect, it, vi } from "vitest";
import { NoteDuration, Tuplet } from "../../../services/types";
import {
  getTupletHostTicks,
  getTupletLocalTickFromClientX,
  getTupletLocalTickWidth,
  getTupletRatioLabel,
  getTupletRegion,
  hexToRgba,
  isTickInsideTupletRegion,
} from "./sequencer-tuplet-utils";

const makeTuplet = (overrides: Partial<Tuplet> = {}): Tuplet => ({
  id: 11,
  tick: 32,
  type: "Tuplet",
  note_count: 3,
  in_space_of: 2,
  base_note_value: NoteDuration.Eighth,
  groupings: [3],
  entries: { by_tick: {}, by_id: {} },
  ...overrides,
});

describe("sequencer-tuplet-utils", () => {
  it("derives host span and display metadata from the tuplet ratio", () => {
    const tuplet = makeTuplet();
    const region = getTupletRegion(tuplet);

    expect(getTupletHostTicks(tuplet)).toBe(32);
    expect(getTupletRatioLabel(tuplet)).toBe("3:2");
    expect(region).toEqual({
      tupletId: 11,
      startTick: 32,
      endTick: 64,
      localLengthTicks: 48,
    });
  });

  it("treats the tuplet region as start-inclusive and end-exclusive", () => {
    const region = getTupletRegion(makeTuplet());

    expect(isTickInsideTupletRegion(32, region)).toBe(true);
    expect(isTickInsideTupletRegion(63, region)).toBe(true);
    expect(isTickInsideTupletRegion(64, region)).toBe(false);
  });

  it("maps client x into local tuplet ticks and clamps the snap result", () => {
    const tuplet = makeTuplet();
    const snapTick =
      vi.fn<
        (
          x: number,
          tickWidth: number,
          ticksPerSnap: number,
          direction: "up" | "down" | "nearest",
        ) => number
      >();

    snapTick.mockReturnValueOnce(-8);
    const startClamped = getTupletLocalTickFromClientX(
      120,
      tuplet,
      4,
      4,
      snapTick,
    );

    snapTick.mockReturnValueOnce(60);
    const endClamped = getTupletLocalTickFromClientX(
      200,
      tuplet,
      4,
      4,
      snapTick,
    );

    expect(getTupletLocalTickWidth(tuplet, 4)).toBeCloseTo(2.6666667, 5);
    expect(snapTick).toHaveBeenNthCalledWith(
      1,
      0,
      2.6666666666666665,
      4,
      "down",
    );
    expect(snapTick).toHaveBeenNthCalledWith(
      2,
      72,
      2.6666666666666665,
      4,
      "down",
    );
    expect(startClamped).toBe(0);
    expect(endClamped).toBe(44);
  });

  it("converts short hex colors to rgba", () => {
    expect(hexToRgba("#abc", 0.25)).toBe("rgba(170, 187, 204, 0.25)");
  });
});
