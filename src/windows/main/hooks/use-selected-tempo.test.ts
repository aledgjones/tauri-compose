import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { createAppState, createWrapper } from "../../../services/test-utils";
import { NoteDuration } from "../../../services/types";
import { useSelectedTempo } from "./use-selected-tempo";

const FLOW_ID = 9;
const SECOND_FLOW_ID = 19;
const MASTER_ID = 10;
const SECOND_MASTER_ID = 20;
const TEMPO_ID = 44;
const SECOND_TEMPO_ID = 54;

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
              by_tick: { 64: [TEMPO_ID] },
              by_id: {
                [TEMPO_ID]: {
                  id: TEMPO_ID,
                  tick: 64,
                  type: "Tempo",
                  text: "Allegro",
                  beat_type: NoteDuration.Quarter,
                  dots: 0,
                  bpm: 120,
                  hidden: false,
                  show_text: true,
                  show_metronome: true,
                  parenthesized: false,
                },
              },
            },
          },
          [SECOND_MASTER_ID]: {
            id: SECOND_MASTER_ID,
            entries: {
              by_tick: { 96: [SECOND_TEMPO_ID] },
              by_id: {
                [SECOND_TEMPO_ID]: {
                  id: SECOND_TEMPO_ID,
                  tick: 96,
                  type: "Tempo",
                  text: "Largo",
                  beat_type: NoteDuration.Quarter,
                  dots: 0,
                  bpm: 56,
                  hidden: false,
                  show_text: true,
                  show_metronome: true,
                  parenthesized: false,
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
          type: "Tempo",
          flow_id: FLOW_ID,
          tempo_id: TEMPO_ID,
          tick: 64,
        },
      },
    } as never,
  });

describe("useSelectedTempo", () => {
  it("returns the selected tempo with flow context", () => {
    const state = buildState();

    const { result } = renderHook(() => useSelectedTempo(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: FLOW_ID,
      tempo: expect.objectContaining({ id: TEMPO_ID, text: "Allegro" }),
    });
  });

  it("returns null when the selected flow is missing", () => {
    const state = buildState();
    delete (state.score.flows.by_id as Record<number, unknown>)[FLOW_ID];

    const { result } = renderHook(() => useSelectedTempo(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null when selection does not resolve to a tempo entry", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "Tempo",
      flow_id: FLOW_ID,
      tempo_id: 999,
      tick: 64,
    };

    const { result } = renderHook(() => useSelectedTempo(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("resolves tempo data from the selected visible flow when multiple flows are present", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "Tempo",
      flow_id: SECOND_FLOW_ID,
      tempo_id: SECOND_TEMPO_ID,
      tick: 96,
    };

    const { result } = renderHook(() => useSelectedTempo(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: SECOND_FLOW_ID,
      tempo: expect.objectContaining({ id: SECOND_TEMPO_ID, text: "Largo" }),
    });
  });
});
