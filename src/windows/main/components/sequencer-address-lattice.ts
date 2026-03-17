import { Tick, Tuplet, UniqueId } from "../../../services/types";
import { noteDurationToTicks } from "../../../services/utils";
import {
  getTupletHostTicks,
  getTupletLocalTickWidth,
} from "./sequencer-tuplet-utils";

export type AxisAddress = {
  tuplet_id: UniqueId | null;
  tick: Tick;
};

export type AxisPoint = {
  address: AxisAddress;
  x: number;
};

export type SequencerAddressLattice = {
  continuous: AxisPoint[];
  byAddressKey: Map<string, number>;
};

export type SnapPoint = {
  continuousIndex: number;
  address: AxisAddress;
  x: number;
};

export type SequencerSnapOverlay = {
  ticksPerSnap: number;
  points: SnapPoint[];
};

export type PixelSpan = {
  startX: number;
  endX: number;
  width: number;
};

export type AddressLookupDirection = "nearest" | "floor" | "ceil";

export const addressKey = (address: AxisAddress) => {
  return address.tuplet_id === null
    ? `host:${address.tick}`
    : `tuplet:${address.tuplet_id}:${address.tick}`;
};

const buildHostPoints = (
  continuous: AxisPoint[],
  startTick: number,
  endTick: number,
  tickWidth: number,
) => {
  for (let tick = startTick; tick <= endTick; tick += 1) {
    continuous.push({
      address: { tuplet_id: null, tick },
      x: tick * tickWidth,
    });
  }
};

const buildTupletPoints = (
  continuous: AxisPoint[],
  tuplet: Tuplet,
  tickWidth: number,
) => {
  const localTickWidth = getTupletLocalTickWidth(tuplet, tickWidth);
  const localLengthTicks =
    localTickWidth === 0
      ? 0
      : tuplet.note_count * noteDurationToTicks(tuplet.base_note_value);

  if (localLengthTicks <= 0) {
    return;
  }

  const startX = tuplet.tick * tickWidth;

  for (let tick = 0; tick < localLengthTicks; tick += 1) {
    continuous.push({
      address: { tuplet_id: tuplet.id, tick },
      x: startX + tick * localTickWidth,
    });
  }
};

export const buildSequencerAddressLattice = (
  flowLength: number,
  tickWidth: number,
  tuplets: Tuplet[],
): SequencerAddressLattice => {
  const continuous: AxisPoint[] = [];
  const orderedTuplets = [...tuplets].sort(
    (left, right) => left.tick - right.tick,
  );
  let hostTick = 0;

  for (const tuplet of orderedTuplets) {
    if (hostTick < tuplet.tick) {
      buildHostPoints(continuous, hostTick, tuplet.tick - 1, tickWidth);
    }

    buildTupletPoints(continuous, tuplet, tickWidth);
    hostTick = tuplet.tick + getTupletHostTicks(tuplet);
  }

  if (hostTick <= flowLength) {
    buildHostPoints(continuous, hostTick, flowLength, tickWidth);
  }

  return {
    continuous,
    byAddressKey: new Map(
      continuous.map((point, index) => [addressKey(point.address), index]),
    ),
  };
};

export const getIndexForAddress = (
  lattice: SequencerAddressLattice,
  address: AxisAddress,
) => {
  return lattice.byAddressKey.get(addressKey(address)) ?? null;
};

export const getAddressAtIndex = (
  lattice: SequencerAddressLattice,
  index: number,
) => {
  return lattice.continuous[index]?.address ?? null;
};

export const getXForAddress = (
  lattice: SequencerAddressLattice,
  address: AxisAddress,
) => {
  const index = getIndexForAddress(lattice, address);

  return index === null ? null : (lattice.continuous[index]?.x ?? null);
};

const findClosestIndexByX = (
  points: Array<{ x: number }>,
  x: number,
  direction: AddressLookupDirection,
) => {
  if (points.length === 0) {
    return null;
  }

  if (x <= points[0].x) {
    return 0;
  }

  const lastIndex = points.length - 1;

  if (x >= points[lastIndex].x) {
    return lastIndex;
  }

  let low = 0;
  let high = lastIndex;

  while (low <= high) {
    const middle = Math.floor((low + high) / 2);
    const middleX = points[middle]?.x ?? 0;

    if (middleX === x) {
      return middle;
    }

    if (middleX < x) {
      low = middle + 1;
    } else {
      high = middle - 1;
    }
  }

  const floorIndex = Math.max(0, high);
  const ceilIndex = Math.min(lastIndex, low);

  switch (direction) {
    case "floor":
      return floorIndex;
    case "ceil":
      return ceilIndex;
    default:
      return x - points[floorIndex].x <= points[ceilIndex].x - x
        ? floorIndex
        : ceilIndex;
  }
};

export const findContinuousIndexForX = (
  lattice: SequencerAddressLattice,
  x: number,
  direction: AddressLookupDirection,
) => {
  return findClosestIndexByX(lattice.continuous, x, direction);
};

export const findAddressForX = (
  lattice: SequencerAddressLattice,
  x: number,
  direction: AddressLookupDirection,
) => {
  const index = findContinuousIndexForX(lattice, x, direction);

  return index === null ? null : getAddressAtIndex(lattice, index);
};

export const advanceIndex = (
  lattice: SequencerAddressLattice,
  startIndex: number,
  duration: number,
) => {
  const nextIndex = startIndex + duration;

  if (nextIndex < 0 || nextIndex >= lattice.continuous.length) {
    return null;
  }

  return nextIndex;
};

export const getSpanForAddressDuration = (
  lattice: SequencerAddressLattice,
  start: AxisAddress,
  duration: number,
): PixelSpan | null => {
  const startIndex = getIndexForAddress(lattice, start);

  if (startIndex === null) {
    return null;
  }

  const endIndex = advanceIndex(lattice, startIndex, duration);

  if (endIndex === null) {
    return null;
  }

  const startX = lattice.continuous[startIndex]?.x;
  const endX = lattice.continuous[endIndex]?.x;

  if (startX === undefined || endX === undefined) {
    return null;
  }

  return {
    startX,
    endX,
    width: endX - startX,
  };
};

export const buildSnapOverlay = (
  lattice: SequencerAddressLattice,
  ticksPerSnap: number,
): SequencerSnapOverlay => {
  const normalizedTicksPerSnap = Math.max(1, ticksPerSnap);
  const points = lattice.continuous
    .map((point, continuousIndex) => ({ point, continuousIndex }))
    .filter(({ point }) => point.address.tick % normalizedTicksPerSnap === 0)
    .map(({ point, continuousIndex }) => ({
      continuousIndex,
      address: point.address,
      x: point.x,
    }));

  return {
    ticksPerSnap: normalizedTicksPerSnap,
    points,
  };
};

export const findSnappedAddressForX = (
  overlay: SequencerSnapOverlay,
  x: number,
  direction: AddressLookupDirection,
) => {
  const index = findClosestIndexByX(overlay.points, x, direction);

  return index === null ? null : (overlay.points[index]?.address ?? null);
};
