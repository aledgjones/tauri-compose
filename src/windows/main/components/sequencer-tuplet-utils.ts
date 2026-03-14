import { Tuplet } from "../../../services/types";
import { noteDurationToTicks } from "../../../services/utils";

export type TupletRegion = {
  startTick: number;
  endTick: number;
};

export const getTupletHostTicks = (tuplet: Tuplet) => {
  return tuplet.in_space_of * noteDurationToTicks(tuplet.base_note_value);
};

export const getTupletRatioLabel = (tuplet: Tuplet) => {
  return `${tuplet.note_count}:${tuplet.in_space_of}`;
};

export const getTupletRegion = (tuplet: Tuplet): TupletRegion => {
  return {
    startTick: tuplet.tick,
    endTick: tuplet.tick + getTupletHostTicks(tuplet),
  };
};

export const isTickInsideTupletRegion = (
  tick: number,
  tuplet: TupletRegion,
) => {
  return tick >= tuplet.startTick && tick < tuplet.endTick;
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
