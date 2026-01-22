import { useMemo } from "react";
import {
  Flow,
  NoteDuration,
  TimeSignature,
  Track,
} from "../../../services/types";

export type Ticks = {
  x: number;
  type: "bar" | "beat" | "sub";
  barNumber?: number;
}[];
export type TickConfig = { tickWidth: number; entries: Ticks; length: number };

export const useTicks = (
  flow: Flow | null,
  master: Track | null,
  zoom: number = 1,
) => {
  return useMemo<TickConfig>(() => {
    if (!flow || !master) {
      return { tickWidth: 0, entries: [], length: 0 };
    }

    const subdivisions = 32;
    const quarterWidth = 80 * zoom;
    const tickWidth = quarterWidth / subdivisions;

    const entries: Ticks = [];

    let currentTimeSig: TimeSignature | null = null;
    let barStartTick = 0;
    let beatTicks = 0;
    let barTicks = 0;
    let barNumber = 1;

    for (let tick = 0; tick < flow.length; tick++) {
      if (master.entries.by_tick[tick]) {
        const timeSigId = master.entries.by_tick[tick].find(
          (id) => master.entries.by_id[id]?.type === "TimeSignature",
        );
        if (timeSigId) {
          currentTimeSig = master.entries.by_id[timeSigId] as TimeSignature;
          barStartTick = tick;
          beatTicks = (subdivisions * 4) / currentTimeSig.beat_type;
          barTicks = currentTimeSig.beats * beatTicks;
        }
      }

      if (currentTimeSig) {
        const tickOffset = tick - barStartTick;
        if (tickOffset % barTicks === 0) {
          entries.push({ x: tick * tickWidth, type: "bar", barNumber });
          barNumber++;
        } else if (tickOffset % beatTicks === 0) {
          entries.push({ x: tick * tickWidth, type: "beat" });
        } else if (
          currentTimeSig.beat_type < NoteDuration.Eighth &&
          tickOffset % (beatTicks / 2) === 0
        ) {
          entries.push({ x: tick * tickWidth, type: "sub" });
        }
      }
    }

    return { entries, tickWidth, length: flow.length };
  }, [flow, master, zoom]);
};
