import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { createAppState, createWrapper } from "../../../services/test-utils";
import { Accidental, ClefDrawType } from "../../../services/types";
import { useSelectedClef } from "./use-selected-clef";

const FLOW_ID = 9;
const SECOND_FLOW_ID = 19;
const STAVE_ID = 11;
const SECOND_STAVE_ID = 21;
const MASTER_ID = 12;
const SECOND_MASTER_ID = 22;
const CLEF_ID = 44;
const SECOND_CLEF_ID = 54;

const buildState = () =>
  createAppState({
    score: {
      flows: {
        order: [FLOW_ID, SECOND_FLOW_ID],
        by_id: {
          [FLOW_ID]: {
            id: FLOW_ID,
            title: "Flow",
            player_ids: [],
            length: 128,
            master_id: 99,
            staves: {
              by_id: {
                [STAVE_ID]: {
                  id: STAVE_ID,
                  lines: [1, 1, 1, 1, 1],
                  master_id: MASTER_ID,
                  track_ids: [],
                },
              },
            },
          },
          [SECOND_FLOW_ID]: {
            id: SECOND_FLOW_ID,
            title: "Flow 2",
            player_ids: [],
            length: 128,
            master_id: 199,
            staves: {
              by_id: {
                [SECOND_STAVE_ID]: {
                  id: SECOND_STAVE_ID,
                  lines: [1, 1, 1, 1, 1],
                  master_id: SECOND_MASTER_ID,
                  track_ids: [],
                },
              },
            },
          },
        },
      },
      tracks: {
        by_id: {
          [MASTER_ID]: {
            id: MASTER_ID,
            entries: {
              by_tick: { 64: [CLEF_ID] },
              by_id: {
                [CLEF_ID]: {
                  id: CLEF_ID,
                  tick: 64,
                  type: "Clef",
                  draw_as: ClefDrawType.G,
                  pitch: { int: 67, accidental: Accidental.Natural },
                  offset: 2,
                },
              },
            },
          },
          [SECOND_MASTER_ID]: {
            id: SECOND_MASTER_ID,
            entries: {
              by_tick: { 96: [SECOND_CLEF_ID] },
              by_id: {
                [SECOND_CLEF_ID]: {
                  id: SECOND_CLEF_ID,
                  tick: 96,
                  type: "Clef",
                  draw_as: ClefDrawType.F,
                  pitch: { int: 53, accidental: Accidental.Natural },
                  offset: -2,
                },
              },
            },
          },
        },
      },
    } as never,
    ui: {
      sequencer: {
        selection: {
          type: "Clef",
          flow_id: FLOW_ID,
          stave_id: STAVE_ID,
          clef_id: CLEF_ID,
          tick: 64,
        },
      },
    } as never,
  });

describe("useSelectedClef", () => {
  it("returns the selected clef with stave context", () => {
    const state = buildState();

    const { result } = renderHook(() => useSelectedClef(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: FLOW_ID,
      stave_id: STAVE_ID,
      clef: expect.objectContaining({ id: CLEF_ID, offset: 2 }),
    });
  });

  it("returns null when the selected tick no longer matches the clef entry", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "Clef",
      flow_id: FLOW_ID,
      stave_id: STAVE_ID,
      clef_id: CLEF_ID,
      tick: 32,
    };

    const { result } = renderHook(() => useSelectedClef(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("resolves the selected clef from a later visible flow and stave", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "Clef",
      flow_id: SECOND_FLOW_ID,
      stave_id: SECOND_STAVE_ID,
      clef_id: SECOND_CLEF_ID,
      tick: 96,
    };

    const { result } = renderHook(() => useSelectedClef(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: SECOND_FLOW_ID,
      stave_id: SECOND_STAVE_ID,
      clef: expect.objectContaining({ id: SECOND_CLEF_ID, offset: -2 }),
    });
  });
});
