import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { useTicks } from "./use-ticks";
import { NoteDuration } from "../../../services/types";
import type { Flow, Track } from "../../../services/types";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------
const makeMasterTrack = (
  timeSigId: number,
  beats: number,
  beatType: NoteDuration,
): Track => ({
  id: 100,
  entries: {
    by_tick: { 0: [timeSigId] },
    by_id: {
      [timeSigId]: {
        id: timeSigId,
        tick: 0,
        type: "TimeSignature",
        beats,
        beat_type: beatType,
      },
    },
  },
});

const makeFlow = (length: number): Flow => ({
  id: 1,
  title: "Flow 1",
  player_ids: [],
  length,
  master_id: 100,
  staves: { by_id: {} },
});

describe("useTicks", () => {
  it("returns empty config when flow and master are null", () => {
    const { result } = renderHook(() => useTicks(null, null, 1));
    expect(result.current).toEqual({ tickWidth: 0, entries: [], length: 0 });
  });

  it("returns empty config when only flow is null", () => {
    const master = makeMasterTrack(1, 4, NoteDuration.Quarter);
    const { result } = renderHook(() => useTicks(null, master, 1));
    expect(result.current).toEqual({ tickWidth: 0, entries: [], length: 0 });
  });

  it("returns bar entries at measure boundaries for 4/4", () => {
    const flow = makeFlow(512);
    const master = makeMasterTrack(1, 4, NoteDuration.Quarter);
    const { result } = renderHook(() => useTicks(flow, master, 1));
    const barEntries = result.current.entries.filter((e) => e.type === "bar");
    // 512 ticks / 128 bar ticks = 4 bars
    expect(barEntries).toHaveLength(4);
    expect(barEntries[0].barNumber).toBe(1);
    expect(barEntries[3].barNumber).toBe(4);
  });

  it("returns beat entries between bars for 4/4", () => {
    const flow = makeFlow(128); // one bar
    const master = makeMasterTrack(1, 4, NoteDuration.Quarter);
    const { result } = renderHook(() => useTicks(flow, master, 1));
    const beatEntries = result.current.entries.filter((e) => e.type === "beat");
    // beats at ticks 32, 64, 96 (4 beats per bar, minus the bar tick at 0)
    expect(beatEntries).toHaveLength(3);
  });

  it("doubles tickWidth when zoom=2", () => {
    const flow = makeFlow(128);
    const master = makeMasterTrack(1, 4, NoteDuration.Quarter);
    const { result: r1 } = renderHook(() => useTicks(flow, master, 1));
    const { result: r2 } = renderHook(() => useTicks(flow, master, 2));
    expect(r2.current.tickWidth).toBe(r1.current.tickWidth * 2);
  });

  it("returns the flow length as the length property", () => {
    const flow = makeFlow(256);
    const master = makeMasterTrack(1, 4, NoteDuration.Quarter);
    const { result } = renderHook(() => useTicks(flow, master, 1));
    expect(result.current.length).toBe(256);
  });

  it("returns sub entries for 4/4 time (beat_type=Quarter < Eighth)", () => {
    // Quarter (4) < Eighth (8), so sub-beat ticks should be generated
    const flow = makeFlow(128); // one bar
    const master = makeMasterTrack(1, 4, NoteDuration.Quarter);
    const { result } = renderHook(() => useTicks(flow, master, 1));
    const subEntries = result.current.entries.filter((e) => e.type === "sub");
    // In 4/4: beatTicks=32, beatTicks/2=16. Sub entries at 16, 48, 80, 112 → 4 subs
    expect(subEntries.length).toBeGreaterThan(0);
  });

  it("returns NO sub entries for 6/8 time (beat_type=Eighth, not < Eighth)", () => {
    // Eighth (8) is NOT < Eighth (8), so the sub branch is false
    const flow = makeFlow(192); // 6/8 bar: beatTicks=(32*4)/8=16, barTicks=6*16=96 → ~2 bars
    const master = makeMasterTrack(1, 6, NoteDuration.Eighth);
    const { result } = renderHook(() => useTicks(flow, master, 1));
    const subEntries = result.current.entries.filter((e) => e.type === "sub");
    expect(subEntries).toHaveLength(0);
  });

  it("handles time signature change at a mid-flow tick", () => {
    // Two time sigs: 4/4 at tick 0, then 3/4 at tick 128
    const timeSig4_4Id = 1;
    const timeSig3_4Id = 2;
    const master: import("../../../services/types").Track = {
      id: 100,
      entries: {
        by_tick: { 0: [timeSig4_4Id], 128: [timeSig3_4Id] },
        by_id: {
          [timeSig4_4Id]: {
            id: timeSig4_4Id,
            tick: 0,
            type: "TimeSignature",
            beats: 4,
            beat_type: NoteDuration.Quarter,
          },
          [timeSig3_4Id]: {
            id: timeSig3_4Id,
            tick: 128,
            type: "TimeSignature",
            beats: 3,
            beat_type: NoteDuration.Quarter,
          },
        },
      },
    };
    const flow = makeFlow(256);
    const { result } = renderHook(() => useTicks(flow, master, 1));
    const barEntries = result.current.entries.filter((e) => e.type === "bar");
    // Should have bars starting at 0 and 128
    expect(barEntries.some((e) => e.x === 0)).toBe(true);
    expect(barEntries.some((e) => e.x === 128 * result.current.tickWidth)).toBe(
      true,
    );
  });
});
