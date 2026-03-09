import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { createAppState, createWrapper } from "../../../services/test-utils";
import { NoteDuration, TimeSignatureDrawType } from "../../../services/types";
import { useSelectedTimeSignature } from "./use-selected-time-signature";

const FLOW_ID = 9;
const MASTER_ID = 10;
const TIME_SIGNATURE_ID = 44;

const buildState = () =>
  createAppState({
    score: {
      flows: {
        order: [FLOW_ID],
        by_id: {
          [FLOW_ID]: {
            id: FLOW_ID,
            title: "Flow",
            player_ids: [],
            length: 128,
            master_id: MASTER_ID,
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
});
