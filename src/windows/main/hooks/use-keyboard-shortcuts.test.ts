import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useKeyboardShortcuts } from "./use-keyboard-shortcuts";
import { createWrapper, createAppState } from "../../../services/test-utils";
import { Accidental, Articulation, PlayerType } from "../../../services/types";

// ---------------------------------------------------------------------------
// Mock the actions module
// ---------------------------------------------------------------------------
vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      tone: {
        shiftPitchByStep: vi.fn().mockResolvedValue(undefined),
        shiftPitchByOctave: vi.fn().mockResolvedValue(undefined),
        remove: vi.fn().mockResolvedValue(undefined),
        setAccidental: vi.fn().mockResolvedValue(undefined),
        setArticulation: vi.fn().mockResolvedValue(undefined),
      },
    },
    ui: {
      sequencer: {
        clearSelection: vi.fn().mockResolvedValue(undefined),
      },
    },
    sampler: {
      audition: vi.fn(),
    },
  },
}));

import { actions } from "../../../services/actions";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------
const FLOW_ID = 5;
const INSTRUMENT_ID = 10;
const STAVE_ID = 100;
const TRACK_ID = 300;
const TONE_ID = 999;

const buildStateWithToneSelection = () => {
  return createAppState({
    score: {
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
            assigned_plugin_id: null, // no audition
            volume: 100,
            pan: 0,
            mute: false,
            solo: false,
          },
        },
      },
      tracks: {
        by_id: {
          [TRACK_ID]: {
            id: TRACK_ID,
            entries: {
              by_tick: { [64]: [TONE_ID] },
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
    } as never,
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
};

const dispatchKey = (key: string, options?: KeyboardEventInit) => {
  act(() => {
    window.dispatchEvent(new KeyboardEvent("keydown", { key, ...options }));
  });
};

describe("useKeyboardShortcuts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("mounts without errors when no selection", () => {
    expect(() => {
      renderHook(() => useKeyboardShortcuts(), {
        wrapper: createWrapper({ appState: createAppState() }),
      });
    }).not.toThrow();
  });

  it("dispatches shiftPitchByStep(+1) on ArrowUp+Option with tone selection", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowUp", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      FLOW_ID,
      TRACK_ID,
      TONE_ID,
      1,
    );
  });

  it("dispatches shiftPitchByStep(-1) on ArrowDown+Option", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "ArrowDown", altKey: true }),
      );
    });

    expect(actions.score.tone.shiftPitchByStep).toHaveBeenCalledWith(
      FLOW_ID,
      TRACK_ID,
      TONE_ID,
      -1,
    );
  });

  it("dispatches shiftPitchByOctave(+1) on ArrowUp+Cmd+Option", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });

    await act(async () => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "ArrowUp",
          metaKey: true,
          altKey: true,
        }),
      );
    });

    expect(actions.score.tone.shiftPitchByOctave).toHaveBeenCalledWith(
      TRACK_ID,
      TONE_ID,
      1,
    );
  });

  it("dispatches remove on Backspace", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Backspace" }));
    });

    expect(actions.score.tone.remove).toHaveBeenCalledWith(TRACK_ID, TONE_ID);
  });

  it("dispatches clearSelection on Escape", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    });

    expect(actions.ui.sequencer.clearSelection).toHaveBeenCalledOnce();
  });

  it("dispatches setArticulation(Staccato) on ']'", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });

    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "]" }));
    });

    expect(actions.score.tone.setArticulation).toHaveBeenCalledWith(
      TRACK_ID,
      TONE_ID,
      Articulation.Staccato,
    );
  });

  it("does not fire any action for unknown key", async () => {
    renderHook(() => useKeyboardShortcuts(), {
      wrapper: createWrapper({ appState: buildStateWithToneSelection() }),
    });

    dispatchKey("F1");

    expect(actions.score.tone.shiftPitchByStep).not.toHaveBeenCalled();
    expect(actions.score.tone.remove).not.toHaveBeenCalled();
    expect(actions.ui.sequencer.clearSelection).not.toHaveBeenCalled();
  });
});
