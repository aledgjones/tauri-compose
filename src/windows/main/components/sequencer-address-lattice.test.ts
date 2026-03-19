import { describe, expect, it } from "vitest";
import { NoteDuration, Tuplet } from "../../../services/types";
import {
  addressKey,
  buildSequencerAddressLattice,
  buildSnapOverlay,
  findSnappedAddressForX,
  getIndexForAddress,
  getSpanForAddressDuration,
} from "./sequencer-address-lattice";

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

describe("sequencer-address-lattice", () => {
  it("replaces the host span inside a tuplet with canonical local addresses", () => {
    const lattice = buildSequencerAddressLattice(128, 4, [makeTuplet()]);

    expect(getIndexForAddress(lattice, { tuplet_id: null, tick: 31 })).toBe(31);
    expect(
      getIndexForAddress(lattice, { tuplet_id: null, tick: 32 }),
    ).toBeNull();
    expect(getIndexForAddress(lattice, { tuplet_id: 11, tick: 0 })).toBe(32);
    expect(getIndexForAddress(lattice, { tuplet_id: 11, tick: 47 })).toBe(79);
    expect(getIndexForAddress(lattice, { tuplet_id: null, tick: 64 })).toBe(80);
  });

  it("projects a crossing host tone through the canonical address span", () => {
    const lattice = buildSequencerAddressLattice(128, 4, [makeTuplet()]);
    const span = getSpanForAddressDuration(
      lattice,
      { tuplet_id: null, tick: 16 },
      32,
    );

    expect(span).not.toBeNull();
    expect(span?.startX).toBe(64);
    expect(span?.width).toBeCloseTo(106.6666667, 5);
  });

  it("builds snap lookup from the canonical lattice rather than host ticks", () => {
    const lattice = buildSequencerAddressLattice(128, 4, [makeTuplet()]);
    const overlay = buildSnapOverlay(lattice, 16);
    const snapped = findSnappedAddressForX(overlay, 171, "nearest");

    expect(snapped).not.toBeNull();
    expect(addressKey(snapped!)).toBe("tuplet:11:16");
  });

  it("keeps tuplet-local snap phase when the tuplet starts off the host snap grid", () => {
    const lattice = buildSequencerAddressLattice(128, 4, [
      makeTuplet({ tick: 8 }),
    ]);
    const overlay = buildSnapOverlay(lattice, 16);
    const snapped = findSnappedAddressForX(overlay, 60, "nearest");

    expect(snapped).not.toBeNull();
    expect(addressKey(snapped!)).toBe("tuplet:11:16");
  });

  it("keeps the tuplet exit boundary addressable before the next host snap point", () => {
    const lattice = buildSequencerAddressLattice(128, 4, [
      makeTuplet({ tick: 8 }),
    ]);
    const overlay = buildSnapOverlay(lattice, 16);

    expect(addressKey(findSnappedAddressForX(overlay, 159, "nearest")!)).toBe(
      "host:40",
    );
    expect(addressKey(findSnappedAddressForX(overlay, 181, "nearest")!)).toBe(
      "host:48",
    );
  });
});
