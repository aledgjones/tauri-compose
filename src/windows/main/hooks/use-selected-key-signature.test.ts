import { describe, expect, it } from "vitest";
import { renderHook } from "@testing-library/react";
import { createAppState, createWrapper } from "../../../services/test-utils";
import { KeySignatureMode } from "../../../services/types";
import { useSelectedKeySignature } from "./use-selected-key-signature";

const FLOW_ID = 9;
const MASTER_ID = 10;
const KEY_SIGNATURE_ID = 44;

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
});
