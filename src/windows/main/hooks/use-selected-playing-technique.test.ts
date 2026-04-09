import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { createAppState, createWrapper } from "../../../services/test-utils";
import { PlayingTechniqueKind } from "../../../services/types";
import { useSelectedPlayingTechnique } from "./use-selected-playing-technique";

const FLOW_ID = 9;
const STAVE_ID = 11;
const STAVE_MASTER_ID = 12;
const PLAYING_TECHNIQUE_ID = 44;

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
            master_id: 99,
            staves: {
              by_id: {
                [STAVE_ID]: {
                  id: STAVE_ID,
                  lines: [1, 1, 1, 1, 1],
                  master_id: STAVE_MASTER_ID,
                  track_ids: [],
                },
              },
            },
          },
        },
      },
      tracks: {
        by_id: {
          [STAVE_MASTER_ID]: {
            id: STAVE_MASTER_ID,
            entries: {
              by_tick: { 64: [PLAYING_TECHNIQUE_ID] },
              by_id: {
                [PLAYING_TECHNIQUE_ID]: {
                  id: PLAYING_TECHNIQUE_ID,
                  tick: 64,
                  type: "PlayingTechnique",
                  hidden: false,
                  kind: PlayingTechniqueKind.Pizz,
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
          type: "PlayingTechnique",
          flow_id: FLOW_ID,
          stave_id: STAVE_ID,
          playing_technique_id: PLAYING_TECHNIQUE_ID,
          tick: 64,
        },
      },
    } as never,
  });

describe("useSelectedPlayingTechnique", () => {
  it("returns the selected playing technique with stave context", () => {
    const state = buildState();

    const { result } = renderHook(() => useSelectedPlayingTechnique(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: FLOW_ID,
      stave_id: STAVE_ID,
      playing_technique: expect.objectContaining({
        id: PLAYING_TECHNIQUE_ID,
        kind: PlayingTechniqueKind.Pizz,
      }),
    });
  });

  it("returns null for a non-playing-technique selection", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "Clef",
      flow_id: FLOW_ID,
      stave_id: STAVE_ID,
      clef_id: 100,
      tick: 64,
    };

    const { result } = renderHook(() => useSelectedPlayingTechnique(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null when the selected flow is missing", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "PlayingTechnique",
      flow_id: 404,
      stave_id: STAVE_ID,
      playing_technique_id: PLAYING_TECHNIQUE_ID,
      tick: 64,
    };

    const { result } = renderHook(() => useSelectedPlayingTechnique(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null when the selected stave is missing", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "PlayingTechnique",
      flow_id: FLOW_ID,
      stave_id: 404,
      playing_technique_id: PLAYING_TECHNIQUE_ID,
      tick: 64,
    };

    const { result } = renderHook(() => useSelectedPlayingTechnique(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null when the stave master track is missing", () => {
    const state = buildState();
    delete state.score.tracks.by_id[STAVE_MASTER_ID];

    const { result } = renderHook(() => useSelectedPlayingTechnique(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null when the playing technique entry is missing", () => {
    const state = buildState();
    delete state.score.tracks.by_id[STAVE_MASTER_ID].entries.by_id[
      PLAYING_TECHNIQUE_ID
    ];

    const { result } = renderHook(() => useSelectedPlayingTechnique(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null when the resolved entry has the wrong type", () => {
    const state = buildState();
    state.score.tracks.by_id[STAVE_MASTER_ID].entries.by_id[
      PLAYING_TECHNIQUE_ID
    ] = {
      id: PLAYING_TECHNIQUE_ID,
      tick: 64,
      type: "Clef",
      hidden: false,
      draw_as: 1,
      pitch: { int: 67, accidental: 0 },
      offset: 2,
    } as never;

    const { result } = renderHook(() => useSelectedPlayingTechnique(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("returns null when the selected tick no longer matches the entry", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "PlayingTechnique",
      flow_id: FLOW_ID,
      stave_id: STAVE_ID,
      playing_technique_id: PLAYING_TECHNIQUE_ID,
      tick: 32,
    };

    const { result } = renderHook(() => useSelectedPlayingTechnique(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });
});
