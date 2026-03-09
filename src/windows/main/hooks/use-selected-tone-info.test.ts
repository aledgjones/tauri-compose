import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { useSelectedToneInfo } from "./use-selected-tone-info";
import { createWrapper, createAppState } from "../../../services/test-utils";
import { Accidental, PlayerType } from "../../../services/types";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------
const FLOW_ID = 5;
const INSTRUMENT_ID = 10;
const STAVE_ID = 100;
const TRACK_ID = 300;
const TONE_ID = 999;

const buildBaseScore = () => ({
  players: {
    order: [1],
    by_id: {
      1: {
        id: 1,
        player_type: PlayerType.Solo,
        instrument_ids: [INSTRUMENT_ID],
      },
    },
  },
  instruments: {
    by_id: {
      [INSTRUMENT_ID]: {
        id: INSTRUMENT_ID,
        definition_id: "piano",
        instrument_type: 0,
        long_name: "Piano",
        short_name: "Pno.",
        stave_ids: [STAVE_ID],
        count: null,
        assigned_plugin_id: null,
        volume: 100,
        pan: 0,
        mute: false,
        solo: false,
      },
    },
  },
  flows: {
    order: [FLOW_ID],
    by_id: {
      [FLOW_ID]: {
        id: FLOW_ID,
        title: "Flow 1",
        player_ids: [1],
        length: 512,
        master_id: 200,
        staves: {
          by_id: {
            [STAVE_ID]: {
              id: STAVE_ID,
              lines: [1, 1, 1, 1, 1],
              master_id: 200,
              track_ids: [TRACK_ID],
            },
          },
        },
      },
    },
  },
  tracks: {
    by_id: {
      [TRACK_ID]: {
        id: TRACK_ID,
        entries: {
          by_tick: {
            [64]: [TONE_ID],
          },
          by_id: {
            [TONE_ID]: {
              id: TONE_ID,
              tick: 64,
              type: "Tone" as const,
              duration: 32,
              pitch: { int: 60, accidental: Accidental.Natural },
              velocity: { int: 100 },
              articulations: [],
            },
          },
        },
      },
    },
  },
});

describe("useSelectedToneInfo", () => {
  it("returns null when selection is null", () => {
    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: createAppState() }),
    });
    expect(result.current).toBeNull();
  });

  it("returns null for tempo selections", () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tempo",
            flow_id: FLOW_ID,
            tempo_id: 123,
            tick: 64,
          },
        },
      } as never,
    });

    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null for directive selections without track context", () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "TimeSignature",
            flow_id: FLOW_ID,
            time_signature_id: 123,
            tick: 64,
          },
        },
      } as never,
    });

    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null when selection references a missing track", () => {
    const state = createAppState({
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: FLOW_ID,
            instrument_id: INSTRUMENT_ID,
            stave_id: STAVE_ID,
            track_id: 9999, // non-existent
            tone_id: TONE_ID,
          },
        },
      } as never,
    });
    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current).toBeNull();
  });

  it('returns { noteName: "Rest", instrumentName, trackLabel } for Rest selection', () => {
    const score = buildBaseScore();
    const state = createAppState({
      score: score as never,
      ui: {
        sequencer: {
          selection: {
            type: "Rest",
            flow_id: FLOW_ID,
            instrument_id: INSTRUMENT_ID,
            stave_id: STAVE_ID,
            track_id: TRACK_ID,
            tick: 64,
          },
        },
      } as never,
    });

    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current?.noteName).toBe("Rest");
    expect(result.current?.instrumentName).toBe("Piano");
  });

  it("returns correct noteName for a Tone selection (C4 = midi 60)", () => {
    const score = buildBaseScore();
    const state = createAppState({
      score: score as never,
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: FLOW_ID,
            instrument_id: INSTRUMENT_ID,
            stave_id: STAVE_ID,
            track_id: TRACK_ID,
            tone_id: TONE_ID,
          },
        },
      } as never,
    });

    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current?.noteName).toBeDefined();
    expect(result.current?.instrumentName).toBe("Piano");
    expect(result.current?.trackLabel).toBeDefined();
  });

  it("builds trackLabel for single-stave instrument", () => {
    const score = buildBaseScore();
    const state = createAppState({
      score: score as never,
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: FLOW_ID,
            instrument_id: INSTRUMENT_ID,
            stave_id: STAVE_ID,
            track_id: TRACK_ID,
            tone_id: TONE_ID,
          },
        },
      } as never,
    });

    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: state }),
    });

    // Single-stave: no "Staff (A)," prefix, just voice label
    expect(result.current?.trackLabel).toContain("voice");
    expect(result.current?.trackLabel).not.toContain("Staff");
  });

  it("returns null when Tone selection exists but tone_id is not found in track entries", () => {
    // Track exists, but the tone with the selected tone_id is absent
    const score = buildBaseScore();
    // Override the track with an empty entries set
    score.tracks.by_id[TRACK_ID] = {
      id: TRACK_ID,
      entries: { by_tick: {}, by_id: {} },
    } as never;

    const state = createAppState({
      score: score as never,
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: FLOW_ID,
            instrument_id: INSTRUMENT_ID,
            stave_id: STAVE_ID,
            track_id: TRACK_ID,
            tone_id: TONE_ID, // not present in empty track
          },
        },
      } as never,
    });

    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current).toBeNull();
  });

  it("returns instrumentName as 'Unknown' when instrument is missing", () => {
    // Track + selection exist but instruments.by_id is empty
    const score = buildBaseScore();
    delete (score.instruments.by_id as Record<number, unknown>)[INSTRUMENT_ID];

    const state = createAppState({
      score: score as never,
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: FLOW_ID,
            instrument_id: INSTRUMENT_ID,
            stave_id: STAVE_ID,
            track_id: TRACK_ID,
            tone_id: TONE_ID,
          },
        },
      } as never,
    });

    const { result } = renderHook(() => useSelectedToneInfo(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current?.instrumentName).toBe("Unknown");
  });
});
