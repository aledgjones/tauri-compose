import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { createAppState, createWrapper } from "../../../services/test-utils";
import { NoteDuration, TimeSignatureDrawType } from "../../../services/types";
import { useSelectedTimeSignature } from "./use-selected-time-signature";

const FLOW_ID = 9;
const SECOND_FLOW_ID = 19;
const MASTER_ID = 10;
const SECOND_MASTER_ID = 20;
const TIME_SIGNATURE_ID = 44;
const SECOND_TIME_SIGNATURE_ID = 54;

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
            master_id: MASTER_ID,
            staves: { by_id: {} },
          },
          [SECOND_FLOW_ID]: {
            id: SECOND_FLOW_ID,
            title: "Flow 2",
            player_ids: [],
            length: 128,
            master_id: SECOND_MASTER_ID,
            staves: { by_id: {} },
          },
        },
      },
      tracks: {
        by_id: {
          [MASTER_ID]: {
            id: MASTER_ID,
            entries: {
              by_tick: { 64: [TIME_SIGNATURE_ID] },
              by_id: {
                [TIME_SIGNATURE_ID]: {
                  id: TIME_SIGNATURE_ID,
                  tick: 64,
                  type: "TimeSignature",
                  beats: 4,
                  beat_type: NoteDuration.Quarter,
                  draw_type: TimeSignatureDrawType.Regular,
                },
              },
            },
          },
          [SECOND_MASTER_ID]: {
            id: SECOND_MASTER_ID,
            entries: {
              by_tick: { 96: [SECOND_TIME_SIGNATURE_ID] },
              by_id: {
                [SECOND_TIME_SIGNATURE_ID]: {
                  id: SECOND_TIME_SIGNATURE_ID,
                  tick: 96,
                  type: "TimeSignature",
                  beats: 3,
                  beat_type: NoteDuration.Quarter,
                  draw_type: TimeSignatureDrawType.Regular,
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
          type: "TimeSignature",
          flow_id: FLOW_ID,
          time_signature_id: TIME_SIGNATURE_ID,
          tick: 64,
        },
      },
    } as never,
  });

describe("useSelectedTimeSignature", () => {
  it("returns the selected time signature with flow context", () => {
    const state = buildState();

    const { result } = renderHook(() => useSelectedTimeSignature(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: FLOW_ID,
      time_signature: expect.objectContaining({
        id: TIME_SIGNATURE_ID,
        draw_type: TimeSignatureDrawType.Regular,
      }),
    });
  });

  it("returns null when backing data is missing", () => {
    const state = buildState();
    delete (state.score.tracks.by_id as Record<number, unknown>)[MASTER_ID];

    const { result } = renderHook(() => useSelectedTimeSignature(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("resolves the selected time signature from a later visible flow", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "TimeSignature",
      flow_id: SECOND_FLOW_ID,
      time_signature_id: SECOND_TIME_SIGNATURE_ID,
      tick: 96,
    };

    const { result } = renderHook(() => useSelectedTimeSignature(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: SECOND_FLOW_ID,
      time_signature: expect.objectContaining({
        id: SECOND_TIME_SIGNATURE_ID,
        beats: 3,
      }),
    });
  });
});
