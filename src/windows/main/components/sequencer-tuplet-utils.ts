import { Tuplet } from "../../../services/types";
import { noteDurationToTicks } from "../../../services/utils";

export type TupletRegion = {
  tupletId: number;
  startTick: number;
  endTick: number;
  localLengthTicks: number;
};

export const getTupletHostTicks = (tuplet: Tuplet) => {
  return tuplet.in_space_of * noteDurationToTicks(tuplet.base_note_value);
};

export const getTupletRatioLabel = (tuplet: Tuplet) => {
  return `${tuplet.note_count}:${tuplet.in_space_of}`;
};

export const getTupletRegion = (tuplet: Tuplet): TupletRegion => {
  return {
    tupletId: tuplet.id,
    startTick: tuplet.tick,
    endTick: tuplet.tick + getTupletHostTicks(tuplet),
    localLengthTicks:
      tuplet.note_count * noteDurationToTicks(tuplet.base_note_value),
  };
};

export const isTickInsideTupletRegion = (
  tick: number,
  tuplet: TupletRegion,
) => {
  return tick >= tuplet.startTick && tick < tuplet.endTick;
};

export const getTupletLocalTickWidth = (tuplet: Tuplet, tickWidth: number) => {
  const localLengthTicks =
    tuplet.note_count * noteDurationToTicks(tuplet.base_note_value);

  if (localLengthTicks === 0) {
    return 0;
  }

  return (getTupletHostTicks(tuplet) * tickWidth) / localLengthTicks;
};

export const getTupletLocalTickFromClientX = (
  clientXWithinTrack: number,
  tuplet: Tuplet,
  tickWidth: number,
  ticksPerSnap: number,
  snapTick: (
    x: number,
    tickWidth: number,
    ticksPerSnap: number,
    direction: "up" | "down" | "nearest",
  ) => number,
) => {
  const localTickWidth = getTupletLocalTickWidth(tuplet, tickWidth);
  const tupletStartX = tuplet.tick * tickWidth;
  const localX = Math.max(0, clientXWithinTrack - tupletStartX);
  const localLengthTicks =
    tuplet.note_count * noteDurationToTicks(tuplet.base_note_value);
  const snappedLocalTick = snapTick(
    localX,
    localTickWidth,
    ticksPerSnap,
    "down",
  );

  return Math.max(
    0,
    Math.min(localLengthTicks - ticksPerSnap, snappedLocalTick),
  );
};

export const hexToRgba = (hexColor: string, alpha: number) => {
  const hex = hexColor.replace("#", "");
  const normalized =
    hex.length === 3
      ? hex
          .split("")
          .map((part) => part + part)
          .join("")
      : hex;

  const red = parseInt(normalized.slice(0, 2), 16);
  const green = parseInt(normalized.slice(2, 4), 16);
  const blue = parseInt(normalized.slice(4, 6), 16);

  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
};
