import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { useBottomPanelState } from "./use-bottom-panel-state";
import { createWrapper, createAppState } from "../../../services/test-utils";
import { PlayerType, Tool, NoteDuration } from "../../../services/types";

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------
const buildState = () =>
  createAppState({
    score: {
      players: {
        order: [1],
        by_id: {
          1: { id: 1, player_type: PlayerType.Solo, instrument_ids: [10] },
        },
      },
      instruments: {
        by_id: {
          10: {
            id: 10,
            definition_id: "piano",
            instrument_type: 0,
            long_name: "Piano",
            short_name: "Pno.",
            stave_ids: [100],
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
        order: [5],
        by_id: {
          5: {
            id: 5,
            title: "Flow 1",
            player_ids: [1],
            length: 512,
            master_id: 200,
            staves: {
              by_id: {
                100: {
                  id: 100,
                  lines: [1, 1, 1, 1, 1],
                  master_id: 200,
                  track_ids: [300],
                },
              },
            },
          },
        },
      },
      tracks: {
        by_id: {
          300: {
            id: 300,
            entries: { by_tick: {}, by_id: {} },
          },
        },
      },
    } as never,
  });

describe("useBottomPanelState", () => {
  it("returns instrumentOptions derived from players → instruments", () => {
    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: buildState() }),
    });
    expect(result.current.instrumentOptions).toHaveLength(1);
    expect(result.current.instrumentOptions[0].label).toBe("Piano");
    expect(result.current.instrumentOptions[0].value).toBe(10);
  });

  it("returns first flowId when none is selected", () => {
    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: buildState() }),
    });
    expect(result.current.flowId).toBe(5);
  });

  it("returns first instrumentId when none is selected", () => {
    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: buildState() }),
    });
    expect(result.current.instrumentId).toBe(10);
  });

  it("returns first trackId when none is selected", () => {
    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: buildState() }),
    });
    expect(result.current.trackId).toBe(300);
  });

  it("returns tool from sequencer state", () => {
    const state = buildState();
    state.ui.sequencer.tool = Tool.Write;
    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current.tool).toBe(Tool.Write);
  });

  it("returns snapValue from sequencer state", () => {
    const state = buildState();
    state.ui.sequencer.snap = NoteDuration.Sixteenth;
    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current.snapValue).toBe(NoteDuration.Sixteenth);
  });

  it("returns null flowId when no flows exist", () => {
    const state = createAppState(); // no flows in initialAppState
    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current.flowId).toBeNull();
  });

  it("returns null instrumentId when no instruments exist", () => {
    const state = createAppState();
    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current.instrumentId).toBeNull();
  });

  it("returns [] tracks when flow id references a non-existent flow", () => {
    // flows.order has an id, but by_id is empty → !flow branch (line 71)
    const state = createAppState({
      score: {
        players: {
          order: [1],
          by_id: {
            1: { id: 1, player_type: PlayerType.Solo, instrument_ids: [10] },
          },
        },
        instruments: {
          by_id: {
            10: {
              id: 10,
              definition_id: "piano",
              instrument_type: 0,
              long_name: "Piano",
              short_name: "Pno.",
              stave_ids: [100],
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
          order: [5],
          by_id: {}, // ← flow 5 missing → !flow branch
        },
        tracks: { by_id: {} },
      } as never,
    });

    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current.tracks).toEqual([]);
    expect(result.current.trackId).toBeNull();
  });

  it("returns [] tracks when stave id is missing from flow staves", () => {
    // instrument.stave_ids = [999] but flow.staves.by_id has no entry for 999
    const state = createAppState({
      score: {
        players: {
          order: [1],
          by_id: {
            1: { id: 1, player_type: PlayerType.Solo, instrument_ids: [10] },
          },
        },
        instruments: {
          by_id: {
            10: {
              id: 10,
              definition_id: "piano",
              instrument_type: 0,
              long_name: "Piano",
              short_name: "Pno.",
              stave_ids: [999], // ← stave 999, not in flow staves
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
          order: [5],
          by_id: {
            5: {
              id: 5,
              title: "Flow 1",
              player_ids: [1],
              length: 512,
              master_id: 200,
              staves: { by_id: {} }, // ← stave 999 missing
            },
          },
        },
        tracks: { by_id: {} },
      } as never,
    });

    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current.tracks).toEqual([]);
  });

  it("returns the explicitly-selected track when it exists in the tracks list", () => {
    const state = buildState();
    state.ui.sequencer = { ...state.ui.sequencer, track: 300 } as never;

    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current.trackId).toBe(300);
  });

  it("returns the explicitly-selected instrumentId when it exists", () => {
    const state = buildState();
    state.ui.sequencer = { ...state.ui.sequencer, instrument: 10 } as never;

    const { result } = renderHook(() => useBottomPanelState(), {
      wrapper: createWrapper({ appState: state }),
    });
    expect(result.current.instrumentId).toBe(10);
  });
});
