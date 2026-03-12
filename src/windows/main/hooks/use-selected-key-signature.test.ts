import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { createAppState, createWrapper } from "../../../services/test-utils";
import { KeySignatureMode } from "../../../services/types";
import { useSelectedKeySignature } from "./use-selected-key-signature";

const FLOW_ID = 9;
const SECOND_FLOW_ID = 19;
const MASTER_ID = 10;
const SECOND_MASTER_ID = 20;
const KEY_SIGNATURE_ID = 44;
const SECOND_KEY_SIGNATURE_ID = 54;

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
              by_tick: { 64: [KEY_SIGNATURE_ID] },
              by_id: {
                [KEY_SIGNATURE_ID]: {
                  id: KEY_SIGNATURE_ID,
                  tick: 64,
                  type: "KeySignature",
                  mode: KeySignatureMode.Major,
                  offset: 2,
                },
              },
            },
          },
          [SECOND_MASTER_ID]: {
            id: SECOND_MASTER_ID,
            entries: {
              by_tick: { 96: [SECOND_KEY_SIGNATURE_ID] },
              by_id: {
                [SECOND_KEY_SIGNATURE_ID]: {
                  id: SECOND_KEY_SIGNATURE_ID,
                  tick: 96,
                  type: "KeySignature",
                  mode: KeySignatureMode.Minor,
                  offset: -1,
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
          type: "KeySignature",
          flow_id: FLOW_ID,
          key_signature_id: KEY_SIGNATURE_ID,
          tick: 64,
        },
      },
    } as never,
  });

describe("useSelectedKeySignature", () => {
  it("returns the selected key signature with flow context", () => {
    const state = buildState();

    const { result } = renderHook(() => useSelectedKeySignature(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: FLOW_ID,
      key_signature: expect.objectContaining({
        id: KEY_SIGNATURE_ID,
        offset: 2,
      }),
    });
  });

  it("returns null when the selection tick no longer matches", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "KeySignature",
      flow_id: FLOW_ID,
      key_signature_id: KEY_SIGNATURE_ID,
      tick: 32,
    };

    const { result } = renderHook(() => useSelectedKeySignature(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toBeNull();
  });

  it("resolves key signature data from the selected visible flow when multiple flows are rendered", () => {
    const state = buildState();
    state.ui.sequencer.selection = {
      type: "KeySignature",
      flow_id: SECOND_FLOW_ID,
      key_signature_id: SECOND_KEY_SIGNATURE_ID,
      tick: 96,
    };

    const { result } = renderHook(() => useSelectedKeySignature(), {
      wrapper: createWrapper({ appState: state }),
    });

    expect(result.current).toEqual({
      flow_id: SECOND_FLOW_ID,
      key_signature: expect.objectContaining({
        id: SECOND_KEY_SIGNATURE_ID,
        offset: -1,
      }),
    });
  });
});
